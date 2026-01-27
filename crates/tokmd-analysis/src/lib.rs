//! # tokmd-analysis
//!
//! Analysis logic and optional enrichers for tokmd receipts.

mod analysis;
mod derived;
#[cfg(feature = "walk")]
mod assets;
#[cfg(feature = "content")]
mod content;
#[cfg(feature = "git")]
mod git;
mod fun;
mod util;

pub use analysis::{
    analyze, AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
};
pub use util::normalize_root;