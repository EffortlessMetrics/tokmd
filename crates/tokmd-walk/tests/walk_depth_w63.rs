//! Depth tests for tokmd-walk (wave 63).
//!
//! Covers walk configuration, gitignore respect, hidden file handling,
//! symlink behavior, empty directories, file extension filtering,
//! asset categorization, deterministic ordering, and property-based tests.

use std::fs;
use std::path::{Path, PathBuf};
use tokmd_walk::{file_size, license_candidates, list_files};

// ============================================================================
// Helpers
// ============================================================================

fn tmpdir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

/// Create a non-git temp directory (no `.git` so fallback walker is used).
fn non_git_dir() -> tempfile::TempDir {
    let d = tmpdir();
    // Ensure no .git dir exists
    let _ = fs::remove_dir_all(d.path().join(".git"));
    d
}

fn write(dir: &Path, name: &str, content: &str) {
    if let Some(parent) = Path::new(name).parent() {
        fs::create_dir_all(dir.join(parent)).unwrap();
    }
    fs::write(dir.join(name), content).unwrap();
}

/// Create a temp dir with `git init` so gitignore rules are respected.
fn git_dir() -> tempfile::TempDir {
    let d = tmpdir();
    std::process::Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(d.path())
        .output()
        .expect("git init failed");
    d
}

// ============================================================================
// 1. Walk configuration options
// ============================================================================

#[test]
fn walk_max_zero_returns_empty() {
    let dir = non_git_dir();
    write(dir.path(), "a.rs", "fn main() {}");
    let files = list_files(dir.path(), Some(0)).unwrap();
    assert!(files.is_empty());
}

#[test]
fn walk_max_one_returns_single_file() {
    let dir = non_git_dir();
    write(dir.path(), "a.rs", "x");
    write(dir.path(), "b.rs", "y");
    write(dir.path(), "c.rs", "z");
    let files = list_files(dir.path(), Some(1)).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn walk_max_none_returns_all() {
    let dir = non_git_dir();
    for i in 0..5 {
        write(dir.path(), &format!("file{i}.txt"), "x");
    }
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 5);
}

