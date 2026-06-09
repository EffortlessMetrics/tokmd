//! Tokei ignore-pattern expansion for canonicalized scan roots.
//!
//! Scan roots are validated and canonicalized before being passed to `tokei`.
//! Relative excludes therefore need canonical-root variants in addition to the
//! caller-provided and globbed forms so excludes keep matching after root
//! normalization.

use std::collections::BTreeSet;
use std::path::Path;

use tokmd_settings::ScanOptions;

use crate::path::{ValidatedRoot, normalize_slashes};

pub(crate) fn ignored_patterns(args: &ScanOptions, roots: &[ValidatedRoot]) -> Vec<String> {
    let mut patterns = BTreeSet::new();

    for pattern in &args.excluded {
        patterns.insert(pattern.clone());

        if is_absolute_pattern(pattern) {
            continue;
        }

        let relative = normalize_relative_ignore_pattern(pattern);
        if relative.is_empty() {
            continue;
        }

        if !relative.starts_with("**/") {
            patterns.insert(format!("**/{relative}"));
        }

        for root in roots {
            let lossy = root.canonical().to_string_lossy();
            let canonical = normalize_slashes(&lossy);
            patterns.insert(format!("{}/{}", canonical.trim_end_matches('/'), relative));
        }
    }

    patterns.into_iter().collect()
}

fn is_absolute_pattern(pattern: &str) -> bool {
    let path = Path::new(pattern);
    path.is_absolute()
        || pattern.starts_with('/')
        || pattern.starts_with('\\')
        || pattern.as_bytes().get(1).is_some_and(|byte| *byte == b':')
}

fn normalize_relative_ignore_pattern(pattern: &str) -> String {
    let mut normalized = normalize_slashes(pattern).into_owned();
    while let Some(rest) = normalized.strip_prefix("./") {
        normalized = rest.to_string();
    }
    normalized.trim_start_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_options(excluded: &[&str]) -> ScanOptions {
        ScanOptions {
            excluded: excluded.iter().map(|pattern| pattern.to_string()).collect(),
            ..ScanOptions::default()
        }
    }

    #[test]
    fn ignore_normalize_relative_pattern_strips_dot_and_slashes() {
        assert_eq!(
            normalize_relative_ignore_pattern("./generated/**"),
            "generated/**"
        );
        assert_eq!(
            normalize_relative_ignore_pattern(r".\generated\**"),
            "generated/**"
        );
        assert_eq!(normalize_relative_ignore_pattern("/target/**"), "target/**");
    }

    #[test]
    fn ignore_patterns_expand_relative_patterns_for_canonical_roots() {
        let dir = tempfile::tempdir().unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();
        let patterns = ignored_patterns(
            &scan_options(&["secret_folder/**"]),
            std::slice::from_ref(&root),
        );
        let lossy = root.canonical().to_string_lossy();
        let canonical = normalize_slashes(&lossy);

        assert!(patterns.contains(&"secret_folder/**".to_string()));
        assert!(patterns.contains(&"**/secret_folder/**".to_string()));
        assert!(patterns.contains(&format!("{canonical}/secret_folder/**")));
    }

    #[test]
    fn ignore_patterns_keep_absolute_patterns_as_given() {
        let dir = tempfile::tempdir().unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();
        let absolute = dir.path().join("target/**").to_string_lossy().to_string();
        let patterns = ignored_patterns(&scan_options(&[&absolute]), &[root]);

        assert_eq!(patterns, vec![absolute]);
    }
}
