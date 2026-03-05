//! W62 depth tests for exclude-pattern normalization and dedup helpers.

use std::path::Path;
use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

// ===========================================================================
// 1. normalize_exclude_pattern – relative paths
// ===========================================================================

#[test]
fn normalize_relative_dot_prefix() {
    let root = Path::new("/project");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("./out/bundle.js")),
        "out/bundle.js"
    );
}

#[test]
fn normalize_relative_no_prefix() {
    let root = Path::new("/project");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("dist/app.js")),
        "dist/app.js"
    );
}

#[test]
fn normalize_relative_backslash() {
    let root = Path::new("/project");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("out\\bundle.js")),
        "out/bundle.js"
    );
}

#[test]
fn normalize_relative_mixed_slashes() {
    let root = Path::new("/project");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("out/sub\\file.js")),
        "out/sub/file.js"
    );
}

#[test]
fn normalize_relative_single_file() {
    let root = Path::new("/project");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("file.txt")),
        "file.txt"
    );
}

#[test]
fn normalize_relative_deeply_nested() {
    let root = Path::new("/project");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("./a/b/c/d/e.txt")),
        "a/b/c/d/e.txt"
    );
}

// ===========================================================================
// 2. normalize_exclude_pattern – absolute paths under root
// ===========================================================================

#[test]
fn normalize_absolute_under_root() {
    let root = std::env::temp_dir().join("w62-root");
    let abs = root.join("out").join("bundle.js");
    assert_eq!(normalize_exclude_pattern(&root, &abs), "out/bundle.js");
}

#[test]
fn normalize_absolute_nested_under_root() {
    let root = std::env::temp_dir().join("w62-root");
    let abs = root.join("a").join("b").join("c.txt");
    assert_eq!(normalize_exclude_pattern(&root, &abs), "a/b/c.txt");
}

#[test]
fn normalize_absolute_outside_root_preserves_path() {
    let root = std::env::temp_dir().join("w62-root");
    let outside = std::env::temp_dir().join("w62-other").join("file.txt");
    let result = normalize_exclude_pattern(&root, &outside);
    // Should not be empty and should use forward slashes
    assert!(!result.is_empty());
    assert!(!result.contains('\\'));
}

// ===========================================================================
// 3. has_exclude_pattern – basic matching
// ===========================================================================

#[test]
fn has_pattern_exact_match() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "out/bundle"));
}

#[test]
fn has_pattern_dot_prefix_match() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "./out/bundle"));
}

#[test]
fn has_pattern_backslash_match() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "out\\bundle"));
}

#[test]
fn has_pattern_no_match() {
    let existing = vec!["out/bundle".to_string()];
    assert!(!has_exclude_pattern(&existing, "dist/app"));
}

#[test]
fn has_pattern_empty_existing() {
    let existing: Vec<String> = vec![];
    assert!(!has_exclude_pattern(&existing, "out/bundle"));
}

#[test]
fn has_pattern_empty_query() {
    let existing = vec!["out/bundle".to_string()];
    assert!(!has_exclude_pattern(&existing, ""));
}

#[test]
fn has_pattern_multiple_existing() {
    let existing = vec![
        "out/bundle".to_string(),
        "dist/app".to_string(),
        "build/output".to_string(),
    ];
    assert!(has_exclude_pattern(&existing, "dist/app"));
    assert!(has_exclude_pattern(&existing, "./build/output"));
    assert!(!has_exclude_pattern(&existing, "src/main"));
}

// ===========================================================================
// 4. has_exclude_pattern – normalization equivalence
// ===========================================================================

#[test]
fn has_pattern_normalizes_existing_entries() {
    let existing = vec!["./out\\bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "out/bundle"));
}

#[test]
fn has_pattern_double_dot_prefix() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "././out/bundle"));
}

#[test]
fn has_pattern_mixed_normalization() {
    let existing = vec![".\\out\\sub/file".to_string()];
    assert!(has_exclude_pattern(&existing, "out/sub/file"));
}

// ===========================================================================
// 5. add_exclude_pattern – insertion
// ===========================================================================

#[test]
fn add_pattern_inserts_new() {
    let mut pats = vec![];
    assert!(add_exclude_pattern(&mut pats, "out/bundle".to_string()));
    assert_eq!(pats.len(), 1);
}

