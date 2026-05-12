//! Diff receipt DTOs.
//!
//! This module owns the serde-stable structures emitted by `tokmd diff`.
//! Public consumers should continue using the root-level re-exports from
//! `tokmd_types`.

use serde::{Deserialize, Serialize};

use crate::ToolInfo;

/// A row in the diff output showing changes for a single language.
///
/// # Examples
///
/// ```
/// use tokmd_types::DiffRow;
///
/// let row = DiffRow {
///     lang: "Rust".to_string(),
///     old_code: 1000, new_code: 1200, delta_code: 200,
///     old_lines: 1500, new_lines: 1800, delta_lines: 300,
///     old_files: 10,   new_files: 12,   delta_files: 2,
///     old_bytes: 40000, new_bytes: 48000, delta_bytes: 8000,
///     old_tokens: 10000, new_tokens: 12000, delta_tokens: 2000,
/// };
/// assert_eq!(row.delta_code, (row.new_code as i64) - (row.old_code as i64));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffRow {
    pub lang: String,
    pub old_code: usize,
    pub new_code: usize,
    pub delta_code: i64,
    pub old_lines: usize,
    pub new_lines: usize,
    pub delta_lines: i64,
    pub old_files: usize,
    pub new_files: usize,
    pub delta_files: i64,
    pub old_bytes: usize,
    pub new_bytes: usize,
    pub delta_bytes: i64,
    pub old_tokens: usize,
    pub new_tokens: usize,
    pub delta_tokens: i64,
}

/// Aggregate totals for the diff.
///
/// # Examples
///
/// ```
/// use tokmd_types::DiffTotals;
///
/// // Default is all zeros
/// let totals = DiffTotals::default();
/// assert_eq!(totals.delta_code, 0);
/// assert_eq!(totals.delta_files, 0);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffTotals {
    pub old_code: usize,
    pub new_code: usize,
    pub delta_code: i64,
    pub old_lines: usize,
    pub new_lines: usize,
    pub delta_lines: i64,
    pub old_files: usize,
    pub new_files: usize,
    pub delta_files: i64,
    pub old_bytes: usize,
    pub new_bytes: usize,
    pub delta_bytes: i64,
    pub old_tokens: usize,
    pub new_tokens: usize,
    pub delta_tokens: i64,
}

/// JSON receipt for diff output with envelope metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReceipt {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub tool: ToolInfo,
    pub mode: String,
    pub from_source: String,
    pub to_source: String,
    pub diff_rows: Vec<DiffRow>,
    pub totals: DiffTotals,
}
