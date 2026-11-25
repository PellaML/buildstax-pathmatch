#[derive(Debug, Clone)]
enum Token {
    Byte(u8),
    One,
    Star,
    Deep,
    Dirs,
    Class(Vec<(u8, u8)>, bool),
}

/// A case-sensitive byte glob. `?` and classes consume one UTF-8 byte.
#[derive(Debug, Clone)]
pub struct Glob(Vec<Token>, Vec<u8>);

/// Invalid escape or character class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternError;

impl std::fmt::Display for PatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid glob escape or character class")
    }
}
impl std::error::Error for PatternError {}

impl Glob {
    pub fn new(pattern: &str) -> Result<Self, PatternError> {
        let p = pattern.as_bytes();
        let mut out = Vec::new();
        let mut i = 0;
        while i < p.len() {
            let b = p[i];
            i += 1;
            out.push(match b {
                b'\\' => {
                    let b = *p.get(i).ok_or(PatternError)?;
                    i += 1;
                    Token::Byte(b)
                }
                b'?' => Token::One,
                b'*' => {
                    let start = i - 1;
                    while p.get(i) == Some(&b'*') {
                        i += 1;
                    }
                    if i - start == 2 && (start == 0 || p[start - 1] == b'/') {
                        match p.get(i) {
                            Some(b'/') => {
                                i += 1;
                                Token::Dirs
                            }
                            None => Token::Deep,
                            _ => Token::Star,
                        }
                    } else {
                        Token::Star
                    }
                }
                b'[' => {
                    let neg = matches!(p.get(i), Some(b'!' | b'^'));
                    i += usize::from(neg);
                    let mut ranges = Vec::new();
                    while p.get(i).is_some() && (p[i] != b']' || ranges.is_empty()) {
                        let lo = class_byte(p, &mut i)?;
                        let hi = if p.get(i) == Some(&b'-')
                            && p.get(i + 1).is_some_and(|&x| x != b']')
                        {
                            i += 1;
                            class_byte(p, &mut i)?
                        } else {
                            lo
                        };
                        if hi < lo {
                            return Err(PatternError);
                        }
                        ranges.push((lo, hi));
                    }
                    if p.get(i) != Some(&b']') {
                        return Err(PatternError);
                    }
                    i += 1;
                    Token::Class(ranges, neg)
                }
                b => Token::Byte(b),
            });
        }
        let required = out
            .split(|t| !matches!(t, Token::Byte(_)))
            .max_by_key(|run| run.len())
            .unwrap_or_default()
            .iter()
            .filter_map(|t| {
                if let Token::Byte(b) = t {
                    Some(*b)
                } else {
                    None
                }
            })
            .collect();
        Ok(Self(out, required))
    }

    /// Match an entire slash-separated path.
    pub fn is_match(&self, path: &str) -> bool {
        let mut p = path.as_bytes();
        let mut tokens = self.0.as_slice();
        while let Some(Token::Byte(b)) = tokens.first() {
            if p.first() != Some(b) {
                return false;
            }
            p = &p[1..];
            tokens = &tokens[1..];
        }
        while let Some(Token::Byte(b)) = tokens.last() {
            if p.last() != Some(b) {
                return false;
            }
            p = &p[..p.len() - 1];
            tokens = &tokens[..tokens.len() - 1];
        }
        match tokens {
            [] => return p.is_empty(),
            [Token::Star] => return !p.contains(&b'/'),
            [Token::Deep] => return true,
            _ => {}
        }
        // Every literal run must occur somewhere in the original candidate.
        if self.1.len() >= 3 && !path.as_bytes().windows(self.1.len()).any(|w| w == self.1) {
            return false;
        }
        let mut row = vec![false; p.len() + 1];
        let mut next = row.clone();
        row[0] = true;
        for t in tokens {
            next[0] = row[0] && matches!(t, Token::Star | Token::Dirs | Token::Deep);
            let mut reachable = row[0];
            for (j, &b) in p.iter().enumerate() {
                next[j + 1] = match t {
                    Token::Byte(x) => row[j] && b == *x,
                    Token::One => row[j] && b != b'/',
                    Token::Class(r, n) => {
                        row[j] && b != b'/' && (r.iter().any(|&(lo, hi)| lo <= b && b <= hi) != *n)
                    }
                    Token::Star => row[j + 1] || (b != b'/' && next[j]),
                    Token::Deep => row[j + 1] || next[j],
                    Token::Dirs => row[j + 1] || (reachable && b == b'/'),
                };
                reachable |= row[j + 1];
            }
            std::mem::swap(&mut row, &mut next);
        }
        row[p.len()]
    }
}

fn class_byte(p: &[u8], i: &mut usize) -> Result<u8, PatternError> {
    if p.get(*i) == Some(&b'\\') {
        *i += 1;
    }
    let b = *p.get(*i).ok_or(PatternError)?;
    *i += 1;
    Ok(b)
}
