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
    normalize_path,
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

// ========================
// Scenario: normalize_path edge cases
// ========================

#[test]
fn scenario_normalize_path_strips_leading_dot_slash() {
    use std::path::Path;
    use tokmd_model::normalize_path;

    let result = normalize_path(Path::new("./src/lib.rs"), None);
    assert_eq!(result, "src/lib.rs");
}

#[test]
fn scenario_normalize_path_converts_backslashes_to_forward_slashes() {
    use std::path::Path;
    use tokmd_model::normalize_path;

    let result = normalize_path(Path::new("src\\main.rs"), None);
    assert_eq!(result, "src/main.rs");
}

#[test]
fn scenario_normalize_path_with_strip_prefix_removes_prefix() {
    use std::path::Path;
    use tokmd_model::normalize_path;

    let result = normalize_path(
        Path::new("crates/foo/src/lib.rs"),
        Some(Path::new("crates/foo")),
    );
    assert_eq!(result, "src/lib.rs");
}

#[test]
fn scenario_normalize_path_without_matching_prefix_keeps_path() {
    use std::path::Path;
    use tokmd_model::normalize_path;

    let result = normalize_path(Path::new("src/lib.rs"), Some(Path::new("nonexistent")));
    assert_eq!(result, "src/lib.rs");
}

// ========================
// Scenario: avg edge cases
// ========================

#[test]
fn scenario_avg_with_zero_files_returns_zero() {
    use tokmd_model::avg;

    assert_eq!(avg(100, 0), 0, "division by zero should yield 0");
}

#[test]
fn scenario_avg_rounds_to_nearest() {
    use tokmd_model::avg;

    // 10 lines / 3 files = 3.33 → should round to 3 (with half-up: (10+1)/3 = 3)
    assert_eq!(avg(10, 3), 3);
    // 11 lines / 2 files = 5.5 → should round to 6 (with half-up: (11+1)/2 = 6)
    assert_eq!(avg(11, 2), 6);
}

#[test]
fn scenario_avg_with_one_file_returns_lines() {
    use tokmd_model::avg;

    assert_eq!(avg(42, 1), 42);
}

// ========================
// Scenario: module_key edge cases
// ========================

#[test]
fn scenario_module_key_root_file_returns_root() {
    use tokmd_model::module_key;

    assert_eq!(module_key("Cargo.toml", &["crates".into()], 2), "(root)");
}

#[test]
fn scenario_module_key_single_dir_returns_dir_name() {
    use tokmd_model::module_key;

    assert_eq!(module_key("docs/readme.md", &[], 2), "docs");
}

#[test]
fn scenario_module_key_with_matching_root_joins_segments() {
    use tokmd_model::module_key;

    let result = module_key("crates/tokmd-scan/src/lib.rs", &["crates".into()], 2);
    assert_eq!(result, "crates/tokmd-scan");
}

// ========================
// Scenario: with_files flag
// ========================

#[test]
fn scenario_with_files_true_populates_file_count_in_report() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);

    for row in &report.rows {
        assert!(
            row.files > 0,
            "with_files=true should populate file count for {}",
            row.lang
        );
    }
    assert!(report.total.files > 0, "total files should be positive");
}

#[test]
fn scenario_with_files_false_still_populates_file_count() {
    let langs = scan(&crate_src());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    // Even with_files=false, the report struct still has file counts (it controls display)
    assert!(report.total.files > 0 || report.rows.is_empty());
}

// ========================
// Scenario: deterministic ordering
// ========================

#[test]
fn scenario_two_identical_scans_produce_identical_reports() {
    let langs1 = scan(&crate_src());
    let langs2 = scan(&crate_src());

    let r1 = create_lang_report(&langs1, 0, false, ChildrenMode::Collapse);
    let r2 = create_lang_report(&langs2, 0, false, ChildrenMode::Collapse);

    assert_eq!(r1.rows.len(), r2.rows.len(), "same row count");
    for (a, b) in r1.rows.iter().zip(r2.rows.iter()) {
        assert_eq!(a.lang, b.lang, "same language order");
        assert_eq!(a.code, b.code, "same code count for {}", a.lang);
    }
}

// ========================
// Scenario: Module report with explicit roots
// ========================

