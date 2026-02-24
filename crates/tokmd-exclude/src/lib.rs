//! Deterministic exclude-pattern normalization and dedupe helpers.

#![forbid(unsafe_code)]

use std::path::Path;

use tokmd_path::normalize_rel_path;

/// Normalize an exclude path into a deterministic pattern.
///
/// Rules:
/// - if `path` is absolute and under `root`, strip the `root` prefix
/// - convert backslashes to `/`
/// - strip one leading `./`
#[must_use]
pub fn normalize_exclude_pattern(root: &Path, path: &Path) -> String {
    let rel = if path.is_absolute() {
        path.strip_prefix(root).unwrap_or(path)
    } else {
        path
    };
    normalize_rel_path(&rel.to_string_lossy())
}

/// Return `true` when `existing` already contains `pattern` after normalization.
#[must_use]
pub fn has_exclude_pattern(existing: &[String], pattern: &str) -> bool {
    let normalized = normalize_rel_path(pattern);
    existing
        .iter()
        .any(|candidate| normalize_rel_path(candidate) == normalized)
}

/// Add a pattern only when non-empty and not already present (after normalization).
///
/// Returns `true` when the pattern was inserted.
pub fn add_exclude_pattern(existing: &mut Vec<String>, pattern: String) -> bool {
    if pattern.is_empty() || has_exclude_pattern(existing, &pattern) {
        return false;
    }
    existing.push(pattern);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_exclude_pattern_strips_root_for_absolute_paths() {
        let root = std::env::temp_dir().join("tokmd-exclude-lib-root");
        let path = root.join(".handoff").join("manifest.json");
        let got = normalize_exclude_pattern(&root, &path);
        assert_eq!(got, ".handoff/manifest.json");
    }

    #[test]
    fn normalize_exclude_pattern_keeps_outside_absolute_paths() {
        let root = std::env::temp_dir().join("tokmd-exclude-lib-root");
        let outside = std::env::temp_dir()
            .join("tokmd-exclude-lib-outside")
            .join("bundle.txt");
        let got = normalize_exclude_pattern(&root, &outside);
        let expected = tokmd_path::normalize_rel_path(&outside.to_string_lossy());
        assert_eq!(got, expected);
    }

    #[test]
    fn add_exclude_pattern_dedupes_after_normalization() {
        let mut existing = vec!["./out\\bundle".to_string()];
        assert!(!add_exclude_pattern(
            &mut existing,
            "out/bundle".to_string()
        ));
        assert_eq!(existing.len(), 1);
    }

    #[test]
    fn add_exclude_pattern_rejects_empty_patterns() {
        let mut existing = vec![];
        assert!(!add_exclude_pattern(&mut existing, String::new()));
        assert!(existing.is_empty());
    }
}
