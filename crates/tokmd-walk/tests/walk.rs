//! Tests for tokmd-walk file listing utilities.

use std::path::PathBuf;

use tempfile::TempDir;
use tokmd_walk::{file_size, license_candidates, list_files};

fn git_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

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

// ========================
// List Files Tests
// ========================

#[test]
fn list_files_returns_files_in_directory() {
    let temp = TempDir::new().unwrap();

    // Create some files
    std::fs::write(temp.path().join("a.txt"), "content").unwrap();
    std::fs::write(temp.path().join("b.txt"), "content").unwrap();
    std::fs::write(temp.path().join("c.txt"), "content").unwrap();

    let files = list_files(temp.path(), None).unwrap();

    // Must return non-empty (kills: list_files -> Ok(vec![]))
    assert!(
        !files.is_empty(),
        "list_files should return files, not empty vec"
    );

    // Must contain actual file paths (kills: list_files -> Ok(vec![Default::default()]))
    for f in &files {
        let name = f.to_string_lossy();
        assert!(
            name.contains(".txt"),
            "file should have .txt extension, got: {}",
            name
        );
    }

    assert_eq!(files.len(), 3);
}

#[test]
fn list_files_with_max_files_zero_returns_empty() {
    let temp = TempDir::new().unwrap();

    // Create files
    std::fs::write(temp.path().join("a.txt"), "content").unwrap();
    std::fs::write(temp.path().join("b.txt"), "content").unwrap();

    // With max_files=0, should return empty (kills: == with != mutation)
    let files = list_files(temp.path(), Some(0)).unwrap();
    assert!(
        files.is_empty(),
        "max_files=0 should return empty vec, got {} files",
        files.len()
    );
}

#[test]
fn list_files_with_max_files_one_returns_one() {
    let temp = TempDir::new().unwrap();

    // Create multiple files
    std::fs::write(temp.path().join("a.txt"), "content").unwrap();
    std::fs::write(temp.path().join("b.txt"), "content").unwrap();
    std::fs::write(temp.path().join("c.txt"), "content").unwrap();

    // With max_files=1, should return exactly 1 file (kills: >= with < mutations)
    let files = list_files(temp.path(), Some(1)).unwrap();
    assert_eq!(files.len(), 1, "max_files=1 should return exactly 1 file");
}

#[test]
fn list_files_with_max_files_two_returns_two() {
    let temp = TempDir::new().unwrap();

    // Create multiple files
    std::fs::write(temp.path().join("a.txt"), "content").unwrap();
    std::fs::write(temp.path().join("b.txt"), "content").unwrap();
    std::fs::write(temp.path().join("c.txt"), "content").unwrap();

    // With max_files=2, should return exactly 2 files (kills: > vs >= mutations)
    let files = list_files(temp.path(), Some(2)).unwrap();
    assert_eq!(files.len(), 2, "max_files=2 should return exactly 2 files");
}

#[test]
fn list_files_excludes_directories() {
    let temp = TempDir::new().unwrap();

    // Create a file
    std::fs::write(temp.path().join("file.txt"), "content").unwrap();

    // Create a subdirectory
    std::fs::create_dir(temp.path().join("subdir")).unwrap();

    // Create a file in subdirectory
    std::fs::write(temp.path().join("subdir/nested.txt"), "content").unwrap();

    let files = list_files(temp.path(), None).unwrap();

    // Should only contain files, not directories (kills: delete ! in is_file check)
    for f in &files {
        let full_path = temp.path().join(f);
        assert!(
            full_path.is_file(),
            "list_files should only return files, not directories: {}",
            f.display()
        );
    }

    // Should have 2 files (file.txt and subdir/nested.txt)
    assert_eq!(files.len(), 2);
}

#[test]
fn list_files_in_git_repo_returns_tracked_files() {
    if !git_available() {
        return;
    }
    let temp = TempDir::new().unwrap();

    // Initialize a git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .output()
        .expect("git init failed");

    // Configure git user for commit
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(temp.path())
        .output()
        .expect("git config email failed");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp.path())
        .output()
        .expect("git config name failed");

    // Explicitly disable GPG signing to prevent test failures in environments without GPG
    std::process::Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(temp.path())
        .output()
        .expect("git config gpgsign failed");

    // Create and add files
    std::fs::write(temp.path().join("tracked.txt"), "tracked").unwrap();
    std::fs::write(temp.path().join("also_tracked.txt"), "also tracked").unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .expect("git add failed");

    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(temp.path())
        .output()
        .expect("git commit failed");

    // Create an untracked file
    std::fs::write(temp.path().join("untracked.txt"), "untracked").unwrap();

    let files = list_files(temp.path(), None).unwrap();

    // git_ls_files should return tracked files only (kills git_ls_files mutations)
    assert!(!files.is_empty(), "should return tracked files");

    // Should have tracked files (and NOT the untracked one)
    let file_names: Vec<String> = files
        .iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect();
    assert!(
        file_names.contains(&"tracked.txt".to_string()),
        "should contain tracked.txt"
    );
    assert!(
        file_names.contains(&"also_tracked.txt".to_string()),
        "should contain also_tracked.txt"
    );
    assert!(
        !file_names.contains(&"untracked.txt".to_string()),
        "should NOT contain untracked.txt"
    );
}

