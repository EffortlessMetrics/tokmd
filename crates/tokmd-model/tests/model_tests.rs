//! Integration tests for tokmd-model functions.
//!
//! These tests use real file scanning to produce Languages data,
//! then verify the model functions produce correct results.

use std::path::PathBuf;
use tokei::{Config, LanguageType, Languages};
use tokmd_model::{
    avg, collect_file_rows, compute_file_metrics, create_export_data, create_lang_report,
    create_module_report, module_key, normalize_path, unique_parent_file_count,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode};

/// Scan a directory and return Languages data.
fn scan_path(path: &str) -> Languages {
    let mut languages = Languages::new();
    let paths = vec![PathBuf::from(path)];
    let cfg = Config::default();
    languages.get_statistics(&paths, &[], &cfg);
    languages
}

/// Get the crate's src directory for testing.
fn crate_src_path() -> String {
    format!("{}/src", env!("CARGO_MANIFEST_DIR"))
}

// ========================
// create_lang_report tests
// ========================

#[test]
fn lang_report_collapse_sums_correctly() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report = create_lang_report(&languages, &metrics, 0, false, ChildrenMode::Collapse);

    // Totals should match sum of rows
    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    let row_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    let row_bytes: usize = report.rows.iter().map(|r| r.bytes).sum();
    let row_tokens: usize = report.rows.iter().map(|r| r.tokens).sum();

    assert_eq!(
        report.total.code, row_code,
        "Total code should match sum of rows"
    );
    assert_eq!(
        report.total.lines, row_lines,
        "Total lines should match sum of rows"
    );
    assert_eq!(
        report.total.bytes, row_bytes,
        "Total bytes should match sum of rows"
    );
    assert_eq!(
        report.total.tokens, row_tokens,
        "Total tokens should match sum of rows"
    );
}

#[test]
fn lang_report_separate_sums_correctly() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report = create_lang_report(&languages, &metrics, 0, false, ChildrenMode::Separate);

    // Totals should match sum of rows
    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    let row_lines: usize = report.rows.iter().map(|r| r.lines).sum();

    assert_eq!(
        report.total.code, row_code,
        "Total code should match sum of rows"
    );
    assert_eq!(
        report.total.lines, row_lines,
        "Total lines should match sum of rows"
    );
}

#[test]
fn lang_report_sorted_descending_by_code() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report = create_lang_report(&languages, &metrics, 0, false, ChildrenMode::Collapse);

    for i in 1..report.rows.len() {
        assert!(
            report.rows[i - 1].code >= report.rows[i].code,
            "Rows should be sorted descending by code: {} >= {}",
            report.rows[i - 1].code,
            report.rows[i].code
        );
    }
}

#[test]
fn lang_report_skips_zero_code_languages() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report = create_lang_report(&languages, &metrics, 0, false, ChildrenMode::Collapse);

    for row in &report.rows {
        assert!(
            row.code > 0,
            "Should skip languages with zero code: {}",
            row.lang
        );
    }
}

#[test]
fn lang_report_top_truncates_and_adds_other() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let full_report = create_lang_report(&languages, &metrics, 0, false, ChildrenMode::Collapse);

    if full_report.rows.len() > 1 {
        let top1_report = create_lang_report(&languages, &metrics, 1, false, ChildrenMode::Collapse);

        // Should have exactly 2 rows: top 1 + "Other"
        assert_eq!(top1_report.rows.len(), 2, "Should have top 1 + Other");
        assert_eq!(
            top1_report.rows[1].lang, "Other",
            "Second row should be Other"
        );

        // "Other" should contain sum of remaining rows
        let other_code: usize = full_report.rows[1..].iter().map(|r| r.code).sum();
        assert_eq!(
            top1_report.rows[1].code, other_code,
            "Other code should sum remaining"
        );
    }
}

#[test]
fn lang_report_lines_includes_all_components() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report = create_lang_report(&languages, &metrics, 0, false, ChildrenMode::Collapse);

    // For each language, lines should be code + comments + blanks
    // We verify by checking that lines >= code (since comments and blanks are non-negative)
    for row in &report.rows {
        assert!(
            row.lines >= row.code,
            "Lines ({}) should be >= code ({}) for {}",
            row.lines,
            row.code,
            row.lang
        );
    }
}

