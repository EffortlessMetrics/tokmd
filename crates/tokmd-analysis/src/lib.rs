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
//! * Enricher orchestration and adapters (delegated to microcrates)
//!
//! ## What does NOT belong here
//! * Output formatting (use tokmd-analysis-format)
//! * CLI argument parsing
//! * File modification

mod analysis;
#[cfg(feature = "git")]
mod churn;
#[cfg(feature = "content")]
mod content;
mod derived;
#[cfg(feature = "git")]
mod git;
mod util;

pub use analysis::{AnalysisContext, AnalysisPreset, AnalysisRequest, ImportGranularity, analyze};
pub use tokmd_analysis_types::NearDupScope;
pub use tokmd_analysis_util::AnalysisLimits;
pub use util::normalize_root;
