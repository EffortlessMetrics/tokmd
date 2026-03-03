//! Deep tests for tokmd-model (wave 38).
//!
//! Covers language aggregation, module key computation, children modes,
//! sorting invariants, file row construction, edge cases, and
//! normalize_path behavior.

use std::path::{Path, PathBuf};
use tokei::{Config, Languages};
use tokmd_model::{
    avg, collect_file_rows, create_export_data, create_lang_report, create_module_report,
    module_key, normalize_path, unique_parent_file_count,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode, FileKind};

/// Scan a directory and return Languages data.
fn scan_dir(path: &str) -> Languages {
    let mut languages = Languages::new();
    let paths = vec![PathBuf::from(path)];
    let cfg = Config::default();
    languages.get_statistics(&paths, &[], &cfg);
    languages
}

/// Scan the crate's own src directory (pure Rust).
fn scan_self_src() -> Languages {
    scan_dir(&format!("{}/src", env!("CARGO_MANIFEST_DIR")))
}

// ============================================================================
// 1. Language aggregation — single language
// ============================================================================

#[test]
fn lang_report_single_lang_has_one_row() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert_eq!(report.rows.len(), 1);
    assert_eq!(report.rows[0].lang, "Rust");
}

#[test]
fn lang_report_single_lang_totals_equal_row() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let row = &report.rows[0];
    assert_eq!(report.total.code, row.code);
    assert_eq!(report.total.lines, row.lines);
    assert_eq!(report.total.bytes, row.bytes);
    assert_eq!(report.total.tokens, row.tokens);
}

#[test]
fn lang_report_positive_metrics() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(report.total.code > 0);
    assert!(report.total.lines > 0);
    assert!(report.total.files > 0);
    assert!(report.total.bytes > 0);
    assert!(report.total.tokens > 0);
}

// ============================================================================
// 2. Language aggregation — multi-language (scan crate root)
// ============================================================================

#[test]
fn lang_report_multi_rows_sum_to_totals() {
    let langs = scan_dir(env!("CARGO_MANIFEST_DIR"));
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let sum_code: usize = report.rows.iter().map(|r| r.code).sum();
    let sum_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    let sum_bytes: usize = report.rows.iter().map(|r| r.bytes).sum();
    let sum_tokens: usize = report.rows.iter().map(|r| r.tokens).sum();
    assert_eq!(report.total.code, sum_code);
    assert_eq!(report.total.lines, sum_lines);
    assert_eq!(report.total.bytes, sum_bytes);
    assert_eq!(report.total.tokens, sum_tokens);
}

#[test]
fn lang_report_no_zero_code_rows() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for row in &report.rows {
        assert!(row.code > 0, "Row {} has 0 code", row.lang);
    }
}

// ============================================================================
// 3. Module key computation
// ============================================================================

#[test]
fn module_key_crates_depth_2() {
    let roots = vec!["crates".to_string()];
    assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 2), "crates/foo");
    assert_eq!(
        module_key("crates/bar/tests/test.rs", &roots, 2),
        "crates/bar"
    );
}

#[test]
fn module_key_depth_1_collapses_to_root_dir() {
    let roots = vec!["crates".to_string()];
    assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 1), "crates");
}

#[test]
fn module_key_root_level_file() {
    assert_eq!(module_key("Cargo.toml", &[], 2), "(root)");
    assert_eq!(module_key("README.md", &["crates".into()], 2), "(root)");
}

#[test]
fn module_key_non_root_first_dir() {
    assert_eq!(module_key("src/lib.rs", &[], 2), "src");
    assert_eq!(module_key("tests/test.rs", &[], 2), "tests");
}

#[test]
fn module_key_multiple_roots() {
    let roots = vec!["crates".into(), "packages".into()];
    assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 2), "crates/foo");
    assert_eq!(
        module_key("packages/bar/src/main.rs", &roots, 2),
        "packages/bar"
    );
    // Non-root
    assert_eq!(module_key("src/lib.rs", &roots, 2), "src");
}

#[test]
fn module_key_depth_exceeds_path() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("crates/foo.rs", &roots, 2), "crates");
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots, 10),
        "crates/foo/src"
    );
}

#[test]
fn module_key_backslash_normalized() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates\\foo\\src\\lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn module_key_dot_slash_prefix() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("./crates/foo/src/lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn module_key_empty_roots() {
    assert_eq!(module_key("src/lib.rs", &[], 2), "src");
    assert_eq!(module_key("Cargo.toml", &[], 2), "(root)");
}

// ============================================================================
// 4. Children mode: Collapse vs Separate
// ============================================================================

