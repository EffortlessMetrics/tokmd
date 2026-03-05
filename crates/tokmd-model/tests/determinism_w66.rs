//! Determinism hardening tests for tokmd-model.
//!
//! Verifies that model building is deterministic: same input -> same output
//! regardless of insertion order.

use proptest::prelude::*;
use tokmd_types::*;

// -- Helpers --

fn make_lang_row(lang: &str, code: usize) -> LangRow {
    LangRow {
        lang: lang.to_string(),
        code,
        lines: code + 50,
        files: 3,
        bytes: code * 4,
        tokens: code,
        avg_lines: if code + 50 > 0 { (code + 50) / 3 } else { 0 },
    }
}

fn make_module_row(module: &str, code: usize) -> ModuleRow {
    ModuleRow {
        module: module.to_string(),
        code,
        lines: code + 30,
        files: 2,
        bytes: code * 4,
        tokens: code,
        avg_lines: if code + 30 > 0 { (code + 30) / 2 } else { 0 },
    }
}

fn make_file_row(path: &str, lang: &str, module: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 10,
        blanks: 5,
        lines: code + 15,
        bytes: code * 4,
        tokens: code,
    }
}

fn totals_from_rows(rows: &[LangRow]) -> Totals {
    let code: usize = rows.iter().map(|r| r.code).sum();
    let lines: usize = rows.iter().map(|r| r.lines).sum();
    let files: usize = rows.iter().map(|r| r.files).sum();
    let bytes: usize = rows.iter().map(|r| r.bytes).sum();
    let tokens: usize = rows.iter().map(|r| r.tokens).sum();
    Totals {
        code,
        lines,
        files,
        bytes,
        tokens,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

fn sort_lang_rows(rows: &mut [LangRow]) {
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
}

fn sort_module_rows(rows: &mut [ModuleRow]) {
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
}

fn sort_file_rows(rows: &mut [FileRow]) {
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));
}

// -- 1. Lang rows: different insertion order -> same sorted output --

#[test]
fn lang_rows_insertion_order_does_not_affect_sorted_output() {
    let mut order_a = vec![
        make_lang_row("Rust", 500),
        make_lang_row("Python", 300),
        make_lang_row("Go", 100),
    ];
    let mut order_b = vec![
        make_lang_row("Go", 100),
        make_lang_row("Rust", 500),
        make_lang_row("Python", 300),
    ];
    let mut order_c = vec![
        make_lang_row("Python", 300),
        make_lang_row("Go", 100),
        make_lang_row("Rust", 500),
    ];
    sort_lang_rows(&mut order_a);
    sort_lang_rows(&mut order_b);
    sort_lang_rows(&mut order_c);
    let json_a = serde_json::to_string(&order_a).unwrap();
    let json_b = serde_json::to_string(&order_b).unwrap();
    let json_c = serde_json::to_string(&order_c).unwrap();
    assert_eq!(json_a, json_b);
    assert_eq!(json_b, json_c);
}

// -- 2. Module rows: different insertion order -> same sorted output --

#[test]
fn module_rows_insertion_order_does_not_affect_sorted_output() {
    let mut order_a = vec![
        make_module_row("crates/tokmd", 800),
        make_module_row("src", 200),
        make_module_row("tests", 50),
    ];
    let mut order_b = vec![
        make_module_row("tests", 50),
        make_module_row("crates/tokmd", 800),
        make_module_row("src", 200),
    ];
    sort_module_rows(&mut order_a);
    sort_module_rows(&mut order_b);
    let json_a = serde_json::to_string(&order_a).unwrap();
    let json_b = serde_json::to_string(&order_b).unwrap();
    assert_eq!(json_a, json_b);
}

// -- 3. File rows: different insertion order -> same sorted output --

