//! Deep contract tests (part 2) for `tokmd-substrate`.
//!
//! Extends coverage beyond `deep.rs` with: Unicode paths, case-sensitive
//! language filtering, single-file substrates, clone mutation independence,
//! JSON value-type invariants, large multi-language substrates, DiffRange
//! edge cases, and combined filter chains.

use std::collections::BTreeMap;

use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

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

fn make_lang(files: usize, code: usize) -> LangSummary {
    LangSummary {
        files,
        code,
        lines: code + files * 20,
        bytes: code * 30,
        tokens: code * 7,
    }
}

fn single_file_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![make_file("src/lib.rs", "Rust", 100, true)],
        lang_summary: {
            let mut m = BTreeMap::new();
            m.insert("Rust".to_string(), make_lang(1, 100));
            m
        },
        diff_range: None,
        total_tokens: 700,
        total_bytes: 3000,
        total_code_lines: 100,
    }
}

// =============================================================================
// 1. Unicode in file paths roundtrip
// =============================================================================

#[test]
fn unicode_file_paths_roundtrip() {
    let files = vec![
        make_file("src/日本語/ファイル.rs", "Rust", 50, true),
        make_file("src/中文/文件.rs", "Rust", 30, false),
        make_file("src/한국어/파일.rs", "Rust", 20, true),
    ];
    let sub = RepoSubstrate {
        repo_root: "/home/ユーザー/プロジェクト".to_string(),
        files,
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 100,
    };
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 3);
    assert!(back.files[0].path.contains("日本語"));
    assert!(back.repo_root.contains("ユーザー"));
}

// =============================================================================
// 2. Unicode in language names
// =============================================================================

#[test]
fn unicode_language_names_roundtrip() {
    let mut summary = BTreeMap::new();
    summary.insert("日本語".to_string(), make_lang(1, 50));
    summary.insert("Ελληνικά".to_string(), make_lang(2, 100));

    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![
            make_file("a.txt", "日本語", 50, false),
            make_file("b.txt", "Ελληνικά", 100, false),
        ],
        lang_summary: summary,
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 150,
    };
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.lang_summary.contains_key("日本語"));
    assert!(back.lang_summary.contains_key("Ελληνικά"));
    assert_eq!(back.files_for_lang("日本語").count(), 1);
}

// =============================================================================
// 3. files_for_lang is case-sensitive
// =============================================================================

#[test]
fn files_for_lang_is_case_sensitive() {
    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![
            make_file("a.rs", "Rust", 10, false),
            make_file("b.rs", "rust", 20, false),
            make_file("c.rs", "RUST", 30, false),
        ],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 60,
    };
    assert_eq!(sub.files_for_lang("Rust").count(), 1);
    assert_eq!(sub.files_for_lang("rust").count(), 1);
    assert_eq!(sub.files_for_lang("RUST").count(), 1);
    assert_eq!(sub.files_for_lang("rUsT").count(), 0);
}

// =============================================================================
// 4. Single file substrate methods work correctly
// =============================================================================

#[test]
fn single_file_substrate_methods() {
    let sub = single_file_substrate();
    assert_eq!(sub.files.len(), 1);
    assert_eq!(sub.diff_files().count(), 1);
    assert_eq!(sub.files_for_lang("Rust").count(), 1);
    assert_eq!(sub.files_for_lang("Go").count(), 0);
}

// =============================================================================
// 5. Clone mutation independence
// =============================================================================

#[test]
fn clone_mutation_independence() {
    let original = single_file_substrate();
    let mut cloned = original.clone();

    // Mutate the clone
    cloned.repo_root = "/other".to_string();
    cloned.total_code_lines = 999;
    cloned.files.push(make_file("new.rs", "Rust", 50, false));
    cloned
        .lang_summary
        .insert("Go".to_string(), make_lang(1, 50));

    // Original should be unchanged
    assert_eq!(original.repo_root, "/repo");
    assert_eq!(original.total_code_lines, 100);
    assert_eq!(original.files.len(), 1);
    assert!(!original.lang_summary.contains_key("Go"));
}

// =============================================================================
// 6. JSON value types for numeric fields
// =============================================================================

