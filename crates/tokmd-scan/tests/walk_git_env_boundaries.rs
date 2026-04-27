use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;
use tokmd_scan::walk::list_files;

fn git_in(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .current_dir(dir);
    cmd
}

struct GitEnvGuard {
    git_dir: Option<OsString>,
    git_work_tree: Option<OsString>,
}

impl Drop for GitEnvGuard {
    fn drop(&mut self) {
        match &self.git_dir {
            Some(value) => unsafe { std::env::set_var("GIT_DIR", value) },
            None => unsafe { std::env::remove_var("GIT_DIR") },
        }
        match &self.git_work_tree {
            Some(value) => unsafe { std::env::set_var("GIT_WORK_TREE", value) },
            None => unsafe { std::env::remove_var("GIT_WORK_TREE") },
        }
    }
}

fn poison_git_env(dir: &TempDir) -> GitEnvGuard {
    let guard = GitEnvGuard {
        git_dir: std::env::var_os("GIT_DIR"),
        git_work_tree: std::env::var_os("GIT_WORK_TREE"),
    };
    unsafe {
        std::env::set_var("GIT_DIR", dir.path().join("bogus-git-dir"));
        std::env::set_var("GIT_WORK_TREE", dir.path().join("bogus-work-tree"));
    }
    guard
}

fn init_git_repo(dir: &Path) {
    let status = git_in(dir).arg("init").status().unwrap();
    assert!(status.success(), "git init failed");
}

fn git_add(dir: &Path, path: &str) {
    let status = git_in(dir).args(["add", path]).status().unwrap();
    assert!(status.success(), "git add {path} failed");
}

#[test]
fn list_files_ignores_inherited_git_env_overrides() {
    let repo = tempfile::tempdir().unwrap();
    let poison = tempfile::tempdir().unwrap();

    init_git_repo(repo.path());
    let status = git_in(repo.path())
        .args(["config", "user.email", "tokmd@example.com"])
        .status()
        .unwrap();
    assert!(status.success(), "git config user.email failed");
    let status = git_in(repo.path())
        .args(["config", "user.name", "tokmd"])
        .status()
        .unwrap();
    assert!(status.success(), "git config user.name failed");

    std::fs::write(repo.path().join("tracked.txt"), "tracked\n").unwrap();
    git_add(repo.path(), "tracked.txt");
    let status = git_in(repo.path())
        .args(["commit", "-m", "tracked"])
        .status()
        .unwrap();
    assert!(status.success(), "git commit failed");

    std::fs::write(repo.path().join("untracked.txt"), "untracked\n").unwrap();

    let _guard = poison_git_env(&poison);
    let files = list_files(repo.path(), None).unwrap();

    assert_eq!(files, vec![std::path::PathBuf::from("tracked.txt")]);
}

#[test]
fn list_files_git_fast_path_returns_root_relative_tracked_paths() {
    let repo = tempfile::tempdir().unwrap();
    init_git_repo(repo.path());

    std::fs::create_dir_all(repo.path().join("src")).unwrap();
    std::fs::write(repo.path().join("root.txt"), "root\n").unwrap();
    std::fs::write(repo.path().join("src/lib.rs"), "pub fn lib() {}\n").unwrap();
    std::fs::write(repo.path().join("untracked.txt"), "untracked\n").unwrap();
    git_add(repo.path(), "root.txt");
    git_add(repo.path(), "src/lib.rs");

    let files = list_files(repo.path(), None).unwrap();

    assert_eq!(
        files,
        vec![PathBuf::from("root.txt"), PathBuf::from("src/lib.rs")]
    );
    assert!(files.iter().all(|path| path.is_relative()));
}

#[test]
fn list_files_git_fast_path_skips_tracked_files_missing_from_worktree() {
    let repo = tempfile::tempdir().unwrap();
    init_git_repo(repo.path());

    std::fs::write(repo.path().join("keep.txt"), "keep\n").unwrap();
    std::fs::write(repo.path().join("gone.txt"), "gone\n").unwrap();
    git_add(repo.path(), "keep.txt");
    git_add(repo.path(), "gone.txt");
    std::fs::remove_file(repo.path().join("gone.txt")).unwrap();

    let files = list_files(repo.path(), None).unwrap();

    assert_eq!(files, vec![PathBuf::from("keep.txt")]);
}

#[test]
fn list_files_git_fast_path_rejects_tracked_symlink_escape_when_supported() {
    let repo = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    init_git_repo(repo.path());

    let outside_file = outside.path().join("secret.txt");
    let link = repo.path().join("secret-link.txt");
    std::fs::write(&outside_file, "secret\n").unwrap();
    if create_file_symlink(&outside_file, &link).is_err() {
        return;
    }
    git_add(repo.path(), "secret-link.txt");

    let err = list_files(repo.path(), None).unwrap_err();
    let message = err.to_string();

    assert!(
        message.contains("escapes scan root"),
        "expected symlink escape rejection, got: {message}"
    );
}

#[test]
fn list_files_git_fast_path_errors_on_tracked_dangling_symlink_when_supported() {
    let repo = tempfile::tempdir().unwrap();
    init_git_repo(repo.path());

    let missing_target = repo.path().join("missing-target.txt");
    let link = repo.path().join("broken-link.txt");
    if create_file_symlink(&missing_target, &link).is_err() {
        return;
    }
    git_add(repo.path(), "broken-link.txt");

    let err = list_files(repo.path(), None).unwrap_err();
    let message = err.to_string();

    assert!(
        message.contains("Failed to resolve bounded path"),
        "expected dangling symlink to fail closed, got: {message}"
    );
}

#[cfg(unix)]
fn create_file_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(windows)]
fn create_file_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(src, dst)
}
