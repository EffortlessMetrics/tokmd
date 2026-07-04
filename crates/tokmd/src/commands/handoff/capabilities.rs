//! Handoff capability detection.

use std::path::Path;

use tokmd_types::{CapabilityState, CapabilityStatus};

use crate::cli;
use crate::git_support::git_cmd;

/// Detect available capabilities for the handoff.
pub(super) fn detect_capabilities(root: &Path, args: &cli::HandoffArgs) -> Vec<CapabilityStatus> {
    let mut capabilities = Vec::new();

    // Check git availability
    let git_available = git_cmd()
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if args.no_git {
        capabilities.push(CapabilityStatus {
            name: "git".to_string(),
            status: CapabilityState::Skipped,
            reason: Some("disabled via --no-git flag".to_string()),
        });
    } else if !git_available {
        capabilities.push(CapabilityStatus {
            name: "git".to_string(),
            status: CapabilityState::Unavailable,
            reason: Some("git command not found".to_string()),
        });
    } else {
        capabilities.push(CapabilityStatus {
            name: "git".to_string(),
            status: CapabilityState::Available,
            reason: None,
        });
    }

    // Check if we're in a git repository
    #[cfg(feature = "git")]
    let in_repo = tokmd_git::repo_root(root).is_some();
    #[cfg(not(feature = "git"))]
    let in_repo = git_cmd()
        .args(["rev-parse", "--git-dir"])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if args.no_git {
        capabilities.push(CapabilityStatus {
            name: "git_repository".to_string(),
            status: CapabilityState::Skipped,
            reason: Some("disabled via --no-git flag".to_string()),
        });
    } else if !in_repo {
        capabilities.push(CapabilityStatus {
            name: "git_repository".to_string(),
            status: CapabilityState::Unavailable,
            reason: Some("not inside a git repository".to_string()),
        });
    } else {
        capabilities.push(CapabilityStatus {
            name: "git_repository".to_string(),
            status: CapabilityState::Available,
            reason: None,
        });
    }

    // Check for shallow clone
    let shallow = git_cmd()
        .args(["rev-parse", "--is-shallow-repository"])
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false);

    if args.no_git || !in_repo {
        capabilities.push(CapabilityStatus {
            name: "git_history".to_string(),
            status: CapabilityState::Skipped,
            reason: Some(if args.no_git {
                "disabled via --no-git flag".to_string()
            } else {
                "not in a git repository".to_string()
            }),
        });
    } else if shallow {
        capabilities.push(CapabilityStatus {
            name: "git_history".to_string(),
            status: CapabilityState::Unavailable,
            reason: Some("shallow clone detected; limited history available".to_string()),
        });
    } else {
        capabilities.push(CapabilityStatus {
            name: "git_history".to_string(),
            status: CapabilityState::Available,
            reason: None,
        });
    }

    capabilities
}

/// Check if we should compute git scores based on capabilities.
pub(super) fn capability_state(
    capabilities: &[CapabilityStatus],
    name: &str,
) -> Option<CapabilityState> {
    capabilities
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.status)
}

pub(super) fn capability_reason(capabilities: &[CapabilityStatus], name: &str) -> Option<String> {
    capabilities
        .iter()
        .find(|c| c.name == name)
        .and_then(|c| c.reason.clone())
}

pub(super) fn should_compute_git(capabilities: &[CapabilityStatus]) -> bool {
    capability_state(capabilities, "git_history") == Some(CapabilityState::Available)
}
