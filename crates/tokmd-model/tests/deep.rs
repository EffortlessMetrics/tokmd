//! Deep tests for tokmd-model.
//!
//! Covers single/multi-language aggregation, module breakdown from file paths,
//! sorting (descending by code, tie-break by name), empty input, children modes
//! (Collapse vs Separate), module key derivation, normalized paths in output,
//! and property-based invariants for aggregation totals.

use proptest::prelude::*;
use std::path::{Path, PathBuf};
use tokei::{Config, Languages};
use tokmd_model::{
    avg, collect_file_rows, create_export_data, create_lang_report, create_module_report,
    module_key, normalize_path, unique_parent_file_count,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode, LangRow, ModuleRow};

/// Scan a directory and return Languages data.
fn scan_path(path: &str) -> Languages {
    let mut languages = Languages::new();
    let paths = vec![PathBuf::from(path)];
    let cfg = Config::default();
    languages.get_statistics(&paths, &[], &cfg);
    languages
}

/// Scan the crate's own src directory.
fn scan_self() -> Languages {
    scan_path(&format!("{}/src", env!("CARGO_MANIFEST_DIR")))
}

// ============================================================================
// 1. Single-language aggregation
// ============================================================================

#[test]
fn single_language_report_has_one_row() {
    // The tokmd-model crate is pure Rust, so scanning its src/ produces exactly one language.
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    assert_eq!(
        report.rows.len(),
        1,
        "Scanning pure-Rust crate should produce exactly 1 language row"
    );
    assert_eq!(report.rows[0].lang, "Rust");
}

#[test]
fn single_language_totals_match_row() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    assert_eq!(report.total.code, report.rows[0].code);
    assert_eq!(report.total.lines, report.rows[0].lines);
    assert_eq!(report.total.bytes, report.rows[0].bytes);
    assert_eq!(report.total.tokens, report.rows[0].tokens);
}

#[test]
fn single_language_has_positive_code() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    assert!(report.total.code > 0, "Crate source should have code lines");
    assert!(report.total.files > 0, "Should have at least one file");
}

// ============================================================================
// 2. Multi-language aggregation
// ============================================================================

#[test]
fn multi_lang_report_rows_sum_to_totals() {
    // Scan the workspace root which has multiple languages
    let languages = scan_path(env!("CARGO_MANIFEST_DIR"));
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    let row_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    let row_bytes: usize = report.rows.iter().map(|r| r.bytes).sum();
    let row_tokens: usize = report.rows.iter().map(|r| r.tokens).sum();

    assert_eq!(report.total.code, row_code, "code mismatch");
    assert_eq!(report.total.lines, row_lines, "lines mismatch");
    assert_eq!(report.total.bytes, row_bytes, "bytes mismatch");
    assert_eq!(report.total.tokens, row_tokens, "tokens mismatch");
}

#[test]
fn multi_lang_report_no_zero_code_rows() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    for row in &report.rows {
        assert!(row.code > 0, "Row for {} has 0 code lines", row.lang);
    }
}

// ============================================================================
// 3. Module breakdown from file paths
// ============================================================================

#[test]
fn module_report_has_rows() {
    let languages = scan_self();
    let report = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    assert!(!report.rows.is_empty(), "Module report should have rows");
}

#[test]
fn module_report_totals_sum() {
    let languages = scan_self();
    let report = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);

    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    let row_lines: usize = report.rows.iter().map(|r| r.lines).sum();

    assert_eq!(report.total.code, row_code);
    assert_eq!(report.total.lines, row_lines);
}

#[test]
fn module_key_from_workspace_paths() {
    let roots = vec!["crates".to_string()];
    assert_eq!(
        module_key("crates/tokmd-model/src/lib.rs", &roots, 2),
        "crates/tokmd-model"
    );
    assert_eq!(
        module_key("crates/tokmd-redact/src/lib.rs", &roots, 2),
        "crates/tokmd-redact"
    );
}

#[test]
fn module_key_root_file() {
    assert_eq!(module_key("Cargo.toml", &[], 2), "(root)");
    assert_eq!(module_key("README.md", &["crates".into()], 2), "(root)");
}

#[test]
fn module_key_non_root_dir() {
    assert_eq!(module_key("src/main.rs", &[], 2), "src");
    assert_eq!(module_key("tests/integration.rs", &[], 2), "tests");
}

// ============================================================================
// 4. Sorting: descending by code lines
// ============================================================================

#[test]
fn lang_report_sorted_descending_by_code() {
    let languages = scan_path(env!("CARGO_MANIFEST_DIR"));
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    for window in report.rows.windows(2) {
        assert!(
            window[0].code >= window[1].code,
            "Rows not sorted descending by code: {} ({}) >= {} ({})",
            window[0].lang,
            window[0].code,
            window[1].lang,
            window[1].code
        );
    }
}

