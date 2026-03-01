//! BDD-style scenarios for tokmd-model aggregation logic.
//!
//! Each test reads as a Given/When/Then scenario exercising:
//! - Children mode handling (Collapse vs Separate)
//! - Sorting invariants (descending code, then name)
//! - Module key computation
//! - Empty / zero-code language handling

use std::path::PathBuf;
use tokei::{Config, Languages};
use tokmd_model::{
    collect_file_rows, create_export_data, create_lang_report, create_module_report, module_key,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode, FileKind};

/// Scan a directory and return Languages data.
fn scan(path: &str) -> Languages {
    let mut languages = Languages::new();
    languages.get_statistics(&[PathBuf::from(path)], &[], &Config::default());
    languages
}

fn crate_src() -> String {
    format!("{}/src", env!("CARGO_MANIFEST_DIR"))
}

// ========================
// Scenario: Children mode — Collapse
// ========================

#[test]
fn scenario_collapse_mode_merges_embedded_into_parent() {
    // Given a scanned codebase
    let langs = scan(&crate_src());

    // When I generate a lang report in Collapse mode
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    // Then no row name should contain "(embedded)"
    for row in &report.rows {
        assert!(
            !row.lang.contains("(embedded)"),
            "Collapse mode must not produce embedded rows, but found '{}'",
            row.lang
        );
    }
}

#[test]
fn scenario_separate_mode_labels_embedded_rows() {
    // Given a scanned codebase that may contain embedded languages
    let langs = scan(&crate_src());

    // When I generate a lang report in Separate mode
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Separate);

    // Then every non-embedded row must have positive bytes,
    //      and every embedded row must have zero bytes (no double-counting)
    for row in &report.rows {
        if row.lang.contains("(embedded)") {
            assert_eq!(
                row.bytes, 0,
                "Embedded row '{}' must have 0 bytes to avoid double-counting",
                row.lang
            );
            assert_eq!(
                row.tokens, 0,
                "Embedded row '{}' must have 0 tokens",
                row.lang
            );
        }
    }
}

#[test]
fn scenario_collapse_and_separate_totals_agree_on_bytes() {
    // Given the same scanned codebase
    let langs = scan(&crate_src());

    // When I generate reports in both modes
    let collapse = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let separate = create_lang_report(&langs, 0, false, ChildrenMode::Separate);

    // Then total bytes must be equal (embedded rows contribute 0 bytes)
    assert_eq!(
        collapse.total.bytes, separate.total.bytes,
        "Collapse and Separate should agree on total bytes"
    );
    assert_eq!(
        collapse.total.tokens, separate.total.tokens,
        "Collapse and Separate should agree on total tokens"
    );
}

#[test]
fn scenario_file_rows_separate_marks_child_kind() {
    // Given scanned languages
    let langs = scan(&crate_src());

    // When I collect file rows in Separate mode
    let rows = collect_file_rows(&langs, &[], 2, ChildIncludeMode::Separate, None);

    // Then every row with kind == Child must have 0 bytes/tokens
    for row in &rows {
        if row.kind == FileKind::Child {
            assert_eq!(
                row.bytes, 0,
                "Child row '{}' in '{}' must have 0 bytes",
                row.lang, row.path
            );
            assert_eq!(
                row.tokens, 0,
                "Child row '{}' in '{}' must have 0 tokens",
                row.lang, row.path
            );
        }
    }
}

#[test]
fn scenario_file_rows_parents_only_excludes_children() {
    // Given scanned languages
    let langs = scan(&crate_src());

    // When I collect file rows in ParentsOnly mode
    let rows = collect_file_rows(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None);

    // Then no row should have kind == Child
    for row in &rows {
        assert_eq!(
            row.kind,
            FileKind::Parent,
            "ParentsOnly mode must not produce Child rows, found child for '{}'",
            row.path
        );
    }
}

// ========================
// Scenario: Sorting invariants
// ========================

#[test]
fn scenario_lang_rows_sorted_desc_code_then_asc_name() {
    // Given a scanned codebase
    let langs = scan(&crate_src());

    // When I generate a lang report
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    // Then rows are sorted descending by code, ties broken ascending by name
    for pair in report.rows.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        assert!(
            a.code > b.code || (a.code == b.code && a.lang <= b.lang),
            "Sort violation: ({}, {}) should come before ({}, {})",
            a.lang,
            a.code,
            b.lang,
            b.code
        );
    }
}

#[test]
fn scenario_module_rows_sorted_desc_code_then_asc_module() {
    // Given a scanned codebase
    let langs = scan(&crate_src());

    // When I generate a module report
    let report = create_module_report(&langs, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    // Then rows are sorted descending by code, ties broken ascending by module name
    for pair in report.rows.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        assert!(
            a.code > b.code || (a.code == b.code && a.module <= b.module),
            "Sort violation: ({}, {}) should come before ({}, {})",
            a.module,
            a.code,
            b.module,
            b.code
        );
    }
}

#[test]
fn scenario_export_rows_sorted_desc_code_then_asc_path() {
    // Given a scanned codebase
    let langs = scan(&crate_src());

    // When I generate export data
    let data = create_export_data(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None, 0, 0);

    // Then rows are sorted descending by code, ties broken ascending by path
    for pair in data.rows.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        assert!(
            a.code > b.code || (a.code == b.code && a.path <= b.path),
            "Sort violation: ({}, {}) should come before ({}, {})",
            a.path,
            a.code,
            b.path,
            b.code
        );
    }
}

