//! W62 depth tests for module-key generation.

use tokmd_module_key::{module_key, module_key_from_normalized};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn roots(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| (*s).to_string()).collect()
}

// ===========================================================================
// 1. Root-level files → "(root)"
// ===========================================================================

#[test]
fn root_plain_file() {
    assert_eq!(module_key("Cargo.toml", &roots(&["crates"]), 2), "(root)");
}

#[test]
fn root_hidden_file() {
    assert_eq!(module_key(".gitignore", &roots(&["crates"]), 2), "(root)");
}

#[test]
fn root_no_extension() {
    assert_eq!(module_key("Makefile", &roots(&["src"]), 3), "(root)");
}

#[test]
fn root_with_leading_dot_slash() {
    assert_eq!(module_key("./README.md", &roots(&["crates"]), 2), "(root)");
}

#[test]
fn root_empty_roots() {
    assert_eq!(module_key("lib.rs", &[], 1), "(root)");
}

// ===========================================================================
// 2. Depth parameter effects
// ===========================================================================

#[test]
fn depth_1_returns_root_segment_only() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 1),
        "crates"
    );
}

#[test]
fn depth_2_returns_two_segments() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn depth_3_returns_three_segments() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 3),
        "crates/foo/src"
    );
}

#[test]
fn depth_exceeds_available_dirs_returns_all_dirs() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 100),
        "crates/foo/src"
    );
}

#[test]
fn depth_0_treated_as_depth_1() {
    // max(1, 0) == 1
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 0),
        "crates"
    );
}

#[test]
fn depth_affects_only_module_root_paths() {
    // Non-root dirs always return first segment regardless of depth
    assert_eq!(
        module_key("src/deep/nested/lib.rs", &roots(&["crates"]), 5),
        "src"
    );
}

// ===========================================================================
// 3. Module root matching
// ===========================================================================

#[test]
fn multiple_roots_first_matched() {
    let r = roots(&["crates", "packages"]);
    assert_eq!(module_key("crates/foo/src/lib.rs", &r, 2), "crates/foo");
}

#[test]
fn multiple_roots_second_matched() {
    let r = roots(&["crates", "packages"]);
    assert_eq!(
        module_key("packages/bar/src/main.rs", &r, 2),
        "packages/bar"
    );
}

#[test]
fn non_root_first_segment_only() {
    assert_eq!(
        module_key("tools/gen/main.rs", &roots(&["crates"]), 2),
        "tools"
    );
}

#[test]
fn root_segment_must_match_exactly() {
    // "crate" is not "crates"
    assert_eq!(
        module_key("crate/foo/lib.rs", &roots(&["crates"]), 2),
        "crate"
    );
}

// ===========================================================================
// 4. Dot-segment filtering
// ===========================================================================

