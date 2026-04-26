//! Depth tests (w63) for tokmd-sensor::substrate: RepoSubstrate construction, language
//! summaries, file listing, total aggregation, empty/single-file handling,
//! substrate sharing, determinism, and property-based verification.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ===========================================================================
// Helpers
// ===========================================================================

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

fn single_file_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![make_file("src/lib.rs", "Rust", 100, false)],
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

fn multi_lang_substrate() -> RepoSubstrate {
    let files = vec![
        make_file("src/lib.rs", "Rust", 200, true),
        make_file("src/main.rs", "Rust", 80, false),
        make_file("app/index.ts", "TypeScript", 150, true),
        make_file("app/util.ts", "TypeScript", 60, false),
        make_file("scripts/build.py", "Python", 40, false),
    ];
    let total_code: usize = files.iter().map(|f| f.code).sum();
    let total_tokens: usize = files.iter().map(|f| f.tokens).sum();
    let total_bytes: usize = files.iter().map(|f| f.bytes).sum();
    let mut lang_summary = BTreeMap::new();
    lang_summary.insert("Rust".to_string(), make_lang(2, 280));
    lang_summary.insert("TypeScript".to_string(), make_lang(2, 210));
    lang_summary.insert("Python".to_string(), make_lang(1, 40));
    RepoSubstrate {
        repo_root: "/project".to_string(),
        files,
        lang_summary,
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "feature".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "app/index.ts".to_string()],
            commit_count: 5,
            insertions: 30,
            deletions: 10,
        }),
        total_tokens,
        total_bytes,
        total_code_lines: total_code,
    }
}

// ===========================================================================
// 1. RepoSubstrate construction from various scan results
// ===========================================================================

#[test]
fn substrate_from_empty_scan() {
    let sub = empty_substrate();
    assert!(sub.files.is_empty());
    assert!(sub.lang_summary.is_empty());
    assert_eq!(sub.total_code_lines, 0);
    assert_eq!(sub.total_tokens, 0);
    assert_eq!(sub.total_bytes, 0);
}

#[test]
fn substrate_from_single_file_scan() {
    let sub = single_file_substrate();
    assert_eq!(sub.files.len(), 1);
    assert_eq!(sub.lang_summary.len(), 1);
    assert_eq!(sub.total_code_lines, 100);
}

#[test]
fn substrate_from_multi_lang_scan() {
    let sub = multi_lang_substrate();
    assert_eq!(sub.files.len(), 5);
    assert_eq!(sub.lang_summary.len(), 3);
}

#[test]
fn substrate_repo_root_preserved() {
    let sub = multi_lang_substrate();
    assert_eq!(sub.repo_root, "/project");
}

#[test]
fn substrate_with_diff_range_populated() {
    let sub = multi_lang_substrate();
    assert!(sub.diff_range.is_some());
    let dr = sub.diff_range.as_ref().unwrap();
    assert_eq!(dr.base, "main");
    assert_eq!(dr.head, "feature");
    assert_eq!(dr.commit_count, 5);
}

#[test]
fn substrate_without_diff_range() {
    let sub = single_file_substrate();
    assert!(sub.diff_range.is_none());
}

// ===========================================================================
// 2. Language summary accuracy
// ===========================================================================

#[test]
fn lang_summary_rust_code_correct() {
    let sub = multi_lang_substrate();
    let rust = sub.lang_summary.get("Rust").unwrap();
    assert_eq!(rust.code, 280);
    assert_eq!(rust.files, 2);
}

#[test]
fn lang_summary_typescript_code_correct() {
    let sub = multi_lang_substrate();
    let ts = sub.lang_summary.get("TypeScript").unwrap();
    assert_eq!(ts.code, 210);
    assert_eq!(ts.files, 2);
}

#[test]
fn lang_summary_python_code_correct() {
    let sub = multi_lang_substrate();
    let py = sub.lang_summary.get("Python").unwrap();
    assert_eq!(py.code, 40);
    assert_eq!(py.files, 1);
}

#[test]
fn lang_summary_missing_language_returns_none() {
    let sub = multi_lang_substrate();
    assert!(!sub.lang_summary.contains_key("Go"));
    assert!(!sub.lang_summary.contains_key("Java"));
}

#[test]
fn lang_summary_btreemap_ordered() {
    let sub = multi_lang_substrate();
    let keys: Vec<&String> = sub.lang_summary.keys().collect();
    assert_eq!(keys, vec!["Python", "Rust", "TypeScript"]);
}

