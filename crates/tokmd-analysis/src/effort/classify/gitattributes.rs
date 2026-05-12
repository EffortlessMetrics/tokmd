//! `.gitattributes` effort-classification rules.
//!
//! This module owns parsing `linguist-generated` and `linguist-vendored`
//! attributes plus the path-pattern matching needed to apply those rules.

use std::fs;
use std::path::{Path, PathBuf};

use super::ClassKind;

#[derive(Debug)]
pub(in crate::effort) struct GitAttrRule {
    pub(in crate::effort) kind: ClassKind,
    pub(in crate::effort) pattern: String,
    #[allow(dead_code)]
    pub(in crate::effort) source: String,
}

pub(in crate::effort) fn load_gitattributes(root: &Path) -> Vec<GitAttrRule> {
    if !has_host_root(root) {
        return Vec::new();
    }

    let path = root.join(".gitattributes");
    let file = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => return Vec::new(),
    };

    let mut rules = Vec::new();
    for raw in file.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace().map(str::trim).collect::<Vec<_>>();
        if parts.len() < 2 {
            continue;
        }

        let pattern = parts.remove(0).to_string();
        let flags = parts.join(" ");

        let kind = if flags.contains("linguist-generated") {
            ClassKind::Generated
        } else if flags.contains("linguist-vendored") {
            ClassKind::Vendored
        } else {
            ClassKind::Unknown
        };

        if !matches!(kind, ClassKind::Unknown) {
            rules.push(GitAttrRule {
                kind,
                pattern,
                source: raw.to_string(),
            });
        }
    }

    rules
}

pub(super) fn matches_path_pattern(path: &str, root: &Path, pattern: &str) -> bool {
    let path_lower = path.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    if pattern_lower.is_empty() {
        return false;
    }

    if pattern_lower.starts_with("*") {
        let suffix = pattern_lower.trim_start_matches('*');
        if suffix.is_empty() {
            return false;
        }
        return path_lower.ends_with(suffix);
    }

    if pattern_lower.ends_with("/") {
        return path_lower.starts_with(&pattern_lower);
    }

    if path_lower.contains(&pattern_lower) {
        return true;
    }

    if !has_host_root(root) {
        return false;
    }

    let full = root.join(PathBuf::from(path));
    full.ends_with(&pattern_lower)
}

fn has_host_root(root: &Path) -> bool {
    !root.as_os_str().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn load_gitattributes_keeps_only_linguist_classification_rules() {
        let dir = tempfile::tempdir().unwrap();
        let mut attrs = File::create(dir.path().join(".gitattributes")).unwrap();
        writeln!(attrs, "# ignored").unwrap();
        writeln!(attrs, "src/generated.rs linguist-generated").unwrap();
        writeln!(attrs, "vendor/** linguist-vendored").unwrap();
        writeln!(attrs, "docs/** text").unwrap();

        let rules = load_gitattributes(dir.path());

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].kind, ClassKind::Generated);
        assert_eq!(rules[0].pattern, "src/generated.rs");
        assert_eq!(rules[1].kind, ClassKind::Vendored);
        assert_eq!(rules[1].pattern, "vendor/**");
    }

    #[test]
    fn matches_path_pattern_handles_suffix_directory_and_contains_patterns() {
        assert!(matches_path_pattern(
            "src/generated/file.rs",
            Path::new(""),
            "src/generated/"
        ));
        assert!(matches_path_pattern(
            "src/bundle.min.js",
            Path::new(""),
            "*.min.js"
        ));
        assert!(matches_path_pattern(
            "third_party/vendor/lib.rs",
            Path::new(""),
            "vendor"
        ));
        assert!(!matches_path_pattern("src/main.rs", Path::new(""), ""));
        assert!(!matches_path_pattern("src/main.rs", Path::new(""), "*"));
    }
}