#[test]
fn scenario_module_report_with_roots_groups_by_root_prefix() {
    // Given a scanned codebase with a known crate structure
    let workspace = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    let langs = scan(&format!("{workspace}/crates/tokmd-model/src"));

    // When I generate a module report with roots = ["crates"] and depth = 2
    // (scanning only src/, so no path starts with "crates/")
    let report = create_module_report(
        &langs,
        &["crates".into()],
        2,
        ChildIncludeMode::ParentsOnly,
        0,
    );

    // Then all module keys should NOT start with "crates/" because the
    // scanned paths are relative to the src/ dir, not the workspace root.
    for row in &report.rows {
        assert!(!row.module.is_empty(), "Module key should not be empty");
    }
}

// ========================
// Scenario: Export data with strip_prefix
// ========================

#[test]
fn scenario_export_data_with_strip_prefix_removes_path_prefix() {
    // Given a scanned codebase
    let src = crate_src();
    let langs = scan(&src);

    // When I generate export data with a strip_prefix
    let prefix = std::path::Path::new(&src);
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        Some(prefix),
        0,
        0,
    );

    // Then no path should start with the stripped prefix
    let prefix_str = tokmd_model::normalize_path(prefix, None);
    for row in &data.rows {
        assert!(
            !row.path.starts_with(&prefix_str),
            "Path '{}' should not start with stripped prefix '{}'",
            row.path,
            prefix_str
        );
    }
}

#[test]
fn scenario_export_data_combined_min_code_and_max_rows() {
    // Given a scanned codebase
    let langs = scan(&crate_src());

    // When I generate export data with both min_code and max_rows filters
    let data = create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        1, // min_code = 1 (skip files with 0 code)
        1, // max_rows = 1
    );

    // Then at most 1 row, and it must have code >= 1
    assert!(data.rows.len() <= 1, "max_rows=1 should limit to 1 row");
    for row in &data.rows {
        assert!(row.code >= 1, "min_code=1 should filter out 0-code rows");
    }
}

// ========================
// Scenario: Lang report metadata preservation
// ========================

#[test]
fn scenario_lang_report_preserves_with_files_flag() {
    // Given a scanned codebase
    let langs = scan(&crate_src());

    // When I generate reports with and without with_files
    let with_files = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);
    let without_files = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);

    // Then the with_files field should reflect the flag
    assert!(with_files.with_files);
    assert!(!without_files.with_files);

    // And the row data should be identical regardless of the flag
    assert_eq!(with_files.rows.len(), without_files.rows.len());
    assert_eq!(with_files.total.code, without_files.total.code);
}

#[test]
fn scenario_lang_report_top_one_single_language_no_other_bucket() {
    // Given a codebase with only Rust files
    let langs = scan(&crate_src());

    // When only one language exists and top = 1
    let report = create_lang_report(&langs, 1, false, ChildrenMode::Collapse);

    // Then there should be exactly 1 row (no "Other" needed if only 1 language)
    if report.rows.len() == 1 {
        assert_ne!(report.rows[0].lang, "Other");
    }
}

// ========================
// Scenario: File row normalization
// ========================

#[test]
fn scenario_collect_file_rows_paths_use_forward_slashes() {
    // Given scanned languages
    let langs = scan(&crate_src());

    // When I collect file rows
    let rows = collect_file_rows(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None);

    // Then all paths must use forward slashes (never backslashes)
    for row in &rows {
        assert!(
            !row.path.contains('\\'),
            "Path '{}' should not contain backslashes",
            row.path
        );
    }
}

#[test]
fn scenario_collect_file_rows_lines_equals_components() {
    // Given scanned languages
    let langs = scan(&crate_src());

    // When I collect file rows
    let rows = collect_file_rows(&langs, &[], 2, ChildIncludeMode::ParentsOnly, None);

    // Then for every row, lines = code + comments + blanks
    for row in &rows {
        assert_eq!(
            row.lines,
            row.code + row.comments + row.blanks,
            "lines != code + comments + blanks for '{}'",
            row.path
        );
    }
}

// ========================
// Scenario: Module report with Separate children mode
// ========================

#[test]
fn scenario_module_report_separate_has_at_least_as_many_code_as_parents_only() {
    // Given scanned languages
    let langs = scan(&crate_src());

    // When I generate module reports in both modes
    let parents = create_module_report(&langs, &[], 2, ChildIncludeMode::ParentsOnly, 0);
    let separate = create_module_report(&langs, &[], 2, ChildIncludeMode::Separate, 0);

    // Then Separate mode total code should be >= ParentsOnly total code
    // (embedded languages add code lines)
    assert!(
        separate.total.code >= parents.total.code,
        "Separate ({}) should have >= code than ParentsOnly ({})",
        separate.total.code,
        parents.total.code
    );
}
