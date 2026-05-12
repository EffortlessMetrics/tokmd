//! Git-backed file listing helpers for repository walks.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::Result;

use crate::path::{BoundedPath, PathViolation, ValidatedRoot};

const GIT_REPO_SHAPING_ENV: &[&str] = &[
    // Repository and object-store overrides.
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_CEILING_DIRECTORIES",
    // Git hooks that can execute helper programs from ambient environment.
    "GIT_SSH",
    "GIT_SSH_COMMAND",
    "GIT_ASKPASS",
    "GIT_PAGER",
    "GIT_EDITOR",
    "GIT_PROXY_COMMAND",
    "GIT_EXTERNAL_DIFF",
];

fn git_cmd() -> Command {
    let mut cmd = Command::new("git");
    for name in GIT_REPO_SHAPING_ENV {
        cmd.env_remove(name);
    }
    cmd
}

pub(super) fn git_ls_files(root: &Path) -> Result<Option<Vec<PathBuf>>> {
    let output = git_cmd()
        .arg("-C")
        .arg(root)
        .arg("ls-files")
        .arg("-z")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let output = match output {
        Ok(out) => out,
        Err(_) => return Ok(None),
    };
    if !output.status.success() {
        return Ok(None);
    }

    let mut files = Vec::new();
    let bytes = output.stdout;
    for part in bytes.split(|b| *b == 0) {
        if part.is_empty() {
            continue;
        }
        let s = String::from_utf8_lossy(part).to_string();
        files.push(PathBuf::from(s));
    }

    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(files))
}

pub(super) fn bound_git_relative_path(
    root: &ValidatedRoot,
    path: &Path,
) -> Result<Option<PathBuf>, PathViolation> {
    match BoundedPath::existing_relative(root, path) {
        Ok(path) => Ok(Some(path.relative().to_path_buf())),
        Err(PathViolation::Missing(_)) => Ok(None),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::fs;

    #[test]
    fn git_cmd_removes_repo_shaping_env_overrides() {
        let removed: BTreeSet<_> = git_cmd()
            .get_envs()
            .filter(|(_, value)| value.is_none())
            .map(|(name, _)| name.to_string_lossy().into_owned())
            .collect();

        for name in GIT_REPO_SHAPING_ENV {
            assert!(removed.contains(*name), "missing env_remove for {name}");
        }
    }

    #[test]
    fn git_cmd_removes_execution_helper_env_overrides() {
        let removed: BTreeSet<_> = git_cmd()
            .get_envs()
            .filter(|(_, value)| value.is_none())
            .map(|(name, _)| name.to_string_lossy().into_owned())
            .collect();

        for name in [
            "GIT_SSH",
            "GIT_SSH_COMMAND",
            "GIT_ASKPASS",
            "GIT_PAGER",
            "GIT_EDITOR",
            "GIT_PROXY_COMMAND",
            "GIT_EXTERNAL_DIFF",
        ] {
            assert!(
                removed.contains(name),
                "missing execution env_remove for {name}"
            );
        }
    }

    #[test]
    fn test_bound_git_relative_path_accepts_existing_relative_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub fn lib() {}\n").unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();

        let bounded = bound_git_relative_path(&root, Path::new("./src/lib.rs"))
            .unwrap()
            .unwrap();

        assert_eq!(bounded, PathBuf::from("src/lib.rs"));
    }

    #[test]
    fn test_bound_git_relative_path_skips_missing_worktree_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();

        let bounded = bound_git_relative_path(&root, Path::new("missing.rs")).unwrap();

        assert!(bounded.is_none());
    }

    #[test]
    fn test_bound_git_relative_path_rejects_parent_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();

        let err = bound_git_relative_path(&root, Path::new("../secret.txt")).unwrap_err();

        assert!(err.to_string().contains("parent traversal"));
    }

    #[test]
    fn test_bound_git_relative_path_rejects_absolute_path() {
        let dir = tempfile::tempdir().unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();

        let err = bound_git_relative_path(&root, Path::new("/secret.txt")).unwrap_err();

        assert!(err.to_string().contains("must be relative"));
    }
}
