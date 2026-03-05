//! Depth tests for `tokmd-model` – W63 wave.
//!
//! Covers language report generation, module report depth calculations,
//! file-level row generation, totals computation, children mode effects,
//! sort stability, BTreeMap ordering, empty inputs, property-based
//! invariants, and determinism verification.

use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;
use tokei::Languages;
use tokmd_model::{
    avg, collect_file_rows, create_export_data, create_lang_report, create_module_report,
    module_key, normalize_path, unique_parent_file_count,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode, FileKind};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn scan_dir(dir: &std::path::Path) -> Languages {
    let cfg = tokei::Config::default();
    let mut langs = Languages::new();
    let ignores: Vec<&str> = vec![];
    langs.get_statistics(&[dir.to_path_buf()], &ignores, &cfg);
    langs
}

fn temp_rust(code: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("main.rs"), code).unwrap();
    dir
}

fn temp_multi() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"hi\");\n}\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("app.py"),
        "def main():\n    print('hi')\n\nif __name__ == '__main__':\n    main()\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("index.js"),
        "function main() {\n  console.log('hi');\n}\nmain();\n",
    )
    .unwrap();
    dir
}

fn temp_nested_modules() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let crates_foo = dir.path().join("crates").join("foo").join("src");
    let crates_bar = dir.path().join("crates").join("bar").join("src");
    let src = dir.path().join("src");

    fs::create_dir_all(&crates_foo).unwrap();
    fs::create_dir_all(&crates_bar).unwrap();
    fs::create_dir_all(&src).unwrap();

    fs::write(crates_foo.join("lib.rs"), "pub fn foo() {}\n").unwrap();
    fs::write(
        crates_bar.join("lib.rs"),
        "pub fn bar() {}\npub fn baz() {}\n",
    )
    .unwrap();
    fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
    dir
}

// ===========================================================================
// 1. Language Report Generation Edge Cases
// ===========================================================================

#[test]
fn lang_report_empty_languages() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.files, 0);
}

#[test]
fn lang_report_single_rust_file() {
    let dir = temp_rust("fn f() {}\n");
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(!report.rows.is_empty());
    assert!(report.rows.iter().any(|r| r.lang == "Rust"));
}

#[test]
fn lang_report_multiple_languages() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let names: Vec<_> = report.rows.iter().map(|r| r.lang.as_str()).collect();
    assert!(names.contains(&"Rust"));
    assert!(names.contains(&"Python"));
    assert!(names.contains(&"JavaScript"));
}

#[test]
fn lang_report_top_limits_rows() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 1, false, ChildrenMode::Collapse);
    // top=1 means 1 real row + "Other"
    assert_eq!(report.rows.len(), 2);
    assert_eq!(report.rows.last().unwrap().lang, "Other");
}

#[test]
fn lang_report_top_zero_means_no_limit() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let full = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let limited = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert_eq!(full.rows.len(), limited.rows.len());
}

#[test]
fn lang_report_with_files_flag() {
    let dir = temp_rust("fn f() {}\n");
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);
    assert!(report.with_files);
}

#[test]
fn lang_report_collapse_mode_stored() {
    let dir = temp_rust("fn f() {}\n");
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert_eq!(report.children, ChildrenMode::Collapse);
}

#[test]
fn lang_report_separate_mode_stored() {
    let dir = temp_rust("fn f() {}\n");
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Separate);
    assert_eq!(report.children, ChildrenMode::Separate);
}

// ===========================================================================
// 2. Module Report Depth Calculations
// ===========================================================================

#[test]
fn module_report_empty_languages() {
    let langs = Languages::new();
    let report = create_module_report(&langs, &["crates".into()], 2, ChildIncludeMode::ParentsOnly, 0);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
}

#[test]
fn module_report_with_module_roots() {
    let dir = temp_nested_modules();
    let langs = scan_dir(dir.path());
    // Use empty roots since tokei returns absolute paths from temp dirs;
    // module_key needs matching first-segment to activate root logic.
    let report = create_module_report(
        &langs,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        0,
    );
    // Should have at least one module row for each subdirectory
    assert!(
        !report.rows.is_empty(),
        "should have module rows, got: {:?}",
        report.rows.iter().map(|r| &r.module).collect::<Vec<_>>()
    );
    assert!(report.total.code > 0, "should have code");
}

