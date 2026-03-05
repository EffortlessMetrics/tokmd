//! W57 depth tests for tokmd-module-key: nested paths, flat paths,
//! root paths, depth limiting, dot-segment filtering, ordering,
//! unicode, and proptest coverage.

use tokmd_module_key::{module_key, module_key_from_normalized};

// ═══════════════════════════════════════════════════════════════════
// Module key derivation — nested paths
// ═══════════════════════════════════════════════════════════════════

#[test]
fn nested_four_levels_depth_2() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates/tokmd/src/commands/run.rs", &roots, 2),
        "crates/tokmd"
    );
}

#[test]
fn nested_four_levels_depth_3() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates/tokmd/src/commands/run.rs", &roots, 3),
        "crates/tokmd/src"
    );
}

#[test]
fn nested_four_levels_depth_4() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates/tokmd/src/commands/run.rs", &roots, 4),
        "crates/tokmd/src/commands"
    );
}

#[test]
fn nested_deep_structure_10_levels() {
    let roots = vec!["src".into()];
    let path = "src/a/b/c/d/e/f/g/h/i/file.rs";
    assert_eq!(module_key(path, &roots, 5), "src/a/b/c/d");
    assert_eq!(module_key(path, &roots, 10), "src/a/b/c/d/e/f/g/h/i");
}

#[test]
fn nested_with_backslash_separators() {
    let roots = vec!["packages".into()];
    assert_eq!(
        module_key(r"packages\web\src\components\Button.tsx", &roots, 3),
        "packages/web/src"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Module key derivation — flat paths
// ═══════════════════════════════════════════════════════════════════

#[test]
fn flat_single_dir_file() {
    let roots = vec!["src".into()];
    assert_eq!(module_key("src/main.rs", &roots, 2), "src");
}

#[test]
fn flat_non_root_single_dir() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("docs/readme.md", &roots, 2), "docs");
}

#[test]
fn flat_non_root_ignores_depth_completely() {
    let roots = vec!["crates".into()];
    for depth in [1, 2, 5, 10, 100] {
        assert_eq!(
            module_key("tools/lint/check.sh", &roots, depth),
            "tools",
            "depth {depth} should not affect non-root key"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// Module key derivation — root-level files
// ═══════════════════════════════════════════════════════════════════

#[test]
fn root_file_with_no_directory() {
    assert_eq!(module_key("Cargo.toml", &[], 2), "(root)");
}

#[test]
fn root_file_with_dot_prefix() {
    assert_eq!(module_key("./Cargo.toml", &[], 2), "(root)");
}

#[test]
fn root_file_with_backslash_dot_prefix() {
    assert_eq!(module_key(r".\Makefile", &[], 2), "(root)");
}

#[test]
fn root_file_leading_slash() {
    assert_eq!(module_key("/README.md", &[], 2), "(root)");
}

#[test]
fn root_empty_string() {
    assert_eq!(module_key("", &[], 2), "(root)");
}

#[test]
fn root_single_slash() {
    assert_eq!(module_key("/", &[], 2), "(root)");
}

// ═══════════════════════════════════════════════════════════════════
// Depth limiting behavior
// ═══════════════════════════════════════════════════════════════════

#[test]
fn depth_zero_clamps_to_one() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("crates/a/b/c.rs", &roots, 0), "crates");
}

#[test]
fn depth_one_only_root_segment() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("crates/foo/bar/baz.rs", &roots, 1), "crates");
}

#[test]
fn depth_exceeds_dir_segments_uses_all() {
    let roots = vec!["crates".into()];
    // Only 2 dir segments: crates/foo
    assert_eq!(module_key("crates/foo/file.rs", &roots, 50), "crates/foo");
}

#[test]
fn depth_progression_monotonically_grows() {
    let roots = vec!["crates".into()];
    let path = "crates/a/b/c/d/e.rs";
    let mut prev_len = 0;
    for depth in 1..=5 {
        let key = module_key(path, &roots, depth);
        assert!(
            key.len() >= prev_len,
            "depth {depth}: key {key:?} shorter than previous"
        );
        prev_len = key.len();
    }
}

