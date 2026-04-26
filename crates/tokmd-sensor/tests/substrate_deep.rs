//! Deep contract tests for `tokmd-sensor::substrate`.
//!
//! Covers error handling for malformed input, forward-compatibility
//! (extra JSON fields), deterministic serialization, double-roundtrip
//! stability, JSON structure invariants, extreme values, and
//! deserialization from external fixtures.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// =============================================================================
// Helpers
// =============================================================================

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

fn populated_substrate() -> RepoSubstrate {
    let files = vec![
        make_file("src/lib.rs", "Rust", 200, true),
        make_file("src/main.rs", "Rust", 100, false),
        make_file("src/util.py", "Python", 80, true),
        make_file("README.md", "Markdown", 30, false),
    ];
    let mut lang_summary = BTreeMap::new();
    lang_summary.insert(
        "Rust".to_string(),
        LangSummary {
            files: 2,
            code: 300,
            lines: 340,
            bytes: 9000,
            tokens: 2100,
        },
    );
    lang_summary.insert(
        "Python".to_string(),
        LangSummary {
            files: 1,
            code: 80,
            lines: 100,
            bytes: 2400,
            tokens: 560,
        },
    );
    lang_summary.insert(
        "Markdown".to_string(),
        LangSummary {
            files: 1,
            code: 30,
            lines: 50,
            bytes: 900,
            tokens: 210,
        },
    );

    RepoSubstrate {
        repo_root: "/home/user/project".to_string(),
        files,
        lang_summary,
        diff_range: Some(DiffRange {
            base: "v1.0.0".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "src/util.py".to_string()],
            commit_count: 7,
            insertions: 50,
            deletions: 20,
        }),
        total_tokens: 2870,
        total_bytes: 12300,
        total_code_lines: 410,
    }
}

// =============================================================================
// 1. Deterministic serialization
// =============================================================================

#[test]
fn deterministic_serialization_empty() {
    let s = empty_substrate();
    let j1 = serde_json::to_string(&s).unwrap();
    let j2 = serde_json::to_string(&s).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn deterministic_serialization_populated() {
    let s = populated_substrate();
    let j1 = serde_json::to_string_pretty(&s).unwrap();
    let j2 = serde_json::to_string_pretty(&s).unwrap();
    assert_eq!(j1, j2);
}

// =============================================================================
// 2. Double-roundtrip stability
// =============================================================================

#[test]
fn double_roundtrip_empty() {
    let json1 = serde_json::to_string(&empty_substrate()).unwrap();
    let mid: RepoSubstrate = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&mid).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn double_roundtrip_populated() {
    let json1 = serde_json::to_string(&populated_substrate()).unwrap();
    let mid: RepoSubstrate = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&mid).unwrap();
    assert_eq!(json1, json2);
}

// =============================================================================
// 3. Error handling: malformed JSON input
// =============================================================================

#[test]
fn reject_empty_string() {
    let result = serde_json::from_str::<RepoSubstrate>("");
    assert!(result.is_err());
}

#[test]
fn reject_invalid_json() {
    let result = serde_json::from_str::<RepoSubstrate>("{broken}");
    assert!(result.is_err());
}

#[test]
fn reject_missing_required_field_repo_root() {
    let json = r#"{
        "files": [],
        "lang_summary": {},
        "total_tokens": 0,
        "total_bytes": 0,
        "total_code_lines": 0
    }"#;
    let result = serde_json::from_str::<RepoSubstrate>(json);
    assert!(result.is_err(), "missing 'repo_root' should fail");
}

#[test]
fn reject_missing_required_field_files() {
    let json = r#"{
        "repo_root": "/repo",
        "lang_summary": {},
        "total_tokens": 0,
        "total_bytes": 0,
        "total_code_lines": 0
    }"#;
    let result = serde_json::from_str::<RepoSubstrate>(json);
    assert!(result.is_err(), "missing 'files' should fail");
}