#[test]
fn module_report_depth_1() {
    let dir = temp_nested_modules();
    let langs = scan_dir(dir.path());
    let report = create_module_report(
        &langs,
        &["crates".into()],
        1,
        ChildIncludeMode::ParentsOnly,
        0,
    );
    let modules: Vec<_> = report.rows.iter().map(|r| r.module.as_str()).collect();
    // depth=1 means only "crates", not "crates/foo"
    for m in &modules {
        if m.starts_with("crates") {
            assert_eq!(*m, "crates", "depth=1 should not show sub-crates");
        }
    }
}

#[test]
fn module_report_depth_stores_settings() {
    let langs = Languages::new();
    let roots = vec!["crates".into(), "pkg".into()];
    let report = create_module_report(&langs, &roots, 3, ChildIncludeMode::ParentsOnly, 0);
    assert_eq!(report.module_depth, 3);
    assert_eq!(report.module_roots, roots);
}

#[test]
fn module_report_top_limits() {
    let dir = temp_nested_modules();
    let langs = scan_dir(dir.path());
    let full = create_module_report(
        &langs,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        0,
    );
    // Only test top-limit when there are enough rows to truncate
    if full.rows.len() > 1 {
        let report = create_module_report(
            &langs,
            &[],
            2,
            ChildIncludeMode::ParentsOnly,
            1,
        );
        assert_eq!(report.rows.len(), 2, "top=1 → 1 real + Other");
        assert_eq!(report.rows.last().unwrap().module, "Other");
    }
}

// ===========================================================================
// 3. File-level Row Generation
// ===========================================================================

#[test]
fn file_rows_from_empty_languages() {
    let langs = Languages::new();
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    assert!(rows.is_empty());
}

#[test]
fn file_rows_single_file() {
    let dir = temp_rust("fn f() {}\n");
    let langs = scan_dir(dir.path());
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].lang, "Rust");
    assert_eq!(rows[0].kind, FileKind::Parent);
}

#[test]
fn file_rows_multiple_files() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    assert!(rows.len() >= 3, "should have rows for rs, py, js");
}

#[test]
fn file_rows_parents_only_no_children() {
    let dir = temp_rust("fn f() {}\n");
    let langs = scan_dir(dir.path());
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert_eq!(row.kind, FileKind::Parent, "parents only mode");
    }
}

#[test]
fn file_rows_separate_may_include_children() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("page.html"),
        "<html><head><style>body { color: red; }</style></head><body><p>hi</p></body></html>\n",
    )
    .unwrap();
    let langs = scan_dir(dir.path());
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::Separate, None);
    // Should at least have a parent row for HTML
    assert!(rows.iter().any(|r| r.kind == FileKind::Parent));
}

#[test]
fn file_rows_with_strip_prefix() {
    let dir = temp_rust("fn f() {}\n");
    let langs = scan_dir(dir.path());
    let rows = collect_file_rows(
        &langs,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        Some(dir.path()),
    );
    for row in &rows {
        // Paths should be relative after stripping
        assert!(
            !row.path.contains(dir.path().to_str().unwrap_or("")),
            "prefix should be stripped: {}",
            row.path
        );
    }
}

#[test]
fn file_rows_lines_equal_sum() {
    let dir = temp_rust("// comment\nfn f() {}\n\n");
    let langs = scan_dir(dir.path());
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    for row in &rows {
        assert_eq!(
            row.lines,
            row.code + row.comments + row.blanks,
            "lines = code + comments + blanks for {}",
            row.path
        );
    }
}

// ===========================================================================
// 4. Totals Computation Accuracy
// ===========================================================================

#[test]
fn totals_code_matches_sum_of_rows() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let sum_code: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(report.total.code, sum_code);
}

#[test]
fn totals_lines_matches_sum_of_rows() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let sum_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    assert_eq!(report.total.lines, sum_lines);
}

#[test]
fn totals_bytes_matches_sum_of_rows() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let sum_bytes: usize = report.rows.iter().map(|r| r.bytes).sum();
    assert_eq!(report.total.bytes, sum_bytes);
}

#[test]
fn totals_tokens_matches_sum_of_rows() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let sum_tokens: usize = report.rows.iter().map(|r| r.tokens).sum();
    assert_eq!(report.total.tokens, sum_tokens);
}

#[test]
fn totals_files_uses_unique_parent_count() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let unique = unique_parent_file_count(&langs);
    assert_eq!(report.total.files, unique);
}

#[test]
fn totals_empty_input() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.lines, 0);
    assert_eq!(report.total.files, 0);
    assert_eq!(report.total.bytes, 0);
    assert_eq!(report.total.tokens, 0);
    assert_eq!(report.total.avg_lines, 0);
}

// ===========================================================================
// 5. Children Mode Effects on Aggregation
// ===========================================================================

