//! # tokmd-analysis
//!
//! Analysis logic and optional enrichers for tokmd receipts.

mod analysis;
#[cfg(feature = "walk")]
mod assets;
#[cfg(feature = "content")]
mod content;
mod derived;
mod fun;
#[cfg(feature = "git")]
mod git;
mod util;

pub use analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity, analyze,
};
pub use util::normalize_root;
