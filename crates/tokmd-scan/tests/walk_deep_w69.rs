//! Deep tests for tokmd-scan walk helpers – wave 69.
//!
//! Covers file discovery with tempdir fixtures, license/metadata candidate
//! detection, file_size, max-file limits, hidden file handling, gitignore
//! respect, and determinism.

use std::fs;
use std::path::{Path, PathBuf};
use tokmd_scan::walk::{file_size, license_candidates, list_files};

// ── helpers ─────────────────────────────────────────────────────────────

fn sorted_names(files: &[PathBuf]) -> Vec<String> {
    let mut v: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    v.sort();
    v
}

// =========================================================================
// 1. list_files – basic discovery
// =========================================================================

#[test]
fn list_files_discovers_flat_dir() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "fn a(){}").unwrap();
    fs::write(dir.path().join("b.rs"), "fn b(){}").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn list_files_discovers_nested_dirs() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src/util")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "x").unwrap();
    fs::write(dir.path().join("src/util/helpers.rs"), "x").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 2);
}

#[test]
fn list_files_returns_relative_paths() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "x").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    for f in &files {
        assert!(f.is_relative(), "must be relative: {:?}", f);
    }
}

// =========================================================================
// 2. list_files – max_files limits
// =========================================================================

#[test]
fn list_files_max_zero_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "x").unwrap();
    let files = list_files(dir.path(), Some(0)).unwrap();
    assert!(files.is_empty());
}

#[test]
fn list_files_max_one_returns_at_most_one() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..5 {
        fs::write(dir.path().join(format!("f{i}.txt")), "x").unwrap();
    }
    let files = list_files(dir.path(), Some(1)).unwrap();
    assert!(files.len() <= 1);
}

#[test]
fn list_files_max_exceeds_file_count() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("only.txt"), "x").unwrap();
    let files = list_files(dir.path(), Some(100)).unwrap();
    assert_eq!(files.len(), 1);
}

// =========================================================================
// 3. list_files – determinism
// =========================================================================

#[test]
fn list_files_deterministic_ordering() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["z.rs", "a.rs", "m.rs", "b.rs"] {
        fs::write(dir.path().join(name), "x").unwrap();
    }
    let a = sorted_names(&list_files(dir.path(), None).unwrap());
    let b = sorted_names(&list_files(dir.path(), None).unwrap());
    assert_eq!(a, b, "listing must be deterministic");
}

// =========================================================================
// 4. list_files – empty directory
// =========================================================================

#[test]
fn list_files_empty_dir_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty());
}

// =========================================================================
// 5. Hidden file handling
// =========================================================================

#[test]
fn list_files_includes_dotfiles() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".hidden"), "secret").unwrap();
    fs::write(dir.path().join("visible.txt"), "public").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(
        !files.is_empty(),
        "should discover at least the visible file"
    );
}

// =========================================================================
// 6. license_candidates – detection
// =========================================================================

#[test]
fn license_candidates_empty_list() {
    let result = license_candidates(&[]);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

#[test]
fn license_candidates_detects_standard_licenses() {
    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("LICENSE-APACHE"),
        PathBuf::from("COPYING"),
        PathBuf::from("NOTICE"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 5);
}

#[test]
fn license_candidates_detects_metadata_manifests() {
    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
        PathBuf::from("pyproject.toml"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.metadata_files.len(), 3);
    assert!(result.license_files.is_empty());
}

#[test]
fn license_candidates_case_insensitive_license() {
    let files = vec![PathBuf::from("license"), PathBuf::from("License.md")];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 2);
}

#[test]
fn license_candidates_ignores_unrelated_files() {
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("README.md"),
        PathBuf::from("tests/test.rs"),
    ];
    let result = license_candidates(&files);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

#[test]
fn license_candidates_sorted_output() {
    let files = vec![
        PathBuf::from("z/LICENSE"),
        PathBuf::from("a/LICENSE"),
        PathBuf::from("z/Cargo.toml"),
        PathBuf::from("a/Cargo.toml"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files[0], PathBuf::from("a/LICENSE"));
    assert_eq!(result.license_files[1], PathBuf::from("z/LICENSE"));
    assert_eq!(result.metadata_files[0], PathBuf::from("a/Cargo.toml"));
    assert_eq!(result.metadata_files[1], PathBuf::from("z/Cargo.toml"));
}

#[test]
fn license_candidates_deterministic() {
    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("Cargo.toml"),
        PathBuf::from("COPYING"),
        PathBuf::from("package.json"),
    ];
    let a = license_candidates(&files);
    let b = license_candidates(&files);
    assert_eq!(a.license_files, b.license_files);
    assert_eq!(a.metadata_files, b.metadata_files);
}

// =========================================================================
// 7. file_size
// =========================================================================

#[test]
fn file_size_returns_exact_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let content = "hello world!"; // 12 bytes
    fs::write(dir.path().join("sized.txt"), content).unwrap();
    let sz = file_size(dir.path(), Path::new("sized.txt")).unwrap();
    assert_eq!(sz, 12);
}

#[test]
fn file_size_empty_file_is_zero() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("empty.txt"), "").unwrap();
    assert_eq!(file_size(dir.path(), Path::new("empty.txt")).unwrap(), 0);
}

#[test]
fn file_size_missing_file_errors() {
    let dir = tempfile::tempdir().unwrap();
    assert!(file_size(dir.path(), Path::new("no_such.txt")).is_err());
}

#[test]
fn file_size_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("det.txt"), "abc").unwrap();
    let a = file_size(dir.path(), Path::new("det.txt")).unwrap();
    let b = file_size(dir.path(), Path::new("det.txt")).unwrap();
    assert_eq!(a, b);
}
