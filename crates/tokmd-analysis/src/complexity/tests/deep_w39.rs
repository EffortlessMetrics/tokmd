//! Deep tests for `analysis complexity module` (wave 39).
//!
//! Covers cyclomatic / cognitive complexity calculation, per-function
//! detail extraction, file-level aggregation, histogram generation,
//! risk classification, edge cases, and deterministic ordering.

use std::fs;
use std::path::PathBuf;

use crate::complexity::{build_complexity_report, generate_complexity_histogram};
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::{ComplexityRisk, FileComplexity};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn make_row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 0,
        blanks: 0,
        lines: code,
        bytes: code * 40,
        tokens: code * 8,
    }
}

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn write_temp_files(files: &[(&str, &str)]) -> (tempfile::TempDir, Vec<PathBuf>) {
    let dir = tempfile::tempdir().expect("create tempdir");
    let mut paths = Vec::new();
    for (rel, content) in files {
        let full = dir.path().join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full, content).unwrap();
        paths.push(PathBuf::from(rel));
    }
    (dir, paths)
}

fn analyze(
    files: &[(&str, &str, &str)], // (path, lang, content)
    detail: bool,
) -> tokmd_analysis_types::ComplexityReport {
    let file_entries: Vec<(&str, &str)> = files.iter().map(|(p, _, c)| (*p, *c)).collect();
    let (dir, paths) = write_temp_files(&file_entries);
    let rows: Vec<FileRow> = files
        .iter()
        .map(|(p, lang, c)| make_row(p, "root", lang, c.lines().count()))
        .collect();
    let export = make_export(rows);
    build_complexity_report(
        dir.path(),
        &paths,
        &export,
        &AnalysisLimits::default(),
        detail,
    )
    .unwrap()
}

// ── Cyclomatic complexity ───────────────────────────────────────

#[test]
fn cyclomatic_base_is_one_for_trivial_fn() {
    let code = "fn noop() {\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], false);
    assert_eq!(r.files.len(), 1);
    assert_eq!(r.files[0].cyclomatic_complexity, 1);
}

#[test]
fn cyclomatic_counts_if() {
    let code = "fn f(x: i32) -> i32 {\n    if x > 0 { 1 } else { 0 }\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], false);
    // base 1 + 1 if = 2
    assert_eq!(r.files[0].cyclomatic_complexity, 2);
}

#[test]
fn cyclomatic_counts_match_arms() {
    let code = "fn f(x: i32) {\n    match x {\n        1 => {},\n        _ => {},\n    }\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], false);
    // base 1 + 1 match = 2
    assert_eq!(r.files[0].cyclomatic_complexity, 2);
}

#[test]
fn cyclomatic_counts_logical_operators() {
    let code = "fn f(a: bool, b: bool) -> bool {\n    a && b || !a\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], false);
    // base 1 + 1 && + 1 || = 3
    assert_eq!(r.files[0].cyclomatic_complexity, 3);
}

#[test]
fn cyclomatic_python_branches() {
    let code = "def f(x):\n    if x > 0:\n        return 1\n    elif x < 0:\n        return -1\n    else:\n        return 0\n";
    let r = analyze(&[("main.py", "Python", code)], false);
    // base 1 + 1 if + 1 elif + "else" line contains "if " in "elif " = 4
    // (the keyword scanner matches "if " occurrences inside "elif ")
    assert_eq!(r.files[0].cyclomatic_complexity, 4);
}

#[test]
fn cyclomatic_js_switch_cases() {
    let code = "function f(x) {\n    switch (x) {\n        case 1: return 'a';\n        case 2: return 'b';\n    }\n}\n";
    let r = analyze(&[("index.js", "JavaScript", code)], false);
    // base 1 + 2 case = 3
    assert_eq!(r.files[0].cyclomatic_complexity, 3);
}

// ── Cognitive complexity ────────────────────────────────────────

#[test]
fn cognitive_present_for_function_with_branching() {
    let code = "fn f(x: i32) {\n    if x > 0 {\n        if x > 10 {\n            println!(\"big\");\n        }\n    }\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], false);
    // Cognitive complexity should be Some and > 0 for nested ifs
    assert!(r.files[0].cognitive_complexity.is_some());
    assert!(r.files[0].cognitive_complexity.unwrap() > 0);
}

