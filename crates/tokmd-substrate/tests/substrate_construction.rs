//! Deeper tests for `RepoSubstrate` construction, field access,
//! determinism, and empty/minimal substrate creation.

use std::collections::BTreeMap;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ── Helpers ─────────────────────────────────────────────────────

fn make_file(path: &str, lang: &str, code: usize, in_diff: bool) -> SubstrateFile {
    SubstrateFile {
        path: path.to_string(),
        lang: lang.to_string(),
        code,
        lines: code + 20,
        bytes: code * 30,
        tokens: code * 7,
        module: path
            .rsplit_once('/')
            .map(|(m, _)| m)
            .unwrap_or("")
            .to_string(),
        in_diff,
    }
}

fn minimal_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: ".".to_string(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    }
}

fn single_file_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![make_file("src/lib.rs", "Rust", 100, false)],
        lang_summary: BTreeMap::from([(
            "Rust".to_string(),
            LangSummary {
                files: 1,
                code: 100,
                lines: 120,
                bytes: 3000,
                tokens: 700,
            },
        )]),
        diff_range: None,
        total_tokens: 700,
        total_bytes: 3000,
        total_code_lines: 100,
    }
}

// ── Empty/minimal substrate creation ────────────────────────────

#[test]
fn empty_substrate_has_zero_totals() {
    let sub = minimal_substrate();
    assert_eq!(sub.total_tokens, 0);
    assert_eq!(sub.total_bytes, 0);
    assert_eq!(sub.total_code_lines, 0);
    assert!(sub.files.is_empty());
    assert!(sub.lang_summary.is_empty());
    assert!(sub.diff_range.is_none());
}

#[test]
fn empty_substrate_diff_files_returns_empty_iterator() {
    let sub = minimal_substrate();
    assert_eq!(sub.diff_files().count(), 0);
}

#[test]
fn empty_substrate_files_for_lang_returns_empty() {
    let sub = minimal_substrate();
    assert_eq!(sub.files_for_lang("Rust").count(), 0);
    assert_eq!(sub.files_for_lang("Python").count(), 0);
    assert_eq!(sub.files_for_lang("").count(), 0);
}

#[test]
fn empty_substrate_serializes_without_diff_range_key() {
    let sub = minimal_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    assert!(!json.contains("diff_range"));
}

#[test]
fn empty_substrate_roundtrips_through_json() {
    let sub = minimal_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let restored: RepoSubstrate = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.repo_root, sub.repo_root);
    assert!(restored.files.is_empty());
    assert!(restored.lang_summary.is_empty());
    assert!(restored.diff_range.is_none());
    assert_eq!(restored.total_tokens, 0);
}

// ── Single-file substrate construction ──────────────────────────

#[test]
fn single_file_substrate_field_access() {
    let sub = single_file_substrate();
    assert_eq!(sub.repo_root, "/repo");
    assert_eq!(sub.files.len(), 1);
    assert_eq!(sub.files[0].path, "src/lib.rs");
    assert_eq!(sub.files[0].lang, "Rust");
    assert_eq!(sub.files[0].code, 100);
    assert_eq!(sub.total_code_lines, 100);
    assert_eq!(sub.lang_summary.len(), 1);
    assert!(sub.lang_summary.contains_key("Rust"));
}

#[test]
fn single_file_not_in_diff_excluded_from_diff_files() {
    let sub = single_file_substrate();
    assert_eq!(sub.diff_files().count(), 0);
}

// ── Substrate construction with diff range ──────────────────────

#[test]
fn substrate_with_diff_range_marks_correct_files() {
    let sub = RepoSubstrate {
        repo_root: "/project".to_string(),
        files: vec![
            make_file("src/a.rs", "Rust", 50, true),
            make_file("src/b.rs", "Rust", 30, false),
            make_file("tests/t.rs", "Rust", 20, true),
        ],
        lang_summary: BTreeMap::new(),
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "feat".to_string(),
            changed_files: vec!["src/a.rs".to_string(), "tests/t.rs".to_string()],
            commit_count: 2,
            insertions: 15,
            deletions: 5,
        }),
        total_tokens: 700,
        total_bytes: 3000,
        total_code_lines: 100,
    };

    let diff: Vec<_> = sub.diff_files().collect();
    assert_eq!(diff.len(), 2);
    let paths: Vec<&str> = diff.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"src/a.rs"));
    assert!(paths.contains(&"tests/t.rs"));
}

#[test]
fn diff_range_fields_accessible() {
    let dr = DiffRange {
        base: "v1.0.0".to_string(),
        head: "v2.0.0".to_string(),
        changed_files: vec!["a.rs".to_string(), "b.rs".to_string(), "c.rs".to_string()],
        commit_count: 15,
        insertions: 200,
        deletions: 100,
    };

    assert_eq!(dr.base, "v1.0.0");
    assert_eq!(dr.head, "v2.0.0");
    assert_eq!(dr.changed_files.len(), 3);
    assert_eq!(dr.commit_count, 15);
    assert_eq!(dr.insertions, 200);
    assert_eq!(dr.deletions, 100);
}

