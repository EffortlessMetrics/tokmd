//! Determinism regression tests for tokmd-model – wave 40.
//!
//! Verifies that aggregation and sorting logic produce identical, stable
//! results for the same input across repeated invocations.
//!
//! Run with: `cargo test -p tokmd-model --test determinism_w40`

use std::path::PathBuf;
use tokei::{Config, Languages};
use tokmd_model::{
    collect_file_rows, create_export_data, create_lang_report, create_module_report, module_key,
    normalize_path,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode};

/// Scan the crate's own source directory for reproducible input.
fn scan_self() -> Languages {
    let mut languages = Languages::new();
    let path = format!("{}/src", env!("CARGO_MANIFEST_DIR"));
    languages.get_statistics(&[PathBuf::from(path)], &[], &Config::default());
    languages
}

// ===========================================================================
// 1. Language aggregation order is deterministic
// ===========================================================================

#[test]
fn lang_aggregation_order_deterministic() {
    let languages = scan_self();
    let a = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let b = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    assert_eq!(a.rows.len(), b.rows.len(), "row count must be stable");
    for (ra, rb) in a.rows.iter().zip(b.rows.iter()) {
        assert_eq!(ra.lang, rb.lang, "language name must be identical");
        assert_eq!(ra.code, rb.code, "code count must be identical");
        assert_eq!(ra.lines, rb.lines, "lines count must be identical");
        assert_eq!(ra.files, rb.files, "files count must be identical");
    }
}

#[test]
fn lang_aggregation_sorted_desc_code_asc_name() {
    let languages = scan_self();
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    for pair in report.rows.windows(2) {
        assert!(
            pair[0].code > pair[1].code
                || (pair[0].code == pair[1].code && pair[0].lang <= pair[1].lang),
            "lang sort violated: {}({}) before {}({})",
            pair[0].lang,
            pair[0].code,
            pair[1].lang,
            pair[1].code
        );
    }
}

#[test]
fn lang_aggregation_separate_mode_deterministic() {
    let languages = scan_self();
    let a = create_lang_report(&languages, 0, false, ChildrenMode::Separate);
    let b = create_lang_report(&languages, 0, false, ChildrenMode::Separate);

    let a_names: Vec<&str> = a.rows.iter().map(|r| r.lang.as_str()).collect();
    let b_names: Vec<&str> = b.rows.iter().map(|r| r.lang.as_str()).collect();
    assert_eq!(a_names, b_names, "separate mode row names must match");
}

// ===========================================================================
// 2. Module key ordering is deterministic
// ===========================================================================

#[test]
fn module_key_ordering_deterministic() {
    let languages = scan_self();
    let a = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);
    let b = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    let a_mods: Vec<&str> = a.rows.iter().map(|r| r.module.as_str()).collect();
    let b_mods: Vec<&str> = b.rows.iter().map(|r| r.module.as_str()).collect();
    assert_eq!(a_mods, b_mods, "module key order must be identical");
}

#[test]
fn module_rows_sorted_desc_code_asc_name() {
    let languages = scan_self();
    let report = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    for pair in report.rows.windows(2) {
        assert!(
            pair[0].code > pair[1].code
                || (pair[0].code == pair[1].code && pair[0].module <= pair[1].module),
            "module sort violated: {}({}) before {}({})",
            pair[0].module,
            pair[0].code,
            pair[1].module,
            pair[1].code
        );
    }
}

#[test]
fn module_keys_use_forward_slashes() {
    let languages = scan_self();
    let report = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    for row in &report.rows {
        assert!(
            !row.module.contains('\\'),
            "module key contains backslash: {}",
            row.module
        );
    }
}

// ===========================================================================
// 3. File row ordering is deterministic (tie-break by path)
// ===========================================================================

#[test]
fn file_row_ordering_deterministic() {
    let languages = scan_self();
    let a = collect_file_rows(&languages, &[], 2, ChildIncludeMode::ParentsOnly, None);
    let b = collect_file_rows(&languages, &[], 2, ChildIncludeMode::ParentsOnly, None);

    assert_eq!(a.len(), b.len(), "file row count must be stable");
    for (ra, rb) in a.iter().zip(b.iter()) {
        assert_eq!(ra.path, rb.path, "file path must be identical");
        assert_eq!(ra.code, rb.code, "code count must be identical");
        assert_eq!(ra.lang, rb.lang, "language must be identical");
    }
}

#[test]
fn file_rows_sorted_desc_code_asc_path() {
    let languages = scan_self();
    let rows = collect_file_rows(&languages, &[], 2, ChildIncludeMode::ParentsOnly, None);

    for pair in rows.windows(2) {
        assert!(
            pair[0].code > pair[1].code
                || (pair[0].code == pair[1].code && pair[0].path <= pair[1].path),
            "file sort violated: {}({}) before {}({})",
            pair[0].path,
            pair[0].code,
            pair[1].path,
            pair[1].code
        );
    }
}

#[test]
fn file_rows_use_forward_slashes() {
    let languages = scan_self();
    let rows = collect_file_rows(&languages, &[], 2, ChildIncludeMode::ParentsOnly, None);

    for row in &rows {
        assert!(
            !row.path.contains('\\'),
            "file path contains backslash: {}",
            row.path
        );
        assert!(
            !row.module.contains('\\'),
            "module key contains backslash: {}",
            row.module
        );
    }
}

// ===========================================================================
// 4. Export data determinism
// ===========================================================================

#[test]
fn export_data_row_count_stable() {
    let languages = scan_self();
    let a = create_export_data(
        &languages,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );
    let b = create_export_data(
        &languages,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );
    assert_eq!(
        a.rows.len(),
        b.rows.len(),
        "export row count must be stable"
    );
}

// ===========================================================================
// 5. normalize_path and module_key determinism
// ===========================================================================

#[test]
fn normalize_path_always_uses_forward_slashes() {
    let cases = [
        ("src\\lib.rs", "src/lib.rs"),
        ("src/lib.rs", "src/lib.rs"),
        ("a\\b\\c\\d.rs", "a/b/c/d.rs"),
        ("file.rs", "file.rs"),
    ];
    for (input, expected) in &cases {
        let result = normalize_path(std::path::Path::new(input), None);
        assert_eq!(
            &result, expected,
            "normalize_path({input}) should produce {expected}"
        );
    }
}

#[test]
fn module_key_deterministic_for_same_input() {
    let path = "src/core/lib.rs";
    let roots: Vec<String> = vec![];
    let a = module_key(path, &roots, 2);
    let b = module_key(path, &roots, 2);
    assert_eq!(a, b, "module_key must be deterministic");
    assert!(!a.contains('\\'), "module_key must not contain backslashes");
}
