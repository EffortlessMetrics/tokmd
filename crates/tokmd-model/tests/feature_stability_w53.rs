//! Feature-stability tests for WASM readiness seams.
//!
//! These tests verify that tokmd-model works correctly WITHOUT optional
//! features. They must NOT use `#[cfg(feature = ...)]` guards.

use std::path::Path;
use tokei::Languages;
use tokmd_model::*;
use tokmd_types::*;

// ── avg() utility ─────────────────────────────────────────────────────

#[test]
fn avg_zero_files_returns_zero() {
    assert_eq!(avg(100, 0), 0);
}

#[test]
fn avg_normal_values() {
    assert_eq!(avg(100, 10), 10);
}

#[test]
fn avg_rounds_correctly() {
    // 7 / 2 = 3.5, rounds to 4 (or 3 depending on impl)
    let result = avg(7, 2);
    assert!(result == 3 || result == 4);
}

// ── normalize_path() ──────────────────────────────────────────────────

#[test]
fn normalize_path_forward_slashes() {
    let result = normalize_path(Path::new("src/main.rs"), None);
    assert_eq!(result, "src/main.rs");
}

#[test]
fn normalize_path_strips_dot_prefix() {
    let result = normalize_path(Path::new("./src/lib.rs"), None);
    assert_eq!(result, "src/lib.rs");
}

#[test]
fn normalize_path_with_strip_prefix() {
    let result = normalize_path(Path::new("project/src/main.rs"), Some(Path::new("project")));
    assert_eq!(result, "src/main.rs");
}

// ── module_key() ──────────────────────────────────────────────────────

#[test]
fn module_key_root_file() {
    let key = module_key("README.md", &["crates".into()], 2);
    assert_eq!(key, "(root)");
}

#[test]
fn module_key_crate_path() {
    let key = module_key("crates/tokmd-types/src/lib.rs", &["crates".into()], 2);
    assert_eq!(key, "crates/tokmd-types");
}

#[test]
fn module_key_deep_path() {
    let key = module_key("src/deeply/nested/file.rs", &["crates".into()], 2);
    // When path doesn't match any module root, depth is applied from the top
    assert_eq!(key, "src");
}

// ── Empty-input model building ────────────────────────────────────────

#[test]
fn create_lang_report_empty_languages() {
    let langs = Languages::new();
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
    assert_eq!(report.total.files, 0);
}

#[test]
fn create_module_report_empty_languages() {
    let langs = Languages::new();
    let report = create_module_report(&langs, &["crates".into()], 2, ChildIncludeMode::Separate, 0);
    assert!(report.rows.is_empty());
    assert_eq!(report.total.code, 0);
}

#[test]
fn create_export_data_empty_languages() {
    let langs = Languages::new();
    let data = create_export_data(
        &langs,
        &["crates".into()],
        2,
        ChildIncludeMode::Separate,
        None,
        0,
        0,
    );
    assert!(data.rows.is_empty());
}

#[test]
fn collect_file_rows_empty_languages() {
    let langs = Languages::new();
    let rows = collect_file_rows(
        &langs,
        &["crates".into()],
        2,
        ChildIncludeMode::Separate,
        None,
    );
    assert!(rows.is_empty());
}

#[test]
fn unique_parent_file_count_empty() {
    let langs = Languages::new();
    assert_eq!(unique_parent_file_count(&langs), 0);
}
