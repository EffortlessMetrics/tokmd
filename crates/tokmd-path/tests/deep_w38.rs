//! Wave-38 deep tests for tokmd-path normalization.
//!
//! Covers edge cases around special characters, consecutive separators,
//! case preservation, very long paths, and cross-function invariants
//! not yet exercised by the existing test suite.

use tokmd_path::{normalize_rel_path, normalize_slashes};

// ═══════════════════════════════════════════════════════════════════
// normalize_slashes — special character preservation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn slashes_preserves_at_sign() {
    assert_eq!(normalize_slashes(r"user@host\path"), "user@host/path");
}

#[test]
fn slashes_preserves_hash() {
    assert_eq!(normalize_slashes(r"dir\file#section"), "dir/file#section");
}

#[test]
fn slashes_preserves_dollar() {
    assert_eq!(normalize_slashes(r"$HOME\dir"), "$HOME/dir");
}

#[test]
fn slashes_preserves_equals() {
    assert_eq!(normalize_slashes(r"key=val\next"), "key=val/next");
}

#[test]
fn slashes_preserves_case() {
    assert_eq!(normalize_slashes(r"Foo\Bar\BAZ"), "Foo/Bar/BAZ");
}

// ═══════════════════════════════════════════════════════════════════
// normalize_slashes — consecutive and mixed separators
// ═══════════════════════════════════════════════════════════════════

#[test]
fn slashes_consecutive_forward_unchanged() {
    assert_eq!(normalize_slashes("a//b///c"), "a//b///c");
}

#[test]
fn slashes_mixed_consecutive_separators() {
    // "\/" → "//"
    assert_eq!(normalize_slashes("\\/"), "//");
}

#[test]
fn slashes_all_backslashes_string() {
    assert_eq!(normalize_slashes("\\\\\\"), "///");
}

// ═══════════════════════════════════════════════════════════════════
// normalize_slashes — path-like edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn slashes_dot_only() {
    assert_eq!(normalize_slashes("."), ".");
}

#[test]
fn slashes_double_dot_only() {
    assert_eq!(normalize_slashes(".."), "..");
}

#[test]
fn slashes_extension_chain() {
    assert_eq!(normalize_slashes(r"dir\file.tar.gz"), "dir/file.tar.gz");
}

#[test]
fn slashes_numeric_segments() {
    assert_eq!(normalize_slashes(r"2024\01\31\data"), "2024/01/31/data");
}

#[test]
fn slashes_very_long_path() {
    let seg = "abcdefghij";
    let input: String = std::iter::repeat_n(seg, 100)
        .collect::<Vec<_>>()
        .join("\\");
    let expected: String = std::iter::repeat_n(seg, 100)
        .collect::<Vec<_>>()
        .join("/");
    assert_eq!(normalize_slashes(&input), expected);
}

#[test]
fn slashes_windows_extended_prefix() {
    assert_eq!(normalize_slashes(r"\\?\C:\long\path"), "//?/C:/long/path");
}

// ═══════════════════════════════════════════════════════════════════
// normalize_rel_path — additional edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn rel_bare_dot_is_unchanged() {
    // "." doesn't start with "./" so it stays
    assert_eq!(normalize_rel_path("."), ".");
}

#[test]
fn rel_dot_slash_dot_yields_dot() {
    // "./.": after stripping "./" we get "."
    assert_eq!(normalize_rel_path("./."), ".");
}

#[test]
fn rel_dot_slash_dotdot_yields_dotdot() {
    // "./..": after stripping "./" we get ".."
    assert_eq!(normalize_rel_path("./.."), "..");
}

#[test]
fn rel_dot_slash_hidden_dir() {
    assert_eq!(normalize_rel_path("./.config/app"), ".config/app");
}

#[test]
fn rel_handles_at_in_path() {
    assert_eq!(normalize_rel_path(r".\@scope\pkg"), "@scope/pkg");
}

#[test]
fn rel_preserves_query_like_suffix() {
    assert_eq!(normalize_rel_path("./path?query=1"), "path?query=1");
}

#[test]
fn rel_very_long_dot_slash_prefix() {
    let prefix = "./".repeat(50);
    let input = format!("{prefix}file.rs");
    assert_eq!(normalize_rel_path(&input), "file.rs");
}

#[test]
fn rel_numeric_only_segments() {
    assert_eq!(normalize_rel_path(r".\1\2\3"), "1/2/3");
}

// ═══════════════════════════════════════════════════════════════════
// Cross-function invariants
// ═══════════════════════════════════════════════════════════════════

#[test]
fn rel_always_subset_of_slash_normalized_length() {
    let cases = [
        "./a/b/c",
        r".\x\y",
        "././deep/path",
        "no/prefix",
        "",
        ".",
        ".hidden",
    ];
    for input in &cases {
        let s = normalize_slashes(input);
        let r = normalize_rel_path(input);
        assert!(
            r.len() <= s.len(),
            "rel ({}) should not be longer than slashes ({}) for {:?}",
            r.len(),
            s.len(),
            input
        );
    }
}

#[test]
fn slash_then_rel_equals_rel_alone() {
    // normalize_rel_path already calls normalize_slashes internally
    let inputs = [r".\src\lib.rs", r"a\b\c", "./foo/bar"];
    for input in &inputs {
        let composed = normalize_rel_path(&normalize_slashes(input));
        let direct = normalize_rel_path(input);
        assert_eq!(composed, direct, "composition mismatch for {:?}", input);
    }
}

#[test]
fn no_backslash_in_any_output() {
    let cases = [
        r"C:\Users\me",
        r"\\server\share",
        r".\.\nested\path",
        r"..\parent\child",
        r"\\?\long",
        "clean/path",
        "",
    ];
    for input in &cases {
        assert!(
            !normalize_slashes(input).contains('\\'),
            "backslash in normalize_slashes for {:?}",
            input
        );
        assert!(
            !normalize_rel_path(input).contains('\\'),
            "backslash in normalize_rel_path for {:?}",
            input
        );
    }
}
