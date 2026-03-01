//! Edge-case tests for tokmd-walk file system traversal.
//!
//! Covers scenarios not addressed by existing test files:
//! - Directories with only hidden files
//! - Symbolic links (platform-conditional)
//! - Very deep directory structures (10+ levels)
//! - Directories with mixed file types
//! - Empty nested directories
//! - Max files limit boundary behavior
//! - File size calculation accuracy

use std::path::PathBuf;

use tempfile::TempDir;
use tokmd_walk::{file_size, list_files};

// ============================================================================
// Helpers
// ============================================================================

/// Create a non-git temp directory so the WalkBuilder fallback path is
/// exercised (git ls-files returns None).
fn non_git_tempdir() -> TempDir {
    TempDir::new().expect("failed to create tempdir")
}

/// Sorted file name strings from `list_files` output.
fn file_names(files: &[PathBuf]) -> Vec<String> {
    files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

// ============================================================================
// Scenario: Directories with only hidden files
// ============================================================================

#[test]
fn directory_with_only_hidden_files_returns_them() {
    // Given: a directory containing only dot-prefixed files
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join(".env"), "SECRET=abc").unwrap();
    std::fs::write(tmp.path().join(".gitignore"), "*.log\n").unwrap();
    std::fs::write(tmp.path().join(".config"), "key=val").unwrap();

    // When: we list files (hidden=false means "don't skip hidden")
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then: all hidden files are included
    assert_eq!(files.len(), 3, "all 3 hidden files should be listed");
    assert!(names.iter().any(|n| n.contains(".env")));
    assert!(names.iter().any(|n| n.contains(".gitignore")));
    assert!(names.iter().any(|n| n.contains(".config")));
}

#[test]
fn hidden_subdirectory_with_only_hidden_files() {
    // Given: a hidden directory containing only hidden files
    let tmp = non_git_tempdir();
    std::fs::create_dir_all(tmp.path().join(".secrets")).unwrap();
    std::fs::write(tmp.path().join(".secrets/.key"), "private").unwrap();
    std::fs::write(tmp.path().join(".secrets/.cert"), "cert-data").unwrap();

    // When: we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then: hidden files in hidden directories are included
    assert_eq!(files.len(), 2);
    assert!(names.iter().any(|n| n.contains(".key")));
    assert!(names.iter().any(|n| n.contains(".cert")));
}

// ============================================================================
// Scenario: Symbolic links (Windows-conditional)
// ============================================================================

#[cfg(windows)]
#[test]
fn file_symlink_excluded_on_windows() {
    use std::os::windows::fs::symlink_file;

    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("real.txt"), "real content").unwrap();

    // Symlink creation may fail without elevated privileges; skip if so
    if symlink_file(tmp.path().join("real.txt"), tmp.path().join("link.txt")).is_err() {
        eprintln!("Skipping file symlink test: insufficient privileges on Windows");
        return;
    }

    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Real file should be present
    assert!(names.iter().any(|n| n.contains("real.txt")));
    // With follow_links=false, symlinks are not regular files
    assert!(
        !names.iter().any(|n| n.contains("link.txt")),
        "file symlinks should be excluded with follow_links=false"
    );
}

#[cfg(unix)]
#[test]
fn broken_symlink_does_not_crash() {
    use std::os::unix::fs::symlink;

    // Given: a symlink pointing to a non-existent target
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("real.txt"), "content").unwrap();
    symlink("/nonexistent/path", tmp.path().join("broken_link")).unwrap();

    // When: we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then: real file is returned, broken symlink is silently skipped
    assert!(names.iter().any(|n| n.contains("real.txt")));
    assert!(
        !names.iter().any(|n| n.contains("broken_link")),
        "broken symlinks should not appear in results"
    );
}

// ============================================================================
// Scenario: Very deep directory structures (10+ levels)
// ============================================================================

