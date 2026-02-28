use std::path::PathBuf;

use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

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

// --- Multi-artifact workspace workflow ---

#[test]
fn accumulates_multiple_artifact_paths_from_different_output_dirs() {
    let root = std::env::temp_dir().join("tokmd-exclude-integration-multi");
    let mut excluded = Vec::new();

    let paths = [
        root.join("out").join("lang.json"),
        root.join("out").join("module.json"),
        root.join(".handoff").join("manifest.json"),
        root.join(".handoff").join("bundle.tar"),
        root.join("coverage").join("lcov.info"),
    ];

    for p in &paths {
        let pattern = normalize_exclude_pattern(&root, p);
        add_exclude_pattern(&mut excluded, pattern);
    }

    assert_eq!(excluded.len(), 5);
    assert_eq!(excluded[0], "out/lang.json");
    assert_eq!(excluded[1], "out/module.json");
    assert_eq!(excluded[2], ".handoff/manifest.json");
    assert_eq!(excluded[3], ".handoff/bundle.tar");
    assert_eq!(excluded[4], "coverage/lcov.info");
}

// --- Dedup with mixed styles across multiple insertions ---

#[test]
fn dedupes_across_three_equivalent_forms() {
    let mut patterns = Vec::new();

    assert!(add_exclude_pattern(
        &mut patterns,
        "out/receipt.json".to_string()
    ));
    assert!(!add_exclude_pattern(
        &mut patterns,
        r"out\receipt.json".to_string()
    ));
    assert!(!add_exclude_pattern(
        &mut patterns,
        "./out/receipt.json".to_string()
    ));

    assert_eq!(patterns.len(), 1);
}

// --- Interaction between has_exclude_pattern and add_exclude_pattern ---

#[test]
fn has_exclude_pattern_agrees_with_add_exclude_pattern_rejection() {
    let mut patterns = vec!["dist/app.js".to_string()];

    assert!(has_exclude_pattern(&patterns, "dist/app.js"));
    assert!(has_exclude_pattern(&patterns, r"dist\app.js"));
    assert!(has_exclude_pattern(&patterns, "./dist/app.js"));

    assert!(!add_exclude_pattern(&mut patterns, "dist/app.js".to_string()));
    assert!(!add_exclude_pattern(
        &mut patterns,
        r"dist\app.js".to_string()
    ));
    assert!(!add_exclude_pattern(
        &mut patterns,
        "./dist/app.js".to_string()
    ));

    assert_eq!(patterns.len(), 1);
}

// --- Large batch dedup ---

#[test]
fn large_batch_of_patterns_dedupes_correctly() {
    let mut patterns = Vec::new();

    for i in 0..100 {
        let p = format!("dir{}/file{}.rs", i % 10, i);
        add_exclude_pattern(&mut patterns, p);
    }

    assert_eq!(patterns.len(), 100);

    // Re-add all with ./ prefix — none should be inserted
    for i in 0..100 {
        let p = format!("./dir{}/file{}.rs", i % 10, i);
        assert!(!add_exclude_pattern(&mut patterns, p));
    }

    assert_eq!(patterns.len(), 100);
}

// --- Root is a file (edge case) ---

#[test]
fn root_and_path_are_identical_produces_empty_pattern_after_strip() {
    let root = std::env::temp_dir().join("tokmd-exclude-integration-same");
    let pattern = normalize_exclude_pattern(&root, &root);
    // Stripping root from itself yields ""
    assert_eq!(pattern, "");
}

// --- Patterns with extensions ---

#[test]
fn normalizes_paths_with_various_common_extensions() {
    let root = PathBuf::from("repo");

    let cases = [
        ("src/main.rs", "src/main.rs"),
        ("package.json", "package.json"),
        ("dist/bundle.min.js", "dist/bundle.min.js"),
        (".gitignore", ".gitignore"),
        ("Makefile", "Makefile"),
    ];

    for (input, expected) in &cases {
        let pattern = normalize_exclude_pattern(&root, PathBuf::from(input).as_path());
        assert_eq!(&pattern, expected, "failed for input: {input}");
    }
}
