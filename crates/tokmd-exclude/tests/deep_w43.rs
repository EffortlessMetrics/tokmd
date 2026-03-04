//! Wave 43 – deep tests for `tokmd-exclude`.

use std::path::Path;

use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

/// Helper: build an absolute root from the temp dir.
fn abs_root(suffix: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("tokmd-excl-w43-{suffix}"))
}

// ── normalize_exclude_pattern ──────────────────────────────────────

#[test]
fn normalize_strips_root_prefix_from_absolute_path() {
    let root = abs_root("a");
    let path = root.join("src").join("main.rs");
    assert_eq!(normalize_exclude_pattern(&root, &path), "src/main.rs");
}

#[test]
fn normalize_strips_root_with_trailing_dirs() {
    let root = abs_root("b");
    let path = root.join("out").join("bundle.js");
    assert_eq!(normalize_exclude_pattern(&root, &path), "out/bundle.js");
}

#[test]
fn normalize_keeps_relative_path_as_is() {
    let root = abs_root("c");
    let path = Path::new("vendor/dep.rs");
    assert_eq!(normalize_exclude_pattern(&root, path), "vendor/dep.rs");
}

#[test]
fn normalize_strips_leading_dot_slash() {
    let root = abs_root("d");
    let path = Path::new("./out/bundle.js");
    assert_eq!(normalize_exclude_pattern(&root, path), "out/bundle.js");
}

#[test]
fn normalize_handles_outside_absolute_path() {
    let root = abs_root("e");
    let outside = abs_root("e-outside").join("file.txt");
    let result = normalize_exclude_pattern(&root, &outside);
    // Cannot strip root → keeps full path (normalized)
    assert!(result.contains("file.txt"));
}

#[test]
fn normalize_handles_dotfiles() {
    let root = abs_root("f");
    let path = root.join(".gitignore");
    assert_eq!(normalize_exclude_pattern(&root, &path), ".gitignore");
}

#[test]
fn normalize_handles_deeply_nested() {
    let root = abs_root("g");
    let path = root.join("b").join("c").join("d").join("e.txt");
    assert_eq!(normalize_exclude_pattern(&root, &path), "b/c/d/e.txt");
}

#[test]
fn normalize_root_equals_path_gives_empty() {
    let root = abs_root("h");
    let result = normalize_exclude_pattern(&root, &root);
    // When path == root, strip_prefix yields "" → normalized to ""
    assert!(result.is_empty());
}

// ── has_exclude_pattern ────────────────────────────────────────────

#[test]
fn has_pattern_finds_exact_match() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "out/bundle"));
}

#[test]
fn has_pattern_matches_with_leading_dot_slash() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "./out/bundle"));
}

#[test]
fn has_pattern_no_false_positive() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(!has_exclude_pattern(&existing, "src/main.rs"));
}

#[test]
fn has_pattern_empty_existing_returns_false() {
    let existing: Vec<String> = vec![];
    assert!(!has_exclude_pattern(&existing, "anything"));
}

#[test]
fn has_pattern_normalizes_backslashes() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "out\\bundle"));
}

#[test]
fn has_pattern_normalizes_both_sides() {
    let existing = vec!["./out\\bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "out/bundle"));
}

// ── add_exclude_pattern ────────────────────────────────────────────

#[test]
fn add_pattern_inserts_new() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "dist".to_string()));
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0], "dist");
}

#[test]
fn add_pattern_rejects_empty() {
    let mut patterns = vec![];
    assert!(!add_exclude_pattern(&mut patterns, String::new()));
    assert!(patterns.is_empty());
}

#[test]
fn add_pattern_rejects_duplicate_exact() {
    let mut patterns = vec!["dist".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, "dist".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_pattern_rejects_duplicate_after_normalization() {
    let mut patterns = vec!["out/bundle".to_string()];
    assert!(!add_exclude_pattern(
        &mut patterns,
        "./out/bundle".to_string()
    ));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_pattern_rejects_backslash_duplicate() {
    let mut patterns = vec!["out/bundle".to_string()];
    assert!(!add_exclude_pattern(
        &mut patterns,
        "out\\bundle".to_string()
    ));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_pattern_multiple_unique_patterns() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "a".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "b".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "c".to_string()));
    assert_eq!(patterns.len(), 3);
}

#[test]
fn add_pattern_preserves_original_form() {
    let mut patterns = vec![];
    add_exclude_pattern(&mut patterns, "out/bundle".to_string());
    assert_eq!(patterns[0], "out/bundle");
}

// ── combined / edge cases ──────────────────────────────────────────

#[test]
fn roundtrip_add_then_has() {
    let mut patterns = vec![];
    add_exclude_pattern(&mut patterns, "target".to_string());
    add_exclude_pattern(&mut patterns, "node_modules".to_string());
    assert!(has_exclude_pattern(&patterns, "target"));
    assert!(has_exclude_pattern(&patterns, "node_modules"));
    assert!(!has_exclude_pattern(&patterns, "vendor"));
}

#[test]
fn add_rejects_normalized_variant_of_existing() {
    let mut patterns = vec![];
    add_exclude_pattern(&mut patterns, "./vendor/lib".to_string());
    // Try adding the same thing without dot-slash
    assert!(!add_exclude_pattern(
        &mut patterns,
        "vendor/lib".to_string()
    ));
    assert_eq!(patterns.len(), 1);
}
