#[cfg(any(feature = "walk", feature = "content", feature = "git"))]
use std::path::Path;

#[cfg(any(feature = "walk", feature = "content", feature = "git"))]
pub(super) const ROOTLESS_FILE_ANALYSIS_WARNING: &str =
    "in-memory analysis has no host root; skipping file-backed enrichers";
#[cfg(feature = "git")]
pub(super) const ROOTLESS_GIT_ANALYSIS_WARNING: &str =
    "in-memory analysis has no host root; skipping git-backed enrichers";

#[cfg(any(feature = "walk", feature = "content", feature = "git"))]
pub(super) fn has_host_root(root: &Path) -> bool {
    !root.as_os_str().is_empty()
}

#[cfg(any(feature = "walk", feature = "content", feature = "git"))]
pub(super) fn push_warning_once(warnings: &mut Vec<String>, warning: &str) {
    if warnings.iter().all(|existing| existing != warning) {
        warnings.push(warning.to_string());
    }
}