// ── Determinism ─────────────────────────────────────────────────

#[test]
fn btreemap_ordering_is_deterministic() {
    let mut summary1 = BTreeMap::new();
    summary1.insert(
        "Zig".to_string(),
        LangSummary {
            files: 1,
            code: 10,
            lines: 20,
            bytes: 300,
            tokens: 70,
        },
    );
    summary1.insert(
        "Ada".to_string(),
        LangSummary {
            files: 2,
            code: 20,
            lines: 40,
            bytes: 600,
            tokens: 140,
        },
    );
    summary1.insert(
        "Rust".to_string(),
        LangSummary {
            files: 3,
            code: 30,
            lines: 60,
            bytes: 900,
            tokens: 210,
        },
    );

    let mut summary2 = BTreeMap::new();
    summary2.insert(
        "Rust".to_string(),
        LangSummary {
            files: 3,
            code: 30,
            lines: 60,
            bytes: 900,
            tokens: 210,
        },
    );
    summary2.insert(
        "Zig".to_string(),
        LangSummary {
            files: 1,
            code: 10,
            lines: 20,
            bytes: 300,
            tokens: 70,
        },
    );
    summary2.insert(
        "Ada".to_string(),
        LangSummary {
            files: 2,
            code: 20,
            lines: 40,
            bytes: 600,
            tokens: 140,
        },
    );

    let keys1: Vec<_> = summary1.keys().collect();
    let keys2: Vec<_> = summary2.keys().collect();
    assert_eq!(keys1, keys2);
    assert_eq!(keys1, vec!["Ada", "Rust", "Zig"]);
}

#[test]
fn serialization_is_deterministic_regardless_of_insertion_order() {
    let sub1 = {
        let mut ls = BTreeMap::new();
        ls.insert(
            "Z".to_string(),
            LangSummary {
                files: 1,
                code: 1,
                lines: 1,
                bytes: 1,
                tokens: 1,
            },
        );
        ls.insert(
            "A".to_string(),
            LangSummary {
                files: 2,
                code: 2,
                lines: 2,
                bytes: 2,
                tokens: 2,
            },
        );
        RepoSubstrate {
            repo_root: "/r".to_string(),
            files: vec![],
            lang_summary: ls,
            diff_range: None,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: 0,
        }
    };

    let sub2 = {
        let mut ls = BTreeMap::new();
        ls.insert(
            "A".to_string(),
            LangSummary {
                files: 2,
                code: 2,
                lines: 2,
                bytes: 2,
                tokens: 2,
            },
        );
        ls.insert(
            "Z".to_string(),
            LangSummary {
                files: 1,
                code: 1,
                lines: 1,
                bytes: 1,
                tokens: 1,
            },
        );
        RepoSubstrate {
            repo_root: "/r".to_string(),
            files: vec![],
            lang_summary: ls,
            diff_range: None,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: 0,
        }
    };

    let j1 = serde_json::to_string(&sub1).unwrap();
    let j2 = serde_json::to_string(&sub2).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn clone_produces_identical_serialization() {
    let sub = RepoSubstrate {
        repo_root: "/project".to_string(),
        files: vec![
            make_file("src/a.rs", "Rust", 50, true),
            make_file("lib/b.py", "Python", 30, false),
        ],
        lang_summary: BTreeMap::from([
            (
                "Rust".to_string(),
                LangSummary {
                    files: 1,
                    code: 50,
                    lines: 70,
                    bytes: 1500,
                    tokens: 350,
                },
            ),
            (
                "Python".to_string(),
                LangSummary {
                    files: 1,
                    code: 30,
                    lines: 50,
                    bytes: 900,
                    tokens: 210,
                },
            ),
        ]),
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "dev".to_string(),
            changed_files: vec!["src/a.rs".to_string()],
            commit_count: 1,
            insertions: 10,
            deletions: 3,
        }),
        total_tokens: 560,
        total_bytes: 2400,
        total_code_lines: 80,
    };

    let cloned = sub.clone();
    let j1 = serde_json::to_string(&sub).unwrap();
    let j2 = serde_json::to_string(&cloned).unwrap();
    assert_eq!(j1, j2);
}

// ── Serde roundtrip ─────────────────────────────────────────────

