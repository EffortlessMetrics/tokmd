//! Property-based tests for tokmd CLI and integration.
//!
//! These tests verify invariants across the CLI's core functionality.

#[cfg(test)]
mod path_normalization {
    use proptest::prelude::*;
    use std::path::Path;
    use tokmd_model::normalize_path;

    proptest! {
        #[test]
        fn never_crashes(s in "\\PC*") {
            let p = Path::new(&s);
            let _ = normalize_path(p, None);
        }

        #[test]
        fn always_forward_slash(s in "\\PC*") {
            let p = Path::new(&s);
            let normalized = normalize_path(p, None);
            prop_assert!(!normalized.contains('\\'), "Contains backslash: {}", normalized);
        }

        #[test]
        fn strips_leading_dot_slash(s in "\\PC*") {
            let p = Path::new(&s);
            let normalized = normalize_path(p, None);
            prop_assert!(!normalized.starts_with("./"), "Starts with ./: {}", normalized);
        }

        #[test]
        fn no_leading_slash(s in "\\PC*") {
            let p = Path::new(&s);
            let normalized = normalize_path(p, None);
            prop_assert!(!normalized.starts_with('/'), "Starts with /: {}", normalized);
        }

        #[test]
        fn idempotent(s in "[a-zA-Z0-9_/\\.]+") {
            let p = Path::new(&s);
            let once = normalize_path(p, None);
            let twice = normalize_path(Path::new(&once), None);
            prop_assert_eq!(&once, &twice, "Not idempotent: '{}' -> '{}'", once, twice);
        }

        #[test]
        fn backslash_to_forward_preserves_segments(
            segments in prop::collection::vec("[a-zA-Z0-9_]+", 1..5)
        ) {
            let with_backslash = segments.join("\\");
            let with_forward = segments.join("/");

            let p_back = Path::new(&with_backslash);
            let p_fwd = Path::new(&with_forward);

            let norm_back = normalize_path(p_back, None);
            let norm_fwd = normalize_path(p_fwd, None);

            prop_assert_eq!(&norm_back, &norm_fwd,
                "Backslash and forward slash should normalize same: {} vs {}", norm_back, norm_fwd);
        }

        #[test]
        fn prefix_stripping_works(
            prefix_parts in prop::collection::vec("[a-zA-Z0-9_]+", 1..3),
            suffix_parts in prop::collection::vec("[a-zA-Z0-9_]+", 1..3)
        ) {
            let prefix = prefix_parts.join("/");
            let suffix = suffix_parts.join("/");
            let full = format!("{}/{}", prefix, suffix);

            let normalized = normalize_path(Path::new(&full), Some(Path::new(&prefix)));

            prop_assert_eq!(&normalized, &suffix,
                "Prefix '{}' not stripped from '{}', got '{}'", prefix, full, normalized);
        }
    }
}

#[cfg(test)]
mod module_key {
    use proptest::prelude::*;
    use tokmd_model::module_key;

    proptest! {
        #[test]
        fn never_crashes(
            path in "\\PC*",
            ref roots in prop::collection::vec("\\PC*", 0..5),
            depth in 0usize..10
        ) {
            let _ = module_key(&path, roots, depth);
        }

        #[test]
        fn root_file_is_root(filename in "[a-zA-Z0-9_]+\\.[a-z]+") {
            // A simple filename should always be (root)
            let key = module_key(&filename, &[], 2);
            prop_assert_eq!(key, "(root)", "File '{}' should be (root)", filename);
        }

        #[test]
        fn no_backslash_in_result(
            path in "[a-zA-Z0-9_/\\\\]+\\.[a-z]+",
            ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
            depth in 1usize..5
        ) {
            let key = module_key(&path, roots, depth);
            prop_assert!(!key.contains('\\'), "Module key contains backslash: {}", key);
        }

        #[test]
        fn deterministic(
            path in "[a-zA-Z0-9_/]+\\.[a-z]+",
            ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
            depth in 1usize..5
        ) {
            let key1 = module_key(&path, roots, depth);
            let key2 = module_key(&path, roots, depth);
            prop_assert_eq!(key1, key2, "Module key should be deterministic");
        }

        #[test]
        fn non_root_dir_returns_first_segment(
            dir in "[a-zA-Z0-9_]+",
            subdir in "[a-zA-Z0-9_]+",
            filename in "[a-zA-Z0-9_]+\\.[a-z]+"
        ) {
            // When first dir is not a root, module key is just that dir
            let path = format!("{}/{}/{}", dir, subdir, filename);
            let roots: Vec<String> = vec![];
            let key = module_key(&path, &roots, 2);
            prop_assert_eq!(&key, &dir, "Non-root path '{}' should return first dir, got '{}'", path, key);
        }

        #[test]
        fn matching_root_respects_depth(
            root in "[a-zA-Z0-9_]+",
            subdirs in prop::collection::vec("[a-zA-Z0-9_]+", 2..5),
            filename in "[a-zA-Z0-9_]+\\.[a-z]+",
            depth in 1usize..4
        ) {
            let path = format!("{}/{}/{}", root, subdirs.join("/"), filename);
            let roots = vec![root.clone()];
            let key = module_key(&path, &roots, depth);

            // Key depth should be min(depth, total_dirs)
            let key_segments: Vec<&str> = key.split('/').collect();
            let total_dirs = subdirs.len() + 1;
            let expected_segments = depth.min(total_dirs);

            prop_assert_eq!(key_segments.len(), expected_segments,
                "Key '{}' has {} segments, expected {} (path='{}', depth={})",
                key, key_segments.len(), expected_segments, path, depth);
        }

        #[test]
        fn normalized_input_equivalence(
            parts in prop::collection::vec("[a-zA-Z0-9_]+", 2..4),
            filename in "[a-zA-Z0-9_]+\\.[a-z]+"
        ) {
            // Different path separators should produce same result
            let fwd = format!("{}/{}", parts.join("/"), filename);
            let back = format!("{}\\{}", parts.join("\\"), filename);
            let dot_fwd = format!("./{}/{}", parts.join("/"), filename);

            let roots: Vec<String> = vec![];
            let k1 = module_key(&fwd, &roots, 2);
            let k2 = module_key(&back, &roots, 2);
            let k3 = module_key(&dot_fwd, &roots, 2);

            prop_assert_eq!(&k1, &k2, "Backslash normalization failed");
            prop_assert_eq!(&k1, &k3, "Dot-slash normalization failed");
        }
    }
}

#[cfg(test)]
mod avg_function {
    use proptest::prelude::*;
    use tokmd_model::avg;

    proptest! {
        #[test]
        fn zero_files_returns_zero(lines in 0usize..1000000) {
            prop_assert_eq!(avg(lines, 0), 0);
        }

        #[test]
        fn zero_lines_returns_zero(files in 1usize..10000) {
            prop_assert_eq!(avg(0, files), 0);
        }

        #[test]
        fn rounds_correctly(lines in 0usize..1000000, files in 1usize..10000) {
            let result = avg(lines, files);
            // Result should be roughly lines/files, within rounding
            let lower = lines / files;
            let upper = if lines % files == 0 { lower } else { lower + 1 };

            prop_assert!(result >= lower && result <= upper,
                "avg({}, {}) = {} not in [{}, {}]", lines, files, result, lower, upper);
        }

        #[test]
        fn exact_division(divisor in 1usize..1000) {
            // When lines divides evenly by files
            let files = divisor;
            let lines = divisor * 100;
            prop_assert_eq!(avg(lines, files), 100);
        }
    }
}