#[test]
fn add_pattern_rejects_empty() {
    let mut pats = vec![];
    assert!(!add_exclude_pattern(&mut pats, String::new()));
    assert!(pats.is_empty());
}

#[test]
fn add_pattern_rejects_duplicate_exact() {
    let mut pats = vec!["out/bundle".to_string()];
    assert!(!add_exclude_pattern(&mut pats, "out/bundle".to_string()));
    assert_eq!(pats.len(), 1);
}

#[test]
fn add_pattern_rejects_duplicate_normalized() {
    let mut pats = vec!["out/bundle".to_string()];
    assert!(!add_exclude_pattern(&mut pats, "./out/bundle".to_string()));
    assert_eq!(pats.len(), 1);
}

#[test]
fn add_pattern_rejects_duplicate_backslash() {
    let mut pats = vec!["out/bundle".to_string()];
    assert!(!add_exclude_pattern(&mut pats, "out\\bundle".to_string()));
    assert_eq!(pats.len(), 1);
}

#[test]
fn add_pattern_accepts_different_paths() {
    let mut pats = vec!["out/bundle".to_string()];
    assert!(add_exclude_pattern(&mut pats, "dist/app".to_string()));
    assert_eq!(pats.len(), 2);
}

#[test]
fn add_pattern_multiple_sequential() {
    let mut pats = vec![];
    assert!(add_exclude_pattern(&mut pats, "a/b".to_string()));
    assert!(add_exclude_pattern(&mut pats, "c/d".to_string()));
    assert!(add_exclude_pattern(&mut pats, "e/f".to_string()));
    assert!(!add_exclude_pattern(&mut pats, "./a/b".to_string()));
    assert_eq!(pats.len(), 3);
}

// ===========================================================================
// 6. add_exclude_pattern – preserves original form
// ===========================================================================

#[test]
fn add_pattern_preserves_original_string() {
    let mut pats = vec![];
    add_exclude_pattern(&mut pats, "./out/bundle".to_string());
    assert_eq!(pats[0], "./out/bundle");
}

#[test]
fn add_pattern_preserves_backslash_original() {
    let mut pats = vec![];
    add_exclude_pattern(&mut pats, "out\\bundle".to_string());
    assert_eq!(pats[0], "out\\bundle");
}

// ===========================================================================
// 7. Whitespace handling
// ===========================================================================

#[test]
fn has_pattern_with_spaces_in_path() {
    let existing = vec!["out dir/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "out dir/bundle"));
}

#[test]
fn normalize_path_with_spaces() {
    let root = Path::new("/project");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("./out dir/bundle.js")),
        "out dir/bundle.js"
    );
}

// ===========================================================================
// 8. Edge cases
// ===========================================================================

#[test]
fn normalize_single_dot_path() {
    let root = Path::new("/project");
    let result = normalize_exclude_pattern(root, Path::new("."));
    // "." is a valid relative path
    assert!(!result.contains('\\'));
}

#[test]
fn has_pattern_case_sensitive() {
    let existing = vec!["Out/Bundle".to_string()];
    assert!(!has_exclude_pattern(&existing, "out/bundle"));
}

#[test]
fn add_pattern_case_sensitive_treated_different() {
    let mut pats = vec!["Out/Bundle".to_string()];
    assert!(add_exclude_pattern(&mut pats, "out/bundle".to_string()));
    assert_eq!(pats.len(), 2);
}

#[test]
fn has_pattern_with_trailing_slash() {
    let existing = vec!["out/dir/".to_string()];
    // Trailing slash is significant
    assert!(!has_exclude_pattern(&existing, "out/dir"));
}

#[test]
fn add_many_patterns_dedup_works() {
    let mut pats = vec![];
    for i in 0..20 {
        add_exclude_pattern(&mut pats, format!("dir{i}/file"));
    }
    assert_eq!(pats.len(), 20);
    // Re-adding should all be rejected
    for i in 0..20 {
        assert!(!add_exclude_pattern(&mut pats, format!("dir{i}/file")));
    }
    assert_eq!(pats.len(), 20);
}

#[test]
fn add_pattern_dot_prefix_variants_all_dedup() {
    let mut pats = vec![];
    assert!(add_exclude_pattern(&mut pats, "a/b".to_string()));
    assert!(!add_exclude_pattern(&mut pats, "./a/b".to_string()));
    assert!(!add_exclude_pattern(&mut pats, "././a/b".to_string()));
    assert_eq!(pats.len(), 1);
}