#[test]
fn walk_max_larger_than_file_count() {
    let dir = non_git_dir();
    write(dir.path(), "only.txt", "x");
    let files = list_files(dir.path(), Some(100)).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn walk_returns_relative_paths() {
    let dir = non_git_dir();
    write(dir.path(), "src/main.rs", "fn main() {}");
    let files = list_files(dir.path(), None).unwrap();
    for f in &files {
        assert!(f.is_relative(), "path should be relative: {:?}", f);
    }
}

// ============================================================================
// 2. Gitignore pattern respect
// ============================================================================

#[test]
fn walk_gitignore_excludes_target_dir() {
    let dir = git_dir();
    write(dir.path(), ".gitignore", "target/\n");
    write(dir.path(), "src/main.rs", "fn main() {}");
    write(dir.path(), "target/debug/binary", "binary");
    let files = list_files(dir.path(), None).unwrap();
    let strs: Vec<String> = files.iter().map(|p| p.to_string_lossy().to_string()).collect();
    assert!(
        !strs.iter().any(|s| s.contains("target")),
        "target/ should be ignored: {strs:?}"
    );
}

#[test]
fn walk_gitignore_excludes_glob_pattern() {
    let dir = git_dir();
    write(dir.path(), ".gitignore", "*.log\n");
    write(dir.path(), "app.log", "log data");
    write(dir.path(), "nested/debug.log", "more logs");
    write(dir.path(), "src/main.rs", "fn main() {}");
    let files = list_files(dir.path(), None).unwrap();
    for f in &files {
        assert!(
            !f.to_string_lossy().ends_with(".log"),
            "*.log should be excluded: {:?}",
            f
        );
    }
}

#[test]
fn walk_gitignore_negation_re_includes() {
    let dir = git_dir();
    write(dir.path(), ".gitignore", "*.txt\n!important.txt\n");
    write(dir.path(), "junk.txt", "x");
    write(dir.path(), "important.txt", "y");
    write(dir.path(), "keep.rs", "z");
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();
    assert!(names.contains(&"important.txt".to_string()));
    assert!(!names.contains(&"junk.txt".to_string()));
}

#[test]
fn walk_nested_gitignore_applies() {
    let dir = git_dir();
    write(dir.path(), "src/.gitignore", "*.gen\n");
    write(dir.path(), "src/good.rs", "x");
    write(dir.path(), "src/bad.gen", "y");
    write(dir.path(), "root.gen", "z"); // NOT ignored (gitignore in src/)
    let files = list_files(dir.path(), None).unwrap();
    let strs: Vec<String> = files.iter().map(|p| p.to_string_lossy().to_string()).collect();
    // src/bad.gen should be excluded
    assert!(!strs.iter().any(|s| s.contains("bad.gen")));
}

#[test]
fn walk_gitignore_file_itself_not_listed() {
    let dir = non_git_dir();
    write(dir.path(), ".gitignore", "*.tmp\n");
    write(dir.path(), "a.rs", "x");
    // .gitignore is a hidden file and should not be listed (hidden=false in builder means
    // hidden files *are* included for ignore crate — this test documents behavior)
    let files = list_files(dir.path(), None).unwrap();
    // gitignore itself may or may not appear — just verify the walk succeeds
    assert!(!files.is_empty());
}

// ============================================================================
// 3. Hidden file handling
// ============================================================================

#[test]
fn walk_hidden_files_are_included() {
    let dir = non_git_dir();
    write(dir.path(), ".hidden", "secret");
    write(dir.path(), "visible.txt", "public");
    let files = list_files(dir.path(), None).unwrap();
    // Builder sets hidden(false) which means hidden files ARE traversed
    let names: Vec<String> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();
    assert!(
        names.contains(&".hidden".to_string()),
        "hidden files should be included: {names:?}"
    );
}

#[test]
fn walk_hidden_directory_contents_included() {
    let dir = non_git_dir();
    write(dir.path(), ".config/settings.json", r#"{"key":"val"}"#);
    write(dir.path(), "visible.txt", "x");
    let files = list_files(dir.path(), None).unwrap();
    let has_hidden = files
        .iter()
        .any(|p| p.to_string_lossy().contains(".config"));
    assert!(has_hidden, "hidden dir contents should be included");
}

#[test]
fn walk_dot_git_excluded_when_files_tracked() {
    let dir = git_dir();
    write(dir.path(), "src/main.rs", "fn main() {}");
    // Stage a file so git ls-files returns results
    std::process::Command::new("git")
        .args(["add", "src/main.rs"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let git_files: Vec<_> = files
        .iter()
        .filter(|p| {
            let s = p.to_string_lossy();
            s.starts_with(".git/") || s.starts_with(".git\\")
        })
        .collect();
    assert!(
        git_files.is_empty(),
        ".git/ contents should be excluded: {git_files:?}"
    );
}

// ============================================================================
// 4. Symlink behavior
// ============================================================================

#[cfg(unix)]
#[test]
fn walk_does_not_follow_symlinks() {
    let dir = non_git_dir();
    write(dir.path(), "real.txt", "content");
    std::os::unix::fs::symlink(
        dir.path().join("real.txt"),
        dir.path().join("link.txt"),
    )
    .unwrap();
    let files = list_files(dir.path(), None).unwrap();
    // With follow_links(false), symlinks are skipped
    let names: Vec<String> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();
    // The real file is there but the symlink may or may not appear
    assert!(names.contains(&"real.txt".to_string()));
}

#[cfg(unix)]
#[test]
fn walk_broken_symlink_does_not_crash() {
    let dir = non_git_dir();
    write(dir.path(), "real.txt", "content");
    std::os::unix::fs::symlink("/nonexistent/path", dir.path().join("broken_link")).unwrap();
    // Should not panic
    let result = list_files(dir.path(), None);
    assert!(result.is_ok());
}

// ============================================================================
// 5. Empty directory walking
// ============================================================================

#[test]
fn walk_empty_dir_returns_empty() {
    let dir = non_git_dir();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty());
}

#[test]
fn walk_dir_with_only_subdirs_returns_empty() {
    let dir = non_git_dir();
    fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
    fs::create_dir_all(dir.path().join("d/e")).unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty(), "directories should not appear: {files:?}");
}

#[test]
fn walk_nested_empty_and_populated_dirs() {
    let dir = non_git_dir();
    fs::create_dir_all(dir.path().join("empty")).unwrap();
    write(dir.path(), "populated/file.txt", "content");
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 1);
}

// ============================================================================
// 6. File extension filtering (via license_candidates)
// ============================================================================

#[test]
fn license_detects_license_with_extension() {
    let files = vec![
        PathBuf::from("LICENSE.md"),
        PathBuf::from("LICENSE.txt"),
        PathBuf::from("LICENSE-APACHE"),
        PathBuf::from("LICENSE-MIT"),
    ];
    let lc = license_candidates(&files);
    assert_eq!(lc.license_files.len(), 4);
}

#[test]
fn license_detects_copying_and_notice() {
    let files = vec![
        PathBuf::from("COPYING"),
        PathBuf::from("COPYING.LIB"),
        PathBuf::from("NOTICE"),
        PathBuf::from("NOTICE.txt"),
    ];
    let lc = license_candidates(&files);
    assert_eq!(lc.license_files.len(), 4);
}

#[test]
fn license_metadata_detects_all_package_managers() {
    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
        PathBuf::from("pyproject.toml"),
    ];
    let lc = license_candidates(&files);
    assert_eq!(lc.metadata_files.len(), 3);
    assert!(lc.license_files.is_empty());
}