#[test]
fn dot_segment_skipped_in_normalized() {
    assert_eq!(
        module_key_from_normalized("crates/./foo/src/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn multiple_dot_segments_skipped() {
    assert_eq!(
        module_key_from_normalized("crates/././foo/src/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn dot_only_directory_becomes_root() {
    assert_eq!(
        module_key_from_normalized("./lib.rs", &roots(&["crates"]), 2),
        "(root)"
    );
}

// ===========================================================================
// 5. Empty-segment filtering
// ===========================================================================

#[test]
fn double_slash_ignored() {
    assert_eq!(
        module_key_from_normalized("crates//foo/src/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn triple_slash_ignored() {
    assert_eq!(
        module_key_from_normalized("crates///foo/src/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

// ===========================================================================
// 6. Cross-platform path normalization
// ===========================================================================

#[test]
fn backslash_converted_to_forward_slash() {
    assert_eq!(
        module_key("crates\\foo\\src\\lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn mixed_slashes_normalized() {
    assert_eq!(
        module_key("crates/foo\\src\\lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn leading_backslash_stripped() {
    assert_eq!(
        module_key("\\crates\\foo\\lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn leading_dot_backslash_stripped() {
    assert_eq!(module_key(".\\src\\lib.rs", &roots(&["src"]), 2), "src");
}

#[test]
fn leading_forward_slash_stripped() {
    assert_eq!(module_key("/src/lib.rs", &roots(&["src"]), 2), "src");
}

// ===========================================================================
// 7. Nested and deep structures
// ===========================================================================

#[test]
fn very_deep_path_depth_2() {
    assert_eq!(
        module_key("crates/a/b/c/d/e/f.rs", &roots(&["crates"]), 2),
        "crates/a"
    );
}

#[test]
fn very_deep_path_depth_5() {
    assert_eq!(
        module_key("crates/a/b/c/d/e/f.rs", &roots(&["crates"]), 5),
        "crates/a/b/c/d"
    );
}

#[test]
fn nested_non_root() {
    assert_eq!(
        module_key("docs/api/v2/reference/index.md", &roots(&["crates"]), 3),
        "docs"
    );
}

// ===========================================================================
// 8. Edge cases
// ===========================================================================

#[test]
fn empty_path_returns_root() {
    // rsplit_once('/') returns None → "(root)"
    assert_eq!(module_key("", &roots(&["crates"]), 2), "(root)");
}

#[test]
fn single_segment_no_slash_returns_root() {
    assert_eq!(module_key("file.txt", &[], 2), "(root)");
}

#[test]
fn file_directly_under_root_dir() {
    assert_eq!(
        module_key("crates/foo.rs", &roots(&["crates"]), 2),
        "crates"
    );
}

#[test]
fn file_directly_under_root_dir_depth_1() {
    assert_eq!(
        module_key("crates/foo.rs", &roots(&["crates"]), 1),
        "crates"
    );
}

#[test]
fn unicode_directory_names() {
    assert_eq!(
        module_key("crates/日本語/src/lib.rs", &roots(&["crates"]), 2),
        "crates/日本語"
    );
}

#[test]
fn hyphenated_directory_names() {
    assert_eq!(
        module_key("crates/my-crate/src/lib.rs", &roots(&["crates"]), 2),
        "crates/my-crate"
    );
}

#[test]
fn underscored_directory_names() {
    assert_eq!(
        module_key("crates/my_crate/src/lib.rs", &roots(&["crates"]), 2),
        "crates/my_crate"
    );
}

#[test]
fn dotted_directory_name_not_filtered() {
    // ".hidden" is not "." so it should be kept
    assert_eq!(
        module_key("crates/.hidden/src/lib.rs", &roots(&["crates"]), 2),
        "crates/.hidden"
    );
}

#[test]
fn filename_with_no_extension() {
    assert_eq!(
        module_key("crates/foo/Makefile", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn filename_with_multiple_dots() {
    assert_eq!(
        module_key("crates/foo/file.test.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn multiple_module_roots_independent() {
    let r = roots(&["crates", "packages", "libs"]);
    assert_eq!(module_key("libs/core/src/main.rs", &r, 2), "libs/core");
}

// ===========================================================================
// 9. module_key_from_normalized specific
// ===========================================================================

#[test]
fn normalized_root_file() {
    assert_eq!(
        module_key_from_normalized("README.md", &roots(&["crates"]), 2),
        "(root)"
    );
}

#[test]
fn normalized_non_root_dir() {
    assert_eq!(
        module_key_from_normalized("src/main.rs", &roots(&["crates"]), 2),
        "src"
    );
}

#[test]
fn normalized_depth_overflow() {
    assert_eq!(
        module_key_from_normalized("crates/foo/bar/baz.rs", &roots(&["crates"]), 10),
        "crates/foo/bar"
    );
}

#[test]
fn normalized_single_dir_under_root() {
    assert_eq!(
        module_key_from_normalized("crates/foo.rs", &roots(&["crates"]), 2),
        "crates"
    );
}

// ===========================================================================
// 10. Property-based tests
// ===========================================================================

mod properties {
    use proptest::prelude::*;
    use tokmd_module_key::{module_key, module_key_from_normalized};

    fn roots(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }

    proptest! {
        #[test]
        fn key_never_contains_backslash(
            seg1 in "[a-z]{1,8}",
            seg2 in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.rs",
            depth in 1usize..10
        ) {
            let path = format!("{seg1}\\{seg2}\\{file}");
            let key = module_key(&path, &roots(&["crates"]), depth);
            prop_assert!(!key.contains('\\'), "key contained backslash: {key}");
        }

        #[test]
        fn key_is_stable_across_calls(
            seg1 in "[a-z]{1,8}",
            seg2 in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.rs",
            depth in 1usize..10
        ) {
            let path = format!("{seg1}/{seg2}/{file}");
            let r = roots(&["crates"]);
            let k1 = module_key(&path, &r, depth);
            let k2 = module_key(&path, &r, depth);
            prop_assert_eq!(k1, k2);
        }

        #[test]
        fn key_never_ends_with_slash(
            seg1 in "[a-z]{1,8}",
            seg2 in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.rs",
            depth in 1usize..10
        ) {
            let path = format!("{seg1}/{seg2}/{file}");
            let key = module_key(&path, &roots(&["crates"]), depth);
            prop_assert!(!key.ends_with('/'), "key ended with /: {key}");
        }

        #[test]
        fn root_files_always_root(
            file in "[a-zA-Z_][a-zA-Z0-9_.]{0,12}"
        ) {
            let key = module_key(&file, &roots(&["crates"]), 2);
            prop_assert_eq!(key, "(root)");
        }

        #[test]
        fn normalized_key_never_contains_backslash(
            seg1 in "[a-z]{1,8}",
            seg2 in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.rs",
            depth in 1usize..10
        ) {
            let path = format!("{seg1}/{seg2}/{file}");
            let key = module_key_from_normalized(&path, &roots(&["crates"]), depth);
            prop_assert!(!key.contains('\\'), "key contained backslash: {key}");
        }

        #[test]
        fn depth_monotonic_prefix(
            seg1 in "[a-z]{1,6}",
            seg2 in "[a-z]{1,6}",
            seg3 in "[a-z]{1,6}",
            file in "[a-z]{1,6}\\.rs"
        ) {
            let path = format!("{seg1}/{seg2}/{seg3}/{file}");
            let r = roots(&[&seg1]);
            let k1 = module_key(&path, &r, 1);
            let k2 = module_key(&path, &r, 2);
            let k3 = module_key(&path, &r, 3);
            // Smaller depth should be a prefix of larger depth
            prop_assert!(k2.starts_with(&k1), "k2={k2} should start with k1={k1}");
            prop_assert!(k3.starts_with(&k2), "k3={k3} should start with k2={k2}");
        }

        #[test]
        fn forward_and_backslash_produce_same_key(
            seg1 in "[a-z]{1,8}",
            seg2 in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.rs",
            depth in 1usize..10
        ) {
            let fwd = format!("{seg1}/{seg2}/{file}");
            let bkwd = format!("{seg1}\\{seg2}\\{file}");
            let r = roots(&["crates"]);
            let k1 = module_key(&fwd, &r, depth);
            let k2 = module_key(&bkwd, &r, depth);
            prop_assert_eq!(k1, k2);
        }
    }
}
