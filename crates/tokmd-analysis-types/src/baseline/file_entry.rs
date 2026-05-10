//! Per-file baseline receipt DTOs.
//!
//! This submodule owns granular baseline entries while preserving the existing
//! `FileBaselineEntry` re-export from `tokmd_analysis_types`.

use serde::{Deserialize, Serialize};

/// Per-file baseline entry for granular complexity tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBaselineEntry {
    /// Normalized file path (forward slashes).
    pub path: String,
    /// Lines of code in this file.
    pub code_lines: u64,
    /// Cyclomatic complexity for this file.
    pub cyclomatic: u32,
    /// Cognitive complexity for this file.
    pub cognitive: u32,
    /// Maximum nesting depth in this file.
    pub max_nesting: u32,
    /// Number of functions in this file.
    pub function_count: u32,
    /// BLAKE3 hash of file content for change detection.
    pub content_hash: Option<String>,
}