#[test]
fn list_files_in_git_repo_respects_max_files() {
    if !git_available() {
        return;
    }
    let temp = TempDir::new().unwrap();

    // Initialize a git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .output()
        .expect("git init failed");

    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(temp.path())
        .output()
        .expect("git config email failed");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp.path())
        .output()
        .expect("git config name failed");

    // Explicitly disable GPG signing
    std::process::Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(temp.path())
        .output()
        .expect("git config gpgsign failed");

    // Create multiple files
    std::fs::write(temp.path().join("a.txt"), "a").unwrap();
    std::fs::write(temp.path().join("b.txt"), "b").unwrap();
    std::fs::write(temp.path().join("c.txt"), "c").unwrap();
    std::fs::write(temp.path().join("d.txt"), "d").unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .expect("git add failed");

    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(temp.path())
        .output()
        .expect("git commit failed");

    // Test truncation with max_files (kills > mutations for git path)
    let files = list_files(temp.path(), Some(2)).unwrap();
    assert_eq!(
        files.len(),
        2,
        "max_files=2 should truncate git output to 2 files"
    );
}

#[test]
fn list_files_in_git_repo_max_files_equals_file_count() {
    if !git_available() {
        return;
    }
    // This test attempts to kill the `> with >=` mutant for the git truncation path.
    // When limit == files.len(), we should NOT truncate.
    // Note: This mutation may be semantically equivalent since truncate(n) when len==n is a no-op.
    let temp = TempDir::new().unwrap();

    // Initialize a git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .output()
        .expect("git init failed");

    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(temp.path())
        .output()
        .expect("git config email failed");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp.path())
        .output()
        .expect("git config name failed");

    // Explicitly disable GPG signing
    std::process::Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(temp.path())
        .output()
        .expect("git config gpgsign failed");

    // Create exactly 3 files
    std::fs::write(temp.path().join("a.txt"), "a").unwrap();
    std::fs::write(temp.path().join("b.txt"), "b").unwrap();
    std::fs::write(temp.path().join("c.txt"), "c").unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .expect("git add failed");

    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(temp.path())
        .output()
        .expect("git commit failed");

    // With max_files=3 and exactly 3 files, should return all 3 (not truncate)
    // This catches the `> with >=` mutation: 3 > 3 is false, but 3 >= 3 is true
    let files = list_files(temp.path(), Some(3)).unwrap();
    assert_eq!(
        files.len(),
        3,
        "max_files=3 with exactly 3 files should return all 3"
    );

    // Verify specific files are present
    let file_names: Vec<String> = files
        .iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect();
    assert!(file_names.contains(&"a.txt".to_string()));
    assert!(file_names.contains(&"b.txt".to_string()));
    assert!(file_names.contains(&"c.txt".to_string()));
}

#[test]
fn list_files_in_git_repo_with_max_files_zero() {
    if !git_available() {
        return;
    }
    let temp = TempDir::new().unwrap();

    // Initialize a git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .output()
        .expect("git init failed");

    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(temp.path())
        .output()
        .expect("git config email failed");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp.path())
        .output()
        .expect("git config name failed");

    // Explicitly disable GPG signing
    std::process::Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(temp.path())
        .output()
        .expect("git config gpgsign failed");

    // Create and commit a file
    std::fs::write(temp.path().join("file.txt"), "content").unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .expect("git add failed");

    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(temp.path())
        .output()
        .expect("git commit failed");

    // max_files=0 should still return empty even in git repo
    let files = list_files(temp.path(), Some(0)).unwrap();
    assert!(
        files.is_empty(),
        "max_files=0 should return empty even in git repo"
    );
}

#[test]
fn list_files_sorted_alphabetically() {
    let temp = TempDir::new().unwrap();

    // Create files in non-sorted order
    std::fs::write(temp.path().join("zebra.txt"), "z").unwrap();
    std::fs::write(temp.path().join("apple.txt"), "a").unwrap();
    std::fs::write(temp.path().join("mango.txt"), "m").unwrap();

    let files = list_files(temp.path(), None).unwrap();

    // Files should be sorted alphabetically
    let file_names: Vec<String> = files
        .iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect();

    assert_eq!(file_names[0], "apple.txt");
    assert_eq!(file_names[1], "mango.txt");
    assert_eq!(file_names[2], "zebra.txt");
}