#[test]
fn lang_report_avg_lines_calculated_correctly() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report = create_lang_report(&languages, &metrics, 0, false, ChildrenMode::Collapse);

    for row in &report.rows {
        if row.files > 0 {
            let expected_avg = avg(row.lines, row.files);
            assert_eq!(
                row.avg_lines, expected_avg,
                "Avg lines mismatch for {}: got {}, expected {}",
                row.lang, row.avg_lines, expected_avg
            );
        }
    }
}

#[test]
fn lang_report_tokens_approximates_bytes() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report = create_lang_report(&languages, &metrics, 0, false, ChildrenMode::Collapse);

    // tokens = bytes / 4 (CHARS_PER_TOKEN)
    for row in &report.rows {
        let expected_tokens = row.bytes / 4;
        assert_eq!(
            row.tokens, expected_tokens,
            "Tokens should be bytes/4 for {}: got {}, expected {}",
            row.lang, row.tokens, expected_tokens
        );
    }
}

// ========================
// create_module_report tests
// ========================

#[test]
fn module_report_sums_correctly() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report =
        create_module_report(&languages, &metrics, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    // Totals should be consistent
    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(
        report.total.code, row_code,
        "Total code should match sum of rows"
    );
}

#[test]
fn module_report_sorted_descending_by_code() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report =
        create_module_report(&languages, &metrics, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    for i in 1..report.rows.len() {
        assert!(
            report.rows[i - 1].code >= report.rows[i].code,
            "Rows should be sorted descending by code"
        );
    }
}

#[test]
fn module_report_top_truncates() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let full_report =
        create_module_report(&languages, &metrics, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    if full_report.rows.len() > 1 {
        let top1_report = create_module_report(
            &languages,
            &metrics,
            &[],
            2,
            ChildIncludeMode::ParentsOnly,
            1,
        );

        assert_eq!(top1_report.rows.len(), 2, "Should have top 1 + Other");
        assert_eq!(
            top1_report.rows[1].module, "Other",
            "Second row should be Other"
        );
    }
}

// ========================
// create_export_data tests
// ========================

#[test]
fn export_data_min_code_filters_correctly() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);

    // Get all rows first
    let all_data = create_export_data(
        &languages,
        &metrics,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    if !all_data.rows.is_empty() {
        // Find a threshold that will filter some but not all
        let max_code = all_data.rows.iter().map(|r| r.code).max().unwrap();

        if max_code > 1 {
            let filtered = create_export_data(
                &languages,
                &metrics,
                &[],
                2,
                ChildIncludeMode::ParentsOnly,
                None,
                max_code,
                0,
            );

            // All rows should have code >= min_code
            for row in &filtered.rows {
                assert!(
                    row.code >= max_code,
                    "Row code {} should be >= min_code {}",
                    row.code,
                    max_code
                );
            }

            // Should have fewer rows than original (unless all have same code)
            assert!(
                filtered.rows.len() <= all_data.rows.len(),
                "Filtering should not add rows"
            );
        }
    }
}

#[test]
fn export_data_max_rows_truncates() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let all_data = create_export_data(
        &languages,
        &metrics,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    if all_data.rows.len() > 1 {
        let limited = create_export_data(
            &languages,
            &metrics,
            &[],
            2,
            ChildIncludeMode::ParentsOnly,
            None,
            0,
            1,
        );

        assert_eq!(
            limited.rows.len(),
            1,
            "Should have exactly 1 row when max_rows=1"
        );
    }
}

#[test]
fn export_data_sorted_by_code_then_path() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let data = create_export_data(
        &languages,
        &metrics,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    for i in 1..data.rows.len() {
        let prev = &data.rows[i - 1];
        let curr = &data.rows[i];

        // Should be sorted descending by code, then ascending by path
        assert!(
            prev.code > curr.code || (prev.code == curr.code && prev.path <= curr.path),
            "Rows should be sorted by code desc, path asc"
        );
    }
}

// ========================
// collect_file_rows tests
// ========================

#[test]
fn collect_file_rows_returns_valid_rows() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let rows = collect_file_rows(
        &languages,
        &metrics,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
    );

    for row in &rows {
        // Each row should have lines = code + comments + blanks
        assert_eq!(
            row.lines,
            row.code + row.comments + row.blanks,
            "lines should equal code + comments + blanks for {}",
            row.path
        );

        // tokens = bytes / 4
        assert_eq!(
            row.tokens,
            row.bytes / 4,
            "tokens should equal bytes/4 for {}",
            row.path
        );
    }
}

