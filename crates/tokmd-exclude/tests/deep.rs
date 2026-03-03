//! Deep tests for tokmd-exclude: exhaustive coverage of pattern normalization,
//! deduplication, and cross-platform path equivalence.

use std::path::{Path, PathBuf};

use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

// ── normalize_exclude_pattern: relative paths ────────────────────

#[test]
fn normalize_plain_relative_path_unchanged() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("src/lib.rs")),
        "src/lib.rs"
    );
}

#[test]
fn normalize_dot_slash_prefix_stripped() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("./src/lib.rs")),
        "src/lib.rs"
    );
}

#[test]
fn normalize_double_dot_slash_prefix_stripped() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("././src/lib.rs")),
        "src/lib.rs"
    );
}

#[test]
fn normalize_dot_backslash_prefix_stripped() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new(r".\src\lib.rs")),
        "src/lib.rs"
    );
}

#[test]
fn normalize_backslash_path_converted_to_forward() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new(r"a\b\c\d.rs")),
        "a/b/c/d.rs"
    );
}

// ── normalize_exclude_pattern: absolute paths ────────────────────

#[test]
fn normalize_absolute_under_root_strips_prefix() {
    let root = std::env::temp_dir().join("deep-test-root");
    let path = root.join("src").join("main.rs");
    assert_eq!(normalize_exclude_pattern(&root, &path), "src/main.rs");
}

#[test]
fn normalize_absolute_outside_root_kept_normalized() {
    let root = std::env::temp_dir().join("deep-root-a");
    let outside = std::env::temp_dir().join("deep-root-b").join("file.txt");
    let result = normalize_exclude_pattern(&root, &outside);
    assert!(!result.contains('\\'));
    assert!(result.contains("file.txt"));
}

#[test]
fn normalize_absolute_root_itself_yields_empty() {
    let root = std::env::temp_dir().join("deep-root-self");
    assert_eq!(normalize_exclude_pattern(&root, &root), "");
}

// ── normalize_exclude_pattern: special characters ────────────────

#[test]
fn normalize_preserves_spaces_in_path() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("my dir/my file.txt")),
        "my dir/my file.txt"
    );
}

#[test]
fn normalize_preserves_unicode_characters() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("données/résumé.txt")),
        "données/résumé.txt"
    );
}

#[test]
fn normalize_preserves_glob_star_pattern() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("**/*.log")),
        "**/*.log"
    );
}

#[test]
fn normalize_preserves_question_mark_glob() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("out/file?.bin")),
        "out/file?.bin"
    );
}

#[test]
fn normalize_preserves_mixed_case() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("Src/MyMod/File.RS")),
        "Src/MyMod/File.RS"
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
fn normalize_preserves_double_dot_in_filename() {
    let root = Path::new("repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("dir/..config")),
        "dir/..config"
    );
}

// ── normalize_exclude_pattern: edge cases ────────────────────────

#[test]
fn normalize_empty_path_returns_empty() {
    let root = Path::new("repo");
    assert_eq!(normalize_exclude_pattern(root, Path::new("")), "");
}

#[test]
fn normalize_bare_dot_returns_dot() {
    let root = Path::new("repo");
    assert_eq!(normalize_exclude_pattern(root, Path::new(".")), ".");
}

#[test]
fn normalize_parent_relative_preserves_dots() {
    let root = Path::new("repo");
    let result = normalize_exclude_pattern(root, Path::new("../sibling/file.rs"));
    assert!(result.contains("sibling"));
    assert!(!result.contains('\\'));
}

// ── has_exclude_pattern: matching ────────────────────────────────

#[test]
fn has_pattern_matches_exact() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(has_exclude_pattern(&existing, "src/lib.rs"));
}

#[test]
fn has_pattern_matches_backslash_variant() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(has_exclude_pattern(&existing, r"src\lib.rs"));
}

#[test]
fn has_pattern_matches_dot_slash_prefix() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(has_exclude_pattern(&existing, "./src/lib.rs"));
}

#[test]
fn has_pattern_matches_dot_backslash_prefix() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(has_exclude_pattern(&existing, r".\src\lib.rs"));
}

#[test]
fn has_pattern_does_not_match_substring() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(!has_exclude_pattern(&existing, "src/lib"));
    assert!(!has_exclude_pattern(&existing, "lib.rs"));
}

#[test]
fn has_pattern_empty_list_returns_false() {
    let existing: Vec<String> = vec![];
    assert!(!has_exclude_pattern(&existing, "anything"));
}

#[test]
fn has_pattern_empty_query_matches_empty_entry() {
    let existing = vec!["".to_string()];
    assert!(has_exclude_pattern(&existing, ""));
}

#[test]
fn has_pattern_empty_query_does_not_match_nonempty() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(!has_exclude_pattern(&existing, ""));
}

// ── add_exclude_pattern: deduplication ───────────────────────────

