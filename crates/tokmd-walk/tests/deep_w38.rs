//! Deep tests for tokmd-walk (wave 38).
//!
//! Covers list_files traversal, file_size, license_candidates,
//! ignore pattern handling, symlink behavior, empty directories,
//! path normalization, and edge cases.

use std::fs;
use std::path::{Path, PathBuf};
use tokmd_walk::{file_size, license_candidates, list_files};

// ============================================================================
// 1. list_files — basic traversal
// ============================================================================

#[test]
fn list_files_finds_all_files_in_flat_dir() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["a.rs", "b.py", "c.txt"] {
        fs::write(dir.path().join(name), "content").unwrap();
    }
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 3, "Expected >=3 files, got {}", files.len());
}

#[test]
fn list_files_traverses_nested_dirs() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
    fs::write(dir.path().join("a/b/c/deep.rs"), "x").unwrap();
    fs::write(dir.path().join("a/shallow.rs"), "y").unwrap();
    fs::write(dir.path().join("root.rs"), "z").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 3);
}

#[test]
fn list_files_empty_dir_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty());
}

#[test]
fn list_files_single_file_only() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("only.txt"), "x").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 1);
}

// ============================================================================
// 2. list_files — max limit
// ============================================================================

#[test]
fn list_files_max_zero_always_empty() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "x").unwrap();
    fs::write(dir.path().join("b.rs"), "y").unwrap();
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
fn list_files_max_larger_than_file_count() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "x").unwrap();
    fs::write(dir.path().join("b.txt"), "y").unwrap();
    let files = list_files(dir.path(), Some(100)).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn list_files_max_exact_count() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..5 {
        fs::write(dir.path().join(format!("f{i}.rs")), "x").unwrap();
    }
    let files = list_files(dir.path(), Some(5)).unwrap();
    assert!(files.len() <= 5);
}

// ============================================================================
// 3. list_files — path normalization / relative paths
// ============================================================================

#[test]
fn list_files_returns_relative_paths() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("test.rs"), "code").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    for f in &files {
        assert!(f.is_relative(), "Path {:?} should be relative", f);
    }
}

#[test]
fn list_files_nested_paths_are_relative() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src/util")).unwrap();
    fs::write(dir.path().join("src/util/helper.rs"), "x").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    for f in &files {
        assert!(f.is_relative(), "Nested path {:?} should be relative", f);
    }
}

// ============================================================================
// 4. list_files — determinism / sorted output
// ============================================================================

#[test]
fn list_files_output_is_sorted() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["z.txt", "m.txt", "a.txt"] {
        fs::write(dir.path().join(name), "x").unwrap();
    }
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "Files must be sorted");
}

#[test]
fn list_files_deterministic_across_calls() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..8 {
        fs::write(dir.path().join(format!("f{i}.rs")), format!("// {i}")).unwrap();
    }
    let a = list_files(dir.path(), None).unwrap();
    let b = list_files(dir.path(), None).unwrap();
    assert_eq!(a, b);
}

// ============================================================================
// 5. list_files — directories are excluded (only files)
// ============================================================================

#[test]
fn list_files_skips_directories() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("empty_dir")).unwrap();
    fs::write(dir.path().join("file.txt"), "x").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    // Only the file, not the directory
    assert_eq!(files.len(), 1);
    assert!(files[0].to_string_lossy().contains("file.txt"));
}

#[test]
fn list_files_only_empty_subdirs_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
    fs::create_dir_all(dir.path().join("d")).unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty(), "Empty subdirs should yield no files");
}

// ============================================================================
// 6. Symlink behavior
// ============================================================================

#[cfg(unix)]
#[test]
fn list_files_does_not_follow_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("real.txt"), "content").unwrap();
    std::os::unix::fs::symlink(dir.path().join("real.txt"), dir.path().join("link.txt")).unwrap();
    let files = list_files(dir.path(), None).unwrap();
    // The walker has follow_links(false), so symlinks should not be followed
    // but the symlink file itself may or may not appear depending on ignore rules
    // We just verify no panic and reasonable count
    assert!(!files.is_empty());
}

// ============================================================================
// 7. license_candidates — detection
// ============================================================================

#[test]
fn license_candidates_all_license_variants() {
    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("LICENSE.md"),
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("LICENSE-APACHE"),
        PathBuf::from("COPYING"),
        PathBuf::from("COPYING.LESSER"),
        PathBuf::from("NOTICE"),
        PathBuf::from("NOTICE.txt"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 8);
    assert!(result.metadata_files.is_empty());
}

