//! W59 — module key derivation: depth, edge cases, dot segments, sorting.

use tokmd_module_key::{module_key, module_key_from_normalized};

// ═══════════════════════════════════════════════════════════════════
// Basic derivation from various paths
// ═══════════════════════════════════════════════════════════════════

#[test]
fn derive_root_level_various_extensions() {
    for name in ["Cargo.toml", "README.md", ".gitignore", "Makefile.bak"] {
        assert_eq!(module_key(name, &[], 2), "(root)", "failed for {name}");
    }
}

#[test]
fn derive_single_dir_no_roots() {
    assert_eq!(module_key("src/main.rs", &[], 2), "src");
}

#[test]
fn derive_single_dir_matching_root() {
    let roots = vec!["src".into()];
    assert_eq!(module_key("src/main.rs", &roots, 2), "src");
}

#[test]
fn derive_two_dirs_matching_root_depth_2() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("crates/foo/lib.rs", &roots, 2), "crates/foo");
}

#[test]
fn derive_three_dirs_matching_root_depth_2() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 2), "crates/foo");
}

#[test]
fn derive_three_dirs_matching_root_depth_3() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots, 3),
        "crates/foo/src"
    );
}

#[test]
fn derive_non_matching_root_ignores_depth() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("docs/guide/intro.md", &roots, 5), "docs");
}

#[test]
fn derive_backslash_normalized() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key(r"crates\tokmd\src\lib.rs", &roots, 2),
        "crates/tokmd"
    );
}

#[test]
fn derive_mixed_separators() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key(r"crates/tokmd\src/lib.rs", &roots, 2),
        "crates/tokmd"
    );
}

#[test]
fn derive_leading_dot_slash_stripped() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("./crates/foo/lib.rs", &roots, 2), "crates/foo");
}

#[test]
fn derive_leading_slash_stripped() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("/crates/foo/lib.rs", &roots, 2), "crates/foo");
}

// ═══════════════════════════════════════════════════════════════════
// Depth-based module grouping
// ═══════════════════════════════════════════════════════════════════

#[test]
fn depth_zero_clamps_to_one_segment() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("crates/a/b/c.rs", &roots, 0), "crates");
}

#[test]
fn depth_one_returns_root_only() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("crates/a/b/c.rs", &roots, 1), "crates");
}

#[test]
fn depth_two_returns_root_plus_one() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("crates/a/b/c.rs", &roots, 2), "crates/a");
}

#[test]
fn depth_three_returns_root_plus_two() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("crates/a/b/c.rs", &roots, 3), "crates/a/b");
}

#[test]
fn depth_exceeds_available_dirs() {
    let roots = vec!["crates".into()];
    // path has 2 dir segments: crates/foo — depth 100 caps at available
    assert_eq!(module_key("crates/foo/lib.rs", &roots, 100), "crates/foo");
}

#[test]
fn depth_progression_adds_segments() {
    let roots = vec!["crates".into()];
    let path = "crates/a/b/c/d/file.rs";
    let expected = [
        (1, "crates"),
        (2, "crates/a"),
        (3, "crates/a/b"),
        (4, "crates/a/b/c"),
        (5, "crates/a/b/c/d"),
    ];
    for (depth, want) in expected {
        assert_eq!(module_key(path, &roots, depth), want, "depth {depth}");
    }
}

#[test]
fn depth_monotonically_grows_segment_count() {
    let roots = vec!["pkg".into()];
    let path = "pkg/a/b/c/d/e/f.rs";
    let mut prev = 0;
    for d in 0..=8 {
        let count = module_key(path, &roots, d).split('/').count();
        assert!(count >= prev, "depth {d}: {count} < {prev}");
        prev = count;
    }
}

// ═══════════════════════════════════════════════════════════════════
// Edge cases: empty path, single component, deeply nested
// ═══════════════════════════════════════════════════════════════════

#[test]
fn edge_empty_path() {
    assert_eq!(module_key("", &[], 2), "(root)");
}

#[test]
fn edge_single_slash() {
    assert_eq!(module_key("/", &[], 2), "(root)");
}

#[test]
fn edge_dot_only() {
    assert_eq!(module_key(".", &[], 2), "(root)");
}

#[test]
fn edge_dot_slash_only() {
    assert_eq!(module_key("./", &[], 2), "(root)");
}

#[test]
fn edge_filename_no_directory() {
    assert_eq!(module_key("lib.rs", &["crates".into()], 2), "(root)");
}