#[test]
fn file_rows_insertion_order_does_not_affect_sorted_output() {
    let mut order_a = vec![
        make_file_row("src/main.rs", "Rust", "src", 120),
        make_file_row("src/lib.rs", "Rust", "src", 80),
        make_file_row("tests/test.py", "Python", "tests", 40),
    ];
    let mut order_b = vec![
        make_file_row("tests/test.py", "Python", "tests", 40),
        make_file_row("src/lib.rs", "Rust", "src", 80),
        make_file_row("src/main.rs", "Rust", "src", 120),
    ];
    sort_file_rows(&mut order_a);
    sort_file_rows(&mut order_b);
    let json_a = serde_json::to_string(&order_a).unwrap();
    let json_b = serde_json::to_string(&order_b).unwrap();
    assert_eq!(json_a, json_b);
}

// -- 4. Module key: path order independence --

#[test]
fn module_key_is_path_order_independent() {
    use tokmd_module_key::module_key;
    let roots = vec!["crates".to_string()];
    let paths = [
        "crates/foo/src/lib.rs",
        "crates/bar/src/main.rs",
        "src/lib.rs",
    ];
    let keys: Vec<String> = paths.iter().map(|p| module_key(p, &roots, 2)).collect();
    let keys_rev: Vec<String> = paths
        .iter()
        .rev()
        .map(|p| module_key(p, &roots, 2))
        .collect();
    assert_eq!(keys[0], keys_rev[2]);
    assert_eq!(keys[1], keys_rev[1]);
    assert_eq!(keys[2], keys_rev[0]);
}

// -- 5. Module key with different separators --

#[test]
fn module_key_normalizes_separators() {
    use tokmd_module_key::module_key;
    let roots = vec!["crates".to_string()];
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots, 2),
        module_key("crates\\foo\\src\\lib.rs", &roots, 2),
    );
}

// -- 6. BTreeMap aggregation order --

#[test]
fn btreemap_aggregation_is_deterministic() {
    use std::collections::BTreeMap;
    let entries = vec![
        ("crates/tokmd", 100usize),
        ("src", 50),
        ("tests", 30),
        ("crates/types", 80),
    ];
    let mut map_fwd: BTreeMap<&str, usize> = BTreeMap::new();
    for &(k, v) in &entries {
        *map_fwd.entry(k).or_default() += v;
    }
    let mut map_rev: BTreeMap<&str, usize> = BTreeMap::new();
    for &(k, v) in entries.iter().rev() {
        *map_rev.entry(k).or_default() += v;
    }
    let keys_fwd: Vec<&&str> = map_fwd.keys().collect();
    let keys_rev: Vec<&&str> = map_rev.keys().collect();
    assert_eq!(keys_fwd, keys_rev);
    assert_eq!(map_fwd, map_rev);
}

// -- 7. Duplicate BTreeMap entries sum correctly --

#[test]
fn btreemap_duplicate_entries_aggregate_deterministically() {
    use std::collections::BTreeMap;
    let mut map1: BTreeMap<String, usize> = BTreeMap::new();
    let mut map2: BTreeMap<String, usize> = BTreeMap::new();
    let entries = vec![("src", 10), ("src", 20), ("lib", 30), ("src", 5)];
    for (k, v) in &entries {
        *map1.entry(k.to_string()).or_default() += v;
    }
    for (k, v) in entries.iter().rev() {
        *map2.entry(k.to_string()).or_default() += v;
    }
    assert_eq!(map1, map2);
    assert_eq!(*map1.get("src").unwrap(), 35);
}

// -- 8. Children mode Collapse serialization is stable --

