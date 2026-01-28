//! Property-based tests for tokmd-model functions.

use proptest::prelude::*;
use std::path::Path;
use tokmd_model::{avg, module_key, normalize_path};

proptest! {
    // ========================
    // Average Function Properties
    // ========================

    #[test]
    fn avg_zero_files_is_zero(lines in 0usize..10000) {
        prop_assert_eq!(avg(lines, 0), 0);
    }

    #[test]
    fn avg_zero_lines_is_zero(files in 1usize..10000) {
        prop_assert_eq!(avg(0, files), 0);
    }

    #[test]
    fn avg_same_value(value in 1usize..10000) {
        // lines == files should give approximately 1
        prop_assert_eq!(avg(value, value), 1);
    }

    #[test]
    fn avg_double(value in 1usize..5000) {
        // 2*value lines, value files should give 2
        prop_assert_eq!(avg(2 * value, value), 2);
    }

    #[test]
    fn avg_rounds_correctly(lines in 0usize..10000, files in 1usize..1000) {
        let result = avg(lines, files);
        let expected = (lines + (files / 2)) / files;
        prop_assert_eq!(result, expected, "Rounding mismatch");
    }

    #[test]
    fn avg_bounded(lines in 0usize..10000, files in 1usize..1000) {
        let result = avg(lines, files);
        // Result should be roughly lines/files, within rounding
        let lower = lines / files;
        let upper = if lines % files == 0 { lower } else { lower + 1 };
        prop_assert!(result >= lower && result <= upper,
            "avg({}, {}) = {} should be in [{}, {}]", lines, files, result, lower, upper);
    }

    // ========================
    // Path Normalization Properties
    // ========================

    #[test]
    fn normalize_path_never_crashes(s in "\\PC*") {
        let p = Path::new(&s);
        let _ = normalize_path(p, None);
    }

    #[test]
    fn normalize_path_always_forward_slash(s in "\\PC*") {
        let p = Path::new(&s);
        let normalized = normalize_path(p, None);
        prop_assert!(!normalized.contains('\\'), "Should not contain backslash: {}", normalized);
    }

    #[test]
    fn normalize_path_no_leading_dot_slash(s in "\\PC*") {
        let p = Path::new(&s);
        let normalized = normalize_path(p, None);
        prop_assert!(!normalized.starts_with("./"), "Should not start with ./: {}", normalized);
    }

    #[test]
    fn normalize_path_no_leading_slash(s in "\\PC*") {
        let p = Path::new(&s);
        let normalized = normalize_path(p, None);
        // After normalization, should not start with /
        prop_assert!(!normalized.starts_with('/'), "Should not start with /: {}", normalized);
    }

    #[test]
    fn normalize_path_idempotent(s in "[a-zA-Z0-9_/\\.]+") {
        let p = Path::new(&s);
        let once = normalize_path(p, None);
        let twice = normalize_path(Path::new(&once), None);
        prop_assert_eq!(once, twice, "Normalization should be idempotent");
    }

    #[test]
    fn normalize_path_prefix_stripping(
        prefix_parts in prop::collection::vec("[a-zA-Z0-9_]+", 1..3),
        suffix_parts in prop::collection::vec("[a-zA-Z0-9_]+", 1..3)
    ) {
        let prefix_path = prefix_parts.join("/");
        let suffix_path = suffix_parts.join("/");
        let full_path = format!("{}/{}", prefix_path, suffix_path);

        let prefix = Path::new(&prefix_path);
        let full = Path::new(&full_path);
        let normalized = normalize_path(full, Some(prefix));

        // The key property is that after stripping the prefix, we get exactly the suffix.
        // Note: We don't check !normalized.starts_with(&prefix_path) because when
        // prefix and suffix contain the same segments (e.g., prefix="_", suffix="_"),
        // the result legitimately starts with the same characters as the prefix.
        prop_assert_eq!(&normalized, &suffix_path,
            "After stripping '{}' from '{}', expected '{}', got '{}'",
            prefix_path, full_path, suffix_path, normalized);
    }

    // ========================
    // Module Key Properties
    // ========================

    #[test]
    fn module_key_never_crashes(
        path in "\\PC*",
        ref roots in prop::collection::vec("\\PC*", 0..5),
        depth in 0usize..10
    ) {
        let _ = module_key(&path, roots, depth);
    }

    #[test]
    fn module_key_root_file_is_root(filename in "[a-zA-Z0-9_]+\\.[a-z]+") {
        // Single filename (no directory) should always be (root)
        let key = module_key(&filename, &[], 2);
        prop_assert_eq!(key, "(root)", "Single file '{}' should be (root)", filename);
    }

    #[test]
    fn module_key_non_matching_root_is_first_dir(
        dir in "[a-zA-Z0-9_]+",
        subdirs in prop::collection::vec("[a-zA-Z0-9_]+", 1..3),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+"
    ) {
        // When first dir is not in roots, module key is just the first dir
        let path_parts: Vec<&str> = std::iter::once(dir.as_str())
            .chain(subdirs.iter().map(|s| s.as_str()))
            .chain(std::iter::once(filename.as_str()))
            .collect();
        let path = path_parts.join("/");

        // Use roots that don't match the dir
        let roots = vec!["nonexistent_root".to_string()];
        let key = module_key(&path, &roots, 3);
        prop_assert_eq!(&key, &dir, "Non-matching root should return first dir: path='{}', key='{}'", path, key);
    }

    #[test]
    fn module_key_matching_root_depth(
        root in "[a-zA-Z0-9_]+",
        subdirs in prop::collection::vec("[a-zA-Z0-9_]+", 2..5),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+",
        depth in 1usize..4
    ) {
        let path_parts: Vec<&str> = std::iter::once(root.as_str())
            .chain(subdirs.iter().map(|s| s.as_str()))
            .chain(std::iter::once(filename.as_str()))
            .collect();
        let path = path_parts.join("/");

        let roots = vec![root.clone()];
        let key = module_key(&path, &roots, depth);

        // Key should be at most `depth` directory segments
        let key_depth = key.split('/').count();
        let max_dirs = subdirs.len() + 1; // root + subdirs
        let expected_depth = depth.min(max_dirs);
        prop_assert_eq!(key_depth, expected_depth,
            "Key '{}' should have depth {}, has {} (path='{}', depth={})",
            key, expected_depth, key_depth, path, depth);
    }

    #[test]
    fn module_key_deterministic(
        path in "[a-zA-Z0-9_/]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 1usize..5
    ) {
        let key1 = module_key(&path, roots, depth);
        let key2 = module_key(&path, roots, depth);
        prop_assert_eq!(key1, key2, "Module key should be deterministic");
    }

    #[test]
    fn module_key_normalized_input(
        parts in prop::collection::vec("[a-zA-Z0-9_]+", 2..4),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+"
    ) {
        let forward_path = format!("{}/{}", parts.join("/"), filename);
        let back_path = format!("{}\\{}", parts.join("\\"), filename);
        let dotslash_path = format!("./{}/{}", parts.join("/"), filename);

        let roots: Vec<String> = vec![];
        let k_forward = module_key(&forward_path, &roots, 2);
        let k_back = module_key(&back_path, &roots, 2);
        let k_dot = module_key(&dotslash_path, &roots, 2);

        prop_assert_eq!(&k_forward, &k_back, "Backslash path should normalize: '{}' vs '{}'", forward_path, back_path);
        prop_assert_eq!(&k_forward, &k_dot, "Dotslash path should normalize: '{}' vs '{}'", forward_path, dotslash_path);
    }

    #[test]
    fn module_key_no_backslash(
        path in "[a-zA-Z0-9_/\\\\]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 1usize..5
    ) {
        let key = module_key(&path, roots, depth);
        prop_assert!(!key.contains('\\'), "Module key should not contain backslash: {}", key);
    }
}

// Note: fold_other_* property tests are in lib.rs where they can access
// the private functions directly instead of reimplementing them.