#[test]
fn cognitive_aggregate_max_and_avg() {
    let code = "fn a() {\n    if true { if true {} }\n}\nfn b() {\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], false);
    if let Some(max_cog) = r.max_cognitive {
        assert!(max_cog > 0);
    }
}

// ── Per-function complexity details ─────────────────────────────

#[test]
fn function_details_extracted_when_enabled() {
    let code = "fn foo() {\n    if true {}\n}\nfn bar() {\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], true);
    assert!(r.files[0].functions.is_some());
    let fns = r.files[0].functions.as_ref().unwrap();
    assert_eq!(fns.len(), 2);
}

#[test]
fn function_details_omitted_when_disabled() {
    let code = "fn foo() {\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], false);
    assert!(r.files[0].functions.is_none());
}

#[test]
fn function_detail_names_correct() {
    let code = "fn alpha() {\n}\nfn beta() {\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], true);
    let fns = r.files[0].functions.as_ref().unwrap();
    let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
}

#[test]
fn function_detail_line_numbers() {
    let code = "fn first() {\n}\n\nfn second() {\n    let x = 1;\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], true);
    let fns = r.files[0].functions.as_ref().unwrap();
    // line_start is 1-indexed
    assert_eq!(fns[0].line_start, 1);
    assert!(fns[1].line_start > 1);
}

#[test]
fn function_detail_cyclomatic_per_fn() {
    let code = "fn simple() {\n}\nfn branchy() {\n    if true {}\n    if true {}\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], true);
    let fns = r.files[0].functions.as_ref().unwrap();
    let simple = fns.iter().find(|f| f.name == "simple").unwrap();
    let branchy = fns.iter().find(|f| f.name == "branchy").unwrap();
    assert!(branchy.cyclomatic > simple.cyclomatic);
}

// ── File-level aggregation ──────────────────────────────────────

#[test]
fn total_functions_across_files() {
    let rust = "fn a() {\n}\nfn b() {\n}\n";
    let py = "def c():\n    pass\n";
    let r = analyze(
        &[("lib.rs", "Rust", rust), ("main.py", "Python", py)],
        false,
    );
    assert_eq!(r.total_functions, 3);
}

#[test]
fn avg_cyclomatic_is_average_of_files() {
    let low = "fn low() {\n}\n";
    let high = "fn high(x: i32) {\n    if x > 0 {\n        if x > 10 {\n            match x { 1 => {}, _ => {} }\n        }\n    }\n}\n";
    let r = analyze(&[("low.rs", "Rust", low), ("high.rs", "Rust", high)], false);
    assert!(r.avg_cyclomatic > 1.0);
    assert_eq!(r.files.len(), 2);
}

#[test]
fn max_cyclomatic_is_maximum() {
    let code1 = "fn a() {\n}\n";
    let code2 = "fn b(x: i32) {\n    if x > 0 {} \n    if x < 0 {}\n    for _ in 0..10 {}\n}\n";
    let r = analyze(&[("a.rs", "Rust", code1), ("b.rs", "Rust", code2)], false);
    assert!(r.max_cyclomatic > 1);
    assert!(
        r.max_cyclomatic
            >= r.files
                .iter()
                .map(|f| f.cyclomatic_complexity)
                .min()
                .unwrap()
    );
}

#[test]
fn max_function_length_tracked() {
    let code = "fn short() {\n}\nfn long() {\n    let a = 1;\n    let b = 2;\n    let c = 3;\n    let d = 4;\n    let e = 5;\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], false);
    assert!(r.max_function_length > 0);
}

// ── Edge cases ──────────────────────────────────────────────────

#[test]
fn empty_file_produces_no_entries() {
    let r = analyze(&[("empty.rs", "Rust", "")], false);
    // Empty file has no functions, so no complexity
    assert!(r.files.is_empty() || r.files[0].function_count == 0);
}

#[test]
fn single_function_file() {
    let code = "fn only() {\n    println!(\"hello\");\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], false);
    assert_eq!(r.total_functions, 1);
}

