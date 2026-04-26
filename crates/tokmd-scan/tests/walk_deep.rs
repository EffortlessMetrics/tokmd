//! Deep tests for tokmd-scan walk helpers filesystem traversal.

use std::fs;
use std::path::{Path, PathBuf};
use tokmd_scan::walk::{file_size, license_candidates, list_files};

// ---- list_files basic ----

#[test]
fn list_files_on_tempdir_with_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
    fs::write(dir.path().join("b.rs"), "fn foo() {}").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 2);
}

#[test]
fn list_files_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty(), "Empty directory should yield no files");
}

#[test]
fn list_files_single_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("only.txt"), "content").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn list_files_nested_directories() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a").join("b").join("c")).unwrap();
    fs::write(
        dir.path().join("a").join("b").join("c").join("deep.rs"),
        "x",
    )
    .unwrap();
    fs::write(dir.path().join("top.rs"), "y").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 2);
}

#[test]
fn list_files_max_zero_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "content").unwrap();
    let files = list_files(dir.path(), Some(0)).unwrap();
    assert!(files.is_empty());
}

#[test]
fn list_files_respects_max_limit() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..10 {
        fs::write(dir.path().join(format!("file{i}.txt")), "x").unwrap();
    }
    let files = list_files(dir.path(), Some(3)).unwrap();
    assert!(
        files.len() <= 3,
        "Expected at most 3 files, got {}",
        files.len()
    );
}

#[test]
fn list_files_max_one() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    fs::write(dir.path().join("b.txt"), "b").unwrap();
    let files = list_files(dir.path(), Some(1)).unwrap();
    assert!(files.len() <= 1);
}

#[test]
fn list_files_max_larger_than_total() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    let files = list_files(dir.path(), Some(100)).unwrap();
    assert_eq!(files.len(), 1);
}

// ---- Path normalization ----

#[test]
fn list_files_returns_relative_paths() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("test.rs"), "code").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    for f in &files {
        assert!(f.is_relative(), "Path {:?} should be relative", f);
    }
}

// ---- Deterministic output (sorted) ----

#[test]
fn list_files_sorted_output() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("z.txt"), "z").unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    fs::write(dir.path().join("m.txt"), "m").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "Files should be sorted");
}

#[test]
fn list_files_deterministic_repeated_calls() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..5 {
        fs::write(dir.path().join(format!("f{i}.rs")), format!("// {i}")).unwrap();
    }
    let files1 = list_files(dir.path(), None).unwrap();
    let files2 = list_files(dir.path(), None).unwrap();
    assert_eq!(
        files1, files2,
        "Repeated calls must produce identical results"
    );
}

// ---- license_candidates ----

#[test]
fn license_candidates_detects_license_files() {
    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("LICENSE.md"),
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("COPYING"),
        PathBuf::from("NOTICE"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 5);
}

#[test]
fn license_candidates_detects_metadata_files() {
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
fn license_candidates_empty_input() {
    let result = license_candidates(&[]);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

#[test]
fn license_candidates_case_insensitive() {
    let files = vec![
        PathBuf::from("license"),
        PathBuf::from("License.txt"),
        PathBuf::from("LICENSE-APACHE"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 3);
}

#[test]
fn license_candidates_no_false_positives() {
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("README.md"),
        PathBuf::from("Makefile"),
        PathBuf::from(".gitignore"),
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
fn license_candidates_mixed() {
    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("Cargo.toml"),
        PathBuf::from("src/lib.rs"),
        PathBuf::from("NOTICE.txt"),
        PathBuf::from("package.json"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 2);
    assert_eq!(result.metadata_files.len(), 2);
}

#[test]
fn license_candidates_copying_variant() {
    let files = vec![PathBuf::from("COPYING.LESSER"), PathBuf::from("copying")];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 2);
}

#[test]
fn license_candidates_notice_variant() {
    let files = vec![PathBuf::from("NOTICE"), PathBuf::from("notice.txt")];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 2);
}

// ---- file_size ----

#[test]
fn file_size_returns_correct_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let content = "hello world";
    fs::write(dir.path().join("test.txt"), content).unwrap();
    let size = file_size(dir.path(), Path::new("test.txt")).unwrap();
    assert_eq!(size, content.len() as u64);
}

#[test]
fn file_size_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("empty.txt"), "").unwrap();
    let size = file_size(dir.path(), Path::new("empty.txt")).unwrap();
    assert_eq!(size, 0);
}

#[test]
fn file_size_missing_file_errors() {
    let dir = tempfile::tempdir().unwrap();
    let result = file_size(dir.path(), Path::new("nonexistent.txt"));
    assert!(result.is_err());
}

#[test]
fn file_size_large_content() {
    let dir = tempfile::tempdir().unwrap();
    let content = "x".repeat(10_000);
    fs::write(dir.path().join("big.txt"), &content).unwrap();
    let size = file_size(dir.path(), Path::new("big.txt")).unwrap();
    assert_eq!(size, 10_000);
}

#[test]
fn file_size_nested_path() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub").join("file.txt"), "abc").unwrap();
    let size = file_size(dir.path(), Path::new("sub/file.txt")).unwrap();
    assert_eq!(size, 3);
}

// ---- LicenseCandidates struct ----

#[test]
fn license_candidates_debug_impl() {
    let result = license_candidates(&[PathBuf::from("LICENSE")]);
    let debug = format!("{:?}", result);
    assert!(debug.contains("LicenseCandidates"));
}

#[test]
fn license_candidates_clone() {
    let result = license_candidates(&[PathBuf::from("LICENSE")]);
    let cloned = result.clone();
    assert_eq!(result.license_files, cloned.license_files);
    assert_eq!(result.metadata_files, cloned.metadata_files);
}

// ---- Property-based tests ----

mod properties {
    use proptest::prelude::*;
    use std::path::PathBuf;
    use tokmd_scan::walk::license_candidates;

    proptest! {
        #[test]
        fn license_candidates_never_panics(
            files in proptest::collection::vec("[a-zA-Z0-9_./-]{1,30}", 0..20)
        ) {
            let paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
            let _ = license_candidates(&paths);
        }

        #[test]
        fn license_files_always_sorted(
            files in proptest::collection::vec("[a-zA-Z0-9_./-]{1,30}", 0..20)
        ) {
            let paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
            let result = license_candidates(&paths);
            let names: Vec<String> = result.license_files.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let mut sorted = names.clone();
            sorted.sort();
            prop_assert_eq!(names, sorted);
        }

        #[test]
        fn metadata_files_always_sorted(
            files in proptest::collection::vec("[a-zA-Z0-9_./-]{1,30}", 0..20)
        ) {
            let paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
            let result = license_candidates(&paths);
            let names: Vec<String> = result.metadata_files.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let mut sorted = names.clone();
            sorted.sort();
            prop_assert_eq!(names, sorted);
        }
    }
}
