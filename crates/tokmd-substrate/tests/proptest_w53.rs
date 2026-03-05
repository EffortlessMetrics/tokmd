//! W53: Property-based tests for `tokmd-substrate` field invariants.
//!
//! Covers: serde roundtrip, diff_files/files_for_lang consistency,
//! BTreeMap ordering, boundary conditions, and determinism.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ── Strategies ──────────────────────────────────────────────────────────

fn arb_lang() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("Rust".to_string()),
        Just("Python".to_string()),
        Just("JavaScript".to_string()),
        Just("Go".to_string()),
        Just("TypeScript".to_string()),
    ]
}

fn arb_substrate_file() -> impl Strategy<Value = SubstrateFile> {
    (
        "[a-z]{1,4}(/[a-z]{1,8}){0,3}/[a-z]{1,8}\\.[a-z]{1,4}",
        arb_lang(),
        0usize..10_000,
        0usize..10_000,
        0usize..100_000,
        0usize..50_000,
        any::<bool>(),
    )
        .prop_map(|(path, lang, code, lines, bytes, tokens, in_diff)| {
            let module = path.split('/').next().unwrap_or("(root)").to_string();
            SubstrateFile {
                path,
                lang,
                code,
                lines,
                bytes,
                tokens,
                module,
                in_diff,
            }
        })
}

fn arb_diff_range() -> impl Strategy<Value = DiffRange> {
    (
        "[a-z]{1,10}",
        "[a-z]{1,10}",
        proptest::collection::vec("[a-z/]{1,30}", 0..8),
        0usize..100,
        0usize..1000,
        0usize..1000,
    )
        .prop_map(
            |(base, head, changed_files, commit_count, insertions, deletions)| DiffRange {
                base,
                head,
                changed_files,
                commit_count,
                insertions,
                deletions,
            },
        )
}

