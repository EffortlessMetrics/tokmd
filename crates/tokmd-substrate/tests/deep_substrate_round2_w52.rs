//! Deep round-2 tests for tokmd-substrate (W52).
//!
//! Covers RepoSubstrate construction, field access, serialization,
//! determinism, and edge cases.

use std::collections::BTreeMap;

use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn single_file_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/project".to_string(),
        files: vec![SubstrateFile {
            path: "src/lib.rs".to_string(),
            lang: "Rust".to_string(),
            code: 100,
            lines: 120,
            bytes: 3000,
            tokens: 750,
            module: "src".to_string(),
            in_diff: false,
        }],
        lang_summary: BTreeMap::from([(
            "Rust".to_string(),
            LangSummary {
                files: 1,
                code: 100,
                lines: 120,
                bytes: 3000,
                tokens: 750,
            },
        )]),
        diff_range: None,
        total_tokens: 750,
        total_bytes: 3000,
        total_code_lines: 100,
    }
}

fn multi_lang_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/workspace".to_string(),
        files: vec![
            SubstrateFile {
                path: "src/lib.rs".to_string(),
                lang: "Rust".to_string(),
                code: 300,
                lines: 400,
                bytes: 9000,
                tokens: 2250,
                module: "src".to_string(),
                in_diff: true,
            },
            SubstrateFile {
                path: "src/main.rs".to_string(),
                lang: "Rust".to_string(),
                code: 50,
                lines: 70,
                bytes: 1500,
                tokens: 375,
                module: "src".to_string(),
                in_diff: false,
            },
            SubstrateFile {
                path: "lib/utils.py".to_string(),
                lang: "Python".to_string(),
                code: 200,
                lines: 250,
                bytes: 6000,
                tokens: 1500,
                module: "lib".to_string(),
                in_diff: true,
            },
            SubstrateFile {
                path: "web/app.ts".to_string(),
                lang: "TypeScript".to_string(),
                code: 150,
                lines: 180,
                bytes: 4500,
                tokens: 1125,
                module: "web".to_string(),
                in_diff: false,
            },
        ],
        lang_summary: BTreeMap::from([
            (
                "Python".to_string(),
                LangSummary {
                    files: 1,
                    code: 200,
                    lines: 250,
                    bytes: 6000,
                    tokens: 1500,
                },
            ),
            (
                "Rust".to_string(),
                LangSummary {
                    files: 2,
                    code: 350,
                    lines: 470,
                    bytes: 10500,
                    tokens: 2625,
                },
            ),
            (
                "TypeScript".to_string(),
                LangSummary {
                    files: 1,
                    code: 150,
                    lines: 180,
                    bytes: 4500,
                    tokens: 1125,
                },
            ),
        ]),
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "feature".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "lib/utils.py".to_string()],
            commit_count: 5,
            insertions: 30,
            deletions: 10,
        }),
        total_tokens: 5250,
        total_bytes: 21000,
        total_code_lines: 700,
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
// Tests: Construction and field access
// ---------------------------------------------------------------------------

#[test]
fn substrate_repo_root_preserved() {
    let sub = single_file_substrate();
    assert_eq!(sub.repo_root, "/project");
}

#[test]
fn substrate_files_accessible() {
    let sub = single_file_substrate();
    assert_eq!(sub.files.len(), 1);
    assert_eq!(sub.files[0].path, "src/lib.rs");
    assert_eq!(sub.files[0].lang, "Rust");
    assert_eq!(sub.files[0].code, 100);
}

#[test]
fn substrate_totals_consistent_with_files() {
    let sub = multi_lang_substrate();
    let file_code_sum: usize = sub.files.iter().map(|f| f.code).sum();
    assert_eq!(sub.total_code_lines, file_code_sum);

    let file_bytes_sum: usize = sub.files.iter().map(|f| f.bytes).sum();
    assert_eq!(sub.total_bytes, file_bytes_sum);

    let file_token_sum: usize = sub.files.iter().map(|f| f.tokens).sum();
    assert_eq!(sub.total_tokens, file_token_sum);
}

#[test]
fn substrate_lang_summary_consistent_with_files() {
    let sub = multi_lang_substrate();
    for (lang, summary) in &sub.lang_summary {
        let lang_files: Vec<_> = sub.files.iter().filter(|f| &f.lang == lang).collect();
        assert_eq!(summary.files, lang_files.len());
        assert_eq!(
            summary.code,
            lang_files.iter().map(|f| f.code).sum::<usize>()
        );
    }
}

