//! Property-based tests for file traversal invariants.
//!
//! These tests exercise `list_files` and `file_size` with arbitrary inputs
//! to verify structural invariants: relative paths, sorting, max_files caps,
//! and file_size accuracy.

use proptest::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;
use tokmd_walk::{file_size, list_files};

// ============================================================================
// Strategies
// ============================================================================

/// Strategy for generating a set of simple filenames (no path separators).
fn arb_filenames(min: usize, max: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::hash_set("[a-z]{1,8}\\.[a-z]{1,4}", min..=max)
        .prop_map(|s| s.into_iter().collect())
}

/// Strategy for file content (arbitrary bytes of bounded length).
fn arb_content() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..=4096)
}

/// Strategy for subdirectory depth (0 = root, up to 4 levels).
fn arb_dir_depth() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-z]{1,6}", 0..=4)
}

/// Strategy for max_files parameter.
fn arb_max_files() -> impl Strategy<Value = Option<usize>> {
    prop_oneof![Just(None), Just(Some(0)), (1..=20usize).prop_map(Some),]
}

// ============================================================================
// list_files invariants
// ============================================================================

proptest! {
    /// All paths returned by list_files are relative (no root prefix).
    #[test]
    fn list_files_returns_relative_paths(
        filenames in arb_filenames(1, 10),
    ) {
        let tmp = TempDir::new().unwrap();
        for name in &filenames {
            std::fs::write(tmp.path().join(name), "x").unwrap();
        }

        let files = list_files(tmp.path(), None).unwrap();

        for f in &files {
            prop_assert!(
                !f.is_absolute(),
                "list_files should return relative paths, got: {}",
                f.display()
            );
        }
    }

    /// Output is always sorted lexicographically by string representation.
    #[test]
    fn list_files_output_is_sorted(
        filenames in arb_filenames(2, 15),
    ) {
        let tmp = TempDir::new().unwrap();
        for name in &filenames {
            std::fs::write(tmp.path().join(name), "x").unwrap();
        }

        let files = list_files(tmp.path(), None).unwrap();
        let strs: Vec<String> = files.iter().map(|p| p.to_string_lossy().to_string()).collect();

        let mut sorted = strs.clone();
        sorted.sort();

        prop_assert_eq!(strs, sorted, "list_files output must be sorted");
    }

    /// max_files is always respected: output length ≤ max_files.
    #[test]
    fn list_files_respects_max_files(
        filenames in arb_filenames(1, 20),
        max_files in arb_max_files(),
    ) {
        let tmp = TempDir::new().unwrap();
        for name in &filenames {
            std::fs::write(tmp.path().join(name), "x").unwrap();
        }

        let files = list_files(tmp.path(), max_files).unwrap();

        if let Some(limit) = max_files {
            prop_assert!(
                files.len() <= limit,
                "list_files returned {} files, exceeding max_files={}",
                files.len(), limit,
            );
        }
    }

    /// max_files=0 always returns empty.
    #[test]
    fn list_files_zero_max_always_empty(
        filenames in arb_filenames(1, 5),
    ) {
        let tmp = TempDir::new().unwrap();
        for name in &filenames {
            std::fs::write(tmp.path().join(name), "x").unwrap();
        }

        let files = list_files(tmp.path(), Some(0)).unwrap();
        prop_assert!(files.is_empty(), "max_files=0 must always return empty");
    }

    /// No path contains the temp root as a prefix (paths are stripped).
    #[test]
    fn list_files_paths_are_stripped_of_root(
        filenames in arb_filenames(1, 5),
    ) {
        let tmp = TempDir::new().unwrap();
        let root_str = tmp.path().to_string_lossy().to_string();
        for name in &filenames {
            std::fs::write(tmp.path().join(name), "x").unwrap();
        }

        let files = list_files(tmp.path(), None).unwrap();

        for f in &files {
            let s = f.to_string_lossy().to_string();
            prop_assert!(
                !s.contains(&root_str),
                "path should not contain root prefix: {}",
                s
            );
        }
    }

    /// list_files only returns files (no directory entries).
    #[test]
    fn list_files_only_returns_files(
        dir_parts in arb_dir_depth(),
        filenames in arb_filenames(1, 5),
    ) {
        let tmp = TempDir::new().unwrap();

        // Create nested dirs
        if !dir_parts.is_empty() {
            let dir_path: PathBuf = dir_parts.iter().collect();
            std::fs::create_dir_all(tmp.path().join(&dir_path)).unwrap();
            // Put a file inside the deepest dir
            std::fs::write(tmp.path().join(&dir_path).join("nested.txt"), "n").unwrap();
        }

        // Also create root-level files
        for name in &filenames {
            std::fs::write(tmp.path().join(name), "x").unwrap();
        }

        let files = list_files(tmp.path(), None).unwrap();

        for f in &files {
            let full = tmp.path().join(f);
            prop_assert!(
                full.is_file(),
                "list_files should only return files: {}",
                f.display()
            );
        }
    }

    /// list_files is deterministic: calling twice yields identical results.
    #[test]
    fn list_files_is_deterministic(
        filenames in arb_filenames(1, 10),
    ) {
        let tmp = TempDir::new().unwrap();
        for name in &filenames {
            std::fs::write(tmp.path().join(name), "x").unwrap();
        }

        let files1 = list_files(tmp.path(), None).unwrap();
        let files2 = list_files(tmp.path(), None).unwrap();

        prop_assert_eq!(files1, files2, "list_files must be deterministic");
    }
}

