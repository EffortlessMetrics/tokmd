//! BDD-style scenario tests for `RepoSubstrate` and related types.
//!
//! Each test follows a Given/When/Then structure to validate
//! substrate creation, field access, filtering, and edge cases.

use std::collections::BTreeMap;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ── helpers ──────────────────────────────────────────────────────

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

fn make_lang(files: usize, code: usize) -> LangSummary {
    LangSummary {
        files,
        code,
        lines: code + files * 20,
        bytes: code * 30,
        tokens: code * 7,
    }
}

fn sample_substrate() -> RepoSubstrate {
    let files = vec![
        make_file("src/lib.rs", "Rust", 100, true),
        make_file("src/main.rs", "Rust", 50, false),
        make_file("tests/integration.rs", "Rust", 30, true),
        make_file("README.md", "Markdown", 20, false),
    ];
    let mut lang_summary = BTreeMap::new();
    lang_summary.insert("Rust".to_string(), make_lang(3, 180));
    lang_summary.insert("Markdown".to_string(), make_lang(1, 20));

    RepoSubstrate {
        repo_root: "/home/user/project".to_string(),
        files,
        lang_summary,
        diff_range: Some(DiffRange {
            base: "v1.0.0".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "tests/integration.rs".to_string()],
            commit_count: 5,
            insertions: 42,
            deletions: 10,
        }),
        total_tokens: 200 * 7,
        total_bytes: 200 * 30,
        total_code_lines: 200,
    }
}

// ── Scenario: Create substrate with multiple languages ───────────

#[test]
fn given_multi_lang_repo_when_created_then_fields_are_accessible() {
    // Given
    let sub = sample_substrate();

    // Then
    assert_eq!(sub.repo_root, "/home/user/project");
    assert_eq!(sub.files.len(), 4);
    assert_eq!(sub.lang_summary.len(), 2);
    assert_eq!(sub.total_code_lines, 200);
    assert_eq!(sub.total_bytes, 200 * 30);
    assert_eq!(sub.total_tokens, 200 * 7);
}

// ── Scenario: Filter diff files ──────────────────────────────────

#[test]
fn given_substrate_with_diff_when_diff_files_called_then_only_changed_returned() {
    // Given
    let sub = sample_substrate();

    // When
    let diff: Vec<_> = sub.diff_files().collect();

    // Then
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().all(|f| f.in_diff));
    let paths: Vec<&str> = diff.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"src/lib.rs"));
    assert!(paths.contains(&"tests/integration.rs"));
}

#[test]
fn given_substrate_with_no_diff_files_when_diff_files_called_then_empty() {
    // Given — all files have in_diff = false
    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![
            make_file("a.rs", "Rust", 10, false),
            make_file("b.rs", "Rust", 20, false),
        ],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };

    // When / Then
    assert_eq!(sub.diff_files().count(), 0);
}

// ── Scenario: Filter files by language ───────────────────────────

#[test]
fn given_multi_lang_repo_when_files_for_lang_called_then_correct_subset() {
    let sub = sample_substrate();

    let rust: Vec<_> = sub.files_for_lang("Rust").collect();
    assert_eq!(rust.len(), 3);

    let md: Vec<_> = sub.files_for_lang("Markdown").collect();
    assert_eq!(md.len(), 1);
    assert_eq!(md[0].path, "README.md");
}

#[test]
fn given_substrate_when_files_for_nonexistent_lang_then_empty() {
    let sub = sample_substrate();
    assert_eq!(sub.files_for_lang("Haskell").count(), 0);
}

// ── Scenario: Empty repository ───────────────────────────────────

