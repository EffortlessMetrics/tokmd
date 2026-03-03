//! Deep tests for tokmd-exclude: pattern normalization, dedup, edge cases.

use std::path::Path;

use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

// ==============================
// normalize_exclude_pattern
// ==============================

#[test]
fn normalize_strips_root_prefix_for_absolute() {
    let root = std::env::temp_dir().join("tokmd-excl-test-root");
    let abs = root.join("out").join("bundle.js");
    assert_eq!(normalize_exclude_pattern(&root, &abs), "out/bundle.js");
}

#[test]
fn normalize_strips_dot_slash_prefix() {
    let root = std::env::temp_dir().join("tokmd-excl-test-root");
    let rel = Path::new("./dist/app.js");
    assert_eq!(normalize_exclude_pattern(&root, rel), "dist/app.js");
}

#[test]
fn normalize_converts_backslashes() {
    let root = std::env::temp_dir().join("tokmd-excl-test-root");
    let rel = Path::new("src\\main.rs");
    let result = normalize_exclude_pattern(&root, rel);
    assert!(
        !result.contains('\\'),
        "should not contain backslashes: {result}"
    );
}

#[test]
fn normalize_keeps_outside_absolute_paths() {
    let root = std::env::temp_dir().join("tokmd-excl-test-root");
    let outside = std::env::temp_dir()
        .join("tokmd-excl-test-outside")
        .join("file.txt");
    let result = normalize_exclude_pattern(&root, &outside);
    // Cannot strip root, so normalizes as-is
    assert!(result.contains("tokmd-excl-test-outside"));
}

#[test]
fn normalize_plain_relative_unchanged() {
    let root = std::env::temp_dir().join("tokmd-excl-test-root");
    let rel = Path::new("target");
    assert_eq!(normalize_exclude_pattern(&root, rel), "target");
}

#[test]
fn normalize_nested_relative_path() {
    let root = std::env::temp_dir().join("tokmd-excl-test-root");
    let rel = Path::new("build/output/generated.rs");
    assert_eq!(
        normalize_exclude_pattern(&root, rel),
        "build/output/generated.rs"
    );
}

// ==============================
// has_exclude_pattern
// ==============================

#[test]
fn has_pattern_exact_match() {
    let existing = vec!["target".to_string()];
    assert!(has_exclude_pattern(&existing, "target"));
}

#[test]
fn has_pattern_dot_slash_normalized() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "./out/bundle"));
}

#[test]
fn has_pattern_backslash_normalized() {
    let existing = vec!["src/generated".to_string()];
    assert!(has_exclude_pattern(&existing, "src\\generated"));
}

#[test]
fn has_pattern_no_match() {
    let existing = vec!["target".to_string()];
    assert!(!has_exclude_pattern(&existing, "dist"));
}

#[test]
fn has_pattern_empty_existing() {
    let existing: Vec<String> = vec![];
    assert!(!has_exclude_pattern(&existing, "anything"));
}

// ==============================
// add_exclude_pattern
// ==============================

#[test]
fn add_inserts_new_pattern() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "target".to_string()));
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0], "target");
}

#[test]
fn add_rejects_empty() {
    let mut patterns = vec![];
    assert!(!add_exclude_pattern(&mut patterns, String::new()));
    assert!(patterns.is_empty());
}

#[test]
fn add_deduplicates_normalized_pattern() {
    let mut patterns = vec!["out/bundle".to_string()];
    assert!(!add_exclude_pattern(
        &mut patterns,
        "./out/bundle".to_string()
    ));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_deduplicates_backslash_variant() {
    let mut patterns = vec!["src/gen".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, "src\\gen".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_allows_distinct_patterns() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "target".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "dist".to_string()));
    assert!(add_exclude_pattern(
        &mut patterns,
        "node_modules".to_string()
    ));
    assert_eq!(patterns.len(), 3);
}

#[test]
fn add_returns_false_for_exact_duplicate() {
    let mut patterns = vec!["target".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, "target".to_string()));
    assert_eq!(patterns.len(), 1);
}

// ==============================
// Edge cases
// ==============================

#[test]
fn normalize_empty_path() {
    let root = std::env::temp_dir().join("tokmd-excl-test-root");
    let empty = Path::new("");
    // Should not panic
    let _ = normalize_exclude_pattern(&root, empty);
}

#[test]
fn has_pattern_with_whitespace_paths() {
    let existing = vec!["my dir/sub".to_string()];
    assert!(has_exclude_pattern(&existing, "my dir/sub"));
    assert!(!has_exclude_pattern(&existing, "my dir/other"));
}

#[test]
fn add_multiple_then_check_has() {
    let mut patterns = vec![];
    add_exclude_pattern(&mut patterns, "a".to_string());
    add_exclude_pattern(&mut patterns, "b".to_string());
    add_exclude_pattern(&mut patterns, "c".to_string());
    assert!(has_exclude_pattern(&patterns, "a"));
    assert!(has_exclude_pattern(&patterns, "b"));
    assert!(has_exclude_pattern(&patterns, "c"));
    assert!(!has_exclude_pattern(&patterns, "d"));
}