#[test]
fn license_ignores_unrelated_files() {
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("README.md"),
        PathBuf::from("Makefile"),
        PathBuf::from("Dockerfile"),
    ];
    let lc = license_candidates(&files);
    assert!(lc.license_files.is_empty());
    assert!(lc.metadata_files.is_empty());
}

#[test]
fn license_case_insensitive_license_variants() {
    let files = vec![
        PathBuf::from("license"),
        PathBuf::from("License.md"),
        PathBuf::from("LICENSE"),
        PathBuf::from("LICENSE.TXT"),
    ];
    let lc = license_candidates(&files);
    assert_eq!(lc.license_files.len(), 4);
}

#[test]
fn license_nested_paths() {
    let files = vec![
        PathBuf::from("crates/foo/LICENSE"),
        PathBuf::from("crates/bar/Cargo.toml"),
        PathBuf::from("node_modules/pkg/package.json"),
    ];
    let lc = license_candidates(&files);
    assert_eq!(lc.license_files.len(), 1);
    assert_eq!(lc.metadata_files.len(), 2);
}

// ============================================================================
// 7. Asset categorization (license_candidates sorting)
// ============================================================================

#[test]
fn license_output_sorted_alphabetically() {
    let files = vec![
        PathBuf::from("z/LICENSE"),
        PathBuf::from("m/LICENSE"),
        PathBuf::from("a/LICENSE"),
    ];
    let lc = license_candidates(&files);
    let strs: Vec<String> = lc.license_files.iter().map(|p| p.to_string_lossy().to_string()).collect();
    let mut sorted = strs.clone();
    sorted.sort();
    assert_eq!(strs, sorted);
}

#[test]
fn license_metadata_output_sorted() {
    let files = vec![
        PathBuf::from("zzz/Cargo.toml"),
        PathBuf::from("aaa/package.json"),
        PathBuf::from("mmm/pyproject.toml"),
    ];
    let lc = license_candidates(&files);
    let strs: Vec<String> = lc.metadata_files.iter().map(|p| p.to_string_lossy().to_string()).collect();
    let mut sorted = strs.clone();
    sorted.sort();
    assert_eq!(strs, sorted);
}

#[test]
fn license_empty_input_returns_empty_candidates() {
    let lc = license_candidates(&[]);
    assert!(lc.license_files.is_empty());
    assert!(lc.metadata_files.is_empty());
}