#[test]
fn add_rejects_empty_pattern() {
    let mut patterns = vec![];
    assert!(!add_exclude_pattern(&mut patterns, String::new()));
    assert!(patterns.is_empty());
}

#[test]
fn add_inserts_first_pattern() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "a/b.rs".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_rejects_exact_duplicate() {
    let mut patterns = vec!["a/b.rs".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, "a/b.rs".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_rejects_backslash_duplicate() {
    let mut patterns = vec!["a/b.rs".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, r"a\b.rs".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_rejects_dot_slash_duplicate() {
    let mut patterns = vec!["a/b.rs".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, "./a/b.rs".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_rejects_dot_backslash_duplicate() {
    let mut patterns = vec!["a/b.rs".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, r".\a\b.rs".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_accepts_distinct_patterns() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "a.rs".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "b.rs".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "c/d.rs".to_string()));
    assert_eq!(patterns.len(), 3);
}

#[test]
fn add_preserves_insertion_order() {
    let mut patterns = vec![];
    add_exclude_pattern(&mut patterns, "z.rs".to_string());
    add_exclude_pattern(&mut patterns, "a.rs".to_string());
    add_exclude_pattern(&mut patterns, "m.rs".to_string());
    assert_eq!(patterns, vec!["z.rs", "a.rs", "m.rs"]);
}

// ── Batch workflow tests ─────────────────────────────────────────

#[test]
fn batch_add_with_mixed_styles_deduplicates() {
    let mut patterns = vec![];
    let inputs = [
        "src/lib.rs",
        r"src\lib.rs",
        "./src/lib.rs",
        r".\src\lib.rs",
        r".\src/lib.rs",
    ];
    let mut inserted = 0;
    for input in &inputs {
        if add_exclude_pattern(&mut patterns, input.to_string()) {
            inserted += 1;
        }
    }
    assert_eq!(inserted, 1);
    assert_eq!(patterns.len(), 1);
}

#[test]
fn batch_100_distinct_then_100_dot_slash_duplicates() {
    let mut patterns = vec![];
    for i in 0..100 {
        add_exclude_pattern(&mut patterns, format!("dir/file{i}.rs"));
    }
    assert_eq!(patterns.len(), 100);

    for i in 0..100 {
        assert!(!add_exclude_pattern(
            &mut patterns,
            format!("./dir/file{i}.rs")
        ));
    }
    assert_eq!(patterns.len(), 100);
}

#[test]
fn multi_artifact_workflow_accumulates_correctly() {
    let root = std::env::temp_dir().join("deep-multi-artifact");
    let mut excluded = vec![];

    let artifacts = [
        root.join("out").join("lang.json"),
        root.join("out").join("module.json"),
        root.join(".handoff").join("manifest.json"),
    ];

    for artifact in &artifacts {
        let pattern = normalize_exclude_pattern(&root, artifact);
        add_exclude_pattern(&mut excluded, pattern);
    }

    assert_eq!(excluded.len(), 3);
    assert_eq!(excluded[0], "out/lang.json");
    assert_eq!(excluded[1], "out/module.json");
    assert_eq!(excluded[2], ".handoff/manifest.json");
}

// ── Cross-function consistency ───────────────────────────────────

#[test]
fn has_agrees_with_add_rejection() {
    let mut patterns = vec!["dist/app.js".to_string()];

    // All equivalent forms should be found by has_ and rejected by add_
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

#[test]
fn normalize_output_always_lacks_backslashes() {
    let root = Path::new("repo");
    let test_paths = [
        r"a\b\c",
        r".\x\y",
        "src/lib.rs",
        r"src\main.rs",
        "",
        ".",
        r"deep\nested\path\to\file.txt",
    ];
    for p in &test_paths {
        let result = normalize_exclude_pattern(root, Path::new(p));
        assert!(
            !result.contains('\\'),
            "backslash found in result for input {p:?}: {result}"
        );
    }
}

// ── Whitespace and special patterns ──────────────────────────────

#[test]
fn whitespace_only_pattern_is_nonempty_and_added() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, " ".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "  ".to_string()));
    assert_eq!(patterns.len(), 2);
}

#[test]
fn deeply_nested_absolute_path_under_root() {
    let root = std::env::temp_dir().join("deep-nest-root");
    let path = root
        .join("a")
        .join("b")
        .join("c")
        .join("d")
        .join("e")
        .join("f.rs");
    assert_eq!(normalize_exclude_pattern(&root, &path), "a/b/c/d/e/f.rs");
}

#[test]
fn single_segment_filename_unchanged() {
    let root = PathBuf::from("repo");
    assert_eq!(
        normalize_exclude_pattern(&root, Path::new("Cargo.lock")),
        "Cargo.lock"
    );
}

#[test]
fn path_with_extension_chain_preserved() {
    let root = PathBuf::from("repo");
    assert_eq!(
        normalize_exclude_pattern(&root, Path::new("dist/bundle.min.js")),
        "dist/bundle.min.js"
    );
}
