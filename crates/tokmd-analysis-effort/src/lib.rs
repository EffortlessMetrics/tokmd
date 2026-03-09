//! # tokmd-analysis-effort
//!
//! Deterministic effort-estimation support for analysis receipts.
//!
//! This crate turns repository inventory plus optional analysis enrichers into a
//! top-level [`tokmd_analysis_types::EffortEstimateReport`].
//!
//! The engine is intentionally layered:
//!
//! 1. build an authored-vs-total size basis,
//! 2. run a deterministic baseline model over authored KLOC,
//! 3. widen or narrow the estimate using observed repo signals,
//! 4. explain the result with drivers and confidence reasons,
//! 5. optionally attach a base/head delta estimate.
//!
//! The implementation is local and receipt-driven. It uses repository files and
//! already-computed analysis reports; it does not call external services.
//!
//! ## Inputs
//!
//! The effort builder can consume:
//!
//! - `ExportData` for per-file size, language, and module context,
//! - `DerivedReport` for totals, tests, polyglot spread, and baseline derived metrics,
//! - optional `GitReport`, `ComplexityReport`, `ApiSurfaceReport`, and
//!   `DuplicateReport` for richer driver extraction and confidence scoring.
//!
//! ## Output contract
//!
//! The main output is [`tokmd_analysis_types::EffortEstimateReport`], which
//! contains:
//!
//! - `size_basis`
//! - `results`
//! - `confidence`
//! - `drivers`
//! - `assumptions`
//! - optional `delta`
//!
//! Rendering layers should treat this crate as the source of estimate semantics
//! and should not infer missing values on their own.

pub mod classify;
pub mod cocomo2;
pub mod cocomo81;
pub mod confidence;
pub mod delta;
pub mod drivers;
pub mod model;
pub mod monte_carlo;
pub mod request;
pub mod size_basis;
pub mod uncertainty;

// Public request/config surface used by orchestration and CLI plumbing.
// Main effort builder entrypoint.
pub use model::build_effort_report;
pub use request::{DeltaInput, EffortLayer, EffortModelKind, EffortRequest};
