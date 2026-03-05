//! W57 depth tests for tokmd-analysis-util.

use std::path::{Path, PathBuf};
use tokmd_analysis_util::{
    AnalysisLimits, empty_file_row, is_infra_lang, is_test_path, normalize_path, normalize_root,
    now_ms, path_depth,
};

// ===========================================================================
// 1. normalize_path exhaustive tests
// ===========================================================================

#[test]
fn normalize_path_strips_single_leading_dot_slash() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("./src/lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_strips_multiple_dot_slash() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("././././src/lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_converts_backslashes() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path(r"src\main\lib.rs", &root), "src/main/lib.rs");
}

#[test]
fn normalize_path_backslash_dot_prefix() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path(r".\src\lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_empty_string() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("", &root), "");
}

#[test]
fn normalize_path_just_dot_slash() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("./", &root), "");
}

#[test]
fn normalize_path_deeply_nested_dot_slash() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("./././././a/b/c.rs", &root), "a/b/c.rs");
}

#[test]
fn normalize_path_no_leading_dot_slash() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("src/lib.rs", &root), "src/lib.rs");
}

#[test]
fn normalize_path_with_unicode() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("./ñoño/café.rs", &root), "ñoño/café.rs");
}

// ===========================================================================
// 2. AnalysisLimits defaults and overrides
// ===========================================================================

#[test]
fn analysis_limits_default_all_none() {
    let limits = AnalysisLimits::default();
    assert!(limits.max_files.is_none());
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
    assert!(limits.max_commits.is_none());
    assert!(limits.max_commit_files.is_none());
}

#[test]
fn analysis_limits_override_individual_fields() {
    let limits = AnalysisLimits {
        max_files: Some(100),
        max_bytes: Some(1024 * 1024),
        max_file_bytes: None,
        max_commits: Some(500),
        max_commit_files: None,
    };
    assert_eq!(limits.max_files, Some(100));
    assert_eq!(limits.max_bytes, Some(1_048_576));
    assert!(limits.max_file_bytes.is_none());
    assert_eq!(limits.max_commits, Some(500));
}

#[test]
fn analysis_limits_clone_preserves_values() {
    let limits = AnalysisLimits {
        max_files: Some(42),
        max_bytes: Some(999),
        max_file_bytes: Some(100),
        max_commits: Some(10),
        max_commit_files: Some(5),
    };
    let cloned = limits.clone();
    assert_eq!(cloned.max_files, Some(42));
    assert_eq!(cloned.max_bytes, Some(999));
    assert_eq!(cloned.max_file_bytes, Some(100));
    assert_eq!(cloned.max_commits, Some(10));
    assert_eq!(cloned.max_commit_files, Some(5));
}

// ===========================================================================
// 3. Shared utility functions
// ===========================================================================

#[test]
fn path_depth_single_segment() {
    assert_eq!(path_depth("file.rs"), 1);
}

#[test]
fn path_depth_multi_segment() {
    assert_eq!(path_depth("src/lib/core.rs"), 3);
}

#[test]
fn path_depth_empty_returns_one() {
    assert_eq!(path_depth(""), 1);
}

#[test]
fn is_test_path_tests_dir() {
    assert!(is_test_path("src/tests/foo.rs"));
}

#[test]
fn is_test_path_test_suffix() {
    assert!(is_test_path("src/foo_test.rs"));
}

#[test]
fn is_test_path_test_prefix() {
    assert!(is_test_path("src/test_foo.rs"));
}

#[test]
fn is_test_path_spec_dir() {
    assert!(is_test_path("src/spec/foo.spec.ts"));
}

#[test]
fn is_test_path_not_test() {
    assert!(!is_test_path("src/main.rs"));
}

#[test]
fn is_infra_lang_positive() {
    for lang in &["json", "yaml", "toml", "markdown", "css", "svg"] {
        assert!(is_infra_lang(lang), "expected infra: {lang}");
    }
}

#[test]
fn is_infra_lang_negative() {
    for lang in &["rust", "python", "go", "java"] {
        assert!(!is_infra_lang(lang), "expected non-infra: {lang}");
    }
}

#[test]
fn empty_file_row_all_zeroes() {
    let row = empty_file_row();
    assert_eq!(row.code, 0);
    assert_eq!(row.comments, 0);
    assert_eq!(row.blanks, 0);
    assert_eq!(row.lines, 0);
    assert_eq!(row.bytes, 0);
    assert_eq!(row.tokens, 0);
    assert!(row.path.is_empty());
}

#[test]
fn now_ms_returns_positive() {
    let ts = now_ms();
    assert!(ts > 0);
}

#[test]
fn normalize_root_returns_something() {
    let root = normalize_root(Path::new("."));
    assert!(!root.as_os_str().is_empty());
}

// ===========================================================================
// 4. Edge case inputs
// ===========================================================================

#[test]
fn normalize_path_special_chars() {
    let root = PathBuf::from("repo");
    assert_eq!(normalize_path("./a b c/d&e.rs", &root), "a b c/d&e.rs");
}

#[test]
fn normalize_path_very_long_path() {
    let root = PathBuf::from("repo");
    let long_path = format!("./{}", "a/".repeat(200));
    let result = normalize_path(&long_path, &root);
    assert!(!result.starts_with("./"));
}

#[test]
fn path_depth_trailing_slash() {
    assert_eq!(path_depth("src/lib/"), 2);
}

#[test]
fn path_depth_double_slash() {
    assert_eq!(path_depth("src//lib"), 2);
}

// ===========================================================================
// 5. Proptest for path normalization invariants
// ===========================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn normalize_path_never_has_backslash(path in "[a-zA-Z0-9_./\\\\]{0,100}") {
            let root = PathBuf::from("repo");
            let out = normalize_path(&path, &root);
            prop_assert!(!out.contains('\\'), "output contains backslash: {out}");
        }

        #[test]
        fn normalize_path_never_starts_with_dot_slash(path in "[a-zA-Z0-9_./\\\\]{0,100}") {
            let root = PathBuf::from("repo");
            let out = normalize_path(&path, &root);
            prop_assert!(!out.starts_with("./"), "output starts with ./: {out}");
        }

        #[test]
        fn normalize_path_is_deterministic(path in "[a-zA-Z0-9_./\\\\]{0,80}") {
            let root = PathBuf::from("repo");
            let a = normalize_path(&path, &root);
            let b = normalize_path(&path, &root);
            prop_assert_eq!(a, b);
        }

        #[test]
        fn path_depth_always_ge_one(path in "\\PC*") {
            prop_assert!(path_depth(&path) >= 1);
        }
    }
}
