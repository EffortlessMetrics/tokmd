//! Deep exclude tests (w48): pattern matching, normalization, dedup,
//! glob-like patterns, property-based verification, and edge cases.

use std::path::Path;

use proptest::prelude::*;
use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

// ===========================================================================
// 1. Exclusion pattern matching
// ===========================================================================

#[test]
fn exact_match_found() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(has_exclude_pattern(&existing, "src/lib.rs"));
}

#[test]
fn no_match_for_different_path() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(!has_exclude_pattern(&existing, "src/main.rs"));
}

#[test]
fn backslash_variant_matches_forward_slash() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(has_exclude_pattern(&existing, r"src\lib.rs"));
}

#[test]
fn dot_slash_prefix_matches() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(has_exclude_pattern(&existing, "./src/lib.rs"));
}

#[test]
fn dot_backslash_prefix_matches() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(has_exclude_pattern(&existing, r".\src\lib.rs"));
}

#[test]
fn partial_path_does_not_match() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(!has_exclude_pattern(&existing, "src"));
    assert!(!has_exclude_pattern(&existing, "lib.rs"));
}

// ===========================================================================
// 2. Glob pattern support (preservation through normalization)
// ===========================================================================

#[test]
fn glob_star_star_preserved() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("**/*.log")),
        "**/*.log"
    );
}

#[test]
fn glob_question_mark_preserved() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("out/file?.bin")),
        "out/file?.bin"
    );
}

#[test]
fn glob_single_star_preserved() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("dist/*.js")),
        "dist/*.js"
    );
}

#[test]
fn glob_pattern_added_and_deduped() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "**/*.log".to_string()));
    assert!(!add_exclude_pattern(&mut patterns, "**/*.log".to_string()));
    assert_eq!(patterns.len(), 1);
}

// ===========================================================================
// 3. Multiple exclusion rules
// ===========================================================================

#[test]
fn multiple_distinct_patterns_all_added() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "a.rs".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "b.rs".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "c/d.rs".to_string()));
    assert_eq!(patterns.len(), 3);
}

#[test]
fn mixed_style_duplicates_rejected() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "src/lib.rs".to_string()));
    assert!(!add_exclude_pattern(
        &mut patterns,
        r"src\lib.rs".to_string()
    ));
    assert!(!add_exclude_pattern(
        &mut patterns,
        "./src/lib.rs".to_string()
    ));
    assert!(!add_exclude_pattern(
        &mut patterns,
        r".\src\lib.rs".to_string()
    ));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn batch_100_patterns_then_duplicates() {
    let mut patterns = vec![];
    for i in 0..100 {
        add_exclude_pattern(&mut patterns, format!("dir/f{i}.rs"));
    }
    assert_eq!(patterns.len(), 100);
    for i in 0..100 {
        assert!(!add_exclude_pattern(
            &mut patterns,
            format!("./dir/f{i}.rs")
        ));
    }
    assert_eq!(patterns.len(), 100);
}

#[test]
fn insertion_order_preserved() {
    let mut patterns = vec![];
    add_exclude_pattern(&mut patterns, "z.rs".to_string());
    add_exclude_pattern(&mut patterns, "a.rs".to_string());
    add_exclude_pattern(&mut patterns, "m.rs".to_string());
    assert_eq!(patterns, vec!["z.rs", "a.rs", "m.rs"]);
}

#[test]
fn normalize_and_add_workflow() {
    let root = std::env::temp_dir().join("w48-exclude-workflow");
    let mut excluded = vec![];

    let paths = [
        root.join("out").join("lang.json"),
        root.join("out").join("module.json"),
        root.join(".handoff").join("manifest.json"),
    ];
    for p in &paths {
        let norm = normalize_exclude_pattern(&root, p);
        add_exclude_pattern(&mut excluded, norm);
    }
    assert_eq!(excluded.len(), 3);
    assert_eq!(excluded[0], "out/lang.json");
    assert_eq!(excluded[1], "out/module.json");
    assert_eq!(excluded[2], ".handoff/manifest.json");
}