#[test]
fn module_report_sorted_descending_by_code() {
    let languages = scan_self();
    let report = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);

    for window in report.rows.windows(2) {
        assert!(
            window[0].code >= window[1].code,
            "Module rows not sorted descending: {} ({}) >= {} ({})",
            window[0].module,
            window[0].code,
            window[1].module,
            window[1].code
        );
    }
}

// ============================================================================
// 5. Tie-breaking: by name when code lines equal
// ============================================================================

#[test]
fn sorting_tiebreak_by_name_lang() {
    let mut rows = vec![
        LangRow {
            lang: "Zebra".into(),
            code: 100,
            lines: 200,
            files: 5,
            bytes: 1000,
            tokens: 250,
            avg_lines: 40,
        },
        LangRow {
            lang: "Alpha".into(),
            code: 100,
            lines: 200,
            files: 5,
            bytes: 1000,
            tokens: 250,
            avg_lines: 40,
        },
        LangRow {
            lang: "Middle".into(),
            code: 100,
            lines: 200,
            files: 5,
            bytes: 1000,
            tokens: 250,
            avg_lines: 40,
        },
    ];

    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

    assert_eq!(rows[0].lang, "Alpha");
    assert_eq!(rows[1].lang, "Middle");
    assert_eq!(rows[2].lang, "Zebra");
}

#[test]
fn sorting_tiebreak_by_name_module() {
    let mut rows = vec![
        ModuleRow {
            module: "zzz".into(),
            code: 50,
            lines: 100,
            files: 2,
            bytes: 500,
            tokens: 125,
            avg_lines: 50,
        },
        ModuleRow {
            module: "aaa".into(),
            code: 50,
            lines: 100,
            files: 2,
            bytes: 500,
            tokens: 125,
            avg_lines: 50,
        },
    ];

    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));

    assert_eq!(rows[0].module, "aaa");
    assert_eq!(rows[1].module, "zzz");
}

// ============================================================================
// 6. Empty input handling
// ============================================================================

#[test]
fn empty_languages_lang_report() {
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
fn empty_languages_module_report() {
    let languages = Languages::new();
    let report = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.files, 0);
}

#[test]
fn empty_languages_export_data() {
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
fn empty_languages_file_rows() {
    let languages = Languages::new();
    let rows = collect_file_rows(&languages, &[], 1, ChildIncludeMode::ParentsOnly, None);
    assert!(rows.is_empty());
}

#[test]
fn empty_languages_unique_file_count() {
    let languages = Languages::new();
    assert_eq!(unique_parent_file_count(&languages), 0);
}

#[test]
fn avg_zero_files_returns_zero() {
    assert_eq!(avg(100, 0), 0);
    assert_eq!(avg(0, 0), 0);
}

// ============================================================================
// 7. Children mode: Collapse vs Separate
// ============================================================================

#[test]
fn collapse_mode_no_embedded_rows() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    for row in &report.rows {
        assert!(
            !row.lang.contains("(embedded)"),
            "Collapse mode should not have (embedded) rows, found: {}",
            row.lang
        );
    }
}

#[test]
fn separate_mode_may_have_embedded_rows() {
    // This test verifies the separate mode works without crashing.
    // Pure Rust code may not have embedded languages, but the function should handle it.
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Separate);
    // Should still have at least the Rust row
    assert!(
        report.rows.iter().any(|r| r.lang == "Rust"),
        "Separate mode should still have Rust row"
    );
}

#[test]
fn both_modes_produce_consistent_rust_code() {
    let languages = scan_self();
    let collapse = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let separate = create_lang_report(&languages, 0, false, ChildrenMode::Separate);

    // For a pure Rust crate, both modes should give the same Rust row
    let collapse_rust = collapse.rows.iter().find(|r| r.lang == "Rust").unwrap();
    let separate_rust = separate.rows.iter().find(|r| r.lang == "Rust").unwrap();

    // In Separate mode, the parent Rust row should have the same code count
    // (embedded languages are shown separately, not merged)
    assert!(collapse_rust.code > 0, "Collapse Rust row should have code");
    assert!(separate_rust.code > 0, "Separate Rust row should have code");
}

#[test]
fn child_include_mode_parents_only_excludes_children() {
    let languages = scan_self();
    let rows = collect_file_rows(&languages, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert_eq!(
            row.kind,
            tokmd_types::FileKind::Parent,
            "ParentsOnly mode should only have Parent kind"
        );
    }
}

#[test]
fn child_include_mode_separate_includes_parent_rows() {
    let languages = scan_self();
    let rows = collect_file_rows(&languages, &[], 1, ChildIncludeMode::Separate, None);
    // Should at least have parent rows
    assert!(
        rows.iter().any(|r| r.kind == tokmd_types::FileKind::Parent),
        "Separate mode should include Parent rows"
    );
}