#[test]
fn json_numeric_fields_are_numbers() {
    let sub = single_file_substrate();
    let value = serde_json::to_value(sub).unwrap();

    assert!(value["total_tokens"].is_number());
    assert!(value["total_bytes"].is_number());
    assert!(value["total_code_lines"].is_number());

    let file = &value["files"][0];
    assert!(file["code"].is_number());
    assert!(file["lines"].is_number());
    assert!(file["bytes"].is_number());
    assert!(file["tokens"].is_number());
    assert!(file["in_diff"].is_boolean());
    assert!(file["path"].is_string());
    assert!(file["lang"].is_string());
    assert!(file["module"].is_string());
}

// =============================================================================
// 7. LangSummary JSON value types
// =============================================================================

#[test]
fn lang_summary_json_value_types() {
    let ls = make_lang(5, 100);
    let value = serde_json::to_value(ls).unwrap();
    assert!(value["files"].is_number());
    assert!(value["code"].is_number());
    assert!(value["lines"].is_number());
    assert!(value["bytes"].is_number());
    assert!(value["tokens"].is_number());
}

// =============================================================================
// 8. DiffRange with empty changed_files
// =============================================================================

#[test]
fn diff_range_empty_changed_files_roundtrip() {
    let dr = DiffRange {
        base: "main".to_string(),
        head: "dev".to_string(),
        changed_files: vec![],
        commit_count: 0,
        insertions: 0,
        deletions: 0,
    };
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert!(back.changed_files.is_empty());
    assert_eq!(back.commit_count, 0);
}

// =============================================================================
// 9. DiffRange with single changed file
// =============================================================================

#[test]
fn diff_range_single_changed_file() {
    let dr = DiffRange {
        base: "v1".to_string(),
        head: "v2".to_string(),
        changed_files: vec!["only.rs".to_string()],
        commit_count: 1,
        insertions: 5,
        deletions: 3,
    };
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert_eq!(back.changed_files, vec!["only.rs"]);
}

// =============================================================================
// 10. Many languages with BTreeMap deterministic ordering
// =============================================================================

#[test]
fn many_languages_btreemap_ordering() {
    let langs = [
        "Zig",
        "Yaml",
        "XML",
        "TypeScript",
        "Swift",
        "SQL",
        "Shell",
        "Rust",
        "Ruby",
        "Python",
        "PHP",
        "Perl",
        "OCaml",
        "Nim",
        "Markdown",
        "Lua",
        "Kotlin",
        "Java",
        "JavaScript",
        "Haskell",
        "Go",
        "Fortran",
        "Erlang",
        "Dart",
        "C++",
        "C#",
        "C",
        "Assembly",
    ];
    let mut summary = BTreeMap::new();
    for lang in &langs {
        summary.insert(lang.to_string(), make_lang(1, 100));
    }

    let sub = RepoSubstrate {
        repo_root: "/polyglot".to_string(),
        files: vec![],
        lang_summary: summary,
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };

    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();

    let keys: Vec<&String> = back.lang_summary.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
    assert_eq!(keys.len(), langs.len());
}

// =============================================================================
// 11. Substrate with all files in same module
// =============================================================================

#[test]
fn all_files_same_module() {
    let files: Vec<_> = (0..10)
        .map(|i| {
            make_file(
                &format!("src/file_{}.rs", i),
                "Rust",
                10 * (i + 1),
                i % 2 == 0,
            )
        })
        .collect();

    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files,
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 550,
    };

    // All files should have module "src"
    for f in &sub.files {
        assert_eq!(f.module, "src");
    }
    assert_eq!(sub.diff_files().count(), 5); // i=0,2,4,6,8
}

// =============================================================================
// 12. Root-level file has empty module
// =============================================================================

#[test]
fn root_level_file_empty_module() {
    let f = make_file("Cargo.toml", "TOML", 20, false);
    assert_eq!(f.module, "");
}

// =============================================================================
// 13. diff_files chained with files_for_lang
// =============================================================================

#[test]
fn diff_files_intersection_with_lang_filter() {
    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![
            make_file("src/lib.rs", "Rust", 100, true),
            make_file("src/main.rs", "Rust", 50, false),
            make_file("src/app.py", "Python", 80, true),
            make_file("src/util.py", "Python", 40, false),
        ],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 270,
    };

    // Diff files that are Rust
    let rust_diff: Vec<_> = sub.diff_files().filter(|f| f.lang == "Rust").collect();
    assert_eq!(rust_diff.len(), 1);
    assert_eq!(rust_diff[0].path, "src/lib.rs");

    // Diff files that are Python
    let py_diff: Vec<_> = sub.diff_files().filter(|f| f.lang == "Python").collect();
    assert_eq!(py_diff.len(), 1);
    assert_eq!(py_diff[0].path, "src/app.py");
}

