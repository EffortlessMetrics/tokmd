//! Expanded contract tests for tokmd-substrate.
//!
//! Covers all-files-in-diff scenarios, many-language substrates,
//! forward-slash path contracts, and diff_range absence.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// =============================================================================
// Helpers
// =============================================================================

fn make_file(path: &str, lang: &str, code: usize, in_diff: bool) -> SubstrateFile {
    SubstrateFile {
        path: path.to_string(),
        lang: lang.to_string(),
        code,
        lines: code + code / 5,
        bytes: code * 30,
        tokens: code * 3,
        module: path
            .rsplit_once('/')
            .map_or("(root)", |(dir, _)| dir)
            .to_string(),
        in_diff,
    }
}

fn make_lang_summary(files: usize, code: usize) -> LangSummary {
    LangSummary {
        files,
        code,
        lines: code + code / 5,
        bytes: code * 30,
        tokens: code * 3,
    }
}

// =============================================================================
// Scenario: All files have in_diff=true
// =============================================================================

#[test]
fn given_substrate_with_all_files_in_diff_when_diff_files_called_then_returns_all() {
    let substrate = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![
            make_file("src/lib.rs", "Rust", 100, true),
            make_file("src/main.rs", "Rust", 50, true),
            make_file("tests/test.rs", "Rust", 30, true),
        ],
        lang_summary: BTreeMap::from([("Rust".to_string(), make_lang_summary(3, 180))]),
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "feature".to_string(),
            changed_files: vec![
                "src/lib.rs".to_string(),
                "src/main.rs".to_string(),
                "tests/test.rs".to_string(),
            ],
            commit_count: 5,
            insertions: 200,
            deletions: 50,
        }),
        total_tokens: 540,
        total_bytes: 5400,
        total_code_lines: 180,
    };

    let diff_files: Vec<_> = substrate.diff_files().collect();
    assert_eq!(diff_files.len(), 3);
    assert!(diff_files.iter().all(|f| f.in_diff));
}

// =============================================================================
// Scenario: No files in diff
// =============================================================================

#[test]
fn given_substrate_with_no_files_in_diff_when_diff_files_called_then_empty() {
    let substrate = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![
            make_file("src/lib.rs", "Rust", 100, false),
            make_file("src/main.rs", "Rust", 50, false),
        ],
        lang_summary: BTreeMap::from([("Rust".to_string(), make_lang_summary(2, 150))]),
        diff_range: None,
        total_tokens: 450,
        total_bytes: 4500,
        total_code_lines: 150,
    };

    let diff_files: Vec<_> = substrate.diff_files().collect();
    assert!(diff_files.is_empty());
}

// =============================================================================
// Scenario: Substrate with many languages
// =============================================================================

#[test]
fn given_substrate_with_many_langs_when_roundtripped_then_btreemap_order_preserved() {
    let langs = [
        "C", "C++", "Go", "Haskell", "Java", "JavaScript", "Kotlin", "Python", "Ruby", "Rust",
        "TypeScript", "Zig",
    ];

    let mut files = Vec::new();
    let mut lang_summary = BTreeMap::new();

    for (i, lang) in langs.iter().enumerate() {
        let code = (i + 1) * 100;
        files.push(make_file(
            &format!("src/{}.ext", lang.to_lowercase()),
            lang,
            code,
            i % 3 == 0,
        ));
        lang_summary.insert(lang.to_string(), make_lang_summary(1, code));
    }

    let substrate = RepoSubstrate {
        repo_root: "/polyglot-repo".to_string(),
        files,
        lang_summary,
        diff_range: None,
        total_tokens: 23400,
        total_bytes: 234_000,
        total_code_lines: 7800,
    };

    let json = serde_json::to_string(&substrate).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();

    // BTreeMap keys should be in alphabetical order
    let keys: Vec<_> = back.lang_summary.keys().collect();
    let mut sorted_keys = keys.clone();
    sorted_keys.sort();
    assert_eq!(keys, sorted_keys, "BTreeMap keys must be sorted");

    assert_eq!(back.lang_summary.len(), 12);
    assert_eq!(back.files.len(), 12);

    // files_for_lang should work correctly
    for lang in &langs {
        let lang_files: Vec<_> = back.files_for_lang(lang).collect();
        assert_eq!(lang_files.len(), 1);
    }
}

// =============================================================================
// Scenario: Forward-slash path contract
// =============================================================================

