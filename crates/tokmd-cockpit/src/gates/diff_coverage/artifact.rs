//! Coverage artifact discovery.
//!
//! Locates a coverage report file on disk by checking the common conventional
//! locations. Today only LCOV (`lcov.info`) is recognized; other formats are
//! discovered but not yet parsed, so they are rejected here to keep the
//! contract simple ("returned path is parseable").

use std::path::{Path, PathBuf};

#[cfg(feature = "git")]
const SEARCH_PATHS: &[&str] = &[
    "coverage/lcov.info",
    "target/coverage/lcov.info",
    "lcov.info",
    "coverage/cobertura.xml",
    "target/coverage/cobertura.xml",
    "cobertura.xml",
    "coverage/coverage.json",
    "target/coverage/coverage.json",
    "coverage.json",
];

/// Locate an LCOV coverage artifact under `repo_root`, if one exists.
///
/// Returns `Some(path)` only when a `lcov.info` file is found. Non-LCOV
/// artifacts (cobertura.xml, coverage.json) cause the search to stop with
/// `None` because they cannot yet be parsed, matching the prior behaviour.
#[cfg(feature = "git")]
pub(super) fn find_lcov_artifact(repo_root: &Path) -> Option<PathBuf> {
    for candidate in SEARCH_PATHS {
        let path = repo_root.join(candidate);
        if path.exists() {
            if path.to_string_lossy().ends_with("lcov.info") {
                return Some(path);
            }
            return None;
        }
    }
    None
}

#[cfg(all(test, feature = "git"))]
mod tests {
    use super::*;

    #[test]
    fn returns_none_when_no_artifact_exists() {
        let dir = tempfile::tempdir().unwrap();
        assert!(find_lcov_artifact(dir.path()).is_none());
    }

    #[test]
    fn finds_lcov_at_repo_root() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("lcov.info"), "SF:src/lib.rs\n").unwrap();
        let found = find_lcov_artifact(dir.path()).expect("should find lcov.info");
        assert!(found.ends_with("lcov.info"));
    }

    #[test]
    fn finds_lcov_under_coverage_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("coverage")).unwrap();
        std::fs::write(dir.path().join("coverage/lcov.info"), "SF:src/lib.rs\n").unwrap();
        let found = find_lcov_artifact(dir.path()).expect("should find coverage/lcov.info");
        assert!(found.ends_with("lcov.info"));
    }

    #[test]
    fn rejects_non_lcov_artifact() {
        // A cobertura.xml shows up first in the search list at coverage/cobertura.xml
        // before coverage.json, so a present cobertura file should short-circuit
        // to None (we cannot parse it yet) rather than scan further.
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("coverage")).unwrap();
        std::fs::write(dir.path().join("coverage/cobertura.xml"), "<x/>").unwrap();
        assert!(find_lcov_artifact(dir.path()).is_none());
    }
}
