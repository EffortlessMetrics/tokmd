//! W54 – Module key edge-case tests.

use tokmd_module_key::{module_key, module_key_from_normalized};

fn roots(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| (*s).to_string()).collect()
}

// ── Root-level file edge cases ─────────────────────────────────

#[test]
fn bare_filename_is_root() {
    assert_eq!(module_key("README.md", &roots(&[]), 2), "(root)");
}

#[test]
fn dotfile_at_root_is_root() {
    assert_eq!(module_key(".gitignore", &roots(&["crates"]), 2), "(root)");
}

#[test]
fn file_with_no_extension_at_root() {
    assert_eq!(module_key("Makefile", &roots(&["src"]), 1), "(root)");
}

// ── Depth limiting edge cases ──────────────────────────────────

#[test]
fn depth_zero_maps_to_root_segment() {
    // depth 0 → max(1,0)=1, so only first segment
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 0),
        "crates"
    );
}

#[test]
fn depth_one_gives_root_only() {
    assert_eq!(
        module_key("crates/foo/bar/baz.rs", &roots(&["crates"]), 1),
        "crates"
    );
}

#[test]
fn depth_exceeds_available_segments() {
    // only 3 directory segments: crates/foo/src, depth=100
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 100),
        "crates/foo/src"
    );
}

#[test]
fn depth_exact_match() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 3),
        "crates/foo/src"
    );
}

// ── Dot segments in module paths ───────────────────────────────

#[test]
fn interior_dot_segments_skipped() {
    assert_eq!(
        module_key_from_normalized("crates/./foo/src/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn multiple_dot_segments_skipped() {
    assert_eq!(
        module_key_from_normalized("crates/./././foo/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn leading_dot_only_dir_is_root() {
    assert_eq!(
        module_key_from_normalized("./lib.rs", &roots(&["crates"]), 2),
        "(root)"
    );
}

// ── Deeply nested module hierarchies ───────────────────────────

#[test]
fn deeply_nested_non_root_is_first_segment() {
    assert_eq!(
        module_key("src/a/b/c/d/e/f/g.rs", &roots(&["crates"]), 5),
        "src"
    );
}

#[test]
fn deeply_nested_in_module_root() {
    assert_eq!(
        module_key("crates/a/b/c/d/e/f/g.rs", &roots(&["crates"]), 4),
        "crates/a/b/c"
    );
}

// ── Single-component paths ─────────────────────────────────────

#[test]
fn single_dir_file() {
    assert_eq!(module_key("src/lib.rs", &roots(&["src"]), 2), "src");
}

#[test]
fn single_dir_non_root() {
    assert_eq!(module_key("docs/readme.md", &roots(&["crates"]), 2), "docs");
}

// ── Unicode and special chars ──────────────────────────────────

#[test]
fn unicode_module_root() {
    assert_eq!(
        module_key("パッケージ/foo/lib.rs", &roots(&["パッケージ"]), 2),
        "パッケージ/foo"
    );
}

#[test]
fn emoji_directory() {
    assert_eq!(module_key("🚀/src/main.rs", &roots(&["🚀"]), 2), "🚀/src");
}

// ── Windows path normalization in module_key ───────────────────

#[test]
fn windows_backslash_module_key() {
    assert_eq!(
        module_key(r"crates\foo\src\lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn leading_dot_backslash_stripped() {
    assert_eq!(
        module_key(r".\crates\foo\lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

// ── Empty segments in normalized paths ─────────────────────────

#[test]
fn consecutive_slashes_empty_segments_skipped() {
    assert_eq!(
        module_key_from_normalized("crates//foo//src/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

// ── Multiple module roots ──────────────────────────────────────

#[test]
fn second_root_detected() {
    let r = roots(&["crates", "packages"]);
    assert_eq!(module_key("packages/bar/src/lib.rs", &r, 2), "packages/bar");
    assert_eq!(module_key("crates/foo/src/lib.rs", &r, 2), "crates/foo");
    assert_eq!(module_key("other/file.rs", &r, 2), "other");
}

// ── Leading slash stripped ─────────────────────────────────────

#[test]
fn leading_slash_stripped() {
    assert_eq!(
        module_key("/crates/foo/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}