#[test]
fn lang_summary_empty_for_empty_substrate() {
    let sub = empty_substrate();
    assert!(sub.lang_summary.is_empty());
}

// ===========================================================================
// 3. File listing correctness
// ===========================================================================

#[test]
fn file_listing_count() {
    let sub = multi_lang_substrate();
    assert_eq!(sub.files.len(), 5);
}

#[test]
fn file_paths_present() {
    let sub = multi_lang_substrate();
    let paths: Vec<&str> = sub.files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"src/lib.rs"));
    assert!(paths.contains(&"src/main.rs"));
    assert!(paths.contains(&"app/index.ts"));
    assert!(paths.contains(&"app/util.ts"));
    assert!(paths.contains(&"scripts/build.py"));
}

#[test]
fn file_lang_assignments() {
    let sub = multi_lang_substrate();
    for f in &sub.files {
        match f.path.as_str() {
            "src/lib.rs" | "src/main.rs" => assert_eq!(f.lang, "Rust"),
            "app/index.ts" | "app/util.ts" => assert_eq!(f.lang, "TypeScript"),
            "scripts/build.py" => assert_eq!(f.lang, "Python"),
            _ => panic!("unexpected file: {}", f.path),
        }
    }
}

#[test]
fn file_module_computed_from_path() {
    let sub = multi_lang_substrate();
    for f in &sub.files {
        let expected_module = f.path.rsplit_once('/').map(|(m, _)| m).unwrap_or("");
        assert_eq!(f.module, expected_module, "module mismatch for {}", f.path);
    }
}

#[test]
fn file_in_diff_flags() {
    let sub = multi_lang_substrate();
    let diff_paths: Vec<&str> = sub
        .files
        .iter()
        .filter(|f| f.in_diff)
        .map(|f| f.path.as_str())
        .collect();
    assert_eq!(diff_paths.len(), 2);
    assert!(diff_paths.contains(&"src/lib.rs"));
    assert!(diff_paths.contains(&"app/index.ts"));
}

#[test]
fn file_not_in_diff_flags() {
    let sub = multi_lang_substrate();
    let non_diff: Vec<&str> = sub
        .files
        .iter()
        .filter(|f| !f.in_diff)
        .map(|f| f.path.as_str())
        .collect();
    assert_eq!(non_diff.len(), 3);
}

// ===========================================================================
// 4. Total line count aggregation
// ===========================================================================

#[test]
fn total_code_lines_matches_file_sum() {
    let sub = multi_lang_substrate();
    let sum: usize = sub.files.iter().map(|f| f.code).sum();
    assert_eq!(sub.total_code_lines, sum);
}

#[test]
fn total_tokens_matches_file_sum() {
    let sub = multi_lang_substrate();
    let sum: usize = sub.files.iter().map(|f| f.tokens).sum();
    assert_eq!(sub.total_tokens, sum);
}

#[test]
fn total_bytes_matches_file_sum() {
    let sub = multi_lang_substrate();
    let sum: usize = sub.files.iter().map(|f| f.bytes).sum();
    assert_eq!(sub.total_bytes, sum);
}

#[test]
fn total_code_lines_for_single_file() {
    let sub = single_file_substrate();
    assert_eq!(sub.total_code_lines, 100);
}

#[test]
fn total_code_lines_for_empty() {
    let sub = empty_substrate();
    assert_eq!(sub.total_code_lines, 0);
}

// ===========================================================================
// 5. Empty repository handling
// ===========================================================================

#[test]
fn empty_repo_diff_files_empty() {
    let sub = empty_substrate();
    assert_eq!(sub.diff_files().count(), 0);
}

#[test]
fn empty_repo_files_for_lang_empty() {
    let sub = empty_substrate();
    assert_eq!(sub.files_for_lang("Rust").count(), 0);
}

#[test]
fn empty_repo_serde_roundtrip() {
    let sub = empty_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.files.is_empty());
    assert_eq!(back.total_code_lines, 0);
}

#[test]
fn empty_repo_json_structure() {
    let sub = empty_substrate();
    let val: serde_json::Value = serde_json::to_value(&sub).unwrap();
    assert!(val["files"].as_array().unwrap().is_empty());
    assert!(val["lang_summary"].as_object().unwrap().is_empty());
    // diff_range should be absent (skip_serializing_if)
    assert!(val.get("diff_range").is_none());
}

// ===========================================================================
// 6. Single-file repository handling
// ===========================================================================