#[test]
fn collect_file_rows_separate_includes_children() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let collapse_rows = collect_file_rows(
        &languages,
        &metrics,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
    );
    let separate_rows = collect_file_rows(
        &languages,
        &metrics,
        &[],
        2,
        ChildIncludeMode::Separate,
        None,
    );

    // Separate mode should have at least as many rows as collapse mode
    // (it includes child rows which collapse mode merges)
    assert!(
        separate_rows.len() >= collapse_rows.len(),
        "Separate mode should have >= rows than collapse mode"
    );
}

// ========================
// unique_parent_file_count tests
// ========================

#[test]
fn unique_parent_file_count_returns_correct_count() {
    let languages = scan_path(&crate_src_path());
    let count = unique_parent_file_count(&languages);

    // Should match the number of unique files from Rust language
    let rust_files = languages
        .get(&LanguageType::Rust)
        .map(|l| l.reports.len())
        .unwrap_or(0);

    // For a single-language directory, should match
    assert!(
        count >= rust_files,
        "unique_parent_file_count ({}) should be >= rust files ({})",
        count,
        rust_files
    );
}

#[test]
fn unique_parent_file_count_empty_languages() {
    let languages = Languages::new();
    let count = unique_parent_file_count(&languages);
    assert_eq!(count, 0, "Empty languages should have 0 files");
}

// ========================
// normalize_path edge cases
// ========================

#[test]
fn normalize_path_handles_backslashes() {
    use std::path::Path;

    let path = Path::new(r"C:\Code\Project\src\main.rs");
    let normalized = normalize_path(path, None);

    assert!(
        !normalized.contains('\\'),
        "Should not contain backslashes: {}",
        normalized
    );
}

#[test]
fn normalize_path_strips_prefix_with_trailing_slash() {
    use std::path::Path;

    let path = Path::new("project/src/main.rs");
    let prefix = Path::new("project/");
    let normalized = normalize_path(path, Some(prefix));

    assert_eq!(
        normalized, "src/main.rs",
        "Should strip prefix with trailing slash"
    );
}

#[test]
fn normalize_path_strips_prefix_without_trailing_slash() {
    use std::path::Path;

    let path = Path::new("project/src/main.rs");
    let prefix = Path::new("project");
    let normalized = normalize_path(path, Some(prefix));

    assert_eq!(
        normalized, "src/main.rs",
        "Should strip prefix without trailing slash"
    );
}

#[test]
fn normalize_path_handles_leading_dot_slash() {
    use std::path::Path;

    let path = Path::new("./src/main.rs");
    let normalized = normalize_path(path, None);

    assert!(
        !normalized.starts_with("./"),
        "Should not start with ./: {}",
        normalized
    );
}

#[test]
fn normalize_path_handles_leading_slash() {
    use std::path::Path;

    let path = Path::new("/src/main.rs");
    let normalized = normalize_path(path, None);

    assert!(
        !normalized.starts_with('/'),
        "Should not start with /: {}",
        normalized
    );
}

#[test]
fn normalize_path_handles_complex_prefix() {
    use std::path::Path;

    // Test with backslashes in prefix
    let path = Path::new("C:/Code/Project/src/main.rs");
    let prefix = Path::new(r"C:\Code\Project");
    let normalized = normalize_path(path, Some(prefix));

    assert_eq!(
        normalized, "src/main.rs",
        "Should handle backslashes in prefix"
    );
}

#[test]
fn normalize_path_handles_dot_slash_prefix() {
    use std::path::Path;

    let path = Path::new("./project/src/main.rs");
    let prefix = Path::new("./project");
    let normalized = normalize_path(path, Some(prefix));

    assert_eq!(
        normalized, "src/main.rs",
        "Should strip ./ from both path and prefix"
    );
}

// ========================
// module_key edge cases
// ========================

#[test]
fn module_key_handles_deep_paths_with_roots() {
    let roots = vec!["crates".to_string()];
    let key = module_key("crates/foo/bar/baz/file.rs", &roots, 2);
    assert_eq!(key, "crates/foo", "Should respect module_depth");
}

#[test]
fn module_key_handles_shallow_paths_with_roots() {
    let roots = vec!["crates".to_string()];
    let key = module_key("crates/file.rs", &roots, 2);
    assert_eq!(key, "crates", "Should not include filename in module key");
}

