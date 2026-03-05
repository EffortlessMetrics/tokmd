//! Deep property-based tests for tokmd-module-key (W75).
//!
//! Covers: key generation from various paths, depth limiting,
//! dot-segment filtering, cross-platform stability, and edge cases.

use proptest::prelude::*;
use tokmd_module_key::{module_key, module_key_from_normalized};

// ── helpers ────────────────────────────────────────────────────────

fn default_roots() -> Vec<String> {
    vec!["crates".into(), "packages".into()]
}

// ── module key generation from various paths ──────────────────────

#[test]
fn root_level_file() {
    assert_eq!(module_key("README.md", &default_roots(), 2), "(root)");
}

#[test]
fn root_level_with_dot_prefix() {
    assert_eq!(module_key("./Cargo.toml", &default_roots(), 2), "(root)");
}

#[test]
fn module_root_crates() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &default_roots(), 2),
        "crates/foo"
    );
}

#[test]
fn module_root_packages() {
    assert_eq!(
        module_key("packages/bar/src/main.rs", &default_roots(), 2),
        "packages/bar"
    );
}

#[test]
fn non_root_directory_uses_first_segment() {
    assert_eq!(module_key("src/lib.rs", &default_roots(), 2), "src");
    assert_eq!(module_key("docs/guide.md", &default_roots(), 2), "docs");
}

// ── depth-limited module keys ──────────────────────────────────────

#[test]
fn depth_1_returns_root_only() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &default_roots(), 1),
        "crates"
    );
}

#[test]
fn depth_3_includes_deeper_segment() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &default_roots(), 3),
        "crates/foo/src"
    );
}

#[test]
fn depth_overflow_uses_available_segments() {
    // Only 3 directory segments (crates/foo/src), depth = 10
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &default_roots(), 10),
        "crates/foo/src"
    );
}

#[test]
fn depth_0_treated_as_1() {
    // module_depth.max(1) means depth=0 acts like depth=1
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &default_roots(), 0),
        "crates"
    );
}

// ── dot-segment filtering ──────────────────────────────────────────

#[test]
fn dot_segments_filtered_in_normalized() {
    let roots = default_roots();
    assert_eq!(
        module_key_from_normalized("crates/./foo/src/lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn dot_only_dir_becomes_root() {
    assert_eq!(
        module_key_from_normalized("./lib.rs", &default_roots(), 2),
        "(root)"
    );
}

#[test]
fn empty_segments_filtered() {
    let roots = default_roots();
    assert_eq!(
        module_key_from_normalized("crates//foo/src/lib.rs", &roots, 2),
        "crates/foo"
    );
}

// ── cross-platform stability ───────────────────────────────────────

#[test]
fn backslash_paths_normalised() {
    assert_eq!(
        module_key(r"crates\foo\src\lib.rs", &default_roots(), 2),
        "crates/foo"
    );
}

#[test]
fn mixed_separator_paths() {
    assert_eq!(
        module_key(r"crates/foo\src\lib.rs", &default_roots(), 2),
        "crates/foo"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// module_key with backslash separators must equal module_key
    /// with forward-slash separators.
    #[test]
    fn cross_platform_separator_equivalence(
        segments in prop::collection::vec("[a-z]{1,8}", 2..5),
    ) {
        let fwd = format!("{}/file.rs", segments.join("/"));
        let bck = format!("{}\\file.rs", segments.join("\\"));
        prop_assert_eq!(
            module_key(&fwd, &[], 2),
            module_key(&bck, &[], 2),
        );
    }

    /// Module key output must never contain backslashes.
    #[test]
    fn module_key_no_backslashes(
        segments in prop::collection::vec("[a-z]{1,8}", 1..6),
    ) {
        let path = format!("{}\\file.rs", segments.join("\\"));
        let key = module_key(&path, &default_roots(), 2);
        prop_assert!(!key.contains('\\'), "key must not contain backslash: {key}");
    }

    /// Root-level files always produce "(root)" regardless of
    /// module roots or depth.
    #[test]
    fn root_file_always_root(
        name in "[a-zA-Z_][a-zA-Z0-9_.]{0,20}",
        depth in 0_usize..10,
    ) {
        let key = module_key(&name, &default_roots(), depth);
        prop_assert_eq!(key, "(root)");
    }
}