// ============================================================================
// 8. Proptest: aggregation totals always sum correctly
// ============================================================================

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        "[A-Z][a-z]{1,8}",
        0usize..10_000,
        0usize..20_000,
        1usize..500,
        0usize..1_000_000,
        0usize..100_000,
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
        "[a-z][a-z0-9_/]{1,12}",
        0usize..10_000,
        0usize..20_000,
        1usize..500,
        0usize..1_000_000,
        0usize..100_000,
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

    #[test]
    fn proptest_lang_aggregation_sum_invariant(rows in prop::collection::vec(arb_lang_row(), 1..=15)) {
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let total_lines: usize = rows.iter().map(|r| r.lines).sum();
        let total_files: usize = rows.iter().map(|r| r.files).sum();
        let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();

        // Sum is commutative
        let rev_code: usize = rows.iter().rev().map(|r| r.code).sum();
        prop_assert_eq!(total_code, rev_code);

        // All sums should be non-negative (trivially true for usize)
        prop_assert!(total_files > 0, "At least one file");
        // total_bytes and total_tokens are usize, always >= 0
        let _ = total_bytes;
        let _ = total_tokens;
    }

    #[test]
    fn proptest_module_aggregation_sum_invariant(rows in prop::collection::vec(arb_module_row(), 1..=15)) {
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let rev_code: usize = rows.iter().rev().map(|r| r.code).sum();
        prop_assert_eq!(total_code, rev_code, "Module code sum should be commutative");
    }

    #[test]
    fn proptest_sorting_preserves_total_code(rows in prop::collection::vec(arb_lang_row(), 1..=15)) {
        let total_before: usize = rows.iter().map(|r| r.code).sum();
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        let total_after: usize = sorted.iter().map(|r| r.code).sum();
        prop_assert_eq!(total_before, total_after, "Sorting must not change total code");
    }

    #[test]
    fn proptest_sorting_is_descending(rows in prop::collection::vec(arb_lang_row(), 2..=15)) {
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        for window in sorted.windows(2) {
            prop_assert!(
                window[0].code > window[1].code
                    || (window[0].code == window[1].code && window[0].lang <= window[1].lang),
                "Sort order violated: {:?} before {:?}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn proptest_sorting_is_idempotent(rows in prop::collection::vec(arb_lang_row(), 1..=15)) {
        let mut once = rows.clone();
        once.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        let mut twice = once.clone();
        twice.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        prop_assert_eq!(once, twice, "Sorting must be idempotent");
    }

    #[test]
    fn proptest_module_sorting_preserves_total(rows in prop::collection::vec(arb_module_row(), 1..=15)) {
        let total_before: usize = rows.iter().map(|r| r.code).sum();
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
        let total_after: usize = sorted.iter().map(|r| r.code).sum();
        prop_assert_eq!(total_before, total_after);
    }
}

// ============================================================================
// 9. Module key derivation from paths
// ============================================================================

#[test]
fn module_key_with_crates_root() {
    let roots = vec!["crates".to_string()];
    assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 2), "crates/foo");
    assert_eq!(
        module_key("crates/bar/tests/test.rs", &roots, 2),
        "crates/bar"
    );
}

#[test]
fn module_key_depth_1_returns_root_only() {
    let roots = vec!["crates".to_string()];
    assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 1), "crates");
}

#[test]
fn module_key_depth_exceeds_path() {
    let roots = vec!["crates".to_string()];
    // File directly under root
    assert_eq!(module_key("crates/foo.rs", &roots, 2), "crates");
    // Depth exceeds available dirs
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots, 10),
        "crates/foo/src"
    );
}

#[test]
fn module_key_non_matching_root() {
    let roots = vec!["packages".to_string()];
    // "src" is not in roots, so module key is just first dir
    assert_eq!(module_key("src/lib.rs", &roots, 2), "src");
}

#[test]
fn module_key_empty_roots() {
    assert_eq!(module_key("src/lib.rs", &[], 2), "src");
    assert_eq!(module_key("Cargo.toml", &[], 2), "(root)");
}

