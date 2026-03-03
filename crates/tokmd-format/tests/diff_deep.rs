//! Deep tests for diff computation: DiffRow construction, DiffReceipt,
//! self-diff, added-only, removed-only, mixed changes, zero-change,
//! large diff, schema version, determinism, and JSON validity.

use serde_json::Value;
use tokmd_format::{compute_diff_rows, compute_diff_totals, create_diff_receipt};
use tokmd_types::{
    ChildrenMode, DiffReceipt, DiffRow, DiffTotals, LangReport, LangRow, SCHEMA_VERSION, Totals,
};

// =============================================================================
// Helpers
// =============================================================================

fn make_lang_row(lang: &str, code: usize, lines: usize, files: usize) -> LangRow {
    LangRow {
        lang: lang.to_string(),
        code,
        lines,
        files,
        bytes: code * 40,
        tokens: code * 3,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

fn make_report(rows: Vec<LangRow>) -> LangReport {
    let total = Totals {
        code: rows.iter().map(|r| r.code).sum(),
        lines: rows.iter().map(|r| r.lines).sum(),
        files: rows.iter().map(|r| r.files).sum(),
        bytes: rows.iter().map(|r| r.bytes).sum(),
        tokens: rows.iter().map(|r| r.tokens).sum(),
        avg_lines: 0,
    };
    LangReport {
        rows,
        total,
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn empty_report() -> LangReport {
    make_report(vec![])
}

// =============================================================================
// Self-diff: same input twice
// =============================================================================

#[test]
fn self_diff_produces_no_rows() {
    let report = make_report(vec![
        make_lang_row("Rust", 500, 700, 10),
        make_lang_row("Python", 200, 300, 5),
    ]);
    let rows = compute_diff_rows(&report, &report);
    // Identical reports produce no diff rows (zero-delta rows are filtered out)
    assert!(rows.is_empty(), "self-diff should produce no rows");
}

#[test]
fn self_diff_totals_all_zero_deltas() {
    let report = make_report(vec![make_lang_row("Rust", 500, 700, 10)]);
    let rows = compute_diff_rows(&report, &report);
    let totals = compute_diff_totals(&rows);
    assert_eq!(totals.delta_code, 0);
    assert_eq!(totals.delta_lines, 0);
    assert_eq!(totals.delta_files, 0);
    assert_eq!(totals.delta_bytes, 0);
    assert_eq!(totals.delta_tokens, 0);
}

#[test]
fn self_diff_produces_empty_result() {
    let report = make_report(vec![make_lang_row("Rust", 500, 700, 10)]);
    let rows = compute_diff_rows(&report, &report);
    // Identical reports are filtered out (zero-delta rows excluded)
    assert!(rows.is_empty(), "self-diff should produce empty rows");
}

// =============================================================================
// Added-only diff: new languages appear
// =============================================================================

#[test]
fn added_only_diff_shows_new_language() {
    let from = empty_report();
    let to = make_report(vec![make_lang_row("Rust", 100, 150, 3)]);
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].lang, "Rust");
    assert_eq!(rows[0].old_code, 0);
    assert_eq!(rows[0].new_code, 100);
    assert_eq!(rows[0].delta_code, 100);
}

#[test]
fn added_only_totals_match_new_report() {
    let from = empty_report();
    let to = make_report(vec![
        make_lang_row("Rust", 100, 150, 3),
        make_lang_row("TOML", 20, 25, 2),
    ]);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    assert_eq!(totals.new_code, 120);
    assert_eq!(totals.old_code, 0);
    assert_eq!(totals.delta_code, 120);
}

// =============================================================================
// Removed-only diff: languages disappear
// =============================================================================

#[test]
fn removed_only_diff_shows_removed_language() {
    let from = make_report(vec![make_lang_row("Python", 200, 300, 5)]);
    let to = empty_report();
    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].lang, "Python");
    assert_eq!(rows[0].old_code, 200);
    assert_eq!(rows[0].new_code, 0);
    assert_eq!(rows[0].delta_code, -200);
}