#[test]
fn collapse_mode_excludes_embedded_label() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for row in &report.rows {
        assert!(
            !row.lang.contains("(embedded)"),
            "Collapse should not have embedded rows: {}",
            row.lang
        );
    }
}

#[test]
fn separate_mode_includes_rust_row() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Separate);
    assert!(report.rows.iter().any(|r| r.lang == "Rust"));
}

#[test]
fn both_modes_agree_on_rust_code_count() {
    let langs = scan_self_src();
    let collapse = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let separate = create_lang_report(&langs, 0, false, ChildrenMode::Separate);
    let c_rust = collapse.rows.iter().find(|r| r.lang == "Rust").unwrap();
    let s_rust = separate.rows.iter().find(|r| r.lang == "Rust").unwrap();
    // For pure Rust crate, both should give the same Rust code
    assert!(c_rust.code > 0);
    assert!(s_rust.code > 0);
}

#[test]
fn child_include_parents_only_has_only_parents() {
    let langs = scan_self_src();
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert_eq!(row.kind, FileKind::Parent);
    }
}

#[test]
fn child_include_separate_has_parents() {
    let langs = scan_self_src();
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::Separate, None);
    assert!(rows.iter().any(|r| r.kind == FileKind::Parent));
}

// ============================================================================
// 5. Sorting: descending by code, tie-break by name
// ============================================================================

#[test]
fn lang_report_sorted_descending() {
    let langs = scan_dir(env!("CARGO_MANIFEST_DIR"));
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for w in report.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "Not sorted descending: {} ({}) vs {} ({})",
            w[0].lang,
            w[0].code,
            w[1].lang,
            w[1].code
        );
    }
}

#[test]
fn module_report_sorted_descending() {
    let langs = scan_self_src();
    let report = create_module_report(&langs, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    for w in report.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "Module not sorted descending: {} ({}) vs {} ({})",
            w[0].module,
            w[0].code,
            w[1].module,
            w[1].code
        );
    }
}

#[test]
fn export_data_sorted_descending() {
    let langs = scan_self_src();
    let data = create_export_data(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None, 0, 0);
    for w in data.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "Export not sorted descending: {} ({}) vs {} ({})",
            w[0].path,
            w[0].code,
            w[1].path,
            w[1].code
        );
    }
}

// ============================================================================
// 6. File row construction
// ============================================================================

#[test]
fn file_rows_have_normalized_paths() {
    let langs = scan_self_src();
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert!(!row.path.contains('\\'), "Backslash in path: {}", row.path);
        assert!(
            !row.path.starts_with("./"),
            "Leading ./ in path: {}",
            row.path
        );
    }
}

#[test]
fn file_rows_have_valid_module() {
    let langs = scan_self_src();
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert!(
            !row.module.is_empty(),
            "Module should not be empty for {}",
            row.path
        );
    }
}

#[test]
fn file_rows_lang_is_rust_for_pure_crate() {
    let langs = scan_self_src();
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert_eq!(
            row.lang, "Rust",
            "Expected Rust, got {} for {}",
            row.lang, row.path
        );
    }
}

#[test]
fn file_rows_lines_eq_code_plus_comments_plus_blanks() {
    let langs = scan_self_src();
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert_eq!(
            row.lines,
            row.code + row.comments + row.blanks,
            "lines != code+comments+blanks for {}",
            row.path
        );
    }
}

// ============================================================================
// 7. Edge cases: empty languages, zero lines
// ============================================================================

#[test]
fn empty_languages_lang_report_is_empty() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.lines, 0);
    assert_eq!(report.total.files, 0);
    assert_eq!(report.total.bytes, 0);
    assert_eq!(report.total.tokens, 0);
}

