//! Deep substrate tests (w48): construction, field accessors, cross-sensor
//! sharing, serde roundtrips, and property-based verification.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ── helpers ─────────────────────────────────────────────────────

fn make_file(path: &str, lang: &str, code: usize, in_diff: bool) -> SubstrateFile {
    SubstrateFile {
        path: path.to_string(),
        lang: lang.to_string(),
        code,
        lines: code + 10,
        bytes: code * 25,
        tokens: code * 6,
        module: path
            .rsplit_once('/')
            .map(|(m, _)| m)
            .unwrap_or("")
            .to_string(),
        in_diff,
    }
}

fn empty_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    }
}

fn multi_lang_substrate() -> RepoSubstrate {
    let files = vec![
        make_file("src/lib.rs", "Rust", 200, true),
        make_file("src/main.rs", "Rust", 100, false),
        make_file("app.py", "Python", 80, true),
        make_file("index.js", "JavaScript", 60, false),
        make_file("README.md", "Markdown", 30, false),
    ];
    let mut lang_summary = BTreeMap::new();
    lang_summary.insert(
        "Rust".to_string(),
        LangSummary {
            files: 2,
            code: 300,
            lines: 320,
            bytes: 7500,
            tokens: 1800,
        },
    );
    lang_summary.insert(
        "Python".to_string(),
        LangSummary {
            files: 1,
            code: 80,
            lines: 90,
            bytes: 2000,
            tokens: 480,
        },
    );
    lang_summary.insert(
        "JavaScript".to_string(),
        LangSummary {
            files: 1,
            code: 60,
            lines: 70,
            bytes: 1500,
            tokens: 360,
        },
    );
    lang_summary.insert(
        "Markdown".to_string(),
        LangSummary {
            files: 1,
            code: 30,
            lines: 40,
            bytes: 750,
            tokens: 180,
        },
    );
    RepoSubstrate {
        repo_root: "/home/user/project".to_string(),
        files,
        lang_summary,
        diff_range: Some(DiffRange {
            base: "v1.0.0".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "app.py".to_string()],
            commit_count: 5,
            insertions: 40,
            deletions: 10,
        }),
        total_tokens: 2820,
        total_bytes: 11750,
        total_code_lines: 470,
    }
}

// ===========================================================================
// 1. RepoSubstrate construction
// ===========================================================================

#[test]
fn construct_empty_substrate() {
    let s = empty_substrate();
    assert_eq!(s.repo_root, "/repo");
    assert!(s.files.is_empty());
    assert!(s.lang_summary.is_empty());
    assert!(s.diff_range.is_none());
    assert_eq!(s.total_tokens, 0);
    assert_eq!(s.total_bytes, 0);
    assert_eq!(s.total_code_lines, 0);
}

#[test]
fn construct_multi_lang_substrate() {
    let s = multi_lang_substrate();
    assert_eq!(s.files.len(), 5);
    assert_eq!(s.lang_summary.len(), 4);
    assert!(s.diff_range.is_some());
    assert_eq!(s.total_code_lines, 470);
}

#[test]
fn substrate_file_fields() {
    let f = make_file("src/util.rs", "Rust", 50, true);
    assert_eq!(f.path, "src/util.rs");
    assert_eq!(f.lang, "Rust");
    assert_eq!(f.code, 50);
    assert_eq!(f.lines, 60);
    assert_eq!(f.bytes, 1250);
    assert_eq!(f.tokens, 300);
    assert_eq!(f.module, "src");
    assert!(f.in_diff);
}

#[test]
fn diff_range_fields() {
    let d = DiffRange {
        base: "main".to_string(),
        head: "feature".to_string(),
        changed_files: vec!["a.rs".to_string(), "b.rs".to_string()],
        commit_count: 3,
        insertions: 20,
        deletions: 5,
    };
    assert_eq!(d.base, "main");
    assert_eq!(d.head, "feature");
    assert_eq!(d.changed_files.len(), 2);
    assert_eq!(d.commit_count, 3);
}

// ===========================================================================
// 2. Substrate sharing across sensors (clone semantics)
// ===========================================================================

#[test]
fn substrate_clone_is_independent() {
    let s1 = multi_lang_substrate();
    let mut s2 = s1.clone();
    s2.repo_root = "/other".to_string();
    s2.files.push(make_file("extra.rs", "Rust", 10, false));
    assert_eq!(s1.repo_root, "/home/user/project");
    assert_eq!(s1.files.len(), 5);
    assert_eq!(s2.files.len(), 6);
}

