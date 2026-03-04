//! Property-based determinism tests for tokmd-types (wave 45).
//!
//! Complements `determinism_props.rs` with additional hardening properties:
//! - Sorting determinism with many ties (same code count)
//! - Sort-then-serialize stability (double-sort produces identical JSON)
//! - Path normalization composition (slashes + rel-path)
//! - BTreeMap merge determinism
//!
//! Run with: `cargo test -p tokmd-types --test determinism_proptest`

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_types::{FileKind, FileRow, LangRow, ModuleRow};

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

/// Generate lang rows where many share the same code count to stress tie-breaking.
fn arb_lang_rows_with_ties() -> impl Strategy<Value = Vec<LangRow>> {
    let code_value = 0usize..1_000;
    code_value.prop_flat_map(|shared_code| {
        proptest::collection::vec(
            (
                "[A-Za-z][A-Za-z0-9]{0,10}",
                prop_oneof![Just(shared_code), 0usize..50_000],
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
                ),
            3..30,
        )
    })
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

fn sort_langs(v: &mut [LangRow]) {
    v.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
}

fn sort_modules(v: &mut [ModuleRow]) {
    v.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
}

fn sort_files(v: &mut [FileRow]) {
    v.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));
}

// =========================================================================
// 1. Sorting with many ties is deterministic
// =========================================================================

proptest! {
    #[test]
    fn lang_sort_with_ties_is_deterministic(mut rows in arb_lang_rows_with_ties()) {
        sort_langs(&mut rows);
        let first: Vec<String> = rows.iter().map(|r| format!("{}:{}", r.lang, r.code)).collect();
        sort_langs(&mut rows);
        let second: Vec<String> = rows.iter().map(|r| format!("{}:{}", r.lang, r.code)).collect();
        prop_assert_eq!(first, second, "sort with ties must be deterministic");
    }

    #[test]
    fn lang_sort_ties_broken_by_name_ascending(mut rows in arb_lang_rows_with_ties()) {
        sort_langs(&mut rows);
        for pair in rows.windows(2) {
            if pair[0].code == pair[1].code {
                prop_assert!(
                    pair[0].lang <= pair[1].lang,
                    "tie-break violated: {} should come before {}",
                    pair[0].lang, pair[1].lang
                );
            }
        }
    }
}

// =========================================================================
// 2. Sort-then-serialize stability (sort → serialize → deserialize → sort → serialize = same)
// =========================================================================

proptest! {
    #[test]
    fn lang_sort_serialize_roundtrip_stable(mut rows in proptest::collection::vec(arb_lang_row(), 2..20)) {
        sort_langs(&mut rows);
        let json1 = serde_json::to_string(&rows).expect("serialize");

        let mut back: Vec<LangRow> = serde_json::from_str(&json1).expect("deserialize");
        sort_langs(&mut back);
        let json2 = serde_json::to_string(&back).expect("re-serialize");

        prop_assert_eq!(&json1, &json2, "sort → JSON → sort → JSON must be stable");
    }

    #[test]
    fn module_sort_serialize_roundtrip_stable(mut rows in proptest::collection::vec(arb_module_row(), 2..20)) {
        sort_modules(&mut rows);
        let json1 = serde_json::to_string(&rows).expect("serialize");

        let mut back: Vec<ModuleRow> = serde_json::from_str(&json1).expect("deserialize");
        sort_modules(&mut back);
        let json2 = serde_json::to_string(&back).expect("re-serialize");

        prop_assert_eq!(&json1, &json2, "sort → JSON → sort → JSON must be stable");
    }

    #[test]
    fn file_sort_serialize_roundtrip_stable(mut rows in proptest::collection::vec(arb_file_row(), 2..20)) {
        sort_files(&mut rows);
        let json1 = serde_json::to_string(&rows).expect("serialize");

        let mut back: Vec<FileRow> = serde_json::from_str(&json1).expect("deserialize");
        sort_files(&mut back);
        let json2 = serde_json::to_string(&back).expect("re-serialize");

        prop_assert_eq!(&json1, &json2, "sort → JSON → sort → JSON must be stable");
    }
}