#[test]
fn given_empty_repo_when_created_then_all_totals_zero() {
    let sub = RepoSubstrate {
        repo_root: "/empty".to_string(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };

    assert!(sub.files.is_empty());
    assert!(sub.lang_summary.is_empty());
    assert!(sub.diff_range.is_none());
    assert_eq!(sub.total_tokens, 0);
    assert_eq!(sub.total_bytes, 0);
    assert_eq!(sub.total_code_lines, 0);
    assert_eq!(sub.diff_files().count(), 0);
    assert_eq!(sub.files_for_lang("Rust").count(), 0);
}

// ── Scenario: Repo without git (no diff_range) ──────────────────

#[test]
fn given_repo_without_git_when_created_then_diff_range_is_none() {
    let sub = RepoSubstrate {
        repo_root: "/no-git".to_string(),
        files: vec![make_file("main.py", "Python", 50, false)],
        lang_summary: {
            let mut m = BTreeMap::new();
            m.insert("Python".to_string(), make_lang(1, 50));
            m
        },
        diff_range: None,
        total_tokens: 350,
        total_bytes: 1500,
        total_code_lines: 50,
    };

    assert!(sub.diff_range.is_none());
    assert_eq!(sub.files.len(), 1);
    assert_eq!(sub.diff_files().count(), 0);
}

// ── Scenario: Serde round-trip preserves all data ────────────────

#[test]
fn given_full_substrate_when_serialized_and_deserialized_then_equal() {
    let original = sample_substrate();
    let json = serde_json::to_string_pretty(&original).unwrap();
    let restored: RepoSubstrate = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.repo_root, original.repo_root);
    assert_eq!(restored.files.len(), original.files.len());
    assert_eq!(restored.lang_summary.len(), original.lang_summary.len());
    assert_eq!(restored.total_tokens, original.total_tokens);
    assert_eq!(restored.total_bytes, original.total_bytes);
    assert_eq!(restored.total_code_lines, original.total_code_lines);

    // Verify file data survived
    for (a, b) in restored.files.iter().zip(original.files.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.lang, b.lang);
        assert_eq!(a.code, b.code);
        assert_eq!(a.lines, b.lines);
        assert_eq!(a.bytes, b.bytes);
        assert_eq!(a.tokens, b.tokens);
        assert_eq!(a.module, b.module);
        assert_eq!(a.in_diff, b.in_diff);
    }

    // Verify diff range survived
    let dr_orig = original.diff_range.unwrap();
    let dr_rest = restored.diff_range.unwrap();
    assert_eq!(dr_rest.base, dr_orig.base);
    assert_eq!(dr_rest.head, dr_orig.head);
    assert_eq!(dr_rest.changed_files, dr_orig.changed_files);
    assert_eq!(dr_rest.commit_count, dr_orig.commit_count);
    assert_eq!(dr_rest.insertions, dr_orig.insertions);
    assert_eq!(dr_rest.deletions, dr_orig.deletions);
}

#[test]
fn given_substrate_without_diff_when_serialized_then_diff_range_absent_in_json() {
    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };

    let json = serde_json::to_string(&sub).unwrap();
    // skip_serializing_if = "Option::is_none" should omit the key
    assert!(!json.contains("diff_range"));

    // Deserialize back should still work
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.diff_range.is_none());
}

// ── Scenario: BTreeMap ordering is deterministic ─────────────────

#[test]
fn given_languages_inserted_out_of_order_when_iterated_then_sorted() {
    let mut summary = BTreeMap::new();
    summary.insert("Zig".to_string(), make_lang(1, 10));
    summary.insert("Ada".to_string(), make_lang(2, 20));
    summary.insert("Rust".to_string(), make_lang(3, 30));

    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![],
        lang_summary: summary,
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };

    let keys: Vec<_> = sub.lang_summary.keys().cloned().collect();
    assert_eq!(keys, vec!["Ada", "Rust", "Zig"]);
}

// ── Scenario: LangSummary field access ───────────────────────────

#[test]
fn given_lang_summary_when_accessed_then_all_fields_correct() {
    let ls = LangSummary {
        files: 10,
        code: 500,
        lines: 700,
        bytes: 15000,
        tokens: 3500,
    };

    assert_eq!(ls.files, 10);
    assert_eq!(ls.code, 500);
    assert_eq!(ls.lines, 700);
    assert_eq!(ls.bytes, 15000);
    assert_eq!(ls.tokens, 3500);
}

// ── Scenario: DiffRange field access ─────────────────────────────

#[test]
fn given_diff_range_when_accessed_then_all_fields_correct() {
    let dr = DiffRange {
        base: "main".to_string(),
        head: "feature/add-tests".to_string(),
        changed_files: vec!["a.rs".to_string(), "b.rs".to_string()],
        commit_count: 7,
        insertions: 100,
        deletions: 50,
    };

    assert_eq!(dr.base, "main");
    assert_eq!(dr.head, "feature/add-tests");
    assert_eq!(dr.changed_files.len(), 2);
    assert_eq!(dr.commit_count, 7);
    assert_eq!(dr.insertions, 100);
    assert_eq!(dr.deletions, 50);
}

// ── Scenario: SubstrateFile module derivation ────────────────────

#[test]
fn given_files_in_nested_modules_when_accessed_then_module_correct() {
    let f1 = make_file("src/lib.rs", "Rust", 10, false);
    assert_eq!(f1.module, "src");

    let f2 = make_file("src/analysis/mod.rs", "Rust", 20, false);
    assert_eq!(f2.module, "src/analysis");

    let f3 = make_file("root_file.rs", "Rust", 5, false);
    assert_eq!(f3.module, "");
}

