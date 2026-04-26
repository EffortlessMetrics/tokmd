//! Wave-42 deep tests for analysis utility helpers.
//!
//! Tests path normalization, depth counting, test-path detection,
//! infra-lang classification, and empty_file_row construction.

use std::path::PathBuf;

use crate::{empty_file_row, is_infra_lang, is_test_path, normalize_path, path_depth};

// ── 1. path_depth with nested paths ─────────────────────────────

#[test]
fn path_depth_counts_nested_segments() {
    assert_eq!(path_depth("a/b/c/d.rs"), 4);
    assert_eq!(path_depth("a.rs"), 1);
    assert_eq!(path_depth("a/b.rs"), 2);
}

// ── 2. path_depth minimum is 1 ─────────────────────────────────

#[test]
fn path_depth_minimum_one_for_empty() {
    assert_eq!(path_depth(""), 1);
}

// ── 3. path_depth ignores trailing slash ────────────────────────

#[test]
fn path_depth_trailing_slash() {
    assert_eq!(path_depth("a/b/"), 2);
    assert_eq!(path_depth("a/b/c/"), 3);
}

// ── 4. is_test_path detects test directory variants ─────────────

#[test]
fn is_test_path_directory_variants() {
    assert!(is_test_path("src/test/foo.rs"));
    assert!(is_test_path("src/tests/foo.rs"));
    assert!(is_test_path("src/__tests__/foo.rs"));
    assert!(is_test_path("src/spec/foo.rs"));
    assert!(is_test_path("src/specs/foo.rs"));
}

// ── 5. is_test_path detects file name patterns ──────────────────

#[test]
fn is_test_path_file_patterns() {
    assert!(is_test_path("src/foo_test.rs"));
    assert!(is_test_path("src/test_foo.rs"));
    assert!(is_test_path("src/foo.test.js"));
    assert!(is_test_path("src/foo.spec.ts"));
}

// ── 6. is_test_path rejects non-test paths ──────────────────────

#[test]
fn is_test_path_rejects_non_test() {
    assert!(!is_test_path("src/main.rs"));
    assert!(!is_test_path("src/lib.rs"));
    assert!(!is_test_path("src/contest/foo.rs")); // "contest" contains "test" but not as dir segment
}

// ── 7. is_infra_lang comprehensive ──────────────────────────────

#[test]
fn is_infra_lang_all_known() {
    let infra = [
        "json",
        "yaml",
        "toml",
        "markdown",
        "xml",
        "html",
        "css",
        "scss",
        "less",
        "makefile",
        "dockerfile",
        "hcl",
        "terraform",
        "nix",
        "cmake",
        "ini",
        "properties",
        "gitignore",
        "gitconfig",
        "editorconfig",
        "csv",
        "tsv",
        "svg",
    ];
    for lang in &infra {
        assert!(is_infra_lang(lang), "{} should be infra", lang);
    }
}

// ── 8. is_infra_lang rejects code languages ─────────────────────

#[test]
fn is_infra_lang_rejects_code() {
    let code = [
        "rust",
        "python",
        "javascript",
        "typescript",
        "go",
        "java",
        "c",
        "cpp",
    ];
    for lang in &code {
        assert!(!is_infra_lang(lang), "{} should not be infra", lang);
    }
}

// ── 9. normalize_path strips root prefix ────────────────────────

#[test]
fn normalize_path_strips_root() {
    let root = PathBuf::from("project");
    assert_eq!(normalize_path("project/src/main.rs", &root), "src/main.rs");
}

// ── 10. normalize_path handles backslashes ──────────────────────

#[test]
fn normalize_path_backslashes() {
    let root = PathBuf::from("project");
    assert_eq!(normalize_path(r"project\src\lib.rs", &root), "src/lib.rs");
}

// ── 11. empty_file_row has all zeros ────────────────────────────

#[test]
fn empty_file_row_zeroed() {
    let row = empty_file_row();
    assert_eq!(row.code, 0);
    assert_eq!(row.comments, 0);
    assert_eq!(row.blanks, 0);
    assert_eq!(row.lines, 0);
    assert_eq!(row.bytes, 0);
    assert_eq!(row.tokens, 0);
    assert_eq!(row.depth, 0);
    assert!(row.path.is_empty());
    assert!(row.module.is_empty());
    assert!(row.lang.is_empty());
}

// ── 12. is_infra_lang is case-insensitive ───────────────────────

#[test]
fn is_infra_lang_case_insensitive() {
    assert!(is_infra_lang("JSON"));
    assert!(is_infra_lang("Yaml"));
    assert!(is_infra_lang("TOML"));
    assert!(is_infra_lang("Html"));
}

// ── 13. path_depth with double slashes ──────────────────────────

#[test]
fn path_depth_double_slashes_ignored() {
    assert_eq!(path_depth("a//b//c"), 3);
}