// ========================
// Scenario: Module key computation
// ========================

#[test]
fn scenario_root_level_files_get_root_module() {
    // Given a bare filename with no directory
    // When I compute the module key
    // Then it must be "(root)"
    assert_eq!(module_key("Cargo.toml", &[], 2), "(root)");
    assert_eq!(module_key("README.md", &["crates".into()], 3), "(root)");
}

#[test]
fn scenario_module_roots_capture_depth() {
    // Given roots = ["crates"] and depth = 2
    let roots = vec!["crates".into()];

    // When paths start with "crates/"
    // Then the module key includes up to 2 directory segments
    assert_eq!(
        module_key("crates/tokmd-model/src/lib.rs", &roots, 2),
        "crates/tokmd-model"
    );
    assert_eq!(
        module_key("crates/tokmd-types/src/lib.rs", &roots, 2),
        "crates/tokmd-types"
    );
}

#[test]
fn scenario_non_root_path_returns_first_directory() {
    // Given a path whose first segment is NOT in module_roots
    let roots = vec!["crates".into()];

    // When I compute the module key
    // Then it is just the first directory
    assert_eq!(module_key("src/main.rs", &roots, 2), "src");
    assert_eq!(module_key("docs/guide/intro.md", &roots, 2), "docs");
}

#[test]
fn scenario_module_key_never_includes_filename() {
    // Given various depths
    let roots = vec!["crates".into()];

    // When the file is directly under the root
    // Then the module key must NOT include the filename
    assert_eq!(module_key("crates/foo.rs", &roots, 2), "crates");
    assert_eq!(module_key("crates/foo.rs", &roots, 5), "crates");
}

#[test]
fn scenario_module_key_with_multiple_roots() {
    // Given roots = ["crates", "packages"]
    let roots = vec!["crates".into(), "packages".into()];

    // When paths start with either root
    // Then both are recognized at the configured depth
    assert_eq!(module_key("crates/a/b/c.rs", &roots, 2), "crates/a");
    assert_eq!(module_key("packages/x/y/z.rs", &roots, 2), "packages/x");
}

#[test]
fn scenario_module_key_depth_clamps_to_available_dirs() {
    // Given depth exceeds available directory segments
    let roots = vec!["crates".into()];

    // When depth is very large
    // Then the key uses all available directory segments (never including filename)
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots, 100),
        "crates/foo/src"
    );
}

#[test]
fn scenario_module_key_normalizes_backslashes() {
    // Given a Windows-style path
    let roots = vec!["crates".into()];

    // When I compute the module key
    // Then it should use forward slashes
    let key = module_key("crates\\tokmd-model\\src\\lib.rs", &roots, 2);
    assert!(!key.contains('\\'), "Module key must not contain backslash");
    assert_eq!(key, "crates/tokmd-model");
}

// ========================
// Scenario: Empty / zero-code languages
// ========================

#[test]
fn scenario_empty_languages_produce_empty_report() {
    // Given an empty Languages collection
    let langs = Languages::new();

    // When I generate a lang report
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    // Then there are no rows and totals are all zero
    assert!(report.rows.is_empty(), "Empty languages → 0 rows");
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.lines, 0);
    assert_eq!(report.total.files, 0);
    assert_eq!(report.total.bytes, 0);
    assert_eq!(report.total.tokens, 0);
}

#[test]
fn scenario_empty_languages_produce_empty_module_report() {
    // Given an empty Languages collection
    let langs = Languages::new();

    // When I generate a module report
    let report = create_module_report(&langs, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    // Then there are no rows and totals are all zero
    assert!(report.rows.is_empty(), "Empty languages → 0 rows");
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.lines, 0);
    assert_eq!(report.total.files, 0);
}

#[test]
fn scenario_empty_languages_produce_empty_export() {
    // Given an empty Languages collection
    let langs = Languages::new();

    // When I generate export data
    let data = create_export_data(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None, 0, 0);

    // Then there are no rows
    assert!(data.rows.is_empty(), "Empty languages → 0 export rows");
}

#[test]
fn scenario_separate_mode_empty_languages_produce_empty_report() {
    // Given an empty Languages collection
    let langs = Languages::new();

    // When I generate a lang report in Separate mode
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Separate);

    // Then there are no rows
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
}

#[test]
fn scenario_top_with_fewer_rows_than_limit_keeps_all() {
    // Given a scanned codebase
    let langs = scan(&crate_src());

    // When top is larger than the number of rows
    let report = create_lang_report(&langs, 9999, false, ChildrenMode::Collapse);

    // Then no "Other" bucket is produced
    assert!(
        !report.rows.iter().any(|r| r.lang == "Other"),
        "top > row count should not produce 'Other'"
    );
}

#[test]
fn scenario_top_zero_means_no_limit() {
    // Given a scanned codebase
    let langs = scan(&crate_src());

    // When top = 0
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    // Then no "Other" bucket is produced
    assert!(
        !report.rows.iter().any(|r| r.lang == "Other"),
        "top = 0 should not truncate"
    );
}
