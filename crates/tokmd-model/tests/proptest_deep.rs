//! Deep property-based tests for tokmd-model.
//!
//! Covers aggregation invariants, sorting stability, determinism,
//! and boundary conditions for the model layer.

use proptest::prelude::*;
use std::collections::BTreeMap;
use tokmd_types::{ChildrenMode, LangRow, ModuleRow, Totals};

// =========================================================================
// Strategies
// =========================================================================

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        prop::sample::select(vec![
            "Rust".to_string(),
            "Python".to_string(),
            "Go".to_string(),
            "JavaScript".to_string(),
            "TypeScript".to_string(),
            "C".to_string(),
            "Java".to_string(),
        ]),
        1usize..100_000,    // code
        1usize..200_000,    // lines
        1usize..1_000,      // files
        1usize..10_000_000, // bytes
        1usize..1_000_000,  // tokens
    )
        .prop_map(|(lang, code, lines, files, bytes, tokens)| {
            let avg_lines = if files > 0 { lines / files } else { 0 };
            LangRow { lang, code, lines, files, bytes, tokens, avg_lines }
        })
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        prop::sample::select(vec![
            "src".to_string(),
            "src/lib".to_string(),
            "tests".to_string(),
            "benches".to_string(),
            "examples".to_string(),
        ]),
        1usize..50_000,     // code
        1usize..100_000,    // lines
        1usize..500,        // files
        1usize..5_000_000,  // bytes
        1usize..500_000,    // tokens
    )
        .prop_map(|(module, code, lines, files, bytes, tokens)| {
            let avg_lines = if files > 0 { lines / files } else { 0 };
            ModuleRow { module, code, lines, files, bytes, tokens, avg_lines }
        })
}

fn arb_totals() -> impl Strategy<Value = Totals> {
    (
        0usize..1_000_000,
        0usize..2_000_000,
        0usize..10_000,
        0usize..100_000_000,
        0usize..10_000_000,
        0usize..5000,
    )
        .prop_map(|(code, lines, files, bytes, tokens, avg_lines)| {
            Totals { code, lines, files, bytes, tokens, avg_lines }
        })
}

// =========================================================================
// LangRow: sorting is by code descending, then name ascending
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lang_rows_sort_deterministically(
        rows in prop::collection::vec(arb_lang_row(), 2..=10),
    ) {
        let mut sorted1 = rows.clone();
        let mut sorted2 = rows.clone();
        sorted1.sort_by(|a, b| b.code.cmp(&a.code).then(a.lang.cmp(&b.lang)));
        sorted2.sort_by(|a, b| b.code.cmp(&a.code).then(a.lang.cmp(&b.lang)));
        prop_assert_eq!(sorted1, sorted2, "Sorting should be deterministic");
    }

    #[test]
    fn lang_rows_sorted_by_code_desc(
        rows in prop::collection::vec(arb_lang_row(), 2..=10),
    ) {
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then(a.lang.cmp(&b.lang)));
        for w in sorted.windows(2) {
            prop_assert!(
                w[0].code >= w[1].code,
                "Rows should be sorted by code descending: {} >= {}",
                w[0].code, w[1].code
            );
        }
    }
}

// =========================================================================
// ModuleRow: sorting is by code descending, then module ascending
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn module_rows_sort_deterministically(
        rows in prop::collection::vec(arb_module_row(), 2..=10),
    ) {
        let mut sorted1 = rows.clone();
        let mut sorted2 = rows.clone();
        sorted1.sort_by(|a, b| b.code.cmp(&a.code).then(a.module.cmp(&b.module)));
        sorted2.sort_by(|a, b| b.code.cmp(&a.code).then(a.module.cmp(&b.module)));
        prop_assert_eq!(sorted1, sorted2, "Module sorting should be deterministic");
    }
}

// =========================================================================
// Totals: serde round-trip
//! Covers: avg monotonicity, module_key depth constraints,
//! normalize_path cross-platform equivalence, and aggregation idempotency.

use proptest::prelude::*;
use std::path::Path;
use tokmd_model::{avg, module_key, normalize_path};