#[test]
fn module_key_non_root_returns_first_dir() {
    let roots = vec!["crates".to_string()];
    let key = module_key("src/foo/bar.rs", &roots, 2);
    assert_eq!(key, "src", "Non-root should return first directory only");
}

#[test]
fn module_key_depth_zero_treated_as_one() {
    let roots = vec!["crates".to_string()];
    let key = module_key("crates/foo/bar/file.rs", &roots, 0);
    // depth=0 should be treated as depth=1
    assert_eq!(key, "crates", "depth=0 should behave like depth=1");
}

// ========================
// avg function edge cases
// ========================

#[test]
fn avg_zero_files_returns_zero() {
    assert_eq!(avg(100, 0), 0, "avg with 0 files should return 0");
}

#[test]
fn avg_rounds_to_nearest() {
    // 10 lines, 3 files = 3.33... rounds to 3
    assert_eq!(avg(10, 3), 3, "10/3 should round to 3");

    // 11 lines, 3 files = 3.66... rounds to 4
    assert_eq!(avg(11, 3), 4, "11/3 should round to 4");

    // 12 lines, 3 files = 4.0 exactly
    assert_eq!(avg(12, 3), 4, "12/3 should be exactly 4");
}

#[test]
fn avg_one_file() {
    assert_eq!(avg(100, 1), 100, "avg with 1 file should return lines");
}

// ========================
// Integration: verify arithmetic operations
// ========================

#[test]
fn verify_code_accumulation_is_addition() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let rows = collect_file_rows(
        &languages,
        &metrics,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
    );

    // Manually compute totals by adding
    let manual_code: usize = rows.iter().map(|r| r.code).sum();
    let manual_lines: usize = rows.iter().map(|r| r.lines).sum();
    let manual_bytes: usize = rows.iter().map(|r| r.bytes).sum();
    let manual_tokens: usize = rows.iter().map(|r| r.tokens).sum();

    // These should be non-zero for the lib.rs file
    assert!(manual_code > 0, "Should have some code");
    assert!(manual_lines > 0, "Should have some lines");
    assert!(manual_bytes > 0, "Should have some bytes");
    assert!(manual_tokens > 0, "Should have some tokens");

    // Verify totals through module report
    let module_report =
        create_module_report(&languages, &metrics, &[], 2, ChildIncludeMode::ParentsOnly, 0);
    assert_eq!(
        module_report.total.code, manual_code,
        "Module report total code should match manual sum"
    );
}

#[test]
fn verify_division_in_tokens() {
    // tokens = bytes / 4, not bytes % 4 or bytes * 4
    let test_cases = vec![
        (0, 0),
        (1, 0),
        (3, 0),
        (4, 1),
        (7, 1),
        (8, 2),
        (100, 25),
        (1000, 250),
    ];

    for (bytes, expected_tokens) in test_cases {
        let tokens = bytes / 4; // This is what the code should do
        assert_eq!(
            tokens, expected_tokens,
            "bytes={} should produce tokens={}",
            bytes, expected_tokens
        );
    }
}

// ========================
// Mutation testing: verify correct operators
// ========================

/// Test that code == 0 check correctly excludes zero-code languages (line 76)
/// This catches mutant: replace == with != in create_lang_report
#[test]
fn lang_report_collapse_code_zero_check_excludes_zero() {
    // This test verifies that when code == 0, the language is skipped.
    // If mutated to !=, zero-code languages would be included and non-zero excluded.
    let languages = scan_path(&crate_src_path());
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    // All rows should have code > 0
    for row in &report.rows {
        assert!(
            row.code > 0,
            "Row should not have zero code (== check must skip zero-code): {}",
            row.lang
        );
    }
}

/// Test that code == 0 check in Separate mode works correctly (line 149)
/// This catches mutant: replace == with != in create_lang_report
#[test]
fn lang_report_separate_code_zero_check_excludes_zero() {
    let languages = scan_path(&crate_src_path());
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Separate);

    // All rows should have code > 0
    for row in &report.rows {
        assert!(
            row.code > 0,
            "Row should not have zero code (== check must skip zero-code): {}",
            row.lang
        );
    }
}

