//! W62 depth tests for tokmd-path: comprehensive coverage of path normalization.

use tokmd_path::{normalize_rel_path, normalize_slashes};

// ── normalize_slashes: basic ───────────────────────────────────────────────

#[test]
fn slashes_empty_string() {
    assert_eq!(normalize_slashes(""), "");
}

#[test]
fn slashes_single_backslash() {
    assert_eq!(normalize_slashes("\\"), "/");
}

#[test]
fn slashes_single_forward() {
    assert_eq!(normalize_slashes("/"), "/");
}

#[test]
fn slashes_no_separators() {
    assert_eq!(normalize_slashes("file.rs"), "file.rs");
}

#[test]
fn slashes_all_backslashes() {
    assert_eq!(normalize_slashes("a\\b\\c\\d"), "a/b/c/d");
}

#[test]
fn slashes_all_forward() {
    assert_eq!(normalize_slashes("a/b/c/d"), "a/b/c/d");
}

#[test]
fn slashes_mixed() {
    assert_eq!(normalize_slashes("a\\b/c\\d"), "a/b/c/d");
}

// ── normalize_slashes: double/triple slashes ───────────────────────────────

#[test]
fn slashes_double_backslash() {
    assert_eq!(normalize_slashes("a\\\\b"), "a//b");
}

#[test]
fn slashes_double_forward_passthrough() {
    assert_eq!(normalize_slashes("a//b"), "a//b");
}

#[test]
fn slashes_triple_backslash() {
    assert_eq!(normalize_slashes("a\\\\\\b"), "a///b");
}

// ── normalize_slashes: trailing/leading ────────────────────────────────────

#[test]
fn slashes_trailing_backslash() {
    assert_eq!(normalize_slashes("src\\"), "src/");
}

#[test]
fn slashes_leading_backslash() {
    assert_eq!(normalize_slashes("\\src"), "/src");
}

#[test]
fn slashes_trailing_forward_unchanged() {
    assert_eq!(normalize_slashes("src/"), "src/");
}

// ── normalize_slashes: Windows-style paths ─────────────────────────────────

#[test]
fn slashes_windows_drive_path() {
    assert_eq!(
        normalize_slashes("C:\\Users\\dev\\project"),
        "C:/Users/dev/project"
    );
}

#[test]
fn slashes_unc_path() {
    assert_eq!(
        normalize_slashes("\\\\server\\share\\file"),
        "//server/share/file"
    );
}

// ── normalize_slashes: Unicode ─────────────────────────────────────────────

#[test]
fn slashes_unicode_path() {
    assert_eq!(normalize_slashes("日本語\\パス"), "日本語/パス");
}

#[test]
fn slashes_emoji_path() {
    assert_eq!(normalize_slashes("📁\\📄"), "📁/📄");
}

#[test]
fn slashes_accented_chars() {
    assert_eq!(normalize_slashes("café\\résumé"), "café/résumé");
}

// ── normalize_slashes: spaces and special chars ────────────────────────────

#[test]
fn slashes_spaces_in_path() {
    assert_eq!(normalize_slashes("my dir\\my file.rs"), "my dir/my file.rs");
}

#[test]
fn slashes_dots_in_path() {
    assert_eq!(normalize_slashes(".hidden\\.config"), ".hidden/.config");
}

// ── normalize_rel_path: basic ──────────────────────────────────────────────

#[test]
fn rel_empty_string() {
    assert_eq!(normalize_rel_path(""), "");
}

#[test]
fn rel_simple_forward() {
    assert_eq!(normalize_rel_path("src/lib.rs"), "src/lib.rs");
}

#[test]
fn rel_simple_backslash() {
    assert_eq!(normalize_rel_path("src\\lib.rs"), "src/lib.rs");
}

// ── normalize_rel_path: dot-slash stripping ────────────────────────────────

#[test]
fn rel_single_dot_slash() {
    assert_eq!(normalize_rel_path("./src/main.rs"), "src/main.rs");
}

#[test]
fn rel_single_dot_backslash() {
    assert_eq!(normalize_rel_path(".\\src\\main.rs"), "src/main.rs");
}

#[test]
fn rel_double_dot_slash() {
    assert_eq!(normalize_rel_path("././src/lib.rs"), "src/lib.rs");
}

#[test]
fn rel_triple_dot_slash() {
    assert_eq!(normalize_rel_path("./././a.rs"), "a.rs");
}

#[test]
fn rel_dot_slash_only() {
    // "./" stripped yields ""
    assert_eq!(normalize_rel_path("./"), "");
}

