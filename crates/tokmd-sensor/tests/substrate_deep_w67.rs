//! Deep tests for tokmd-sensor::substrate: RepoSubstrate data types (W67)

use std::collections::BTreeMap;

use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_file(path: &str, lang: &str, code: usize, in_diff: bool) -> SubstrateFile {
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
            .unwrap_or(".")
            .to_string(),
        in_diff,
    }
}

fn make_substrate(files: Vec<SubstrateFile>, diff: Option<DiffRange>) -> RepoSubstrate {
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
        diff_range: diff,
        total_tokens,
        total_bytes,
        total_code_lines,
    }
}

fn sample_diff() -> DiffRange {
    DiffRange {
        base: "main".to_string(),
        head: "HEAD".to_string(),
        changed_files: vec!["src/lib.rs".to_string(), "src/utils.rs".to_string()],
        commit_count: 5,
        insertions: 40,
        deletions: 10,
    }
}

// ---------------------------------------------------------------------------
// Tests: construction and field access
// ---------------------------------------------------------------------------

#[test]
fn empty_substrate_has_zero_totals() {
    let sub = make_substrate(vec![], None);
    assert_eq!(sub.total_code_lines, 0);
    assert_eq!(sub.total_bytes, 0);
    assert_eq!(sub.total_tokens, 0);
    assert!(sub.files.is_empty());
    assert!(sub.lang_summary.is_empty());
}

#[test]
fn single_file_substrate_totals_match() {
    let sub = make_substrate(vec![make_file("a.rs", "Rust", 100, false)], None);
    assert_eq!(sub.total_code_lines, 100);
    assert_eq!(sub.total_bytes, 3000);
    assert_eq!(sub.total_tokens, 800);
    assert_eq!(sub.files.len(), 1);
}

#[test]
fn multi_file_substrate_aggregates_correctly() {
    let sub = make_substrate(
        vec![
            make_file("a.rs", "Rust", 100, false),
            make_file("b.rs", "Rust", 50, false),
            make_file("c.py", "Python", 30, false),
        ],
        None,
    );
    assert_eq!(sub.total_code_lines, 180);
    assert_eq!(sub.files.len(), 3);
    assert_eq!(sub.lang_summary.len(), 2);
    assert_eq!(sub.lang_summary["Rust"].files, 2);
    assert_eq!(sub.lang_summary["Rust"].code, 150);
    assert_eq!(sub.lang_summary["Python"].files, 1);
    assert_eq!(sub.lang_summary["Python"].code, 30);
}

#[test]
fn repo_root_is_preserved() {
    let sub = make_substrate(vec![], None);
    assert_eq!(sub.repo_root, "/repo");
}

// ---------------------------------------------------------------------------
// Tests: diff_files filter
// ---------------------------------------------------------------------------

#[test]
fn diff_files_returns_only_in_diff() {
    let sub = make_substrate(
        vec![
            make_file("a.rs", "Rust", 10, true),
            make_file("b.rs", "Rust", 20, false),
            make_file("c.rs", "Rust", 30, true),
        ],
        None,
    );
    let diff: Vec<_> = sub.diff_files().collect();
    assert_eq!(diff.len(), 2);
    assert_eq!(diff[0].path, "a.rs");
    assert_eq!(diff[1].path, "c.rs");
}

#[test]
fn diff_files_empty_when_none_in_diff() {
    let sub = make_substrate(vec![make_file("a.rs", "Rust", 10, false)], None);
    assert_eq!(sub.diff_files().count(), 0);
}

#[test]
fn diff_files_all_when_all_in_diff() {
    let sub = make_substrate(
        vec![
            make_file("a.rs", "Rust", 10, true),
            make_file("b.rs", "Rust", 20, true),
        ],
        None,
    );
    assert_eq!(sub.diff_files().count(), 2);
}

// ---------------------------------------------------------------------------
// Tests: files_for_lang filter
// ---------------------------------------------------------------------------

#[test]
fn files_for_lang_filters_correctly() {
    let sub = make_substrate(
        vec![
            make_file("a.rs", "Rust", 10, false),
            make_file("b.py", "Python", 20, false),
            make_file("c.rs", "Rust", 30, false),
        ],
        None,
    );
    let rust: Vec<_> = sub.files_for_lang("Rust").collect();
    assert_eq!(rust.len(), 2);
    let py: Vec<_> = sub.files_for_lang("Python").collect();
    assert_eq!(py.len(), 1);
}

#[test]
fn files_for_lang_returns_empty_for_missing_lang() {
    let sub = make_substrate(vec![make_file("a.rs", "Rust", 10, false)], None);
    assert_eq!(sub.files_for_lang("Go").count(), 0);
}

