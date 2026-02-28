use std::path::PathBuf;

use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

#[test]
fn given_relative_windows_style_path_when_normalized_then_forward_slash_pattern_is_returned() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from(r".\ctx-bundle\manifest.json");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "ctx-bundle/manifest.json");
}

#[test]
fn given_absolute_path_under_root_when_normalized_then_root_relative_pattern_is_returned() {
    let root = std::env::temp_dir().join("tokmd-exclude-bdd-root");
    let path = root.join(".handoff").join("code.txt");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, ".handoff/code.txt");
}

#[test]
fn given_existing_equivalent_pattern_when_adding_then_pattern_is_not_inserted_twice() {
    let mut existing = vec![r".\ctx-bundle\manifest.json".to_string()];

    let inserted = add_exclude_pattern(&mut existing, "./ctx-bundle/manifest.json".to_string());

    assert!(!inserted);
    assert_eq!(existing.len(), 1);
    assert!(has_exclude_pattern(&existing, "ctx-bundle/manifest.json"));
}

// --- Deeply nested paths ---

#[test]
fn given_deeply_nested_relative_path_when_normalized_then_all_segments_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("a/b/c/d/e/f/g.rs");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "a/b/c/d/e/f/g.rs");
}

#[test]
fn given_deeply_nested_windows_path_when_normalized_then_forward_slashes_used() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from(r".\src\utils\helpers\crypto\hash.rs");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "src/utils/helpers/crypto/hash.rs");
}

// --- Special characters in filenames ---

#[test]
fn given_path_with_spaces_when_normalized_then_spaces_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("my project/some file.txt");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "my project/some file.txt");
}

#[test]
fn given_path_with_dots_when_normalized_then_dots_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from(".hidden/.secret/..config");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, ".hidden/.secret/..config");
}

#[test]
fn given_path_with_hyphens_and_underscores_when_normalized_then_chars_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("my-crate/some_module/file-name_v2.rs");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "my-crate/some_module/file-name_v2.rs");
}

#[test]
fn given_path_with_unicode_when_normalized_then_unicode_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("données/résumé.txt");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "données/résumé.txt");
}

// --- Glob-like patterns in paths ---

#[test]
fn given_path_with_glob_star_when_normalized_then_star_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("*.log");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "*.log");
}

#[test]
fn given_path_with_double_star_glob_when_normalized_then_glob_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("**/*.tmp");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "**/*.tmp");
}

#[test]
fn given_path_with_question_mark_glob_when_normalized_then_glob_preserved() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("build/output?.bin");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "build/output?.bin");
}

// --- has_exclude_pattern edge cases ---

#[test]
fn given_empty_list_when_checking_pattern_then_returns_false() {
    let existing: Vec<String> = vec![];

    assert!(!has_exclude_pattern(&existing, "src/lib.rs"));
}

#[test]
fn given_list_with_backslash_variant_when_checking_forward_slash_then_returns_true() {
    let existing = vec![r"src\utils\mod.rs".to_string()];

    assert!(has_exclude_pattern(&existing, "src/utils/mod.rs"));
}

#[test]
fn given_list_with_dot_slash_prefix_when_checking_without_prefix_then_returns_true() {
    let existing = vec!["./out/bundle.js".to_string()];

    assert!(has_exclude_pattern(&existing, "out/bundle.js"));
}

#[test]
fn given_list_without_prefix_when_checking_with_dot_slash_then_returns_true() {
    let existing = vec!["out/bundle.js".to_string()];

    assert!(has_exclude_pattern(&existing, "./out/bundle.js"));
}

#[test]
fn given_similar_but_different_patterns_when_checking_then_returns_false() {
    let existing = vec!["src/lib.rs".to_string()];

    assert!(!has_exclude_pattern(&existing, "src/lib.rsx"));
    assert!(!has_exclude_pattern(&existing, "src/libs.rs"));
    assert!(!has_exclude_pattern(&existing, "src2/lib.rs"));
}

// --- add_exclude_pattern edge cases ---

#[test]
fn given_multiple_distinct_patterns_when_adding_then_all_inserted() {
    let mut existing = Vec::new();

    assert!(add_exclude_pattern(&mut existing, "a.rs".to_string()));
    assert!(add_exclude_pattern(&mut existing, "b.rs".to_string()));
    assert!(add_exclude_pattern(&mut existing, "c/d.rs".to_string()));

    assert_eq!(existing.len(), 3);
}

#[test]
fn given_pattern_already_present_when_adding_with_backslash_variant_then_not_inserted() {
    let mut existing = vec!["src/main.rs".to_string()];

    let inserted = add_exclude_pattern(&mut existing, r"src\main.rs".to_string());

    assert!(!inserted);
    assert_eq!(existing.len(), 1);
}

#[test]
fn given_pattern_already_present_when_adding_with_dot_slash_prefix_then_not_inserted() {
    let mut existing = vec!["dist/app.js".to_string()];

    let inserted = add_exclude_pattern(&mut existing, "./dist/app.js".to_string());

    assert!(!inserted);
    assert_eq!(existing.len(), 1);
}

// --- Single-segment paths ---

#[test]
fn given_single_filename_when_normalized_then_unchanged() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("Cargo.lock");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "Cargo.lock");
}

#[test]
fn given_single_directory_name_when_normalized_then_unchanged() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("target");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "target");
}

// --- Dot-slash prefix stripping ---

#[test]
fn given_dot_slash_prefix_when_normalized_then_prefix_stripped() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from("./src/lib.rs");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "src/lib.rs");
}

#[test]
fn given_dot_backslash_prefix_when_normalized_then_prefix_stripped() {
    let root = PathBuf::from("repo");
    let path = PathBuf::from(r".\src\lib.rs");

    let pattern = normalize_exclude_pattern(&root, &path);

    assert_eq!(pattern, "src/lib.rs");
}