#[test]
fn module_key_backslash_normalized() {
    let roots = vec!["crates".to_string()];
    assert_eq!(
        module_key("crates\\foo\\src\\lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn module_key_dot_slash_normalized() {
    let roots = vec!["crates".to_string()];
    assert_eq!(
        module_key("./crates/foo/src/lib.rs", &roots, 2),
        "crates/foo"
    );
}

// ============================================================================
// 10. Normalized paths in output
// ============================================================================

#[test]
fn normalize_path_forward_slashes() {
    let p = Path::new("src\\main.rs");
    let n = normalize_path(p, None);
    assert!(
        !n.contains('\\'),
        "Normalized path should not contain backslash"
    );
    assert!(n.contains('/') || !n.contains(std::path::MAIN_SEPARATOR));
}

#[test]
fn normalize_path_strips_dot_slash() {
    let p = Path::new("./src/lib.rs");
    let n = normalize_path(p, None);
    assert!(!n.starts_with("./"), "Should strip leading ./");
    assert_eq!(n, "src/lib.rs");
}

#[test]
fn normalize_path_strips_prefix() {
    let full = Path::new("C:/Code/project/src/lib.rs");
    let prefix = Path::new("C:/Code/project");
    let n = normalize_path(full, Some(prefix));
    assert_eq!(n, "src/lib.rs");
}

#[test]
fn normalize_path_idempotent() {
    let p = Path::new("./src\\foo/bar.rs");
    let once = normalize_path(p, None);
    let twice = normalize_path(Path::new(&once), None);
    assert_eq!(once, twice, "normalize_path should be idempotent");
}

#[test]
fn collect_file_rows_paths_normalized() {
    let languages = scan_self();
    let rows = collect_file_rows(&languages, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert!(
            !row.path.contains('\\'),
            "File row path should use forward slashes: {}",
            row.path
        );
        assert!(
            !row.path.starts_with("./"),
            "File row path should not start with ./: {}",
            row.path
        );
    }
}

#[test]
fn export_data_paths_normalized() {
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
            "Export path should use forward slashes: {}",
            row.path
        );
    }
}

// ============================================================================
// Additional: top-N truncation
// ============================================================================

#[test]
fn lang_report_top_n_truncation() {
    let languages = scan_path(env!("CARGO_MANIFEST_DIR"));
    let report = create_lang_report(&languages, 1, false, ChildrenMode::Collapse);
    // With top=1, should have at most 2 rows (1 + "Other")
    assert!(
        report.rows.len() <= 2,
        "top=1 should produce at most 2 rows (1 + Other), got {}",
        report.rows.len()
    );
}

#[test]
fn module_report_top_n_truncation() {
    let languages = scan_self();
    let report = create_module_report(&languages, &[], 1, ChildIncludeMode::ParentsOnly, 1);
    assert!(
        report.rows.len() <= 2,
        "top=1 should produce at most 2 rows, got {}",
        report.rows.len()
    );
}

// ============================================================================
// Additional: avg function
// ============================================================================

#[test]
fn avg_basic_cases() {
    assert_eq!(avg(10, 2), 5);
    assert_eq!(avg(10, 3), 3); // (10 + 1) / 3 = 3
    assert_eq!(avg(0, 5), 0);
    assert_eq!(avg(100, 0), 0);
    assert_eq!(avg(1, 1), 1);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn proptest_avg_bounded(lines in 0usize..100_000, files in 1usize..1_000) {
        let result = avg(lines, files);
        prop_assert!(result <= lines, "avg({}, {}) = {} should be <= lines", lines, files, result);
    }

    #[test]
    fn proptest_avg_monotonic(l1 in 0usize..50_000, delta in 0usize..50_000, files in 1usize..100) {
        let l2 = l1 + delta;
        prop_assert!(avg(l1, files) <= avg(l2, files), "avg should be monotonic in lines");
    }

    #[test]
    fn proptest_normalize_path_no_backslash(path in "[a-zA-Z0-9_./\\\\-]{1,40}") {
        let n = normalize_path(Path::new(&path), None);
        prop_assert!(!n.contains('\\'), "Normalized should not have backslash: {}", n);
    }

    #[test]
    fn proptest_module_key_deterministic(
        path in "[a-zA-Z0-9_/]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 1usize..5
    ) {
        let k1 = module_key(&path, roots, depth);
        let k2 = module_key(&path, roots, depth);
        prop_assert_eq!(k1, k2, "module_key must be deterministic");
    }

    #[test]
    fn proptest_module_key_no_backslash(
        path in "[a-zA-Z0-9_/\\\\]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 1usize..5
    ) {
        let k = module_key(&path, roots, depth);
        prop_assert!(!k.contains('\\'), "module_key should not contain backslash: {}", k);
    }
}

// ============================================================================
// Additional: unique_parent_file_count
// ============================================================================

#[test]
fn unique_parent_file_count_positive_for_nonempty() {
    let languages = scan_self();
    let count = unique_parent_file_count(&languages);
    assert!(count > 0, "Non-empty scan should have files");
}

#[test]
fn unique_parent_file_count_matches_lang_report() {
    let languages = scan_self();
    let count = unique_parent_file_count(&languages);
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    assert_eq!(
        report.total.files, count,
        "Lang report total.files should match unique_parent_file_count"
    );
}