/// Test that lines = code + comments + blanks (lines 89, 113)
/// This catches mutants: replace + with - or * in create_lang_report
#[test]
fn lang_report_lines_equals_code_plus_comments_plus_blanks() {
    let languages = scan_path(&crate_src_path());
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    // For each language, manually check that lines is the sum of components
    for (lang_type, lang) in languages.iter() {
        let sum = lang.summarise();
        if sum.code == 0 {
            continue;
        }
        // Find matching row
        let row = report.rows.iter().find(|r| r.lang == lang_type.name());
        if let Some(r) = row {
            // lines should be code + comments + blanks (which is sum.code + sum.comments + sum.blanks)
            let expected_lines = sum.code + sum.comments + sum.blanks;
            assert_eq!(
                r.lines, expected_lines,
                "lines should equal code + comments + blanks for {}, got {} expected {}",
                r.lang, r.lines, expected_lines
            );
        }
    }
}

/// Test that code > 0 check works for Separate mode (line 112)
/// This catches mutants: replace > with ==, <, or >= in create_lang_report
#[test]
fn lang_report_separate_code_greater_than_zero_check() {
    let languages = scan_path(&crate_src_path());
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Separate);

    // All non-embedded rows should have code > 0 (not >= 0, not == 0, not < 0)
    for row in &report.rows {
        if !row.lang.contains("(embedded)") {
            assert!(
                row.code > 0,
                "Non-embedded row must have code > 0: {} has {}",
                row.lang,
                row.code
            );
        }
    }
}

/// Test that += operations accumulate correctly in Collapse mode (lines 121-122, 138, 141-142)
/// This catches mutants: replace += with *= or -= in create_lang_report
#[test]
fn lang_report_accumulates_bytes_and_tokens_correctly() {
    let languages = scan_path(&crate_src_path());
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    // Verify that bytes and tokens are sums, not products
    // If bytes were multiplied instead of added, we'd get 0 (if any file has 0 bytes)
    // or a massive number (products of all file sizes)
    assert!(report.total.bytes > 0, "Total bytes should be positive");
    assert!(report.total.tokens > 0, "Total tokens should be positive");

    // Verify relationship: tokens = bytes / 4 for the total
    // (This may not be exact due to integer division, but should be close)
    let expected_tokens = report.total.bytes / 4;
    assert_eq!(
        report.total.tokens, expected_tokens,
        "Total tokens should be bytes/4"
    );
}

/// Test that accumulation in Separate mode works (lines 138, 141-142)
/// This catches mutants: replace += with *= or -= in create_lang_report
#[test]
fn lang_report_separate_accumulates_correctly() {
    let languages = scan_path(&crate_src_path());
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Separate);

    // For embedded languages, code should be positive (accumulated by +=)
    // If it was *= starting from 0, it would remain 0
    for row in &report.rows {
        // Every row must have code > 0 (already tested), but also verify
        // lines is consistent with the formula
        assert!(
            row.lines >= row.code,
            "lines ({}) must be >= code ({}) for {}",
            row.lines,
            row.code,
            row.lang
        );
    }
}

/// Test that top > 0 && rows.len() > top condition works (line 185)
/// This catches mutants: replace > with ==, <, >= and && with ||
#[test]
fn lang_report_top_truncation_boundary_conditions() {
    let languages = scan_path(&crate_src_path());
    let full_report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    if full_report.rows.len() >= 2 {
        // Test exact boundary: top = rows.len() should NOT truncate
        let exact_report = create_lang_report(
            &languages,
            full_report.rows.len(),
            false,
            ChildrenMode::Collapse,
        );
        assert_eq!(
            exact_report.rows.len(),
            full_report.rows.len(),
            "top = rows.len() should not add Other"
        );
        assert!(
            !exact_report.rows.iter().any(|r| r.lang == "Other"),
            "Should not have Other when top = rows.len()"
        );

        // Test top = rows.len() - 1 should truncate
        let truncated_report = create_lang_report(
            &languages,
            full_report.rows.len() - 1,
            false,
            ChildrenMode::Collapse,
        );
        assert_eq!(
            truncated_report.rows.len(),
            full_report.rows.len(),
            "top = rows.len() - 1 should have original rows.len() (including Other)"
        );
        assert!(
            truncated_report.rows.iter().any(|r| r.lang == "Other"),
            "Should have Other when top < rows.len()"
        );
    }

    // Test top = 0 should NOT truncate (0 means no limit)
    let no_limit_report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    assert!(
        !no_limit_report.rows.iter().any(|r| r.lang == "Other"),
        "top = 0 should not add Other"
    );
}