#[test]
fn single_file_repo_one_file() {
    let sub = single_file_substrate();
    assert_eq!(sub.files.len(), 1);
    assert_eq!(sub.files[0].path, "src/lib.rs");
}

#[test]
fn single_file_repo_one_language() {
    let sub = single_file_substrate();
    assert_eq!(sub.lang_summary.len(), 1);
    assert!(sub.lang_summary.contains_key("Rust"));
}

#[test]
fn single_file_repo_totals_match_file() {
    let sub = single_file_substrate();
    let f = &sub.files[0];
    assert_eq!(sub.total_code_lines, f.code);
    assert_eq!(sub.total_tokens, f.tokens);
    assert_eq!(sub.total_bytes, f.bytes);
}

#[test]
fn single_file_files_for_lang() {
    let sub = single_file_substrate();
    assert_eq!(sub.files_for_lang("Rust").count(), 1);
    assert_eq!(sub.files_for_lang("Python").count(), 0);
}

// ===========================================================================
// 7. Substrate sharing across sensors (clone independence)
// ===========================================================================

#[test]
fn cloned_substrate_is_independent() {
    let sub = multi_lang_substrate();
    let mut clone = sub.clone();
    clone.total_code_lines = 999;
    clone.files.push(make_file("extra.rs", "Rust", 50, false));
    assert_ne!(sub.total_code_lines, clone.total_code_lines);
    assert_ne!(sub.files.len(), clone.files.len());
}

#[test]
fn cloned_substrate_files_are_deep_copies() {
    let sub = multi_lang_substrate();
    let mut clone = sub.clone();
    clone.files[0].code = 9999;
    assert_ne!(sub.files[0].code, clone.files[0].code);
}

#[test]
fn cloned_substrate_lang_summary_independent() {
    let sub = multi_lang_substrate();
    let mut clone = sub.clone();
    clone
        .lang_summary
        .insert("Go".to_string(), make_lang(1, 50));
    assert!(!sub.lang_summary.contains_key("Go"));
    assert!(clone.lang_summary.contains_key("Go"));
}

#[test]
fn cloned_substrate_diff_range_independent() {
    let sub = multi_lang_substrate();
    let mut clone = sub.clone();
    clone.diff_range = None;
    assert!(sub.diff_range.is_some());
    assert!(clone.diff_range.is_none());
}

// ===========================================================================
// 8. DiffRange fields and edge cases
// ===========================================================================

#[test]
fn diff_range_changed_files_correct() {
    let sub = multi_lang_substrate();
    let dr = sub.diff_range.as_ref().unwrap();
    assert_eq!(dr.changed_files.len(), 2);
    assert!(dr.changed_files.contains(&"src/lib.rs".to_string()));
    assert!(dr.changed_files.contains(&"app/index.ts".to_string()));
}

#[test]
fn diff_range_stats() {
    let sub = multi_lang_substrate();
    let dr = sub.diff_range.as_ref().unwrap();
    assert_eq!(dr.insertions, 30);
    assert_eq!(dr.deletions, 10);
    assert_eq!(dr.commit_count, 5);
}

#[test]
fn diff_range_empty_changed_files() {
    let sub = RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![make_file("src/lib.rs", "Rust", 50, false)],
        lang_summary: BTreeMap::new(),
        diff_range: Some(DiffRange {
            base: "v1.0".to_string(),
            head: "v1.1".to_string(),
            changed_files: vec![],
            commit_count: 0,
            insertions: 0,
            deletions: 0,
        }),
        total_tokens: 350,
        total_bytes: 1500,
        total_code_lines: 50,
    };
    assert_eq!(sub.diff_files().count(), 0);
}

#[test]
fn diff_range_serde_roundtrip() {
    let dr = DiffRange {
        base: "main".to_string(),
        head: "feature/test".to_string(),
        changed_files: vec!["a.rs".to_string(), "b.rs".to_string()],
        commit_count: 3,
        insertions: 100,
        deletions: 50,
    };
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "main");
    assert_eq!(back.head, "feature/test");
    assert_eq!(back.changed_files.len(), 2);
    assert_eq!(back.commit_count, 3);
}

// ===========================================================================
// 9. diff_files() and files_for_lang() methods
// ===========================================================================

#[test]
fn diff_files_returns_only_in_diff() {
    let sub = multi_lang_substrate();
    let diff: Vec<_> = sub.diff_files().collect();
    assert!(diff.iter().all(|f| f.in_diff));
}

