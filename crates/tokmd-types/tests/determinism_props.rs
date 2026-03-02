//! Property-based determinism hardening tests for tokmd-types.
//!
//! These tests verify type-level invariants that underpin the determinism
//! guarantee: BTreeMap ordering, sorting comparators, path normalization,
//! redaction stability, module key stability, and schema version consistency.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_types::{
    CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, FileKind, FileRow,
    HANDOFF_SCHEMA_VERSION, LangRow, ModuleRow, SCHEMA_VERSION, Totals,
    cockpit::COCKPIT_SCHEMA_VERSION,
};

// =========================================================================
// Helpers: arbitrary generators
// =========================================================================

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        "[A-Za-z][A-Za-z0-9 ]{0,15}",
        0usize..50_000,
        0usize..100_000,
        1usize..5_000,
        0usize..5_000_000,
        0usize..500_000,
        0usize..500,
    )
        .prop_map(
            |(lang, code, lines, files, bytes, tokens, avg_lines)| LangRow {
                lang,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines,
            },
        )
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        "[a-z][a-z0-9_/]{0,20}",
        0usize..50_000,
        0usize..100_000,
        1usize..5_000,
        0usize..5_000_000,
        0usize..500_000,
        0usize..500,
    )
        .prop_map(
            |(module, code, lines, files, bytes, tokens, avg_lines)| ModuleRow {
                module,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines,
            },
        )
}

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        "[a-z][a-z0-9_/]{1,30}\\.[a-z]{1,4}",
        "[a-z][a-z0-9_/]{0,15}",
        "[A-Za-z][A-Za-z0-9 ]{0,10}",
        prop_oneof![Just(FileKind::Parent), Just(FileKind::Child)],
        0usize..50_000,
        0usize..10_000,
        0usize..10_000,
        0usize..100_000,
        0usize..5_000_000,
        0usize..500_000,
    )
        .prop_map(
            |(path, module, lang, kind, code, comments, blanks, lines, bytes, tokens)| FileRow {
                path,
                module,
                lang,
                kind,
                code,
                comments,
                blanks,
                lines,
                bytes,
                tokens,
            },
        )
}

// =========================================================================
// 1. BTreeMap ordering: keys are always sorted
// =========================================================================

proptest! {
    #[test]
    fn btreemap_string_keys_always_sorted(
        entries in proptest::collection::vec(
            ("[a-z]{1,10}", 0u64..1000),
            1..50
        )
    ) {
        let map: BTreeMap<String, u64> = entries.into_iter().collect();
        let keys: Vec<&String> = map.keys().collect();
        for pair in keys.windows(2) {
            prop_assert!(pair[0] <= pair[1], "BTreeMap keys out of order: {:?} > {:?}", pair[0], pair[1]);
        }
    }

    #[test]
    fn btreemap_serialization_preserves_order(
        entries in proptest::collection::vec(
            ("[a-z]{1,10}", 0u64..1000),
            1..30
        )
    ) {
        let map: BTreeMap<String, u64> = entries.into_iter().collect();
        let json = serde_json::to_string(&map).expect("serialize");
        let back: BTreeMap<String, u64> = serde_json::from_str(&json).expect("deserialize");
        let keys_before: Vec<&String> = map.keys().collect();
        let keys_after: Vec<&String> = back.keys().collect();
        prop_assert_eq!(keys_before, keys_after, "BTreeMap key order not preserved through JSON roundtrip");
    }
}

// =========================================================================
// 2. Sorting invariants: lang rows
// =========================================================================

proptest! {
    #[test]
    fn lang_sort_descending_code_then_name(
        mut rows in proptest::collection::vec(arb_lang_row(), 2..20)
    ) {
        rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

        for pair in rows.windows(2) {
            prop_assert!(
                pair[0].code > pair[1].code
                    || (pair[0].code == pair[1].code && pair[0].lang <= pair[1].lang),
                "lang sort violated: {}({}) before {}({})",
                pair[0].lang, pair[0].code, pair[1].lang, pair[1].code
            );
        }
    }

    #[test]
    fn module_sort_descending_code_then_name(
        mut rows in proptest::collection::vec(arb_module_row(), 2..20)
    ) {
        rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));

        for pair in rows.windows(2) {
            prop_assert!(
                pair[0].code > pair[1].code
                    || (pair[0].code == pair[1].code && pair[0].module <= pair[1].module),
                "module sort violated: {}({}) before {}({})",
                pair[0].module, pair[0].code, pair[1].module, pair[1].code
            );
        }
    }

    #[test]
    fn file_sort_descending_code_then_path(
        mut rows in proptest::collection::vec(arb_file_row(), 2..20)
    ) {
        rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));

        for pair in rows.windows(2) {
            prop_assert!(
                pair[0].code > pair[1].code
                    || (pair[0].code == pair[1].code && pair[0].path <= pair[1].path),
                "file sort violated: {}({}) before {}({})",
                pair[0].path, pair[0].code, pair[1].path, pair[1].code
            );
        }
    }
}

