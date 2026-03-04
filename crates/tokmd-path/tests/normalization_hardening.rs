//! Hardening tests for path normalization edge cases.

use proptest::prelude::*;
use tokmd_path::{normalize_rel_path, normalize_slashes};

// ── Mixed separators (forward + back slashes) ───────────────────────

#[test]
fn mixed_separators_all_become_forward() {
    assert_eq!(normalize_slashes(r"a\b/c\d/e"), "a/b/c/d/e");
}

#[test]
fn mixed_separators_complex() {
    assert_eq!(
        normalize_slashes(r"foo\bar/baz\qux/quux"),
        "foo/bar/baz/qux/quux"
    );
}

#[test]
fn mixed_rel_path() {
    assert_eq!(normalize_rel_path(r".\src/lib\mod.rs"), "src/lib/mod.rs");
}

#[test]
fn alternating_separators() {
    assert_eq!(normalize_slashes(r"a/b\c/d\e/f\g"), "a/b/c/d/e/f/g");
}

// ── UNC paths (\\server\share) ──────────────────────────────────────

#[test]
fn unc_path_backslashes_converted() {
    assert_eq!(
        normalize_slashes(r"\\server\share\dir\file.txt"),
        "//server/share/dir/file.txt"
    );
}

#[test]
fn unc_path_rel_normalization() {
    let result = normalize_rel_path(r"\\server\share\file.txt");
    assert!(!result.contains('\\'));
    assert_eq!(result, "//server/share/file.txt");
}

#[test]
fn unc_path_with_deep_nesting() {
    assert_eq!(
        normalize_slashes(r"\\host\share\a\b\c\d.rs"),
        "//host/share/a/b/c/d.rs"
    );
}

// ── Drive letter paths (C:\foo) ─────────────────────────────────────

#[test]
fn drive_letter_path_normalized() {
    assert_eq!(
        normalize_slashes(r"C:\Users\me\code\file.rs"),
        "C:/Users/me/code/file.rs"
    );
}

#[test]
fn drive_letter_with_forward_slashes_unchanged() {
    assert_eq!(
        normalize_slashes("D:/Projects/rust/main.rs"),
        "D:/Projects/rust/main.rs"
    );
}

#[test]
fn drive_letter_rel_path() {
    let result = normalize_rel_path(r"C:\code\src\lib.rs");
    assert_eq!(result, "C:/code/src/lib.rs");
    assert!(!result.contains('\\'));
}

// ── Relative paths (../foo/bar) ─────────────────────────────────────

#[test]
fn dotdot_preserved_by_normalize_slashes() {
    assert_eq!(normalize_slashes("../foo/bar"), "../foo/bar");
}

#[test]
fn dotdot_preserved_by_normalize_rel_path() {
    assert_eq!(normalize_rel_path("../foo/bar"), "../foo/bar");
}

#[test]
fn dotdot_with_backslash() {
    assert_eq!(normalize_rel_path(r"..\foo\bar"), "../foo/bar");
}

#[test]
fn dotdot_nested() {
    assert_eq!(
        normalize_rel_path("../../deep/path/file.rs"),
        "../../deep/path/file.rs"
    );
}

#[test]
fn dot_slash_then_dotdot() {
    assert_eq!(normalize_rel_path("./../foo/bar.rs"), "../foo/bar.rs");
}

// ── Path components with spaces ─────────────────────────────────────

#[test]
fn spaces_in_directory_names() {
    assert_eq!(
        normalize_slashes(r"My Documents\Projects\file.txt"),
        "My Documents/Projects/file.txt"
    );
}

#[test]
fn spaces_in_filename() {
    assert_eq!(normalize_slashes(r"src\my file.rs"), "src/my file.rs");
}

#[test]
fn spaces_rel_path() {
    assert_eq!(
        normalize_rel_path(r".\My Code\src\lib.rs"),
        "My Code/src/lib.rs"
    );
}

#[test]
fn leading_and_trailing_spaces_in_segments() {
    // Spaces are part of the path content, not stripped
    let result = normalize_slashes(r" src \ lib.rs ");
    assert_eq!(result, " src / lib.rs ");
}

