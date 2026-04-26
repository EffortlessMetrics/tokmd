//! W74 – Substrate integration tests.
//!
//! Tests `RepoSubstrate` construction, field population, serialization
//! roundtrip, and edge cases (empty repo, multi-language, diff filtering).

use std::collections::BTreeMap;

use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn single_lang_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/project".to_string(),
        files: vec![
            SubstrateFile {
                path: "src/lib.rs".to_string(),
                lang: "Rust".to_string(),
                code: 150,
                lines: 200,
                bytes: 4500,
                tokens: 1125,
                module: "src".to_string(),
                in_diff: true,
            },
            SubstrateFile {
                path: "src/util.rs".to_string(),
                lang: "Rust".to_string(),
                code: 50,
                lines: 60,
                bytes: 1500,
                tokens: 375,
                module: "src".to_string(),
                in_diff: false,
            },
        ],
        lang_summary: BTreeMap::from([(
            "Rust".to_string(),
            LangSummary {
                files: 2,
                code: 200,
                lines: 260,
                bytes: 6000,
                tokens: 1500,
            },
        )]),
        diff_range: Some(DiffRange {
            base: "v1.0.0".to_string(),
            head: "v1.1.0".to_string(),
            changed_files: vec!["src/lib.rs".to_string()],
            commit_count: 5,
            insertions: 20,
            deletions: 8,
        }),
        total_tokens: 1500,
        total_bytes: 6000,
        total_code_lines: 200,
    }
}

fn multi_lang_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/polyglot".to_string(),
        files: vec![
            SubstrateFile {
                path: "src/lib.rs".to_string(),
                lang: "Rust".to_string(),
                code: 100,
                lines: 120,
                bytes: 3000,
                tokens: 750,
                module: "src".to_string(),
                in_diff: false,
            },
            SubstrateFile {
                path: "src/index.ts".to_string(),
                lang: "TypeScript".to_string(),
                code: 80,
                lines: 100,
                bytes: 2400,
                tokens: 600,
                module: "src".to_string(),
                in_diff: true,
            },
            SubstrateFile {
                path: "scripts/build.py".to_string(),
                lang: "Python".to_string(),
                code: 30,
                lines: 40,
                bytes: 900,
                tokens: 225,
                module: "scripts".to_string(),
                in_diff: false,
            },
        ],
        lang_summary: BTreeMap::from([
            (
                "Python".to_string(),
                LangSummary {
                    files: 1,
                    code: 30,
                    lines: 40,
                    bytes: 900,
                    tokens: 225,
                },
            ),
            (
                "Rust".to_string(),
                LangSummary {
                    files: 1,
                    code: 100,
                    lines: 120,
                    bytes: 3000,
                    tokens: 750,
                },
            ),
            (
                "TypeScript".to_string(),
                LangSummary {
                    files: 1,
                    code: 80,
                    lines: 100,
                    bytes: 2400,
                    tokens: 600,
                },
            ),
        ]),
        diff_range: None,
        total_tokens: 1575,
        total_bytes: 6300,
        total_code_lines: 210,
    }
}

fn empty_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/empty".to_string(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    }
}

// ---------------------------------------------------------------------------
// 1. Construction and field population
// ---------------------------------------------------------------------------

#[test]
fn substrate_repo_root_stored() {
    let sub = single_lang_substrate();
    assert_eq!(sub.repo_root, "/project");
}

#[test]
fn substrate_file_count_matches() {
    let sub = single_lang_substrate();
    assert_eq!(sub.files.len(), 2);
}

#[test]
fn substrate_totals_are_consistent() {
    let sub = single_lang_substrate();
    let sum_code: usize = sub.files.iter().map(|f| f.code).sum();
    let sum_tokens: usize = sub.files.iter().map(|f| f.tokens).sum();
    let sum_bytes: usize = sub.files.iter().map(|f| f.bytes).sum();
    assert_eq!(sub.total_code_lines, sum_code);
    assert_eq!(sub.total_tokens, sum_tokens);
    assert_eq!(sub.total_bytes, sum_bytes);
}

