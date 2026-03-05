//! Determinism hardening tests for tokmd-path.
//!
//! Verifies that path normalization is idempotent, deterministic,
//! and handles all edge cases consistently.

use proptest::prelude::*;
use tokmd_path::{normalize_rel_path, normalize_slashes};

// -- 1. Forward slash normalization is idempotent --

#[test]
fn normalize_slashes_is_idempotent() {
    let paths = [
        "src\\lib.rs",
        "a\\b\\c\\d",
        "already/forward",
        "mixed\\path/here",
        "",
        "\\",
        "\\\\server\\share",
    ];
    for p in &paths {
        let once = normalize_slashes(p);
        let twice = normalize_slashes(&once);
        assert_eq!(once, twice, "not idempotent for {p:?}");
    }
}

// -- 2. Mixed slashes normalize consistently --

#[test]
fn mixed_slashes_normalize_consistently() {
    assert_eq!(normalize_slashes("a\\b/c\\d/e"), "a/b/c/d/e");
    assert_eq!(normalize_slashes("a/b/c/d/e"), "a/b/c/d/e");
    assert_eq!(normalize_slashes("a\\b\\c\\d\\e"), "a/b/c/d/e");
}

// -- 3. No backslashes in output --

#[test]
fn normalize_slashes_removes_all_backslashes() {
    let inputs = ["\\", "\\\\", "a\\b", "\\a\\b\\c", "a\\\\b"];
    for input in &inputs {
        let result = normalize_slashes(input);
        assert!(!result.contains('\\'), "output still contains backslash for {input:?}: {result}");
    }
}

// -- 4. Empty path --

#[test]
fn empty_path_normalizes_to_empty() {
    assert_eq!(normalize_slashes(""), "");
    assert_eq!(normalize_rel_path(""), "");
}

// -- 5. Root paths --

#[test]
fn root_path_normalization() {
    assert_eq!(normalize_slashes("/"), "/");
    assert_eq!(normalize_slashes("\\"), "/");
}

// -- 6. UNC paths --

#[test]
fn unc_path_normalization() {
    assert_eq!(normalize_slashes("\\\\server\\share\\file"), "//server/share/file");
    let once = normalize_slashes("\\\\server\\share");
    let twice = normalize_slashes(&once);
    assert_eq!(once, twice);
}

// -- 7. Paths with dots --

#[test]
fn paths_with_dots_normalize_slashes_only() {
    assert_eq!(normalize_slashes(".\\src\\lib.rs"), "./src/lib.rs");
    assert_eq!(normalize_slashes("..\\parent\\file.rs"), "../parent/file.rs");
}

// -- 8. normalize_rel_path strips leading ./ --

#[test]
fn normalize_rel_path_strips_dot_slash() {
    assert_eq!(normalize_rel_path("./src/main.rs"), "src/main.rs");
    assert_eq!(normalize_rel_path(".\\src\\main.rs"), "src/main.rs");
    assert_eq!(normalize_rel_path("././src/lib.rs"), "src/lib.rs");
}

// -- 9. normalize_rel_path preserves ../ prefix --

#[test]
fn normalize_rel_path_preserves_parent_reference() {
    assert_eq!(normalize_rel_path("../src/main.rs"), "../src/main.rs");
    assert_eq!(normalize_rel_path("..\\src\\main.rs"), "../src/main.rs");
}

// -- 10. normalize_rel_path is idempotent --

#[test]
fn normalize_rel_path_is_idempotent() {
    let paths = [
        "./src/lib.rs",
        ".\\src\\main.rs",
        "src/lib.rs",
        "../lib.rs",
        "././src/lib.rs",
        "",
        "file.rs",
    ];
    for p in &paths {
        let once = normalize_rel_path(p);
        let twice = normalize_rel_path(&once);
        assert_eq!(once, twice, "normalize_rel_path not idempotent for {p:?}");
    }
}

// -- 11. normalize_rel_path determinism (same input -> same output) --

#[test]
fn normalize_rel_path_is_deterministic_100_times() {
    let input = ".\\crates\\tokmd\\src\\lib.rs";
    let results: Vec<String> = (0..100).map(|_| normalize_rel_path(input)).collect();
    assert!(results.windows(2).all(|w| w[0] == w[1]));
}

// -- 12. Trailing slash handling --

#[test]
fn trailing_slash_is_preserved() {
    assert_eq!(normalize_slashes("src\\"), "src/");
    assert_eq!(normalize_slashes("src/"), "src/");
}

// -- 13. Single component paths --

#[test]
fn single_component_paths() {
    assert_eq!(normalize_slashes("file.rs"), "file.rs");
    assert_eq!(normalize_rel_path("file.rs"), "file.rs");
    assert_eq!(normalize_rel_path("./file.rs"), "file.rs");
}

// -- 14. Paths with spaces --

#[test]
fn paths_with_spaces_normalize_correctly() {
    assert_eq!(normalize_slashes("my dir\\my file.rs"), "my dir/my file.rs");
    let once = normalize_slashes("path with spaces\\file name.txt");
    let twice = normalize_slashes(&once);
    assert_eq!(once, twice);
}

// -- 15. Multiple consecutive dot-slash stripping --

#[test]
fn multiple_consecutive_dot_slash_stripped() {
    assert_eq!(normalize_rel_path("./././a.rs"), "a.rs");
    assert_eq!(normalize_rel_path(".\\.\\.\\a.rs"), "a.rs");
}

// -- Property tests --

proptest! {
    #[test]
    fn prop_normalize_slashes_no_backslash(path in "\\PC*") {
        let result = normalize_slashes(&path);
        prop_assert!(!result.contains('\\'));
    }

    #[test]
    fn prop_normalize_slashes_idempotent(path in "\\PC*") {
        let once = normalize_slashes(&path);
        let twice = normalize_slashes(&once);
        prop_assert_eq!(once, twice);
    }

    #[test]
    fn prop_normalize_rel_path_no_backslash(path in "\\PC*") {
        let result = normalize_rel_path(&path);
        prop_assert!(!result.contains('\\'));
    }

    #[test]
    fn prop_normalize_rel_path_idempotent(path in "\\PC*") {
        let once = normalize_rel_path(&path);
        let twice = normalize_rel_path(&once);
        prop_assert_eq!(once, twice);
    }

    #[test]
    fn prop_normalize_rel_path_no_leading_dot_slash(path in "\\PC*") {
        let result = normalize_rel_path(&path);
        prop_assert!(!result.starts_with("./"), "output starts with ./: {result}");
    }
}