// =========================================================================
// 3. Sort stability (idempotence)
// =========================================================================

proptest! {
    #[test]
    fn lang_sort_is_idempotent(
        mut rows in proptest::collection::vec(arb_lang_row(), 2..20)
    ) {
        let sort_fn = |v: &mut Vec<LangRow>| {
            v.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        };
        sort_fn(&mut rows);
        let first_pass: Vec<String> = rows.iter().map(|r| format!("{}:{}", r.lang, r.code)).collect();
        sort_fn(&mut rows);
        let second_pass: Vec<String> = rows.iter().map(|r| format!("{}:{}", r.lang, r.code)).collect();
        prop_assert_eq!(first_pass, second_pass, "sort must be idempotent");
    }
}

// =========================================================================
// 4. Path normalization edge cases
// =========================================================================

proptest! {
    #[test]
    fn normalize_slashes_removes_all_backslashes(path in "[a-z0-9_./\\\\]{1,50}") {
        let normalized = tokmd_path::normalize_slashes(&path);
        prop_assert!(!normalized.contains('\\'), "backslash in normalized: {normalized}");
    }

    #[test]
    fn normalize_slashes_is_idempotent(path in "[a-z0-9_./\\\\]{1,50}") {
        let once = tokmd_path::normalize_slashes(&path);
        let twice = tokmd_path::normalize_slashes(&once);
        prop_assert_eq!(&once, &twice, "normalize_slashes not idempotent");
    }

    #[test]
    fn normalize_rel_path_no_backslash(path in "[a-z0-9_./\\\\]{1,50}") {
        let normalized = tokmd_path::normalize_rel_path(&path);
        prop_assert!(!normalized.contains('\\'), "backslash in normalized rel path: {normalized}");
    }

    #[test]
    fn normalize_rel_path_is_idempotent(path in "[a-z0-9_./\\\\]{1,50}") {
        let once = tokmd_path::normalize_rel_path(&path);
        let twice = tokmd_path::normalize_rel_path(&once);
        prop_assert_eq!(&once, &twice, "normalize_rel_path not idempotent");
    }
}

#[test]
fn path_normalize_trailing_slash() {
    let result = tokmd_path::normalize_slashes("src/lib/");
    assert_eq!(result, "src/lib/");
}

#[test]
fn path_normalize_dot_segments() {
    let result = tokmd_path::normalize_rel_path("./src/main.rs");
    assert_eq!(result, "src/main.rs");
}

#[test]
fn path_normalize_double_separators() {
    let result = tokmd_path::normalize_slashes("src//lib.rs");
    assert_eq!(result, "src//lib.rs");
}

#[test]
fn path_normalize_windows_backslash() {
    let result = tokmd_path::normalize_slashes(r"crates\tokmd\src\main.rs");
    assert_eq!(result, "crates/tokmd/src/main.rs");
}

#[test]
fn path_normalize_mixed_separators() {
    let result = tokmd_path::normalize_slashes(r"crates/tokmd\src/main.rs");
    assert_eq!(result, "crates/tokmd/src/main.rs");
}

#[test]
fn path_normalize_dot_backslash_prefix() {
    let result = tokmd_path::normalize_rel_path(r".\src\main.rs");
    assert_eq!(result, "src/main.rs");
}

#[test]
fn path_normalize_empty_string() {
    assert_eq!(tokmd_path::normalize_slashes(""), "");
    assert_eq!(tokmd_path::normalize_rel_path(""), "");
}

#[test]
fn path_normalize_just_dot() {
    assert_eq!(tokmd_path::normalize_rel_path("."), ".");
}