#[test]
fn substrate_clone_preserves_all_fields() {
    let s = multi_lang_substrate();
    let c = s.clone();
    assert_eq!(s.repo_root, c.repo_root);
    assert_eq!(s.files.len(), c.files.len());
    assert_eq!(s.lang_summary.len(), c.lang_summary.len());
    assert_eq!(s.total_tokens, c.total_tokens);
    assert_eq!(s.total_bytes, c.total_bytes);
    assert_eq!(s.total_code_lines, c.total_code_lines);
}

// ===========================================================================
// 3. Field accessors
// ===========================================================================

#[test]
fn diff_files_returns_only_in_diff() {
    let s = multi_lang_substrate();
    let diff: Vec<_> = s.diff_files().collect();
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().all(|f| f.in_diff));
    let paths: Vec<&str> = diff.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"src/lib.rs"));
    assert!(paths.contains(&"app.py"));
}

#[test]
fn diff_files_empty_when_none_in_diff() {
    let mut s = empty_substrate();
    s.files.push(make_file("a.rs", "Rust", 10, false));
    assert_eq!(s.diff_files().count(), 0);
}

#[test]
fn files_for_lang_filters_correctly() {
    let s = multi_lang_substrate();
    assert_eq!(s.files_for_lang("Rust").count(), 2);
    assert_eq!(s.files_for_lang("Python").count(), 1);
    assert_eq!(s.files_for_lang("JavaScript").count(), 1);
    assert_eq!(s.files_for_lang("Markdown").count(), 1);
    assert_eq!(s.files_for_lang("Go").count(), 0);
}

#[test]
fn lang_summary_btreemap_sorted() {
    let s = multi_lang_substrate();
    let keys: Vec<&String> = s.lang_summary.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "BTreeMap keys must be sorted");
}

// ===========================================================================
// 4. Serde roundtrip
// ===========================================================================

#[test]
fn serde_roundtrip_empty() {
    let s = empty_substrate();
    let json = serde_json::to_string(&s).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.files.is_empty());
    assert!(back.diff_range.is_none());
}

#[test]
fn serde_roundtrip_populated() {
    let s = multi_lang_substrate();
    let json = serde_json::to_string_pretty(&s).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 5);
    assert_eq!(back.lang_summary.len(), 4);
    assert_eq!(back.total_code_lines, 470);
    assert!(back.diff_range.is_some());
}

#[test]
fn double_roundtrip_bytes_identical() {
    let s = multi_lang_substrate();
    let j1 = serde_json::to_string(&s).unwrap();
    let mid: RepoSubstrate = serde_json::from_str(&j1).unwrap();
    let j2 = serde_json::to_string(&mid).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn diff_range_none_omitted_from_json() {
    let s = empty_substrate();
    let json = serde_json::to_string(&s).unwrap();
    assert!(!json.contains("\"diff_range\""));
}

#[test]
fn diff_range_some_present_in_json() {
    let s = multi_lang_substrate();
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("\"diff_range\""));
    assert!(json.contains("v1.0.0"));
}

// ===========================================================================
// 5. Property test: substrate roundtrips through serde
// ===========================================================================

proptest! {
    #[test]
    fn prop_substrate_roundtrip(
        n_files in 0usize..20,
        has_diff in any::<bool>(),
        root in "[a-z/]{1,30}",
    ) {
        let files: Vec<SubstrateFile> = (0..n_files)
            .map(|i| SubstrateFile {
                path: format!("file_{i}.rs"),
                lang: "Rust".to_string(),
                code: i * 10,
                lines: i * 12,
                bytes: i * 100,
                tokens: i * 6,
                module: "src".to_string(),
                in_diff: i % 2 == 0,
            })
            .collect();
        let diff_range = if has_diff {
            Some(DiffRange {
                base: "main".to_string(),
                head: "HEAD".to_string(),
                changed_files: vec![],
                commit_count: 1,
                insertions: 0,
                deletions: 0,
            })
        } else {
            None
        };
        let s = RepoSubstrate {
            repo_root: root,
            files,
            lang_summary: BTreeMap::new(),
            diff_range,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: 0,
        };

        let json = serde_json::to_string(&s).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.files.len(), n_files);
        prop_assert_eq!(back.diff_range.is_some(), has_diff);
        prop_assert_eq!(back.repo_root, s.repo_root);
    }

    #[test]
    fn prop_diff_files_count_matches(
        n_files in 1usize..30,
        diff_ratio in 0usize..100,
    ) {
        let files: Vec<SubstrateFile> = (0..n_files)
            .map(|i| make_file(&format!("f{i}.rs"), "Rust", 10, (i * 100 / n_files) < diff_ratio))
            .collect();
        let s = RepoSubstrate {
            repo_root: "/r".to_string(),
            files,
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: 0,
        };
        let diff_count = s.diff_files().count();
        let expected = s.files.iter().filter(|f| f.in_diff).count();
        prop_assert_eq!(diff_count, expected);
    }
}