#[test]
fn reject_missing_required_field_total_tokens() {
    let json = r#"{
        "repo_root": "/repo",
        "files": [],
        "lang_summary": {},
        "total_bytes": 0,
        "total_code_lines": 0
    }"#;
    let result = serde_json::from_str::<RepoSubstrate>(json);
    assert!(result.is_err(), "missing 'total_tokens' should fail");
}

#[test]
fn reject_files_not_array() {
    let json = r#"{
        "repo_root": "/repo",
        "files": "not-an-array",
        "lang_summary": {},
        "total_tokens": 0,
        "total_bytes": 0,
        "total_code_lines": 0
    }"#;
    let result = serde_json::from_str::<RepoSubstrate>(json);
    assert!(result.is_err(), "'files' as string should fail");
}

#[test]
fn reject_total_code_lines_as_string() {
    let json = r#"{
        "repo_root": "/repo",
        "files": [],
        "lang_summary": {},
        "total_tokens": 0,
        "total_bytes": 0,
        "total_code_lines": "not a number"
    }"#;
    let result = serde_json::from_str::<RepoSubstrate>(json);
    assert!(result.is_err(), "string for usize field should fail");
}

#[test]
fn reject_negative_numeric_field() {
    let json = r#"{
        "repo_root": "/repo",
        "files": [],
        "lang_summary": {},
        "total_tokens": -1,
        "total_bytes": 0,
        "total_code_lines": 0
    }"#;
    let result = serde_json::from_str::<RepoSubstrate>(json);
    assert!(result.is_err(), "negative value for usize should fail");
}

#[test]
fn reject_substrate_file_missing_path() {
    let json = r#"{
        "lang": "Rust",
        "code": 10,
        "lines": 20,
        "bytes": 300,
        "tokens": 70,
        "module": "src",
        "in_diff": false
    }"#;
    let result = serde_json::from_str::<SubstrateFile>(json);
    assert!(result.is_err(), "missing 'path' should fail");
}

#[test]
fn reject_substrate_file_missing_in_diff() {
    let json = r#"{
        "path": "src/lib.rs",
        "lang": "Rust",
        "code": 10,
        "lines": 20,
        "bytes": 300,
        "tokens": 70,
        "module": "src"
    }"#;
    let result = serde_json::from_str::<SubstrateFile>(json);
    assert!(result.is_err(), "missing 'in_diff' should fail");
}

#[test]
fn reject_lang_summary_code_as_string() {
    let json = r#"{
        "files": 1,
        "code": "not-a-number",
        "lines": 20,
        "bytes": 300,
        "tokens": 70
    }"#;
    let result = serde_json::from_str::<LangSummary>(json);
    assert!(result.is_err(), "string for usize should fail");
}

// =============================================================================
// 4. Forward compatibility: extra/unknown fields ignored
// =============================================================================

#[test]
fn forward_compat_extra_substrate_fields_ignored() {
    let json = r#"{
        "repo_root": "/repo",
        "files": [],
        "lang_summary": {},
        "total_tokens": 0,
        "total_bytes": 0,
        "total_code_lines": 0,
        "future_field": "value",
        "future_number": 42
    }"#;
    let sub: RepoSubstrate = serde_json::from_str(json).unwrap();
    assert_eq!(sub.repo_root, "/repo");
}

#[test]
fn forward_compat_extra_file_fields_ignored() {
    let json = r#"{
        "path": "src/lib.rs",
        "lang": "Rust",
        "code": 100,
        "lines": 120,
        "bytes": 3000,
        "tokens": 700,
        "module": "src",
        "in_diff": true,
        "future_complexity": 42,
        "future_tags": ["a"]
    }"#;
    let file: SubstrateFile = serde_json::from_str(json).unwrap();
    assert_eq!(file.path, "src/lib.rs");
    assert_eq!(file.code, 100);
    assert!(file.in_diff);
}

#[test]
fn forward_compat_extra_diff_range_fields_ignored() {
    let json = r#"{
        "base": "main",
        "head": "dev",
        "changed_files": [],
        "commit_count": 0,
        "insertions": 0,
        "deletions": 0,
        "future_merge_base": "abc123"
    }"#;
    let dr: DiffRange = serde_json::from_str(json).unwrap();
    assert_eq!(dr.base, "main");
}