// =============================================================================
// 14. LangSummary with zero values
// =============================================================================

#[test]
fn lang_summary_zero_values_roundtrip() {
    let ls = LangSummary {
        files: 0,
        code: 0,
        lines: 0,
        bytes: 0,
        tokens: 0,
    };
    let json = serde_json::to_string(&ls).unwrap();
    let back: LangSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files, 0);
    assert_eq!(back.code, 0);
}

// =============================================================================
// 15. LangSummary with max values
// =============================================================================

#[test]
fn lang_summary_max_values_roundtrip() {
    let ls = LangSummary {
        files: usize::MAX,
        code: usize::MAX,
        lines: usize::MAX,
        bytes: usize::MAX,
        tokens: usize::MAX,
    };
    let json = serde_json::to_string(&ls).unwrap();
    let back: LangSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files, usize::MAX);
    assert_eq!(back.tokens, usize::MAX);
}

// =============================================================================
// 16. Substrate JSON pretty vs compact equivalence
// =============================================================================

#[test]
fn pretty_vs_compact_json_data_equivalence() {
    let sub = single_file_substrate();
    let compact = serde_json::to_string(&sub).unwrap();
    let pretty = serde_json::to_string_pretty(&sub).unwrap();

    let from_compact: RepoSubstrate = serde_json::from_str(&compact).unwrap();
    let from_pretty: RepoSubstrate = serde_json::from_str(&pretty).unwrap();

    let re1 = serde_json::to_string(&from_compact).unwrap();
    let re2 = serde_json::to_string(&from_pretty).unwrap();
    assert_eq!(re1, re2);
}

// =============================================================================
// 17. SubstrateFile with special characters in path
// =============================================================================

#[test]
fn substrate_file_special_chars_in_path() {
    let special_paths = [
        "src/file with spaces.rs",
        "src/file-with-dashes.rs",
        "src/file_with_underscores.rs",
        "src/file.multiple.dots.rs",
        "src/@scope/package.ts",
    ];

    for path in &special_paths {
        let f = SubstrateFile {
            path: path.to_string(),
            lang: "Rust".to_string(),
            code: 10,
            lines: 20,
            bytes: 300,
            tokens: 70,
            module: "src".to_string(),
            in_diff: false,
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: SubstrateFile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, *path);
    }
}

// =============================================================================
// 18. DiffRange changed_files order preserved
// =============================================================================

#[test]
fn diff_range_changed_files_order_preserved() {
    let files: Vec<String> = (0..20).rev().map(|i| format!("file_{:03}.rs", i)).collect();
    let dr = DiffRange {
        base: "main".to_string(),
        head: "dev".to_string(),
        changed_files: files.clone(),
        commit_count: 20,
        insertions: 100,
        deletions: 50,
    };
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert_eq!(back.changed_files, files);
}

// =============================================================================
// 19. Deserialization rejects wrong types for boolean field
// =============================================================================

#[test]
fn reject_in_diff_as_string() {
    let json = r#"{
        "path": "a.rs",
        "lang": "Rust",
        "code": 10,
        "lines": 20,
        "bytes": 300,
        "tokens": 70,
        "module": "src",
        "in_diff": "yes"
    }"#;
    assert!(serde_json::from_str::<SubstrateFile>(json).is_err());
}

#[test]
fn reject_in_diff_as_number() {
    let json = r#"{
        "path": "a.rs",
        "lang": "Rust",
        "code": 10,
        "lines": 20,
        "bytes": 300,
        "tokens": 70,
        "module": "src",
        "in_diff": 1
    }"#;
    assert!(serde_json::from_str::<SubstrateFile>(json).is_err());
}

// =============================================================================
// 20. Substrate with duplicate file paths (degenerate case)
// =============================================================================

#[test]
fn substrate_with_duplicate_paths() {
    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![
            make_file("src/lib.rs", "Rust", 100, true),
            make_file("src/lib.rs", "Rust", 200, false),
        ],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 300,
    };

    // Struct doesn't enforce uniqueness—both entries survive
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 2);
    // diff_files returns 1 (only first has in_diff=true)
    assert_eq!(back.diff_files().count(), 1);
    // files_for_lang returns both
    assert_eq!(back.files_for_lang("Rust").count(), 2);
}

