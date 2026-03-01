use std::path::PathBuf;

use proptest::prelude::*;
use tokmd_exclude::{add_exclude_pattern, normalize_exclude_pattern};

fn segment() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{1,8}".prop_map(String::from)
}

fn rel_path() -> impl Strategy<Value = String> {
    prop::collection::vec(segment(), 1..6).prop_map(|parts| parts.join("/"))
}

proptest! {
    #[test]
    fn normalize_exclude_pattern_never_contains_backslashes(path in "\\PC*") {
        let root = PathBuf::from("repo");
        let normalized = normalize_exclude_pattern(&root, PathBuf::from(path).as_path());
        prop_assert!(!normalized.contains('\\'));
    }

    #[test]
    fn normalize_exclude_pattern_is_deterministic(path in "\\PC*") {
        let root = PathBuf::from("repo");
        let p = PathBuf::from(path);
        let a = normalize_exclude_pattern(&root, &p);
        let b = normalize_exclude_pattern(&root, &p);
        prop_assert_eq!(a, b);
    }

    #[test]
    fn normalize_exclude_pattern_strips_root_prefix(
        root_parts in prop::collection::vec(segment(), 1..4),
        rel_parts in prop::collection::vec(segment(), 1..4),
    ) {
        let mut root = std::env::temp_dir().join("tokmd-exclude-prop-root");
        for part in &root_parts {
            root.push(part);
        }

        let mut full = root.clone();
        for part in &rel_parts {
            full.push(part);
        }

        let expected = rel_parts.join("/");
        let normalized = normalize_exclude_pattern(&root, &full);
        prop_assert_eq!(normalized, expected);
    }

    #[test]
    fn normalize_exclude_pattern_is_idempotent_for_relative_paths(path in rel_path()) {
        let root = PathBuf::from("repo");
        let first = normalize_exclude_pattern(&root, PathBuf::from(&path).as_path());
        let second = normalize_exclude_pattern(&root, PathBuf::from(&first).as_path());
        prop_assert_eq!(first, second);
    }

    #[test]
    fn add_exclude_pattern_is_set_like(incoming in rel_path()) {
        let mut patterns = Vec::new();
        let incoming_norm = normalize_exclude_pattern(PathBuf::from("repo").as_path(), PathBuf::from(&incoming).as_path());
        let _ = add_exclude_pattern(&mut patterns, incoming.clone());
        let _ = add_exclude_pattern(&mut patterns, format!("./{incoming}"));

        let count = patterns
            .iter()
            .filter(|p| normalize_exclude_pattern(PathBuf::from("repo").as_path(), PathBuf::from(p).as_path()) == incoming_norm)
            .count();
        prop_assert!(count <= 1);
    }
}