#[test]
fn diff_files_count_matches_flag_count() {
    let sub = multi_lang_substrate();
    let diff_count = sub.diff_files().count();
    let flag_count = sub.files.iter().filter(|f| f.in_diff).count();
    assert_eq!(diff_count, flag_count);
}

#[test]
fn files_for_lang_rust_count() {
    let sub = multi_lang_substrate();
    assert_eq!(sub.files_for_lang("Rust").count(), 2);
}

#[test]
fn files_for_lang_typescript_count() {
    let sub = multi_lang_substrate();
    assert_eq!(sub.files_for_lang("TypeScript").count(), 2);
}

#[test]
fn files_for_lang_python_count() {
    let sub = multi_lang_substrate();
    assert_eq!(sub.files_for_lang("Python").count(), 1);
}

#[test]
fn files_for_lang_nonexistent() {
    let sub = multi_lang_substrate();
    assert_eq!(sub.files_for_lang("Haskell").count(), 0);
}

#[test]
fn files_for_lang_case_sensitive() {
    let sub = multi_lang_substrate();
    assert_eq!(sub.files_for_lang("rust").count(), 0);
    assert_eq!(sub.files_for_lang("RUST").count(), 0);
}

// ===========================================================================
// 10. Serde roundtrip and JSON shape
// ===========================================================================

#[test]
fn serde_roundtrip_multi_lang() {
    let sub = multi_lang_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), sub.files.len());
    assert_eq!(back.total_code_lines, sub.total_code_lines);
    assert_eq!(back.lang_summary.len(), sub.lang_summary.len());
}

#[test]
fn serde_roundtrip_preserves_diff_range() {
    let sub = multi_lang_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    let dr = back.diff_range.unwrap();
    assert_eq!(dr.base, "main");
    assert_eq!(dr.head, "feature");
}

#[test]
fn json_shape_has_all_top_level_fields() {
    let sub = multi_lang_substrate();
    let val: serde_json::Value = serde_json::to_value(&sub).unwrap();
    assert!(val.get("repo_root").is_some());
    assert!(val.get("files").is_some());
    assert!(val.get("lang_summary").is_some());
    assert!(val.get("diff_range").is_some());
    assert!(val.get("total_tokens").is_some());
    assert!(val.get("total_bytes").is_some());
    assert!(val.get("total_code_lines").is_some());
}

#[test]
fn json_files_is_array() {
    let sub = multi_lang_substrate();
    let val: serde_json::Value = serde_json::to_value(&sub).unwrap();
    assert!(val["files"].is_array());
    assert_eq!(val["files"].as_array().unwrap().len(), 5);
}

#[test]
fn json_lang_summary_is_object() {
    let sub = multi_lang_substrate();
    let val: serde_json::Value = serde_json::to_value(&sub).unwrap();
    assert!(val["lang_summary"].is_object());
}

// ===========================================================================
// 11. Determinism verification
// ===========================================================================

