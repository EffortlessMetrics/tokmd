//! Edge-case and boundary-condition tests for tokmd-model.

use std::path::{Path, PathBuf};
use tokei::{Config, Languages};
use tokmd_model::{
    collect_file_rows, create_lang_report, create_module_report, module_key, normalize_path,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode};

/// Scan a directory and return Languages data.
fn scan_dir(path: &str) -> Languages {
    let mut languages = Languages::new();
    languages.get_statistics(&[PathBuf::from(path)], &[], &Config::default());
    languages
}

fn scan_self_src() -> Languages {
    scan_dir(&format!("{}/src", env!("CARGO_MANIFEST_DIR")))
}

// ---------------------------------------------------------------------------
// Zero-code file
// ---------------------------------------------------------------------------

#[test]
fn single_file_zero_code_positive_blanks_comments() {
    let dir = tempfile::tempdir().unwrap();
    // A file with only comments and blanks, no code
    std::fs::write(
        dir.path().join("empty_code.rs"),
        "// comment\n// another comment\n\n\n\n",
    )
    .unwrap();
    let mut langs = Languages::new();
    langs.get_statistics(&[dir.path().to_path_buf()], &[], &Config::default());
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    if !report.rows.is_empty() {
        assert_eq!(report.rows[0].code, 0);
    }
    assert_eq!(report.total.code, 0);
}

// ---------------------------------------------------------------------------
// Module key for root-level file
// ---------------------------------------------------------------------------

#[test]
fn module_key_root_level_file_returns_root() {
    assert_eq!(module_key("Cargo.toml", &[], 2), "(root)");
    assert_eq!(module_key("README.md", &["crates".into()], 1), "(root)");
}

// ---------------------------------------------------------------------------
// Deeply nested module path
// ---------------------------------------------------------------------------

#[test]
fn module_key_deeply_nested_10_levels() {
    let deep = "a/b/c/d/e/f/g/h/i/j/deep.rs";
    // With no module roots, returns first segment
    assert_eq!(module_key(deep, &[], 2), "a");
}

#[test]
fn module_key_deeply_nested_with_root_respects_depth() {
    let deep = "crates/my-crate/src/inner/deep/mod.rs";
    assert_eq!(module_key(deep, &["crates".into()], 2), "crates/my-crate");
    assert_eq!(
        module_key(deep, &["crates".into()], 3),
        "crates/my-crate/src"
    );
}

// ---------------------------------------------------------------------------
// Empty Languages (no files scanned)
// ---------------------------------------------------------------------------

#[test]
fn lang_report_empty_languages() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.files, 0);
}

#[test]
fn module_report_empty_languages() {
    let langs = Languages::new();
    let report = create_module_report(&langs, &[], 2, ChildIncludeMode::Separate, 0);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
}

// ---------------------------------------------------------------------------
// Aggregation with real source
// ---------------------------------------------------------------------------

#[test]
fn aggregation_total_code_is_sum_of_rows() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let sum: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(sum, report.total.code);
}

#[test]
fn aggregation_top_limits_rows() {
    let langs = scan_self_src();
    let report = create_lang_report(&langs, 1, false, ChildrenMode::Collapse);
    // top=1 means at most 1 row (or 2 with "Other" if more langs exist)
    assert!(report.rows.len() <= 2);
}

// ---------------------------------------------------------------------------
// Children collapse with no children (noop)
// ---------------------------------------------------------------------------

#[test]
fn children_collapse_no_children_is_noop() {
    let langs = scan_self_src();
    let collapsed = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let separate = create_lang_report(&langs, 0, false, ChildrenMode::Separate);
    // Pure Rust crate has no embedded languages, both modes yield same code count
    assert_eq!(collapsed.total.code, separate.total.code);
}

// ---------------------------------------------------------------------------
// normalize_path edge cases
// ---------------------------------------------------------------------------

#[test]
fn normalize_path_strips_dot_prefix() {
    let result = normalize_path(Path::new("./src/lib.rs"), None);
    assert_eq!(result, "src/lib.rs");
}

#[test]
fn normalize_path_with_strip_prefix() {
    let result = normalize_path(Path::new("src/inner/file.rs"), Some(Path::new("src")));
    assert_eq!(result, "inner/file.rs");
}

// ---------------------------------------------------------------------------
// collect_file_rows root-level
// ---------------------------------------------------------------------------

#[test]
fn collect_file_rows_root_file_module_is_root() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    let mut langs = Languages::new();
    langs.get_statistics(&[dir.path().to_path_buf()], &[], &Config::default());
    let rows = collect_file_rows(&langs, &[], 2, ChildIncludeMode::Separate, Some(dir.path()));
    assert!(!rows.is_empty());
    assert_eq!(rows[0].module, "(root)");
}
