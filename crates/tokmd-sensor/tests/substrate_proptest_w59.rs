//! Property-based tests for `tokmd-sensor::substrate` invariants.
//!
//! Verifies serde roundtrips, totals consistency, and structural
//! invariants for arbitrary substrate inputs.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ── Strategies ───────────────────────────────────────────────────

fn arb_lang_summary() -> impl Strategy<Value = LangSummary> {
    (
        0..100usize,
        0..10_000usize,
        0..10_000usize,
        0..1_000_000usize,
        0..100_000usize,
    )
        .prop_map(|(files, code, lines, bytes, tokens)| LangSummary {
            files,
            code,
            lines,
            bytes,
            tokens,
        })
}

fn arb_substrate_file() -> impl Strategy<Value = SubstrateFile> {
    (
        "[a-z]+(/[a-z]+){0,3}\\.[a-z]{1,4}",
        "[A-Z][a-z]+",
        0usize..5_000,
        0usize..10_000,
        0usize..500_000,
        0usize..125_000,
        "[a-z]+(/[a-z]+){0,2}",
        any::<bool>(),
    )
        .prop_map(
            |(path, lang, code, lines, bytes, tokens, module, in_diff)| SubstrateFile {
                path,
                lang,
                code,
                lines,
                bytes,
                tokens,
                module,
                in_diff,
            },
        )
}

fn arb_diff_range() -> impl Strategy<Value = DiffRange> {
    (
        "[a-z]+",
        "[a-z]+",
        proptest::collection::vec("[a-z/]+\\.[a-z]{1,4}", 0..10),
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

fn arb_substrate_consistent() -> impl Strategy<Value = RepoSubstrate> {
    proptest::collection::vec(arb_substrate_file(), 0..15).prop_map(|files| {
        let mut lang_summary: BTreeMap<String, LangSummary> = BTreeMap::new();
        for f in &files {
            let e = lang_summary.entry(f.lang.clone()).or_insert(LangSummary {
                files: 0,
                code: 0,
                lines: 0,
                bytes: 0,
                tokens: 0,
            });
            e.files += 1;
            e.code += f.code;
            e.lines += f.lines;
            e.bytes += f.bytes;
            e.tokens += f.tokens;
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
    })
}

// ── Property tests ───────────────────────────────────────────────

proptest! {
    #[test]
    fn substrate_file_serde_roundtrip(file in arb_substrate_file()) {
        let json = serde_json::to_string(&file).unwrap();
        let back: SubstrateFile = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.path, file.path);
        prop_assert_eq!(back.lang, file.lang);
        prop_assert_eq!(back.code, file.code);
        prop_assert_eq!(back.lines, file.lines);
        prop_assert_eq!(back.bytes, file.bytes);
        prop_assert_eq!(back.tokens, file.tokens);
        prop_assert_eq!(back.module, file.module);
        prop_assert_eq!(back.in_diff, file.in_diff);
    }

    #[test]
    fn lang_summary_serde_roundtrip(ls in arb_lang_summary()) {
        let json = serde_json::to_string(&ls).unwrap();
        let back: LangSummary = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.files, ls.files);
        prop_assert_eq!(back.code, ls.code);
        prop_assert_eq!(back.lines, ls.lines);
        prop_assert_eq!(back.bytes, ls.bytes);
        prop_assert_eq!(back.tokens, ls.tokens);
    }

    #[test]
    fn diff_range_serde_roundtrip(dr in arb_diff_range()) {
        let json = serde_json::to_string(&dr).unwrap();
        let back: DiffRange = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.base, dr.base);
        prop_assert_eq!(back.head, dr.head);
        prop_assert_eq!(back.changed_files.len(), dr.changed_files.len());
        prop_assert_eq!(back.commit_count, dr.commit_count);
        prop_assert_eq!(back.insertions, dr.insertions);
        prop_assert_eq!(back.deletions, dr.deletions);
    }

    #[test]
    fn substrate_serde_roundtrip(sub in arb_substrate_consistent()) {
        let json = serde_json::to_string(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.files.len(), sub.files.len());
        prop_assert_eq!(back.lang_summary.len(), sub.lang_summary.len());
        prop_assert_eq!(back.total_code_lines, sub.total_code_lines);
        prop_assert_eq!(back.total_tokens, sub.total_tokens);
        prop_assert_eq!(back.total_bytes, sub.total_bytes);
    }

    #[test]
    fn totals_equal_file_sums(sub in arb_substrate_consistent()) {
        let sum_code: usize = sub.files.iter().map(|f| f.code).sum();
        let sum_tokens: usize = sub.files.iter().map(|f| f.tokens).sum();
        let sum_bytes: usize = sub.files.iter().map(|f| f.bytes).sum();
        prop_assert_eq!(sub.total_code_lines, sum_code);
        prop_assert_eq!(sub.total_tokens, sum_tokens);
        prop_assert_eq!(sub.total_bytes, sum_bytes);
    }

    #[test]
    fn lang_summary_file_count_equals_file_vec_len(sub in arb_substrate_consistent()) {
        let summary_total: usize = sub.lang_summary.values().map(|l| l.files).sum();
        prop_assert_eq!(summary_total, sub.files.len());
    }

    #[test]
    fn lang_summary_code_equals_total(sub in arb_substrate_consistent()) {
        let summary_code: usize = sub.lang_summary.values().map(|l| l.code).sum();
        prop_assert_eq!(summary_code, sub.total_code_lines);
    }

    #[test]
    fn diff_files_subset_of_files(sub in arb_substrate_consistent()) {
        let diff_count = sub.diff_files().count();
        let in_diff_count = sub.files.iter().filter(|f| f.in_diff).count();
        prop_assert_eq!(diff_count, in_diff_count);
    }

    #[test]
    fn files_for_lang_partition(sub in arb_substrate_consistent()) {
        // files_for_lang for each known language should cover all files
        let mut total = 0usize;
        for lang in sub.lang_summary.keys() {
            total += sub.files_for_lang(lang).count();
        }
        prop_assert_eq!(total, sub.files.len());
    }

    #[test]
    fn clone_preserves_all_fields(sub in arb_substrate_consistent()) {
        let cloned = sub.clone();
        let j1 = serde_json::to_string(&sub).unwrap();
        let j2 = serde_json::to_string(&cloned).unwrap();
        prop_assert_eq!(j1, j2);
    }

    #[test]
    fn lang_summary_keys_sorted(sub in arb_substrate_consistent()) {
        let keys: Vec<&String> = sub.lang_summary.keys().collect();
        for w in keys.windows(2) {
            prop_assert!(w[0] <= w[1], "BTreeMap keys must be sorted");
        }
    }

    #[test]
    fn empty_files_means_zero_totals(
        root in "[a-z/]+",
    ) {
        let sub = RepoSubstrate {
            repo_root: root,
            files: vec![],
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: 0,
        };
        prop_assert_eq!(sub.total_code_lines, 0);
        prop_assert_eq!(sub.total_tokens, 0);
        prop_assert_eq!(sub.total_bytes, 0);
        prop_assert!(sub.diff_files().next().is_none());
    }
}