// =============================================================================
// 21. DiffRange clone independence
// =============================================================================

#[test]
fn diff_range_clone_independence() {
    let original = DiffRange {
        base: "main".to_string(),
        head: "dev".to_string(),
        changed_files: vec!["a.rs".to_string()],
        commit_count: 3,
        insertions: 10,
        deletions: 5,
    };
    let mut cloned = original.clone();
    cloned.base = "release".to_string();
    cloned.changed_files.push("b.rs".to_string());

    assert_eq!(original.base, "main");
    assert_eq!(original.changed_files.len(), 1);
    assert_eq!(cloned.changed_files.len(), 2);
}

// =============================================================================
// 22. Substrate forward compat: extra fields in substrate file
// =============================================================================

#[test]
fn forward_compat_extra_substrate_file_nested_object() {
    let json = r#"{
        "path": "src/lib.rs",
        "lang": "Rust",
        "code": 100,
        "lines": 120,
        "bytes": 3000,
        "tokens": 700,
        "module": "src",
        "in_diff": true,
        "future_metadata": {"complexity": 12, "authors": ["alice", "bob"]}
    }"#;
    let file: SubstrateFile = serde_json::from_str(json).unwrap();
    assert_eq!(file.path, "src/lib.rs");
    assert!(file.in_diff);
}

// =============================================================================
// 23. Substrate total fields independent of file list
// =============================================================================

#[test]
fn substrate_totals_not_auto_computed() {
    // Total fields are stored, not computed from files
    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![make_file("a.rs", "Rust", 100, false)],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 999, // intentionally wrong
        total_bytes: 888,
        total_code_lines: 777,
    };
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    // The "wrong" values should be preserved as-is (pure data struct)
    assert_eq!(back.total_tokens, 999);
    assert_eq!(back.total_bytes, 888);
    assert_eq!(back.total_code_lines, 777);
}

// =============================================================================
// 24. Debug formatting on all types
// =============================================================================

#[test]
fn debug_formatting_all_types() {
    let sub = single_file_substrate();
    let _ = format!("{:?}", sub);
    let _ = format!("{:?}", sub.files[0]);

    let ls = make_lang(1, 100);
    let _ = format!("{:?}", ls);

    let dr = DiffRange {
        base: "a".into(),
        head: "b".into(),
        changed_files: vec![],
        commit_count: 0,
        insertions: 0,
        deletions: 0,
    };
    let _ = format!("{:?}", dr);
}

// =============================================================================
// 25. Substrate files Vec preserves insertion order
// =============================================================================

#[test]
fn substrate_files_preserve_insertion_order() {
    let files: Vec<SubstrateFile> = (0..10)
        .map(|i| make_file(&format!("file_{:02}.rs", 9 - i), "Rust", 10, false))
        .collect();

    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: files.clone(),
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    };

    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();

    for (i, f) in back.files.iter().enumerate() {
        assert_eq!(f.path, files[i].path);
    }
}

// =============================================================================
// 26. DiffRange with Unicode refs
// =============================================================================

#[test]
fn diff_range_unicode_refs() {
    let dr = DiffRange {
        base: "ブランチ/メイン".to_string(),
        head: "機能/新しい".to_string(),
        changed_files: vec!["ソース/ファイル.rs".to_string()],
        commit_count: 1,
        insertions: 5,
        deletions: 2,
    };
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "ブランチ/メイン");
    assert_eq!(back.head, "機能/新しい");
}

// =============================================================================
// 27. Reject DiffRange missing required field
// =============================================================================

#[test]
fn reject_diff_range_missing_base() {
    let json = r#"{
        "head": "dev",
        "changed_files": [],
        "commit_count": 0,
        "insertions": 0,
        "deletions": 0
    }"#;
    assert!(serde_json::from_str::<DiffRange>(json).is_err());
}

#[test]
fn reject_diff_range_missing_changed_files() {
    let json = r#"{
        "base": "main",
        "head": "dev",
        "commit_count": 0,
        "insertions": 0,
        "deletions": 0
    }"#;
    assert!(serde_json::from_str::<DiffRange>(json).is_err());
}