#[test]
fn children_mode_collapse_report_is_stable() {
    let rows = vec![make_lang_row("Rust", 500), make_lang_row("Python", 300)];
    let total = totals_from_rows(&rows);
    let report = LangReport {
        rows,
        total,
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let json1 = serde_json::to_string(&report).unwrap();
    let json2 = serde_json::to_string(&report).unwrap();
    assert_eq!(json1, json2);
}

// -- 9. Children mode Separate serialization is stable --

#[test]
fn children_mode_separate_report_is_stable() {
    let rows = vec![
        make_lang_row("Rust", 500),
        make_lang_row("JavaScript (embedded)", 100),
    ];
    let total = totals_from_rows(&rows);
    let report = LangReport {
        rows,
        total,
        with_files: true,
        children: ChildrenMode::Separate,
        top: 0,
    };
    let json1 = serde_json::to_string(&report).unwrap();
    let json2 = serde_json::to_string(&report).unwrap();
    assert_eq!(json1, json2);
}

// -- 10. avg function is deterministic --

#[test]
fn avg_function_determinism() {
    use tokmd_model::avg;
    for lines in [0, 1, 7, 100, 999, 10000] {
        for files in [1, 2, 3, 7, 100] {
            let r1 = avg(lines, files);
            let r2 = avg(lines, files);
            assert_eq!(r1, r2, "avg({lines}, {files}) not deterministic");
        }
    }
    assert_eq!(tokmd_model::avg(100, 0), 0);
}

// -- 11. normalize_path is deterministic --

#[test]
fn normalize_path_determinism() {
    use std::path::Path;
    use tokmd_model::normalize_path;
    let paths = [
        "src/main.rs",
        "crates\\tokmd\\src\\lib.rs",
        "./tests/test.rs",
    ];
    for p in &paths {
        let n1 = normalize_path(Path::new(p), None);
        let n2 = normalize_path(Path::new(p), None);
        assert_eq!(n1, n2, "normalize_path({p:?}) not deterministic");
    }
}

// -- 12. normalize_path cross-platform separator consistency --

#[test]
fn normalize_path_forward_slash_consistency() {
    use std::path::Path;
    use tokmd_model::normalize_path;
    let n1 = normalize_path(Path::new("src/lib.rs"), None);
    assert!(!n1.contains('\\'), "output must use forward slashes only");
}

// -- 13. ExportData sorted by code desc then path asc --

#[test]
fn export_data_sort_code_desc_then_path_asc() {
    let mut rows = vec![
        make_file_row("src/b.rs", "Rust", "src", 100),
        make_file_row("src/a.rs", "Rust", "src", 100),
        make_file_row("src/c.rs", "Rust", "src", 200),
    ];
    sort_file_rows(&mut rows);
    assert_eq!(rows[0].path, "src/c.rs");
    assert_eq!(rows[1].path, "src/a.rs");
    assert_eq!(rows[2].path, "src/b.rs");
}

// -- 14. Top-N folding produces Other row deterministically --

#[test]
fn top_n_folding_is_deterministic() {
    let mut rows = vec![
        make_lang_row("Rust", 500),
        make_lang_row("Python", 300),
        make_lang_row("Go", 100),
        make_lang_row("Java", 50),
    ];
    sort_lang_rows(&mut rows);
    let top = 2;
    let other_code: usize = rows[top..].iter().map(|r| r.code).sum();
    let other_lines: usize = rows[top..].iter().map(|r| r.lines).sum();
    let other_files: usize = rows[top..].iter().map(|r| r.files).sum();
    let other = LangRow {
        lang: "Other".to_string(),
        code: other_code,
        lines: other_lines,
        files: other_files,
        bytes: rows[top..].iter().map(|r| r.bytes).sum(),
        tokens: rows[top..].iter().map(|r| r.tokens).sum(),
        avg_lines: tokmd_model::avg(other_lines, other_files),
    };
    rows.truncate(top);
    rows.push(other);

    let mut rows2 = vec![
        make_lang_row("Java", 50),
        make_lang_row("Go", 100),
        make_lang_row("Python", 300),
        make_lang_row("Rust", 500),
    ];
    sort_lang_rows(&mut rows2);
    let other_code2: usize = rows2[top..].iter().map(|r| r.code).sum();
    let other_lines2: usize = rows2[top..].iter().map(|r| r.lines).sum();
    let other_files2: usize = rows2[top..].iter().map(|r| r.files).sum();
    let other2 = LangRow {
        lang: "Other".to_string(),
        code: other_code2,
        lines: other_lines2,
        files: other_files2,
        bytes: rows2[top..].iter().map(|r| r.bytes).sum(),
        tokens: rows2[top..].iter().map(|r| r.tokens).sum(),
        avg_lines: tokmd_model::avg(other_lines2, other_files2),
    };
    rows2.truncate(top);
    rows2.push(other2);

    let json1 = serde_json::to_string(&rows).unwrap();
    let json2 = serde_json::to_string(&rows2).unwrap();
    assert_eq!(json1, json2);
}

// -- 15. Module report with ChildIncludeMode variants --

#[test]
fn module_report_child_include_mode_serialization_stable() {
    for mode in [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly] {
        let report = ModuleReport {
            rows: vec![make_module_row("src", 100)],
            total: Totals {
                code: 100,
                lines: 130,
                files: 2,
                bytes: 400,
                tokens: 100,
                avg_lines: 65,
            },
            module_roots: vec![],
            module_depth: 1,
            children: mode,
            top: 0,
        };
        let json1 = serde_json::to_string(&report).unwrap();
        let json2 = serde_json::to_string(&report).unwrap();
        assert_eq!(json1, json2, "ChildIncludeMode::{mode:?} not stable");
    }
}

// -- 16. Tie-breaking by name when codes are equal --

#[test]
fn tie_breaking_by_name_is_deterministic() {
    let mut rows = vec![
        make_lang_row("Zeta", 100),
        make_lang_row("Alpha", 100),
        make_lang_row("Mid", 100),
    ];
    sort_lang_rows(&mut rows);
    assert_eq!(rows[0].lang, "Alpha");
    assert_eq!(rows[1].lang, "Mid");
    assert_eq!(rows[2].lang, "Zeta");
}

// -- Property tests --

proptest! {
    #[test]
    fn prop_lang_sort_any_permutation(
        a in 0usize..10_000,
        b in 0usize..10_000,
        c in 0usize..10_000,
        d in 0usize..10_000,
    ) {
        let raw = vec![
            make_lang_row("Alpha", a),
            make_lang_row("Beta", b),
            make_lang_row("Gamma", c),
            make_lang_row("Delta", d),
        ];
        let mut fwd = raw.clone();
        let mut rev = raw.into_iter().rev().collect::<Vec<_>>();
        sort_lang_rows(&mut fwd);
        sort_lang_rows(&mut rev);
        let json_fwd = serde_json::to_string(&fwd).unwrap();
        let json_rev = serde_json::to_string(&rev).unwrap();
        prop_assert_eq!(json_fwd, json_rev);
    }

    #[test]
    fn prop_module_sort_any_permutation(
        a in 0usize..10_000,
        b in 0usize..10_000,
        c in 0usize..10_000,
    ) {
        let raw = vec![
            make_module_row("mod_a", a),
            make_module_row("mod_b", b),
            make_module_row("mod_c", c),
        ];
        let mut fwd = raw.clone();
        let mut rev = raw.into_iter().rev().collect::<Vec<_>>();
        sort_module_rows(&mut fwd);
        sort_module_rows(&mut rev);
        let json_fwd = serde_json::to_string(&fwd).unwrap();
        let json_rev = serde_json::to_string(&rev).unwrap();
        prop_assert_eq!(json_fwd, json_rev);
    }

    #[test]
    fn prop_file_sort_any_permutation(
        a in 0usize..10_000,
        b in 0usize..10_000,
    ) {
        let raw = vec![
            make_file_row("x.rs", "Rust", "src", a),
            make_file_row("y.rs", "Rust", "src", b),
        ];
        let mut fwd = raw.clone();
        let mut rev = raw.into_iter().rev().collect::<Vec<_>>();
        sort_file_rows(&mut fwd);
        sort_file_rows(&mut rev);
        let json_fwd = serde_json::to_string(&fwd).unwrap();
        let json_rev = serde_json::to_string(&rev).unwrap();
        prop_assert_eq!(json_fwd, json_rev);
    }

    #[test]
    fn prop_btreemap_ordering_is_lexicographic(
        keys in prop::collection::vec("[a-z]{1,8}", 2..10),
    ) {
        use std::collections::BTreeMap;
        let mut map: BTreeMap<String, usize> = BTreeMap::new();
        for (i, k) in keys.iter().enumerate() {
            *map.entry(k.clone()).or_default() += i;
        }
        let collected: Vec<&String> = map.keys().collect();
        let mut sorted = collected.clone();
        sorted.sort();
        prop_assert_eq!(collected, sorted, "BTreeMap keys must be lexicographic");
    }
}
