//! Git command construction with process-environment isolation.
//!
//! This module owns the subprocess boundary for git invocations. Callers should
//! build git commands here so ambient repository and helper environment
//! variables cannot bypass an explicit `-C` path or launch unexpected helpers.

use std::process::Command;

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

/// Create a `Command` for git with process-environment isolation.
///
/// Strips repo, worktree, index, object-store, discovery, and execution helper
/// overrides so inherited environment variables cannot bypass the explicit
/// `-C` path used by functions in this crate or launch ambient helper programs.
pub fn git_cmd() -> Command {
    let mut cmd = Command::new("git");
    for name in GIT_REPO_SHAPING_ENV {
        cmd.env_remove(name);
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

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
}
