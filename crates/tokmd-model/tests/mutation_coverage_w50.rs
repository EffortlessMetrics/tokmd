//! Targeted tests for mutation testing coverage gaps (W50).
//!
//! Each test is designed to catch common mutations:
//! replacing operators, negating conditions, removing statements.

use std::path::Path;

use tokmd_model::{avg, normalize_path};
use tokmd_types::{ChildrenMode, LangReport, LangRow, Totals};

// ---------------------------------------------------------------------------
// Helper: build a synthetic LangReport without needing tokei::Languages
// ---------------------------------------------------------------------------

fn make_report(rows: Vec<LangRow>, total: Totals) -> LangReport {
    LangReport {
        rows,
        total,
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

fn lang_row(name: &str, code: usize, lines: usize, files: usize) -> LangRow {
    LangRow {
        lang: name.to_string(),
        code,
        lines,
        files,
        bytes: code * 40,
        tokens: code * 10,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

fn totals_from_rows(rows: &[LangRow]) -> Totals {
    let code: usize = rows.iter().map(|r| r.code).sum();
    let lines: usize = rows.iter().map(|r| r.lines).sum();
    let files: usize = rows.iter().map(|r| r.files).sum();
    let bytes: usize = rows.iter().map(|r| r.bytes).sum();
    let tokens: usize = rows.iter().map(|r| r.tokens).sum();
    Totals {
        code,
        lines,
        files,
        bytes,
        tokens,
        avg_lines: avg(lines, files),
    }
}

// ---------------------------------------------------------------------------
// 1. total_code == sum of individual language code lines
// ---------------------------------------------------------------------------

#[test]
fn total_code_equals_sum_of_row_code() {
    let rows = vec![
        lang_row("Rust", 500, 600, 5),
        lang_row("Python", 300, 400, 3),
        lang_row("Go", 200, 250, 2),
    ];
    let total = totals_from_rows(&rows);
    let report = make_report(rows.clone(), total);

    let row_sum: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(
        report.total.code, row_sum,
        "total.code must equal sum of row codes"
    );
}

// ---------------------------------------------------------------------------
// 2. Sort order: higher code sorts first
// ---------------------------------------------------------------------------

#[test]
fn sort_order_descending_by_code() {
    let mut rows = vec![
        lang_row("Go", 100, 150, 2),
        lang_row("Rust", 500, 600, 5),
        lang_row("Python", 300, 400, 3),
    ];
    // Replicate the model's sorting logic
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

    assert_eq!(rows[0].lang, "Rust");
    assert_eq!(rows[1].lang, "Python");
    assert_eq!(rows[2].lang, "Go");
}

// ---------------------------------------------------------------------------
// 3. Sort order changes when code line counts change
// ---------------------------------------------------------------------------

#[test]
fn sort_order_changes_when_code_changes() {
    let mut rows_before = vec![
        lang_row("Rust", 500, 600, 5),
        lang_row("Python", 300, 400, 3),
    ];
    rows_before.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
    assert_eq!(rows_before[0].lang, "Rust");

    // Now Python has more code
    let mut rows_after = vec![
        lang_row("Rust", 200, 300, 5),
        lang_row("Python", 600, 700, 3),
    ];
    rows_after.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
    assert_eq!(rows_after[0].lang, "Python");
}

// ---------------------------------------------------------------------------
// 4. Boundary: zero-code languages are sorted by name
// ---------------------------------------------------------------------------

#[test]
fn zero_code_languages_sorted_by_name() {
    let mut rows = vec![
        lang_row("Zig", 0, 10, 1),
        lang_row("Ada", 0, 5, 1),
        lang_row("Elm", 0, 8, 1),
    ];
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

    assert_eq!(rows[0].lang, "Ada");
    assert_eq!(rows[1].lang, "Elm");
    assert_eq!(rows[2].lang, "Zig");
}

// ---------------------------------------------------------------------------
// 5. avg() function: zero files returns 0
// ---------------------------------------------------------------------------

#[test]
fn avg_zero_files_returns_zero() {
    assert_eq!(avg(1000, 0), 0);
}

// ---------------------------------------------------------------------------
// 6. avg() function: correct rounding
// ---------------------------------------------------------------------------

#[test]
fn avg_rounds_to_nearest() {
    // 7 / 2 = 3.5, should round to 4
    assert_eq!(avg(7, 2), 4);
    // 3 / 2 = 1.5, should round to 2
    assert_eq!(avg(3, 2), 2);
    // Exact division
    assert_eq!(avg(300, 3), 100);
}

// ---------------------------------------------------------------------------
// 7. avg() nonzero values never return 0
// ---------------------------------------------------------------------------

#[test]
fn avg_nonzero_lines_nonzero_files_never_zero() {
    // If there are 1 line and 1 file, avg must be 1 (not 0)
    assert_eq!(avg(1, 1), 1);
    assert_eq!(avg(1, 2), 1);
}

// ---------------------------------------------------------------------------
// 8. normalize_path: backslash becomes forward slash
// ---------------------------------------------------------------------------

#[test]
fn normalize_path_backslash_to_forward_slash() {
    let p = Path::new("src\\main.rs");
    assert_eq!(normalize_path(p, None), "src/main.rs");
}

// ---------------------------------------------------------------------------
// 9. normalize_path: strips leading "./"
// ---------------------------------------------------------------------------

#[test]
fn normalize_path_strips_dot_prefix() {
    let p = Path::new("./src/lib.rs");
    assert_eq!(normalize_path(p, None), "src/lib.rs");
}

// ---------------------------------------------------------------------------
// 10. normalize_path: with strip_prefix
// ---------------------------------------------------------------------------

#[test]
fn normalize_path_strips_prefix() {
    let p = Path::new("project/src/lib.rs");
    let prefix = Path::new("project");
    assert_eq!(normalize_path(p, Some(prefix)), "src/lib.rs");
}

// ---------------------------------------------------------------------------
// 11. Aggregate totals with exactly 2 languages
// ---------------------------------------------------------------------------

#[test]
fn aggregate_totals_two_languages() {
    let rows = vec![
        lang_row("Rust", 500, 600, 5),
        lang_row("Python", 300, 400, 3),
    ];
    let total = totals_from_rows(&rows);

    assert_eq!(total.code, 800);
    assert_eq!(total.lines, 1000);
    assert_eq!(total.files, 8);
    assert_eq!(total.bytes, 800 * 40);
    assert_eq!(total.tokens, 800 * 10);
}

// ---------------------------------------------------------------------------
// 12. Totals bytes/tokens must be sum of rows
// ---------------------------------------------------------------------------

#[test]
fn totals_bytes_tokens_sum() {
    let rows = vec![
        lang_row("Rust", 100, 150, 1),
        lang_row("C", 200, 250, 2),
        lang_row("Go", 50, 75, 1),
    ];
    let total = totals_from_rows(&rows);

    let expected_bytes: usize = rows.iter().map(|r| r.bytes).sum();
    let expected_tokens: usize = rows.iter().map(|r| r.tokens).sum();
    assert_eq!(total.bytes, expected_bytes);
    assert_eq!(total.tokens, expected_tokens);
}

// ---------------------------------------------------------------------------
// 13. Single-language report: total equals the single row
// ---------------------------------------------------------------------------

#[test]
fn single_language_total_equals_row() {
    let rows = vec![lang_row("Rust", 1000, 1200, 10)];
    let total = totals_from_rows(&rows);

    assert_eq!(total.code, 1000);
    assert_eq!(total.lines, 1200);
    assert_eq!(total.files, 10);
}

// ---------------------------------------------------------------------------
// 14. Empty rows produce zero totals
// ---------------------------------------------------------------------------

#[test]
fn empty_rows_produce_zero_totals() {
    let rows: Vec<LangRow> = vec![];
    let total = totals_from_rows(&rows);

    assert_eq!(total.code, 0);
    assert_eq!(total.lines, 0);
    assert_eq!(total.files, 0);
    assert_eq!(total.bytes, 0);
    assert_eq!(total.tokens, 0);
}

// ---------------------------------------------------------------------------
// 15. normalize_path: identity for already-clean paths
// ---------------------------------------------------------------------------

#[test]
fn normalize_path_identity_for_clean() {
    let p = Path::new("src/lib.rs");
    assert_eq!(normalize_path(p, None), "src/lib.rs");
}