// ═══════════════════════════════════════════════════════════════════
// Dot-segment filtering
// ═══════════════════════════════════════════════════════════════════

#[test]
fn from_normalized_dot_segment_skipped_in_middle() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key_from_normalized("crates/./foo/lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn from_normalized_multiple_dot_segments_skipped() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key_from_normalized("crates/./././foo/lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn from_normalized_dot_only_dir_becomes_root() {
    assert_eq!(module_key_from_normalized("./file.rs", &[], 2), "(root)");
}

#[test]
fn from_normalized_empty_segments_filtered() {
    let roots = vec!["src".into()];
    assert_eq!(
        module_key_from_normalized("src///mod/file.rs", &roots, 2),
        "src/mod"
    );
}

#[test]
fn module_key_leading_dot_slash_stripped_before_root_match() {
    let roots = vec!["crates".into()];
    // ./crates/foo/lib.rs -> strip ./ -> crates/foo/lib.rs -> match root
    assert_eq!(module_key("./crates/foo/lib.rs", &roots, 2), "crates/foo");
}

// ═══════════════════════════════════════════════════════════════════
// Deterministic ordering of module keys
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ordering_sorted_module_keys_stable() {
    let roots = vec!["crates".into()];
    let paths = [
        "crates/zzz/lib.rs",
        "crates/aaa/lib.rs",
        "crates/mmm/lib.rs",
    ];
    let mut keys: Vec<String> = paths.iter().map(|p| module_key(p, &roots, 2)).collect();
    let mut keys2 = keys.clone();
    keys.sort();
    keys2.sort();
    assert_eq!(keys, keys2);
    assert_eq!(keys, vec!["crates/aaa", "crates/mmm", "crates/zzz"]);
}

#[test]
fn ordering_root_sorts_before_alpha() {
    let roots = vec!["crates".into()];
    let mut keys = [
        module_key("crates/foo/lib.rs", &roots, 2),
        module_key("Cargo.toml", &roots, 2),
        module_key("src/main.rs", &roots, 2),
    ];
    keys.sort();
    assert_eq!(keys[0], "(root)");
}

#[test]
fn ordering_equivalent_input_forms_same_key() {
    let roots = vec!["crates".into()];
    let variants = [
        "crates/foo/src/lib.rs",
        r"crates\foo\src\lib.rs",
        "./crates/foo/src/lib.rs",
        r".\crates\foo\src\lib.rs",
        "/crates/foo/src/lib.rs",
    ];
    let keys: Vec<String> = variants.iter().map(|p| module_key(p, &roots, 2)).collect();
    assert!(keys.iter().all(|k| k == "crates/foo"));
}

#[test]
fn ordering_repeated_calls_identical() {
    let roots = vec!["crates".into()];
    let path = "crates/tokmd-scan/src/lib.rs";
    let results: Vec<String> = (0..50).map(|_| module_key(path, &roots, 2)).collect();
    assert!(results.iter().all(|r| r == &results[0]));
}

// ═══════════════════════════════════════════════════════════════════
// Unicode directory names
// ═══════════════════════════════════════════════════════════════════

#[test]
fn unicode_cjk_directories() {
    let roots = vec!["项目".into()];
    assert_eq!(module_key("项目/源码/main.rs", &roots, 2), "项目/源码");
}

#[test]
fn unicode_cyrillic_directories() {
    let roots = vec!["проект".into()];
    assert_eq!(module_key("проект/код/main.rs", &roots, 2), "проект/код");
}

#[test]
fn unicode_accented_directories() {
    let roots = vec!["données".into()];
    assert_eq!(
        module_key("données/résumé/file.txt", &roots, 2),
        "données/résumé"
    );
}

#[test]
fn unicode_emoji_directories() {
    let roots = vec!["📁".into()];
    assert_eq!(module_key("📁/📂/📄.txt", &roots, 2), "📁/📂");
}

#[test]
fn unicode_mixed_scripts() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates/日本語-module/src/lib.rs", &roots, 2),
        "crates/日本語-module"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Output format guarantees
// ═══════════════════════════════════════════════════════════════════

#[test]
fn output_no_backslash_ever() {
    let roots = vec!["crates".into()];
    let inputs = [
        r"crates\foo\bar.rs",
        r".\crates\foo\lib.rs",
        r"src\main.rs",
        r"C:\drive\file.rs",
    ];
    for input in &inputs {
        let key = module_key(input, &roots, 3);
        assert!(!key.contains('\\'), "backslash in key for {input:?}: {key}");
    }
}

#[test]
fn output_no_trailing_slash() {
    let roots = vec!["crates".into()];
    for depth in 1..=5 {
        let key = module_key("crates/a/b/c/d/e.rs", &roots, depth);
        assert!(!key.ends_with('/'), "trailing / at depth {depth}: {key}");
    }
}

#[test]
fn output_no_leading_dot_slash() {
    let roots = vec!["crates".into()];
    let key = module_key("./crates/foo/lib.rs", &roots, 2);
    assert!(!key.starts_with("./"), "key starts with ./: {key}");
}

// ═══════════════════════════════════════════════════════════════════
// Multiple roots interactions
// ═══════════════════════════════════════════════════════════════════

#[test]
fn multiple_roots_first_match_wins() {
    // Both "crates" paths match "crates" root
    let roots = vec!["crates".into(), "packages".into()];
    assert_eq!(module_key("crates/a/lib.rs", &roots, 2), "crates/a");
    assert_eq!(module_key("packages/b/lib.rs", &roots, 2), "packages/b");
    assert_eq!(module_key("other/c/lib.rs", &roots, 2), "other");
}

#[test]
fn root_substring_no_false_positive() {
    let roots = vec!["src".into()];
    // "src-extra" is not the same as "src"
    assert_eq!(module_key("src-extra/foo/bar.rs", &roots, 2), "src-extra");
}

// ═══════════════════════════════════════════════════════════════════
// proptest
// ═══════════════════════════════════════════════════════════════════

mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn module_key_deterministic(
            path in "[a-z]{1,5}(/[a-z]{1,5}){0,5}/[a-z]{1,5}\\.rs",
            depth in 1..10usize,
        ) {
            let roots = vec!["crates".into()];
            let k1 = module_key(&path, &roots, depth);
            let k2 = module_key(&path, &roots, depth);
            prop_assert_eq!(&k1, &k2);
        }

        #[test]
        fn module_key_no_backslash_in_output(
            path in "[a-zA-Z\\\\/.]{0,100}",
            depth in 1..10usize,
        ) {
            let roots = vec!["crates".into(), "src".into()];
            let key = module_key(&path, &roots, depth);
            prop_assert!(!key.contains('\\'),
                "backslash in key for {:?}: {}", path, key);
        }

        #[test]
        fn module_key_no_trailing_slash(
            path in "[a-z]{1,5}(/[a-z]{1,5}){1,5}/[a-z]{1,5}\\.rs",
            depth in 1..10usize,
        ) {
            let roots = vec!["crates".into()];
            let key = module_key(&path, &roots, depth);
            prop_assert!(!key.ends_with('/'),
                "trailing slash for {:?}: {}", path, key);
        }

        #[test]
        fn module_key_output_is_root_or_valid_path(
            path in "[a-z]{1,5}(/[a-z]{1,5}){0,4}/[a-z]+\\.rs",
            depth in 1..10usize,
        ) {
            let roots: Vec<String> = vec![];
            let key = module_key(&path, &roots, depth);
            prop_assert!(
                key == "(root)" || key.chars().all(|c| c != '\\'),
                "invalid key for {:?}: {}", path, key
            );
        }

        #[test]
        fn increasing_depth_never_shortens_key(
            path in "[a-z]{1,3}(/[a-z]{1,3}){2,6}/[a-z]+\\.rs",
        ) {
            let first_seg = path.split('/').next().unwrap().to_string();
            let roots = vec![first_seg];
            let mut prev_len = 0;
            for depth in 1..=6 {
                let key = module_key(&path, &roots, depth);
                prop_assert!(key.len() >= prev_len,
                    "key shortened at depth {}: {:?} (prev_len={})", depth, key, prev_len);
                prev_len = key.len();
            }
        }
    }
}
