use std::path::PathBuf;

use tokmd_exclude::{add_exclude_pattern, normalize_exclude_pattern};

#[test]
fn normalizes_workspace_output_paths_for_scan_exclusion() {
    let root = std::env::temp_dir().join("tokmd-exclude-integration-root");
    let out_file = root.join("artifacts").join("receipt.json");
    let bundle_dir = root.join(".handoff");

    let mut excluded = Vec::new();
    let out_pattern = normalize_exclude_pattern(&root, &out_file);
    let bundle_pattern = normalize_exclude_pattern(&root, &bundle_dir);

    assert!(add_exclude_pattern(&mut excluded, out_pattern.clone()));
    assert!(add_exclude_pattern(&mut excluded, bundle_pattern.clone()));

    assert_eq!(out_pattern, "artifacts/receipt.json");
    assert_eq!(bundle_pattern, ".handoff");
    assert_eq!(excluded, vec!["artifacts/receipt.json", ".handoff"]);
}

#[test]
fn keeps_outside_root_paths_stable_and_normalized() {
    let root = std::env::temp_dir().join("tokmd-exclude-integration-root");
    let outside = std::env::temp_dir()
        .join("tokmd-exclude-integration-other")
        .join("bundle.txt");
    let expected = tokmd_path::normalize_rel_path(&outside.to_string_lossy());

    let normalized = normalize_exclude_pattern(&root, &outside);

    assert_eq!(normalized, expected);
    assert!(!normalized.contains('\\'));
}

#[test]
fn dedupes_equivalent_patterns_across_windows_and_posix_styles() {
    let root = PathBuf::from("repo");
    let mut patterns = Vec::new();

    let a = normalize_exclude_pattern(&root, PathBuf::from(r".\ctx-bundle\bundle.txt").as_path());
    let b = normalize_exclude_pattern(&root, PathBuf::from("./ctx-bundle/bundle.txt").as_path());

    assert!(add_exclude_pattern(&mut patterns, a));
    assert!(!add_exclude_pattern(&mut patterns, b));
    assert_eq!(patterns, vec!["ctx-bundle/bundle.txt"]);
}