// ---------------------------------------------------------------------------
// Tests: DiffRange
// ---------------------------------------------------------------------------

#[test]
fn diff_range_fields_accessible() {
    let d = sample_diff();
    assert_eq!(d.base, "main");
    assert_eq!(d.head, "HEAD");
    assert_eq!(d.changed_files.len(), 2);
    assert_eq!(d.commit_count, 5);
    assert_eq!(d.insertions, 40);
    assert_eq!(d.deletions, 10);
}

#[test]
fn substrate_with_diff_range_preserves_it() {
    let sub = make_substrate(vec![], Some(sample_diff()));
    assert!(sub.diff_range.is_some());
    let dr = sub.diff_range.unwrap();
    assert_eq!(dr.base, "main");
}

#[test]
fn substrate_without_diff_range_is_none() {
    let sub = make_substrate(vec![], None);
    assert!(sub.diff_range.is_none());
}

// ---------------------------------------------------------------------------
// Tests: serde round-trips
// ---------------------------------------------------------------------------

#[test]
fn substrate_serde_roundtrip_without_diff() {
    let sub = make_substrate(vec![make_file("a.rs", "Rust", 100, false)], None);
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 1);
    assert_eq!(back.total_code_lines, 100);
    assert!(back.diff_range.is_none());
}

#[test]
fn substrate_serde_roundtrip_with_diff() {
    let sub = make_substrate(
        vec![make_file("a.rs", "Rust", 100, true)],
        Some(sample_diff()),
    );
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert!(back.diff_range.is_some());
    assert_eq!(back.diff_range.unwrap().commit_count, 5);
}

#[test]
fn diff_range_omitted_from_json_when_none() {
    let sub = make_substrate(vec![], None);
    let json = serde_json::to_string(&sub).unwrap();
    assert!(
        !json.contains("diff_range"),
        "None diff_range should be omitted"
    );
}

#[test]
fn substrate_file_serde_roundtrip() {
    let f = make_file("src/main.rs", "Rust", 50, true);
    let json = serde_json::to_string(&f).unwrap();
    let back: SubstrateFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "src/main.rs");
    assert_eq!(back.lang, "Rust");
    assert_eq!(back.code, 50);
    assert!(back.in_diff);
}

#[test]
fn lang_summary_serde_roundtrip() {
    let ls = LangSummary {
        files: 3,
        code: 200,
        lines: 300,
        bytes: 9000,
        tokens: 1600,
    };
    let json = serde_json::to_string(&ls).unwrap();
    let back: LangSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files, 3);
    assert_eq!(back.code, 200);
}

// ---------------------------------------------------------------------------
// Tests: BTreeMap ordering (determinism)
// ---------------------------------------------------------------------------

#[test]
fn lang_summary_keys_sorted_alphabetically() {
    let sub = make_substrate(
        vec![
            make_file("z.go", "Go", 10, false),
            make_file("a.rs", "Rust", 20, false),
            make_file("b.py", "Python", 30, false),
        ],
        None,
    );
    let keys: Vec<&String> = sub.lang_summary.keys().collect();
    assert_eq!(keys, vec!["Go", "Python", "Rust"]);
}

#[test]
fn substrate_json_deterministic_across_builds() {
    let sub1 = make_substrate(
        vec![
            make_file("a.rs", "Rust", 10, false),
            make_file("b.py", "Python", 20, false),
        ],
        Some(sample_diff()),
    );
    let sub2 = make_substrate(
        vec![
            make_file("a.rs", "Rust", 10, false),
            make_file("b.py", "Python", 20, false),
        ],
        Some(sample_diff()),
    );
    let j1 = serde_json::to_string(&sub1).unwrap();
    let j2 = serde_json::to_string(&sub2).unwrap();
    assert_eq!(j1, j2, "identical inputs must produce identical JSON");
}

// ---------------------------------------------------------------------------
// Tests: substrate shareability (Clone)
// ---------------------------------------------------------------------------

#[test]
fn substrate_is_cloneable_for_multi_sensor_sharing() {
    let sub = make_substrate(vec![make_file("a.rs", "Rust", 10, false)], None);
    let clone = sub.clone();
    assert_eq!(clone.total_code_lines, sub.total_code_lines);
    assert_eq!(clone.files.len(), sub.files.len());
}

#[test]
fn cloned_substrate_json_matches_original() {
    let sub = make_substrate(
        vec![
            make_file("a.rs", "Rust", 10, true),
            make_file("b.py", "Python", 20, false),
        ],
        Some(sample_diff()),
    );
    let clone = sub.clone();
    let j1 = serde_json::to_string(&sub).unwrap();
    let j2 = serde_json::to_string(&clone).unwrap();
    assert_eq!(j1, j2);
}