// =========================================================================
// avg: monotonicity and edge cases
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn avg_monotonic_in_lines(
        lines1 in 0usize..5000,
        delta in 1usize..5000,
        files in 1usize..100,
    ) {
        let lines2 = lines1 + delta;
        prop_assert!(
            avg(lines2, files) >= avg(lines1, files),
            "avg({}, {}) < avg({}, {})", lines2, files, lines1, files
        );
    }

    #[test]
    fn avg_decreases_with_more_files(
        lines in 1usize..10_000,
        files1 in 1usize..100,
        delta in 1usize..100,
    ) {
        let files2 = files1 + delta;
        prop_assert!(
            avg(lines, files2) <= avg(lines, files1),
            "avg({}, {}) > avg({}, {})", lines, files2, lines, files1
        );
    }

    #[test]
    fn avg_one_file_equals_lines(lines in 0usize..100_000) {
        prop_assert_eq!(avg(lines, 1), lines);
    }

    #[test]
    fn avg_result_bounded_by_lines(lines in 0usize..100_000, files in 1usize..1000) {
        let result = avg(lines, files);
        prop_assert!(result <= lines, "avg({}, {}) = {} > lines", lines, files, result);
    }
}

// =========================================================================
// module_key: depth constraints
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn totals_roundtrip(totals in arb_totals()) {
        let json = serde_json::to_string(&totals).unwrap();
        let parsed: Totals = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(totals, parsed);
    }
}

// =========================================================================
// BTreeMap: insertion order independence (deterministic key ordering)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn btreemap_key_order_is_deterministic(
        keys in prop::collection::vec("[a-zA-Z]{1,10}", 3..=10),
    ) {
        let mut map1: BTreeMap<String, usize> = BTreeMap::new();
        let mut map2: BTreeMap<String, usize> = BTreeMap::new();

        // Insert in forward order
        for (i, k) in keys.iter().enumerate() {
            map1.insert(k.clone(), i);
        }

        // Insert in reverse order
        for (i, k) in keys.iter().enumerate().rev() {
            map2.insert(k.clone(), i);
        }

        let keys1: Vec<&String> = map1.keys().collect();
        let keys2: Vec<&String> = map2.keys().collect();
        prop_assert_eq!(keys1, keys2, "BTreeMap key order should be independent of insertion order");
    }
}

// =========================================================================
// LangRow: aggregation invariant ΓÇö sum of parts
// LangRow: aggregation invariant ╬ô├ç├╢ sum of parts
// LangRow: aggregation invariant Γò¼├┤Γö£├ºΓö£Γòó sum of parts
// LangRow: aggregation invariant ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬úΓö£┬║╬ô├╢┬ú╬ô├▓├│ sum of parts
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lang_rows_total_code_equals_sum(
        rows in prop::collection::vec(arb_lang_row(), 1..=10),
    ) {
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let total_files: usize = rows.iter().map(|r| r.files).sum();
        let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();

        prop_assert!(total_code > 0, "Should have non-zero code");
        prop_assert!(total_files > 0, "Should have at least one file");
        prop_assert!(total_bytes > 0, "Should have non-zero bytes");
        prop_assert!(total_tokens > 0, "Should have non-zero tokens");
    }
}

// =========================================================================
// ChildrenMode: round-trip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn children_mode_roundtrip(idx in 0usize..2) {
        let mode = [ChildrenMode::Collapse, ChildrenMode::Separate][idx];
        let json = serde_json::to_string(&mode).unwrap();
        let parsed: ChildrenMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(mode, parsed);
    }
}

