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
