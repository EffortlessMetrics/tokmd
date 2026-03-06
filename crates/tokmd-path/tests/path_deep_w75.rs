//! Deep property-based tests for tokmd-path (W75).
//!
//! Covers: idempotency, backslash elimination, consecutive-slash
//! absence, Windows paths, UNC paths, Unicode, and prefix stripping.

use proptest::prelude::*;
use tokmd_path::{normalize_rel_path, normalize_slashes};

// ── proptest: idempotency ──────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn normalize_slashes_idempotent(path in "\\PC{0,120}") {
        let once = normalize_slashes(&path);
        let twice = normalize_slashes(&once);
        prop_assert_eq!(&once, &twice);
    }

    #[test]
    fn normalize_rel_path_idempotent(path in "\\PC{0,120}") {
        let once = normalize_rel_path(&path);
        let twice = normalize_rel_path(&once);
        prop_assert_eq!(&once, &twice);
    }
}

// ── proptest: no backslashes in output ─────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn normalize_slashes_never_contains_backslash(path in "\\PC{0,120}") {
        let out = normalize_slashes(&path);
        prop_assert!(
            !out.contains('\\'),
            "output must not contain backslash: {out}"
        );
    }

    #[test]
    fn normalize_rel_path_never_contains_backslash(path in "\\PC{0,120}") {
        let out = normalize_rel_path(&path);
        prop_assert!(
            !out.contains('\\'),
            "output must not contain backslash: {out}"
        );
    }
}

// ── proptest: no consecutive slashes (normalize_rel_path preserves input slashes) ──

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// For clean single-slash inputs, normalization must not introduce
    /// consecutive slashes.
    #[test]
    fn normalize_slashes_no_double_slash_on_clean_input(
        segments in prop::collection::vec("[a-z]{1,10}", 1..6),
    ) {
        let path = segments.join("\\");
        let out = normalize_slashes(&path);
        prop_assert!(
            !out.contains("//"),
            "unexpected consecutive slashes in: {out}"
        );
    }
}

// ── Windows-style paths ────────────────────────────────────────────

#[test]
fn windows_drive_path() {
    assert_eq!(
        normalize_slashes(r"C:\Users\dev\project\src\main.rs"),
        "C:/Users/dev/project/src/main.rs"
    );
}

#[test]
fn windows_drive_letter_preserved() {
    let out = normalize_slashes(r"D:\foo\bar");
    assert!(out.starts_with("D:"));
    assert!(!out.contains('\\'));
}

#[test]
fn windows_mixed_separators() {
    assert_eq!(
        normalize_slashes(r"C:\Users/dev\src/lib.rs"),
        "C:/Users/dev/src/lib.rs"
    );
}

// ── UNC paths ──────────────────────────────────────────────────────

#[test]
fn unc_path_normalised() {
    assert_eq!(
        normalize_slashes(r"\\server\share\folder\file.txt"),
        "//server/share/folder/file.txt"
    );
}

#[test]
fn unc_path_preserves_double_slash_prefix() {
    let out = normalize_slashes(r"\\host\vol");
    assert!(out.starts_with("//"));
}

// ── Unicode paths ──────────────────────────────────────────────────

#[test]
fn unicode_path_normalised() {
    assert_eq!(
        normalize_slashes("données\\résumé\\café.txt"),
        "données/résumé/café.txt"
    );
}

#[test]
fn cjk_path_normalised() {
    assert_eq!(normalize_slashes("项目\\源码\\主.rs"), "项目/源码/主.rs");
}

#[test]
fn emoji_path_normalised() {
    assert_eq!(normalize_slashes("🚀\\launch\\🌍.md"), "🚀/launch/🌍.md");
}

// ── prefix stripping (normalize_rel_path) ──────────────────────────

#[test]
fn strip_single_dot_slash() {
    assert_eq!(normalize_rel_path("./src/lib.rs"), "src/lib.rs");
}

#[test]
fn strip_multiple_dot_slashes() {
    assert_eq!(normalize_rel_path("././././a.rs"), "a.rs");
}

#[test]
fn strip_dot_backslash() {
    assert_eq!(normalize_rel_path(r".\src\main.rs"), "src/main.rs");
}

#[test]
fn no_strip_double_dot() {
    // `../` is NOT a redundant prefix — must be preserved.
    assert_eq!(normalize_rel_path("../lib.rs"), "../lib.rs");
}

#[test]
fn empty_input() {
    assert_eq!(normalize_slashes(""), "");
    assert_eq!(normalize_rel_path(""), "");
}

#[test]
fn single_dot_preserved_as_dot() {
    // A bare "." is not prefixed with "./" so it stays.
    let out = normalize_rel_path(".");
    assert_eq!(out, ".");
}
