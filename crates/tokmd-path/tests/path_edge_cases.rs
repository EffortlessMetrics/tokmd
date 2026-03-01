//! Edge-case tests for path normalization utilities.

use tokmd_path::{normalize_rel_path, normalize_slashes};

// =============================================================================
// Windows backslash paths → forward slash
// =============================================================================

#[test]
fn backslash_simple_path() {
    assert_eq!(normalize_slashes(r"foo\bar\baz.rs"), "foo/bar/baz.rs");
}

#[test]
fn backslash_drive_letter_path() {
    assert_eq!(
        normalize_slashes(r"C:\Users\me\project\src\main.rs"),
        "C:/Users/me/project/src/main.rs"
    );
}

#[test]
fn backslash_mixed_separators() {
    assert_eq!(
        normalize_slashes(r"foo/bar\baz/qux\file.rs"),
        "foo/bar/baz/qux/file.rs"
    );
}

// =============================================================================
// UNC paths (\\server\share)
// =============================================================================

#[test]
fn unc_path_basic() {
    assert_eq!(
        normalize_slashes(r"\\server\share\dir\file.txt"),
        "//server/share/dir/file.txt"
    );
}

#[test]
fn unc_path_no_trailing_file() {
    assert_eq!(normalize_slashes(r"\\server\share"), "//server/share");
}

#[test]
fn unc_path_with_spaces() {
    assert_eq!(
        normalize_slashes(r"\\server\my share\my dir\file.txt"),
        "//server/my share/my dir/file.txt"
    );
}

#[test]
fn rel_path_unc_preserved() {
    // UNC paths don't start with `./`, so normalize_rel_path just normalizes slashes.
    assert_eq!(
        normalize_rel_path(r"\\server\share\file.rs"),
        "//server/share/file.rs"
    );
}

// =============================================================================
// Paths with dots (./foo/../bar)
// =============================================================================

#[test]
fn dot_segments_preserved_by_normalize_slashes() {
    // normalize_slashes does NOT resolve `.` or `..`; it only fixes separators.
    assert_eq!(normalize_slashes(r".\foo\..\bar"), "./foo/../bar");
}

#[test]
fn dot_dot_path_preserved() {
    assert_eq!(normalize_slashes("../../../file.rs"), "../../../file.rs");
}

#[test]
fn rel_path_strips_leading_dot_slash_but_keeps_inner_dots() {
    assert_eq!(normalize_rel_path("./foo/../bar/./baz"), "foo/../bar/./baz");
}

#[test]
fn rel_path_preserves_double_dot_prefix() {
    assert_eq!(normalize_rel_path("../src/main.rs"), "../src/main.rs");
}

#[test]
fn rel_path_only_strips_one_leading_dot_slash() {
    assert_eq!(normalize_rel_path("././foo"), "./foo");
}

// =============================================================================
// Empty paths
// =============================================================================

#[test]
fn empty_string_normalize_slashes() {
    assert_eq!(normalize_slashes(""), "");
}

#[test]
fn empty_string_normalize_rel_path() {
    assert_eq!(normalize_rel_path(""), "");
}

// =============================================================================
// Root-only paths
// =============================================================================

#[test]
fn single_forward_slash() {
    assert_eq!(normalize_slashes("/"), "/");
    assert_eq!(normalize_rel_path("/"), "/");
}

#[test]
fn single_backslash_becomes_forward_slash() {
    assert_eq!(normalize_slashes("\\"), "/");
    assert_eq!(normalize_rel_path("\\"), "/");
}

#[test]
fn bare_dot_slash() {
    assert_eq!(normalize_rel_path("./"), "");
}

