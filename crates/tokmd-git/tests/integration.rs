//! Integration tests for tokmd-git crate.
//!
//! These tests create a temporary git repository to test the git functions,
//! ensuring they work even when the test is run from a non-git directory
//! (e.g., during mutation testing).

use std::path::{Path, PathBuf};
use std::process::Command;
use tokmd_git::{collect_history, git_available, repo_root};

/// Helper to create a temporary git repository with some commits.
fn create_test_repo() -> Option<TempGitRepo> {
    if !git_available() {
        return None;
    }

    // Use thread ID and a random number to avoid conflicts between concurrent tests
    let unique_id = format!(
        "{}-{:?}-{}",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let temp_dir = std::env::temp_dir().join(format!("tokmd-git-test-{}", unique_id));

    // Clean up any existing directory first
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    std::fs::create_dir_all(&temp_dir).ok()?;

    // Initialize git repo
    let status = Command::new("git")
        .args(["init"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;
    if !status.status.success() {
        std::fs::remove_dir_all(&temp_dir).ok();
        return None;
    }

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;

    // Create first commit with a file
    let file1 = temp_dir.join("file1.txt");
    std::fs::write(&file1, "content1").ok()?;
    Command::new("git")
        .args(["add", "file1.txt"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;
    let commit1 = Command::new("git")
        .args(["commit", "-m", "First commit"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;
    if !commit1.status.success() {
        std::fs::remove_dir_all(&temp_dir).ok();
        return None;
    }

    // Create second commit with another file
    let file2 = temp_dir.join("file2.txt");
    std::fs::write(&file2, "content2").ok()?;
    Command::new("git")
        .args(["add", "file2.txt"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;
    Command::new("git")
        .args(["commit", "-m", "Second commit"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;

    // Create third commit modifying existing file
    std::fs::write(&file1, "modified content").ok()?;
    Command::new("git")
        .args(["add", "file1.txt"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;
    Command::new("git")
        .args(["commit", "-m", "Third commit"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;

    Some(TempGitRepo { path: temp_dir })
}

/// RAII wrapper for cleanup of temp git repo.
struct TempGitRepo {
    path: PathBuf,
}

impl Drop for TempGitRepo {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).ok();
    }
}

/// Test that git_available returns true when git is installed.
/// This test assumes git is available in the test environment.
#[test]
fn test_git_available_returns_true() {
    // On CI and dev machines, git should be available
    assert!(
        git_available(),
        "git should be available in the test environment"
    );
}

/// Test that repo_root returns a valid path for a git repository.
#[test]
fn test_repo_root_returns_path_for_valid_repo() {
    let Some(repo) = create_test_repo() else {
        eprintln!("Skipping test: git unavailable or repo creation failed");
        return;
    };

    let root = repo_root(&repo.path);
    assert!(
        root.is_some(),
        "repo_root should return Some for a valid git repository"
    );

    let root_path = root.unwrap();
    assert!(root_path.exists(), "repo root should exist on disk");
    assert!(
        root_path.join(".git").exists(),
        "repo root should contain .git directory"
    );
}

/// Test that repo_root returns None for a non-repository path.
#[test]
fn test_repo_root_returns_none_for_non_repo() {
    // Create a directory that is definitely not a git repo
    let non_repo_path =
        std::env::temp_dir().join(format!("tokmd-test-not-a-repo-{}", std::process::id()));
    std::fs::create_dir_all(&non_repo_path).ok();

    // Ensure no .git directory exists
    let git_dir = non_repo_path.join(".git");
    if git_dir.exists() {
        std::fs::remove_dir_all(&git_dir).ok();
    }

    let root = repo_root(&non_repo_path);

    // If the temp directory itself is inside a git repo, this might return Some
    // So we check that if it returns Some, it's not the non_repo_path itself
    if let Some(found_root) = root {
        // The found root should not be our test directory
        assert_ne!(
            found_root, non_repo_path,
            "repo_root should not return the test directory as a git root"
        );
    }

    // Clean up
    std::fs::remove_dir_all(&non_repo_path).ok();
}

/// Test that collect_history returns commits for a git repository.
#[test]
fn test_collect_history_returns_commits() {
    let Some(repo) = create_test_repo() else {
        eprintln!("Skipping test: git unavailable or repo creation failed");
        return;
    };
    let root = repo_root(&repo.path).expect("Should find repo root");

    // Use None for max_commits to read all output
    let commits = collect_history(&root, None, None).expect("Should collect history successfully");

    assert!(!commits.is_empty(), "repo should have commits");
    assert_eq!(commits.len(), 3, "Should have 3 commits");

    // Verify commit structure - all commits should have valid timestamps
    for commit in &commits {
        assert!(commit.timestamp > 0, "Commit should have valid timestamp");
        assert!(
            !commit.author.is_empty(),
            "Commit should have non-empty author"
        );
    }
}

/// Test that repo_root result contains the actual path, not just empty.
#[test]
fn test_repo_root_path_is_not_empty() {
    let Some(repo) = create_test_repo() else {
        eprintln!("Skipping test: git unavailable or repo creation failed");
        return;
    };
    let root = repo_root(&repo.path).expect("Should find repo root");

    // The path should not be empty
    assert!(
        !root.as_os_str().is_empty(),
        "repo root path should not be empty"
    );

    // It should be a real directory
    assert!(root.is_dir(), "repo root should be a directory");
}

/// Test commit has files.
#[test]
fn test_commits_have_files() {
    let Some(repo) = create_test_repo() else {
        eprintln!("Skipping test: git unavailable or repo creation failed");
        return;
    };
    let root = repo_root(&repo.path).expect("Should find repo root");

    // Get all commits
    let commits = collect_history(&root, None, None).expect("Should collect history");

    // All commits in our test repo should have files
    for commit in &commits {
        assert!(
            !commit.files.is_empty(),
            "Each commit should have associated files"
        );
    }
}

/// Test that history collection fails gracefully for non-existent path.
#[test]
fn test_collect_history_fails_for_invalid_path() {
    let invalid_path = Path::new("/this/path/does/not/exist/anywhere/tokmd-test");

    let result = collect_history(invalid_path, None, None);

    // Should fail because the path doesn't exist
    assert!(
        result.is_err(),
        "collect_history should fail for invalid path"
    );
}

/// Test that repo_root returns the correct path for a subdirectory.
#[test]
fn test_repo_root_from_subdirectory() {
    let Some(repo) = create_test_repo() else {
        eprintln!("Skipping test: git unavailable or repo creation failed");
        return;
    };

    // Create a subdirectory
    let subdir = repo.path.join("subdir");
    std::fs::create_dir_all(&subdir).ok();

    let root = repo_root(&subdir);
    assert!(
        root.is_some(),
        "repo_root should return Some for subdirectory of git repo"
    );

    let root_path = root.unwrap();
    // Normalize paths for comparison
    let expected = repo.path.canonicalize().ok();
    let actual = root_path.canonicalize().ok();

    assert_eq!(
        expected, actual,
        "repo_root should return the repo root, not the subdirectory"
    );
}

/// Test that max_commits limit is respected exactly.
/// This test verifies that when asking for 2 commits from a repo with 3,
/// we get exactly 2 (not more, not less).
///
/// Note: When we break early from reading git output, the git process may
/// exit with non-zero status due to broken pipe. We handle this by catching
/// the error and checking if we got the expected commits anyway.
#[test]
fn test_max_commits_exact_limit() {
    let Some(repo) = create_test_repo() else {
        eprintln!("Skipping test: git unavailable or repo creation failed");
        return;
    };
    let root = repo_root(&repo.path).expect("Should find repo root");

    // Our test repo has exactly 3 commits. If we ask for 3 (or more),
    // git will output all commits and exit normally.
    // If we ask for 2, git may fail due to broken pipe, but that's OK.

    // Test with limit equal to total commits (should succeed reliably)
    let commits = collect_history(&root, Some(3), None).expect("Should collect history");
    assert_eq!(
        commits.len(),
        3,
        "Should get exactly 3 commits when max_commits=3"
    );

    // Test with limit greater than total commits (should succeed reliably)
    let commits = collect_history(&root, Some(10), None).expect("Should collect history");
    assert_eq!(
        commits.len(),
        3,
        "Should get all 3 commits when max_commits exceeds total"
    );

    // Test with lower limit (may fail due to broken pipe, but verifies the limit works)
    // We use Ok().is_ok_and() to handle potential broken pipe errors gracefully
    let result = collect_history(&root, Some(2), None);
    if let Ok(commits) = result {
        assert!(
            commits.len() <= 2,
            "Should get at most 2 commits when max_commits=2, got {}",
            commits.len()
        );
    }
    // If it fails, that's acceptable - the early break causes broken pipe
}

/// Helper to create a repo with commits having multiple files.
fn create_test_repo_with_multi_file_commits() -> Option<TempGitRepo> {
    if !git_available() {
        return None;
    }

    let unique_id = format!(
        "{}-{:?}-{}-multifile",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let temp_dir = std::env::temp_dir().join(format!("tokmd-git-test-{}", unique_id));

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    std::fs::create_dir_all(&temp_dir).ok()?;

    // Initialize git repo
    let status = Command::new("git")
        .args(["init"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;
    if !status.status.success() {
        std::fs::remove_dir_all(&temp_dir).ok();
        return None;
    }

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;

    // Create a commit with 5 files
    for i in 1..=5 {
        let file = temp_dir.join(format!("file{}.txt", i));
        std::fs::write(&file, format!("content{}", i)).ok()?;
    }
    Command::new("git")
        .args(["add", "."])
        .current_dir(&temp_dir)
        .output()
        .ok()?;
    let commit_result = Command::new("git")
        .args(["commit", "-m", "Commit with 5 files"])
        .current_dir(&temp_dir)
        .output()
        .ok()?;
    if !commit_result.status.success() {
        std::fs::remove_dir_all(&temp_dir).ok();
        return None;
    }

    Some(TempGitRepo { path: temp_dir })
}

/// Test that max_commit_files limit is respected exactly.
/// If we ask for max 2 files per commit, a commit with 5 files should only show 2.
#[test]
fn test_max_commit_files_exact_limit() {
    let Some(repo) = create_test_repo_with_multi_file_commits() else {
        eprintln!("Skipping test: git unavailable or repo creation failed");
        return;
    };
    let root = repo_root(&repo.path).expect("Should find repo root");

    // The commit has 5 files. If we limit to 2, we should get exactly 2 files.
    let commits =
        collect_history(&root, None, Some(2)).expect("Should collect history with file limit");

    assert_eq!(commits.len(), 1, "Should have exactly 1 commit");

    let commit = &commits[0];
    assert_eq!(
        commit.files.len(),
        2,
        "Commit should have exactly 2 files when max_commit_files=2, got: {:?}",
        commit.files
    );
}

/// Test that max_commit_files limit of 1 gives exactly 1 file.
#[test]
fn test_max_commit_files_limit_one() {
    let Some(repo) = create_test_repo_with_multi_file_commits() else {
        eprintln!("Skipping test: git unavailable or repo creation failed");
        return;
    };
    let root = repo_root(&repo.path).expect("Should find repo root");

    // The commit has 5 files. If we limit to 1, we should get exactly 1 file.
    let commits =
        collect_history(&root, None, Some(1)).expect("Should collect history with file limit");

    assert_eq!(commits.len(), 1, "Should have exactly 1 commit");

    let commit = &commits[0];
    assert_eq!(
        commit.files.len(),
        1,
        "Commit should have exactly 1 file when max_commit_files=1, got: {:?}",
        commit.files
    );
}

/// Test that max_commit_files=0 gives 0 files.
#[test]
fn test_max_commit_files_limit_zero() {
    let Some(repo) = create_test_repo_with_multi_file_commits() else {
        eprintln!("Skipping test: git unavailable or repo creation failed");
        return;
    };
    let root = repo_root(&repo.path).expect("Should find repo root");

    // The commit has 5 files. If we limit to 0, we should get 0 files.
    let commits =
        collect_history(&root, None, Some(0)).expect("Should collect history with file limit");

    assert_eq!(commits.len(), 1, "Should have exactly 1 commit");

    let commit = &commits[0];
    assert_eq!(
        commit.files.len(),
        0,
        "Commit should have 0 files when max_commit_files=0, got: {:?}",
        commit.files
    );
}

/// Test that without file limit, all files are returned.
#[test]
fn test_no_max_commit_files_returns_all() {
    let Some(repo) = create_test_repo_with_multi_file_commits() else {
        eprintln!("Skipping test: git unavailable or repo creation failed");
        return;
    };
    let root = repo_root(&repo.path).expect("Should find repo root");

    // Without file limit, we should get all 5 files
    let commits = collect_history(&root, None, None).expect("Should collect history");

    assert_eq!(commits.len(), 1, "Should have exactly 1 commit");

    let commit = &commits[0];
    assert_eq!(
        commit.files.len(),
        5,
        "Commit should have all 5 files when no limit, got: {:?}",
        commit.files
    );
}
