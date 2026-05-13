//! # tokmd-core
//!
//! **Tier 4 (Library Facade)**
//!
//! This crate is the **primary library interface** for `tokmd`.
//! It coordinates scanning, aggregation, and modeling to produce code inventory receipts.
//!
//! If you are embedding `tokmd` into another Rust application, depend on this crate
//! and `tokmd-types`. Avoid depending on `tokmd-scan` or `tokmd-model` directly unless necessary.
//!
//! ## What belongs here
//! * High-level workflow coordination
//! * Simplified API for library consumers
//! * Re-exports for convenience
//! * FFI-friendly JSON entrypoint
//!
//! ## What does NOT belong here
//! * CLI argument parsing (use tokmd crate)
//! * Low-level scanning logic (use tokmd-scan)
//! * Aggregation details (use tokmd-model)
//!
//! ## Example
//!
//! ```rust
//! use tokmd_core::{lang_workflow, settings::{ScanSettings, LangSettings}};
//!
//! // Configure scan
//! let scan = ScanSettings::current_dir();
//! let lang = LangSettings {
//!     top: 10,
//!     files: true,
//!     ..Default::default()
//! };
//!
//! // Run pipeline
//! let receipt = lang_workflow(&scan, &lang).expect("Scan failed");
//! assert!(receipt.report.rows.len() > 0);
//! ```
//!
//! ## JSON API (for bindings)
//!
//! ```rust
//! use tokmd_core::ffi::run_json;
//!
//! let result = run_json("lang", r#"{"paths": ["."], "top": 10}"#);
//! let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
//! assert_eq!(parsed["ok"], true);
//! ```

#![forbid(unsafe_code)]

#[cfg(all(test, feature = "analysis"))]
use tokmd_analysis as analysis;

// Public modules
pub mod context_git;
pub mod context_policy;
pub mod error;
pub mod ffi;
mod receipts;
pub mod settings;
mod workflows;
pub use tokmd_scan::InMemoryFile;
pub use tokmd_types as types;
#[cfg(feature = "cockpit")]
pub use workflows::cockpit_workflow;
#[cfg(all(test, feature = "cockpit"))]
use workflows::parse_cockpit_range_mode;
pub use workflows::{
    TimedWorkflow, WorkflowTiming, diff_workflow, export_workflow, export_workflow_from_inputs,
    lang_workflow, lang_workflow_from_inputs, module_workflow, module_workflow_from_inputs,
    timed_export_workflow, timed_lang_workflow, timed_module_workflow,
};
#[cfg(feature = "analysis")]
pub use workflows::{
    analyze_workflow, analyze_workflow_from_inputs, supports_rootless_in_memory_analyze_preset,
};
#[cfg(all(test, feature = "analysis"))]
use workflows::{parse_analysis_preset, parse_effort_request};

use tokmd_types::SCHEMA_VERSION;

pub(crate) use receipts::{build_export_receipt, build_lang_receipt, build_module_receipt};

// =============================================================================
// Settings-based workflows (new API for bindings)
// =============================================================================

// =============================================================================
// Analysis formatting facade (requires `analysis` feature)
// =============================================================================

/// Analysis formatting re-exports for Tier 5 products.
///
/// This module provides Tier 4 facade access to Tier 3 analysis formatting,
/// maintaining tier boundary compliance for tokmd CLI and other products.
///
/// ## Example
///
/// ```rust
/// use tokmd_core::analysis_facade::{render, RenderedOutput};
/// use tokmd_types::AnalysisFormat;
/// use tokmd_analysis_types::AnalysisReceipt;
///
/// fn format_analysis(receipt: &AnalysisReceipt, format: AnalysisFormat) -> anyhow::Result<String> {
///     match render(receipt, format)? {
///         RenderedOutput::Text(text) => Ok(text),
///         RenderedOutput::Binary(_) => Err(anyhow::anyhow!("Binary output not supported")),
///     }
/// }
/// ```
#[cfg(feature = "analysis")]
pub mod analysis_facade {
    /// Render an analysis receipt to the specified format.
    ///
    /// # Arguments
    /// * `receipt` — The analysis receipt to render (from `tokmd_analysis_types`)
    /// * `format` — Target output format (from `tokmd_types::AnalysisFormat`)
    ///
    /// # Returns
    /// `RenderedOutput` enum containing either text or binary data
    ///
    /// # Errors
    /// Returns error if:
    /// - JSON/XML serialization fails
    /// - `fun` feature is disabled but OBJ/MIDI format requested
    pub use tokmd_format::analysis::render;

    /// Output container for rendered analysis.
    ///
    /// ## Variants
    /// - `Text(String)` — Textual formats: Markdown, JSON, XML, SVG, Mermaid, Tree, HTML
    /// - `Binary(Vec<u8>)` — Binary formats: MIDI (requires `fun` feature)
    pub use tokmd_format::analysis::RenderedOutput;
}

// =============================================================================
// Re-exports for binding convenience
// =============================================================================

/// Re-export schema version for bindings.
pub const CORE_SCHEMA_VERSION: u32 = SCHEMA_VERSION;

/// Re-export analysis schema version for bindings.
#[cfg(feature = "analysis")]
pub const CORE_ANALYSIS_SCHEMA_VERSION: u32 = tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION;

/// Get the current tokmd version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod mutation_tests;
#[cfg(test)]
mod tests;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub mod readme_doctests {}