// ===========================================================================
// 4. Property test: excluded paths are never included
// ===========================================================================

proptest! {
    #[test]
    fn prop_add_then_has_always_finds(
        path in "[a-z]{1,5}(/[a-z]{1,5}){0,3}/[a-z]{1,5}\\.rs",
    ) {
        let mut patterns = vec![];
        add_exclude_pattern(&mut patterns, path.clone());
        prop_assert!(has_exclude_pattern(&patterns, &path));
    }

    #[test]
    fn prop_normalized_never_has_backslash(
        segs in prop::collection::vec("[a-z]{1,5}", 1..5),
    ) {
        let root = Path::new("root");
        let raw = segs.join("\\");
        let result = normalize_exclude_pattern(root, Path::new(&raw));
        prop_assert!(!result.contains('\\'), "backslash in: {result}");
    }

    #[test]
    fn prop_duplicate_forms_rejected(
        base in "[a-z]{1,4}/[a-z]{1,4}\\.rs",
    ) {
        let mut patterns = vec![];
        add_exclude_pattern(&mut patterns, base.clone());
        let dot_slash = format!("./{base}");
        prop_assert!(!add_exclude_pattern(&mut patterns, dot_slash));
        prop_assert_eq!(patterns.len(), 1);
    }
}

// ===========================================================================
// 5. Edge cases: empty patterns, wildcard-only patterns
// ===========================================================================

#[test]
fn empty_pattern_rejected() {
    let mut patterns = vec![];
    assert!(!add_exclude_pattern(&mut patterns, String::new()));
    assert!(patterns.is_empty());
}

#[test]
fn empty_query_does_not_match_nonempty() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(!has_exclude_pattern(&existing, ""));
}

#[test]
fn empty_query_matches_empty_entry() {
    let existing = vec!["".to_string()];
    assert!(has_exclude_pattern(&existing, ""));
}

#[test]
fn whitespace_only_pattern_accepted() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, " ".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn wildcard_star_only_pattern_accepted() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "*".to_string()));
    assert!(has_exclude_pattern(&patterns, "*"));
}

#[test]
fn wildcard_double_star_accepted() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "**".to_string()));
    assert!(has_exclude_pattern(&patterns, "**"));
}

#[test]
fn normalize_empty_path_returns_empty() {
    let root = Path::new("repo");
    assert_eq!(normalize_exclude_pattern(root, Path::new("")), "");
}

#[test]
fn normalize_dot_returns_dot() {
    let root = Path::new("repo");
    assert_eq!(normalize_exclude_pattern(root, Path::new(".")), ".");
}

#[test]
fn normalize_absolute_under_root_strips_prefix() {
    let root = std::env::temp_dir().join("w48-root");
    let path = root.join("src").join("main.rs");
    assert_eq!(normalize_exclude_pattern(&root, &path), "src/main.rs");
}

#[test]
fn normalize_absolute_outside_root_kept() {
    let root = std::env::temp_dir().join("w48-root-a");
    let outside = std::env::temp_dir().join("w48-root-b").join("file.txt");
    let result = normalize_exclude_pattern(&root, &outside);
    assert!(!result.contains('\\'));
    assert!(result.contains("file.txt"));
}

#[test]
fn normalize_preserves_unicode() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("données/résumé.txt")),
        "données/résumé.txt"
    );
}

#[test]
fn normalize_preserves_hidden_dotfiles() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new(".gitignore")),
        ".gitignore"
    );
}

#[test]
fn has_agrees_with_add_rejection() {
    let mut patterns = vec!["dist/app.js".to_string()];
    let equivalents = [
        "dist/app.js",
        r"dist\app.js",
        "./dist/app.js",
        r".\dist\app.js",
    ];
    for eq in &equivalents {
        assert!(has_exclude_pattern(&patterns, eq), "has failed for {eq}");
        assert!(
            !add_exclude_pattern(&mut patterns, eq.to_string()),
            "add accepted {eq}"
        );
    }
    assert_eq!(patterns.len(), 1);
}
