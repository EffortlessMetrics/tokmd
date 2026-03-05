//! Walk module boundary tests.
//!
//! Verifies that tokmd-walk functions handle empty directories, non-existent
//! paths, ignored files, and asset detection edge cases gracefully.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokmd_walk::{file_size, license_candidates, list_files};

// ── list_files: empty directory ──────────────────────────────────────

#[test]
fn list_files_empty_dir_returns_empty_vec() {
    let dir = TempDir::new().unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty(), "empty directory should yield no files");
}

#[test]
fn list_files_empty_dir_with_max_files() {
    let dir = TempDir::new().unwrap();
    let files = list_files(dir.path(), Some(10)).unwrap();
    assert!(files.is_empty());
}

#[test]
fn list_files_max_files_zero_returns_empty() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "hello").unwrap();
    let files = list_files(dir.path(), Some(0)).unwrap();
    assert!(files.is_empty(), "max_files=0 should return empty vec");
}

// ── list_files: non-existent paths ───────────────────────────────────

#[test]
fn list_files_nonexistent_path_returns_error_or_empty() {
    let bogus = PathBuf::from("__nonexistent_w53_dir__");
    // May error or return empty depending on implementation
    match list_files(&bogus, None) {
        Ok(files) => assert!(files.is_empty()),
        Err(_) => {} // error is acceptable
    }
}

// ── list_files: directories with only ignored files ──────────────────

#[test]
fn list_files_respects_gitignore() {
    let dir = TempDir::new().unwrap();
    // Create .gitignore that ignores *.log
    fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    fs::write(dir.path().join("debug.log"), "log content").unwrap();
    fs::write(dir.path().join("app.rs"), "fn main() {}").unwrap();
    // Create .git marker for ignore crate to work
    fs::create_dir(dir.path().join(".git")).unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let has_log = files.iter().any(|p| p.to_string_lossy().contains(".log"));
    assert!(!has_log, "ignored .log files should not appear");
    let has_rs = files.iter().any(|p| p.to_string_lossy().contains("app.rs"));
    assert!(has_rs, "non-ignored .rs files should appear");
}

#[test]
fn list_files_results_are_sorted() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("c.txt"), "c").unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    fs::write(dir.path().join("b.txt"), "b").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let sorted: Vec<_> = {
        let mut v = files.clone();
        v.sort();
        v
    };
    assert_eq!(files, sorted, "list_files should return sorted results");
}

#[test]
fn list_files_respects_max_files_limit() {
    let dir = TempDir::new().unwrap();
    for i in 0..10 {
        fs::write(dir.path().join(format!("f{i}.txt")), format!("content {i}")).unwrap();
    }
    let files = list_files(dir.path(), Some(3)).unwrap();
    assert!(files.len() <= 3, "should respect max_files limit, got {}", files.len());
}

// ── license_candidates ───────────────────────────────────────────────

#[test]
fn license_candidates_detects_license_files() {
    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("src/main.rs"),
    ];
    let cands = license_candidates(&files);
    assert_eq!(cands.license_files.len(), 2);
}

#[test]
fn license_candidates_detects_copying_and_notice() {
    let files = vec![
        PathBuf::from("COPYING"),
        PathBuf::from("NOTICE"),
        PathBuf::from("README.md"),
    ];
    let cands = license_candidates(&files);
    assert!(cands.license_files.len() >= 2, "should detect COPYING and NOTICE");
}

#[test]
fn license_candidates_detects_metadata_files() {
    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
        PathBuf::from("pyproject.toml"),
        PathBuf::from("src/lib.rs"),
    ];
    let cands = license_candidates(&files);
    assert_eq!(cands.metadata_files.len(), 3);
}

#[test]
fn license_candidates_empty_input() {
    let cands = license_candidates(&[]);
    assert!(cands.license_files.is_empty());
    assert!(cands.metadata_files.is_empty());
}

#[test]
fn license_candidates_results_are_sorted() {
    let files = vec![
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("LICENSE-APACHE"),
        PathBuf::from("COPYING"),
    ];
    let cands = license_candidates(&files);
    let sorted = {
        let mut v = cands.license_files.clone();
        v.sort();
        v
    };
    assert_eq!(cands.license_files, sorted, "license files should be sorted");
}

// ── file_size ────────────────────────────────────────────────────────

#[test]
fn file_size_returns_correct_bytes() {
    let dir = TempDir::new().unwrap();
    let content = "hello world";
    fs::write(dir.path().join("test.txt"), content).unwrap();
    let size = file_size(dir.path(), &PathBuf::from("test.txt")).unwrap();
    assert_eq!(size, content.len() as u64);
}

#[test]
fn file_size_empty_file_returns_zero() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("empty.txt"), "").unwrap();
    let size = file_size(dir.path(), &PathBuf::from("empty.txt")).unwrap();
    assert_eq!(size, 0);
}

#[test]
fn file_size_nonexistent_file_returns_error() {
    let dir = TempDir::new().unwrap();
    let result = file_size(dir.path(), &PathBuf::from("missing.txt"));
    assert!(result.is_err(), "non-existent file should return error");
}