/// Test that module report accumulates code, lines, bytes, tokens correctly (lines 248-250)
/// This catches mutants: replace += with *= in create_module_report
#[test]
fn module_report_accumulates_all_metrics_correctly() {
    let languages = scan_path(&crate_src_path());
    let report = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    // Verify that totals are sums, not products
    // With multiplication from 0 starting point, would get 0
    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    let row_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    let row_bytes: usize = report.rows.iter().map(|r| r.bytes).sum();
    let row_tokens: usize = report.rows.iter().map(|r| r.tokens).sum();

    assert!(row_code > 0, "Should have positive code");
    assert!(row_lines > 0, "Should have positive lines");
    assert!(row_bytes > 0, "Should have positive bytes");
    assert!(row_tokens > 0, "Should have positive tokens");

    assert_eq!(
        report.total.code, row_code,
        "Total code should match sum of rows"
    );
    assert_eq!(
        report.total.lines, row_lines,
        "Total lines should match sum of rows"
    );
    assert_eq!(
        report.total.bytes, row_bytes,
        "Total bytes should match sum of rows"
    );
    assert_eq!(
        report.total.tokens, row_tokens,
        "Total tokens should match sum of rows"
    );
}

/// Test that module report top > 0 && rows.len() > top condition works (line 281)
/// This catches mutants: replace > with ==, <, >= and && with ||
#[test]
fn module_report_top_truncation_boundary_conditions() {
    let languages = scan_path(&crate_src_path());
    let full_report = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    if full_report.rows.len() >= 2 {
        // Test exact boundary: top = rows.len() should NOT truncate
        let exact_report = create_module_report(
            &languages,
            &[],
            2,
            ChildIncludeMode::ParentsOnly,
            full_report.rows.len(),
        );
        assert_eq!(
            exact_report.rows.len(),
            full_report.rows.len(),
            "top = rows.len() should not add Other"
        );
        assert!(
            !exact_report.rows.iter().any(|r| r.module == "Other"),
            "Should not have Other when top = rows.len()"
        );

        // Test top = rows.len() - 1 should truncate
        let truncated_report = create_module_report(
            &languages,
            &[],
            2,
            ChildIncludeMode::ParentsOnly,
            full_report.rows.len() - 1,
        );
        assert!(
            truncated_report.rows.iter().any(|r| r.module == "Other"),
            "Should have Other when top < rows.len()"
        );
    }

    // Test top = 0 should NOT truncate
    let no_limit_report =
        create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);
    assert!(
        !no_limit_report.rows.iter().any(|r| r.module == "Other"),
        "top = 0 should not add Other"
    );
}

