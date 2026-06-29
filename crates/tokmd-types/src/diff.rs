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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_diff_row() -> DiffRow {
        DiffRow {
            lang: "Rust".to_string(),
            old_code: 100,
            new_code: 150,
            delta_code: 50,
            old_lines: 200,
            new_lines: 260,
            delta_lines: 60,
            old_files: 5,
            new_files: 7,
            delta_files: 2,
            old_bytes: 5_000,
            new_bytes: 6_500,
            delta_bytes: 1_500,
            old_tokens: 1_200,
            new_tokens: 1_700,
            delta_tokens: 500,
        }
    }

    fn sample_diff_totals() -> DiffTotals {
        DiffTotals {
            old_code: 1_000,
            new_code: 1_200,
            delta_code: 200,
            old_lines: 1_500,
            new_lines: 1_800,
            delta_lines: 300,
            old_files: 10,
            new_files: 12,
            delta_files: 2,
            old_bytes: 40_000,
            new_bytes: 48_000,
            delta_bytes: 8_000,
            old_tokens: 10_000,
            new_tokens: 12_000,
            delta_tokens: 2_000,
        }
    }

    // ── DiffRow ──────────────────────────────────────────────────────
    #[test]
    fn diff_row_serde_roundtrip() {
        let row = sample_diff_row();
        let json = serde_json::to_string(&row).expect("serialize");
        let back: DiffRow = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, row);
    }

    #[test]
    fn diff_row_field_names_stable() {
        let row = sample_diff_row();
        let value = serde_json::to_value(&row).expect("to_value");
        for key in [
            "lang",
            "old_code",
            "new_code",
            "delta_code",
            "old_lines",
            "new_lines",
            "delta_lines",
            "old_files",
            "new_files",
            "delta_files",
            "old_bytes",
            "new_bytes",
            "delta_bytes",
            "old_tokens",
            "new_tokens",
            "delta_tokens",
        ] {
            assert!(
                value.get(key).is_some(),
                "missing expected field `{key}` in DiffRow JSON: {value}"
            );
        }
    }

    #[test]
    fn diff_row_delta_consistency() {
        let row = sample_diff_row();
        assert_eq!(
            row.delta_code,
            (row.new_code as i64) - (row.old_code as i64)
        );
        assert_eq!(
            row.delta_lines,
            (row.new_lines as i64) - (row.old_lines as i64)
        );
        assert_eq!(
            row.delta_files,
            (row.new_files as i64) - (row.old_files as i64)
        );
        assert_eq!(
            row.delta_bytes,
            (row.new_bytes as i64) - (row.old_bytes as i64)
        );
        assert_eq!(
            row.delta_tokens,
            (row.new_tokens as i64) - (row.old_tokens as i64)
        );
    }

    #[test]
    fn diff_totals_delta_consistency() {
        let totals = sample_diff_totals();
        assert_eq!(
            totals.delta_code,
            (totals.new_code as i64) - (totals.old_code as i64)
        );
        assert_eq!(
            totals.delta_lines,
            (totals.new_lines as i64) - (totals.old_lines as i64)
        );
        assert_eq!(
            totals.delta_files,
            (totals.new_files as i64) - (totals.old_files as i64)
        );
        assert_eq!(
            totals.delta_bytes,
            (totals.new_bytes as i64) - (totals.old_bytes as i64)
        );
        assert_eq!(
            totals.delta_tokens,
            (totals.new_tokens as i64) - (totals.old_tokens as i64)
        );
    }

    // ── DiffTotals ───────────────────────────────────────────────────

    #[test]
    fn diff_totals_serde_roundtrip() {
        let totals = sample_diff_totals();
        let json = serde_json::to_string(&totals).expect("serialize");
        let back: DiffTotals = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, totals);
    }

    #[test]
    fn diff_totals_default_is_all_zero() {
        let totals = DiffTotals::default();
        let value = serde_json::to_value(&totals).expect("to_value");
        for key in [
            "old_code",
            "new_code",
            "delta_code",
            "old_lines",
            "new_lines",
            "delta_lines",
            "old_files",
            "new_files",
            "delta_files",
            "old_bytes",
            "new_bytes",
            "delta_bytes",
            "old_tokens",
            "new_tokens",
            "delta_tokens",
        ] {
            assert_eq!(
                value[key], 0,
                "expected default DiffTotals.{key} to serialize to 0, got {}",
                value[key]
            );
        }
    }

    #[test]
    fn diff_totals_negative_deltas_roundtrip() {
        let totals = DiffTotals {
            old_code: 200,
            new_code: 50,
            delta_code: -150,
            old_files: 8,
            new_files: 4,
            delta_files: -4,
            ..DiffTotals::default()
        };
        let json = serde_json::to_string(&totals).expect("serialize");
        let back: DiffTotals = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, totals);
        assert_eq!(back.delta_code, -150);
        assert_eq!(back.delta_files, -4);
    }

    // ── DiffReceipt ──────────────────────────────────────────────────
    #[test]
    fn diff_receipt_serde_roundtrip() {
        let receipt = DiffReceipt {
            schema_version: crate::SCHEMA_VERSION,
            generated_at_ms: 1_700_000_000_000,
            tool: ToolInfo {
                name: "tokmd".into(),
                version: "1.2.3".into(),
            },
            mode: "diff".into(),
            from_source: "old.json".into(),
            to_source: "new.json".into(),
            diff_rows: vec![sample_diff_row()],
            totals: sample_diff_totals(),
        };

        let json = serde_json::to_string(&receipt).expect("serialize");
        let value: serde_json::Value = serde_json::from_str(&json).expect("to value");
        // Field stability check
        for key in [
            "schema_version",
            "generated_at_ms",
            "tool",
            "mode",
            "from_source",
            "to_source",
            "diff_rows",
            "totals",
        ] {
            assert!(
                value.get(key).is_some(),
                "missing expected field `{key}` in DiffReceipt JSON"
            );
        }

        let back: DiffReceipt = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.schema_version, receipt.schema_version);
        assert_eq!(back.mode, "diff");
        assert_eq!(back.from_source, "old.json");
        assert_eq!(back.to_source, "new.json");
        assert_eq!(back.diff_rows.len(), 1);
        assert_eq!(back.diff_rows[0], receipt.diff_rows[0]);
        assert_eq!(back.totals, receipt.totals);
    }

    #[test]
    fn diff_receipt_preserves_row_order() {
        let mut rows = Vec::new();
        for (i, lang) in ["Rust", "Go", "C++", "Python"].iter().enumerate() {
            let mut row = sample_diff_row();
            row.lang = (*lang).to_string();
            row.delta_code = i as i64;
            rows.push(row);
        }
        let receipt = DiffReceipt {
            schema_version: crate::SCHEMA_VERSION,
            generated_at_ms: 0,
            tool: ToolInfo::default(),
            mode: "diff".into(),
            from_source: "a".into(),
            to_source: "b".into(),
            diff_rows: rows.clone(),
            totals: DiffTotals::default(),
        };
        let json = serde_json::to_string(&receipt).expect("serialize");
        let back: DiffReceipt = serde_json::from_str(&json).expect("deserialize");
        let back_langs: Vec<_> = back.diff_rows.iter().map(|r| r.lang.as_str()).collect();
        assert_eq!(back_langs, vec!["Rust", "Go", "C++", "Python"]);
    }
}