// ── Scenario: Clone preserves all data ───────────────────────────

#[test]
fn given_substrate_when_cloned_then_identical() {
    let original = sample_substrate();
    let cloned = original.clone();

    assert_eq!(cloned.repo_root, original.repo_root);
    assert_eq!(cloned.files.len(), original.files.len());
    assert_eq!(cloned.total_code_lines, original.total_code_lines);
    assert_eq!(cloned.total_bytes, original.total_bytes);
    assert_eq!(cloned.total_tokens, original.total_tokens);
}

// ── Scenario: Mutability ─────────────────────────────────────────

#[test]
fn given_substrate_when_fields_modified_then_changes_persisted() {
    let mut sub = sample_substrate();

    // Mutate scalar fields
    sub.total_code_lines = 999;
    sub.repo_root = "/new/root".to_string();
    assert_eq!(sub.total_code_lines, 999);
    assert_eq!(sub.repo_root, "/new/root");

    // Add a new file
    sub.files.push(make_file("new.rs", "Rust", 10, true));
    assert_eq!(sub.files.len(), 5);

    // Add a new language
    sub.lang_summary.insert("Go".to_string(), make_lang(1, 100));
    assert!(sub.lang_summary.contains_key("Go"));

    // Set diff_range to None
    sub.diff_range = None;
    assert!(sub.diff_range.is_none());
}

// ── Scenario: Large substrate ────────────────────────────────────

#[test]
fn given_large_repo_when_filtered_then_correct_counts() {
    let mut files = Vec::new();
    let mut total_code = 0usize;
    for i in 0..1000 {
        let lang = if i % 3 == 0 {
            "Rust"
        } else if i % 3 == 1 {
            "Python"
        } else {
            "Go"
        };
        let code = (i % 50) + 1;
        total_code += code;
        files.push(make_file(
            &format!("src/file_{i}.rs"),
            lang,
            code,
            i % 5 == 0,
        ));
    }

    let sub = RepoSubstrate {
        repo_root: "/large".to_string(),
        files,
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: total_code * 7,
        total_bytes: total_code * 30,
        total_code_lines: total_code,
    };

    assert_eq!(sub.files.len(), 1000);

    // Every 5th file is in_diff (i=0,5,10,...,995) → 200 files
    assert_eq!(sub.diff_files().count(), 200);

    // Every 3rd file (i%3==0) is Rust → 334 files
    let rust_count = sub.files_for_lang("Rust").count();
    assert_eq!(rust_count, 334);

    let python_count = sub.files_for_lang("Python").count();
    assert_eq!(python_count, 333);

    let go_count = sub.files_for_lang("Go").count();
    assert_eq!(go_count, 333);
}

// ── Scenario: Deserialize from hand-written JSON ─────────────────

#[test]
fn given_minimal_json_when_deserialized_then_valid_substrate() {
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

#[test]
fn given_json_with_diff_range_when_deserialized_then_populated() {
    let json = r#"{
        "repo_root": "/test",
        "files": [{
            "path": "main.rs",
            "lang": "Rust",
            "code": 42,
            "lines": 50,
            "bytes": 1200,
            "tokens": 300,
            "module": "",
            "in_diff": true
        }],
        "lang_summary": {
            "Rust": {"files": 1, "code": 42, "lines": 50, "bytes": 1200, "tokens": 300}
        },
        "diff_range": {
            "base": "main",
            "head": "dev",
            "changed_files": ["main.rs"],
            "commit_count": 1,
            "insertions": 5,
            "deletions": 2
        },
        "total_tokens": 300,
        "total_bytes": 1200,
        "total_code_lines": 42
    }"#;

    let sub: RepoSubstrate = serde_json::from_str(json).unwrap();
    assert_eq!(sub.files.len(), 1);
    assert_eq!(sub.files[0].code, 42);
    assert!(sub.files[0].in_diff);
    let dr = sub.diff_range.unwrap();
    assert_eq!(dr.base, "main");
    assert_eq!(dr.head, "dev");
    assert_eq!(dr.changed_files, vec!["main.rs"]);
}

// ── Scenario: Debug formatting ───────────────────────────────────

#[test]
fn given_substrate_types_when_debug_formatted_then_no_panic() {
    let sub = sample_substrate();
    let _ = format!("{sub:?}");
    let _ = format!("{:?}", sub.files[0]);
    let _ = format!("{:?}", sub.lang_summary["Rust"]);
    let _ = format!("{:?}", sub.diff_range.unwrap());
}
