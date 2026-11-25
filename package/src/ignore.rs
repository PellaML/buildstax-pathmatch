use crate::{Glob, PatternError};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Unmatched,
    Ignore { line: usize },
    Include { line: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleError {
    pub line: usize,
    pub source: PatternError,
}
impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.source)
    }
}
impl std::error::Error for RuleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug, Clone)]
struct Rule {
    line: usize,
    include: bool,
    dir: bool,
    rooted: bool,
    glob: Glob,
}

/// Rules from a single root ignore file. Input paths must be normalized,
/// root-relative UTF-8 with `/` separators, without a trailing slash.
#[derive(Debug, Default, Clone)]
pub struct Ignore {
    literals: HashMap<String, [Option<usize>; 2]>,
    extensions: HashMap<String, [Option<usize>; 2]>,
    rules: Vec<Rule>,
    fallback: Vec<usize>,
}

impl Ignore {
    pub fn new(text: &str) -> Result<Self, RuleError> {
        Self::compile(text, true)
    }

    /// Linear reference strategy for performance comparisons.
    #[doc(hidden)]
    pub fn new_linear(text: &str) -> Result<Self, RuleError> {
        Self::compile(text, false)
    }

    fn compile(text: &str, index: bool) -> Result<Self, RuleError> {
        let mut result = Self::default();
        for (i, raw) in text.trim_start_matches('\u{feff}').lines().enumerate() {
            let mut s = raw;
            while let Some(rest) = s.strip_suffix(' ') {
                if rest.bytes().rev().take_while(|&b| b == b'\\').count() % 2 == 1 {
                    break;
                }
                s = rest;
            }
            if s.is_empty() || s.starts_with('#') {
                continue;
            }
            let include = s.starts_with('!');
            if include {
                s = &s[1..];
            }
            let anchored = s.starts_with('/');
            if anchored {
                s = &s[1..];
            }
            let dir = s.ends_with('/')
                && s.bytes().rev().skip(1).take_while(|&b| b == b'\\').count() % 2 == 0;
            if dir {
                s = &s[..s.len() - 1];
            }
            if s.is_empty() {
                continue;
            }
            let rooted = anchored || s.contains('/');
            let line = i + 1;
            let glob = Glob::new(s).map_err(|source| RuleError { line, source })?;
            let id = result.rules.len();
            result.rules.push(Rule {
                line,
                include,
                dir,
                rooted,
                glob,
            });
            let plain = |x: &str| !x.bytes().any(|b| matches!(b, b'*' | b'?' | b'[' | b'\\'));
            let slot = if index && !rooted && plain(s) {
                Some(result.literals.entry(s.into()).or_default())
            } else if let Some(ext) = s
                .strip_prefix("*.")
                .filter(|x| index && !rooted && plain(x) && !x.contains('.'))
            {
                Some(result.extensions.entry(ext.into()).or_default())
            } else {
                None
            };
            if let Some(slot) = slot {
                slot[1] = Some(id);
                if !dir {
                    slot[0] = Some(id);
                }
            } else {
                result.fallback.push(id);
            }
        }
        Ok(result)
    }

    /// Evaluate ancestors first: a child cannot re-include itself beneath an
    /// ignored directory. The decision identifies the winning source line.
    pub fn decide(&self, path: &str, is_dir: bool) -> Decision {
        let mut start = 0;
        for (end, _) in path.match_indices('/') {
            let decision = self.direct(&path[..end], &path[start..end], true);
            if matches!(decision, Decision::Ignore { .. }) {
                return decision;
            }
            start = end + 1;
        }
        self.direct(path, &path[start..], is_dir)
    }

    fn direct(&self, path: &str, name: &str, dir: bool) -> Decision {
        let slot = usize::from(dir);
        let mut best = self.literals.get(name).and_then(|x| x[slot]);
        if let Some((_, ext)) = name.rsplit_once('.') {
            best = best.max(self.extensions.get(ext).and_then(|x| x[slot]));
        }
        for &id in self.fallback.iter().rev() {
            if best.is_some_and(|b| id <= b) {
                break;
            }
            let r = &self.rules[id];
            if (!r.dir || dir) && r.glob.is_match(if r.rooted { path } else { name }) {
                best = Some(id);
                break;
            }
        }
        match best.map(|id| &self.rules[id]) {
            None => Decision::Unmatched,
            Some(r) if r.include => Decision::Include { line: r.line },
            Some(r) => Decision::Ignore { line: r.line },
        }
    }
}
