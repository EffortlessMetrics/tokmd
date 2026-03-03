//! Deep tests for tokmd-path: exhaustive coverage of normalize_slashes
//! and normalize_rel_path with edge cases, unicode, and cross-platform paths.

use tokmd_path::{normalize_rel_path, normalize_slashes};

// ═══════════════════════════════════════════════════════════════════
// normalize_slashes
// ═══════════════════════════════════════════════════════════════════

#[test]
fn slashes_replaces_single_backslash() {
    assert_eq!(normalize_slashes(r"a\b"), "a/b");
}

#[test]
fn slashes_replaces_multiple_backslashes() {
    assert_eq!(normalize_slashes(r"a\b\c\d\e"), "a/b/c/d/e");
}

#[test]
fn slashes_leaves_forward_slashes_alone() {
    assert_eq!(normalize_slashes("a/b/c"), "a/b/c");
}

#[test]
fn slashes_handles_mixed_separators() {
    assert_eq!(normalize_slashes(r"a/b\c/d\e"), "a/b/c/d/e");
}

#[test]
fn slashes_empty_string() {
    assert_eq!(normalize_slashes(""), "");
}

#[test]
fn slashes_single_backslash_becomes_forward() {
    assert_eq!(normalize_slashes("\\"), "/");
}

#[test]
fn slashes_single_forward_unchanged() {
    assert_eq!(normalize_slashes("/"), "/");
}

#[test]
fn slashes_consecutive_backslashes() {
    assert_eq!(normalize_slashes("a\\\\b"), "a//b");
}

