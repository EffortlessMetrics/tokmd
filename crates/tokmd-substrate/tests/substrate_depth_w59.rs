//! Depth tests for `RepoSubstrate` construction, field access, serde,
//! sharing, and edge cases.

use std::collections::BTreeMap;

use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ── Helpers ──────────────────────────────────────────────────────

fn make_file(path: &str, lang: &str, code: usize) -> SubstrateFile {
    SubstrateFile {
        path: path.to_string(),
        lang: lang.to_string(),
        code,
        lines: code + 20,
        bytes: code * 30,
        tokens: code * 8,
        module: path
            .rsplit_once('/')
            .map(|(m, _)| m)
            .unwrap_or("")
            .to_string(),
        in_diff: false,
    }
}

fn make_file_in_diff(path: &str, lang: &str, code: usize) -> SubstrateFile {
    let mut f = make_file(path, lang, code);
    f.in_diff = true;
    f
}

fn substrate_from_files(files: Vec<SubstrateFile>) -> RepoSubstrate {
    let mut lang_summary: BTreeMap<String, LangSummary> = BTreeMap::new();
    for f in &files {
        let e = lang_summary.entry(f.lang.clone()).or_insert(LangSummary {
            files: 0,
            code: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
        });
        e.files += 1;
        e.code += f.code;
        e.lines += f.lines;
        e.bytes += f.bytes;
        e.tokens += f.tokens;
    }
    let total_tokens = files.iter().map(|f| f.tokens).sum();
    let total_bytes = files.iter().map(|f| f.bytes).sum();
    let total_code_lines = files.iter().map(|f| f.code).sum();
    RepoSubstrate {
        repo_root: "/repo".to_string(),
        files,
        lang_summary,
        diff_range: None,
        total_tokens,
        total_bytes,
        total_code_lines,
    }
}

fn empty_substrate() -> RepoSubstrate {
    substrate_from_files(vec![])
}

fn multi_lang_substrate() -> RepoSubstrate {
    substrate_from_files(vec![
        make_file("src/lib.rs", "Rust", 200),
        make_file("src/main.py", "Python", 150),
        make_file("tests/test.py", "Python", 80),
        make_file_in_diff("src/app.ts", "TypeScript", 300),
        make_file_in_diff("src/new.rs", "Rust", 50),
    ])
}

fn sample_diff_range() -> DiffRange {
    DiffRange {
        base: "main".to_string(),
        head: "feature/w59".to_string(),
        changed_files: vec!["src/app.ts".to_string(), "src/new.rs".to_string()],
        commit_count: 5,
        insertions: 42,
        deletions: 10,
    }
}

// ── Construction tests ───────────────────────────────────────────

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
fn single_file_substrate_totals() {
    let f = make_file("src/lib.rs", "Rust", 100);
    let sub = substrate_from_files(vec![f]);
    assert_eq!(sub.total_code_lines, 100);
    assert_eq!(sub.total_tokens, 800);
    assert_eq!(sub.total_bytes, 3000);
    assert_eq!(sub.files.len(), 1);
    assert_eq!(sub.lang_summary.len(), 1);
}

#[test]
fn multi_file_totals_are_sum_of_parts() {
    let sub = multi_lang_substrate();
    let expected_code = 200 + 150 + 80 + 300 + 50;
    assert_eq!(sub.total_code_lines, expected_code);
    let expected_tokens: usize = sub.files.iter().map(|f| f.tokens).sum();
    assert_eq!(sub.total_tokens, expected_tokens);
}

#[test]
fn lang_summary_aggregates_correctly() {
    let sub = multi_lang_substrate();
    assert_eq!(sub.lang_summary.len(), 3);

    let rust = sub.lang_summary.get("Rust").unwrap();
    assert_eq!(rust.files, 2);
    assert_eq!(rust.code, 250); // 200 + 50

    let python = sub.lang_summary.get("Python").unwrap();
    assert_eq!(python.files, 2);
    assert_eq!(python.code, 230); // 150 + 80

    let ts = sub.lang_summary.get("TypeScript").unwrap();
    assert_eq!(ts.files, 1);
    assert_eq!(ts.code, 300);
}

#[test]
fn lang_summary_keys_are_sorted() {
    let sub = multi_lang_substrate();
    let keys: Vec<&String> = sub.lang_summary.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "BTreeMap guarantees sorted keys");
}

// ── Field access tests ───────────────────────────────────────────

#[test]
fn repo_root_is_preserved() {
    let sub = empty_substrate();
    assert_eq!(sub.repo_root, "/repo");
}

#[test]
fn diff_range_none_by_default() {
    let sub = empty_substrate();
    assert!(sub.diff_range.is_none());
}

