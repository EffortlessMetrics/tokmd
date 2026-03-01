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

    // ── Deep round-trip ──────────────────────────────────────────

    /// Deep field-by-field round-trip for every nested structure.
    #[test]
    fn repo_substrate_deep_roundtrip(sub in arb_repo_substrate()) {
        let json = serde_json::to_string(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();

        for (orig, rest) in sub.files.iter().zip(back.files.iter()) {
            prop_assert_eq!(&rest.path, &orig.path);
            prop_assert_eq!(&rest.lang, &orig.lang);
            prop_assert_eq!(rest.code, orig.code);
            prop_assert_eq!(rest.lines, orig.lines);
            prop_assert_eq!(rest.bytes, orig.bytes);
            prop_assert_eq!(rest.tokens, orig.tokens);
            prop_assert_eq!(&rest.module, &orig.module);
            prop_assert_eq!(rest.in_diff, orig.in_diff);
        }

        for (key, orig_ls) in &sub.lang_summary {
            let rest_ls = back.lang_summary.get(key).unwrap();
            prop_assert_eq!(rest_ls.files, orig_ls.files);
            prop_assert_eq!(rest_ls.code, orig_ls.code);
            prop_assert_eq!(rest_ls.lines, orig_ls.lines);
            prop_assert_eq!(rest_ls.bytes, orig_ls.bytes);
            prop_assert_eq!(rest_ls.tokens, orig_ls.tokens);
        }

        match (&sub.diff_range, &back.diff_range) {
            (Some(orig), Some(rest)) => {
                prop_assert_eq!(&rest.base, &orig.base);
                prop_assert_eq!(&rest.head, &orig.head);
                prop_assert_eq!(&rest.changed_files, &orig.changed_files);
                prop_assert_eq!(rest.commit_count, orig.commit_count);
                prop_assert_eq!(rest.insertions, orig.insertions);
                prop_assert_eq!(rest.deletions, orig.deletions);
            }
            (None, None) => {}
            _ => prop_assert!(false, "diff_range presence mismatch"),
        }
    }

    // ── Path normalization ───────────────────────────────────────

    /// Strategy-generated paths never contain backslashes.
    #[test]
    fn paths_contain_no_backslashes(sub in arb_repo_substrate()) {
        prop_assert!(
            !sub.repo_root.contains('\\'),
            "repo_root contains backslash: {}",
            sub.repo_root
        );
        for f in &sub.files {
            prop_assert!(
                !f.path.contains('\\'),
                "file path contains backslash: {}",
                f.path
            );
            prop_assert!(
                !f.module.contains('\\'),
                "module contains backslash: {}",
                f.module
            );
        }
        if let Some(dr) = &sub.diff_range {
            for cf in &dr.changed_files {
                prop_assert!(
                    !cf.contains('\\'),
                    "changed_file contains backslash: {}",
                    cf
                );
            }
        }
    }

    /// Backslash-containing paths survive serde unchanged,
    /// confirming normalization is the caller's responsibility.
    #[test]
    fn backslash_paths_preserved_through_serde(
        path in r"[a-z\\]{2,30}",
    ) {
        let file = SubstrateFile {
            path: path.clone(),
            lang: "Rust".to_string(),
            code: 10,
            lines: 20,
            bytes: 300,
            tokens: 70,
            module: String::new(),
            in_diff: false,
        };
        let json = serde_json::to_string(&file).unwrap();
        let back: SubstrateFile = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.path, &path);
    }

    // ── Idempotent serialization ─────────────────────────────────

    /// serialize → deserialize → serialize yields identical JSON.
    #[test]
    fn serialization_idempotent(sub in arb_repo_substrate()) {
        let json1 = serde_json::to_string(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&back).unwrap();
        prop_assert_eq!(json1, json2);
    }

    /// Pretty-printed JSON also round-trips identically.
    #[test]
    fn pretty_json_roundtrip(sub in arb_repo_substrate()) {
        let json1 = serde_json::to_string_pretty(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string_pretty(&back).unwrap();
        prop_assert_eq!(json1, json2);
    }

    // ── Default / empty substrate ────────────────────────────────

    /// An empty substrate with zero values is valid for all accessors.
    #[test]
    fn empty_substrate_valid(root in "[a-z/]{1,30}") {
        let sub = RepoSubstrate {
            repo_root: root,
            files: vec![],
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: 0,
        };
        prop_assert_eq!(sub.diff_files().count(), 0);
        prop_assert_eq!(sub.files_for_lang("Rust").count(), 0);
        prop_assert!(sub.files.is_empty());
        prop_assert!(sub.lang_summary.is_empty());
        // Must survive round-trip
        let json = serde_json::to_string(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
        prop_assert!(back.files.is_empty());
        prop_assert!(back.diff_range.is_none());
    }

    // ── Partition properties ─────────────────────────────────────

    /// Every file belongs to exactly one files_for_lang partition.
    #[test]
    fn files_for_lang_partitions_all(sub in arb_repo_substrate()) {
        let unique_langs: std::collections::BTreeSet<&str> =
            sub.files.iter().map(|f| f.lang.as_str()).collect();
        let mut covered = 0usize;
        for lang in &unique_langs {
            covered += sub.files_for_lang(lang).count();
        }
        prop_assert_eq!(covered, sub.files.len());
    }

    /// diff_files + non-diff files = total file count.
    #[test]
    fn diff_non_diff_partition(sub in arb_repo_substrate()) {
        let diff_count = sub.diff_files().count();
        let non_diff_count = sub.files.iter().filter(|f| !f.in_diff).count();
        prop_assert_eq!(diff_count + non_diff_count, sub.files.len());
    }

    /// files_for_lang with a non-existent language always returns empty.
    #[test]
    fn files_for_nonexistent_lang_empty(
        sub in arb_repo_substrate(),
        fake_lang in "ZZZZZ_[0-9]{5}",
    ) {
        prop_assert_eq!(sub.files_for_lang(&fake_lang).count(), 0);
    }

    // ── JSON structural properties ───────────────────────────────

    /// JSON output is always valid JSON (parseable as serde_json::Value).
    #[test]
    fn json_output_is_valid(sub in arb_repo_substrate()) {
        let json = serde_json::to_string(&sub).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        prop_assert!(val.is_object());
        prop_assert!(val.get("repo_root").is_some());
        prop_assert!(val.get("files").unwrap().is_array());
        prop_assert!(val.get("lang_summary").unwrap().is_object());
    }

    /// File count in JSON array matches struct field.
    #[test]
    fn json_file_array_len_matches(sub in arb_repo_substrate()) {
        let json = serde_json::to_string(&sub).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        let arr = val.get("files").unwrap().as_array().unwrap();
        prop_assert_eq!(arr.len(), sub.files.len());
    }

    /// lang_summary key count in JSON matches struct field.
    #[test]
    fn json_lang_summary_key_count_matches(sub in arb_repo_substrate()) {
        let json = serde_json::to_string(&sub).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = val.get("lang_summary").unwrap().as_object().unwrap();
        prop_assert_eq!(obj.len(), sub.lang_summary.len());
    }
}