#[test]
fn deep_directory_structure_10_levels() {
    // Given: a directory nested 10 levels deep with a file at the bottom
    let tmp = non_git_tempdir();
    let deep_path = tmp.path().join("a/b/c/d/e/f/g/h/i/j");
    std::fs::create_dir_all(&deep_path).unwrap();
    std::fs::write(deep_path.join("deep.txt"), "found me").unwrap();
    // Also a file at the root
    std::fs::write(tmp.path().join("root.txt"), "root").unwrap();

    // When: we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then: both root and deeply nested files are found
    assert_eq!(files.len(), 2);
    assert!(names.iter().any(|n| n.contains("root.txt")));
    assert!(
        names.iter().any(|n| n.contains("deep.txt")),
        "file at 10 levels deep should be found"
    );
}

#[test]
fn deep_directory_structure_15_levels() {
    // Given: a directory nested 15 levels deep
    let tmp = non_git_tempdir();
    let deep_path = tmp
        .path()
        .join("l1/l2/l3/l4/l5/l6/l7/l8/l9/l10/l11/l12/l13/l14/l15");
    std::fs::create_dir_all(&deep_path).unwrap();
    std::fs::write(deep_path.join("very_deep.rs"), "fn main() {}").unwrap();

    // When: we list files
    let files = list_files(tmp.path(), None).unwrap();

    // Then: the deeply nested file is found
    assert_eq!(files.len(), 1);
    let path_str = files[0].to_string_lossy();
    assert!(
        path_str.contains("very_deep.rs"),
        "file at 15 levels deep should be found"
    );
}

#[test]
fn deep_directories_with_files_at_each_level() {
    // Given: files at every level of a 10-deep structure
    let tmp = non_git_tempdir();
    let mut current = tmp.path().to_path_buf();

    for i in 0..10 {
        std::fs::write(current.join(format!("level_{}.txt", i)), "data").unwrap();
        current = current.join(format!("dir_{}", i));
        std::fs::create_dir_all(&current).unwrap();
    }
    // One more file at the deepest level
    std::fs::write(current.join("level_10.txt"), "data").unwrap();

    // When: we list files
    let files = list_files(tmp.path(), None).unwrap();

    // Then: all 11 files are found
    assert_eq!(files.len(), 11, "should find files at all 11 levels");
}

// ============================================================================
// Scenario: Directories with mixed file types
// ============================================================================

#[test]
fn mixed_file_types_in_same_directory() {
    // Given: a directory with various file extensions
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("code.rs"), "fn main() {}").unwrap();
    std::fs::write(tmp.path().join("config.toml"), "[package]").unwrap();
    std::fs::write(tmp.path().join("readme.md"), "# Hello").unwrap();
    std::fs::write(tmp.path().join("data.json"), "{}").unwrap();
    std::fs::write(tmp.path().join("binary.bin"), vec![0u8; 64]).unwrap();
    std::fs::write(tmp.path().join("script.py"), "print('hi')").unwrap();

    // When: we list files
    let files = list_files(tmp.path(), None).unwrap();

    // Then: all file types are included
    assert_eq!(files.len(), 6, "all 6 files should be listed");

    let names = file_names(&files);
    assert!(names.iter().any(|n| n.ends_with(".rs")));
    assert!(names.iter().any(|n| n.ends_with(".toml")));
    assert!(names.iter().any(|n| n.ends_with(".md")));
    assert!(names.iter().any(|n| n.ends_with(".json")));
    assert!(names.iter().any(|n| n.ends_with(".bin")));
    assert!(names.iter().any(|n| n.ends_with(".py")));
}

#[test]
fn files_with_no_extension() {
    // Given: files without extensions
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("Makefile"), "all:\n\techo hi").unwrap();
    std::fs::write(tmp.path().join("Dockerfile"), "FROM ubuntu").unwrap();
    std::fs::write(tmp.path().join("Justfile"), "build:\n\tcargo build").unwrap();

    // When: we list files
    let files = list_files(tmp.path(), None).unwrap();

    // Then: extension-less files are included
    assert_eq!(files.len(), 3);
}