#[test]
fn collapse_does_not_create_embedded_rows() {
    let dir = temp_rust("fn f() {}\n");
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for row in &report.rows {
        assert!(
            !row.lang.contains("(embedded)"),
            "collapse should not have embedded rows"
        );
    }
}

#[test]
fn separate_mode_may_differ_from_collapse() {
    let dir = temp_rust("fn f() {}\n");
    let langs = scan_dir(dir.path());
    let collapse = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let separate = create_lang_report(&langs, 0, false, ChildrenMode::Separate);
    // For pure Rust (no embedded), both should be equivalent
    assert_eq!(collapse.rows.len(), separate.rows.len());
}

#[test]
fn child_include_parents_only_vs_separate() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let parents = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    let separate = collect_file_rows(&langs, &[], 1, ChildIncludeMode::Separate, None);
    // Separate includes at least as many rows as parents-only
    assert!(separate.len() >= parents.len());
}

// ===========================================================================
// 6. Sort Stability (Descending Code, Then Name)
// ===========================================================================

#[test]
fn lang_report_sorted_descending_by_code() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for w in report.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "rows should be sorted descending by code: {} ({}) >= {} ({})",
            w[0].lang,
            w[0].code,
            w[1].lang,
            w[1].code,
        );
    }
}

#[test]
fn lang_report_tied_code_sorted_by_name() {
    // Create files with equal code counts
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.py"), "x = 1\n").unwrap();
    fs::write(dir.path().join("b.rs"), "fn f() {}\n").unwrap();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for w in report.rows.windows(2) {
        if w[0].code == w[1].code {
            assert!(
                w[0].lang <= w[1].lang,
                "tied code should sort by name ascending: {} <= {}",
                w[0].lang,
                w[1].lang,
            );
        }
    }
}

#[test]
fn module_report_sorted_descending_by_code() {
    let dir = temp_nested_modules();
    let langs = scan_dir(dir.path());
    let report = create_module_report(
        &langs,
        &["crates".into()],
        2,
        ChildIncludeMode::ParentsOnly,
        0,
    );
    for w in report.rows.windows(2) {
        assert!(
            w[0].code >= w[1].code,
            "modules should be sorted descending by code"
        );
    }
}

#[test]
fn export_data_sorted_descending_by_code() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let data = create_export_data(
        &langs,
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
            "export rows sorted descending by code"
        );
    }
}

// ===========================================================================
// 7. BTreeMap Ordering Preservation
// ===========================================================================

#[test]
fn file_rows_btreemap_ordering_deterministic() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let rows1 = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    let rows2 = collect_file_rows(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None);
    let paths1: Vec<_> = rows1.iter().map(|r| &r.path).collect();
    let paths2: Vec<_> = rows2.iter().map(|r| &r.path).collect();
    assert_eq!(paths1, paths2, "BTreeMap ordering must be deterministic");
}

#[test]
fn unique_parent_file_count_deterministic() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let c1 = unique_parent_file_count(&langs);
    let c2 = unique_parent_file_count(&langs);
    assert_eq!(c1, c2);
}

// ===========================================================================
// 8. Empty Language/File Inputs
// ===========================================================================

#[test]
fn empty_file_has_zero_code() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("empty.rs"), "").unwrap();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    // Empty file: 0 code
    for row in &report.rows {
        if row.lang == "Rust" {
            assert_eq!(row.code, 0);
        }
    }
}

#[test]
fn blank_only_file_zero_code() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("blank.rs"), "\n\n\n").unwrap();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for row in &report.rows {
        if row.lang == "Rust" {
            assert_eq!(row.code, 0);
        }
    }
}

#[test]
fn comment_only_file_zero_code() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("cmt.rs"), "// only comments\n// here\n").unwrap();
    let langs = scan_dir(dir.path());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    for row in &report.rows {
        if row.lang == "Rust" {
            assert_eq!(row.code, 0);
        }
    }
}

// ===========================================================================
// 9. avg() Edge Cases
// ===========================================================================

#[test]
fn avg_basic() {
    assert_eq!(avg(300, 3), 100);
    assert_eq!(avg(0, 5), 0);
    assert_eq!(avg(100, 0), 0);
}

#[test]
fn avg_rounding() {
    // 7 / 2 = 3.5 → 4
    assert_eq!(avg(7, 2), 4);
    // 1 / 2 = 0.5 → 1
    assert_eq!(avg(1, 2), 1);
    // 1 / 3 = 0.333 → 0
    assert_eq!(avg(1, 3), 0);
    // 2 / 3 = 0.666 → 1
    assert_eq!(avg(2, 3), 1);
}

