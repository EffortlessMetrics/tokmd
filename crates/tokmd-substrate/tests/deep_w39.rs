//! Deep tests for tokmd-substrate – wave 39.

use std::collections::BTreeMap;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_file(path: &str, lang: &str, code: usize, in_diff: bool) -> SubstrateFile {
    SubstrateFile {
        path: path.into(),
        lang: lang.into(),
        code,
        lines: code + 20,
        bytes: code * 30,
        tokens: code * 3,
        module: path.rsplit_once('/').map(|(m, _)| m).unwrap_or(".").into(),
        in_diff,
    }
}

fn sample_substrate() -> RepoSubstrate {
    let files = vec![
        make_file("src/lib.rs", "Rust", 100, true),
        make_file("src/main.rs", "Rust", 50, false),
        make_file("tests/test.py", "Python", 30, true),
    ];
    let mut lang = BTreeMap::new();
    lang.insert(
        "Rust".into(),
        LangSummary {
            files: 2,
            code: 150,
            lines: 190,
            bytes: 4500,
            tokens: 450,
        },
    );
    lang.insert(
        "Python".into(),
        LangSummary {
            files: 1,
            code: 30,
            lines: 50,
            bytes: 900,
            tokens: 90,
        },
    );
    RepoSubstrate {
        repo_root: "/repo".into(),
        files,
        lang_summary: lang,
        diff_range: Some(DiffRange {
            base: "main".into(),
            head: "HEAD".into(),
            changed_files: vec!["src/lib.rs".into(), "tests/test.py".into()],
            commit_count: 5,
            insertions: 20,
            deletions: 3,
        }),
        total_tokens: 540,
        total_bytes: 5400,
        total_code_lines: 180,
    }
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

#[test]
fn substrate_construction_basic_fields() {
    let s = sample_substrate();
    assert_eq!(s.repo_root, "/repo");
    assert_eq!(s.files.len(), 3);
    assert_eq!(s.total_code_lines, 180);
    assert_eq!(s.total_bytes, 5400);
    assert_eq!(s.total_tokens, 540);
}

#[test]
fn substrate_lang_summary_btreemap_order() {
    let s = sample_substrate();
    let keys: Vec<_> = s.lang_summary.keys().collect();
    // BTreeMap ensures alphabetical ordering
    assert_eq!(keys, vec!["Python", "Rust"]);
}

#[test]
fn substrate_diff_range_present() {
    let s = sample_substrate();
    let dr = s.diff_range.as_ref().unwrap();
    assert_eq!(dr.base, "main");
    assert_eq!(dr.head, "HEAD");
    assert_eq!(dr.commit_count, 5);
    assert_eq!(dr.insertions, 20);
    assert_eq!(dr.deletions, 3);
    assert_eq!(dr.changed_files.len(), 2);
}

// ---------------------------------------------------------------------------
// Field accessors: diff_files / files_for_lang
// ---------------------------------------------------------------------------

#[test]
fn diff_files_returns_only_changed() {
    let s = sample_substrate();
    let diff: Vec<_> = s.diff_files().collect();
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().all(|f| f.in_diff));
}

#[test]
fn files_for_lang_filters_correctly() {
    let s = sample_substrate();
    let rust: Vec<_> = s.files_for_lang("Rust").collect();
    assert_eq!(rust.len(), 2);
    let py: Vec<_> = s.files_for_lang("Python").collect();
    assert_eq!(py.len(), 1);
}

#[test]
fn files_for_lang_unknown_returns_empty() {
    let s = sample_substrate();
    let go: Vec<_> = s.files_for_lang("Go").collect();
    assert!(go.is_empty());
}

// ---------------------------------------------------------------------------
// Serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn serde_roundtrip_full() {
    let s = sample_substrate();
    let json = serde_json::to_string(&s).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.repo_root, s.repo_root);
    assert_eq!(back.files.len(), s.files.len());
    assert_eq!(back.total_code_lines, s.total_code_lines);
    assert_eq!(back.lang_summary.len(), s.lang_summary.len());
    assert!(back.diff_range.is_some());
}

#[test]
fn serde_roundtrip_no_diff_range() {
    let s = RepoSubstrate {
        repo_root: "/empty".into(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };
    let json = serde_json::to_string(&s).unwrap();
    // diff_range should be omitted via skip_serializing_if
    assert!(!json.contains("\"diff_range\""));
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.diff_range.is_none());
}