#[test]
fn mix_of_hidden_and_visible_files() {
    // Given: a mix of hidden and visible files at various depths
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("visible.txt"), "v").unwrap();
    std::fs::write(tmp.path().join(".hidden"), "h").unwrap();
    std::fs::create_dir_all(tmp.path().join("sub")).unwrap();
    std::fs::write(tmp.path().join("sub/visible2.txt"), "v2").unwrap();
    std::fs::write(tmp.path().join("sub/.hidden2"), "h2").unwrap();
    std::fs::create_dir_all(tmp.path().join(".hdir")).unwrap();
    std::fs::write(tmp.path().join(".hdir/in_hidden.txt"), "ih").unwrap();

    // When: we list files
    let files = list_files(tmp.path(), None).unwrap();

    // Then: all 5 files are included (hidden=false means include hidden)
    assert_eq!(files.len(), 5, "all hidden and visible files should appear");
}

// ============================================================================
// Scenario: Empty nested directories
// ============================================================================

#[test]
fn deeply_nested_empty_dirs_produce_no_results() {
    // Given: several levels of empty nested directories
    let tmp = non_git_tempdir();
    std::fs::create_dir_all(tmp.path().join("a/b/c/d/e")).unwrap();
    std::fs::create_dir_all(tmp.path().join("x/y/z")).unwrap();
    std::fs::create_dir_all(tmp.path().join("empty")).unwrap();

    // When: we list files
    let files = list_files(tmp.path(), None).unwrap();

    // Then: no files are returned
    assert!(
        files.is_empty(),
        "empty nested dirs should produce no results"
    );
}

#[test]
fn single_file_among_many_empty_dirs() {
    // Given: many empty directories but one file
    let tmp = non_git_tempdir();
    std::fs::create_dir_all(tmp.path().join("empty1")).unwrap();
    std::fs::create_dir_all(tmp.path().join("empty2/sub")).unwrap();
    std::fs::create_dir_all(tmp.path().join("empty3/a/b")).unwrap();
    std::fs::create_dir_all(tmp.path().join("has_file")).unwrap();
    std::fs::write(tmp.path().join("has_file/only.txt"), "alone").unwrap();

    // When: we list files
    let files = list_files(tmp.path(), None).unwrap();

    // Then: only the one file is returned
    assert_eq!(files.len(), 1);
    assert!(files[0].to_string_lossy().contains("only.txt"));
}

// ============================================================================
// Scenario: Max files limit behavior
// ============================================================================

#[test]
fn max_files_exactly_matches_file_count() {
    // Given: exactly 5 files
    let tmp = non_git_tempdir();
    for i in 0..5 {
        std::fs::write(tmp.path().join(format!("file_{}.txt", i)), "data").unwrap();
    }

    // When: max_files = 5 (exactly the count)
    let files = list_files(tmp.path(), Some(5)).unwrap();

    // Then: all 5 are returned
    assert_eq!(files.len(), 5);
}

#[test]
fn max_files_one_less_than_total() {
    // Given: 5 files
    let tmp = non_git_tempdir();
    for i in 0..5 {
        std::fs::write(tmp.path().join(format!("file_{}.txt", i)), "data").unwrap();
    }

    // When: max_files = 4
    let files = list_files(tmp.path(), Some(4)).unwrap();

    // Then: exactly 4 are returned
    assert_eq!(files.len(), 4);
}

#[test]
fn max_files_exceeds_total() {
    // Given: 3 files
    let tmp = non_git_tempdir();
    for i in 0..3 {
        std::fs::write(tmp.path().join(format!("f_{}.txt", i)), "data").unwrap();
    }

    // When: max_files = 100 (exceeds actual count)
    let files = list_files(tmp.path(), Some(100)).unwrap();

    // Then: all 3 are returned (no padding)
    assert_eq!(files.len(), 3);
}