#[test]
fn forward_compat_extra_lang_summary_fields_ignored() {
    let json = r#"{
        "files": 5,
        "code": 200,
        "lines": 250,
        "bytes": 6000,
        "tokens": 1400,
        "future_avg_complexity": 3.5
    }"#;
    let ls: LangSummary = serde_json::from_str(json).unwrap();
    assert_eq!(ls.files, 5);
    assert_eq!(ls.code, 200);
}

// =============================================================================
// 5. JSON structure invariants
// =============================================================================

#[test]
fn json_required_keys_present_in_populated_substrate() {
    let value = serde_json::to_value(populated_substrate()).unwrap();
    let obj = value.as_object().unwrap();
    for key in [
        "repo_root",
        "files",
        "lang_summary",
        "total_tokens",
        "total_bytes",
        "total_code_lines",
    ] {
        assert!(obj.contains_key(key), "missing required key: {key}");
    }
    // diff_range is optional but present here
    assert!(obj.contains_key("diff_range"));
}

#[test]
fn json_file_has_all_required_keys() {
    let value = serde_json::to_value(populated_substrate()).unwrap();
    let file = value["files"][0].as_object().unwrap();
    for key in [
        "path", "lang", "code", "lines", "bytes", "tokens", "module", "in_diff",
    ] {
        assert!(file.contains_key(key), "file missing key: {key}");
    }
}

#[test]
fn json_lang_summary_has_all_required_keys() {
    let value = serde_json::to_value(populated_substrate()).unwrap();
    let rust = value["lang_summary"]["Rust"].as_object().unwrap();
    for key in ["files", "code", "lines", "bytes", "tokens"] {
        assert!(rust.contains_key(key), "lang_summary missing key: {key}");
    }
}

#[test]
fn json_diff_range_has_all_required_keys() {
    let value = serde_json::to_value(populated_substrate()).unwrap();
    let dr = value["diff_range"].as_object().unwrap();
    for key in [
        "base",
        "head",
        "changed_files",
        "commit_count",
        "insertions",
        "deletions",
    ] {
        assert!(dr.contains_key(key), "diff_range missing key: {key}");
    }
}

#[test]
fn json_lang_summary_keys_sorted() {
    let value = serde_json::to_value(populated_substrate()).unwrap();
    let summary = value["lang_summary"].as_object().unwrap();
    let keys: Vec<&String> = summary.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "lang_summary keys should be sorted");
}

// =============================================================================
// 6. Extreme values
// =============================================================================

#[test]
fn extreme_usize_values_roundtrip() {
    let file = SubstrateFile {
        path: "extreme.rs".to_string(),
        lang: "Rust".to_string(),
        code: usize::MAX,
        lines: usize::MAX,
        bytes: usize::MAX,
        tokens: usize::MAX,
        module: "".to_string(),
        in_diff: true,
    };
    let json = serde_json::to_string(&file).unwrap();
    let back: SubstrateFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.code, usize::MAX);
    assert_eq!(back.lines, usize::MAX);
    assert_eq!(back.bytes, usize::MAX);
    assert_eq!(back.tokens, usize::MAX);
}

#[test]
fn zero_values_file_roundtrip() {
    let file = SubstrateFile {
        path: "empty.rs".to_string(),
        lang: "Rust".to_string(),
        code: 0,
        lines: 0,
        bytes: 0,
        tokens: 0,
        module: "".to_string(),
        in_diff: false,
    };
    let json = serde_json::to_string(&file).unwrap();
    let back: SubstrateFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.code, 0);
    assert_eq!(back.lines, 0);
    assert_eq!(back.bytes, 0);
    assert_eq!(back.tokens, 0);
}

#[test]
fn extreme_diff_range_roundtrip() {
    let dr = DiffRange {
        base: "a".repeat(500),
        head: "b".repeat(500),
        changed_files: (0..500).map(|i| format!("file_{i}.rs")).collect(),
        commit_count: usize::MAX,
        insertions: usize::MAX,
        deletions: usize::MAX,
    };
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert_eq!(back.changed_files.len(), 500);
    assert_eq!(back.commit_count, usize::MAX);
}

