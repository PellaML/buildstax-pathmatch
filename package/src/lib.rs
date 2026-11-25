#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod glob;
mod ignore;

pub use glob::{Glob, PatternError};
pub use ignore::{Decision, Ignore, RuleError};
