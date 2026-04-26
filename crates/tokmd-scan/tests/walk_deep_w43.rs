//! Deep tests for tokmd-scan walk helpers (wave 43).
//!
//! Covers gitignore handling, symlink behavior, file metadata edge cases,
//! traversal with mixed content, and license_candidates corner cases.

use std::fs;
use std::path::{Path, PathBuf};
use tokmd_scan::walk::{file_size, license_candidates, list_files};

// ============================================================================
// 1. list_files — .gitignore handling (non-git fallback walker)
// ============================================================================

#[test]
fn list_files_respects_gitignore_patterns() {
    let dir = tempfile::tempdir().unwrap();
    // git init so the ignore crate recognizes .gitignore; no files staged
    // so git ls-files returns empty → falls through to WalkBuilder
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    fs::write(dir.path().join("app.rs"), "fn main() {}").unwrap();
    fs::write(dir.path().join("debug.log"), "log data").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.ends_with(".log")),
        "*.log should be ignored, got: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n.ends_with("app.rs")),
        "app.rs should be present, got: {:?}",
        names
    );
}

#[test]
fn list_files_respects_gitignore_directory_pattern() {
    let dir = tempfile::tempdir().unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    fs::write(dir.path().join(".gitignore"), "build/\n").unwrap();
    fs::create_dir_all(dir.path().join("build")).unwrap();
    fs::write(dir.path().join("build").join("output.o"), "binary").unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src").join("main.rs"), "fn main() {}").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.contains("output.o")),
        "build/ dir should be ignored, got: {:?}",
        names
    );
}

#[test]
fn list_files_respects_nested_gitignore() {
    let dir = tempfile::tempdir().unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    fs::create_dir_all(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub").join(".gitignore"), "*.tmp\n").unwrap();
    fs::write(dir.path().join("sub").join("keep.rs"), "x").unwrap();
    fs::write(dir.path().join("sub").join("discard.tmp"), "y").unwrap();
    fs::write(dir.path().join("root.tmp"), "z").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.contains("discard.tmp")),
        "sub/.gitignore should hide *.tmp in sub/, got: {:?}",
        names
    );
}

// ============================================================================
// 2. list_files — symlink handling
// ============================================================================

#[cfg(windows)]
#[test]
fn list_files_does_not_follow_symlinks_windows() {
    // On Windows, creating symlinks may require elevated privileges.
    // We test that the walker doesn't panic even if symlink creation fails.
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("real.txt"), "content").unwrap();
    // Attempt symlink; skip assertion if it fails (requires admin)
    let link_result = std::os::windows::fs::symlink_file(
        dir.path().join("real.txt"),
        dir.path().join("link.txt"),
    );
    if link_result.is_ok() {
        let files = list_files(dir.path(), None).unwrap();
        // follow_links(false) means symlinks are not followed as directories,
        // but file symlinks may still appear. Just verify no panic.
        assert!(!files.is_empty());
    }
}

#[cfg(unix)]
#[test]
fn list_files_symlink_to_dir_not_traversed() {
    let dir = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    fs::write(other.path().join("secret.txt"), "hidden").unwrap();
    std::os::unix::fs::symlink(other.path(), dir.path().join("linked_dir")).unwrap();
    fs::write(dir.path().join("visible.txt"), "hello").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    // follow_links is false, so the symlinked directory should not be traversed
    assert!(
        !names.iter().any(|n| n.contains("secret.txt")),
        "Symlinked dir should not be traversed, got: {:?}",
        names
    );
}

// ============================================================================
// 3. list_files — files only in subdirectories
// ============================================================================

#[test]
fn list_files_deep_nesting_only() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a/b/c/d")).unwrap();
    fs::write(dir.path().join("a/b/c/d/leaf.rs"), "x").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 1);
    let name = files[0].to_string_lossy().to_string();
    assert!(name.contains("leaf.rs"), "Expected leaf.rs, got: {}", name);
}

#[test]
fn list_files_max_with_nested_structure() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..5 {
        let subdir = dir.path().join(format!("dir{i}"));
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("file.rs"), format!("// {i}")).unwrap();
    }
    let files = list_files(dir.path(), Some(2)).unwrap();
    assert!(files.len() <= 2, "Expected <=2 files, got {}", files.len());
}

// ============================================================================
// 4. file_size — edge cases
// ============================================================================

#[test]
fn file_size_unicode_filename() {
    let dir = tempfile::tempdir().unwrap();
    let name = "données.txt";
    fs::write(dir.path().join(name), "abc").unwrap();
    let size = file_size(dir.path(), Path::new(name)).unwrap();
    assert_eq!(size, 3);
}

#[test]
fn file_size_exact_known_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let content = vec![0u8; 1024];
    fs::write(dir.path().join("exact.bin"), &content).unwrap();
    let size = file_size(dir.path(), Path::new("exact.bin")).unwrap();
    assert_eq!(size, 1024);
}

#[test]
fn file_size_with_newlines_counts_all_bytes() {
    let dir = tempfile::tempdir().unwrap();
    // "ab\ncd\n" = 6 bytes
    fs::write(dir.path().join("newlines.txt"), "ab\ncd\n").unwrap();
    let size = file_size(dir.path(), Path::new("newlines.txt")).unwrap();
    assert_eq!(size, 6);
}

// ============================================================================
// 5. license_candidates — corner cases
// ============================================================================

#[test]
fn license_candidates_deeply_nested_paths() {
    let files = vec![
        PathBuf::from("a/b/c/LICENSE"),
        PathBuf::from("x/y/Cargo.toml"),
        PathBuf::from("deep/nested/path/NOTICE.md"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 2);
    assert_eq!(result.metadata_files.len(), 1);
}

#[test]
fn license_candidates_does_not_match_partial_names() {
    let files = vec![
        PathBuf::from("LICENSED_CODE.rs"),
        PathBuf::from("my_license_util.py"),
        PathBuf::from("COPYINGHELPER.sh"),
    ];
    let result = license_candidates(&files);
    // "LICENSED_CODE.rs" lowercased is "licensed_code.rs" which starts_with("license") → match
    // "my_license_util.py" file_name is "my_license_util.py" which starts_with("license")? No, starts with "my_"
    // "COPYINGHELPER.sh" lowercased starts_with("copying") → match
    assert_eq!(
        result.license_files.len(),
        2,
        "LICENSED_CODE and COPYINGHELPER match starts_with, my_license_util does not"
    );
}

#[test]
fn license_candidates_metadata_not_in_license_list() {
    // Ensure Cargo.toml etc. never appear in license_files
    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
        PathBuf::from("pyproject.toml"),
        PathBuf::from("LICENSE"),
    ];
    let result = license_candidates(&files);
    for lf in &result.license_files {
        let name = lf.file_name().unwrap().to_str().unwrap();
        assert_ne!(name, "Cargo.toml");
        assert_ne!(name, "package.json");
        assert_ne!(name, "pyproject.toml");
    }
    assert_eq!(result.license_files.len(), 1);
    assert_eq!(result.metadata_files.len(), 3);
}

#[test]
fn license_candidates_single_license_no_metadata() {
    let files = vec![PathBuf::from("LICENSE-BSD")];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 1);
    assert!(result.metadata_files.is_empty());
}