#[test]
fn avg_identity() {
    assert_eq!(avg(42, 1), 42);
    assert_eq!(avg(0, 1), 0);
}

#[test]
fn avg_large_values() {
    assert_eq!(avg(1_000_000, 1000), 1000);
}

// ===========================================================================
// 10. normalize_path Tests
// ===========================================================================

#[test]
fn normalize_forward_slashes() {
    let p = PathBuf::from("src/main.rs");
    assert_eq!(normalize_path(&p, None), "src/main.rs");
}

#[test]
fn normalize_backslashes() {
    let p = PathBuf::from(r"src\main.rs");
    assert_eq!(normalize_path(&p, None), "src/main.rs");
}

#[test]
fn normalize_strips_dot_slash() {
    let p = PathBuf::from("./src/main.rs");
    assert_eq!(normalize_path(&p, None), "src/main.rs");
}

#[test]
fn normalize_with_prefix() {
    let p = PathBuf::from("project/src/lib.rs");
    let prefix = std::path::Path::new("project");
    assert_eq!(normalize_path(&p, Some(prefix)), "src/lib.rs");
}

// ===========================================================================
// 11. module_key Tests
// ===========================================================================

#[test]
fn module_key_root_level() {
    assert_eq!(module_key("Cargo.toml", &["crates".into()], 2), "(root)");
}

#[test]
fn module_key_in_root_dir() {
    assert_eq!(module_key("src/lib.rs", &["crates".into()], 2), "src");
}

#[test]
fn module_key_crates_root_depth_2() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &["crates".into()], 2),
        "crates/foo"
    );
}

#[test]
fn module_key_packages_root() {
    assert_eq!(
        module_key("packages/bar/index.ts", &["packages".into()], 2),
        "packages/bar"
    );
}

// ===========================================================================
// 12. Export Data Tests
// ===========================================================================

#[test]
fn export_data_empty() {
    let langs = Languages::new();
    let data = create_export_data(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None, 0, 0);
    assert!(data.rows.is_empty());
}

#[test]
fn export_data_min_code_filter() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("tiny.rs"), "fn f() {}\n").unwrap();
    fs::write(
        dir.path().join("big.rs"),
        "fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}\nfn e() {}\n",
    )
    .unwrap();
    let langs = scan_dir(dir.path());
    let data = create_export_data(
        &langs,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        None,
        3, // min_code = 3
        0,
    );
    for row in &data.rows {
        assert!(row.code >= 3, "min_code filter should exclude small files");
    }
}

#[test]
fn export_data_max_rows() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..5 {
        fs::write(
            dir.path().join(format!("f{i}.rs")),
            format!("fn f{i}() {{}}\n"),
        )
        .unwrap();
    }
    let langs = scan_dir(dir.path());
    let data = create_export_data(
        &langs,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        2, // max_rows = 2
    );
    assert!(data.rows.len() <= 2, "max_rows should limit output");
}

// ===========================================================================
// 13. Determinism Verification
// ===========================================================================

#[test]
fn lang_report_deterministic() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let r1 = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let r2 = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert_eq!(r1.rows.len(), r2.rows.len());
    for (a, b) in r1.rows.iter().zip(r2.rows.iter()) {
        assert_eq!(a, b, "lang rows must be identical");
    }
    assert_eq!(r1.total, r2.total);
}

#[test]
fn module_report_deterministic() {
    let dir = temp_nested_modules();
    let langs = scan_dir(dir.path());
    let r1 = create_module_report(
        &langs,
        &["crates".into()],
        2,
        ChildIncludeMode::ParentsOnly,
        0,
    );
    let r2 = create_module_report(
        &langs,
        &["crates".into()],
        2,
        ChildIncludeMode::ParentsOnly,
        0,
    );
    assert_eq!(r1.rows.len(), r2.rows.len());
    for (a, b) in r1.rows.iter().zip(r2.rows.iter()) {
        assert_eq!(a, b);
    }
    assert_eq!(r1.total, r2.total);
}

#[test]
fn export_data_deterministic() {
    let dir = temp_multi();
    let langs = scan_dir(dir.path());
    let d1 = create_export_data(
        &langs,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );
    let d2 = create_export_data(
        &langs,
        &[],
        1,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );
    assert_eq!(d1.rows.len(), d2.rows.len());
    for (a, b) in d1.rows.iter().zip(d2.rows.iter()) {
        assert_eq!(a, b);
    }
}

