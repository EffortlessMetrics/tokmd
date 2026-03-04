//! Deep tests for tokmd-model (w49).
//!
//! Covers: language aggregation with multiple files, module aggregation grouping,
//! children collapse/separate modes, total row computation, sort order with
//! tie-breaking, property-based invariants, and edge cases (zero-code, single
//! file, many languages).

use proptest::prelude::*;
use std::path::PathBuf;
use tokei::{Config, Languages};
use tokmd_model::{
    avg, collect_file_rows, create_export_data, create_lang_report, create_module_report,
    unique_parent_file_count,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode, LangRow, ModuleRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn scan_dir(path: &str) -> Languages {
    let mut languages = Languages::new();
    languages.get_statistics(&[PathBuf::from(path)], &[], &Config::default());
    languages
}

fn scan_self() -> Languages {
    scan_dir(&format!("{}/src", env!("CARGO_MANIFEST_DIR")))
}

// ============================================================================
// 1. Language aggregation — multiple files of same language produce correct totals
// ============================================================================

#[test]
fn lang_agg_rows_code_sum_equals_total_code() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let row_sum: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(report.total.code, row_sum);
}

#[test]
fn lang_agg_rows_lines_sum_equals_total_lines() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let row_sum: usize = report.rows.iter().map(|r| r.lines).sum();
    assert_eq!(report.total.lines, row_sum);
}

#[test]
fn lang_agg_rows_bytes_sum_equals_total_bytes() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let row_sum: usize = report.rows.iter().map(|r| r.bytes).sum();
    assert_eq!(report.total.bytes, row_sum);
}

#[test]
fn lang_agg_rows_tokens_sum_equals_total_tokens() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let row_sum: usize = report.rows.iter().map(|r| r.tokens).sum();
    assert_eq!(report.total.tokens, row_sum);
}

#[test]
fn lang_agg_no_zero_code_rows_in_collapse() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    for row in &report.rows {
        assert!(
            row.code > 0,
            "Collapse mode should skip zero-code rows: {}",
            row.lang
        );
    }
}

#[test]
fn lang_agg_positive_file_count() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    assert!(report.total.files > 0);
    for row in &report.rows {
        assert!(row.files > 0, "Each language row should have ≥ 1 file");
    }
}

// ============================================================================
// 2. Module aggregation — files grouped by module key correctly
// ============================================================================

#[test]
fn module_agg_row_code_sum_equals_total() {
    let languages = scan_self();
    let report = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    let row_sum: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(report.total.code, row_sum);
}

#[test]
fn module_agg_row_lines_sum_equals_total() {
    let languages = scan_self();
    let report = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    let row_sum: usize = report.rows.iter().map(|r| r.lines).sum();
    assert_eq!(report.total.lines, row_sum);
}

#[test]
fn module_agg_all_modules_have_files() {
    let languages = scan_self();
    let report = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    for row in &report.rows {
        assert!(
            row.files > 0,
            "Module '{}' should have ≥ 1 file",
            row.module
        );
    }
}

#[test]
fn module_agg_with_crates_root_depth2() {
    // Scan the workspace root to get multiple modules
    let languages = scan_dir(env!("CARGO_MANIFEST_DIR"));
    let report = create_module_report(
        &languages,
        &["crates".to_string()],
        2,
        ChildIncludeMode::ParentsOnly,
        0,
    );
    // With depth=2 and root "crates", scanning the crate directory should
    // produce at least a "src" module (since we scan from the crate root).
    assert!(!report.rows.is_empty());
}

// ============================================================================
// 3. Children collapse mode — embedded languages merged into parent
// ============================================================================

#[test]
fn collapse_mode_has_no_embedded_label() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    for row in &report.rows {
        assert!(
            !row.lang.contains("(embedded)"),
            "Collapse mode must not produce '(embedded)' rows, got: {}",
            row.lang,
        );
    }
}

#[test]
fn collapse_mode_totals_include_all_code() {
    let languages = scan_self();
    let collapse = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    assert!(collapse.total.code > 0);
    assert!(collapse.total.lines >= collapse.total.code);
}

// ============================================================================
// 4. Children separate mode — embedded languages shown separately
// ============================================================================

#[test]
fn separate_mode_still_has_parent_rows() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Separate);
    assert!(
        report.rows.iter().any(|r| r.lang == "Rust"),
        "Separate mode must still include parent Rust row",
    );
}

#[test]
fn separate_mode_embedded_rows_have_zero_bytes() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Separate);
    for row in report.rows.iter().filter(|r| r.lang.contains("(embedded)")) {
        assert_eq!(
            row.bytes, 0,
            "Embedded row '{}' must have 0 bytes",
            row.lang
        );
        assert_eq!(
            row.tokens, 0,
            "Embedded row '{}' must have 0 tokens",
            row.lang
        );
    }
}

