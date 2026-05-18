#![cfg(feature = "cockpit")]

use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use tokmd_core::{cockpit_workflow, settings::CockpitSettings};
use tokmd_types::cockpit::GateStatus;

fn cwd_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct CurrentDirGuard {
    original: PathBuf,
}

impl CurrentDirGuard {
    fn enter(path: &Path) -> Self {
        let original = env::current_dir().expect("current dir");
        env::set_current_dir(path).expect("set test current dir");
        Self { original }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

fn git(repo: &Path, args: &[&str]) {
    let status = tokmd_git::git_cmd()
        .arg("-C")
        .arg(repo)
        .args(args)
        .status()
        .expect("run git");
    assert!(status.success(), "git {:?} failed", args);
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write fixture file");
}

#[test]
fn cockpit_workflow_computes_receipt_from_settings() {
    if !tokmd_git::git_available() {
        eprintln!("skipping cockpit workflow contract test because git is unavailable");
        return;
    }

    let repo = tempfile::tempdir().expect("temp repo");
    git(repo.path(), &["init", "-b", "main"]);
    git(repo.path(), &["config", "user.email", "tokmd@example.com"]);
    git(repo.path(), &["config", "user.name", "tokmd"]);
    // Avoid host commit-signing config bleeding into this fixture repo.
    git(repo.path(), &["config", "commit.gpgsign", "false"]);
    git(repo.path(), &["config", "tag.gpgsign", "false"]);

    write(&repo.path().join("README.md"), "# Demo\n");
    git(repo.path(), &["add", "README.md"]);
    git(repo.path(), &["commit", "-m", "base"]);

    write(
        &repo.path().join("README.md"),
        "# Demo\n\nThis change gives cockpit a reviewable file.\n",
    );
    git(repo.path(), &["add", "README.md"]);
    git(repo.path(), &["commit", "-m", "head"]);

    let _lock = cwd_lock().lock().expect("cwd lock");
    let _cwd = CurrentDirGuard::enter(repo.path());
    let receipt = cockpit_workflow(&CockpitSettings {
        base: "HEAD~1".to_string(),
        head: "HEAD".to_string(),
        range_mode: "2dot".to_string(),
        baseline: None,
    })
    .expect("cockpit workflow should compute from settings");

    assert_eq!(receipt.mode, "cockpit");
    assert_eq!(receipt.base_ref, "HEAD~1");
    assert_eq!(receipt.head_ref, "HEAD");
    assert_eq!(receipt.change_surface.files_changed, 1);
    assert_eq!(receipt.review_plan.len(), 1);
    assert_eq!(receipt.review_plan[0].path, "README.md");
    assert_eq!(receipt.evidence.mutation.meta.status, GateStatus::Skipped);
    assert!(receipt.evidence.mutation.meta.scope.relevant.is_empty());
}