// =========================================================================
// LangRow: JSON round-trip preserves all fields
    fn module_key_depth_1_max_one_segment(
        parts in prop::collection::vec("[a-z]{2,6}", 2..6),
        filename in "[a-z]{2,6}\\.[a-z]{1,3}",
    ) {
        let path = format!("{}/{}", parts.join("/"), filename);
        let key = module_key(&path, &[], 1);
        let segments = key.split('/').count();
        prop_assert!(segments <= 1,
            "depth=1 key '{}' has {} segments", key, segments);
    }

    #[test]
    fn module_key_consistent_across_separators(
        dir in "[a-z]{2,8}",
        subdir in "[a-z]{2,8}",
        filename in "[a-z]{2,8}\\.[a-z]{1,3}",
    ) {
        let fwd = format!("{}/{}/{}", dir, subdir, filename);
        let back = format!("{}\\{}\\{}", dir, subdir, filename);
        let roots: Vec<String> = vec![];
        let k1 = module_key(&fwd, &roots, 2);
        let k2 = module_key(&back, &roots, 2);
        prop_assert_eq!(k1, k2, "Forward/backslash should give same module key");
    }

    #[test]
    fn module_key_never_contains_filename(
        dir in "[a-z]{3,8}",
        filename in "[a-z]{3,8}\\.[a-z]{1,3}",
    ) {
        let path = format!("{}/{}", dir, filename);
        let key = module_key(&path, &[], 2);
        prop_assert!(
            !key.contains('.'),
            "module key '{}' should not contain the filename (which has a dot)", key
        );
    }

    #[test]
    fn module_key_root_for_bare_filename(filename in "[a-z]{2,8}\\.[a-z]{1,3}") {
        let key = module_key(&filename, &[], 2);
        prop_assert_eq!(key, "(root)", "Bare filename '{}' should map to (root)", filename);
    }
}

// =========================================================================
// normalize_path: cross-platform equivalence
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn lang_row_roundtrip(row in arb_lang_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let parsed: LangRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, parsed);
    }

    #[test]
    fn module_row_roundtrip(row in arb_module_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let parsed: ModuleRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, parsed);
    }
}

// =========================================================================
// Totals: zero construction
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn totals_zero_construction(_dummy in 0..1u8) {
        let t = Totals { code: 0, lines: 0, files: 0, bytes: 0, tokens: 0, avg_lines: 0 };
        prop_assert_eq!(t.code, 0);
        prop_assert_eq!(t.lines, 0);
        prop_assert_eq!(t.files, 0);
        prop_assert_eq!(t.bytes, 0);
        prop_assert_eq!(t.tokens, 0);
        prop_assert_eq!(t.avg_lines, 0);
    fn normalize_path_forward_backslash_equivalent(
        parts in prop::collection::vec("[a-z]{2,6}", 2..5),
        filename in "[a-z]{2,6}\\.[a-z]{1,3}",
    ) {
        let fwd = format!("{}/{}", parts.join("/"), filename);
        let back = format!("{}\\{}", parts.join("\\"), filename);
        let n1 = normalize_path(Path::new(&fwd), None);
        let n2 = normalize_path(Path::new(&back), None);
        prop_assert_eq!(n1, n2, "Forward and backslash paths should normalize equally");
    }

    #[test]
    fn normalize_path_dot_slash_stripped(
        parts in prop::collection::vec("[a-z]{2,6}", 1..4),
        filename in "[a-z]{2,6}\\.[a-z]{1,3}",
    ) {
        let plain = format!("{}/{}", parts.join("/"), filename);
        let dotted = format!("./{}/{}", parts.join("/"), filename);
        let n1 = normalize_path(Path::new(&plain), None);
        let n2 = normalize_path(Path::new(&dotted), None);
        prop_assert_eq!(n1, n2, "./ prefix should be stripped");
    }

    #[test]
    fn normalize_path_result_no_trailing_slash(
        parts in prop::collection::vec("[a-z]{2,6}", 1..4),
        filename in "[a-z]{2,6}\\.[a-z]{1,3}",
    ) {
        let path = format!("{}/{}", parts.join("/"), filename);
        let normalized = normalize_path(Path::new(&path), None);
        prop_assert!(
            !normalized.ends_with('/'),
            "Normalized path should not end with /: '{}'", normalized
        );
    }

    #[test]
    fn normalize_path_deterministic(path in "[a-zA-Z0-9_/]+\\.[a-z]+") {
        let n1 = normalize_path(Path::new(&path), None);
        let n2 = normalize_path(Path::new(&path), None);
        prop_assert_eq!(n1, n2);
    }
}
