//! Deep tests for tokmd-path — W69
//!
//! Covers: normalize_slashes, normalize_rel_path, edge cases,
//! Windows-style paths, UNC paths, and property-based tests.

use proptest::prelude::*;
use tokmd_path::{normalize_rel_path, normalize_slashes};

// ── normalize_slashes — basic ───────────────────────────────────────

#[test]
fn slashes_replaces_single_backslash() {
    assert_eq!(normalize_slashes(r"a\b"), "a/b");
}

#[test]
fn slashes_replaces_multiple_backslashes() {
    assert_eq!(normalize_slashes(r"a\b\c\d"), "a/b/c/d");
}

#[test]
fn slashes_preserves_forward_slashes() {
    assert_eq!(normalize_slashes("a/b/c"), "a/b/c");
}

#[test]
fn slashes_mixed_separators() {
    assert_eq!(normalize_slashes(r"a\b/c\d"), "a/b/c/d");
}

#[test]
fn slashes_empty_string() {
    assert_eq!(normalize_slashes(""), "");
}

#[test]
fn slashes_only_backslashes() {
    assert_eq!(normalize_slashes(r"\\\"), "///");
}

// ── normalize_slashes — Windows paths ───────────────────────────────

#[test]
fn slashes_windows_drive_path() {
    assert_eq!(normalize_slashes(r"C:\Users\dev\src"), "C:/Users/dev/src");
}

#[test]
fn slashes_unc_path() {
    assert_eq!(
        normalize_slashes(r"\\server\share\dir"),
        "//server/share/dir"
    );
}

#[test]
fn slashes_windows_extended_path() {
    assert_eq!(
        normalize_slashes(r"\\?\C:\long\path"),
        "//?/C:/long/path"
    );
}

// ── normalize_rel_path — basic ──────────────────────────────────────

#[test]
fn rel_strips_single_dot_slash() {
    assert_eq!(normalize_rel_path("./src/main.rs"), "src/main.rs");
}

#[test]
fn rel_strips_dot_backslash() {
    assert_eq!(normalize_rel_path(r".\src\main.rs"), "src/main.rs");
}

#[test]
fn rel_strips_multiple_dot_slash() {
    assert_eq!(normalize_rel_path("././././a.rs"), "a.rs");
}

#[test]
fn rel_preserves_parent_prefix() {
    assert_eq!(normalize_rel_path("../lib.rs"), "../lib.rs");
}

#[test]
fn rel_preserves_absolute_path() {
    assert_eq!(normalize_rel_path("/usr/local/bin"), "/usr/local/bin");
}

#[test]
fn rel_empty_string() {
    assert_eq!(normalize_rel_path(""), "");
}

#[test]
fn rel_dot_only() {
    // "./." -> after slash normalize becomes "./."; strip "./" -> "."
    assert_eq!(normalize_rel_path("./."), ".");
}

#[test]
fn rel_dot_slash_only_becomes_empty() {
    // "./" -> after normalize becomes "./"; strip "./" -> ""
    assert_eq!(normalize_rel_path("./"), "");
}

#[test]
fn rel_deeply_nested_dot_slash() {
    assert_eq!(
        normalize_rel_path("./././src/./inner"),
        "src/./inner"
    );
}

// ── idempotency ─────────────────────────────────────────────────────

#[test]
fn slashes_idempotent_concrete() {
    let input = r"a\b/c\d";
    let once = normalize_slashes(input);
    let twice = normalize_slashes(&once);
    assert_eq!(once, twice);
}

#[test]
fn rel_idempotent_concrete() {
    let input = r".\src\lib.rs";
    let once = normalize_rel_path(input);
    let twice = normalize_rel_path(&once);
    assert_eq!(once, twice);
}

// ── determinism ─────────────────────────────────────────────────────

#[test]
fn slashes_deterministic_across_calls() {
    let a = normalize_slashes(r"foo\bar\baz");
    let b = normalize_slashes(r"foo\bar\baz");
    assert_eq!(a, b);
}

#[test]
fn rel_deterministic_across_calls() {
    let a = normalize_rel_path("./src/main.rs");
    let b = normalize_rel_path("./src/main.rs");
    assert_eq!(a, b);
}

// ── property-based tests ────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_slashes_no_backslashes(path in "\\PC{0,200}") {
        let normalized = normalize_slashes(&path);
        prop_assert!(!normalized.contains('\\'), "got backslash in: {normalized}");
    }

    #[test]
    fn prop_slashes_idempotent(path in "\\PC{0,200}") {
        let once = normalize_slashes(&path);
        let twice = normalize_slashes(&once);
        prop_assert_eq!(&once, &twice);
    }

    #[test]
    fn prop_rel_no_backslashes(path in "\\PC{0,200}") {
        let normalized = normalize_rel_path(&path);
        prop_assert!(!normalized.contains('\\'), "got backslash in: {normalized}");
    }

    #[test]
    fn prop_rel_idempotent(path in "\\PC{0,200}") {
        let once = normalize_rel_path(&path);
        let twice = normalize_rel_path(&once);
        prop_assert_eq!(&once, &twice);
    }

    #[test]
    fn prop_slashes_length_preserved_or_same(path in "\\PC{0,200}") {
        let normalized = normalize_slashes(&path);
        prop_assert_eq!(path.len(), normalized.len(),
            "normalize_slashes should not change length");
    }

    #[test]
    fn prop_rel_no_leading_dot_slash_after_normalize(path in "\\PC{0,200}") {
        let normalized = normalize_rel_path(&path);
        prop_assert!(!normalized.starts_with("./"),
            "should not start with ./ after normalize: {normalized}");
    }
}
