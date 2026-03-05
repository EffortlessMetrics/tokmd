//! Wave-59 depth tests for path normalization — edge cases, Windows paths,
//! UNC paths, dots, unicode, and deeply-nested inputs.

use tokmd_path::{normalize_rel_path, normalize_slashes};

// ── normalize_slashes: basic ─────────────────────────────────────────────

#[test]
fn slashes_empty_string() {
    assert_eq!(normalize_slashes(""), "");
}

#[test]
fn slashes_single_backslash() {
    assert_eq!(normalize_slashes("\\"), "/");
}

#[test]
fn slashes_single_forward_slash() {
    assert_eq!(normalize_slashes("/"), "/");
}

#[test]
fn slashes_only_forward_slashes_unchanged() {
    let p = "a/b/c/d/e";
    assert_eq!(normalize_slashes(p), p);
}

#[test]
fn slashes_all_backslashes() {
    assert_eq!(normalize_slashes("a\\b\\c\\d"), "a/b/c/d");
}

#[test]
fn slashes_mixed_separators() {
    assert_eq!(normalize_slashes("a\\b/c\\d/e"), "a/b/c/d/e");
}

// ── normalize_slashes: Windows paths ─────────────────────────────────────

#[test]
fn slashes_windows_drive_letter() {
    assert_eq!(normalize_slashes("C:\\Users\\dev\\src"), "C:/Users/dev/src");
}

#[test]
fn slashes_unc_path() {
    assert_eq!(
        normalize_slashes("\\\\server\\share\\dir"),
        "//server/share/dir"
    );
}

#[test]
fn slashes_windows_long_path_prefix() {
    assert_eq!(
        normalize_slashes("\\\\?\\C:\\long\\path"),
        "//?/C:/long/path"
    );
}

// ── normalize_slashes: special characters ────────────────────────────────

#[test]
fn slashes_preserves_dots() {
    assert_eq!(normalize_slashes("..\\..\\src"), "../../src");
    assert_eq!(normalize_slashes(".\\local"), "./local");
}

#[test]
fn slashes_preserves_spaces() {
    assert_eq!(
        normalize_slashes("my dir\\sub dir\\file.rs"),
        "my dir/sub dir/file.rs"
    );
}

#[test]
fn slashes_consecutive_backslashes() {
    assert_eq!(normalize_slashes("a\\\\b\\\\\\c"), "a//b///c");
}

#[test]
fn slashes_trailing_backslash() {
    assert_eq!(normalize_slashes("dir\\"), "dir/");
}

// ── normalize_slashes: unicode ───────────────────────────────────────────

#[test]
fn slashes_cjk_path() {
    assert_eq!(normalize_slashes("项目\\源码\\主.rs"), "项目/源码/主.rs");
}

#[test]
fn slashes_emoji_path() {
    assert_eq!(normalize_slashes("📁\\📄"), "📁/📄");
}

#[test]
fn slashes_arabic_path() {
    assert_eq!(normalize_slashes("مجلد\\ملف"), "مجلد/ملف");
}

// ── normalize_slashes: length preservation ───────────────────────────────

#[test]
fn slashes_length_preserved() {
    let inputs = [
        "",
        "abc",
        "a\\b\\c",
        "\\\\unc\\share",
        "no/change",
        "a\\b/c\\d",
    ];
    for input in &inputs {
        assert_eq!(
            normalize_slashes(input).len(),
            input.len(),
            "length should be preserved for '{input}'"
        );
    }
}

// ── normalize_rel_path: basic ────────────────────────────────────────────

#[test]
fn rel_empty_string() {
    assert_eq!(normalize_rel_path(""), "");
}

#[test]
fn rel_dot_only() {
    // "./" stripped → ""
    // but "." alone has no "/" so no strip
    assert_eq!(normalize_rel_path("."), ".");
}

#[test]
fn rel_dot_slash_only() {
    assert_eq!(normalize_rel_path("./"), "");
}

#[test]
fn rel_dot_backslash_only() {
    assert_eq!(normalize_rel_path(".\\"), "");
}

#[test]
fn rel_strips_leading_dot_slash() {
    assert_eq!(normalize_rel_path("./src/main.rs"), "src/main.rs");
}

#[test]
fn rel_strips_leading_dot_backslash() {
    assert_eq!(normalize_rel_path(".\\src\\main.rs"), "src/main.rs");
}

#[test]
fn rel_strips_multiple_dot_slash() {
    assert_eq!(normalize_rel_path("././././src/lib.rs"), "src/lib.rs");
}

#[test]
fn rel_preserves_double_dot() {
    assert_eq!(normalize_rel_path("../lib.rs"), "../lib.rs");
    assert_eq!(normalize_rel_path("..\\lib.rs"), "../lib.rs");
}

#[test]
fn rel_preserves_mid_path_dot() {
    assert_eq!(normalize_rel_path("src/./lib.rs"), "src/./lib.rs");
}

#[test]
fn rel_absolute_path_unchanged() {
    assert_eq!(normalize_rel_path("/usr/bin"), "/usr/bin");
}

#[test]
fn rel_windows_absolute_normalizes_slashes() {
    assert_eq!(normalize_rel_path("C:\\src\\lib.rs"), "C:/src/lib.rs");
}

// ── normalize_rel_path: deeply nested ────────────────────────────────────

#[test]
fn rel_deeply_nested_path() {
    let deep = "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z";
    assert_eq!(normalize_rel_path(&format!("./{deep}")), deep);
}

#[test]
fn rel_long_path_over_260_chars() {
    let segment = "abcdefghij/";
    let long: String = segment.repeat(30);
    let input = format!("./{long}file.rs");
    let expected = format!("{long}file.rs");
    assert_eq!(normalize_rel_path(&input), expected);
}

// ── normalize_rel_path: unicode ──────────────────────────────────────────

#[test]
fn rel_unicode_path_normalization() {
    assert_eq!(
        normalize_rel_path(".\\项目\\源码\\主.rs"),
        "项目/源码/主.rs"
    );
}

#[test]
fn rel_emoji_with_dot_prefix() {
    assert_eq!(normalize_rel_path("./📁/📄.txt"), "📁/📄.txt");
}

// ── idempotency ──────────────────────────────────────────────────────────

#[test]
fn slashes_idempotent_batch() {
    let inputs = [
        "",
        "a\\b",
        "C:\\Users\\dev",
        "already/forward",
        "\\\\unc\\share",
    ];
    for input in &inputs {
        let once = normalize_slashes(input);
        let twice = normalize_slashes(&once);
        assert_eq!(
            once, twice,
            "normalize_slashes not idempotent for '{input}'"
        );
    }
}

#[test]
fn rel_idempotent_batch() {
    let inputs = [
        "",
        "./src/main.rs",
        ".\\src\\main.rs",
        "src/main.rs",
        "././a/b",
        "../up",
    ];
    for input in &inputs {
        let once = normalize_rel_path(input);
        let twice = normalize_rel_path(&once);
        assert_eq!(
            once, twice,
            "normalize_rel_path not idempotent for '{input}'"
        );
    }
}

// ── determinism ──────────────────────────────────────────────────────────

#[test]
fn both_functions_deterministic_100_iterations() {
    for _ in 0..100 {
        assert_eq!(normalize_slashes("a\\b\\c"), "a/b/c");
        assert_eq!(normalize_rel_path("./src\\lib.rs"), "src/lib.rs");
    }
}