// ── Very long paths (260+ chars) ────────────────────────────────────

#[test]
fn long_path_270_chars() {
    let segment = "abcdefghij"; // 10 chars
    let segments: Vec<&str> = (0..26).map(|_| segment).collect();
    let long_path = segments.join("/") + "/file.rs";
    assert!(long_path.len() > 260);
    let result = normalize_slashes(&long_path);
    assert!(!result.contains('\\'));
    assert_eq!(result, long_path);
}

#[test]
fn long_path_with_backslashes() {
    let segment = "abcdefghij";
    let segments: Vec<&str> = (0..26).map(|_| segment).collect();
    let long_path = segments.join("\\") + "\\file.rs";
    assert!(long_path.len() > 260);
    let result = normalize_slashes(&long_path);
    assert!(!result.contains('\\'));
}

#[test]
fn long_path_rel_normalization() {
    let segment = "longdir_xyz";
    let segments: Vec<&str> = (0..25).map(|_| segment).collect();
    let long_path = format!("./{}/file.rs", segments.join("/"));
    assert!(long_path.len() > 260);
    let result = normalize_rel_path(&long_path);
    assert!(!result.starts_with("./"));
    assert!(!result.contains('\\'));
}

// ── Empty and minimal inputs ────────────────────────────────────────

#[test]
fn empty_string() {
    assert_eq!(normalize_slashes(""), "");
    assert_eq!(normalize_rel_path(""), "");
}

#[test]
fn single_dot() {
    assert_eq!(normalize_slashes("."), ".");
    assert_eq!(normalize_rel_path("."), ".");
}

#[test]
fn single_slash() {
    assert_eq!(normalize_slashes("/"), "/");
    assert_eq!(normalize_rel_path("/"), "/");
}

#[test]
fn single_backslash() {
    assert_eq!(normalize_slashes("\\"), "/");
    assert_eq!(normalize_rel_path("\\"), "/");
}

#[test]
fn dot_slash_only() {
    assert_eq!(normalize_rel_path("./"), "");
}

#[test]
fn multiple_dot_slashes() {
    assert_eq!(normalize_rel_path("./././"), "");
}

// ── Property test: normalize is idempotent ──────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn normalize_slashes_idempotent(path in "\\PC{0,200}") {
        let once = normalize_slashes(&path);
        let twice = normalize_slashes(&once);
        prop_assert_eq!(&once, &twice);
    }

    #[test]
    fn normalize_rel_path_idempotent(path in "\\PC{0,200}") {
        let once = normalize_rel_path(&path);
        let twice = normalize_rel_path(&once);
        prop_assert_eq!(&once, &twice);
    }

    #[test]
    fn normalize_slashes_never_contains_backslash(path in "\\PC{0,200}") {
        let result = normalize_slashes(&path);
        prop_assert!(!result.contains('\\'));
    }

    #[test]
    fn normalize_rel_path_never_contains_backslash(path in "\\PC{0,200}") {
        let result = normalize_rel_path(&path);
        prop_assert!(!result.contains('\\'));
    }

    #[test]
    fn long_path_idempotent(
        segments in prop::collection::vec("[a-zA-Z0-9_]{5,15}", 20..40),
    ) {
        let path = format!("{}/file.rs", segments.join("/"));
        let once = normalize_rel_path(&path);
        let twice = normalize_rel_path(&once);
        prop_assert_eq!(&once, &twice);
    }

    #[test]
    fn mixed_separator_path_idempotent(
        segments in prop::collection::vec("[a-zA-Z0-9_]+", 2..8),
    ) {
        // Build a path with alternating / and \
        let mut path = String::new();
        for (i, seg) in segments.iter().enumerate() {
            if i > 0 {
                if i % 2 == 0 { path.push('/'); } else { path.push('\\'); }
            }
            path.push_str(seg);
        }
        path.push_str(".rs");
        let once = normalize_slashes(&path);
        let twice = normalize_slashes(&once);
        prop_assert_eq!(&once, &twice);
        prop_assert!(!once.contains('\\'));
    }
}
