//! # tokmd-analysis
//!
//! Analysis logic and optional enrichers for tokmd receipts.

mod analysis;
mod archetype;
#[cfg(feature = "walk")]
mod assets;
#[cfg(feature = "git")]
mod churn;
#[cfg(feature = "content")]
mod content;
mod derived;
#[cfg(all(feature = "content", feature = "walk"))]
mod entropy;
#[cfg(feature = "git")]
mod fingerprint;
mod fun;
#[cfg(feature = "git")]
mod git;
#[cfg(all(feature = "content", feature = "walk"))]
mod license;
mod topics;
mod util;

pub use analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity, analyze,
};
pub use util::normalize_root;