#[test]
fn serde_roundtrip_substrate_file() {
    let f = make_file("src/main.rs", "Rust", 42, false);
    let json = serde_json::to_string(&f).unwrap();
    let back: SubstrateFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "src/main.rs");
    assert_eq!(back.lang, "Rust");
    assert_eq!(back.code, 42);
    assert!(!back.in_diff);
}

#[test]
fn serde_roundtrip_lang_summary() {
    let ls = LangSummary {
        files: 10,
        code: 500,
        lines: 700,
        bytes: 15000,
        tokens: 1500,
    };
    let json = serde_json::to_string(&ls).unwrap();
    let back: LangSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files, 10);
    assert_eq!(back.tokens, 1500);
}

#[test]
fn serde_roundtrip_diff_range() {
    let dr = DiffRange {
        base: "v1.0.0".into(),
        head: "v2.0.0".into(),
        changed_files: vec!["a.rs".into()],
        commit_count: 1,
        insertions: 5,
        deletions: 2,
    };
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "v1.0.0");
    assert_eq!(back.head, "v2.0.0");
    assert_eq!(back.changed_files.len(), 1);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_repo_substrate() {
    let s = RepoSubstrate {
        repo_root: "/empty".into(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };
    assert!(s.files.is_empty());
    assert!(s.lang_summary.is_empty());
    assert_eq!(s.diff_files().count(), 0);
    assert_eq!(s.files_for_lang("Rust").count(), 0);
}

#[test]
fn large_repo_many_files() {
    let files: Vec<SubstrateFile> = (0..1000)
        .map(|i| make_file(&format!("src/file_{i}.rs"), "Rust", 100 + i, i % 3 == 0))
        .collect();
    let total_code: usize = files.iter().map(|f| f.code).sum();
    let total_tokens: usize = files.iter().map(|f| f.tokens).sum();
    let total_bytes: usize = files.iter().map(|f| f.bytes).sum();
    let s = RepoSubstrate {
        repo_root: "/big".into(),
        files,
        lang_summary: BTreeMap::from([(
            "Rust".into(),
            LangSummary {
                files: 1000,
                code: total_code,
                lines: total_code + 20000,
                bytes: total_bytes,
                tokens: total_tokens,
            },
        )]),
        diff_range: None,
        total_tokens,
        total_bytes,
        total_code_lines: total_code,
    };
    assert_eq!(s.files.len(), 1000);
    assert_eq!(s.files_for_lang("Rust").count(), 1000);
    // Every 3rd file is in_diff → ceil(1000/3) = 334
    assert_eq!(s.diff_files().count(), 334);

    // Serde roundtrip for large substrate
    let json = serde_json::to_string(&s).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 1000);
    assert_eq!(back.total_code_lines, total_code);
}

#[test]
fn multi_language_substrate() {
    let files = vec![
        make_file("src/app.ts", "TypeScript", 200, false),
        make_file("src/index.js", "JavaScript", 150, true),
        make_file("lib/utils.py", "Python", 80, false),
    ];
    let mut lang_summary = BTreeMap::new();
    lang_summary.insert(
        "TypeScript".into(),
        LangSummary {
            files: 1,
            code: 200,
            lines: 220,
            bytes: 6000,
            tokens: 600,
        },
    );
    lang_summary.insert(
        "JavaScript".into(),
        LangSummary {
            files: 1,
            code: 150,
            lines: 170,
            bytes: 4500,
            tokens: 450,
        },
    );
    lang_summary.insert(
        "Python".into(),
        LangSummary {
            files: 1,
            code: 80,
            lines: 100,
            bytes: 2400,
            tokens: 240,
        },
    );
    let s = RepoSubstrate {
        repo_root: "/multi".into(),
        files,
        lang_summary,
        diff_range: None,
        total_tokens: 1290,
        total_bytes: 12900,
        total_code_lines: 430,
    };
    assert_eq!(s.lang_summary.len(), 3);
    // BTreeMap order: JavaScript < Python < TypeScript
    let keys: Vec<_> = s.lang_summary.keys().collect();
    assert_eq!(keys, vec!["JavaScript", "Python", "TypeScript"]);
}

#[test]
fn substrate_file_module_key_from_path() {
    let f = make_file("crates/tokmd-scan/src/lib.rs", "Rust", 50, false);
    assert_eq!(f.module, "crates/tokmd-scan/src");
}

#[test]
fn substrate_file_root_module_key() {
    // File at root has no parent directory
    let f = make_file("Cargo.toml", "TOML", 10, false);
    assert_eq!(f.module, ".");
}
