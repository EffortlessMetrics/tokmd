//! W59 – Property-based tests for tokmd-exclude.

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
    /// Once a pattern is added then re-evaluated through `has_exclude_pattern`,
    /// the result remains `true` regardless of the slash style used.
    #[test]
    fn excluded_stays_excluded_after_re_evaluation(path in rel_path()) {
        let mut patterns = Vec::new();
        add_exclude_pattern(&mut patterns, path.clone());

        // Re-evaluate with forward slashes.
        prop_assert!(has_exclude_pattern(&patterns, &path));

        // Re-evaluate with backslash variant.
        let bs = path.replace('/', "\\");
        prop_assert!(has_exclude_pattern(&patterns, &bs));

        // Re-evaluate with ./ prefix.
        let prefixed = format!("./{path}");
        prop_assert!(has_exclude_pattern(&patterns, &prefixed));
    }

    /// `normalize_exclude_pattern` never produces a string with backslashes.
    #[test]
    fn normalize_never_produces_backslashes(path in rel_path()) {
        let root = PathBuf::from("root");
        let got = normalize_exclude_pattern(&root, PathBuf::from(&path).as_path());
        prop_assert!(!got.contains('\\'), "backslash found in: {got}");
    }

    /// `normalize_exclude_pattern` is idempotent on relative paths.
    #[test]
    fn normalize_idempotent(path in rel_path()) {
        let root = PathBuf::from("root");
        let first = normalize_exclude_pattern(&root, PathBuf::from(&path).as_path());
        let second = normalize_exclude_pattern(&root, PathBuf::from(&first).as_path());
        prop_assert_eq!(first, second);
    }

    /// Adding a path then its `./-` prefixed form never grows the vec beyond 1.
    #[test]
    fn add_dot_slash_prefix_never_duplicates(path in rel_path()) {
        let mut patterns = Vec::new();
        add_exclude_pattern(&mut patterns, path.clone());
        let dot_prefixed = format!("./{path}");
        let inserted = add_exclude_pattern(&mut patterns, dot_prefixed);
        prop_assert!(!inserted);
        prop_assert_eq!(patterns.len(), 1);
    }

    /// Adding a path then its backslash form never grows the vec beyond 1.
    #[test]
    fn add_backslash_variant_never_duplicates(path in rel_path()) {
        let mut patterns = Vec::new();
        add_exclude_pattern(&mut patterns, path.clone());
        let bs = path.replace('/', "\\");
        let inserted = add_exclude_pattern(&mut patterns, bs);
        prop_assert!(!inserted);
        prop_assert_eq!(patterns.len(), 1);
    }

    /// Two distinct paths yield two entries.
    #[test]
    fn two_distinct_paths_yield_two_entries(
        a in rel_path(),
        b in rel_path(),
    ) {
        prop_assume!(a != b);
        let mut patterns = Vec::new();
        add_exclude_pattern(&mut patterns, a);
        add_exclude_pattern(&mut patterns, b);
        prop_assert_eq!(patterns.len(), 2);
    }

    /// The result of `normalize_exclude_pattern` never starts with `./`.
    #[test]
    fn result_never_starts_with_dot_slash(path in rel_path()) {
        let root = PathBuf::from("repo");
        let got = normalize_exclude_pattern(&root, PathBuf::from(&path).as_path());
        prop_assert!(!got.starts_with("./"), "starts with ./: {got}");
    }

    /// Normalization preserves segment count for relative paths.
    #[test]
    fn segment_count_preserved(segments in prop::collection::vec(segment(), 1..6)) {
        let root = PathBuf::from("repo");
        let path_str = segments.join("/");
        let got = normalize_exclude_pattern(&root, PathBuf::from(&path_str).as_path());
        let got_segs: Vec<&str> = got.split('/').collect();
        prop_assert_eq!(got_segs.len(), segments.len());
    }
}