#[test]
fn full_substrate_survives_json_roundtrip() {
    let sub = RepoSubstrate {
        repo_root: "/my/project".to_string(),
        files: vec![
            make_file("src/lib.rs", "Rust", 200, true),
            make_file("src/main.rs", "Rust", 80, false),
            make_file("test.py", "Python", 60, true),
        ],
        lang_summary: BTreeMap::from([
            (
                "Rust".to_string(),
                LangSummary {
                    files: 2,
                    code: 280,
                    lines: 320,
                    bytes: 8400,
                    tokens: 1960,
                },
            ),
            (
                "Python".to_string(),
                LangSummary {
                    files: 1,
                    code: 60,
                    lines: 80,
                    bytes: 1800,
                    tokens: 420,
                },
            ),
        ]),
        diff_range: Some(DiffRange {
            base: "v1.0".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "test.py".to_string()],
            commit_count: 7,
            insertions: 45,
            deletions: 12,
        }),
        total_tokens: 2380,
        total_bytes: 10200,
        total_code_lines: 340,
    };

    let json = serde_json::to_string_pretty(&sub).unwrap();
    let restored: RepoSubstrate = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.repo_root, sub.repo_root);
    assert_eq!(restored.files.len(), sub.files.len());
    assert_eq!(restored.total_code_lines, sub.total_code_lines);
    assert_eq!(restored.total_bytes, sub.total_bytes);
    assert_eq!(restored.total_tokens, sub.total_tokens);

    for (a, b) in restored.files.iter().zip(sub.files.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.lang, b.lang);
        assert_eq!(a.code, b.code);
        assert_eq!(a.in_diff, b.in_diff);
    }

    let dr = restored.diff_range.unwrap();
    assert_eq!(dr.base, "v1.0");
    assert_eq!(dr.head, "HEAD");
    assert_eq!(dr.changed_files.len(), 2);
    assert_eq!(dr.commit_count, 7);
}

#[test]
fn substrate_without_diff_omits_diff_range_in_json() {
    let sub = single_file_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    assert!(!json.contains("diff_range"));

    let restored: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(restored.diff_range.is_none());
}

#[test]
fn deserialization_from_minimal_json() {
    let json = r#"{
        "repo_root": "/test",
        "files": [],
        "lang_summary": {},
        "total_tokens": 0,
        "total_bytes": 0,
        "total_code_lines": 0
    }"#;

    let sub: RepoSubstrate = serde_json::from_str(json).unwrap();
    assert_eq!(sub.repo_root, "/test");
    assert!(sub.files.is_empty());
    assert!(sub.diff_range.is_none());
}

// ── SubstrateFile module derivation ─────────────────────────────

#[test]
fn substrate_file_module_from_nested_path() {
    let f = make_file("src/analysis/mod.rs", "Rust", 10, false);
    assert_eq!(f.module, "src/analysis");
}

#[test]
fn substrate_file_module_from_root_file() {
    let f = make_file("main.rs", "Rust", 10, false);
    assert_eq!(f.module, "");
}

#[test]
fn substrate_file_module_from_single_dir() {
    let f = make_file("src/lib.rs", "Rust", 10, false);
    assert_eq!(f.module, "src");
}

// ── LangSummary field access ────────────────────────────────────

#[test]
fn lang_summary_all_fields() {
    let ls = LangSummary {
        files: 5,
        code: 1000,
        lines: 1200,
        bytes: 30000,
        tokens: 7000,
    };

    assert_eq!(ls.files, 5);
    assert_eq!(ls.code, 1000);
    assert_eq!(ls.lines, 1200);
    assert_eq!(ls.bytes, 30000);
    assert_eq!(ls.tokens, 7000);
}

#[test]
fn lang_summary_roundtrips() {
    let ls = LangSummary {
        files: 3,
        code: 500,
        lines: 600,
        bytes: 15000,
        tokens: 3500,
    };

    let json = serde_json::to_string(&ls).unwrap();
    let restored: LangSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.files, ls.files);
    assert_eq!(restored.code, ls.code);
    assert_eq!(restored.tokens, ls.tokens);
}

// ── Debug formatting ────────────────────────────────────────────

#[test]
fn all_types_implement_debug_without_panic() {
    let sub = single_file_substrate();
    let _ = format!("{sub:?}");
    let _ = format!("{:?}", sub.files[0]);
    let _ = format!("{:?}", sub.lang_summary["Rust"]);

    let dr = DiffRange {
        base: "a".to_string(),
        head: "b".to_string(),
        changed_files: vec![],
        commit_count: 0,
        insertions: 0,
        deletions: 0,
    };
    let _ = format!("{dr:?}");

    let empty = minimal_substrate();
    let _ = format!("{empty:?}");
}

// ── Mutability ──────────────────────────────────────────────────

#[test]
fn substrate_fields_are_mutable() {
    let mut sub = minimal_substrate();

    sub.repo_root = "/new".to_string();
    sub.total_code_lines = 42;
    sub.files.push(make_file("new.rs", "Rust", 42, true));
    sub.lang_summary.insert(
        "Rust".to_string(),
        LangSummary {
            files: 1,
            code: 42,
            lines: 62,
            bytes: 1260,
            tokens: 294,
        },
    );
    sub.diff_range = Some(DiffRange {
        base: "a".to_string(),
        head: "b".to_string(),
        changed_files: vec!["new.rs".to_string()],
        commit_count: 1,
        insertions: 42,
        deletions: 0,
    });

    assert_eq!(sub.repo_root, "/new");
    assert_eq!(sub.total_code_lines, 42);
    assert_eq!(sub.files.len(), 1);
    assert_eq!(sub.diff_files().count(), 1);
    assert!(sub.diff_range.is_some());
}