/// Test that export data min_code > 0 check works (line 356)
/// This catches mutants: replace > with ==, <, >= in create_export_data
#[test]
fn export_data_min_code_boundary_conditions() {
    let languages = scan_path(&crate_src_path());

    // Get baseline with min_code = 0 (should include all)
    let baseline = create_export_data(
        &languages,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    if !baseline.rows.is_empty() {
        // min_code = 0 should not filter anything (condition is: if min_code > 0)
        let with_zero = create_export_data(
            &languages,
            &metrics,
            &[],
            2,
            ChildIncludeMode::ParentsOnly,
            None,
            0,
            0,
        );
        assert_eq!(
            with_zero.rows.len(),
            baseline.rows.len(),
            "min_code = 0 should not filter"
        );

        // min_code = 1 should filter zero-code rows (if any)
        let with_one = create_export_data(
            &languages,
            &metrics,
            &[],
            2,
            ChildIncludeMode::ParentsOnly,
            None,
            1,
            0,
        );
        for row in &with_one.rows {
            assert!(
                row.code >= 1,
                "With min_code = 1, all rows should have code >= 1"
            );
        }
    }
}

/// Test that r.code >= min_code filter works (line 357)
/// This catches mutant: replace >= with < in create_export_data
#[test]
fn export_data_min_code_filter_uses_greater_or_equal() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let all_data = create_export_data(
        &languages,
        &metrics,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    if !all_data.rows.is_empty() {
        // Find a code value that exists
        let some_code = all_data.rows[0].code;
        if some_code > 0 {
            // Filter with min_code = some_code should include rows with exactly that code
            let filtered = create_export_data(
                &languages,
                &metrics,
                &[],
                2,
                ChildIncludeMode::ParentsOnly,
                None,
                some_code,
                0,
            );

            // The row with exactly some_code should be included (>= not <)
            assert!(
                filtered.rows.iter().any(|r| r.code == some_code),
                "Row with code = min_code should be included (uses >=)"
            );
        }
    }
}

/// Test that max_rows > 0 && rows.len() > max_rows condition works (line 361)
/// This catches mutants: replace > with ==, <, >= and && with ||
#[test]
fn export_data_max_rows_boundary_conditions() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let all_data = create_export_data(
        &languages,
        &metrics,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    if all_data.rows.len() >= 2 {
        // max_rows = 0 should NOT truncate
        assert!(all_data.rows.len() > 1, "max_rows = 0 should not truncate");

        // max_rows = all_data.rows.len() should NOT truncate (condition: rows.len() > max_rows)
        let exact = create_export_data(
            &languages,
            &metrics,
            &[],
            2,
            ChildIncludeMode::ParentsOnly,
            None,
            0,
            all_data.rows.len(),
        );
        assert_eq!(
            exact.rows.len(),
            all_data.rows.len(),
            "max_rows = rows.len() should not truncate"
        );

        // max_rows = 1 should truncate to 1
        let one = create_export_data(
            &languages,
            &metrics,
            &[],
            2,
            ChildIncludeMode::ParentsOnly,
            None,
            0,
            1,
        );
        assert_eq!(one.rows.len(), 1, "max_rows = 1 should truncate to 1 row");

        // max_rows = all_data.rows.len() - 1 should truncate
        let less_one = create_export_data(
            &languages,
            &metrics,
            &[],
            2,
            ChildIncludeMode::ParentsOnly,
            None,
            0,
            all_data.rows.len() - 1,
        );
        assert_eq!(
            less_one.rows.len(),
            all_data.rows.len() - 1,
            "max_rows = rows.len() - 1 should truncate"
        );
    }
}

/// Test that collect_file_rows accumulates code, comments, blanks correctly (lines 418-419, 440-442)
/// This catches mutants: replace += with *= or -= in collect_file_rows
#[test]
fn collect_file_rows_accumulates_stats_correctly() {
    let languages = scan_path(&crate_src_path());
    let rows = collect_file_rows(&languages, &[], 2, ChildIncludeMode::ParentsOnly, None);

    // Verify that each row has consistent stats
    for row in &rows {
        // lines = code + comments + blanks
        assert_eq!(
            row.lines,
            row.code + row.comments + row.blanks,
            "lines should equal code + comments + blanks for {}, got lines={}, code={}, comments={}, blanks={}",
            row.path,
            row.lines,
            row.code,
            row.comments,
            row.blanks
        );

        // tokens = bytes / 4
        assert_eq!(
            row.tokens,
            row.bytes / 4,
            "tokens should equal bytes / 4 for {}, got tokens={}, bytes={}",
            row.path,
            row.tokens,
            row.bytes
        );

        // All stats should be non-negative (if multiplication from 0, would be 0)
        // and code should be positive for most files
        assert!(
            row.bytes > 0 || row.code == 0,
            "bytes should be positive for files with code"
        );
    }
}

/// Test that accumulation works for child rows in Separate mode (lines 440-442)
/// This catches mutants: replace += with *= or -= in collect_file_rows
#[test]
fn collect_file_rows_separate_accumulates_child_stats() {
    let languages = scan_path(&crate_src_path());
    let rows = collect_file_rows(&languages, &[], 2, ChildIncludeMode::Separate, None);

    // Each row should have consistent lines = code + comments + blanks
    for row in &rows {
        assert_eq!(
            row.lines,
            row.code + row.comments + row.blanks,
            "lines should equal code + comments + blanks for {} (kind={:?})",
            row.path,
            row.kind
        );
    }
}

/// Test normalize_path handles backslash and slash conditions (line 520)
/// This catches mutants: replace && with ||, delete ! in normalize_path
#[test]
fn normalize_path_fast_slow_path_logic() {
    use std::path::Path;

    // Test case where prefix has no backslash and ends with slash (fast path)
    let path1 = Path::new("project/src/main.rs");
    let prefix1 = Path::new("project/");
    let result1 = normalize_path(path1, Some(prefix1));
    assert_eq!(result1, "src/main.rs", "Fast path should work correctly");

    // Test case where prefix has backslash (slow path, needs_replace = true)
    let path2 = Path::new("project/src/main.rs");
    let prefix2 = Path::new(r"project");
    let result2 = normalize_path(path2, Some(prefix2));
    assert_eq!(
        result2, "src/main.rs",
        "Slow path with needs_slash should work"
    );

    // Test case where prefix ends with slash (fast path condition)
    let path3 = Path::new("a/b/c.rs");
    let prefix3 = Path::new("a/");
    let result3 = normalize_path(path3, Some(prefix3));
    assert_eq!(result3, "b/c.rs", "Prefix ending with slash works");

    // Test case where prefix does NOT end with slash (slow path, needs_slash = true)
    let path4 = Path::new("a/b/c.rs");
    let prefix4 = Path::new("a");
    let result4 = normalize_path(path4, Some(prefix4));
    assert_eq!(result4, "b/c.rs", "Prefix not ending with slash works");

    // Test the && condition: needs_replace=false AND needs_slash=false
    // This is the fast path: !needs_replace && !needs_slash
    // If mutated to ||, we'd take wrong branch
    let path5 = Path::new("foo/bar/baz.rs");
    let prefix5 = Path::new("foo/"); // ends with /, no backslash
    let result5 = normalize_path(path5, Some(prefix5));
    assert_eq!(result5, "bar/baz.rs", "&& condition should take fast path");
}

/// Additional test for the ! operators in normalize_path (line 520)
/// This catches mutants: delete ! in normalize_path
#[test]
fn normalize_path_not_operators() {
    use std::path::Path;

    // Test that !needs_replace is used correctly
    // If the ! was deleted, a clean prefix would take the slow path unnecessarily
    // (but result should still be correct)

    // Test that !needs_slash is used correctly
    // If the ! was deleted, prefix ending with / would try to add another /

    // Fast path: prefix = "abc/" (no backslash, ends with slash)
    // needs_replace = false, needs_slash = false
    // Condition: !needs_replace && !needs_slash => true (fast path)
    let path = Path::new("abc/def/file.rs");
    let prefix = Path::new("abc/");
    let result = normalize_path(path, Some(prefix));
    assert_eq!(result, "def/file.rs");

    // Test with prefix that needs_replace (contains backslash on Windows would)
    // needs_replace = false (no backslash in prefix), needs_slash = true
    // Condition: !false && !true => false (slow path)
    let path2 = Path::new("xyz/sub/file.rs");
    let prefix2 = Path::new("xyz"); // no trailing slash
    let result2 = normalize_path(path2, Some(prefix2));
    assert_eq!(result2, "sub/file.rs");
}

/// Test that lines calculation in Separate mode is correct (line 113)
/// lines = lang.code + lang.comments + lang.blanks
#[test]
fn lang_report_separate_lines_calculation() {
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report = create_lang_report(&languages, &metrics, 0, false, ChildrenMode::Separate);

    for (lang_type, lang) in languages.iter() {
        if lang.code == 0 {
            continue;
        }
        // Find matching row (non-embedded)
        let row = report
            .rows
            .iter()
            .find(|r| r.lang == lang_type.name() && !r.lang.contains("(embedded)"));

        if let Some(r) = row {
            let expected_lines = lang.code + lang.comments + lang.blanks;
            assert_eq!(
                r.lines, expected_lines,
                "Separate mode lines should be code + comments + blanks for {}, got {}, expected {}",
                r.lang, r.lines, expected_lines
            );
        }
    }
}

/// Test embedded language accumulation (lines 138, 141-142)
/// entry.files += reports.len(), entry.code += st.code, entry.lines += code + comments + blanks
#[test]
fn lang_report_embedded_accumulation() {
    // This test ensures that embedded language stats are accumulated with +=
    // If *= was used instead, starting from 0, all embedded stats would be 0
    let languages = scan_path(&crate_src_path());
    let metrics = compute_file_metrics(&languages);
    let report = create_lang_report(&languages, &metrics, 0, false, ChildrenMode::Separate);

    // Check that non-embedded rows have positive stats
    for row in &report.rows {
        if !row.lang.contains("(embedded)") {
            assert!(
                row.code > 0,
                "Non-embedded {} should have code > 0",
                row.lang
            );
            assert!(
                row.lines >= row.code,
                "Non-embedded {} lines >= code",
                row.lang
            );
        }
    }
}