#[test]
fn substrate_lang_summary_aggregates_correctly() {
    let sub = single_lang_substrate();
    let rust = &sub.lang_summary["Rust"];
    assert_eq!(rust.files, 2);
    assert_eq!(rust.code, 200);
    assert_eq!(rust.lines, 260);
}

// ---------------------------------------------------------------------------
// 2. Diff filtering
// ---------------------------------------------------------------------------

#[test]
fn diff_files_returns_only_changed() {
    let sub = single_lang_substrate();
    let diff: Vec<_> = sub.diff_files().collect();
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].path, "src/lib.rs");
}

#[test]
fn diff_files_empty_when_no_changes() {
    let sub = empty_substrate();
    let diff: Vec<_> = sub.diff_files().collect();
    assert!(diff.is_empty());
}

#[test]
fn diff_range_fields_populated() {
    let sub = single_lang_substrate();
    let dr = sub.diff_range.as_ref().unwrap();
    assert_eq!(dr.base, "v1.0.0");
    assert_eq!(dr.head, "v1.1.0");
    assert_eq!(dr.commit_count, 5);
    assert_eq!(dr.insertions, 20);
    assert_eq!(dr.deletions, 8);
    assert_eq!(dr.changed_files.len(), 1);
}

// ---------------------------------------------------------------------------
// 3. Multi-language substrates
// ---------------------------------------------------------------------------

#[test]
fn multi_lang_summary_keys_sorted() {
    let sub = multi_lang_substrate();
    let keys: Vec<_> = sub.lang_summary.keys().cloned().collect();
    assert_eq!(keys, vec!["Python", "Rust", "TypeScript"]);
}

#[test]
fn files_for_lang_filters_correctly() {
    let sub = multi_lang_substrate();
    let ts: Vec<_> = sub.files_for_lang("TypeScript").collect();
    assert_eq!(ts.len(), 1);
    assert_eq!(ts[0].path, "src/index.ts");

    let go: Vec<_> = sub.files_for_lang("Go").collect();
    assert!(go.is_empty());
}

// ---------------------------------------------------------------------------
// 4. Empty substrate edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_substrate_has_zero_totals() {
    let sub = empty_substrate();
    assert_eq!(sub.total_code_lines, 0);
    assert_eq!(sub.total_tokens, 0);
    assert_eq!(sub.total_bytes, 0);
    assert!(sub.files.is_empty());
    assert!(sub.lang_summary.is_empty());
}

#[test]
fn empty_substrate_diff_range_is_none() {
    let sub = empty_substrate();
    assert!(sub.diff_range.is_none());
}

// ---------------------------------------------------------------------------
// 5. Serialization roundtrip
// ---------------------------------------------------------------------------

#[test]
fn serde_roundtrip_single_lang() {
    let sub = single_lang_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), sub.files.len());
    assert_eq!(back.total_code_lines, sub.total_code_lines);
    assert_eq!(back.repo_root, sub.repo_root);
    assert!(back.diff_range.is_some());
}

#[test]
fn serde_roundtrip_multi_lang() {
    let sub = multi_lang_substrate();
    let json = serde_json::to_string_pretty(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lang_summary.len(), 3);
    assert_eq!(back.files.len(), 3);
    assert!(back.diff_range.is_none());
}

#[test]
fn serde_roundtrip_empty() {
    let sub = empty_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.files.is_empty());
    assert_eq!(back.total_code_lines, 0);
    // diff_range should be omitted from JSON (skip_serializing_if)
    assert!(!json.contains("diff_range"));
}

#[test]
fn substrate_file_in_diff_preserved_through_serde() {
    let sub = single_lang_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    let in_diff_count = back.files.iter().filter(|f| f.in_diff).count();
    assert_eq!(in_diff_count, 1);
    assert!(back.files.iter().find(|f| f.in_diff).unwrap().path == "src/lib.rs");
}