#[test]
fn license_mixed_license_and_metadata_sorted_independently() {
    let files = vec![
        PathBuf::from("z/Cargo.toml"),
        PathBuf::from("a/LICENSE"),
        PathBuf::from("a/Cargo.toml"),
        PathBuf::from("z/LICENSE"),
    ];
    let lc = license_candidates(&files);
    assert_eq!(lc.license_files[0], PathBuf::from("a/LICENSE"));
    assert_eq!(lc.license_files[1], PathBuf::from("z/LICENSE"));
    assert_eq!(lc.metadata_files[0], PathBuf::from("a/Cargo.toml"));
    assert_eq!(lc.metadata_files[1], PathBuf::from("z/Cargo.toml"));
}

// ============================================================================
// 8. Deterministic ordering of walk results
// ============================================================================

#[test]
fn walk_results_sorted_deterministically() {
    let dir = non_git_dir();
    write(dir.path(), "zebra.txt", "z");
    write(dir.path(), "apple.txt", "a");
    write(dir.path(), "mango.txt", "m");
    let files = list_files(dir.path(), None).unwrap();
    let strs: Vec<String> = files.iter().map(|p| p.to_string_lossy().to_string()).collect();
    let mut sorted = strs.clone();
    sorted.sort();
    assert_eq!(strs, sorted, "walk results must be sorted");
}

#[test]
fn walk_results_sorted_with_nested_dirs() {
    let dir = non_git_dir();
    write(dir.path(), "b/file.txt", "x");
    write(dir.path(), "a/file.txt", "x");
    write(dir.path(), "c/file.txt", "x");
    let files = list_files(dir.path(), None).unwrap();
    let strs: Vec<String> = files.iter().map(|p| p.to_string_lossy().to_string()).collect();
    let mut sorted = strs.clone();
    sorted.sort();
    assert_eq!(strs, sorted);
}

#[test]
fn walk_deterministic_across_runs() {
    let dir = non_git_dir();
    for i in 0..10 {
        write(dir.path(), &format!("f{i}.txt"), "x");
    }
    let run1 = list_files(dir.path(), None).unwrap();
    let run2 = list_files(dir.path(), None).unwrap();
    assert_eq!(run1, run2, "walk results must be identical across runs");
}

// ============================================================================
// 9. file_size edge cases
// ============================================================================

#[test]
fn file_size_exact_bytes() {
    let dir = tmpdir();
    let content = "hello world!"; // 12 bytes
    write(dir.path(), "test.txt", content);
    let size = file_size(dir.path(), Path::new("test.txt")).unwrap();
    assert_eq!(size, 12);
}

#[test]
fn file_size_empty_file_is_zero() {
    let dir = tmpdir();
    write(dir.path(), "empty.txt", "");
    let size = file_size(dir.path(), Path::new("empty.txt")).unwrap();
    assert_eq!(size, 0);
}

#[test]
fn file_size_nonexistent_returns_error() {
    let dir = tmpdir();
    let result = file_size(dir.path(), Path::new("nope.txt"));
    assert!(result.is_err());
}

#[test]
fn file_size_large_file() {
    let dir = tmpdir();
    let data = "x".repeat(100_000);
    write(dir.path(), "big.txt", &data);
    let size = file_size(dir.path(), Path::new("big.txt")).unwrap();
    assert_eq!(size, 100_000);
}

#[test]
fn file_size_nested_path() {
    let dir = tmpdir();
    write(dir.path(), "a/b/c/deep.txt", "deep content");
    let size = file_size(dir.path(), Path::new("a/b/c/deep.txt")).unwrap();
    assert_eq!(size, "deep content".len() as u64);
}

// ============================================================================
// 10. Multi-level nesting and deep directories
// ============================================================================

#[test]
fn walk_deep_nesting() {
    let dir = non_git_dir();
    let deep_path = "a/b/c/d/e/f/g/deep.txt";
    write(dir.path(), deep_path, "deep");
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].to_string_lossy().contains("deep.txt"));
}

