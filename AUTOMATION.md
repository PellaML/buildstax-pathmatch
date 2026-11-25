# Verification and release

```text
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --all-targets -- -D warnings
python referee.py
cargo run --locked --release -p pathmatch-bench
cargo run --locked --manifest-path consumer_app/Cargo.toml
cargo clippy --locked --manifest-path consumer_app/Cargo.toml --all-targets -- -D warnings
```

The referee copies the package to temporary storage, runs the unchanged integration target, and only counts mutants that compile and fail tests. The benchmark checks ignore/non-ignore parity before timing. It uses precompiled rules, 96 deterministic paths per workload, 15 samples of 12 passes, alternating candidate order, and adjacent incumbent A/A runs. Report medians in ns/path and median absolute paired relative difference in bps. Keep regressions visible. This is not a cold-start, memory, standalone glob, or holdout benchmark.
