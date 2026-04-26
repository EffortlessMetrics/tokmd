//! Depth tests for tokmd-sensor::substrate (w58).
//!
//! Covers construction, serde roundtrips, filtering, determinism,
//! edge cases (empty, minimal), and property-based invariants.

use std::collections::BTreeMap;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

fn make_file(path: &str, lang: &str, code: usize, in_diff: bool) -> SubstrateFile {
    SubstrateFile {
        path: path.to_string(),
        lang: lang.to_string(),
        code,
        lines: code + 20,
        bytes: code * 30,
        tokens: code * 3,
        module: path
            .rsplit_once('/')
            .map(|(dir, _)| dir)
            .unwrap_or(".")
            .to_string(),
        in_diff,
    }
}

fn make_lang(files: usize, code: usize) -> LangSummary {
    LangSummary {
        files,
        code,
        lines: code + files * 20,
        bytes: code * 30,
        tokens: code * 3,
    }
}

fn sample_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/my/repo".to_string(),
        files: vec![
            make_file("src/lib.rs", "Rust", 200, true),
            make_file("src/main.rs", "Rust", 50, false),
            make_file("tests/test.py", "Python", 80, true),
        ],
        lang_summary: BTreeMap::from([
            ("Python".to_string(), make_lang(1, 80)),
            ("Rust".to_string(), make_lang(2, 250)),
        ]),
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "feature-x".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "tests/test.py".to_string()],
            commit_count: 5,
            insertions: 42,
            deletions: 10,
        }),
        total_tokens: 990,
        total_bytes: 9900,
        total_code_lines: 330,
    }
}

// ── Construction ────────────────────────────────────────────────────

#[test]
fn substrate_stores_repo_root() {
    let s = sample_substrate();
    assert_eq!(s.repo_root, "/my/repo");
}

#[test]
fn substrate_file_count_matches() {
    let s = sample_substrate();
    assert_eq!(s.files.len(), 3);
}

#[test]
fn substrate_lang_summary_keys_sorted() {
    let s = sample_substrate();
    let keys: Vec<_> = s.lang_summary.keys().cloned().collect();
    assert_eq!(keys, vec!["Python", "Rust"]);
}

// ── Filtering methods ───────────────────────────────────────────────

#[test]
fn diff_files_returns_only_in_diff() {
    let s = sample_substrate();
    let diff: Vec<_> = s.diff_files().collect();
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().all(|f| f.in_diff));
}

#[test]
fn diff_files_empty_when_none_in_diff() {
    let s = RepoSubstrate {
        files: vec![make_file("a.rs", "Rust", 10, false)],
        ..sample_substrate()
    };
    assert_eq!(s.diff_files().count(), 0);
}

#[test]
fn files_for_lang_filters_correctly() {
    let s = sample_substrate();
    let rust: Vec<_> = s.files_for_lang("Rust").collect();
    assert_eq!(rust.len(), 2);
    assert!(rust.iter().all(|f| f.lang == "Rust"));
}

#[test]
fn files_for_lang_nonexistent_returns_empty() {
    let s = sample_substrate();
    assert_eq!(s.files_for_lang("Go").count(), 0);
}

#[test]
fn files_for_lang_case_sensitive() {
    let s = sample_substrate();
    assert_eq!(s.files_for_lang("rust").count(), 0);
    assert_eq!(s.files_for_lang("Rust").count(), 2);
}

// ── Serde roundtrips ────────────────────────────────────────────────

#[test]
fn serde_json_roundtrip_full() {
    let s = sample_substrate();
    let json = serde_json::to_string_pretty(&s).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.repo_root, s.repo_root);
    assert_eq!(back.files.len(), s.files.len());
    assert_eq!(back.total_tokens, s.total_tokens);
    assert_eq!(back.total_bytes, s.total_bytes);
    assert_eq!(back.total_code_lines, s.total_code_lines);
    assert_eq!(back.lang_summary.len(), s.lang_summary.len());
}

#[test]
fn serde_roundtrip_without_diff_range() {
    let s = RepoSubstrate {
        diff_range: None,
        ..sample_substrate()
    };
    let json = serde_json::to_string(&s).unwrap();
    // diff_range should be omitted from JSON when None
    assert!(!json.contains("diff_range"));
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.diff_range.is_none());
}

