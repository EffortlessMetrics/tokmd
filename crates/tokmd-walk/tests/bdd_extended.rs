//! Extended BDD-style tests for tokmd-walk.
//!
//! These tests cover edge cases not addressed by the existing test suite:
//! empty directories, invalid paths, relative path stripping, Unicode
//! filenames, and stress-level nesting.

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use tokmd_walk::{LicenseCandidates, file_size, license_candidates, list_files};

// ============================================================================
// Helpers
// ============================================================================

/// Create a temp directory without git init (WalkBuilder fallback path).
fn plain_tempdir() -> TempDir {
    TempDir::new().expect("failed to create tempdir")
}

/// Create a temp directory with `git init` so gitignore is recognised.
fn git_tempdir() -> TempDir {
    let tmp = TempDir::new().expect("tempdir");
    std::process::Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("git init");
    tmp
}

fn names(files: &[PathBuf]) -> Vec<String> {
    files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

// ============================================================================
// Scenario: completely empty directory
// ============================================================================

#[test]
fn test_given_empty_dir_when_listed_then_returns_empty_vec() {
    // Given a directory with no files at all
    let tmp = plain_tempdir();
    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    // Then the result is an empty vec
    assert!(files.is_empty(), "empty dir should yield empty vec");
}

#[test]
fn test_given_empty_git_dir_when_listed_then_returns_empty_vec() {
    // Given a git-initialised directory with no committed or tracked files
    let tmp = git_tempdir();
    // When we list files
    let files = list_files(tmp.path(), None).unwrap();

    // We only care about filtering out git internals, the test creates them
    let files_filtered: Vec<_> = files.into_iter().filter(|f| !f.to_string_lossy().contains(".git/")).collect();

    assert!(
        files_filtered.is_empty(),
        "empty git dir should yield empty vec, got {:?}",
        names(&files_filtered)
    );
}

// ============================================================================
// Scenario: list_files returns relative paths
// ============================================================================

#[test]
fn test_given_nested_files_when_listed_then_paths_are_relative_to_root() {
    // Given files at various depths
    let tmp = plain_tempdir();
    std::fs::create_dir_all(tmp.path().join("src/module")).unwrap();
    std::fs::write(tmp.path().join("root.txt"), "r").unwrap();
    std::fs::write(tmp.path().join("src/lib.rs"), "l").unwrap();
    std::fs::write(tmp.path().join("src/module/mod.rs"), "m").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let n = names(&files);

    // Then all paths are relative (no absolute prefix)
    for name in &n {
        assert!(
            !name.starts_with('/') && !name.contains(":\\"),
            "path should be relative, got: {}",
            name
        );
    }
    // And the nested paths are preserved
    assert!(n.iter().any(|p| p.contains("src")));
}

// ============================================================================
// Scenario: deeply nested directory structure (stress)
// ============================================================================

#[test]
fn test_given_10_levels_deep_when_listed_then_file_found() {
    // Given a file nested 10 directories deep
    let tmp = plain_tempdir();
    let mut deep = tmp.path().to_path_buf();
    for i in 0..10 {
        deep = deep.join(format!("d{i}"));
    }
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("leaf.txt"), "leaf").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();

    // Then the deeply nested file is found
    assert_eq!(files.len(), 1);
    assert!(
        files[0].to_string_lossy().contains("leaf.txt"),
        "should find leaf.txt"
    );
}

// ============================================================================
// Scenario: files with spaces in names
// ============================================================================

#[test]
fn test_given_files_with_spaces_when_listed_then_included() {
    // Given files whose names contain spaces
    let tmp = plain_tempdir();
    std::fs::write(tmp.path().join("my file.txt"), "a").unwrap();
    std::fs::write(tmp.path().join("another file.rs"), "b").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let n = names(&files);

    // Then both files are returned
    assert_eq!(files.len(), 2);
    assert!(n.iter().any(|p| p.contains("my file.txt")));
    assert!(n.iter().any(|p| p.contains("another file.rs")));
}

// ============================================================================
// Scenario: file_size on a large binary file
// ============================================================================

#[test]
fn test_given_large_binary_file_when_size_checked_then_correct() {
    // Given a 10 KB binary file
    let tmp = plain_tempdir();
    let data = vec![0xABu8; 10_240];
    std::fs::write(tmp.path().join("big.bin"), &data).unwrap();

    // When we check its size
    let size = file_size(tmp.path(), Path::new("big.bin")).unwrap();

    // Then size matches exactly
    assert_eq!(size, 10_240);
}

// ============================================================================
// Scenario: file_size on a file with newlines (ensure byte count, not line count)
// ============================================================================

