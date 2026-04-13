use std::process::Command;

/// Create a `git` command without inheriting repo-shaping overrides.
pub(crate) fn git_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR").env_remove("GIT_WORK_TREE");
    cmd
}
