//! Tests for tokmd-walk file listing utilities.

use std::path::PathBuf;

use tempfile::TempDir;
use tokmd_walk::{file_size, license_candidates};

// ========================
// License Candidates Tests
// ========================

#[test]
fn license_candidates_finds_license_file() {
    let files = vec![
        PathBuf::from("src/lib.rs"),
        PathBuf::from("LICENSE"),
        PathBuf::from("README.md"),
    ];

    let candidates = license_candidates(&files);
    assert_eq!(candidates.license_files, vec![PathBuf::from("LICENSE")]);
    assert!(candidates.metadata_files.is_empty());
}

#[test]
fn license_candidates_finds_license_with_extension() {
    let files = vec![
        PathBuf::from("LICENSE.txt"),
        PathBuf::from("LICENSE.md"),
        PathBuf::from("LICENSE-MIT"),
    ];

    let candidates = license_candidates(&files);
    assert_eq!(candidates.license_files.len(), 3);
    assert!(
        candidates
            .license_files
            .contains(&PathBuf::from("LICENSE.txt"))
    );
    assert!(
        candidates
            .license_files
            .contains(&PathBuf::from("LICENSE.md"))
    );
    assert!(
        candidates
            .license_files
            .contains(&PathBuf::from("LICENSE-MIT"))
    );
}

#[test]
fn license_candidates_finds_copying_file() {
    let files = vec![PathBuf::from("COPYING"), PathBuf::from("src/main.rs")];

    let candidates = license_candidates(&files);
    assert_eq!(candidates.license_files, vec![PathBuf::from("COPYING")]);
}

#[test]
fn license_candidates_finds_notice_file() {
    let files = vec![PathBuf::from("NOTICE"), PathBuf::from("NOTICE.txt")];

    let candidates = license_candidates(&files);
    assert_eq!(candidates.license_files.len(), 2);
}

#[test]
fn license_candidates_case_insensitive() {
    let files = vec![
        PathBuf::from("license"),
        PathBuf::from("License"),
        PathBuf::from("LICENSE"),
        PathBuf::from("copying"),
        PathBuf::from("Copying"),
        PathBuf::from("COPYING"),
        PathBuf::from("notice"),
        PathBuf::from("Notice"),
        PathBuf::from("NOTICE"),
    ];

    let candidates = license_candidates(&files);
    // All should be found due to case-insensitive matching
    assert_eq!(candidates.license_files.len(), 9);
}

#[test]
fn license_candidates_finds_cargo_toml() {
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("src/lib.rs")];

    let candidates = license_candidates(&files);
    assert!(candidates.license_files.is_empty());
    assert_eq!(candidates.metadata_files, vec![PathBuf::from("Cargo.toml")]);
}

#[test]
fn license_candidates_finds_package_json() {
    let files = vec![PathBuf::from("package.json"), PathBuf::from("src/index.js")];

    let candidates = license_candidates(&files);
    assert!(candidates.license_files.is_empty());
    assert_eq!(
        candidates.metadata_files,
        vec![PathBuf::from("package.json")]
    );
}

#[test]
fn license_candidates_finds_pyproject_toml() {
    let files = vec![
        PathBuf::from("pyproject.toml"),
        PathBuf::from("src/__init__.py"),
    ];

    let candidates = license_candidates(&files);
    assert!(candidates.license_files.is_empty());
    assert_eq!(
        candidates.metadata_files,
        vec![PathBuf::from("pyproject.toml")]
    );
}

#[test]
fn license_candidates_metadata_case_insensitive() {
    let files = vec![
        PathBuf::from("cargo.toml"),
        PathBuf::from("Cargo.toml"),
        PathBuf::from("CARGO.TOML"),
        PathBuf::from("Package.json"),
        PathBuf::from("PACKAGE.JSON"),
        PathBuf::from("PyProject.toml"),
    ];

    let candidates = license_candidates(&files);
    // Case insensitive matching
    assert_eq!(candidates.metadata_files.len(), 6);
}