#[test]
fn diff_range_when_set() {
    let mut sub = multi_lang_substrate();
    sub.diff_range = Some(sample_diff_range());
    let dr = sub.diff_range.as_ref().unwrap();
    assert_eq!(dr.base, "main");
    assert_eq!(dr.head, "feature/w59");
    assert_eq!(dr.changed_files.len(), 2);
    assert_eq!(dr.commit_count, 5);
    assert_eq!(dr.insertions, 42);
    assert_eq!(dr.deletions, 10);
}

// ── diff_files() method tests ────────────────────────────────────

#[test]
fn diff_files_returns_only_in_diff() {
    let sub = multi_lang_substrate();
    let diff: Vec<&SubstrateFile> = sub.diff_files().collect();
    assert_eq!(diff.len(), 2);
    for f in &diff {
        assert!(f.in_diff);
    }
}

#[test]
fn diff_files_empty_when_none_in_diff() {
    let sub = substrate_from_files(vec![
        make_file("a.rs", "Rust", 10),
        make_file("b.rs", "Rust", 20),
    ]);
    let diff: Vec<&SubstrateFile> = sub.diff_files().collect();
    assert!(diff.is_empty());
}

#[test]
fn diff_files_all_when_all_in_diff() {
    let sub = substrate_from_files(vec![
        make_file_in_diff("a.rs", "Rust", 10),
        make_file_in_diff("b.rs", "Rust", 20),
    ]);
    let diff: Vec<&SubstrateFile> = sub.diff_files().collect();
    assert_eq!(diff.len(), 2);
}

// ── files_for_lang() method tests ────────────────────────────────

#[test]
fn files_for_lang_filters_correctly() {
    let sub = multi_lang_substrate();
    let rust_files: Vec<&SubstrateFile> = sub.files_for_lang("Rust").collect();
    assert_eq!(rust_files.len(), 2);
    for f in &rust_files {
        assert_eq!(f.lang, "Rust");
    }
}

#[test]
fn files_for_lang_returns_empty_for_missing() {
    let sub = multi_lang_substrate();
    let go_files: Vec<&SubstrateFile> = sub.files_for_lang("Go").collect();
    assert!(go_files.is_empty());
}

#[test]
fn files_for_lang_case_sensitive() {
    let sub = multi_lang_substrate();
    let lower: Vec<&SubstrateFile> = sub.files_for_lang("rust").collect();
    assert!(lower.is_empty(), "language match should be case-sensitive");
}

#[test]
fn files_for_lang_on_empty_substrate() {
    let sub = empty_substrate();
    let files: Vec<&SubstrateFile> = sub.files_for_lang("Rust").collect();
    assert!(files.is_empty());
}

// ── SubstrateFile field tests ────────────────────────────────────

#[test]
fn substrate_file_module_from_path() {
    let f = make_file("src/utils/helpers.rs", "Rust", 50);
    assert_eq!(f.module, "src/utils");
}

#[test]
fn substrate_file_module_root_file() {
    let f = make_file("lib.rs", "Rust", 50);
    assert_eq!(f.module, "");
}

// ── Serde roundtrip tests ────────────────────────────────────────

#[test]
fn substrate_full_roundtrip() {
    let mut sub = multi_lang_substrate();
    sub.diff_range = Some(sample_diff_range());
    let json = serde_json::to_string_pretty(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.repo_root, sub.repo_root);
    assert_eq!(back.files.len(), sub.files.len());
    assert_eq!(back.lang_summary.len(), sub.lang_summary.len());
    assert_eq!(back.total_code_lines, sub.total_code_lines);
    assert_eq!(back.total_tokens, sub.total_tokens);
    assert_eq!(back.total_bytes, sub.total_bytes);
    assert!(back.diff_range.is_some());
}

#[test]
fn substrate_empty_roundtrip() {
    let sub = empty_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.files.is_empty());
    assert!(back.lang_summary.is_empty());
    assert_eq!(back.total_code_lines, 0);
}

#[test]
fn substrate_without_diff_omits_field() {
    let sub = empty_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    assert!(
        !json.contains("diff_range"),
        "None diff_range should be omitted via skip_serializing_if"
    );
}

#[test]
fn substrate_with_diff_includes_field() {
    let mut sub = empty_substrate();
    sub.diff_range = Some(sample_diff_range());
    let json = serde_json::to_string(&sub).unwrap();
    assert!(json.contains("diff_range"));
    assert!(json.contains("feature/w59"));
}

#[test]
fn diff_range_roundtrip() {
    let dr = sample_diff_range();
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "main");
    assert_eq!(back.head, "feature/w59");
    assert_eq!(back.changed_files.len(), 2);
    assert_eq!(back.commit_count, 5);
    assert_eq!(back.insertions, 42);
    assert_eq!(back.deletions, 10);
}

