//! Deep edge-case tests for module key derivation.

use proptest::prelude::*;
use tokmd_module_key::{module_key, module_key_from_normalized};

// ── Paths with dots (./src, ../lib) ─────────────────────────────────

#[test]
fn dot_slash_src_is_stripped() {
    let roots = vec!["src".into()];
    assert_eq!(module_key("./src/lib.rs", &roots, 2), "src");
}

#[test]
fn double_dot_slash_is_not_stripped() {
    let roots = vec!["src".into()];
    // "../lib" — the ".." is the first dir segment, not a module root
    assert_eq!(module_key("../lib/foo.rs", &roots, 2), "..");
}

#[test]
fn dot_slash_dot_slash_fully_stripped() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("././crates/foo/lib.rs", &roots, 2), "crates/foo");
}

#[test]
fn dot_in_directory_name_not_confused_with_dot_segment() {
    let roots = vec![".config".into()];
    assert_eq!(
        module_key(".config/settings/app.toml", &roots, 2),
        ".config/settings"
    );
}

#[test]
fn dotdot_as_module_root() {
    // ".." as a module root should still work via matching
    let roots = vec!["..".into()];
    assert_eq!(module_key("../lib/foo.rs", &roots, 2), "../lib");
}

// ── Deeply nested paths (10+ levels) ────────────────────────────────

#[test]
fn ten_level_nesting_with_root() {
    let roots = vec!["a".into()];
    let path = "a/b/c/d/e/f/g/h/i/j/file.rs";
    assert_eq!(module_key(path, &roots, 5), "a/b/c/d/e");
}

#[test]
fn ten_level_nesting_depth_exceeds_dirs() {
    let roots = vec!["a".into()];
    let path = "a/b/c/d/e/f/g/h/i/j/file.rs";
    // 10 dir segments, depth 20 should use all 10
    assert_eq!(module_key(path, &roots, 20), "a/b/c/d/e/f/g/h/i/j");
}

#[test]
fn deep_nesting_without_root_returns_first_dir() {
    let roots: Vec<String> = vec![];
    let path = "x/y/z/a/b/c/d/e/f/g/h/i.rs";
    assert_eq!(module_key(path, &roots, 5), "x");
}

#[test]
fn fifteen_level_nesting_depth_3() {
    let roots = vec!["root".into()];
    let segments: Vec<&str> = (0..14).map(|_| "sub").collect();
    let path = format!("root/{}/file.rs", segments.join("/"));
    let key = module_key(&path, &roots, 3);
    assert_eq!(key.split('/').count(), 3);
}

// ── Windows-style paths (backslashes) ───────────────────────────────

#[test]
fn pure_backslash_path() {
    let roots = vec!["src".into()];
    assert_eq!(module_key(r"src\main.rs", &roots, 2), "src");
}

#[test]
fn backslash_with_root_and_depth() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key(r"crates\tokmd\src\lib.rs", &roots, 3),
        "crates/tokmd/src"
    );
}

#[test]
fn mixed_slash_backslash() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key(r"crates/foo\bar/baz\qux.rs", &roots, 4),
        "crates/foo/bar/baz"
    );
}

#[test]
fn dot_backslash_prefix() {
    let roots = vec!["lib".into()];
    assert_eq!(module_key(r".\lib\mod.rs", &roots, 2), "lib");
}

// ── Empty path segments ─────────────────────────────────────────────

#[test]
fn consecutive_slashes_skipped_in_normalized() {
    let roots = vec!["a".into()];
    assert_eq!(
        module_key_from_normalized("a///b///c/file.rs", &roots, 3),
        "a/b/c"
    );
}

#[test]
fn trailing_slash_before_filename_in_normalized() {
    let roots = vec!["src".into()];
    // "src//lib.rs" — empty segment between slashes
    assert_eq!(module_key_from_normalized("src//lib.rs", &roots, 2), "src");
}

#[test]
fn only_slashes_yields_root() {
    assert_eq!(module_key("///", &[], 2), "(root)");
}

// ── Unicode path components ─────────────────────────────────────────

#[test]
fn unicode_dir_chinese() {
    let roots = vec!["项目".into()];
    assert_eq!(module_key("项目/源码/main.rs", &roots, 2), "项目/源码");
}

#[test]
fn unicode_dir_emoji() {
    let roots = vec!["📁".into()];
    assert_eq!(module_key("📁/data/file.txt", &roots, 2), "📁/data");
}

#[test]
fn unicode_dir_japanese() {
    let roots = vec!["ソース".into()];
    assert_eq!(module_key("ソース/コア/lib.rs", &roots, 2), "ソース/コア");
}

#[test]
fn unicode_mixed_with_ascii() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates/données/src/lib.rs", &roots, 2),
        "crates/données"
    );
}

#[test]
fn unicode_non_root_returns_first_segment() {
    let roots: Vec<String> = vec![];
    assert_eq!(module_key("über/cool/file.rs", &roots, 3), "über");
}

// ── Property test: deterministic for same input ─────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn module_key_is_deterministic(
        dirs in prop::collection::vec("[a-zA-Z0-9_]+", 1..8),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..4),
        depth in 0usize..10
    ) {
        let path = format!("{}/{}", dirs.join("/"), filename);
        let k1 = module_key(&path, roots, depth);
        let k2 = module_key(&path, roots, depth);
        prop_assert_eq!(&k1, &k2);
    }

    #[test]
    fn module_key_from_normalized_is_deterministic(
        dirs in prop::collection::vec("[a-zA-Z0-9_]+", 1..8),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..4),
        depth in 0usize..10
    ) {
        let path = format!("{}/{}", dirs.join("/"), filename);
        let k1 = module_key_from_normalized(&path, roots, depth);
        let k2 = module_key_from_normalized(&path, roots, depth);
        prop_assert_eq!(&k1, &k2);
    }

    #[test]
    fn deep_nesting_never_panics(
        depth_count in 1usize..20,
        depth in 0usize..25,
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3)
    ) {
        let segments: Vec<String> = (0..depth_count).map(|i| format!("d{i}")).collect();
        let path = format!("{}/file.rs", segments.join("/"));
        let key = module_key(&path, roots, depth);
        prop_assert!(!key.is_empty());
        prop_assert!(!key.contains('\\'));
    }

    #[test]
    fn backslash_and_forward_slash_equivalent(
        dirs in prop::collection::vec("[a-zA-Z0-9_]+", 2..6),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 1usize..5
    ) {
        let forward = format!("{}/{}", dirs.join("/"), filename);
        let backward = format!("{}\\{}", dirs.join("\\"), filename);
        let k_fwd = module_key(&forward, roots, depth);
        let k_bwd = module_key(&backward, roots, depth);
        prop_assert_eq!(&k_fwd, &k_bwd);
    }
}
