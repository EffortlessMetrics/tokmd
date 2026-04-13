#![cfg(feature = "git")]

use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;
use tokmd_cockpit::get_file_stats;
use tokmd_git::GitRangeMode;

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

#[test]
fn get_file_stats_ignores_inherited_git_env_overrides() {
    let repo = tempfile::tempdir().unwrap();
    let poison = tempfile::tempdir().unwrap();

    let status = git_in(repo.path())
        .args(["init", "-b", "main"])
        .status()
        .unwrap();
    assert!(status.success(), "git init failed");
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

    std::fs::write(repo.path().join("tracked.txt"), "one\n").unwrap();
    let status = git_in(repo.path()).args(["add", "."]).status().unwrap();
    assert!(status.success(), "git add base failed");
    let status = git_in(repo.path())
        .args(["commit", "-m", "base"])
        .status()
        .unwrap();
    assert!(status.success(), "git commit base failed");

    std::fs::write(repo.path().join("tracked.txt"), "one\ntwo\n").unwrap();
    let status = git_in(repo.path()).args(["add", "."]).status().unwrap();
    assert!(status.success(), "git add head failed");
    let status = git_in(repo.path())
        .args(["commit", "-m", "head"])
        .status()
        .unwrap();
    assert!(status.success(), "git commit head failed");

    let _guard = poison_git_env(&poison);
    let stats = get_file_stats(repo.path(), "HEAD~1", "HEAD", GitRangeMode::TwoDot).unwrap();

    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].path, "tracked.txt");
}