#[test]
fn determinism_serialization() {
    let sub = multi_lang_substrate();
    let j1 = serde_json::to_string(&sub).unwrap();
    let j2 = serde_json::to_string(&sub).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn determinism_after_clone() {
    let sub = multi_lang_substrate();
    let clone = sub.clone();
    let j1 = serde_json::to_string(&sub).unwrap();
    let j2 = serde_json::to_string(&clone).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn determinism_btreemap_key_order() {
    let sub = multi_lang_substrate();
    let keys1: Vec<String> = sub.lang_summary.keys().cloned().collect();
    let keys2: Vec<String> = sub.lang_summary.keys().cloned().collect();
    assert_eq!(keys1, keys2);
    // Verify alphabetical order
    for w in keys1.windows(2) {
        assert!(w[0] <= w[1], "keys not sorted: {} > {}", w[0], w[1]);
    }
}

// ===========================================================================
// 12. SubstrateFile field invariants
// ===========================================================================

#[test]
fn substrate_file_lines_gte_code() {
    let sub = multi_lang_substrate();
    for f in &sub.files {
        assert!(f.lines >= f.code, "lines < code for {}", f.path);
    }
}

#[test]
fn substrate_file_bytes_positive_when_code_positive() {
    let sub = multi_lang_substrate();
    for f in &sub.files {
        if f.code > 0 {
            assert!(f.bytes > 0, "bytes should be > 0 when code > 0: {}", f.path);
        }
    }
}

#[test]
fn substrate_file_tokens_positive_when_code_positive() {
    let sub = multi_lang_substrate();
    for f in &sub.files {
        if f.code > 0 {
            assert!(
                f.tokens > 0,
                "tokens should be > 0 when code > 0: {}",
                f.path
            );
        }
    }
}

// ===========================================================================
// 13. Property tests
// ===========================================================================

proptest! {
    #[test]
    fn prop_total_code_is_file_sum(
        codes in proptest::collection::vec(0usize..1000, 0..20)
    ) {
        let files: Vec<SubstrateFile> = codes.iter().enumerate().map(|(i, &code)| {
            make_file(&format!("file_{i}.rs"), "Rust", code, false)
        }).collect();
        let total: usize = files.iter().map(|f| f.code).sum();
        let sub = RepoSubstrate {
            repo_root: "/repo".to_string(),
            files,
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: total,
        };
        prop_assert_eq!(sub.total_code_lines, sub.files.iter().map(|f| f.code).sum::<usize>());
    }

    #[test]
    fn prop_serde_roundtrip(
        n_files in 0usize..10,
        code_base in 1usize..500
    ) {
        let files: Vec<SubstrateFile> = (0..n_files).map(|i| {
            make_file(&format!("f{i}.rs"), "Rust", code_base + i, false)
        }).collect();
        let total_code: usize = files.iter().map(|f| f.code).sum();
        let total_tokens: usize = files.iter().map(|f| f.tokens).sum();
        let total_bytes: usize = files.iter().map(|f| f.bytes).sum();
        let sub = RepoSubstrate {
            repo_root: "/repo".to_string(),
            files,
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens,
            total_bytes,
            total_code_lines: total_code,
        };
        let json = serde_json::to_string(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.files.len(), sub.files.len());
        prop_assert_eq!(back.total_code_lines, sub.total_code_lines);
    }

    #[test]
    fn prop_diff_files_subset_of_files(
        n in 0usize..10
    ) {
        let files: Vec<SubstrateFile> = (0..n).map(|i| {
            make_file(&format!("f{i}.rs"), "Rust", 10, i % 2 == 0)
        }).collect();
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
        prop_assert!(diff_count <= sub.files.len());
    }

    #[test]
    fn prop_files_for_lang_subset(
        n in 0usize..10
    ) {
        let files: Vec<SubstrateFile> = (0..n).map(|i| {
            let lang = if i % 2 == 0 { "Rust" } else { "Go" };
            make_file(&format!("f{i}.rs"), lang, 10, false)
        }).collect();
        let sub = RepoSubstrate {
            repo_root: "/r".to_string(),
            files,
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: 0,
            total_bytes: 0,
            total_code_lines: 0,
        };
        let rust_count = sub.files_for_lang("Rust").count();
        let go_count = sub.files_for_lang("Go").count();
        prop_assert_eq!(rust_count + go_count, sub.files.len());
    }

    #[test]
    fn prop_deterministic_serialization(
        code in 1usize..1000
    ) {
        let sub = RepoSubstrate {
            repo_root: "/r".to_string(),
            files: vec![make_file("a.rs", "Rust", code, false)],
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: code * 7,
            total_bytes: code * 30,
            total_code_lines: code,
        };
        let j1 = serde_json::to_string(&sub).unwrap();
        let j2 = serde_json::to_string(&sub).unwrap();
        prop_assert_eq!(j1, j2);
    }

    #[test]
    fn prop_clone_equality(
        code in 1usize..1000
    ) {
        let sub = RepoSubstrate {
            repo_root: "/r".to_string(),
            files: vec![make_file("a.rs", "Rust", code, false)],
            lang_summary: BTreeMap::new(),
            diff_range: None,
            total_tokens: code * 7,
            total_bytes: code * 30,
            total_code_lines: code,
        };
        let clone = sub.clone();
        let j1 = serde_json::to_string(&sub).unwrap();
        let j2 = serde_json::to_string(&clone).unwrap();
        prop_assert_eq!(j1, j2);
    }

    #[test]
    fn prop_lang_summary_file_count_nonneg(
        n_files in 0usize..20,
        n_code in 0usize..500
    ) {
        let ls = make_lang(n_files, n_code);
        prop_assert!(ls.files == n_files);
        prop_assert!(ls.code == n_code);
        prop_assert!(ls.lines >= n_code);
    }

    #[test]
    fn prop_empty_substrate_totals_zero(dummy in 0usize..1) {
        let _ = dummy;
        let sub = empty_substrate();
        prop_assert_eq!(sub.total_code_lines, 0);
        prop_assert_eq!(sub.total_tokens, 0);
        prop_assert_eq!(sub.total_bytes, 0);
        prop_assert!(sub.files.is_empty());
    }
}