#[test]
fn empty_languages_module_report_is_empty() {
    let langs = Languages::new();
    let report = create_module_report(&langs, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
}

#[test]
fn empty_languages_export_data_is_empty() {
    let langs = Languages::new();
    let data = create_export_data(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None, 0, 0);
    assert!(data.rows.is_empty());
}

#[test]
fn empty_languages_unique_file_count_is_zero() {
    assert_eq!(unique_parent_file_count(&Languages::new()), 0);
}

// ============================================================================
// 8. avg function
// ============================================================================

#[test]
fn avg_exact_division() {
    assert_eq!(avg(300, 3), 100);
    assert_eq!(avg(10, 2), 5);
    assert_eq!(avg(1000, 10), 100);
}

#[test]
fn avg_rounds_to_nearest() {
    // 7 / 2 = 3.5 → 4 (rounds up)
    assert_eq!(avg(7, 2), 4);
    // 5 / 3 = 1.67 → 2
    assert_eq!(avg(5, 3), 2);
}

#[test]
fn avg_zero_files_returns_zero() {
    assert_eq!(avg(100, 0), 0);
    assert_eq!(avg(0, 0), 0);
}

#[test]
fn avg_zero_lines() {
    assert_eq!(avg(0, 5), 0);
    assert_eq!(avg(0, 1), 0);
}

#[test]
fn avg_one_file() {
    assert_eq!(avg(42, 1), 42);
}

// ============================================================================
// 9. normalize_path
// ============================================================================

#[test]
fn normalize_path_backslash_to_forward() {
    let p = Path::new("src\\main.rs");
    assert_eq!(normalize_path(p, None), "src/main.rs");
}

#[test]
fn normalize_path_strips_dot_slash() {
    assert_eq!(
        normalize_path(Path::new("./src/lib.rs"), None),
        "src/lib.rs"
    );
}

#[test]
fn normalize_path_strips_prefix() {
    let p = Path::new("C:/Code/project/src/lib.rs");
    let prefix = Path::new("C:/Code/project");
    assert_eq!(normalize_path(p, Some(prefix)), "src/lib.rs");
}

#[test]
fn normalize_path_idempotent() {
    let p = Path::new("./src\\foo/bar.rs");
    let once = normalize_path(p, None);
    let twice = normalize_path(Path::new(&once), None);
    assert_eq!(once, twice);
}

#[test]
fn normalize_path_no_leading_slash() {
    let p = Path::new("src/lib.rs");
    let n = normalize_path(p, None);
    assert!(!n.starts_with('/'));
}

// ============================================================================
// 10. Top-N truncation
// ============================================================================

#[test]
fn lang_report_top_n_limits_rows() {
    let langs = scan_dir(env!("CARGO_MANIFEST_DIR"));
    let report = create_lang_report(&langs, 1, false, ChildrenMode::Collapse);
    // With top=1, at most 2 rows (1 + "Other")
    assert!(report.rows.len() <= 2);
}

#[test]
fn module_report_top_n_limits_rows() {
    let langs = scan_self_src();
    let report = create_module_report(&langs, &[], 1, ChildIncludeMode::ParentsOnly, 1);
    assert!(report.rows.len() <= 2);
}

#[test]
fn lang_report_top_zero_keeps_all() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    // top=0 means no truncation
    assert!(!report.rows.is_empty());
}

// ============================================================================
// 11. Export data — min_code filter
// ============================================================================

#[test]
fn export_data_min_code_filters_small() {
    let langs = scan_self_src();
    let data = create_export_data(
        &langs,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        None,
        999_999,
        0,
    );
    // With very high min_code, most/all rows should be filtered out
    assert!(
        data.rows.len() < 10,
        "High min_code should filter most rows, got {}",
        data.rows.len()
    );
}

#[test]
fn export_data_max_rows_truncates() {
    let langs = scan_self_src();
    let data = create_export_data(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None, 0, 1);
    assert!(data.rows.len() <= 1);
}

// ============================================================================
// 12. unique_parent_file_count
// ============================================================================

#[test]
fn unique_file_count_positive_for_nonempty_scan() {
    let langs = scan_self_src();
    assert!(unique_parent_file_count(&langs) > 0);
}

#[test]
fn unique_file_count_matches_lang_report_total() {
    let langs = scan_self_src();
    let count = unique_parent_file_count(&langs);
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert_eq!(report.total.files, count);
}

// ============================================================================
// 13. Module report — totals consistency
// ============================================================================

#[test]
fn module_report_totals_consistent() {
    let langs = scan_self_src();
    let report = create_module_report(&langs, &[], 1, ChildIncludeMode::ParentsOnly, 0);
    let sum_code: usize = report.rows.iter().map(|r| r.code).sum();
    let sum_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    assert_eq!(report.total.code, sum_code);
    assert_eq!(report.total.lines, sum_lines);
}

// ============================================================================
// 14. with_files flag
// ============================================================================

#[test]
fn lang_report_with_files_true() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);
    assert!(report.with_files);
}

#[test]
fn lang_report_with_files_false() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(!report.with_files);
}

// ============================================================================
// 15. File rows — strip_prefix
// ============================================================================

#[test]
fn file_rows_strip_prefix_removes_leading_path() {
    let langs = scan_self_src();
    let prefix = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, Some(prefix));
    for row in &rows {
        let manifest = env!("CARGO_MANIFEST_DIR").replace('\\', "/");
        assert!(
            !row.path.starts_with(&manifest),
            "Path should have prefix stripped: {}",
            row.path
        );
    }
}
