use buildstax_pathmatch::{Decision as D, Glob, Ignore, PatternError, RuleError};

#[test]
fn glob_contract() -> Result<(), PatternError> {
    for (pattern, path, expected) in [
        ("*.rs", "x/y.rs", false),
        ("*.rs", "main.rs", true),
        ("a?c", "a/c", false),
        ("a?c", "abc", true),
        ("x/**/z", "x/z", true),
        ("x/**/z", "x/a/b/z", true),
        ("x/**/z", "x/az", false),
        ("x/**", "x/a/b", true),
        ("x/**", "x", false),
        ("[!a-c]", "d", true),
        ("[!a-c]", "b", false),
        ("[!a-c]", "/", false),
        (r"a\*b", "a*b", true),
        ("ab*cd*ef", "abxcdyef", true),
    ] {
        assert_eq!(
            Glob::new(pattern)?.is_match(path),
            expected,
            "{pattern} vs {path}"
        );
    }
    for p in ["[z-a]", "[", "x\\"] {
        assert!(Glob::new(p).is_err());
    }
    Ok(())
}

#[test]
fn ignore_contract() -> Result<(), RuleError> {
    let text = "# comment\r\n*.log\n!keep.log\ncache/\n!cache/keep.log\n/root\n\\#literal\ntrail\\ \nzone/*\n!zone/good\nname\n!name/\n";
    let indexed = Ignore::new(text)?;
    let linear = Ignore::new_linear(text)?;
    for (path, dir, expected) in [
        ("x/a.log", false, D::Ignore { line: 2 }),
        ("x/keep.log", false, D::Include { line: 3 }),
        ("cache/keep.log", false, D::Ignore { line: 4 }),
        ("cache", false, D::Unmatched),
        ("x/root", false, D::Unmatched),
        ("root", true, D::Ignore { line: 6 }),
        ("#literal", false, D::Ignore { line: 7 }),
        ("trail ", false, D::Ignore { line: 8 }),
        ("zone/good", false, D::Include { line: 10 }),
        ("zone/bad/sub", false, D::Ignore { line: 9 }),
        ("name", false, D::Ignore { line: 11 }),
        ("name", true, D::Include { line: 12 }),
    ] {
        assert_eq!(indexed.decide(path, dir), expected, "{path}");
        assert_eq!(linear.decide(path, dir), expected, "{path}");
    }
    assert_eq!(Ignore::new("# comment\n[").err().map(|x| x.line), Some(2));
    Ok(())
}