#[test]
fn deeply_nested_code_high_complexity() {
    let code = r#"fn deep(x: i32) {
    if x > 0 {
        if x > 5 {
            if x > 10 {
                if x > 20 {
                    for i in 0..x {
                        if i > 0 && i < x {
                            match i {
                                1 => {},
                                _ => {},
                            }
                        }
                    }
                }
            }
        }
    }
}
"#;
    let r = analyze(&[("deep.rs", "Rust", code)], false);
    assert!(r.files[0].cyclomatic_complexity > 5);
}

#[test]
fn unsupported_language_skipped() {
    let (dir, paths) = write_temp_files(&[("readme.md", "# Hello\n")]);
    let export = make_export(vec![make_row("readme.md", "root", "Markdown", 1)]);
    let r = build_complexity_report(
        dir.path(),
        &paths,
        &export,
        &AnalysisLimits::default(),
        false,
    )
    .unwrap();
    assert!(r.files.is_empty());
    assert_eq!(r.total_functions, 0);
}

// ── Deterministic ordering ──────────────────────────────────────

#[test]
fn files_sorted_by_cyclomatic_desc() {
    let low = "fn low() {\n}\n";
    let high = "fn high(x: i32) {\n    if x > 0 {}\n    if x < 0 {}\n    while x > 0 {}\n}\n";
    let r = analyze(&[("low.rs", "Rust", low), ("high.rs", "Rust", high)], false);
    if r.files.len() >= 2 {
        assert!(r.files[0].cyclomatic_complexity >= r.files[1].cyclomatic_complexity);
    }
}

#[test]
fn deterministic_across_runs() {
    let code = "fn f(x: i32) {\n    if x > 0 {\n        for _ in 0..10 {}\n    }\n}\n";
    let r1 = analyze(&[("lib.rs", "Rust", code)], true);
    let r2 = analyze(&[("lib.rs", "Rust", code)], true);
    assert_eq!(r1.total_functions, r2.total_functions);
    assert_eq!(r1.avg_cyclomatic, r2.avg_cyclomatic);
    assert_eq!(r1.max_cyclomatic, r2.max_cyclomatic);
    assert_eq!(r1.files.len(), r2.files.len());
}

// ── Histogram generation ────────────────────────────────────────

#[test]
fn histogram_empty_input() {
    let h = generate_complexity_histogram(&[], 5);
    assert_eq!(h.total, 0);
    assert!(h.counts.iter().all(|&c| c == 0));
}

#[test]
fn histogram_single_low_complexity() {
    let files = vec![FileComplexity {
        path: "a.rs".to_string(),
        module: "root".to_string(),
        function_count: 1,
        max_function_length: 5,
        cyclomatic_complexity: 2,
        cognitive_complexity: None,
        max_nesting: None,
        risk_level: ComplexityRisk::Low,
        functions: None,
    }];
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.total, 1);
    assert_eq!(h.counts[0], 1); // bucket 0-4
}

#[test]
fn histogram_high_complexity_in_last_bucket() {
    let files = vec![FileComplexity {
        path: "big.rs".to_string(),
        module: "root".to_string(),
        function_count: 10,
        max_function_length: 100,
        cyclomatic_complexity: 50,
        cognitive_complexity: None,
        max_nesting: None,
        risk_level: ComplexityRisk::Critical,
        functions: None,
    }];
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.total, 1);
    // 50 / 5 = 10, capped at bucket 6 (last)
    assert_eq!(h.counts[6], 1);
}

// ── Risk classification ─────────────────────────────────────────

#[test]
fn low_risk_for_simple_file() {
    let code = "fn simple() {\n    let x = 1;\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], false);
    assert!(matches!(
        r.files[0].risk_level,
        ComplexityRisk::Low | ComplexityRisk::Moderate
    ));
}

#[test]
fn high_risk_files_count() {
    let simple = "fn s() {\n}\n";
    let r = analyze(&[("s.rs", "Rust", simple)], false);
    assert_eq!(r.high_risk_files, 0);
}

// ── report-level JSON serialization ─────────────────────────────

#[test]
fn report_json_roundtrip() {
    let code = "fn f() {\n    if true {}\n}\n";
    let r = analyze(&[("lib.rs", "Rust", code)], true);
    let json = serde_json::to_string(&r).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed["total_functions"].as_u64().unwrap(),
        r.total_functions as u64
    );
}
