use std::path::Path;
use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

fn tmp_root() -> std::path::PathBuf {
    std::env::temp_dir().join("tokmd-exclude-test-root")
}

// ── normalize_exclude_pattern: relative paths ──────────────────────

#[test]
fn normalize_strips_leading_dot_slash() {
    let root = tmp_root();
    assert_eq!(
        normalize_exclude_pattern(&root, Path::new("./dist/bundle.js")),
        "dist/bundle.js"
    );
}

#[test]
fn normalize_converts_backslashes_to_forward_slashes() {
    let root = tmp_root();
    assert_eq!(
        normalize_exclude_pattern(&root, Path::new("out\\build\\app.js")),
        "out/build/app.js"
    );
}

#[test]
fn normalize_plain_relative_path_unchanged() {
    let root = tmp_root();
    assert_eq!(
        normalize_exclude_pattern(&root, Path::new("src/lib.rs")),
        "src/lib.rs"
    );
}

// ── normalize_exclude_pattern: absolute paths ──────────────────────

#[test]
fn normalize_absolute_under_root_strips_root_prefix() {
    let root = tmp_root();
    let abs = root.join("target").join("debug").join("app");
    assert_eq!(normalize_exclude_pattern(&root, &abs), "target/debug/app");
}

#[test]
fn normalize_absolute_outside_root_keeps_full_path() {
    let root = tmp_root();
    let outside = std::env::temp_dir()
        .join("tokmd-exclude-test-other")
        .join("file.txt");
    let result = normalize_exclude_pattern(&root, &outside);
    assert!(result.contains("tokmd-exclude-test-other"));
    assert!(result.contains("file.txt"));
}

// ── normalize_exclude_pattern: edge cases ──────────────────────────

#[test]
fn normalize_single_filename_stays_single_filename() {
    let root = tmp_root();
    assert_eq!(
        normalize_exclude_pattern(&root, Path::new("Cargo.toml")),
        "Cargo.toml"
    );
}

#[test]
fn normalize_deeply_nested_path() {
    let root = tmp_root();
    let deep = Path::new("a/b/c/d/e/f/g.rs");
    assert_eq!(normalize_exclude_pattern(&root, deep), "a/b/c/d/e/f/g.rs");
}

#[test]
fn normalize_dot_in_directory_names_preserved() {
    let root = tmp_root();
    assert_eq!(
        normalize_exclude_pattern(&root, Path::new(".hidden/secret.txt")),
        ".hidden/secret.txt"
    );
}

#[test]
fn normalize_multiple_leading_dot_slash_strips_once() {
    let root = tmp_root();
    let result = normalize_exclude_pattern(&root, Path::new("./a/b.rs"));
    assert_eq!(result, "a/b.rs");
}

// ── has_exclude_pattern: matching ──────────────────────────────────

#[test]
fn has_pattern_matches_exact() {
    let patterns = vec!["dist/out".to_string()];
    assert!(has_exclude_pattern(&patterns, "dist/out"));
}

#[test]
fn has_pattern_matches_after_normalization_dot_slash() {
    let patterns = vec!["dist/out".to_string()];
    assert!(has_exclude_pattern(&patterns, "./dist/out"));
}

#[test]
fn has_pattern_matches_after_normalization_backslash() {
    let patterns = vec!["dist/out".to_string()];
    assert!(has_exclude_pattern(&patterns, "dist\\out"));
}

#[test]
fn has_pattern_no_match_for_different_path() {
    let patterns = vec!["dist/out".to_string()];
    assert!(!has_exclude_pattern(&patterns, "build/out"));
}

#[test]
fn has_pattern_empty_list_never_matches() {
    let patterns: Vec<String> = vec![];
    assert!(!has_exclude_pattern(&patterns, "anything"));
}

#[test]
fn has_pattern_case_sensitive() {
    let patterns = vec!["Dist/Out".to_string()];
    assert!(!has_exclude_pattern(&patterns, "dist/out"));
}

// ── add_exclude_pattern: insertion ─────────────────────────────────

#[test]
fn add_pattern_inserts_new_pattern() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "build/out".to_string()));
    assert_eq!(patterns, vec!["build/out"]);
}

#[test]
fn add_pattern_rejects_empty_string() {
    let mut patterns = vec![];
    assert!(!add_exclude_pattern(&mut patterns, String::new()));
    assert!(patterns.is_empty());
}

#[test]
fn add_pattern_rejects_exact_duplicate() {
    let mut patterns = vec!["a/b".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, "a/b".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_pattern_rejects_normalized_duplicate_backslash() {
    let mut patterns = vec!["a/b".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, "a\\b".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_pattern_rejects_normalized_duplicate_dot_slash() {
    let mut patterns = vec!["a/b".to_string()];
    assert!(!add_exclude_pattern(&mut patterns, "./a/b".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_pattern_allows_multiple_distinct() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "a".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "b".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "c/d".to_string()));
    assert_eq!(patterns.len(), 3);
}

// ── cross-function: round-trip ─────────────────────────────────────

#[test]
fn normalize_then_has_pattern_round_trip() {
    let root = Path::new("/project");
    let normalized = normalize_exclude_pattern(root, Path::new("./out/bundle.js"));
    let patterns = vec![normalized];
    assert!(has_exclude_pattern(&patterns, "out/bundle.js"));
    assert!(has_exclude_pattern(&patterns, "./out/bundle.js"));
    assert!(has_exclude_pattern(&patterns, "out\\bundle.js"));
}

#[test]
fn add_then_has_pattern_round_trip() {
    let mut patterns = vec![];
    add_exclude_pattern(&mut patterns, "./target/debug".to_string());
    assert!(has_exclude_pattern(&patterns, "target/debug"));
    assert!(has_exclude_pattern(&patterns, "./target/debug"));
    assert!(has_exclude_pattern(&patterns, "target\\debug"));
}
