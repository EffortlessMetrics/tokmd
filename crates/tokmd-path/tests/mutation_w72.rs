//! Mutation-hardening tests for `tokmd-path`.
//!
//! Targets: operator-swap, boolean-flip, and boundary-condition
//! mutation survivors in path normalization.

use tokmd_path::{normalize_rel_path, normalize_slashes};

// ── normalize_slashes ────────────────────────────────────────────────

#[test]
fn slashes_backslash_converted() {
    assert_eq!(normalize_slashes(r"a\b\c"), "a/b/c");
}

#[test]
fn slashes_forward_slash_unchanged() {
    assert_eq!(normalize_slashes("a/b/c"), "a/b/c");
}

#[test]
fn slashes_empty_string() {
    assert_eq!(normalize_slashes(""), "");
}

#[test]
fn slashes_single_backslash() {
    assert_eq!(normalize_slashes("\\"), "/");
}

#[test]
fn slashes_no_separators() {
    assert_eq!(normalize_slashes("filename.rs"), "filename.rs");
}

#[test]
fn slashes_mixed_separators() {
    assert_eq!(normalize_slashes(r"a/b\c/d\e"), "a/b/c/d/e");
}

#[test]
fn slashes_consecutive_backslashes() {
    assert_eq!(normalize_slashes(r"a\\b"), "a//b");
}

#[test]
fn slashes_idempotent() {
    let input = r"x\y\z";
    let once = normalize_slashes(input);
    let twice = normalize_slashes(&once);
    assert_eq!(once, twice);
}

// ── normalize_rel_path ───────────────────────────────────────────────

#[test]
fn rel_strips_leading_dot_slash() {
    assert_eq!(normalize_rel_path("./src/lib.rs"), "src/lib.rs");
}

#[test]
fn rel_strips_multiple_dot_slash() {
    assert_eq!(normalize_rel_path("././src/lib.rs"), "src/lib.rs");
}

#[test]
fn rel_strips_backslash_dot() {
    assert_eq!(normalize_rel_path(r".\src\main.rs"), "src/main.rs");
}

#[test]
fn rel_preserves_double_dot_prefix() {
    // `../` must NOT be stripped — only `./`
    assert_eq!(normalize_rel_path("../src/lib.rs"), "../src/lib.rs");
}

#[test]
fn rel_empty_string() {
    assert_eq!(normalize_rel_path(""), "");
}

#[test]
fn rel_bare_dot_slash() {
    // "./" alone → ""
    assert_eq!(normalize_rel_path("./"), "");
}

#[test]
fn rel_idempotent() {
    let cases = [
        r".\src\lib.rs",
        "./a/b/c",
        "already/normalized",
        "../keep/this",
    ];
    for input in cases {
        let once = normalize_rel_path(input);
        let twice = normalize_rel_path(&once);
        assert_eq!(once, twice, "not idempotent for: {input}");
    }
}