#[test]
fn removed_only_totals_show_negative_delta() {
    let from = make_report(vec![make_lang_row("Python", 200, 300, 5)]);
    let to = empty_report();
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    assert!(totals.delta_code < 0);
    assert_eq!(totals.delta_code, -200);
}

// =============================================================================
// Mixed changes: additions, removals, modifications
// =============================================================================

#[test]
fn mixed_diff_shows_all_languages() {
    let from = make_report(vec![
        make_lang_row("Rust", 500, 700, 10),
        make_lang_row("Python", 200, 300, 5),
    ]);
    let to = make_report(vec![
        make_lang_row("Rust", 600, 800, 12),
        make_lang_row("Go", 100, 150, 3),
    ]);
    let rows = compute_diff_rows(&from, &to);
    let langs: Vec<&str> = rows.iter().map(|r| r.lang.as_str()).collect();
    assert!(langs.contains(&"Rust"), "modified language should appear");
    assert!(langs.contains(&"Python"), "removed language should appear");
    assert!(langs.contains(&"Go"), "added language should appear");
}

#[test]
fn mixed_diff_deltas_correct() {
    let from = make_report(vec![
        make_lang_row("Rust", 500, 700, 10),
        make_lang_row("Python", 200, 300, 5),
    ]);
    let to = make_report(vec![
        make_lang_row("Rust", 600, 800, 12),
        make_lang_row("Go", 100, 150, 3),
    ]);
    let rows = compute_diff_rows(&from, &to);
    let rust_row = rows.iter().find(|r| r.lang == "Rust").unwrap();
    assert_eq!(rust_row.delta_code, 100);

    let python_row = rows.iter().find(|r| r.lang == "Python").unwrap();
    assert_eq!(python_row.delta_code, -200);

    let go_row = rows.iter().find(|r| r.lang == "Go").unwrap();
    assert_eq!(go_row.delta_code, 100);
}

// =============================================================================
// Zero-change diff: both empty
// =============================================================================

#[test]
fn zero_change_diff_empty_reports() {
    let from = empty_report();
    let to = empty_report();
    let rows = compute_diff_rows(&from, &to);
    assert!(
        rows.is_empty(),
        "empty-to-empty diff should produce no rows"
    );
}

#[test]
fn zero_change_diff_totals_all_zero() {
    let rows: Vec<DiffRow> = vec![];
    let totals = compute_diff_totals(&rows);
    assert_eq!(totals, DiffTotals::default());
}

// =============================================================================
// Large diff
// =============================================================================

#[test]
fn large_diff_many_languages() {
    let from_rows: Vec<LangRow> = (0..50)
        .map(|i| make_lang_row(&format!("Lang{i}"), i * 10, i * 15, i.max(1)))
        .collect();
    let to_rows: Vec<LangRow> = (25..75)
        .map(|i| make_lang_row(&format!("Lang{i}"), i * 12, i * 18, i.max(1)))
        .collect();
    let from = make_report(from_rows);
    let to = make_report(to_rows);
    let rows = compute_diff_rows(&from, &to);

    // Should contain all unique languages from both
    assert!(rows.len() >= 50, "should have at least 50 unique languages");
    assert!(rows.len() <= 75, "should have at most 75 unique languages");

    // Verify totals are consistent
    let totals = compute_diff_totals(&rows);
    assert_eq!(
        totals.delta_code,
        totals.new_code as i64 - totals.old_code as i64
    );
}

// =============================================================================
// DiffReceipt construction
// =============================================================================

