//! W59 – Exclude pattern matching: normalization, dedup, edge cases.

use std::path::{Path, PathBuf};
use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

// ── normalize_exclude_pattern ────────────────────────────────────────

#[test]
fn normalize_strips_single_dot_slash_prefix() {
    let root = Path::new("repo");
    let got = normalize_exclude_pattern(root, Path::new("./src/main.rs"));
    assert_eq!(got, "src/main.rs");
}

#[test]
fn normalize_strips_repeated_dot_slash_prefixes() {
    let root = Path::new("repo");
    let got = normalize_exclude_pattern(root, Path::new("././././a.rs"));
    assert_eq!(got, "a.rs");
}

#[test]
fn normalize_converts_backslashes_to_forward_slashes() {
    let root = Path::new("repo");
    let got = normalize_exclude_pattern(root, Path::new(r"src\lib\util.rs"));
    assert_eq!(got, "src/lib/util.rs");
}

#[test]
fn normalize_absolute_under_root_strips_root() {
    let root = std::env::temp_dir().join("w59-root");
    let abs = root.join("src").join("main.rs");
    let got = normalize_exclude_pattern(&root, &abs);
    assert_eq!(got, "src/main.rs");
}

#[test]
fn normalize_absolute_outside_root_keeps_full() {
    let root = std::env::temp_dir().join("w59-root");
    let outside = std::env::temp_dir().join("w59-other").join("file.txt");
    let got = normalize_exclude_pattern(&root, &outside);
    let expected = tokmd_path::normalize_rel_path(&outside.to_string_lossy());
    assert_eq!(got, expected);
}

#[test]
fn normalize_preserves_dotfiles() {
    let root = Path::new("project");
    let got = normalize_exclude_pattern(root, Path::new(".gitignore"));
    assert_eq!(got, ".gitignore");
}

#[test]
fn normalize_preserves_hidden_directories() {
    let root = Path::new("project");
    let got = normalize_exclude_pattern(root, Path::new(".config/settings.json"));
    assert_eq!(got, ".config/settings.json");
}

#[test]
fn normalize_empty_relative_path() {
    let root = Path::new("project");
    let got = normalize_exclude_pattern(root, Path::new(""));
    assert_eq!(got, "");
}

#[test]
fn normalize_single_segment_file() {
    let root = Path::new("project");
    let got = normalize_exclude_pattern(root, Path::new("Makefile"));
    assert_eq!(got, "Makefile");
}

#[test]
fn normalize_deeply_nested_path() {
    let root = Path::new("repo");
    let got = normalize_exclude_pattern(root, Path::new("a/b/c/d/e/f.rs"));
    assert_eq!(got, "a/b/c/d/e/f.rs");
}

#[test]
fn normalize_preserves_double_dot_parent_refs() {
    let root = Path::new("repo");
    let got = normalize_exclude_pattern(root, Path::new("../sibling/file.txt"));
    assert_eq!(got, "../sibling/file.txt");
}

// ── has_exclude_pattern ──────────────────────────────────────────────

#[test]
fn has_exclude_finds_exact_match() {
    let existing = vec!["src/main.rs".to_string()];
    assert!(has_exclude_pattern(&existing, "src/main.rs"));
}

#[test]
fn has_exclude_finds_backslash_variant() {
    let existing = vec!["src/main.rs".to_string()];
    assert!(has_exclude_pattern(&existing, r"src\main.rs"));
}

#[test]
fn has_exclude_finds_dot_slash_variant() {
    let existing = vec!["src/main.rs".to_string()];
    assert!(has_exclude_pattern(&existing, "./src/main.rs"));
}

#[test]
fn has_exclude_returns_false_for_absent_pattern() {
    let existing = vec!["src/main.rs".to_string()];
    assert!(!has_exclude_pattern(&existing, "src/lib.rs"));
}

#[test]
fn has_exclude_empty_existing_list() {
    let existing: Vec<String> = vec![];
    assert!(!has_exclude_pattern(&existing, "any/path"));
}

#[test]
fn has_exclude_empty_pattern_against_nonempty_list() {
    let existing = vec!["src/main.rs".to_string()];
    assert!(!has_exclude_pattern(&existing, ""));
}

#[test]
fn has_exclude_multiple_patterns_finds_second() {
    let existing = vec!["a/b.rs".to_string(), "c/d.rs".to_string()];
    assert!(has_exclude_pattern(&existing, "c/d.rs"));
}

#[test]
fn has_exclude_case_sensitive() {
    let existing = vec!["SRC/Main.rs".to_string()];
    assert!(!has_exclude_pattern(&existing, "src/main.rs"));
    assert!(has_exclude_pattern(&existing, "SRC/Main.rs"));
}

// ── add_exclude_pattern ──────────────────────────────────────────────

#[test]
fn add_returns_true_for_new_pattern() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "src/a.rs".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_returns_false_for_exact_duplicate() {
    let mut patterns = vec!["src/a.rs".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, "src/a.rs".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_returns_false_for_backslash_duplicate() {
    let mut patterns = vec!["src/a.rs".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, r"src\a.rs".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_returns_false_for_dot_slash_duplicate() {
    let mut patterns = vec!["src/a.rs".to_string()];
    assert!(!add_exclude_pattern(
        &mut patterns,
        "./src/a.rs".to_string()
    ));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_rejects_empty_string() {
    let mut patterns = vec![];
    assert!(!add_exclude_pattern(&mut patterns, String::new()));
    assert!(patterns.is_empty());
}

#[test]
fn add_accumulates_distinct_patterns() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "a/b.rs".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "c/d.rs".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "e/f.rs".to_string()));
    assert_eq!(patterns.len(), 3);
}

// ── Combined workflows ───────────────────────────────────────────────

#[test]
fn normalize_then_add_dedupes_cross_platform_paths() {
    let root = PathBuf::from("project");
    let mut patterns = vec![];
    let p1 = normalize_exclude_pattern(&root, Path::new(r".\out\result.json"));
    let p2 = normalize_exclude_pattern(&root, Path::new("./out/result.json"));
    assert!(add_exclude_pattern(&mut patterns, p1));
    assert!(!add_exclude_pattern(&mut patterns, p2));
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0], "out/result.json");
}

#[test]
fn add_then_has_roundtrip() {
    let mut patterns = vec![];
    add_exclude_pattern(&mut patterns, "dist/bundle.js".to_string());
    assert!(has_exclude_pattern(&patterns, "dist/bundle.js"));
    assert!(has_exclude_pattern(&patterns, r"dist\bundle.js"));
    assert!(has_exclude_pattern(&patterns, "./dist/bundle.js"));
}

#[test]
fn batch_insert_with_mixed_styles_dedupes() {
    let mut patterns = vec![];
    let inputs = [
        "out/lang.json",
        r"out\lang.json",
        "./out/lang.json",
        r".\out\lang.json",
    ];
    for input in &inputs {
        add_exclude_pattern(&mut patterns, (*input).to_string());
    }
    assert_eq!(patterns.len(), 1);
}

#[test]
fn root_equal_to_path_yields_empty() {
    let root = std::env::temp_dir().join("w59-same");
    let got = normalize_exclude_pattern(&root, &root);
    assert_eq!(got, "");
}
