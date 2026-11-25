# BuildStax Pathmatch

An Apache-2.0 Rust library for full-path glob matching and explaining decisions from one root `.gitignore` file. The library has no runtime dependencies.

```rust
use buildstax_pathmatch::{Decision, Glob, Ignore};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert!(Glob::new("src/**/*.rs")?.is_match("src/lib.rs"));
    let rules = Ignore::new("target/\n*.log\n!keep.log\n")?;
    assert_eq!(rules.decide("logs/keep.log", false), Decision::Include { line: 3 });
    Ok(())
}
```

Pass normalized, root-relative UTF-8 paths with `/` separators and no trailing slash; the boolean tells the matcher whether the leaf is a directory. On Windows, normalize project-relative paths before calling. `Ignore` matches ancestors before the leaf and returns the blocking ancestor's rule line when applicable. `Unmatched` and `Include` may both be retained by a filter; `Include` denotes an explicit negation. Malformed rules return the first error with a 1-based line number.

Supports `*`, `?`, whole-component `**`, byte ranges and negated classes, escapes, directory-only rules, anchoring, and last-match precedence. This is not a full Git implementation: no traversal, nested rule discovery, byte paths, POSIX named classes, case folding, or normalization. Wildcards and classes operate on bytes. Brace alternation is not supported.

Run `cargo run -p pathmatch-demo -- demo/example.ignore`, then enter `logs/app.log`, `logs/keep.log`, or `target/` on stdin. A trailing slash in the demo marks directory input.

See [the decision journal](DECISION_JOURNAL.md) for benchmark results and limits, [automation](AUTOMATION.md) for reproduction, and [the pilot plan](COMMERCIAL_PLAN.md) for the paid offer.
