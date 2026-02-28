use std::path::PathBuf;

use proptest::prelude::*;
use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

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

    // --- New property tests ---

    #[test]
    fn normalize_never_starts_with_dot_slash(path in rel_path()) {
        let root = PathBuf::from("repo");
        let normalized = normalize_exclude_pattern(&root, PathBuf::from(&path).as_path());
        prop_assert!(!normalized.starts_with("./"), "got: {}", normalized);
    }

    #[test]
    fn has_exclude_pattern_symmetric_with_backslash_variant(path in rel_path()) {
        let forward = path.clone();
        let backslash = path.replace('/', "\\");
        let existing = vec![forward.clone()];
        prop_assert!(has_exclude_pattern(&existing, &forward));
        prop_assert!(has_exclude_pattern(&existing, &backslash));
    }

    #[test]
    fn has_exclude_pattern_symmetric_with_dot_slash_prefix(path in rel_path()) {
        let plain = path.clone();
        let prefixed = format!("./{path}");
        let existing = vec![plain.clone()];
        prop_assert!(has_exclude_pattern(&existing, &plain));
        prop_assert!(has_exclude_pattern(&existing, &prefixed));
    }

    #[test]
    fn add_then_has_always_true(path in rel_path()) {
        let mut patterns = Vec::new();
        add_exclude_pattern(&mut patterns, path.clone());
        prop_assert!(has_exclude_pattern(&patterns, &path));
    }

    #[test]
    fn add_exclude_pattern_returns_false_on_second_insert(path in rel_path()) {
        let mut patterns = Vec::new();
        let first = add_exclude_pattern(&mut patterns, path.clone());
        let second = add_exclude_pattern(&mut patterns, path.clone());
        prop_assert!(first);
        prop_assert!(!second);
        prop_assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn normalize_preserves_path_segments(
        segments in prop::collection::vec(segment(), 1..6),
    ) {
        let root = PathBuf::from("repo");
        let path_str = segments.join("/");
        let normalized = normalize_exclude_pattern(&root, PathBuf::from(&path_str).as_path());
        let normalized_segments: Vec<&str> = normalized.split('/').collect();
        prop_assert_eq!(normalized_segments.len(), segments.len());
        for (got, expected) in normalized_segments.iter().zip(segments.iter()) {
            prop_assert_eq!(*got, expected.as_str());
        }
    }

    #[test]
    fn add_exclude_pattern_with_dot_slash_and_backslash_variant_dedupes(path in rel_path()) {
        let mut patterns = Vec::new();
        add_exclude_pattern(&mut patterns, path.clone());
        let backslash = path.replace('/', "\\");
        let dot_prefixed = format!("./{path}");
        let dot_backslash = format!(".\\{}", path.replace('/', "\\"));

        let b = add_exclude_pattern(&mut patterns, backslash);
        let c = add_exclude_pattern(&mut patterns, dot_prefixed);
        let d = add_exclude_pattern(&mut patterns, dot_backslash);

        prop_assert!(!b);
        prop_assert!(!c);
        prop_assert!(!d);
        prop_assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn empty_pattern_never_inserted(path in rel_path()) {
        let mut patterns = vec![path];
        let inserted = add_exclude_pattern(&mut patterns, String::new());
        prop_assert!(!inserted);
    }
}