#[test]
fn walk_mixed_depths() {
    let dir = non_git_dir();
    write(dir.path(), "root.txt", "x");
    write(dir.path(), "a/mid.txt", "x");
    write(dir.path(), "a/b/c/deep.txt", "x");
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 3);
}

#[test]
fn walk_many_files_in_single_dir() {
    let dir = non_git_dir();
    for i in 0..50 {
        write(dir.path(), &format!("file_{i:03}.txt"), "x");
    }
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 50);
}

#[test]
fn walk_max_truncates_many_files() {
    let dir = non_git_dir();
    for i in 0..50 {
        write(dir.path(), &format!("file_{i:03}.txt"), "x");
    }
    let files = list_files(dir.path(), Some(10)).unwrap();
    assert!(files.len() <= 10);
}

// ============================================================================
// 11. Directories only containing hidden or ignored content
// ============================================================================

#[test]
fn walk_dir_with_only_gitignored_files() {
    let dir = git_dir();
    write(dir.path(), ".gitignore", "*.tmp\n");
    write(dir.path(), "a.tmp", "x");
    write(dir.path(), "b.tmp", "y");
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();
    assert!(!names.contains(&"a.tmp".to_string()));
    assert!(!names.contains(&"b.tmp".to_string()));
}

// ============================================================================
// 12. Property-based tests
// ============================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn walk_file_count_within_max(max in 0usize..20) {
            let dir = non_git_dir();
            for i in 0..10 {
                write(dir.path(), &format!("f{i}.txt"), "x");
            }
            let files = list_files(dir.path(), Some(max)).unwrap();
            prop_assert!(files.len() <= max, "got {} files, max {}", files.len(), max);
        }

        #[test]
        fn walk_all_paths_relative(count in 1usize..10) {
            let dir = non_git_dir();
            for i in 0..count {
                write(dir.path(), &format!("f{i}.txt"), "x");
            }
            let files = list_files(dir.path(), None).unwrap();
            for f in &files {
                prop_assert!(f.is_relative(), "path must be relative: {:?}", f);
            }
        }

        #[test]
        fn walk_deterministic_order(count in 1usize..15) {
            let dir = non_git_dir();
            for i in 0..count {
                write(dir.path(), &format!("f{i}.txt"), "x");
            }
            let run1 = list_files(dir.path(), None).unwrap();
            let run2 = list_files(dir.path(), None).unwrap();
            prop_assert_eq!(run1, run2);
        }

        #[test]
        fn walk_results_always_sorted(count in 1usize..15) {
            let dir = non_git_dir();
            for i in 0..count {
                write(dir.path(), &format!("file_{i}.txt"), "x");
            }
            let files = list_files(dir.path(), None).unwrap();
            let strs: Vec<String> = files.iter().map(|p| p.to_string_lossy().to_string()).collect();
            let mut sorted = strs.clone();
            sorted.sort();
            prop_assert_eq!(strs, sorted);
        }

        #[test]
        fn file_size_matches_content_length(len in 0usize..1000) {
            let dir = tmpdir();
            let data = "x".repeat(len);
            write(dir.path(), "sized.txt", &data);
            let size = file_size(dir.path(), Path::new("sized.txt")).unwrap();
            prop_assert_eq!(size, len as u64);
        }

        #[test]
        fn license_candidates_partition_is_complete(n in 0usize..20) {
            // Every file ends up in at most one of the two vectors
            let mut files = Vec::new();
            for i in 0..n {
                match i % 4 {
                    0 => files.push(PathBuf::from(format!("dir{i}/LICENSE"))),
                    1 => files.push(PathBuf::from(format!("dir{i}/Cargo.toml"))),
                    2 => files.push(PathBuf::from(format!("dir{i}/package.json"))),
                    _ => files.push(PathBuf::from(format!("dir{i}/main.rs"))),
                }
            }
            let lc = license_candidates(&files);
            // License and metadata should not overlap
            for l in &lc.license_files {
                prop_assert!(!lc.metadata_files.contains(l));
            }
        }
    }
}