// ---------------------------------------------------------------------------
// Tests: diff_files / files_for_lang helpers
// ---------------------------------------------------------------------------

#[test]
fn diff_files_returns_only_in_diff_files() {
    let sub = multi_lang_substrate();
    let diff: Vec<_> = sub.diff_files().collect();
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().all(|f| f.in_diff));
    let paths: Vec<&str> = diff.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"src/lib.rs"));
    assert!(paths.contains(&"lib/utils.py"));
}

#[test]
fn diff_files_empty_when_no_diff() {
    let sub = single_file_substrate();
    let diff: Vec<_> = sub.diff_files().collect();
    assert!(diff.is_empty());
}

#[test]
fn files_for_lang_returns_correct_subset() {
    let sub = multi_lang_substrate();
    let rust: Vec<_> = sub.files_for_lang("Rust").collect();
    assert_eq!(rust.len(), 2);
    assert!(rust.iter().all(|f| f.lang == "Rust"));

    let py: Vec<_> = sub.files_for_lang("Python").collect();
    assert_eq!(py.len(), 1);
    assert_eq!(py[0].path, "lib/utils.py");
}

#[test]
fn files_for_lang_returns_empty_for_unknown_lang() {
    let sub = multi_lang_substrate();
    let go: Vec<_> = sub.files_for_lang("Go").collect();
    assert!(go.is_empty());
}

// ---------------------------------------------------------------------------
// Tests: Serialization roundtrip
// ---------------------------------------------------------------------------

#[test]
fn serde_roundtrip_with_diff_range() {
    let sub = multi_lang_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), sub.files.len());
    assert_eq!(back.total_code_lines, sub.total_code_lines);
    assert!(back.diff_range.is_some());
    let dr = back.diff_range.unwrap();
    assert_eq!(dr.base, "main");
    assert_eq!(dr.head, "feature");
    assert_eq!(dr.changed_files.len(), 2);
    assert_eq!(dr.commit_count, 5);
}

#[test]
fn serde_roundtrip_without_diff_range() {
    let sub = single_file_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.diff_range.is_none());
    // diff_range should be omitted from JSON when None
    assert!(!json.contains("\"diff_range\""));
}

#[test]
fn serde_roundtrip_empty_substrate() {
    let sub = empty_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.files.is_empty());
    assert!(back.lang_summary.is_empty());
    assert_eq!(back.total_code_lines, 0);
    assert_eq!(back.total_bytes, 0);
    assert_eq!(back.total_tokens, 0);
}

// ---------------------------------------------------------------------------
// Tests: Deterministic construction (BTreeMap ordering)
// ---------------------------------------------------------------------------

#[test]
fn lang_summary_keys_are_sorted() {
    let sub = multi_lang_substrate();
    let keys: Vec<&String> = sub.lang_summary.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "BTreeMap should preserve sorted key order");
}

#[test]
fn deterministic_json_output() {
    let sub1 = multi_lang_substrate();
    let sub2 = multi_lang_substrate();
    let json1 = serde_json::to_string(&sub1).unwrap();
    let json2 = serde_json::to_string(&sub2).unwrap();
    assert_eq!(
        json1, json2,
        "Identical substrates must produce identical JSON"
    );
}

// ---------------------------------------------------------------------------
// Tests: DiffRange fields
// ---------------------------------------------------------------------------

#[test]
fn diff_range_fields_accessible() {
    let dr = DiffRange {
        base: "v1.0.0".to_string(),
        head: "v2.0.0".to_string(),
        changed_files: vec!["a.rs".to_string(), "b.rs".to_string()],
        commit_count: 42,
        insertions: 100,
        deletions: 50,
    };
    assert_eq!(dr.base, "v1.0.0");
    assert_eq!(dr.head, "v2.0.0");
    assert_eq!(dr.changed_files.len(), 2);
    assert_eq!(dr.commit_count, 42);
    assert_eq!(dr.insertions, 100);
    assert_eq!(dr.deletions, 50);
}

#[test]
fn diff_range_serde_roundtrip() {
    let dr = DiffRange {
        base: "main".to_string(),
        head: "HEAD".to_string(),
        changed_files: vec!["src/lib.rs".to_string()],
        commit_count: 1,
        insertions: 5,
        deletions: 2,
    };
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, dr.base);
    assert_eq!(back.head, dr.head);
    assert_eq!(back.changed_files, dr.changed_files);
}