// ============================================================================
// file_size invariants
// ============================================================================

proptest! {
    /// file_size matches the byte length of the content written.
    #[test]
    fn file_size_matches_content_length(
        content in arb_content(),
    ) {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("probe.bin"), &content).unwrap();

        let size = file_size(tmp.path(), std::path::Path::new("probe.bin")).unwrap();
        prop_assert_eq!(
            size as usize,
            content.len(),
            "file_size should match written content length"
        );
    }

    /// file_size for an empty file is always 0.
    #[test]
    fn file_size_empty_is_zero(_dummy in 0..10u8) {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("empty"), "").unwrap();

        let size = file_size(tmp.path(), std::path::Path::new("empty")).unwrap();
        prop_assert_eq!(size, 0, "empty file should have size 0");
    }

    /// file_size for nested files works correctly.
    #[test]
    fn file_size_nested_matches_content(
        dir_parts in prop::collection::vec("[a-z]{1,4}", 1..=3),
        content in arb_content(),
    ) {
        let tmp = TempDir::new().unwrap();
        let dir_path: PathBuf = dir_parts.iter().collect();
        std::fs::create_dir_all(tmp.path().join(&dir_path)).unwrap();

        let rel = dir_path.join("file.bin");
        std::fs::write(tmp.path().join(&rel), &content).unwrap();

        let size = file_size(tmp.path(), &rel).unwrap();
        prop_assert_eq!(
            size as usize,
            content.len(),
            "nested file_size should match content length"
        );
    }
}

// ============================================================================
// Gitignore + traversal interaction
// ============================================================================

proptest! {
    /// Files matching a .gitignore glob never appear in list_files output.
    #[test]
    fn gitignored_files_never_listed(
        good_names in prop::collection::vec("[a-z]{1,6}\\.rs", 1..=5),
        bad_names in prop::collection::vec("[a-z]{1,6}\\.log", 1..=5),
    ) {
        let tmp = TempDir::new().unwrap();
        // git init so .gitignore is honoured by the ignore crate
        std::process::Command::new("git")
            .arg("init")
            .current_dir(tmp.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "*.log\n").unwrap();

        for name in &good_names {
            std::fs::write(tmp.path().join(name), "good").unwrap();
        }
        for name in &bad_names {
            std::fs::write(tmp.path().join(name), "bad").unwrap();
        }

        let files = list_files(tmp.path(), None).unwrap();
        let names: Vec<String> = files.iter().map(|p| p.to_string_lossy().to_string()).collect();

        for bad in &bad_names {
            prop_assert!(
                !names.contains(bad),
                "gitignored file {} should not appear in output",
                bad
            );
        }
    }

    /// Non-ignored files are always present in list_files output.
    #[test]
    fn non_ignored_files_always_listed(
        good_names in prop::collection::hash_set("[a-z]{1,6}\\.rs", 1..=5),
    ) {
        let good_names: Vec<String> = good_names.into_iter().collect();
        let tmp = TempDir::new().unwrap();
        // git init so .gitignore is honoured by the ignore crate
        std::process::Command::new("git")
            .arg("init")
            .current_dir(tmp.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "*.log\n").unwrap();

        for name in &good_names {
            std::fs::write(tmp.path().join(name), "good").unwrap();
        }

        let files = list_files(tmp.path(), None).unwrap();
        let names: Vec<String> = files.iter().map(|p| p.to_string_lossy().to_string()).collect();

        for good in &good_names {
            prop_assert!(
                names.contains(good),
                "non-ignored file {} should appear in output",
                good
            );
        }
    }
}