// =============================================================================
// 7. Empty string edge cases
// =============================================================================

#[test]
fn empty_strings_roundtrip() {
    let sub = RepoSubstrate {
        repo_root: String::new(),
        files: vec![SubstrateFile {
            path: String::new(),
            lang: String::new(),
            code: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
            module: String::new(),
            in_diff: false,
        }],
        lang_summary: {
            let mut m = BTreeMap::new();
            m.insert(
                String::new(),
                LangSummary {
                    files: 1,
                    code: 0,
                    lines: 0,
                    bytes: 0,
                    tokens: 0,
                },
            );
            m
        },
        diff_range: Some(DiffRange {
            base: String::new(),
            head: String::new(),
            changed_files: vec![String::new()],
            commit_count: 0,
            insertions: 0,
            deletions: 0,
        }),
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.repo_root, "");
    assert_eq!(back.files[0].path, "");
    assert_eq!(back.files[0].lang, "");
    assert!(back.lang_summary.contains_key(""));
    let dr = back.diff_range.unwrap();
    assert_eq!(dr.base, "");
    assert_eq!(dr.changed_files, vec![""]);
}

// =============================================================================
// 8. Clone deep equality
// =============================================================================

#[test]
fn clone_produces_identical_json() {
    let original = populated_substrate();
    let cloned = original.clone();
    let j1 = serde_json::to_string(&original).unwrap();
    let j2 = serde_json::to_string(&cloned).unwrap();
    assert_eq!(j1, j2);
}

// =============================================================================
// 9. diff_files and files_for_lang interaction
// =============================================================================

#[test]
fn diff_files_subset_of_files_for_lang() {
    let sub = populated_substrate();
    let diff_rust: Vec<_> = sub.diff_files().filter(|f| f.lang == "Rust").collect();
    let all_rust: Vec<_> = sub.files_for_lang("Rust").collect();
    // Every diff Rust file should also be in all_rust
    for df in &diff_rust {
        assert!(
            all_rust.iter().any(|f| f.path == df.path),
            "diff file {} not found in files_for_lang",
            df.path
        );
    }
    assert!(diff_rust.len() <= all_rust.len());
}

#[test]
fn diff_files_count_consistent_with_in_diff_flag() {
    let sub = populated_substrate();
    let expected = sub.files.iter().filter(|f| f.in_diff).count();
    assert_eq!(sub.diff_files().count(), expected);
}

// =============================================================================
// 10. Deserialization from external JSON fixture
// =============================================================================

#[test]
fn deserialize_minimal_external_json() {
    let json = r#"{
        "repo_root": "/external/repo",
        "files": [],
        "lang_summary": {},
        "total_tokens": 0,
        "total_bytes": 0,
        "total_code_lines": 0
    }"#;
    let sub: RepoSubstrate = serde_json::from_str(json).unwrap();
    assert_eq!(sub.repo_root, "/external/repo");
    assert!(sub.files.is_empty());
    assert!(sub.diff_range.is_none());
}

#[test]
fn deserialize_complete_external_json() {
    let json = r#"{
        "repo_root": "/ext",
        "files": [
            {
                "path": "src/main.go",
                "lang": "Go",
                "code": 500,
                "lines": 600,
                "bytes": 15000,
                "tokens": 3500,
                "module": "src",
                "in_diff": true
            },
            {
                "path": "go.mod",
                "lang": "Go Module",
                "code": 10,
                "lines": 15,
                "bytes": 300,
                "tokens": 70,
                "module": "",
                "in_diff": false
            }
        ],
        "lang_summary": {
            "Go": {"files": 1, "code": 500, "lines": 600, "bytes": 15000, "tokens": 3500},
            "Go Module": {"files": 1, "code": 10, "lines": 15, "bytes": 300, "tokens": 70}
        },
        "diff_range": {
            "base": "main",
            "head": "feature/api",
            "changed_files": ["src/main.go"],
            "commit_count": 3,
            "insertions": 25,
            "deletions": 10
        },
        "total_tokens": 3570,
        "total_bytes": 15300,
        "total_code_lines": 510
    }"#;
    let sub: RepoSubstrate = serde_json::from_str(json).unwrap();
    assert_eq!(sub.files.len(), 2);
    assert_eq!(sub.lang_summary.len(), 2);
    assert_eq!(sub.files[0].lang, "Go");
    assert!(sub.files[0].in_diff);
    assert!(!sub.files[1].in_diff);
    let dr = sub.diff_range.unwrap();
    assert_eq!(dr.base, "main");
    assert_eq!(dr.changed_files, vec!["src/main.go"]);
    assert_eq!(sub.total_code_lines, 510);
}