#[test]
fn rel_dot_dot_not_stripped() {
    // "../" is NOT a "./" prefix, should be preserved
    assert_eq!(normalize_rel_path("../lib.rs"), "../lib.rs");
}

#[test]
fn rel_dot_dot_backslash_not_stripped() {
    assert_eq!(normalize_rel_path("..\\lib.rs"), "../lib.rs");
}

#[test]
fn rel_dot_in_middle_not_stripped() {
    // "./a/./b" — only leading "./" removed
    assert_eq!(normalize_rel_path("./a/./b"), "a/./b");
}

// ── normalize_rel_path: combined ───────────────────────────────────────────

#[test]
fn rel_backslash_plus_dot() {
    assert_eq!(normalize_rel_path(".\\foo\\bar"), "foo/bar");
}

#[test]
fn rel_mixed_separators_with_dot() {
    assert_eq!(normalize_rel_path("./a\\b/c\\d"), "a/b/c/d");
}

#[test]
fn rel_unicode() {
    assert_eq!(normalize_rel_path("./données\\fichier"), "données/fichier");
}

#[test]
fn rel_spaces() {
    assert_eq!(
        normalize_rel_path("./my dir\\my file.rs"),
        "my dir/my file.rs"
    );
}

// ── normalize_rel_path: Windows absolute ───────────────────────────────────

#[test]
fn rel_windows_absolute_passthrough() {
    // Absolute Windows paths have no "./" prefix to strip.
    assert_eq!(normalize_rel_path("C:\\Users\\dev"), "C:/Users/dev");
}

// ── normalize_rel_path: trailing slashes ───────────────────────────────────

#[test]
fn rel_trailing_forward_slash() {
    assert_eq!(normalize_rel_path("./src/"), "src/");
}

#[test]
fn rel_trailing_backslash() {
    assert_eq!(normalize_rel_path(".\\src\\"), "src/");
}

// ── idempotency hand-written ───────────────────────────────────────────────

#[test]
fn slashes_idempotent_hand() {
    let input = "a\\b\\c";
    let once = normalize_slashes(input);
    let twice = normalize_slashes(&once);
    assert_eq!(once, twice);
}

#[test]
fn rel_idempotent_hand() {
    let input = ".\\src\\main.rs";
    let once = normalize_rel_path(input);
    let twice = normalize_rel_path(&once);
    assert_eq!(once, twice);
}

// ── determinism ────────────────────────────────────────────────────────────

#[test]
fn slashes_deterministic() {
    let input = "foo\\bar\\baz";
    let expected = normalize_slashes(input);
    for _ in 0..100 {
        assert_eq!(normalize_slashes(input), expected);
    }
}

#[test]
fn rel_deterministic() {
    let input = "./foo\\bar";
    let expected = normalize_rel_path(input);
    for _ in 0..100 {
        assert_eq!(normalize_rel_path(input), expected);
    }
}

// ── proptest ───────────────────────────────────────────────────────────────

mod property_tests {
    use proptest::prelude::*;
    use tokmd_path::{normalize_rel_path, normalize_slashes};

    proptest! {
        #[test]
        fn slashes_never_contain_backslash(path in "\\PC*") {
            let n = normalize_slashes(&path);
            prop_assert!(!n.contains('\\'), "backslash in output: {n}");
        }

        #[test]
        fn slashes_idempotent(path in "\\PC*") {
            let once = normalize_slashes(&path);
            let twice = normalize_slashes(&once);
            prop_assert_eq!(&once, &twice);
        }

        #[test]
        fn slashes_length_unchanged(path in "\\PC*") {
            // Replacing \\ with / is char-for-char, so length must be equal.
            let n = normalize_slashes(&path);
            prop_assert_eq!(path.len(), n.len(),
                "length changed: input={}, output={}", path.len(), n.len());
        }

        #[test]
        fn rel_never_contain_backslash(path in "\\PC*") {
            let n = normalize_rel_path(&path);
            prop_assert!(!n.contains('\\'), "backslash in output: {n}");
        }

        #[test]
        fn rel_idempotent(path in "\\PC*") {
            let once = normalize_rel_path(&path);
            let twice = normalize_rel_path(&once);
            prop_assert_eq!(&once, &twice);
        }

        #[test]
        fn rel_no_leading_dot_slash(path in "\\PC*") {
            let n = normalize_rel_path(&path);
            prop_assert!(!n.starts_with("./"),
                "output starts with ./: {n}");
        }

        #[test]
        fn rel_shorter_or_equal(path in "\\PC*") {
            let n = normalize_rel_path(&path);
            prop_assert!(n.len() <= path.len(),
                "output longer than input: {} > {}", n.len(), path.len());
        }
    }
}
