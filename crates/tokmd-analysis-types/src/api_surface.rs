//! API surface receipt DTOs.
//!
//! These contract types remain re-exported from the crate root to preserve
//! existing `tokmd_analysis_types::...` names.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Public API surface analysis report.
///
/// Computes public export ratios per language and module by scanning
/// source files for exported symbols (pub fn, export function, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSurfaceReport {
    /// Total items discovered across all languages.
    pub total_items: usize,
    /// Items with public visibility.
    pub public_items: usize,
    /// Items with internal/private visibility.
    pub internal_items: usize,
    /// Ratio of public to total items (0.0-1.0).
    pub public_ratio: f64,
    /// Ratio of documented public items (0.0-1.0).
    pub documented_ratio: f64,
    /// Per-language breakdown.
    pub by_language: BTreeMap<String, LangApiSurface>,
    /// Per-module breakdown.
    pub by_module: Vec<ModuleApiRow>,
    /// Top exporters (files with most public items).
    pub top_exporters: Vec<ApiExportItem>,
}

/// Per-language API surface breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangApiSurface {
    /// Total items in this language.
    pub total_items: usize,
    /// Public items in this language.
    pub public_items: usize,
    /// Internal items in this language.
    pub internal_items: usize,
    /// Public ratio for this language.
    pub public_ratio: f64,
}

/// Per-module API surface row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleApiRow {
    /// Module path.
    pub module: String,
    /// Total items in this module.
    pub total_items: usize,
    /// Public items in this module.
    pub public_items: usize,
    /// Public ratio for this module.
    pub public_ratio: f64,
}

/// A file that exports many public items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiExportItem {
    /// File path.
    pub path: String,
    /// Language of the file.
    pub lang: String,
    /// Number of public items exported.
    pub public_items: usize,
    /// Total items in the file.
    pub total_items: usize,
}