#[test]
fn diff_receipt_schema_version_matches_constant() {
    let receipt = create_diff_receipt("from.json", "to.json", vec![], DiffTotals::default());
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn diff_receipt_mode_is_diff() {
    let receipt = create_diff_receipt("a", "b", vec![], DiffTotals::default());
    assert_eq!(receipt.mode, "diff");
}

#[test]
fn diff_receipt_sources_preserved() {
    let receipt = create_diff_receipt(
        "run1/receipt.json",
        "run2/receipt.json",
        vec![],
        DiffTotals::default(),
    );
    assert_eq!(receipt.from_source, "run1/receipt.json");
    assert_eq!(receipt.to_source, "run2/receipt.json");
}

#[test]
fn diff_receipt_tool_info_present() {
    let receipt = create_diff_receipt("a", "b", vec![], DiffTotals::default());
    assert_eq!(receipt.tool.name, "tokmd");
    assert!(!receipt.tool.version.is_empty());
}

#[test]
fn diff_receipt_generated_at_ms_nonzero() {
    let receipt = create_diff_receipt("a", "b", vec![], DiffTotals::default());
    assert!(receipt.generated_at_ms > 0);
}

// =============================================================================
// JSON output validity
// =============================================================================

#[test]
fn diff_receipt_json_valid() {
    let from = make_report(vec![make_lang_row("Rust", 500, 700, 10)]);
    let to = make_report(vec![make_lang_row("Rust", 600, 800, 12)]);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let receipt = create_diff_receipt("from.json", "to.json", rows, totals);
    let json_str = serde_json::to_string_pretty(&receipt).unwrap();
    let val: Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(val["schema_version"], SCHEMA_VERSION);
    assert_eq!(val["mode"], "diff");
    assert!(val["diff_rows"].is_array());
    assert!(val["totals"].is_object());
    assert!(val["tool"].is_object());
}

#[test]
fn diff_receipt_json_has_all_required_keys() {
    let receipt = create_diff_receipt("a", "b", vec![], DiffTotals::default());
    let val: Value = serde_json::to_value(&receipt).unwrap();
    for key in &[
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "from_source",
        "to_source",
        "diff_rows",
        "totals",
    ] {
        assert!(val.get(key).is_some(), "missing key: {key}");
    }
}

// =============================================================================
// Deterministic output
// =============================================================================

#[test]
fn diff_rows_deterministic_order() {
    let from = make_report(vec![
        make_lang_row("Rust", 500, 700, 10),
        make_lang_row("Python", 200, 300, 5),
        make_lang_row("Go", 100, 150, 3),
    ]);
    let to = make_report(vec![
        make_lang_row("Go", 120, 180, 4),
        make_lang_row("Rust", 550, 750, 11),
        make_lang_row("Python", 210, 310, 5),
    ]);
    let rows1 = compute_diff_rows(&from, &to);
    let rows2 = compute_diff_rows(&from, &to);

    let langs1: Vec<&str> = rows1.iter().map(|r| r.lang.as_str()).collect();
    let langs2: Vec<&str> = rows2.iter().map(|r| r.lang.as_str()).collect();
    assert_eq!(langs1, langs2, "diff row order should be deterministic");
}

#[test]
fn diff_rows_sorted_alphabetically() {
    let from = make_report(vec![
        make_lang_row("Rust", 500, 700, 10),
        make_lang_row("Go", 100, 150, 3),
        make_lang_row("Python", 200, 300, 5),
    ]);
    let to = make_report(vec![
        make_lang_row("Rust", 550, 750, 11),
        make_lang_row("Go", 120, 180, 4),
        make_lang_row("Python", 210, 310, 5),
    ]);
    let rows = compute_diff_rows(&from, &to);
    let langs: Vec<&str> = rows.iter().map(|r| r.lang.as_str()).collect();
    let mut sorted = langs.clone();
    sorted.sort();
    assert_eq!(langs, sorted, "diff rows should be sorted by language name");
}

// =============================================================================
// DiffRow serde roundtrip
// =============================================================================

#[test]
fn diff_row_serde_roundtrip() {
    let row = DiffRow {
        lang: "Rust".to_string(),
        old_code: 100,
        new_code: 120,
        delta_code: 20,
        old_lines: 200,
        new_lines: 220,
        delta_lines: 20,
        old_files: 10,
        new_files: 11,
        delta_files: 1,
        old_bytes: 5000,
        new_bytes: 6000,
        delta_bytes: 1000,
        old_tokens: 250,
        new_tokens: 300,
        delta_tokens: 50,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: DiffRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn diff_totals_default_is_all_zeros() {
    let t = DiffTotals::default();
    assert_eq!(t.old_code, 0);
    assert_eq!(t.new_code, 0);
    assert_eq!(t.delta_code, 0);
    assert_eq!(t.old_lines, 0);
    assert_eq!(t.new_lines, 0);
    assert_eq!(t.delta_lines, 0);
}

#[test]
fn diff_totals_serde_roundtrip() {
    let t = DiffTotals {
        old_code: 500,
        new_code: 600,
        delta_code: 100,
        old_lines: 700,
        new_lines: 800,
        delta_lines: 100,
        old_files: 10,
        new_files: 12,
        delta_files: 2,
        old_bytes: 20000,
        new_bytes: 24000,
        delta_bytes: 4000,
        old_tokens: 1500,
        new_tokens: 1800,
        delta_tokens: 300,
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: DiffTotals = serde_json::from_str(&json).unwrap();
    assert_eq!(back, t);
}

// =============================================================================
// DiffReceipt serde roundtrip
// =============================================================================

#[test]
fn diff_receipt_serde_roundtrip() {
    let from = make_report(vec![make_lang_row("Rust", 500, 700, 10)]);
    let to = make_report(vec![make_lang_row("Rust", 600, 800, 12)]);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    let receipt = create_diff_receipt("from.json", "to.json", rows, totals);

    let json_str = serde_json::to_string(&receipt).unwrap();
    let back: DiffReceipt = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.schema_version, receipt.schema_version);
    assert_eq!(back.mode, receipt.mode);
    assert_eq!(back.from_source, receipt.from_source);
    assert_eq!(back.to_source, receipt.to_source);
    assert_eq!(back.diff_rows.len(), receipt.diff_rows.len());
    assert_eq!(back.totals, receipt.totals);
}

// =============================================================================
// Totals consistency
// =============================================================================

#[test]
fn diff_totals_delta_equals_new_minus_old() {
    let from = make_report(vec![
        make_lang_row("Rust", 500, 700, 10),
        make_lang_row("Python", 200, 300, 5),
    ]);
    let to = make_report(vec![
        make_lang_row("Rust", 600, 800, 12),
        make_lang_row("Go", 100, 150, 3),
    ]);
    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);
    assert_eq!(
        totals.delta_code,
        totals.new_code as i64 - totals.old_code as i64
    );
    assert_eq!(
        totals.delta_lines,
        totals.new_lines as i64 - totals.old_lines as i64
    );
    assert_eq!(
        totals.delta_files,
        totals.new_files as i64 - totals.old_files as i64
    );
    assert_eq!(
        totals.delta_bytes,
        totals.new_bytes as i64 - totals.old_bytes as i64
    );
    assert_eq!(
        totals.delta_tokens,
        totals.new_tokens as i64 - totals.old_tokens as i64
    );
}

// =============================================================================
// Row-level delta consistency
// =============================================================================

#[test]
fn each_diff_row_delta_equals_new_minus_old() {
    let from = make_report(vec![
        make_lang_row("Rust", 500, 700, 10),
        make_lang_row("Python", 200, 300, 5),
    ]);
    let to = make_report(vec![
        make_lang_row("Rust", 600, 800, 12),
        make_lang_row("Python", 180, 280, 4),
    ]);
    let rows = compute_diff_rows(&from, &to);
    for row in &rows {
        assert_eq!(
            row.delta_code,
            row.new_code as i64 - row.old_code as i64,
            "{}: delta_code mismatch",
            row.lang
        );
        assert_eq!(
            row.delta_lines,
            row.new_lines as i64 - row.old_lines as i64,
            "{}: delta_lines mismatch",
            row.lang
        );
        assert_eq!(
            row.delta_files,
            row.new_files as i64 - row.old_files as i64,
            "{}: delta_files mismatch",
            row.lang
        );
    }
}
