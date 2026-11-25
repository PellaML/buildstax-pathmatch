# Decision journal

## What the incumbents protect

- [ignore 0.4.35](https://crates.io/crates/ignore/0.4.35) couples ignore matching to walking and supports broader ingestion than a single UTF-8 rule file. Keep it for traversal and compatibility-sensitive use. It is the measured baseline.
- [globset 0.4.22](https://crates.io/crates/globset/0.4.22) is an independently callable matcher, also used by `ignore`; it amortizes general multi-pattern matching and has wider options. Removing this machinery is a scope trade, not a drop-in improvement.
- [Git's specification](https://git-scm.com/docs/gitignore) explains why re-including a file beneath an excluded parent cannot work: Git doesn't list excluded directories. Keep the parent check even when leaf-only matching looks faster. Full Git parity has not been verified.

## Bottleneck and measurements

For literal-heavy policy files the linear candidate repeats byte comparisons for most rules at each path component. The indexed candidate hashes names/extensions and merges the latest applicable indexed rule with a reverse scan of general rules. Common globs trim literal ends and special-case a single wildcard; the general bounded row algorithm avoids exponential backtracking but costs time proportional to path length times pattern size (including class checks) and allocates two rows. The longest literal-run check rejects some near misses before allocation.

Windows, Rust 1.93, one local release run per iteration. Lower ns/path is better.

| Workload | `ignore` | Indexed + literal check | Linear + literal check | Paired A/A |
|---|---:|---:|---:|---:|
| Small (12 rules) | 383.9 | 76.4 | 134.3 | 57 bps |
| Large (1,028 rules) | 389.9 | 81.8 | 8,905.0 | 284 bps |
| Adversarial (36 rules) | 465.2 | 3,482.1 | 3,470.1 | 98 bps |

Before literal filtering, indexed medians were 75.5, 85.4, and 56,273.5 ns/path against incumbent 378.5, 410.6, and 470.8. Keep indexing for ordinary inputs and literal filtering for bounded near-miss improvement. Reject linear as the default and reject an unconditional claim of superiority: indexed still loses the adversarial workload. Benchmark inputs are generated, not customer traces; concurrency, other CPUs, compile time, allocations and standalone glob speed are unverified.

## Pre-mortem

- Real corpora are fallback-heavy: profile buyer-owned fixtures and keep `ignore` if no end-to-end win.
- Semantic mismatch hides needed files: start with shadow output, compare with Git on supported normalized inputs, and block rollout on unexplained diffs.
- Throughput improvement doesn't change the CI bill: require measurable end-to-end savings and a paid commitment, not positive feedback.

The defensible commercial asset is integration know-how, a repeatable corpus validation process, customer-approved operational evidence, and support—not restriction of the Apache-licensed implementation.
