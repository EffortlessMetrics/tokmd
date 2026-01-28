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

        prop_assert!(!normalized.starts_with(&prefix_path),
            "Prefix '{}' should be stripped from '{}', got '{}'", prefix_path, full_path, normalized);
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

// ========================
// Aggregation Properties (fold_other_*)
// ========================
// These test the totals preservation invariant

mod aggregation {
    use proptest::prelude::*;
    use tokmd_types::{LangRow, ModuleRow};

    fn fold_other_lang(rows: &[LangRow]) -> LangRow {
        let mut code = 0usize;
        let mut lines = 0usize;
        let mut files = 0usize;
        let mut bytes = 0usize;
        let mut tokens = 0usize;

        for r in rows {
            code += r.code;
            lines += r.lines;
            files += r.files;
            bytes += r.bytes;
            tokens += r.tokens;
        }

        let avg_lines = if files == 0 { 0 } else { (lines + (files / 2)) / files };
        LangRow {
            lang: "Other".to_string(),
            code,
            lines,
            files,
            bytes,
            tokens,
            avg_lines,
        }
    }

    fn fold_other_module(rows: &[ModuleRow]) -> ModuleRow {
        let mut code = 0usize;
        let mut lines = 0usize;
        let mut files = 0usize;
        let mut bytes = 0usize;
        let mut tokens = 0usize;

        for r in rows {
            code += r.code;
            lines += r.lines;
            files += r.files;
            bytes += r.bytes;
            tokens += r.tokens;
        }

        let avg_lines = if files == 0 { 0 } else { (lines + (files / 2)) / files };
        ModuleRow {
            module: "Other".to_string(),
            code,
            lines,
            files,
            bytes,
            tokens,
            avg_lines,
        }
    }

    fn arb_lang_row() -> impl Strategy<Value = LangRow> {
        (
            "[a-zA-Z]+",
            0usize..10000,
            0usize..20000,
            0usize..1000,
            0usize..1000000,
            0usize..100000,
        ).prop_map(|(lang, code, lines, files, bytes, tokens)| {
            let avg_lines = if files == 0 { 0 } else { (lines + (files / 2)) / files };
            LangRow { lang, code, lines, files, bytes, tokens, avg_lines }
        })
    }

    fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
        (
            "[a-zA-Z0-9_/]+",
            0usize..10000,
            0usize..20000,
            0usize..1000,
            0usize..1000000,
            0usize..100000,
        ).prop_map(|(module, code, lines, files, bytes, tokens)| {
            let avg_lines = if files == 0 { 0 } else { (lines + (files / 2)) / files };
            ModuleRow { module, code, lines, files, bytes, tokens, avg_lines }
        })
    }

    proptest! {
        #[test]
        fn fold_lang_preserves_totals(rows in prop::collection::vec(arb_lang_row(), 0..10)) {
            let folded = fold_other_lang(&rows);

            let total_code: usize = rows.iter().map(|r| r.code).sum();
            let total_lines: usize = rows.iter().map(|r| r.lines).sum();
            let total_files: usize = rows.iter().map(|r| r.files).sum();
            let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
            let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();

            prop_assert_eq!(folded.code, total_code, "Code mismatch");
            prop_assert_eq!(folded.lines, total_lines, "Lines mismatch");
            prop_assert_eq!(folded.files, total_files, "Files mismatch");
            prop_assert_eq!(folded.bytes, total_bytes, "Bytes mismatch");
            prop_assert_eq!(folded.tokens, total_tokens, "Tokens mismatch");
        }

        #[test]
        fn fold_lang_empty_is_zero(_dummy in 0..1u8) {
            let folded = fold_other_lang(&[]);
            prop_assert_eq!(folded.code, 0);
            prop_assert_eq!(folded.lines, 0);
            prop_assert_eq!(folded.files, 0);
            prop_assert_eq!(folded.bytes, 0);
            prop_assert_eq!(folded.tokens, 0);
            prop_assert_eq!(folded.lang, "Other");
        }

        #[test]
        fn fold_module_preserves_totals(rows in prop::collection::vec(arb_module_row(), 0..10)) {
            let folded = fold_other_module(&rows);

            let total_code: usize = rows.iter().map(|r| r.code).sum();
            let total_lines: usize = rows.iter().map(|r| r.lines).sum();
            let total_files: usize = rows.iter().map(|r| r.files).sum();
            let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
            let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();

            prop_assert_eq!(folded.code, total_code, "Code mismatch");
            prop_assert_eq!(folded.lines, total_lines, "Lines mismatch");
            prop_assert_eq!(folded.files, total_files, "Files mismatch");
            prop_assert_eq!(folded.bytes, total_bytes, "Bytes mismatch");
            prop_assert_eq!(folded.tokens, total_tokens, "Tokens mismatch");
        }

        #[test]
        fn fold_module_empty_is_zero(_dummy in 0..1u8) {
            let folded = fold_other_module(&[]);
            prop_assert_eq!(folded.code, 0);
            prop_assert_eq!(folded.lines, 0);
            prop_assert_eq!(folded.files, 0);
            prop_assert_eq!(folded.bytes, 0);
            prop_assert_eq!(folded.tokens, 0);
            prop_assert_eq!(folded.module, "Other");
        }

        #[test]
        fn fold_associative_lang(
            rows1 in prop::collection::vec(arb_lang_row(), 0..5),
            rows2 in prop::collection::vec(arb_lang_row(), 0..5)
        ) {
            // Folding all at once should equal folding parts and combining
            let all: Vec<_> = rows1.iter().chain(rows2.iter()).cloned().collect();
            let fold_all = fold_other_lang(&all);

            let fold1 = fold_other_lang(&rows1);
            let fold2 = fold_other_lang(&rows2);
            let combined = fold_other_lang(&[fold1, fold2]);

            prop_assert_eq!(fold_all.code, combined.code);
            prop_assert_eq!(fold_all.lines, combined.lines);
            prop_assert_eq!(fold_all.files, combined.files);
            prop_assert_eq!(fold_all.bytes, combined.bytes);
            prop_assert_eq!(fold_all.tokens, combined.tokens);
        }
    }
}
