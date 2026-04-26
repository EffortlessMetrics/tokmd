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
#[cfg(all(feature = "content", feature = "walk"))]
mod api_surface;
#[cfg(feature = "archetype")]
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
#[cfg(feature = "effort")]
mod effort;
#[cfg(all(feature = "content", feature = "walk"))]
mod entropy;
#[cfg(feature = "git")]
mod fingerprint;
#[cfg(feature = "fun")]
mod fun;
#[cfg(feature = "git")]
mod git;
mod grid;
#[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
mod halstead;
#[cfg(feature = "content")]
mod imports;
#[cfg(all(feature = "content", feature = "walk"))]
mod license;
#[cfg(all(feature = "content", feature = "walk"))]
mod maintainability;
#[cfg(feature = "content")]
mod near_dup;
#[cfg(feature = "topics")]
mod topics;
mod util;

pub use analysis::{AnalysisContext, AnalysisPreset, AnalysisRequest, ImportGranularity, analyze};
pub use derived::{build_tree, derive_report};
#[cfg(feature = "effort")]
pub use effort::{EffortLayer, EffortModelKind, EffortRequest};
pub use grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetKind, PresetPlan, preset_plan_for,
    preset_plan_for_name,
};
pub use tokmd_analysis_types::AnalysisLimits;
pub use tokmd_analysis_types::NearDupScope;
pub use util::normalize_root;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub mod readme_doctests {}