// =========================================================================
// 3. Path normalization composition and idempotency
// =========================================================================

proptest! {
    #[test]
    fn normalize_slashes_then_rel_path_is_idempotent(path in "[a-z0-9_./\\\\]{1,50}") {
        let step1 = tokmd_path::normalize_slashes(&path);
        let step2 = tokmd_path::normalize_rel_path(&step1);
        let step3 = tokmd_path::normalize_rel_path(&step2);
        prop_assert_eq!(&step2, &step3, "composed normalization not idempotent");
    }

    #[test]
    fn normalize_slashes_preserves_forward_slashes(path in "[a-z0-9_/]{1,50}") {
        let normalized = tokmd_path::normalize_slashes(&path);
        prop_assert_eq!(&path as &str, &normalized as &str, "forward-slash-only paths must be unchanged");
    }

    #[test]
    fn normalize_rel_path_output_has_no_backslash(path in "[a-z0-9_./\\\\]{1,50}") {
        let normalized = tokmd_path::normalize_rel_path(&path);
        prop_assert!(!normalized.contains('\\'), "backslash in normalized rel path: {normalized}");
    }
}

// =========================================================================
// 4. BTreeMap merge determinism
// =========================================================================

proptest! {
    #[test]
    fn btreemap_merge_order_independent(
        entries_a in proptest::collection::vec(("[a-z]{1,8}", 0u64..1000), 1..20),
        entries_b in proptest::collection::vec(("[a-z]{1,8}", 0u64..1000), 1..20),
    ) {
        // Merge A into B
        let mut map_ab: BTreeMap<String, u64> = entries_a.iter().cloned().collect();
        for (k, v) in &entries_b {
            map_ab.entry(k.clone()).or_insert(*v);
        }

        // Merge B into A
        let mut map_ba: BTreeMap<String, u64> = entries_b.iter().cloned().collect();
        for (k, v) in &entries_a {
            // Use insert to match "first writer wins" from map_ab
            map_ba.insert(k.clone(), *v);
        }

        let keys_ab: Vec<&String> = map_ab.keys().collect();
        let keys_ba: Vec<&String> = map_ba.keys().collect();
        prop_assert_eq!(keys_ab, keys_ba, "BTreeMap key order must be merge-order independent");
    }
}

// =========================================================================
// 5. Redaction determinism
// =========================================================================

proptest! {
    #[test]
    fn redact_short_hash_deterministic_on_repeated_calls(input in "[a-z0-9_/\\\\.]{1,60}") {
        let results: Vec<String> = (0..5).map(|_| tokmd_redact::short_hash(&input)).collect();
        for (i, r) in results.iter().enumerate().skip(1) {
            prop_assert_eq!(&results[0], r, "short_hash diverged on call {}", i);
        }
    }

    #[test]
    fn redact_path_preserves_extension(
        stem in "[a-z]{1,5}(/[a-z]{1,5}){0,3}",
        ext in "[a-z]{1,4}"
    ) {
        let path = format!("{stem}/file.{ext}");
        let redacted = tokmd_redact::redact_path(&path);
        prop_assert!(
            redacted.ends_with(&format!(".{ext}")),
            "extension not preserved: input={} redacted={}", path, redacted
        );
    }

    #[test]
    fn redact_cross_platform_path_equivalence(
        segments in proptest::collection::vec("[a-z]{1,8}", 2..5),
        ext in "[a-z]{1,3}"
    ) {
        let unix_path = format!("{}/file.{ext}", segments.join("/"));
        let win_path = format!("{}\\file.{ext}", segments.join("\\"));
        let h_unix = tokmd_redact::short_hash(&unix_path);
        let h_win = tokmd_redact::short_hash(&win_path);
        prop_assert_eq!(h_unix, h_win, "hash must be cross-platform consistent: unix={} win={}", unix_path, win_path);
    }
}