#[test]
fn path_normalize_parent_ref_preserved() {
    assert_eq!(
        tokmd_path::normalize_rel_path("../src/lib.rs"),
        "../src/lib.rs"
    );
}

// =========================================================================
// 5. Module key determinism
// =========================================================================

#[test]
fn module_key_same_path_same_result() {
    let roots = vec!["crates".to_string()];
    let path = "crates/tokmd/src/lib.rs";
    let k1 = tokmd_module_key::module_key(path, &roots, 2);
    let k2 = tokmd_module_key::module_key(path, &roots, 2);
    assert_eq!(k1, k2, "module_key must be deterministic");
}

#[test]
fn module_key_uses_forward_slashes() {
    let roots = vec!["crates".to_string()];
    let key = tokmd_module_key::module_key(r"crates\tokmd\src\lib.rs", &roots, 2);
    assert!(!key.contains('\\'), "module key contains backslash: {key}");
    assert_eq!(key, "crates/tokmd");
}

#[test]
fn module_key_dot_prefix_stripped() {
    let roots = vec!["crates".to_string()];
    let k1 = tokmd_module_key::module_key("crates/foo/src/lib.rs", &roots, 2);
    let k2 = tokmd_module_key::module_key("./crates/foo/src/lib.rs", &roots, 2);
    assert_eq!(k1, k2, "leading ./ must not affect module key");
}

#[test]
fn module_key_root_level_files() {
    let roots = vec!["crates".to_string()];
    assert_eq!(
        tokmd_module_key::module_key("Cargo.toml", &roots, 2),
        "(root)"
    );
    assert_eq!(
        tokmd_module_key::module_key("./README.md", &roots, 2),
        "(root)"
    );
}

proptest! {
    #[test]
    fn module_key_is_deterministic(
        path in "[a-z]{1,5}(/[a-z]{1,5}){1,4}/[a-z]{1,5}\\.[a-z]{1,3}",
        depth in 1usize..5
    ) {
        let roots = vec!["crates".to_string(), "packages".to_string()];
        let k1 = tokmd_module_key::module_key(&path, &roots, depth);
        let k2 = tokmd_module_key::module_key(&path, &roots, depth);
        prop_assert_eq!(k1, k2, "module_key must always return the same value");
    }

    #[test]
    fn module_key_no_backslashes(
        path in "[a-z0-9_./\\\\]{1,40}/[a-z]{1,5}\\.[a-z]{1,3}",
        depth in 1usize..5
    ) {
        let roots = vec!["crates".to_string()];
        let key = tokmd_module_key::module_key(&path, &roots, depth);
        prop_assert!(!key.contains('\\'), "module key contains backslash: {key}");
    }
}

// =========================================================================
// 6. Redaction determinism
// =========================================================================

#[test]
fn redact_same_path_same_hash() {
    let h1 = tokmd_redact::short_hash("src/main.rs");
    let h2 = tokmd_redact::short_hash("src/main.rs");
    assert_eq!(h1, h2, "short_hash must be deterministic");
}

#[test]
fn redact_path_same_input_same_output() {
    let r1 = tokmd_redact::redact_path("src/secrets/config.json");
    let r2 = tokmd_redact::redact_path("src/secrets/config.json");
    assert_eq!(r1, r2, "redact_path must be deterministic");
}

#[test]
fn redact_cross_platform_consistency() {
    let h_unix = tokmd_redact::short_hash("src/lib.rs");
    let h_win = tokmd_redact::short_hash(r"src\lib.rs");
    assert_eq!(h_unix, h_win, "hash must be cross-platform consistent");
}

#[test]
fn redact_dot_prefix_consistency() {
    let h1 = tokmd_redact::short_hash("src/lib.rs");
    let h2 = tokmd_redact::short_hash("./src/lib.rs");
    assert_eq!(h1, h2, "leading ./ must not affect hash");
}

#[test]
fn redact_path_preserves_extension() {
    let redacted = tokmd_redact::redact_path("src/main.rs");
    assert!(
        redacted.ends_with(".rs"),
        "extension not preserved: {redacted}"
    );
}

#[test]
fn redact_path_length_is_fixed() {
    let r1 = tokmd_redact::redact_path("a.rs");
    let r2 = tokmd_redact::redact_path("very/deep/nested/path/to/file.rs");
    assert_eq!(
        r1.len(),
        r2.len(),
        "redacted paths should have same length for same extension"
    );
}

