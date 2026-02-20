//! # tokmd-analysis
//!
//! **Tier 3 (Orchestration)**
//!
//! Analysis logic and optional enrichers for tokmd receipts. Computes derived
//! metrics and orchestrates optional analysis modules based on presets.
//!
//! ## What belongs here
//! * Analysis orchestration and module coordination
//! * Derived metric computation
//! * Preset-based feature inclusion
//! * Enricher implementations (archetype, topics, git metrics, etc.)
//!
//! ## What does NOT belong here
//! * Output formatting (use tokmd-analysis-format)
//! * CLI argument parsing
//! * File modification

mod analysis;
#[cfg(all(feature = "content", feature = "walk"))]
mod api_surface;
mod archetype;
#[cfg(feature = "walk")]
mod assets;
#[cfg(feature = "git")]
mod churn;
#[cfg(all(feature = "content", feature = "walk"))]
mod complexity;
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
#[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
mod halstead;
#[cfg(all(feature = "content", feature = "walk"))]
mod license;
#[cfg(feature = "content")]
mod near_dup;
mod topics;
mod util;

pub use analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity, analyze,
};
pub use tokmd_analysis_types::NearDupScope;
pub use util::normalize_root;