// =============================================================================
// Property tests
// =============================================================================

fn arb_lang_summary() -> impl Strategy<Value = LangSummary> {
    (
        0..100usize,
        0..10_000usize,
        0..20_000usize,
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
        "[a-z/]{1,40}",
        "[A-Za-z]{1,15}",
        0..10_000usize,
        0..10_000usize,
        0..1_000_000usize,
        0..100_000usize,
        "[a-z/]{0,20}",
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
        "[a-z0-9./-]{1,20}",
        "[a-z0-9./-]{1,20}",
        prop::collection::vec("[a-z/.]{1,30}", 0..10),
        0..500usize,
        0..10_000usize,
        0..10_000usize,
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

proptest! {
    /// Serialization is deterministic for arbitrary substrates.
    #[test]
    fn prop_serialization_deterministic(
        root in "[a-z/]{1,30}",
        files in prop::collection::vec(arb_substrate_file(), 0..5),
        langs in prop::collection::btree_map("[A-Za-z]{1,10}", arb_lang_summary(), 0..3),
        diff_range in prop::option::of(arb_diff_range()),
        total_tokens in 0usize..100_000,
        total_bytes in 0usize..1_000_000,
        total_code_lines in 0usize..100_000,
    ) {
        let sub = RepoSubstrate {
            repo_root: root,
            files,
            lang_summary: langs,
            diff_range,
            total_tokens,
            total_bytes,
            total_code_lines,
        };
        let j1 = serde_json::to_string(&sub).unwrap();
        let j2 = serde_json::to_string(&sub).unwrap();
        prop_assert_eq!(&j1, &j2);
    }

    /// Double roundtrip is always stable.
    #[test]
    fn prop_double_roundtrip_stable(
        root in "[a-z/]{1,30}",
        files in prop::collection::vec(arb_substrate_file(), 0..5),
        total in 0usize..100_000,
    ) {
        let sub = RepoSubstrate {
            repo_root: root,
            files,
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: total,
            total_bytes: total,
            total_code_lines: total,
        };
        let json1 = serde_json::to_string(&sub).unwrap();
        let mid: RepoSubstrate = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&mid).unwrap();
        prop_assert_eq!(json1, json2);
    }

    /// diff_files() always returns a subset of files.
    #[test]
    fn prop_diff_files_is_subset(
        files in prop::collection::vec(arb_substrate_file(), 0..20),
    ) {
        let sub = RepoSubstrate {
            repo_root: "/r".to_string(),
            files,
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: 0,
        };
        let diff_count = sub.diff_files().count();
        let expected = sub.files.iter().filter(|f| f.in_diff).count();
        prop_assert_eq!(diff_count, expected);
        prop_assert!(diff_count <= sub.files.len());
    }

    /// files_for_lang returns only matching files.
    #[test]
    fn prop_files_for_lang_correct(
        files in prop::collection::vec(arb_substrate_file(), 0..20),
        lang in "[A-Za-z]{1,10}",
    ) {
        let sub = RepoSubstrate {
            repo_root: "/r".to_string(),
            files,
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: 0,
        };
        for f in sub.files_for_lang(&lang) {
            prop_assert_eq!(&f.lang, &lang);
        }
        let count = sub.files_for_lang(&lang).count();
        let expected = sub.files.iter().filter(|f| f.lang == lang).count();
        prop_assert_eq!(count, expected);
    }
}