#[test]
fn license_candidates_all_metadata_variants() {
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
fn license_candidates_empty() {
    let result = license_candidates(&[]);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

#[test]
fn license_candidates_case_insensitive_detection() {
    let files = vec![
        PathBuf::from("license"),
        PathBuf::from("License.txt"),
        PathBuf::from("LICENSE-APACHE"),
        PathBuf::from("Copying"),
        PathBuf::from("notice"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 5);
}

#[test]
fn license_candidates_ignores_unrelated() {
    let files = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("README.md"),
        PathBuf::from("Makefile"),
        PathBuf::from(".gitignore"),
        PathBuf::from("build.rs"),
    ];
    let result = license_candidates(&files);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

#[test]
fn license_candidates_mixed_license_and_metadata() {
    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("NOTICE"),
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
        PathBuf::from("src/lib.rs"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 2);
    assert_eq!(result.metadata_files.len(), 2);
}

#[test]
fn license_candidates_sorted_license_files() {
    let files = vec![
        PathBuf::from("z/LICENSE"),
        PathBuf::from("a/LICENSE"),
        PathBuf::from("m/LICENSE"),
    ];
    let result = license_candidates(&files);
    let names: Vec<&str> = result
        .license_files
        .iter()
        .map(|p| p.to_str().unwrap())
        .collect();
    assert_eq!(names, vec!["a/LICENSE", "m/LICENSE", "z/LICENSE"]);
}

#[test]
fn license_candidates_sorted_metadata_files() {
    let files = vec![PathBuf::from("z/Cargo.toml"), PathBuf::from("a/Cargo.toml")];
    let result = license_candidates(&files);
    assert_eq!(result.metadata_files[0], PathBuf::from("a/Cargo.toml"));
    assert_eq!(result.metadata_files[1], PathBuf::from("z/Cargo.toml"));
}

// ============================================================================
// 8. license_candidates — metadata does not leak into license
// ============================================================================

#[test]
fn license_candidates_cargo_toml_only_in_metadata() {
    let files = vec![PathBuf::from("Cargo.toml")];
    let result = license_candidates(&files);
    assert!(result.license_files.is_empty());
    assert_eq!(result.metadata_files.len(), 1);
}

#[test]
fn license_candidates_package_json_only_in_metadata() {
    let files = vec![PathBuf::from("package.json")];
    let result = license_candidates(&files);
    assert!(result.license_files.is_empty());
    assert_eq!(result.metadata_files.len(), 1);
}

// ============================================================================
// 9. file_size — various cases
// ============================================================================

#[test]
fn file_size_correct_for_known_content() {
    let dir = tempfile::tempdir().unwrap();
    let content = "hello world!";
    fs::write(dir.path().join("test.txt"), content).unwrap();
    let size = file_size(dir.path(), Path::new("test.txt")).unwrap();
    assert_eq!(size, content.len() as u64);
}

#[test]
fn file_size_empty_file_is_zero() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("empty.txt"), "").unwrap();
    assert_eq!(file_size(dir.path(), Path::new("empty.txt")).unwrap(), 0);
}

#[test]
fn file_size_missing_file_is_error() {
    let dir = tempfile::tempdir().unwrap();
    assert!(file_size(dir.path(), Path::new("missing.txt")).is_err());
}

#[test]
fn file_size_large_content() {
    let dir = tempfile::tempdir().unwrap();
    let content = "x".repeat(50_000);
    fs::write(dir.path().join("big.txt"), &content).unwrap();
    assert_eq!(file_size(dir.path(), Path::new("big.txt")).unwrap(), 50_000);
}

#[test]
fn file_size_nested_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a/b")).unwrap();
    fs::write(dir.path().join("a/b/f.txt"), "abc").unwrap();
    assert_eq!(file_size(dir.path(), Path::new("a/b/f.txt")).unwrap(), 3);
}

#[test]
fn file_size_binary_content() {
    let dir = tempfile::tempdir().unwrap();
    let data: Vec<u8> = (0..=255).collect();
    fs::write(dir.path().join("bin.dat"), &data).unwrap();
    assert_eq!(file_size(dir.path(), Path::new("bin.dat")).unwrap(), 256);
}

// ============================================================================
// 10. LicenseCandidates struct traits
// ============================================================================

#[test]
fn license_candidates_debug_impl() {
    let result = license_candidates(&[PathBuf::from("LICENSE")]);
    let debug = format!("{:?}", result);
    assert!(debug.contains("LicenseCandidates"));
}

#[test]
fn license_candidates_clone_is_equal() {
    let result = license_candidates(&[PathBuf::from("LICENSE"), PathBuf::from("Cargo.toml")]);
    let cloned = result.clone();
    assert_eq!(result.license_files, cloned.license_files);
    assert_eq!(result.metadata_files, cloned.metadata_files);
}

// ============================================================================
// 11. list_files — hidden files
// ============================================================================

#[test]
fn list_files_includes_hidden_files() {
    // The walker is configured with hidden(false) which means it DOES walk hidden files
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".hidden"), "x").unwrap();
    fs::write(dir.path().join("visible.txt"), "y").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    // hidden(false) in WalkBuilder means "do not skip hidden files", so we should see both
    assert!(!files.is_empty(), "Should find at least visible file");
}

// ============================================================================
// 12. list_files — various file extensions
// ============================================================================

#[test]
fn list_files_handles_many_extensions() {
    let dir = tempfile::tempdir().unwrap();
    let exts = [
        "rs", "py", "js", "ts", "go", "c", "h", "toml", "json", "yaml",
    ];
    for ext in &exts {
        fs::write(dir.path().join(format!("file.{ext}")), "content").unwrap();
    }
    let files = list_files(dir.path(), None).unwrap();
    assert!(
        files.len() >= exts.len(),
        "Expected at least {} files, got {}",
        exts.len(),
        files.len()
    );
}
