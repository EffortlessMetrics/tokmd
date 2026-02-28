//! Property-based tests for `tokmd-substrate` types.
//!
//! Uses `proptest` to verify serialization round-trips and
//! structural invariants hold for arbitrary inputs.

use std::collections::BTreeMap;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

use proptest::prelude::*;

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
        "[a-z/]{1,50}",    // path
        "[A-Za-z]{1,20}",  // lang
        0..10_000usize,    // code
        0..10_000usize,    // lines
        0..1_000_000usize, // bytes
        0..100_000usize,   // tokens
        "[a-z/]{0,30}",    // module
        any::<bool>(),     // in_diff
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
        "[a-z0-9./-]{1,30}",                           // base
        "[a-z0-9./-]{1,30}",                           // head
        prop::collection::vec("[a-z/.]{1,40}", 0..10), // changed_files
        0..500usize,                                   // commit_count
        0..10_000usize,                                // insertions
        0..10_000usize,                                // deletions
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

fn arb_lang_summary_map() -> impl Strategy<Value = BTreeMap<String, LangSummary>> {
    prop::collection::btree_map("[A-Za-z]{1,15}", arb_lang_summary(), 0..5)
}

fn arb_repo_substrate() -> impl Strategy<Value = RepoSubstrate> {
    (
        "[a-z/]{1,50}",                                     // repo_root
        prop::collection::vec(arb_substrate_file(), 0..10), // files
        arb_lang_summary_map(),                             // lang_summary
        prop::option::of(arb_diff_range()),                 // diff_range
        0..1_000_000usize,                                  // total_tokens
        0..10_000_000usize,                                 // total_bytes
        0..1_000_000usize,                                  // total_code_lines
    )
        .prop_map(
            |(
                repo_root,
                files,
                lang_summary,
                diff_range,
                total_tokens,
                total_bytes,
                total_code_lines,
            )| {
                RepoSubstrate {
                    repo_root,
                    files,
                    lang_summary,
                    diff_range,
                    total_tokens,
                    total_bytes,
                    total_code_lines,
                }
            },
        )
}

// ── Properties ───────────────────────────────────────────────────

proptest! {
    /// SubstrateFile survives JSON round-trip.
    #[test]
    fn substrate_file_roundtrip(file in arb_substrate_file()) {
        let json = serde_json::to_string(&file).unwrap();
        let back: SubstrateFile = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.path, &file.path);
        prop_assert_eq!(&back.lang, &file.lang);
        prop_assert_eq!(back.code, file.code);
        prop_assert_eq!(back.lines, file.lines);
        prop_assert_eq!(back.bytes, file.bytes);
        prop_assert_eq!(back.tokens, file.tokens);
        prop_assert_eq!(&back.module, &file.module);
        prop_assert_eq!(back.in_diff, file.in_diff);
    }

    /// LangSummary survives JSON round-trip.
    #[test]
    fn lang_summary_roundtrip(ls in arb_lang_summary()) {
        let json = serde_json::to_string(&ls).unwrap();
        let back: LangSummary = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.files, ls.files);
        prop_assert_eq!(back.code, ls.code);
        prop_assert_eq!(back.lines, ls.lines);
        prop_assert_eq!(back.bytes, ls.bytes);
        prop_assert_eq!(back.tokens, ls.tokens);
    }

    /// DiffRange survives JSON round-trip.
    #[test]
    fn diff_range_roundtrip(dr in arb_diff_range()) {
        let json = serde_json::to_string(&dr).unwrap();
        let back: DiffRange = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.base, &dr.base);
        prop_assert_eq!(&back.head, &dr.head);
        prop_assert_eq!(&back.changed_files, &dr.changed_files);
        prop_assert_eq!(back.commit_count, dr.commit_count);
        prop_assert_eq!(back.insertions, dr.insertions);
        prop_assert_eq!(back.deletions, dr.deletions);
    }

    /// Full RepoSubstrate survives JSON round-trip.
    #[test]
    fn repo_substrate_roundtrip(sub in arb_repo_substrate()) {
        let json = serde_json::to_string(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.repo_root, &sub.repo_root);
        prop_assert_eq!(back.files.len(), sub.files.len());
        prop_assert_eq!(back.lang_summary.len(), sub.lang_summary.len());
        prop_assert_eq!(back.total_tokens, sub.total_tokens);
        prop_assert_eq!(back.total_bytes, sub.total_bytes);
        prop_assert_eq!(back.total_code_lines, sub.total_code_lines);
        prop_assert_eq!(back.diff_range.is_some(), sub.diff_range.is_some());
    }

    /// diff_files() count never exceeds total file count.
    #[test]
    fn diff_files_count_le_total(sub in arb_repo_substrate()) {
        prop_assert!(sub.diff_files().count() <= sub.files.len());
    }

    /// diff_files() returns only files with in_diff == true.
    #[test]
    fn diff_files_all_in_diff(sub in arb_repo_substrate()) {
        for f in sub.diff_files() {
            prop_assert!(f.in_diff);
        }
    }

    /// files_for_lang() returns only files matching the language.
    #[test]
    fn files_for_lang_correct(sub in arb_repo_substrate(), lang in "[A-Za-z]{1,10}") {
        for f in sub.files_for_lang(&lang) {
            prop_assert_eq!(&f.lang, &lang);
        }
    }

    /// lang_summary keys are always sorted (BTreeMap invariant).
    #[test]
    fn lang_summary_keys_sorted(sub in arb_repo_substrate()) {
        let keys: Vec<_> = sub.lang_summary.keys().collect();
        for w in keys.windows(2) {
            prop_assert!(w[0] <= w[1], "BTreeMap keys should be sorted");
        }
    }

    /// When diff_range is None, JSON output must not contain "diff_range".
    #[test]
    fn no_diff_range_skipped_in_json(sub in arb_repo_substrate()) {
        if sub.diff_range.is_none() {
            let json = serde_json::to_string(&sub).unwrap();
            prop_assert!(!json.contains("diff_range"));
        }
    }

    /// Clone produces identical data.
    #[test]
    fn clone_identical(sub in arb_repo_substrate()) {
        let cloned = sub.clone();
        let j1 = serde_json::to_string(&sub).unwrap();
        let j2 = serde_json::to_string(&cloned).unwrap();
        prop_assert_eq!(j1, j2);
    }
}