#[test]
fn license_candidates_nested_files() {
    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("packages/foo/LICENSE"),
        PathBuf::from("packages/foo/Cargo.toml"),
        PathBuf::from("packages/bar/package.json"),
    ];

    let candidates = license_candidates(&files);
    assert_eq!(candidates.license_files.len(), 2);
    assert_eq!(candidates.metadata_files.len(), 2);
}

#[test]
fn license_candidates_sorted_output() {
    let files = vec![
        PathBuf::from("packages/zoo/LICENSE"),
        PathBuf::from("LICENSE"),
        PathBuf::from("apps/LICENSE"),
        PathBuf::from("packages/zoo/Cargo.toml"),
        PathBuf::from("Cargo.toml"),
        PathBuf::from("apps/package.json"),
    ];

    let candidates = license_candidates(&files);

    // License files should be sorted
    assert_eq!(candidates.license_files[0], PathBuf::from("LICENSE"));
    assert_eq!(candidates.license_files[1], PathBuf::from("apps/LICENSE"));
    assert_eq!(
        candidates.license_files[2],
        PathBuf::from("packages/zoo/LICENSE")
    );

    // Metadata files should be sorted
    assert_eq!(candidates.metadata_files[0], PathBuf::from("Cargo.toml"));
    assert_eq!(
        candidates.metadata_files[1],
        PathBuf::from("apps/package.json")
    );
    assert_eq!(
        candidates.metadata_files[2],
        PathBuf::from("packages/zoo/Cargo.toml")
    );
}

#[test]
fn license_candidates_empty_input() {
    let files: Vec<PathBuf> = vec![];

    let candidates = license_candidates(&files);
    assert!(candidates.license_files.is_empty());
    assert!(candidates.metadata_files.is_empty());
}

#[test]
fn license_candidates_no_matches() {
    let files = vec![
        PathBuf::from("src/lib.rs"),
        PathBuf::from("tests/test.rs"),
        PathBuf::from("README.md"),
    ];

    let candidates = license_candidates(&files);
    assert!(candidates.license_files.is_empty());
    assert!(candidates.metadata_files.is_empty());
}

#[test]
fn license_candidates_distinguishes_license_vs_metadata() {
    let files = vec![PathBuf::from("LICENSE"), PathBuf::from("Cargo.toml")];

    let candidates = license_candidates(&files);
    // LICENSE should be in license_files, not metadata_files
    assert_eq!(candidates.license_files, vec![PathBuf::from("LICENSE")]);
    // Cargo.toml should be in metadata_files, not license_files
    assert_eq!(candidates.metadata_files, vec![PathBuf::from("Cargo.toml")]);
}

// ========================
// File Size Tests
// ========================

#[test]
fn file_size_basic() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std::fs::write(&file_path, "hello world").unwrap();

    let size = file_size(temp.path(), std::path::Path::new("test.txt")).unwrap();
    assert_eq!(size, 11);
}

#[test]
fn file_size_empty_file() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("empty.txt");
    std::fs::write(&file_path, "").unwrap();

    let size = file_size(temp.path(), std::path::Path::new("empty.txt")).unwrap();
    assert_eq!(size, 0);
}

#[test]
fn file_size_nested_path() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join("sub/dir")).unwrap();
    let file_path = temp.path().join("sub/dir/file.txt");
    std::fs::write(&file_path, "content here").unwrap();

    let size = file_size(temp.path(), std::path::Path::new("sub/dir/file.txt")).unwrap();
    assert_eq!(size, 12);
}

#[test]
fn file_size_nonexistent_file() {
    let temp = TempDir::new().unwrap();

    let result = file_size(temp.path(), std::path::Path::new("nonexistent.txt"));
    assert!(result.is_err());
}

#[test]
fn file_size_with_bytes() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("binary.bin");
    // Write 1024 bytes
    std::fs::write(&file_path, vec![0u8; 1024]).unwrap();

    let size = file_size(temp.path(), std::path::Path::new("binary.bin")).unwrap();
    assert_eq!(size, 1024);
}