#[test]
fn edge_deeply_nested_20_levels() {
    let roots = vec!["a".into()];
    let dirs: Vec<&str> = (0..20).map(|_| "x").collect();
    let path = format!("a/{}/file.rs", dirs.join("/"));
    let key = module_key(&path, &roots, 5);
    assert_eq!(key.split('/').count(), 5);
}

#[test]
fn edge_single_char_segments() {
    let roots = vec!["a".into()];
    assert_eq!(module_key("a/b/c/d.rs", &roots, 3), "a/b/c");
}

#[test]
fn edge_hyphenated_segments() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates/tokmd-scan/src/lib.rs", &roots, 2),
        "crates/tokmd-scan"
    );
}

#[test]
fn edge_underscored_segments() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates/my_crate/src/lib.rs", &roots, 2),
        "crates/my_crate"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Dot segment filtering (`.` segments should be filtered out)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn dot_segment_in_middle_skipped() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key_from_normalized("crates/./foo/lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn multiple_dot_segments_all_skipped() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key_from_normalized("crates/./././bar/lib.rs", &roots, 2),
        "crates/bar"
    );
}

#[test]
fn dot_segment_with_depth_3() {
    let roots = vec!["crates".into()];
    // crates/./foo/src/lib.rs → dirs: crates, foo, src (dot filtered)
    assert_eq!(
        module_key_from_normalized("crates/./foo/src/lib.rs", &roots, 3),
        "crates/foo/src"
    );
}

#[test]
fn empty_segments_filtered() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key_from_normalized("crates///foo/lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn dot_only_dir_becomes_root() {
    assert_eq!(module_key_from_normalized("./lib.rs", &[], 2), "(root)");
}

#[test]
fn dot_only_dir_with_root_config() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key_from_normalized("./lib.rs", &roots, 2), "(root)");
}

// ═══════════════════════════════════════════════════════════════════
// Sorting behavior of module keys
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sort_root_before_alpha() {
    let roots = vec!["crates".into()];
    let mut keys = [module_key("crates/z/lib.rs", &roots, 2),
        module_key("README.md", &roots, 2),
        module_key("crates/a/lib.rs", &roots, 2)];
    keys.sort();
    assert_eq!(keys[0], "(root)");
}

#[test]
fn sort_alphabetical_within_module_root() {
    let roots = vec!["crates".into()];
    let mut keys: Vec<String> = [
        "crates/zzz/lib.rs",
        "crates/aaa/lib.rs",
        "crates/mmm/lib.rs",
    ]
    .iter()
    .map(|p| module_key(p, &roots, 2))
    .collect();
    keys.sort();
    assert_eq!(keys, ["crates/aaa", "crates/mmm", "crates/zzz"]);
}

#[test]
fn sort_deterministic_repeated_calls() {
    let roots = vec!["crates".into()];
    let path = "crates/tokmd-format/src/lib.rs";
    let results: Vec<String> = (0..20).map(|_| module_key(path, &roots, 2)).collect();
    assert!(results.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn sort_all_variant_forms_produce_same_key() {
    let roots = vec!["crates".into()];
    let variants = [
        "crates/foo/lib.rs",
        r"crates\foo\lib.rs",
        "./crates/foo/lib.rs",
        r".\crates\foo\lib.rs",
        "/crates/foo/lib.rs",
    ];
    let keys: Vec<String> = variants.iter().map(|p| module_key(p, &roots, 2)).collect();
    assert!(keys.windows(2).all(|w| w[0] == w[1]));
}

// ═══════════════════════════════════════════════════════════════════
// Multiple roots
// ═══════════════════════════════════════════════════════════════════

#[test]
fn multiple_roots_each_matches_independently() {
    let roots = vec!["crates".into(), "packages".into(), "libs".into()];
    assert_eq!(module_key("crates/a/lib.rs", &roots, 2), "crates/a");
    assert_eq!(module_key("packages/b/lib.rs", &roots, 2), "packages/b");
    assert_eq!(module_key("libs/c/lib.rs", &roots, 2), "libs/c");
}

#[test]
fn non_matching_root_returns_first_dir_regardless_of_depth() {
    let roots = vec!["crates".into()];
    for depth in [1, 2, 5, 10] {
        assert_eq!(
            module_key("vendor/lib/deep/file.rs", &roots, depth),
            "vendor",
            "depth {depth}"
        );
    }
}

#[test]
fn root_prefix_no_false_match() {
    // "crates-extra" should NOT match the root "crates"
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates-extra/foo/lib.rs", &roots, 2),
        "crates-extra"
    );
}
