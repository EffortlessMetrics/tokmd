//! W54 – Exclude pattern edge-case tests.

use std::path::Path;
use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

// ── Glob pattern edge cases ────────────────────────────────────

#[test]
fn glob_star_pattern_preserved() {
    let root = Path::new("/project");
    let p = Path::new("*.log");
    assert_eq!(normalize_exclude_pattern(root, p), "*.log");
}

#[test]
fn glob_double_star_pattern_preserved() {
    let root = Path::new("/project");
    let p = Path::new("**/*.rs");
    assert_eq!(normalize_exclude_pattern(root, p), "**/*.rs");
}

#[test]
fn glob_question_mark_preserved() {
    let root = Path::new("/project");
    let p = Path::new("file?.txt");
    assert_eq!(normalize_exclude_pattern(root, p), "file?.txt");
}

#[test]
fn glob_bracket_pattern_preserved() {
    let root = Path::new("/project");
    let p = Path::new("[abc].txt");
    assert_eq!(normalize_exclude_pattern(root, p), "[abc].txt");
}

// ── Nested exclusion patterns ──────────────────────────────────

#[test]
fn nested_dir_pattern() {
    let root = Path::new("/project");
    let p = Path::new("./out/dist/bundle.js");
    assert_eq!(normalize_exclude_pattern(root, p), "out/dist/bundle.js");
}

#[test]
fn deeply_nested_exclusion() {
    let root = Path::new("/project");
    let p = Path::new("./a/b/c/d/e/f.txt");
    assert_eq!(normalize_exclude_pattern(root, p), "a/b/c/d/e/f.txt");
}

// ── Override / dedup patterns ──────────────────────────────────

#[test]
fn add_deduplicates_cross_platform_variants() {
    let mut patterns = vec!["out/bundle.js".to_string()];
    // These are all equivalent after normalization
    assert!(!add_exclude_pattern(
        &mut patterns,
        r"out\bundle.js".to_string()
    ));
    assert!(!add_exclude_pattern(
        &mut patterns,
        "./out/bundle.js".to_string()
    ));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn add_allows_distinct_patterns() {
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "a.txt".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "b.txt".to_string()));
    assert!(add_exclude_pattern(&mut patterns, "c.txt".to_string()));
    assert_eq!(patterns.len(), 3);
}

#[test]
fn has_pattern_finds_backslash_variant() {
    let existing = vec!["src/lib.rs".to_string()];
    assert!(has_exclude_pattern(&existing, r"src\lib.rs"));
}

#[test]
fn has_pattern_finds_dot_slash_variant() {
    let existing = vec!["out/bundle.js".to_string()];
    assert!(has_exclude_pattern(&existing, "./out/bundle.js"));
}

// ── Empty patterns ─────────────────────────────────────────────

#[test]
fn empty_pattern_rejected() {
    let mut patterns = vec![];
    assert!(!add_exclude_pattern(&mut patterns, String::new()));
    assert!(patterns.is_empty());
}

#[test]
fn has_exclude_empty_list() {
    let existing: Vec<String> = vec![];
    assert!(!has_exclude_pattern(&existing, "anything"));
}

// ── Pattern with special regex chars ───────────────────────────

#[test]
fn pattern_with_parentheses() {
    let root = Path::new("/project");
    let p = Path::new("dir(1)/file.txt");
    assert_eq!(normalize_exclude_pattern(root, p), "dir(1)/file.txt");
}

#[test]
fn pattern_with_plus_and_caret() {
    let root = Path::new("/project");
    let p = Path::new("a+b^c/file.rs");
    assert_eq!(normalize_exclude_pattern(root, p), "a+b^c/file.rs");
}

#[test]
fn pattern_with_dollar_sign() {
    let root = Path::new("/project");
    let p = Path::new("$special/file.txt");
    assert_eq!(normalize_exclude_pattern(root, p), "$special/file.txt");
}

// ── Unicode patterns ───────────────────────────────────────────

#[test]
fn unicode_pattern_preserved() {
    let root = Path::new("/project");
    let p = Path::new("日本語/テスト.txt");
    assert_eq!(normalize_exclude_pattern(root, p), "日本語/テスト.txt");
}

#[test]
fn emoji_pattern_preserved() {
    let root = Path::new("/project");
    let p = Path::new("🚀/launch.txt");
    assert_eq!(normalize_exclude_pattern(root, p), "🚀/launch.txt");
}

// ── Whitespace patterns ────────────────────────────────────────

#[test]
fn whitespace_only_pattern_is_accepted() {
    // Whitespace-only is non-empty, so it gets added
    let mut patterns = vec![];
    assert!(add_exclude_pattern(&mut patterns, "   ".to_string()));
    assert_eq!(patterns.len(), 1);
}

#[test]
fn path_with_spaces_preserved() {
    let root = Path::new("/project");
    let p = Path::new("my dir/my file.txt");
    assert_eq!(normalize_exclude_pattern(root, p), "my dir/my file.txt");
}

// ── Determinism ────────────────────────────────────────────────

#[test]
fn normalize_is_deterministic() {
    let root = Path::new("/project");
    let p = Path::new("./src/main.rs");
    let r1 = normalize_exclude_pattern(root, p);
    let r2 = normalize_exclude_pattern(root, p);
    assert_eq!(r1, r2);
}

// ── Idempotence ────────────────────────────────────────────────

#[test]
fn normalize_is_idempotent() {
    let root = Path::new("/project");
    let p = Path::new("./src/main.rs");
    let first = normalize_exclude_pattern(root, p);
    let second = normalize_exclude_pattern(root, Path::new(&first));
    assert_eq!(first, second);
}
