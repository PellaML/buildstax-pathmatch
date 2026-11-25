use buildstax_pathmatch::{Decision, Ignore};
use ignore::gitignore::GitignoreBuilder;
use std::{error::Error, hint::black_box, time::Instant};

fn measure(paths: &[String], f: impl Fn(&str) -> bool) -> f64 {
    let start = Instant::now();
    for _ in 0..12 {
        for path in paths {
            black_box(f(black_box(path)));
        }
    }
    start.elapsed().as_nanos() as f64 / (12 * paths.len()) as f64
}
fn median(mut xs: Vec<f64>) -> f64 {
    xs.sort_by(f64::total_cmp);
    xs[xs.len() / 2]
}
fn main() -> Result<(), Box<dyn Error>> {
    for (label, count) in [("small", 8), ("large", 1024), ("adversarial", 32)] {
        let mut rules = vec![
            "target/".to_owned(),
            "*.log".into(),
            "!keep.log".into(),
            "/local/*.tmp".into(),
        ];
        for i in 0..count {
            rules.push(if label == "adversarial" {
                format!("*a*a*a*a*a*a[0-9]{i:04}*")
            } else {
                format!("artifact-{i}.cache")
            });
        }
        let paths: Vec<_> = (0..96)
            .map(|i| {
                if label == "adversarial" {
                    return format!("{}b{i:04}", "a".repeat(36));
                }
                match i % 7 {
                    0 => format!("src/module-{i}.rs"),
                    1 => format!("target/debug/{i}.exe"),
                    2 => format!("logs/{i}.log"),
                    3 => "logs/keep.log".into(),
                    4 => format!("artifact-{}.cache", i % count),
                    5 => format!("local/{i}.tmp"),
                    _ => format!("src/deep/mod-{i}/lib.rs"),
                }
            })
            .collect();
        let mut builder = GitignoreBuilder::new(".");
        for rule in &rules {
            builder.add_line(None, rule)?;
        }
        let base = builder.build()?;
        let text = rules.join("\n");
        let linear = Ignore::new_linear(&text)?;
        let indexed = Ignore::new(&text)?;
        let a = |p: &str| base.matched_path_or_any_parents(p, false).is_ignore();
        let b = |p: &str| matches!(indexed.decide(p, false), Decision::Ignore { .. });
        let c = |p: &str| matches!(linear.decide(p, false), Decision::Ignore { .. });
        for p in &paths {
            if a(p) != b(p) || b(p) != c(p) {
                return Err(format!("decision mismatch: {p}").into());
            }
        }
        for f in [&a as &dyn Fn(&str) -> bool, &b, &c] {
            black_box(measure(&paths, f));
        }
        let (mut bas, mut idx, mut lin, mut aa) = (vec![], vec![], vec![], vec![]);
        for i in 0..15 {
            let x = measure(&paths, a);
            let y = measure(&paths, a);
            bas.push((x + y) / 2.0);
            aa.push(10_000.0 * (x - y).abs() / ((x + y) / 2.0));
            if i % 2 == 0 {
                idx.push(measure(&paths, b));
                lin.push(measure(&paths, c));
            } else {
                lin.push(measure(&paths, c));
                idx.push(measure(&paths, b));
            }
        }
        println!(
            "{label}: incumbent {:.1}, indexed {:.1}, linear {:.1} ns/path; paired A/A {:.0} bps",
            median(bas),
            median(idx),
            median(lin),
            median(aa)
        );
    }
    Ok(())
}