#[test]
fn normalize_preserves_deep_nesting() {
    let root = Path::new("/project");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("./a/b/c/d/e/f/g.txt")),
        "a/b/c/d/e/f/g.txt"
    );
}

#[test]
fn has_pattern_with_extension_dots() {
    let existing = vec!["out/file.test.js".to_string()];
    assert!(has_exclude_pattern(&existing, "out/file.test.js"));
    assert!(!has_exclude_pattern(&existing, "out/file.js"));
}

#[test]
fn normalize_hidden_directory() {
    let root = Path::new("/project");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("./.hidden/file.txt")),
        ".hidden/file.txt"
    );
}

#[test]
fn has_pattern_hidden_directory() {
    let existing = vec![".hidden/file.txt".to_string()];
    assert!(has_exclude_pattern(&existing, "./.hidden/file.txt"));
}

// ===========================================================================
// 9. Property-based tests
// ===========================================================================

mod properties {
    use proptest::prelude::*;
    use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};
    use std::path::Path;

    proptest! {
        #[test]
        fn normalize_never_contains_backslash(
            seg1 in "[a-z]{1,8}",
            seg2 in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.[a-z]{1,4}"
        ) {
            let path_str = format!("{seg1}/{seg2}/{file}");
            let root = Path::new("/project");
            let result = normalize_exclude_pattern(root, Path::new(&path_str));
            prop_assert!(!result.contains('\\'), "result contained backslash: {result}");
        }

        #[test]
        fn normalize_strips_dot_prefix(
            seg in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.[a-z]{1,4}"
        ) {
            let path_str = format!("./{seg}/{file}");
            let root = Path::new("/project");
            let result = normalize_exclude_pattern(root, Path::new(&path_str));
            prop_assert!(!result.starts_with("./"), "result starts with ./: {result}");
        }

        #[test]
        fn has_pattern_is_deterministic(
            seg1 in "[a-z]{1,8}",
            seg2 in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.[a-z]{1,4}"
        ) {
            let pattern = format!("{seg1}/{seg2}/{file}");
            let existing = vec![pattern.clone()];
            let r1 = has_exclude_pattern(&existing, &pattern);
            let r2 = has_exclude_pattern(&existing, &pattern);
            prop_assert_eq!(r1, r2);
        }

        #[test]
        fn add_then_has_returns_true(
            seg1 in "[a-z]{1,8}",
            seg2 in "[a-z]{1,8}"
        ) {
            let pattern = format!("{seg1}/{seg2}");
            let mut pats = vec![];
            add_exclude_pattern(&mut pats, pattern.clone());
            prop_assert!(has_exclude_pattern(&pats, &pattern));
        }

        #[test]
        fn add_twice_only_inserts_once(
            seg in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.[a-z]{1,4}"
        ) {
            let pattern = format!("{seg}/{file}");
            let mut pats = vec![];
            let first = add_exclude_pattern(&mut pats, pattern.clone());
            let second = add_exclude_pattern(&mut pats, pattern.clone());
            prop_assert!(first);
            prop_assert!(!second);
            prop_assert_eq!(pats.len(), 1);
        }

        #[test]
        fn dot_prefix_equivalent(
            seg in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.[a-z]{1,4}"
        ) {
            let plain = format!("{seg}/{file}");
            let dotted = format!("./{seg}/{file}");
            let existing = vec![plain.clone()];
            prop_assert!(has_exclude_pattern(&existing, &dotted));
        }

        #[test]
        fn normalize_idempotent(
            seg1 in "[a-z]{1,8}",
            seg2 in "[a-z]{1,8}",
            file in "[a-z]{1,8}\\.[a-z]{1,4}"
        ) {
            let root = Path::new("/project");
            let path_str = format!("{seg1}/{seg2}/{file}");
            let first = normalize_exclude_pattern(root, Path::new(&path_str));
            let second = normalize_exclude_pattern(root, Path::new(&first));
            prop_assert_eq!(first, second);
        }

        #[test]
        fn empty_pattern_never_added(
            seg in "[a-z]{1,8}"
        ) {
            let _ = seg; // unused, just to drive proptest
            let mut pats = vec![];
            let result = add_exclude_pattern(&mut pats, String::new());
            prop_assert!(!result);
            prop_assert!(pats.is_empty());
        }
    }
}
