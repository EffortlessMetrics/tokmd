//! Depth tests for `tokmd-scan::walk` – W58.
//!
//! Exercises list_files, license_candidates, and file_size against synthetic
//! directory trees with various structures, gitignore integration, hidden
//! files, deeply nested paths, and classification patterns.

use std::fs;
use std::path::{Path, PathBuf};

use tokmd_scan::walk::{file_size, license_candidates, list_files};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_file(base: &Path, rel: &str, content: &str) {
    let path = base.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(&path, content).expect("write test file");
}

// ===========================================================================
// 1. list_files with various directory structures
// ===========================================================================

#[test]
fn list_files_returns_all_files_flat() {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), "a.rs", "fn a() {}");
    write_file(dir.path(), "b.py", "x = 1");
    write_file(dir.path(), "c.js", "var c;");
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 3, "expected ≥3, got {}", files.len());
}

#[test]
fn list_files_nested_structure() {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), "src/main.rs", "fn main() {}");
    write_file(dir.path(), "src/lib.rs", "pub mod foo;");
    write_file(dir.path(), "tests/test.rs", "#[test] fn t() {}");
    write_file(dir.path(), "README.md", "# Hello");
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 4, "expected ≥4, got {}", files.len());
}

#[test]
fn list_files_empty_dir_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty());
}

#[test]
fn list_files_max_zero() {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), "a.rs", "x");
    let files = list_files(dir.path(), Some(0)).unwrap();
    assert!(files.is_empty());
}

#[test]
fn list_files_max_limits_count() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..20 {
        write_file(dir.path(), &format!("f{i}.txt"), "data");
    }
    let files = list_files(dir.path(), Some(5)).unwrap();
    assert!(files.len() <= 5, "expected ≤5, got {}", files.len());
}

// ===========================================================================
// 2. Gitignore integration
// ===========================================================================

#[test]
fn list_files_respects_gitignore() {
    let dir = tempfile::tempdir().unwrap();
    // Initialize a git repo so gitignore is respected
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    write_file(dir.path(), ".gitignore", "ignored/\n");
    write_file(dir.path(), "kept.rs", "fn kept() {}");
    write_file(dir.path(), "ignored/secret.rs", "fn secret() {}");
    let files = list_files(dir.path(), None).unwrap();
    let has_ignored = files.iter().any(|f| f.to_string_lossy().contains("secret"));
    assert!(!has_ignored, "ignored files should be excluded: {files:?}");
}

#[test]
fn list_files_respects_gitignore_glob_pattern() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    write_file(dir.path(), ".gitignore", "*.log\n");
    write_file(dir.path(), "app.rs", "fn main() {}");
    write_file(dir.path(), "debug.log", "log data");
    let files = list_files(dir.path(), None).unwrap();
    let has_log = files.iter().any(|f| f.to_string_lossy().ends_with(".log"));
    assert!(!has_log, ".log files should be gitignored: {files:?}");
}

// ===========================================================================
// 3. license_candidates detection
// ===========================================================================

#[test]
fn license_candidates_detects_all_license_variants() {
    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("LICENSE.md"),
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("LICENSE-APACHE"),
        PathBuf::from("COPYING"),
        PathBuf::from("NOTICE"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 6);
}

#[test]
fn license_candidates_detects_metadata_files() {
    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
        PathBuf::from("pyproject.toml"),
    ];
    let result = license_candidates(&files);
    assert!(result.license_files.is_empty());
    assert_eq!(result.metadata_files.len(), 3);
}

#[test]
fn license_candidates_ignores_unrelated() {
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("README.md"),
        PathBuf::from("test.py"),
    ];
    let result = license_candidates(&files);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

#[test]
fn license_candidates_case_insensitive_matching() {
    let files = vec![
        PathBuf::from("license"),
        PathBuf::from("License.txt"),
        PathBuf::from("LICENSE.MD"),
        PathBuf::from("copying"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 4);
}

#[test]
fn license_candidates_returns_sorted() {
    let files = vec![
        PathBuf::from("z/LICENSE"),
        PathBuf::from("a/LICENSE"),
        PathBuf::from("m/LICENSE"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files[0], PathBuf::from("a/LICENSE"));
    assert_eq!(result.license_files[1], PathBuf::from("m/LICENSE"));
    assert_eq!(result.license_files[2], PathBuf::from("z/LICENSE"));
}

#[test]
fn license_candidates_empty_input() {
    let result = license_candidates(&[]);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

// ===========================================================================
// 4. Hidden files/directories
// ===========================================================================

#[test]
fn list_files_includes_hidden_files() {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), ".hidden_file", "secret");
    write_file(dir.path(), "visible.rs", "fn f() {}");
    let files = list_files(dir.path(), None).unwrap();
    let has_hidden = files
        .iter()
        .any(|f| f.to_string_lossy().contains(".hidden_file"));
    // list_files sets hidden(false) in WalkBuilder, but the `ignore` crate
    // default is to skip hidden. We verify the function runs without error.
    assert!(!files.is_empty());
    // Hidden files may or may not appear depending on walker settings.
    // The key invariant: the visible file is always found.
    let has_visible = files
        .iter()
        .any(|f| f.to_string_lossy().contains("visible"));
    assert!(has_visible, "visible file must be found: {files:?}");
    // With hidden(false), hidden files ARE shown by the `ignore` crate.
    assert!(has_hidden, ".hidden_file should be included: {files:?}");
}

#[test]
fn list_files_hidden_directory_contents() {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), ".config/settings.toml", "[settings]");
    write_file(dir.path(), "main.rs", "fn main() {}");
    let files = list_files(dir.path(), None).unwrap();
    let has_main = files
        .iter()
        .any(|f| f.to_string_lossy().contains("main.rs"));
    assert!(has_main);
}

// ===========================================================================
// 5. Deeply nested directories
// ===========================================================================

#[test]
fn list_files_deeply_nested() {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), "a/b/c/d/e/f/deep.txt", "deep content");
    let files = list_files(dir.path(), None).unwrap();
    let has_deep = files
        .iter()
        .any(|f| f.to_string_lossy().contains("deep.txt"));
    assert!(has_deep, "deeply nested file should be found: {files:?}");
}

#[test]
fn list_files_many_nested_siblings() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..5 {
        for j in 0..3 {
            write_file(dir.path(), &format!("dir{i}/file{j}.txt"), "x");
        }
    }
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 15, "expected ≥15 files, got {}", files.len());
}

// ===========================================================================
// 6. file_size tests
// ===========================================================================

#[test]
fn file_size_known_content() {
    let dir = tempfile::tempdir().unwrap();
    let content = "hello, world!";
    write_file(dir.path(), "test.txt", content);
    let size = file_size(dir.path(), Path::new("test.txt")).unwrap();
    assert_eq!(size, content.len() as u64);
}

#[test]
fn file_size_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), "empty.txt", "");
    let size = file_size(dir.path(), Path::new("empty.txt")).unwrap();
    assert_eq!(size, 0);
}

#[test]
fn file_size_missing_file_errors() {
    let dir = tempfile::tempdir().unwrap();
    let result = file_size(dir.path(), Path::new("nope.txt"));
    assert!(result.is_err());
}

#[test]
fn file_size_nested_path() {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), "sub/dir/file.txt", "1234567890");
    let size = file_size(dir.path(), Path::new("sub/dir/file.txt")).unwrap();
    assert_eq!(size, 10);
}