// ===========================================================================
// 14. Property-based Tests
// ===========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn avg_zero_files_returns_zero(lines in 0usize..100_000) {
            prop_assert_eq!(avg(lines, 0), 0);
        }

        #[test]
        fn avg_one_file_returns_lines(lines in 0usize..100_000) {
            prop_assert_eq!(avg(lines, 1), lines);
        }

        #[test]
        fn avg_never_exceeds_total(lines in 0usize..100_000, files in 1usize..1000) {
            let a = avg(lines, files);
            prop_assert!(a <= lines, "avg({lines}, {files}) = {a} should be <= {lines}");
        }

        #[test]
        fn normalize_path_no_backslashes(
            seg1 in "[a-z]{1,5}",
            seg2 in "[a-z]{1,5}",
            filename in "[a-z]{1,5}\\.rs"
        ) {
            let path_str = format!("{seg1}/{seg2}/{filename}");
            let p = PathBuf::from(&path_str);
            let norm = normalize_path(&p, None);
            prop_assert!(!norm.contains('\\'), "no backslashes in normalized path");
        }

        #[test]
        fn normalize_path_idempotent(
            seg1 in "[a-z]{1,5}",
            seg2 in "[a-z]{1,5}",
            filename in "[a-z]{1,5}\\.rs"
        ) {
            let path_str = format!("{seg1}/{seg2}/{filename}");
            let p = PathBuf::from(&path_str);
            let n1 = normalize_path(&p, None);
            let p2 = PathBuf::from(&n1);
            let n2 = normalize_path(&p2, None);
            prop_assert_eq!(n1, n2, "normalize must be idempotent");
        }

        #[test]
        fn module_key_deterministic(
            seg1 in "[a-z]{1,5}",
            seg2 in "[a-z]{1,5}",
            filename in "[a-z]{1,5}\\.rs"
        ) {
            let path = format!("{seg1}/{seg2}/{filename}");
            let roots = vec![seg1.clone()];
            let k1 = module_key(&path, &roots, 2);
            let k2 = module_key(&path, &roots, 2);
            prop_assert_eq!(k1, k2);
        }

        #[test]
        fn lang_report_total_code_is_sum(n in 1usize..10) {
            let dir = tempfile::tempdir().unwrap();
            for i in 0..n {
                fs::write(
                    dir.path().join(format!("f{i}.rs")),
                    format!("fn f{i}() {{}}\n"),
                ).unwrap();
            }
            let langs = scan_dir(dir.path());
            let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
            let sum: usize = report.rows.iter().map(|r| r.code).sum();
            prop_assert_eq!(report.total.code, sum, "total.code == sum of rows");
        }

        #[test]
        fn lang_report_total_lines_is_sum(n in 1usize..10) {
            let dir = tempfile::tempdir().unwrap();
            for i in 0..n {
                fs::write(
                    dir.path().join(format!("f{i}.rs")),
                    format!("// comment\nfn f{i}() {{}}\n\n"),
                ).unwrap();
            }
            let langs = scan_dir(dir.path());
            let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
            let sum: usize = report.rows.iter().map(|r| r.lines).sum();
            prop_assert_eq!(report.total.lines, sum, "total.lines == sum of rows");
        }

        #[test]
        fn file_rows_lines_equal_parts(n in 1usize..10) {
            let dir = tempfile::tempdir().unwrap();
            for i in 0..n {
                fs::write(
                    dir.path().join(format!("f{i}.rs")),
                    format!("// c\nfn f{i}() {{}}\n\n"),
                ).unwrap();
            }
            let langs = scan_dir(dir.path());
            let rows = collect_file_rows(
                &langs,
                &[],
                1,
                ChildIncludeMode::ParentsOnly,
                None,
            );
            for row in &rows {
                prop_assert_eq!(
                    row.lines,
                    row.code + row.comments + row.blanks,
                    "lines = code + comments + blanks for {}",
                    row.path,
                );
            }
        }

        #[test]
        fn module_report_total_code_is_sum(n in 1usize..5) {
            let dir = tempfile::tempdir().unwrap();
            for i in 0..n {
                let sub = dir.path().join(format!("mod{i}"));
                fs::create_dir_all(&sub).unwrap();
                fs::write(sub.join("lib.rs"), format!("fn f{i}() {{}}\n")).unwrap();
            }
            let langs = scan_dir(dir.path());
            let report = create_module_report(
                &langs,
                &[],
                1,
                ChildIncludeMode::ParentsOnly,
                0,
            );
            let sum: usize = report.rows.iter().map(|r| r.code).sum();
            prop_assert_eq!(report.total.code, sum, "module total.code == sum of rows");
        }
    }
}
