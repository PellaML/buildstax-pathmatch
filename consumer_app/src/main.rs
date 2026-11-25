use buildstax_pathmatch::{Decision, Ignore, RuleError};

fn main() -> Result<(), RuleError> {
    let rules = Ignore::new("out/\n*.snap\n!golden.snap\n")?;
    let shipped: Vec<_> = [
        "src/lib.rs",
        "out/debug/app",
        "tests/new.snap",
        "tests/golden.snap",
    ]
    .into_iter()
    .filter(|path| !matches!(rules.decide(path, false), Decision::Ignore { .. }))
    .collect();
    assert_eq!(shipped, ["src/lib.rs", "tests/golden.snap"]);
    println!("shipping {shipped:?}");
    Ok(())
}