#[test]
fn serde_preserves_diff_range_fields() {
    let s = sample_substrate();
    let json = serde_json::to_string(&s).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    let dr = back.diff_range.unwrap();
    assert_eq!(dr.base, "main");
    assert_eq!(dr.head, "feature-x");
    assert_eq!(dr.changed_files.len(), 2);
    assert_eq!(dr.commit_count, 5);
    assert_eq!(dr.insertions, 42);
    assert_eq!(dr.deletions, 10);
}

#[test]
fn serde_preserves_file_fields() {
    let s = sample_substrate();
    let json = serde_json::to_string(&s).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    let f = &back.files[0];
    assert_eq!(f.path, "src/lib.rs");
    assert_eq!(f.lang, "Rust");
    assert_eq!(f.code, 200);
    assert_eq!(f.module, "src");
    assert!(f.in_diff);
}

// ── Empty / minimal ─────────────────────────────────────────────────

#[test]
fn empty_substrate() {
    let s = RepoSubstrate {
        repo_root: String::new(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };
    assert_eq!(s.diff_files().count(), 0);
    assert_eq!(s.files_for_lang("Rust").count(), 0);
    let json = serde_json::to_string(&s).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.files.is_empty());
}

#[test]
fn single_file_substrate() {
    let s = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![make_file("main.go", "Go", 100, false)],
        lang_summary: BTreeMap::from([("Go".to_string(), make_lang(1, 100))]),
        diff_range: None,
        total_tokens: 300,
        total_bytes: 3000,
        total_code_lines: 100,
    };
    assert_eq!(s.files_for_lang("Go").count(), 1);
    assert_eq!(s.diff_files().count(), 0);
}

// ── Determinism ─────────────────────────────────────────────────────

#[test]
fn btreemap_key_order_deterministic() {
    let mut summary = BTreeMap::new();
    summary.insert("Zig".to_string(), make_lang(1, 10));
    summary.insert("Ada".to_string(), make_lang(1, 20));
    summary.insert("Rust".to_string(), make_lang(3, 300));
    summary.insert("C".to_string(), make_lang(2, 100));
    let keys: Vec<_> = summary.keys().cloned().collect();
    assert_eq!(keys, vec!["Ada", "C", "Rust", "Zig"]);
}

#[test]
fn serialization_is_deterministic() {
    let s1 = sample_substrate();
    let s2 = sample_substrate();
    let json1 = serde_json::to_string(&s1).unwrap();
    let json2 = serde_json::to_string(&s2).unwrap();
    assert_eq!(json1, json2);
}

// ── Clone ───────────────────────────────────────────────────────────

#[test]
fn clone_is_independent() {
    let s = sample_substrate();
    let mut cloned = s.clone();
    cloned.repo_root = "/other".to_string();
    cloned.files.push(make_file("extra.rs", "Rust", 10, false));
    assert_eq!(s.repo_root, "/my/repo");
    assert_eq!(s.files.len(), 3);
}

// ── Property-based tests ────────────────────────────────────────────

mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn diff_files_count_le_total(
            n_files in 1_usize..20,
            n_diff in 0_usize..20,
        ) {
            let n_diff = n_diff.min(n_files);
            let files: Vec<SubstrateFile> = (0..n_files)
                .map(|i| make_file(
                    &format!("f{i}.rs"),
                    "Rust",
                    10,
                    i < n_diff,
                ))
                .collect();
            let s = RepoSubstrate {
                repo_root: "/r".into(),
                files,
                lang_summary: BTreeMap::new(),
                diff_range: None,
                total_tokens: 0,
                total_bytes: 0,
                total_code_lines: 0,
            };
            prop_assert!(s.diff_files().count() <= s.files.len());
            prop_assert_eq!(s.diff_files().count(), n_diff);
        }

        #[test]
        fn serde_roundtrip_preserves_file_count(n in 0_usize..30) {
            let files: Vec<SubstrateFile> = (0..n)
                .map(|i| make_file(&format!("f{i}.rs"), "Rust", i * 10, false))
                .collect();
            let s = RepoSubstrate {
                repo_root: "/r".into(),
                files,
                lang_summary: BTreeMap::new(),
                diff_range: None,
                total_tokens: 0,
                total_bytes: 0,
                total_code_lines: 0,
            };
            let json = serde_json::to_string(&s).unwrap();
            let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(back.files.len(), n);
        }
    }
}
