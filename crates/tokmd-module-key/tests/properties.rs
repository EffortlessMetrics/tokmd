use proptest::prelude::*;
use tokmd_module_key::module_key;

proptest! {
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
        let key = module_key(&filename, &[], 2);
        prop_assert_eq!(key, "(root)");
    }

    #[test]
    fn module_key_non_matching_root_is_first_dir(
        dir in "[a-zA-Z0-9_]+",
        subdirs in prop::collection::vec("[a-zA-Z0-9_]+", 1..3),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+"
    ) {
        let path_parts: Vec<&str> = std::iter::once(dir.as_str())
            .chain(subdirs.iter().map(|s| s.as_str()))
            .chain(std::iter::once(filename.as_str()))
            .collect();
        let path = path_parts.join("/");

        let roots = vec!["does_not_match".to_string()];
        let key = module_key(&path, &roots, 3);
        prop_assert_eq!(&key, &dir);
    }

    #[test]
    fn module_key_matching_root_respects_depth(
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
        let key_depth = key.split('/').count();
        let max_dirs = subdirs.len() + 1;
        let expected_depth = depth.min(max_dirs);

        prop_assert_eq!(key_depth, expected_depth);
    }

    #[test]
    fn module_key_deterministic(
        path in "[a-zA-Z0-9_/]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 1usize..5
    ) {
        let k1 = module_key(&path, roots, depth);
        let k2 = module_key(&path, roots, depth);
        prop_assert_eq!(k1, k2);
    }

    #[test]
    fn module_key_normalizes_separator_forms(
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

        prop_assert_eq!(&k_forward, &k_back);
        prop_assert_eq!(&k_forward, &k_dot);
    }

    #[test]
    fn module_key_output_never_contains_backslash(
        path in "[a-zA-Z0-9_/\\\\]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 1usize..5
    ) {
        let key = module_key(&path, roots, depth);
        prop_assert!(!key.contains('\\'));
    }

    #[test]
    fn module_key_never_ends_with_slash(
        path in "[a-zA-Z0-9_/]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 0usize..6
    ) {
        let key = module_key(&path, roots, depth);
        prop_assert!(!key.ends_with('/'));
    }

    #[test]
    fn module_key_never_empty(
        path in "[a-zA-Z0-9_./\\\\]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 0usize..6
    ) {
        let key = module_key(&path, roots, depth);
        prop_assert!(!key.is_empty());
    }

    #[test]
    fn module_key_segments_are_subset_of_path_dirs(
        dirs in prop::collection::vec("[a-zA-Z0-9_]+", 1..5),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+",
        depth in 1usize..5
    ) {
        let path = format!("{}/{}", dirs.join("/"), filename);
        let roots = vec![dirs[0].clone()];
        let key = module_key(&path, &roots, depth);

        // Every segment of the key must appear in the original dir segments
        for seg in key.split('/') {
            prop_assert!(dirs.contains(&seg.to_string()),
                "key segment {:?} not in path dirs {:?}", seg, dirs);
        }
    }

    #[test]
    fn module_key_depth_zero_equals_depth_one(
        root in "[a-zA-Z0-9_]+",
        subdirs in prop::collection::vec("[a-zA-Z0-9_]+", 1..4),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+"
    ) {
        let path = format!("{}/{}/{}", root, subdirs.join("/"), filename);
        let roots = vec![root.clone()];
        let k0 = module_key(&path, &roots, 0);
        let k1 = module_key(&path, &roots, 1);
        prop_assert_eq!(k0, k1);
    }

    #[test]
    fn module_key_dot_slash_prefix_is_idempotent(
        dirs in prop::collection::vec("[a-zA-Z0-9_]+", 1..4),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 1usize..4
    ) {
        let plain = format!("{}/{}", dirs.join("/"), filename);
        let dotslash = format!("./{}/{}", dirs.join("/"), filename);
        let k_plain = module_key(&plain, roots, depth);
        let k_dot = module_key(&dotslash, roots, depth);
        prop_assert_eq!(k_plain, k_dot);
    }

    #[test]
    fn module_key_increasing_depth_grows_monotonically(
        root in "[a-zA-Z0-9_]+",
        subdirs in prop::collection::vec("[a-zA-Z0-9_]+", 3..6),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+"
    ) {
        let path = format!("{}/{}/{}", root, subdirs.join("/"), filename);
        let roots = vec![root.clone()];
        let mut prev_len = 0;
        for d in 1..=5 {
            let key = module_key(&path, &roots, d);
            let cur_len = key.split('/').count();
            prop_assert!(cur_len >= prev_len,
                "depth {} produced fewer segments ({}) than depth {} ({})",
                d, cur_len, d - 1, prev_len);
            prev_len = cur_len;
        }
    }
}