#[test]
fn slashes_trailing_backslash() {
    assert_eq!(normalize_slashes(r"foo\bar\"), "foo/bar/");
}

#[test]
fn slashes_leading_backslash() {
    assert_eq!(normalize_slashes(r"\foo\bar"), "/foo/bar");
}

#[test]
fn slashes_windows_drive_path() {
    assert_eq!(
        normalize_slashes(r"C:\Users\me\file.rs"),
        "C:/Users/me/file.rs"
    );
}

#[test]
fn slashes_unc_path() {
    assert_eq!(
        normalize_slashes(r"\\server\share\dir"),
        "//server/share/dir"
    );
}

#[test]
fn slashes_preserves_spaces() {
    assert_eq!(
        normalize_slashes(r"my dir\my file.txt"),
        "my dir/my file.txt"
    );
}

#[test]
fn slashes_preserves_unicode() {
    assert_eq!(
        normalize_slashes(r"données\résumé\日本語.rs"),
        "données/résumé/日本語.rs"
    );
}

#[test]
fn slashes_preserves_dots() {
    assert_eq!(normalize_slashes(r".\foo\..\bar"), "./foo/../bar");
}

#[test]
fn slashes_filename_only_unchanged() {
    assert_eq!(normalize_slashes("file.rs"), "file.rs");
}

#[test]
fn slashes_preserves_length() {
    let input = r"a\b\c\d";
    assert_eq!(normalize_slashes(input).len(), input.len());
}

#[test]
fn slashes_idempotent_manual() {
    let input = r"foo\bar\baz";
    let once = normalize_slashes(input);
    let twice = normalize_slashes(&once);
    assert_eq!(once, twice);
}

#[test]
fn slashes_no_backslashes_in_output_manual() {
    let inputs = [
        r"a\b",
        r"\\unc\path",
        r"C:\dir",
        r".\rel\path",
        "clean/path",
    ];
    for input in &inputs {
        let result = normalize_slashes(input);
        assert!(
            !result.contains('\\'),
            "backslash in output for {input:?}: {result}"
        );
    }
}

#[test]
fn slashes_preserves_hyphens_underscores() {
    assert_eq!(
        normalize_slashes(r"my-crate\some_mod\file-v2.rs"),
        "my-crate/some_mod/file-v2.rs"
    );
}

// ═══════════════════════════════════════════════════════════════════
// normalize_rel_path
// ═══════════════════════════════════════════════════════════════════

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
    assert_eq!(normalize_rel_path("././././foo"), "foo");
}

#[test]
fn rel_preserves_double_dot_prefix() {
    assert_eq!(normalize_rel_path("../lib.rs"), "../lib.rs");
}

#[test]
fn rel_preserves_double_dot_backslash_prefix() {
    assert_eq!(normalize_rel_path(r"..\src\lib.rs"), "../src/lib.rs");
}

#[test]
fn rel_empty_string() {
    assert_eq!(normalize_rel_path(""), "");
}

#[test]
fn rel_bare_dot_slash_yields_empty() {
    assert_eq!(normalize_rel_path("./"), "");
}

#[test]
fn rel_bare_dot_backslash_yields_empty() {
    assert_eq!(normalize_rel_path(r".\"), "");
}

#[test]
fn rel_bare_forward_slash_unchanged() {
    assert_eq!(normalize_rel_path("/"), "/");
}

#[test]
fn rel_absolute_windows_path_normalized() {
    assert_eq!(normalize_rel_path(r"C:\src\main.rs"), "C:/src/main.rs");
}

#[test]
fn rel_preserves_hidden_dotfile() {
    assert_eq!(normalize_rel_path(".hidden"), ".hidden");
}

#[test]
fn rel_dot_dot_not_stripped() {
    // ".." doesn't start with "./" so it should stay
    assert_eq!(normalize_rel_path(".."), "..");
}

#[test]
fn rel_preserves_spaces() {
    assert_eq!(
        normalize_rel_path(r".\my dir\my file.rs"),
        "my dir/my file.rs"
    );
}

#[test]
fn rel_preserves_unicode() {
    assert_eq!(
        normalize_rel_path(r".\données\résumé.rs"),
        "données/résumé.rs"
    );
}

#[test]
fn rel_idempotent_manual() {
    let cases = [
        "./src/lib.rs",
        r".\src\lib.rs",
        "src/lib.rs",
        "../outside",
        "",
        ".",
        ".hidden/file",
    ];
    for input in &cases {
        let once = normalize_rel_path(input);
        let twice = normalize_rel_path(&once);
        assert_eq!(once, twice, "not idempotent for {input:?}");
    }
}

#[test]
fn rel_no_backslashes_in_output_manual() {
    let inputs = [
        r".\foo\bar",
        r"a\b\c",
        r"..\parent\child",
        r"C:\drive\path",
        "clean/path",
    ];
    for input in &inputs {
        let result = normalize_rel_path(input);
        assert!(
            !result.contains('\\'),
            "backslash in output for {input:?}: {result}"
        );
    }
}

#[test]
fn rel_output_never_longer_than_slash_normalized() {
    let inputs = [
        "./src/lib.rs",
        r".\src\lib.rs",
        "././nested/path",
        "plain/path",
    ];
    for input in &inputs {
        let slash_only = normalize_slashes(input);
        let rel = normalize_rel_path(input);
        assert!(
            rel.len() <= slash_only.len(),
            "rel ({}) longer than slashes ({}) for {input:?}",
            rel.len(),
            slash_only.len()
        );
    }
}

#[test]
fn rel_deeply_nested_path_segments_preserved() {
    let input = "a/b/c/d/e/f/g/h/i/j.rs";
    assert_eq!(normalize_rel_path(input), input);
}

#[test]
fn rel_deeply_nested_backslash_path() {
    assert_eq!(normalize_rel_path(r"a\b\c\d\e\f\g.rs"), "a/b/c/d/e/f/g.rs");
}

#[test]
fn rel_extension_chain_preserved() {
    assert_eq!(
        normalize_rel_path("./dist/bundle.min.js"),
        "dist/bundle.min.js"
    );
}

#[test]
fn rel_glob_patterns_preserved() {
    assert_eq!(normalize_rel_path("./**/*.log"), "**/*.log");
    assert_eq!(normalize_rel_path("./build/out?.bin"), "build/out?.bin");
}

// ═══════════════════════════════════════════════════════════════════
// Cross-function consistency
// ═══════════════════════════════════════════════════════════════════

#[test]
fn rel_result_is_suffix_of_slashes_result() {
    let inputs = [
        "./foo/bar",
        r".\foo\bar",
        "plain/path",
        r"a\b",
        "././nested",
    ];
    for input in &inputs {
        let slash = normalize_slashes(input);
        let rel = normalize_rel_path(input);
        assert!(
            slash.ends_with(&rel),
            "rel {rel:?} not suffix of slashes {slash:?} for {input:?}"
        );
    }
}
