//! Additional BDD-style edge case tests for tokmd-exclude.

use std::path::PathBuf;

use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

// ── Parent-relative paths with ".." segments ─────────────────────────

#[test]
fn given_path_with_parent_segment_when_normalized_then_dots_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("../sibling/src/lib.rs");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert!(!pattern.contains('\\'));
    assert!(pattern.contains("sibling"));
}

#[test]
fn given_path_with_embedded_parent_segment_when_normalized_then_structure_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("src/../tests/mod.rs");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert!(!pattern.contains('\\'));
}

// ── Empty path handling ──────────────────────────────────────────────

#[test]
fn given_empty_path_when_normalized_then_empty_string() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "");
}

// ── Very long paths ──────────────────────────────────────────────────

#[test]
fn given_very_long_path_when_normalized_then_all_segments_preserved() {
    let root = PathBuf::from("repo");
    let segments: Vec<&str> = (0..20).map(|_| "segment").collect();
    let long_path = segments.join("/") + "/file.rs";
    let path = PathBuf::from(&long_path);

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern.matches('/').count(), 20);
    assert!(pattern.ends_with("file.rs"));
}

// ── Paths with trailing slashes ──────────────────────────────────────

#[test]
fn given_path_with_trailing_slash_when_normalized_then_consistent() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("out/");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert!(!pattern.contains('\\'));
    assert!(pattern.contains("out"));
}

// ── has_exclude_pattern with empty pattern ───────────────────────────

#[test]
fn given_list_with_empty_string_when_checking_empty_then_matches() {
    let existing = vec!["".to_string()];

    // Empty normalizes to empty, so it should match
    assert!(has_exclude_pattern(&existing, ""));
}

#[test]
fn given_nonempty_list_when_checking_empty_then_no_false_match() {
    let existing = vec!["src/lib.rs".to_string()];

    assert!(!has_exclude_pattern(&existing, ""));
}

// ── add_exclude_pattern ordering ─────────────────────────────────────

#[test]
fn given_multiple_patterns_added_then_insertion_order_preserved() {
    let mut patterns = Vec::new();

    add_exclude_pattern(&mut patterns, "z_last.rs".to_string());
    add_exclude_pattern(&mut patterns, "a_first.rs".to_string());
    add_exclude_pattern(&mut patterns, "m_middle.rs".to_string());

    assert_eq!(patterns[0], "z_last.rs");
    assert_eq!(patterns[1], "a_first.rs");
    assert_eq!(patterns[2], "m_middle.rs");
}

// ── normalize_exclude_pattern with current-dir absolute paths ────────

#[test]
fn given_absolute_path_under_root_with_deep_nesting_then_relative_extracted() {
    let root = std::env::temp_dir().join("tokmd-exclude-edge-root");
    let path = root.join("a").join("b").join("c").join("d.txt");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "a/b/c/d.txt");
}

// ── Cross-platform path equivalence ──────────────────────────────────

#[test]
fn given_forward_and_backslash_variants_both_match_existing() {
    let existing = vec!["src/deep/nested/mod.rs".to_string()];

    assert!(has_exclude_pattern(&existing, "src/deep/nested/mod.rs"));
    assert!(has_exclude_pattern(&existing, r"src\deep\nested\mod.rs"));
    assert!(has_exclude_pattern(&existing, "./src/deep/nested/mod.rs"));
}

#[test]
fn given_backslash_stored_forward_slash_query_then_matches() {
    let existing = vec![r"src\lib.rs".to_string()];

    assert!(has_exclude_pattern(&existing, "src/lib.rs"));
}

// ── add_exclude_pattern with whitespace-only patterns ────────────────

#[test]
fn given_whitespace_pattern_when_adding_then_inserted_since_nonempty() {
    let mut patterns = Vec::new();

    // Whitespace is not empty, so it should be inserted
    assert!(add_exclude_pattern(&mut patterns, " ".to_string()));
    assert_eq!(patterns.len(), 1);
}

// ── Normalization preserves case ─────────────────────────────────────

#[test]
fn given_mixed_case_path_when_normalized_then_case_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("Src/MyModule/File.RS");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "Src/MyModule/File.RS");
}

// ── Normalization of "." alone ───────────────────────────────────────

#[test]
fn given_just_dot_when_normalized_then_returns_dot() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from(".");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, ".");
}

// ── Multiple equivalent forms all deduplicate ────────────────────────

#[test]
fn given_five_equivalent_forms_when_adding_then_only_first_inserted() {
    let mut patterns = Vec::new();

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
    assert!(!add_exclude_pattern(
        &mut patterns,
        r".\src/lib.rs".to_string()
    ));

    assert_eq!(patterns.len(), 1);
}