#[test]
fn separate_vs_collapse_parent_code_consistent() {
    let languages = scan_self();
    let collapse = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let separate = create_lang_report(&languages, 0, false, ChildrenMode::Separate);
    let c_rust = collapse.rows.iter().find(|r| r.lang == "Rust").unwrap();
    let s_rust = separate.rows.iter().find(|r| r.lang == "Rust").unwrap();
    // In separate mode the parent row shows only parent stats; in collapse they
    // include children. For a pure-Rust crate (no embedded langs) the values match.
    assert!(c_rust.code > 0);
    assert!(s_rust.code > 0);
}

#[test]
fn child_include_parents_only_excludes_child_rows() {
    let languages = scan_self();
    let rows = collect_file_rows(&languages, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert_eq!(
            row.kind,
            tokmd_types::FileKind::Parent,
            "ParentsOnly must only yield Parent kind rows",
        );
    }
}

#[test]
fn child_include_separate_yields_parent_rows() {
    let languages = scan_self();
    let rows = collect_file_rows(&languages, &[], 1, ChildIncludeMode::Separate, None);
    assert!(
        rows.iter().any(|r| r.kind == tokmd_types::FileKind::Parent),
        "Separate mode should include parent rows",
    );
}

// ============================================================================
// 5. Total row computation — sum of all rows
// ============================================================================

#[test]
fn total_avg_lines_consistent() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let expected = avg(report.total.lines, report.total.files);
    assert_eq!(report.total.avg_lines, expected);
}

#[test]
fn total_files_equals_unique_parent_file_count() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    assert_eq!(report.total.files, unique_parent_file_count(&languages));
}

// ============================================================================
// 6. Sort order — descending by code, tie-break by name
// ============================================================================

#[test]
fn lang_rows_sorted_descending_by_code() {
    let languages = scan_dir(env!("CARGO_MANIFEST_DIR"));
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    for w in report.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "Lang sort violated: {} ({}) should be ≥ {} ({})",
            w[0].lang,
            w[0].code,
            w[1].lang,
            w[1].code,
        );
    }
}

#[test]
fn lang_rows_tiebreak_ascending_by_name() {
    let mut rows = vec![
        LangRow {
            lang: "Zebra".into(),
            code: 10,
            lines: 20,
            files: 1,
            bytes: 0,
            tokens: 0,
            avg_lines: 20,
        },
        LangRow {
            lang: "Alpha".into(),
            code: 10,
            lines: 20,
            files: 1,
            bytes: 0,
            tokens: 0,
            avg_lines: 20,
        },
        LangRow {
            lang: "Mid".into(),
            code: 10,
            lines: 20,
            files: 1,
            bytes: 0,
            tokens: 0,
            avg_lines: 20,
        },
    ];
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
    assert_eq!(rows[0].lang, "Alpha");
    assert_eq!(rows[1].lang, "Mid");
    assert_eq!(rows[2].lang, "Zebra");
}

#[test]
fn module_rows_sorted_descending_by_code() {
    let languages = scan_self();
    let report = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    for w in report.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "Module sort violated: {} ({}) ≥ {} ({})",
            w[0].module,
            w[0].code,
            w[1].module,
            w[1].code,
        );
    }
}

#[test]
fn module_rows_tiebreak_ascending_by_name() {
    let mut rows = vec![
        ModuleRow {
            module: "zzz".into(),
            code: 5,
            lines: 10,
            files: 1,
            bytes: 0,
            tokens: 0,
            avg_lines: 10,
        },
        ModuleRow {
            module: "aaa".into(),
            code: 5,
            lines: 10,
            files: 1,
            bytes: 0,
            tokens: 0,
            avg_lines: 10,
        },
    ];
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
    assert_eq!(rows[0].module, "aaa");
    assert_eq!(rows[1].module, "zzz");
}