#[test]
fn lang_summary_roundtrip() {
    let ls = LangSummary {
        files: 10,
        code: 500,
        lines: 600,
        bytes: 15000,
        tokens: 3750,
    };
    let json = serde_json::to_string(&ls).unwrap();
    let back: LangSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files, 10);
    assert_eq!(back.code, 500);
    assert_eq!(back.lines, 600);
    assert_eq!(back.bytes, 15000);
    assert_eq!(back.tokens, 3750);
}

#[test]
fn substrate_file_roundtrip() {
    let f = make_file_in_diff("src/lib.rs", "Rust", 100);
    let json = serde_json::to_string(&f).unwrap();
    let back: SubstrateFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "src/lib.rs");
    assert_eq!(back.lang, "Rust");
    assert_eq!(back.code, 100);
    assert!(back.in_diff);
}

// ── Clone tests ──────────────────────────────────────────────────

#[test]
fn substrate_clone_is_independent() {
    let sub = multi_lang_substrate();
    let mut cloned = sub.clone();
    cloned.repo_root = "/other".to_string();
    assert_eq!(sub.repo_root, "/repo");
    assert_eq!(cloned.repo_root, "/other");
}

#[test]
fn substrate_file_clone_is_independent() {
    let f = make_file("a.rs", "Rust", 50);
    let mut cloned = f.clone();
    cloned.code = 999;
    assert_eq!(f.code, 50);
    assert_eq!(cloned.code, 999);
}

// ── Sharing (immutable borrow) tests ─────────────────────────────

#[test]
fn substrate_shared_across_consumers() {
    let sub = multi_lang_substrate();

    // Simulate multiple consumers reading from the same substrate
    let consumer1_langs: Vec<&str> = sub.lang_summary.keys().map(|k| k.as_str()).collect();
    let consumer2_code = sub.total_code_lines;
    let consumer3_diff: Vec<&str> = sub.diff_files().map(|f| f.path.as_str()).collect();

    assert_eq!(consumer1_langs.len(), 3);
    assert_eq!(consumer2_code, 780);
    assert_eq!(consumer3_diff.len(), 2);
}

#[test]
fn substrate_ref_multiple_iterators() {
    let sub = multi_lang_substrate();

    // Multiple iterators on same substrate
    let rust_count = sub.files_for_lang("Rust").count();
    let python_count = sub.files_for_lang("Python").count();
    let diff_count = sub.diff_files().count();

    assert_eq!(rust_count, 2);
    assert_eq!(python_count, 2);
    assert_eq!(diff_count, 2);
}

// ── Edge cases ───────────────────────────────────────────────────

#[test]
fn substrate_single_zero_code_file() {
    let f = SubstrateFile {
        path: "empty.txt".to_string(),
        lang: "Text".to_string(),
        code: 0,
        lines: 0,
        bytes: 0,
        tokens: 0,
        module: "".to_string(),
        in_diff: false,
    };
    let sub = substrate_from_files(vec![f]);
    assert_eq!(sub.total_code_lines, 0);
    assert_eq!(sub.files.len(), 1);
    assert_eq!(sub.lang_summary.get("Text").unwrap().code, 0);
}

#[test]
fn substrate_many_languages() {
    let langs = [
        "Rust",
        "Python",
        "Go",
        "Java",
        "C",
        "C++",
        "JavaScript",
        "TypeScript",
        "Ruby",
        "Shell",
    ];
    let files: Vec<SubstrateFile> = langs
        .iter()
        .enumerate()
        .map(|(i, lang)| make_file(&format!("src/file{i}.x"), lang, (i + 1) * 10))
        .collect();
    let sub = substrate_from_files(files);
    assert_eq!(sub.lang_summary.len(), 10);
    assert_eq!(sub.files.len(), 10);
}

#[test]
fn diff_range_empty_changed_files() {
    let dr = DiffRange {
        base: "v1.0".to_string(),
        head: "v2.0".to_string(),
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

#[test]
fn substrate_large_values() {
    let f = SubstrateFile {
        path: "huge.rs".to_string(),
        lang: "Rust".to_string(),
        code: 1_000_000,
        lines: 1_200_000,
        bytes: 50_000_000,
        tokens: 12_000_000,
        module: "".to_string(),
        in_diff: false,
    };
    let sub = substrate_from_files(vec![f]);
    assert_eq!(sub.total_code_lines, 1_000_000);
    assert_eq!(sub.total_bytes, 50_000_000);
}

#[test]
fn substrate_debug_impl() {
    let sub = empty_substrate();
    let dbg = format!("{:?}", sub);
    assert!(dbg.contains("RepoSubstrate"));
    assert!(dbg.contains("repo_root"));
}

#[test]
fn substrate_file_debug_impl() {
    let f = make_file("a.rs", "Rust", 10);
    let dbg = format!("{:?}", f);
    assert!(dbg.contains("SubstrateFile"));
    assert!(dbg.contains("a.rs"));
}