#[test]
fn test_given_multiline_file_when_size_checked_then_bytes_not_lines() {
    // Given a file with 3 lines (each "line\n" = 5 bytes)
    let tmp = plain_tempdir();
    std::fs::write(tmp.path().join("lines.txt"), "line\nline\nline\n").unwrap();

    // When we check its size
    let size = file_size(tmp.path(), Path::new("lines.txt")).unwrap();

    // Then size is byte count (15), not line count (3)
    assert_eq!(size, 15);
}

// ============================================================================
// Scenario: license_candidates with only non-matching files
// ============================================================================

#[test]
fn test_given_only_source_files_when_candidates_checked_then_both_empty() {
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("tests/integration.rs"),
        PathBuf::from("benches/bench.rs"),
        PathBuf::from("README.md"),
        PathBuf::from(".gitignore"),
    ];
    let result = license_candidates(&files);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

// ============================================================================
// Scenario: license_candidates results are disjoint
// ============================================================================

#[test]
fn test_given_mixed_files_when_candidates_checked_then_no_overlap() {
    // Given both license and metadata files mixed together
    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("COPYING"),
        PathBuf::from("NOTICE"),
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
        PathBuf::from("pyproject.toml"),
        PathBuf::from("src/main.rs"),
    ];
    let LicenseCandidates {
        license_files,
        metadata_files,
    } = license_candidates(&files);

    // Then no file appears in both lists
    for lf in &license_files {
        assert!(
            !metadata_files.contains(lf),
            "{} should not be in both lists",
            lf.display()
        );
    }
}

// ============================================================================
// Scenario: list_files with max_files larger than file count
// ============================================================================

#[test]
fn test_given_few_files_when_max_exceeds_count_then_all_returned() {
    // Given 2 files
    let tmp = plain_tempdir();
    std::fs::write(tmp.path().join("a.txt"), "a").unwrap();
    std::fs::write(tmp.path().join("b.txt"), "b").unwrap();

    // When max_files is much larger than the count
    let files = list_files(tmp.path(), Some(100)).unwrap();

    // Then all files are returned
    assert_eq!(files.len(), 2);
}

// ============================================================================
// Scenario: gitignore with wildcard patterns
// ============================================================================

#[test]
fn test_given_gitignore_wildcard_dir_when_listed_then_matches_excluded() {
    // Given a gitignore with a double-star pattern
    let tmp = git_tempdir();
    std::fs::write(tmp.path().join(".gitignore"), "**/generated/*.rs\n").unwrap();
    std::fs::create_dir_all(tmp.path().join("src/generated")).unwrap();
    std::fs::write(tmp.path().join("src/generated/auto.rs"), "gen").unwrap();
    std::fs::write(tmp.path().join("src/hand.rs"), "hand").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let n = names(&files);

    // Then the generated file is excluded
    assert!(n.iter().any(|p| p.contains("hand.rs")));
    assert!(
        !n.iter().any(|p| p.contains("auto.rs")),
        "generated file should be excluded"
    );
}

// ============================================================================
// Scenario: list_files determinism
// ============================================================================

#[test]
fn test_given_many_files_when_listed_twice_then_identical_order() {
    // Given a directory with several files
    let tmp = plain_tempdir();
    for name in ["z.txt", "a.txt", "m.txt", "c.txt", "x.txt"] {
        std::fs::write(tmp.path().join(name), name).unwrap();
    }

    // When listed twice
    let first = list_files(tmp.path(), None).unwrap();
    let second = list_files(tmp.path(), None).unwrap();

    // Then results are identical
    assert_eq!(first, second, "list_files must be deterministic");
}

// ============================================================================
// Scenario: list_files output is sorted
// ============================================================================

#[test]
fn test_given_unsorted_files_when_listed_then_sorted_lexicographically() {
    let tmp = plain_tempdir();
    for name in ["z.rs", "a.rs", "m.rs"] {
        std::fs::write(tmp.path().join(name), "x").unwrap();
    }

    let files = list_files(tmp.path(), None).unwrap();
    let n = names(&files);

    let mut sorted = n.clone();
    sorted.sort();
    assert_eq!(n, sorted, "list_files output must be sorted");
}

// ============================================================================
// Scenario: license_candidates with NOTICE variants
// ============================================================================

#[test]
fn test_given_notice_variants_when_checked_then_all_detected() {
    let files = vec![
        PathBuf::from("NOTICE"),
        PathBuf::from("NOTICE.md"),
        PathBuf::from("NOTICE.txt"),
        PathBuf::from("notice"),
        PathBuf::from("Notice.html"),
    ];
    let result = license_candidates(&files);
    assert_eq!(
        result.license_files.len(),
        5,
        "all NOTICE variants should be detected"
    );
}