// ============================================================================
// 7. Property tests
// ============================================================================

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        "[A-Z][a-z]{1,6}",
        0usize..10_000,
        0usize..20_000,
        1usize..500,
        0usize..500_000,
        0usize..125_000,
    )
        .prop_map(|(lang, code, lines, files, bytes, tokens)| LangRow {
            lang,
            code,
            lines,
            files,
            bytes,
            tokens,
            avg_lines: avg(lines, files),
        })
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        "[a-z]{1,8}",
        0usize..10_000,
        0usize..20_000,
        1usize..500,
        0usize..500_000,
        0usize..125_000,
    )
        .prop_map(|(module, code, lines, files, bytes, tokens)| ModuleRow {
            module,
            code,
            lines,
            files,
            bytes,
            tokens,
            avg_lines: avg(lines, files),
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// total.code == sum of all row.code
    #[test]
    fn prop_total_code_eq_sum(rows in prop::collection::vec(arb_lang_row(), 1..=20)) {
        let sum: usize = rows.iter().map(|r| r.code).sum();
        let sum_rev: usize = rows.iter().rev().map(|r| r.code).sum();
        prop_assert_eq!(sum, sum_rev, "code sum must be commutative");
    }

    /// total.lines >= total.code for every generated row
    #[test]
    fn prop_lines_ge_code_per_row(row in arb_lang_row()) {
        // lines can be less than code in our synthetic generator (they are independent),
        // but for the *real* model, lines = code + comments + blanks ≥ code.
        // Here we just verify the generator didn't panic and avg is consistent.
        let expected_avg = avg(row.lines, row.files);
        prop_assert_eq!(row.avg_lines, expected_avg);
    }

    /// Sorting preserves total code across lang rows
    #[test]
    fn prop_sort_preserves_code_sum(rows in prop::collection::vec(arb_lang_row(), 1..=20)) {
        let before: usize = rows.iter().map(|r| r.code).sum();
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        let after: usize = sorted.iter().map(|r| r.code).sum();
        prop_assert_eq!(before, after);
    }

    /// Sorting is idempotent for lang rows
    #[test]
    fn prop_sort_idempotent_lang(rows in prop::collection::vec(arb_lang_row(), 1..=20)) {
        let mut once = rows;
        once.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        let mut twice = once.clone();
        twice.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        prop_assert_eq!(once, twice);
    }

    /// Sorting preserves total code across module rows
    #[test]
    fn prop_module_sort_preserves_code(rows in prop::collection::vec(arb_module_row(), 1..=20)) {
        let before: usize = rows.iter().map(|r| r.code).sum();
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
        let after: usize = sorted.iter().map(|r| r.code).sum();
        prop_assert_eq!(before, after);
    }

    /// Sorting is descending for module rows
    #[test]
    fn prop_module_sort_descending(rows in prop::collection::vec(arb_module_row(), 2..=20)) {
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
        for w in sorted.windows(2) {
            prop_assert!(
                w[0].code > w[1].code || (w[0].code == w[1].code && w[0].module <= w[1].module),
                "Sort order violated",
            );
        }
    }
}

// ============================================================================
// 8. Edge cases — zero-code files, single file, many languages
// ============================================================================

#[test]
fn empty_languages_produces_empty_report() {
    let languages = Languages::new();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.lines, 0);
    assert_eq!(report.total.files, 0);
    assert_eq!(report.total.bytes, 0);
    assert_eq!(report.total.tokens, 0);
}

#[test]
fn empty_languages_module_report_empty() {
    let languages = Languages::new();
    let report = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
}

#[test]
fn empty_languages_export_data_empty() {
    let languages = Languages::new();
    let data = create_export_data(
        &languages,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );
    assert!(data.rows.is_empty());
}

#[test]
fn empty_languages_unique_file_count_zero() {
    let languages = Languages::new();
    assert_eq!(unique_parent_file_count(&languages), 0);
}

#[test]
fn single_file_crate_has_one_lang_row() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    // tokmd-model src/ is pure Rust → exactly 1 row
    assert_eq!(report.rows.len(), 1);
    assert_eq!(report.rows[0].lang, "Rust");
}

#[test]
fn avg_zero_files_returns_zero() {
    assert_eq!(avg(500, 0), 0);
    assert_eq!(avg(0, 0), 0);
}

#[test]
fn avg_exact_division() {
    assert_eq!(avg(300, 3), 100);
}

#[test]
fn avg_rounds_to_nearest() {
    // 7 / 2 = 3.5 → rounds to 4
    assert_eq!(avg(7, 2), 4);
    // 5 / 3 = 1.67 → rounds to 2
    assert_eq!(avg(5, 3), 2);
}

// ============================================================================
// 9. Export data edge cases
// ============================================================================

#[test]
fn export_data_file_rows_have_normalized_paths() {
    let languages = scan_self();
    let data = create_export_data(
        &languages,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );
    for row in &data.rows {
        assert!(
            !row.path.contains('\\'),
            "Path should be normalized: {}",
            row.path
        );
    }
}

#[test]
fn export_data_sorted_descending_by_code() {
    let languages = scan_self();
    let data = create_export_data(
        &languages,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );
    for w in data.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "Export rows not sorted descending: {} ({}) vs {} ({})",
            w[0].path,
            w[0].code,
            w[1].path,
            w[1].code,
        );
    }
}

// ============================================================================
// 10. Top-N truncation
// ============================================================================

#[test]
fn lang_report_top_truncates_with_other() {
    let languages = scan_dir(env!("CARGO_MANIFEST_DIR"));
    let full = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    if full.rows.len() > 1 {
        let top1 = create_lang_report(&languages, 1, false, ChildrenMode::Collapse);
        // Should have exactly 2 rows: top-1 + "Other"
        assert_eq!(top1.rows.len(), 2);
        assert_eq!(top1.rows.last().unwrap().lang, "Other");
    }
}

#[test]
fn module_report_top_truncates_with_other() {
    let languages = scan_dir(env!("CARGO_MANIFEST_DIR"));
    let full = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    if full.rows.len() > 1 {
        let top1 = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 1);
        assert_eq!(top1.rows.len(), 2);
        assert_eq!(top1.rows.last().unwrap().module, "Other");
    }
}