#[test]
fn bare_dot_backslash() {
    assert_eq!(normalize_rel_path(r".\"), "");
}

#[test]
fn bare_dot_no_slash_is_hidden_file() {
    assert_eq!(normalize_rel_path(".hidden"), ".hidden");
}

#[test]
fn just_dot() {
    // Single dot with no slash — doesn't match `./` prefix, kept as-is.
    assert_eq!(normalize_slashes("."), ".");
    assert_eq!(normalize_rel_path("."), ".");
}

// =============================================================================
// Very long paths
// =============================================================================

#[test]
fn very_long_path_backslashes() {
    let long = (0..500)
        .map(|i| format!("dir{i}"))
        .collect::<Vec<_>>()
        .join("\\");
    let result = normalize_slashes(&long);
    assert!(!result.contains('\\'));
    assert_eq!(result.matches('/').count(), 499);
}

#[test]
fn very_long_path_rel_prefix() {
    let long = format!("./{}", "a/".repeat(500));
    let result = normalize_rel_path(&long);
    assert!(!result.starts_with("./"));
    assert!(result.starts_with("a/"));
}

#[test]
fn very_long_single_segment() {
    let segment = "x".repeat(10_000);
    assert_eq!(normalize_slashes(&segment), segment);
}

// =============================================================================
// Unicode characters in paths
// =============================================================================

#[test]
fn unicode_cjk_path() {
    assert_eq!(normalize_slashes(r"中文\路径\文件.rs"), "中文/路径/文件.rs");
}

#[test]
fn unicode_accented_path() {
    assert_eq!(
        normalize_slashes(r"données\résumé\café.txt"),
        "données/résumé/café.txt"
    );
}

#[test]
fn unicode_cyrillic_path() {
    assert_eq!(normalize_slashes(r"путь\к\файлу.rs"), "путь/к/файлу.rs");
}

#[test]
fn unicode_emoji_path() {
    assert_eq!(normalize_slashes("📁\\📂\\📄.txt"), "📁/📂/📄.txt");
}

#[test]
fn unicode_rel_path() {
    assert_eq!(
        normalize_rel_path(r".\données\résumé.rs"),
        "données/résumé.rs"
    );
}

#[test]
fn unicode_mixed_scripts() {
    assert_eq!(
        normalize_slashes(r"src\日本語\données\путь.rs"),
        "src/日本語/données/путь.rs"
    );
}

// =============================================================================
// Trailing slashes
// =============================================================================

#[test]
fn trailing_backslash_preserved_as_forward() {
    assert_eq!(normalize_slashes(r"foo\bar\"), "foo/bar/");
}

#[test]
fn trailing_forward_slash_unchanged() {
    assert_eq!(normalize_slashes("foo/bar/"), "foo/bar/");
}

#[test]
fn rel_path_trailing_slash() {
    assert_eq!(normalize_rel_path("./foo/bar/"), "foo/bar/");
}

// =============================================================================
// Multiple consecutive separators
// =============================================================================

#[test]
fn consecutive_backslashes() {
    assert_eq!(normalize_slashes("a\\\\b"), "a//b");
}

#[test]
fn consecutive_forward_slashes_unchanged() {
    assert_eq!(normalize_slashes("a//b"), "a//b");
}

#[test]
fn triple_backslashes() {
    assert_eq!(normalize_slashes("a\\\\\\b"), "a///b");
}

#[test]
fn consecutive_mixed_separators() {
    assert_eq!(normalize_slashes("a/\\b"), "a//b");
    assert_eq!(normalize_slashes("a\\/b"), "a//b");
}

#[test]
fn all_backslashes() {
    assert_eq!(normalize_slashes("\\\\\\\\"), "////");
}

#[test]
fn rel_path_consecutive_separators_after_dot() {
    assert_eq!(normalize_rel_path(".//foo"), "/foo");
}

// =============================================================================
// Idempotency checks for edge cases
// =============================================================================

#[test]
fn idempotent_empty() {
    let once = normalize_slashes("");
    let twice = normalize_slashes(&once);
    assert_eq!(once, twice);
}

#[test]
fn idempotent_unc() {
    let once = normalize_slashes(r"\\server\share");
    let twice = normalize_slashes(&once);
    assert_eq!(once, twice);
}

#[test]
fn idempotent_rel_path_with_dots() {
    let once = normalize_rel_path("./foo/../bar");
    let twice = normalize_rel_path(&once);
    assert_eq!(once, twice);
}

#[test]
fn idempotent_unicode() {
    let once = normalize_slashes(r"données\résumé");
    let twice = normalize_slashes(&once);
    assert_eq!(once, twice);
}

// =============================================================================
// Length preservation
// =============================================================================

#[test]
fn length_preserved_for_backslash_replacement() {
    let input = r"a\b\c\d\e";
    let output = normalize_slashes(input);
    assert_eq!(input.len(), output.len());
}

#[test]
fn length_preserved_for_already_forward() {
    let input = "a/b/c/d/e";
    let output = normalize_slashes(input);
    assert_eq!(input.len(), output.len());
}