fn arb_substrate(files: Vec<SubstrateFile>) -> RepoSubstrate {
    let mut lang_summary = BTreeMap::new();
    for f in &files {
        let entry = lang_summary
            .entry(f.lang.clone())
            .or_insert(LangSummary {
                files: 0,
                code: 0,
                lines: 0,
                bytes: 0,
                tokens: 0,
            });
        entry.files += 1;
        entry.code += f.code;
        entry.lines += f.lines;
        entry.bytes += f.bytes;
        entry.tokens += f.tokens;
    }
    let total_tokens = files.iter().map(|f| f.tokens).sum();
    let total_bytes = files.iter().map(|f| f.bytes).sum();
    let total_code_lines = files.iter().map(|f| f.code).sum();

    RepoSubstrate {
        repo_root: "/repo".to_string(),
        files,
        lang_summary,
        diff_range: None,
        total_tokens,
        total_bytes,
        total_code_lines,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(120))]

    // 1. Serde roundtrip preserves all fields
    #[test]
    fn serde_roundtrip(files in proptest::collection::vec(arb_substrate_file(), 0..20)) {
        let sub = arb_substrate(files);
        let json = serde_json::to_string(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sub.files.len(), back.files.len());
        prop_assert_eq!(sub.total_tokens, back.total_tokens);
        prop_assert_eq!(sub.total_bytes, back.total_bytes);
        prop_assert_eq!(sub.total_code_lines, back.total_code_lines);
        prop_assert_eq!(sub.lang_summary.len(), back.lang_summary.len());
    }

    // 2. Roundtrip with diff_range present
    #[test]
    fn serde_roundtrip_with_diff(
        files in proptest::collection::vec(arb_substrate_file(), 1..10),
        diff in arb_diff_range(),
    ) {
        let mut sub = arb_substrate(files);
        sub.diff_range = Some(diff);
        let json = serde_json::to_string(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
        prop_assert!(back.diff_range.is_some());
        let back_diff = back.diff_range.unwrap();
        prop_assert_eq!(&sub.diff_range.as_ref().unwrap().base, &back_diff.base);
        prop_assert_eq!(&sub.diff_range.as_ref().unwrap().head, &back_diff.head);
    }

    // 3. diff_files returns only in_diff=true files
    #[test]
    fn diff_files_filter_correct(files in proptest::collection::vec(arb_substrate_file(), 0..30)) {
        let sub = arb_substrate(files);
        let diff_count = sub.diff_files().count();
        let expected = sub.files.iter().filter(|f| f.in_diff).count();
        prop_assert_eq!(diff_count, expected);
    }

    // 4. files_for_lang returns only matching language
    #[test]
    fn files_for_lang_correct(
        files in proptest::collection::vec(arb_substrate_file(), 0..30),
        lang in arb_lang(),
    ) {
        let sub = arb_substrate(files);
        let count = sub.files_for_lang(&lang).count();
        let expected = sub.files.iter().filter(|f| f.lang == lang).count();
        prop_assert_eq!(count, expected);
    }

    // 5. lang_summary keys are sorted (BTreeMap guarantee)
    #[test]
    fn lang_summary_keys_sorted(files in proptest::collection::vec(arb_substrate_file(), 1..20)) {
        let sub = arb_substrate(files);
        let keys: Vec<_> = sub.lang_summary.keys().cloned().collect();
        let mut sorted = keys.clone();
        sorted.sort();
        prop_assert_eq!(keys, sorted);
    }

    // 6. total_tokens equals sum of file tokens
    #[test]
    fn total_tokens_consistency(files in proptest::collection::vec(arb_substrate_file(), 0..20)) {
        let sub = arb_substrate(files);
        let sum: usize = sub.files.iter().map(|f| f.tokens).sum();
        prop_assert_eq!(sub.total_tokens, sum);
    }

    // 7. total_bytes equals sum of file bytes
    #[test]
    fn total_bytes_consistency(files in proptest::collection::vec(arb_substrate_file(), 0..20)) {
        let sub = arb_substrate(files);
        let sum: usize = sub.files.iter().map(|f| f.bytes).sum();
        prop_assert_eq!(sub.total_bytes, sum);
    }

    // 8. total_code_lines equals sum of file code
    #[test]
    fn total_code_lines_consistency(files in proptest::collection::vec(arb_substrate_file(), 0..20)) {
        let sub = arb_substrate(files);
        let sum: usize = sub.files.iter().map(|f| f.code).sum();
        prop_assert_eq!(sub.total_code_lines, sum);
    }

    // 9. Empty substrate: all totals zero
    #[test]
    fn empty_substrate_zero_totals(_dummy in 0..1u8) {
        let sub = arb_substrate(vec![]);
        prop_assert_eq!(sub.total_tokens, 0);
        prop_assert_eq!(sub.total_bytes, 0);
        prop_assert_eq!(sub.total_code_lines, 0);
        prop_assert!(sub.lang_summary.is_empty());
        prop_assert!(sub.diff_files().next().is_none());
    }

    // 10. lang_summary file count matches files_for_lang count
    #[test]
    fn lang_summary_file_count_matches(files in proptest::collection::vec(arb_substrate_file(), 1..20)) {
        let sub = arb_substrate(files);
        for (lang, summary) in &sub.lang_summary {
            let actual = sub.files_for_lang(lang).count();
            prop_assert_eq!(summary.files, actual, "mismatch for lang {}", lang);
        }
    }

    // 11. Serialization is deterministic
    #[test]
    fn serialization_deterministic(files in proptest::collection::vec(arb_substrate_file(), 0..10)) {
        let sub = arb_substrate(files);
        let a = serde_json::to_string(&sub).unwrap();
        let b = serde_json::to_string(&sub).unwrap();
        prop_assert_eq!(a, b);
    }

    // 12. DiffRange roundtrip
    #[test]
    fn diff_range_roundtrip(diff in arb_diff_range()) {
        let json = serde_json::to_string(&diff).unwrap();
        let back: DiffRange = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&diff.base, &back.base);
        prop_assert_eq!(&diff.head, &back.head);
        prop_assert_eq!(diff.changed_files.len(), back.changed_files.len());
        prop_assert_eq!(diff.commit_count, back.commit_count);
        prop_assert_eq!(diff.insertions, back.insertions);
        prop_assert_eq!(diff.deletions, back.deletions);
    }
}
