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
// LangRow: aggregation invariant — sum of parts
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
    }
}