#[test]
fn max_files_one_from_many() {
    // Given: 20 files
    let tmp = non_git_tempdir();
    for i in 0..20 {
        std::fs::write(tmp.path().join(format!("file_{:02}.txt", i)), "data").unwrap();
    }

    // When: max_files = 1
    let files = list_files(tmp.path(), Some(1)).unwrap();

    // Then: exactly 1 file is returned
    assert_eq!(files.len(), 1);
}

#[test]
fn max_files_zero_always_empty() {
    // Given: files exist
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("a.txt"), "a").unwrap();

    // When: max_files = 0
    let files = list_files(tmp.path(), Some(0)).unwrap();

    // Then: empty
    assert!(files.is_empty());
}

#[test]
fn max_files_with_nested_structure() {
    // Given: files distributed across nested directories
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("root.txt"), "r").unwrap();
    std::fs::create_dir_all(tmp.path().join("a/b")).unwrap();
    std::fs::write(tmp.path().join("a/mid.txt"), "m").unwrap();
    std::fs::write(tmp.path().join("a/b/deep.txt"), "d").unwrap();

    // When: max_files = 2
    let files = list_files(tmp.path(), Some(2)).unwrap();

    // Then: at most 2 files
    assert_eq!(files.len(), 2, "max_files should cap at 2");
}

// ============================================================================
// Scenario: File size calculation accuracy
// ============================================================================

#[test]
fn file_size_exact_known_content() {
    let tmp = non_git_tempdir();
    let content = "Hello, World!"; // 13 bytes
    std::fs::write(tmp.path().join("greeting.txt"), content).unwrap();

    let size = file_size(tmp.path(), std::path::Path::new("greeting.txt")).unwrap();
    assert_eq!(size, 13);
}

#[test]
fn file_size_binary_content() {
    let tmp = non_git_tempdir();
    let content: Vec<u8> = (0..=255).collect(); // 256 bytes
    std::fs::write(tmp.path().join("binary.dat"), &content).unwrap();

    let size = file_size(tmp.path(), std::path::Path::new("binary.dat")).unwrap();
    assert_eq!(size, 256);
}

#[test]
fn file_size_large_file() {
    let tmp = non_git_tempdir();
    let content = vec![b'x'; 100_000]; // 100KB
    std::fs::write(tmp.path().join("large.bin"), &content).unwrap();

    let size = file_size(tmp.path(), std::path::Path::new("large.bin")).unwrap();
    assert_eq!(size, 100_000);
}

#[test]
fn file_size_unicode_content() {
    let tmp = non_git_tempdir();
    // Multi-byte UTF-8: each emoji is 4 bytes
    let content = "🦀🦀🦀"; // 3 × 4 = 12 bytes
    std::fs::write(tmp.path().join("emoji.txt"), content).unwrap();

    let size = file_size(tmp.path(), std::path::Path::new("emoji.txt")).unwrap();
    assert_eq!(size, 12, "3 four-byte emojis = 12 bytes");
}

#[test]
fn file_size_deeply_nested() {
    let tmp = non_git_tempdir();
    let deep = tmp.path().join("a/b/c/d/e/f/g/h/i/j");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("deep.txt"), "1234567890").unwrap();

    let size = file_size(
        tmp.path(),
        std::path::Path::new("a/b/c/d/e/f/g/h/i/j/deep.txt"),
    )
    .unwrap();
    assert_eq!(size, 10);
}

#[test]
fn file_size_error_on_directory() {
    let tmp = non_git_tempdir();
    std::fs::create_dir_all(tmp.path().join("subdir")).unwrap();

    // file_size on a directory should still return metadata (dir size varies by OS)
    // but the important thing is it doesn't panic
    let result = file_size(tmp.path(), std::path::Path::new("subdir"));
    // On most OSes, metadata() on a directory succeeds but returns 0 or some value
    assert!(result.is_ok(), "file_size on directory should not error");
}

#[test]
fn file_size_nonexistent_returns_error() {
    let tmp = non_git_tempdir();

    let result = file_size(tmp.path(), std::path::Path::new("ghost.txt"));
    assert!(result.is_err(), "nonexistent file should return error");
}