proptest! {
    #[test]
    fn short_hash_is_deterministic(input in "\\PC{1,50}") {
        let h1 = tokmd_redact::short_hash(&input);
        let h2 = tokmd_redact::short_hash(&input);
        prop_assert_eq!(h1, h2, "short_hash not deterministic");
    }

    #[test]
    fn short_hash_always_16_chars(input in "\\PC{1,50}") {
        let hash = tokmd_redact::short_hash(&input);
        prop_assert_eq!(hash.len(), 16, "short_hash length is not 16: {}", hash);
    }

    #[test]
    fn redact_path_is_deterministic(input in "[a-z]{1,5}(/[a-z]{1,5}){0,3}/[a-z]{1,8}\\.[a-z]{1,4}") {
        let r1 = tokmd_redact::redact_path(&input);
        let r2 = tokmd_redact::redact_path(&input);
        prop_assert_eq!(r1, r2, "redact_path not deterministic");
    }
}

// =========================================================================
// 7. Schema version stability
// =========================================================================

#[test]
fn schema_versions_are_stable() {
    assert_eq!(SCHEMA_VERSION, 2);
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3);
}

// =========================================================================
// 8. Serialization round-trip determinism
// =========================================================================

proptest! {
    #[test]
    fn lang_row_json_roundtrip_deterministic(row in arb_lang_row()) {
        let json1 = serde_json::to_string(&row).expect("serialize");
        let back: LangRow = serde_json::from_str(&json1).expect("deserialize");
        let json2 = serde_json::to_string(&back).expect("re-serialize");
        prop_assert_eq!(&json1, &json2, "LangRow JSON roundtrip not deterministic");
    }

    #[test]
    fn module_row_json_roundtrip_deterministic(row in arb_module_row()) {
        let json1 = serde_json::to_string(&row).expect("serialize");
        let back: ModuleRow = serde_json::from_str(&json1).expect("deserialize");
        let json2 = serde_json::to_string(&back).expect("re-serialize");
        prop_assert_eq!(&json1, &json2, "ModuleRow JSON roundtrip not deterministic");
    }

    #[test]
    fn file_row_json_roundtrip_deterministic(row in arb_file_row()) {
        let json1 = serde_json::to_string(&row).expect("serialize");
        let back: FileRow = serde_json::from_str(&json1).expect("deserialize");
        let json2 = serde_json::to_string(&back).expect("re-serialize");
        prop_assert_eq!(&json1, &json2, "FileRow JSON roundtrip not deterministic");
    }

    #[test]
    fn totals_json_roundtrip_deterministic(
        code in 0usize..100_000,
        lines in 0usize..200_000,
        files in 0usize..10_000,
        bytes in 0usize..10_000_000,
        tokens in 0usize..1_000_000,
        avg_lines in 0usize..1_000,
    ) {
        let totals = Totals { code, lines, files, bytes, tokens, avg_lines };
        let json1 = serde_json::to_string(&totals).expect("serialize");
        let back: Totals = serde_json::from_str(&json1).expect("deserialize");
        let json2 = serde_json::to_string(&back).expect("re-serialize");
        prop_assert_eq!(&json1, &json2, "Totals JSON roundtrip not deterministic");
    }
}

// =========================================================================
// 9. BTreeMap as collection type never HashMap
// =========================================================================

#[test]
fn btreemap_insertion_order_independent() {
    let mut map_a: BTreeMap<String, usize> = BTreeMap::new();
    map_a.insert("zebra".into(), 1);
    map_a.insert("alpha".into(), 2);
    map_a.insert("middle".into(), 3);

    let mut map_b: BTreeMap<String, usize> = BTreeMap::new();
    map_b.insert("middle".into(), 3);
    map_b.insert("alpha".into(), 2);
    map_b.insert("zebra".into(), 1);

    let keys_a: Vec<&String> = map_a.keys().collect();
    let keys_b: Vec<&String> = map_b.keys().collect();
    assert_eq!(
        keys_a, keys_b,
        "BTreeMap must sort keys regardless of insertion order"
    );

    let json_a = serde_json::to_string(&map_a).unwrap();
    let json_b = serde_json::to_string(&map_b).unwrap();
    assert_eq!(
        json_a, json_b,
        "BTreeMap JSON must be identical regardless of insertion order"
    );
}