#[test]
fn given_substrate_files_when_paths_inspected_then_no_backslashes() {
    let substrate = RepoSubstrate {
        repo_root: "/home/user/project".to_string(),
        files: vec![
            make_file("src/lib.rs", "Rust", 100, true),
            make_file("src/core/mod.rs", "Rust", 50, false),
            make_file("tests/integration/test_api.rs", "Rust", 30, true),
        ],
        lang_summary: BTreeMap::from([("Rust".to_string(), make_lang_summary(3, 180))]),
        diff_range: None,
        total_tokens: 540,
        total_bytes: 5400,
        total_code_lines: 180,
    };

    // Repo root should use forward slashes
    assert!(!substrate.repo_root.contains('\\'));

    // All file paths should use forward slashes
    for file in &substrate.files {
        assert!(
            !file.path.contains('\\'),
            "Path '{}' should not contain backslashes",
            file.path
        );
    }
}

// =============================================================================
// Scenario: DiffRange roundtrip with large numbers
// =============================================================================

#[test]
fn given_diff_range_with_large_values_when_roundtripped_then_preserved() {
    let range = DiffRange {
        base: "v1.0.0".to_string(),
        head: "v2.0.0".to_string(),
        changed_files: (0..100)
            .map(|i| format!("src/file{i}.rs"))
            .collect(),
        commit_count: 1000,
        insertions: 50_000,
        deletions: 30_000,
    };

    let json = serde_json::to_string(&range).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();

    assert_eq!(back.base, "v1.0.0");
    assert_eq!(back.head, "v2.0.0");
    assert_eq!(back.changed_files.len(), 100);
    assert_eq!(back.commit_count, 1000);
    assert_eq!(back.insertions, 50_000);
    assert_eq!(back.deletions, 30_000);
}

// =============================================================================
// Scenario: Empty substrate (zero files)
// =============================================================================

#[test]
fn given_empty_substrate_when_roundtripped_then_all_zeros_preserved() {
    let substrate = RepoSubstrate {
        repo_root: "/empty".to_string(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };

    let json = serde_json::to_string(&substrate).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();

    assert!(back.files.is_empty());
    assert!(back.lang_summary.is_empty());
    assert!(back.diff_range.is_none());
    assert_eq!(back.total_tokens, 0);

    // Methods should work on empty substrate
    assert_eq!(back.diff_files().count(), 0);
    assert_eq!(back.files_for_lang("Rust").count(), 0);
}

// =============================================================================
// Scenario: Substrate without diff_range omits it from JSON
// =============================================================================

#[test]
fn given_substrate_without_diff_range_when_serialized_then_field_omitted() {
    let substrate = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![make_file("src/lib.rs", "Rust", 100, false)],
        lang_summary: BTreeMap::from([("Rust".to_string(), make_lang_summary(1, 100))]),
        diff_range: None,
        total_tokens: 300,
        total_bytes: 3000,
        total_code_lines: 100,
    };

    let json = serde_json::to_string(&substrate).unwrap();
    assert!(
        !json.contains("\"diff_range\""),
        "diff_range should be omitted when None"
    );
}

// =============================================================================
// Property: SubstrateFile roundtrip preserves all numeric fields
// =============================================================================

proptest! {
    #[test]
    fn prop_substrate_file_roundtrip(
        code in 0usize..100_000,
        lines in 0usize..200_000,
        bytes in 0usize..10_000_000,
        tokens in 0usize..1_000_000,
        in_diff in any::<bool>(),
    ) {
        let file = SubstrateFile {
            path: "src/lib.rs".to_string(),
            lang: "Rust".to_string(),
            code,
            lines,
            bytes,
            tokens,
            module: "src".to_string(),
            in_diff,
        };

        let json = serde_json::to_string(&file).unwrap();
        let back: SubstrateFile = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(back.code, code);
        prop_assert_eq!(back.lines, lines);
        prop_assert_eq!(back.bytes, bytes);
        prop_assert_eq!(back.tokens, tokens);
        prop_assert_eq!(back.in_diff, in_diff);
    }

    #[test]
    fn prop_diff_files_count_matches_in_diff_true_count(
        n_files in 1usize..50,
        diff_pct in 0usize..100,
    ) {
        let files: Vec<SubstrateFile> = (0..n_files)
            .map(|i| make_file(
                &format!("src/file{i}.rs"),
                "Rust",
                100,
                (i * 100 / n_files) < diff_pct,
            ))
            .collect();

        let expected_diff_count = files.iter().filter(|f| f.in_diff).count();

        let substrate = RepoSubstrate {
            repo_root: "/repo".to_string(),
            files,
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: 0,
        };

        prop_assert_eq!(substrate.diff_files().count(), expected_diff_count);
    }
}
