//! Wave-38 deep tests for `tokmd-analysis-halstead`.
//!
//! Focuses on areas not yet covered by the existing deep/bdd/edge/unit suites:
//! - Ruby-specific tokenization (spaceship <=>, regex =~, splat ...)
//! - C-family specific tokenization (sizeof, typeof, ::, ->)
//! - round_f64 edge cases (negative values, large values, zero decimals)
//! - is_halstead_lang coverage for all supported + unsupported languages
//! - operators_for_lang returns empty for unsupported
//! - HalsteadMetrics serde round-trip
//! - HalsteadMetrics JSON shape validation
//! - time_seconds and estimated_bugs formulas
//! - Comment-only file produces all-zero metrics
//! - Multi-file aggregation merges operator/operand counts
//! - Unsupported language files are skipped in build_halstead_report
//! - FileTokenCounts fields consistency
//! - Tokenizer handles CRLF line endings
//! - Logical operators (&&, ||) counted correctly
//! - Nested expressions with many distinct operands

use std::path::PathBuf;

use tokmd_analysis_halstead::{
    build_halstead_report, is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead,
};
use tokmd_analysis_types::HalsteadMetrics;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn no_limits() -> AnalysisLimits {
    AnalysisLimits {
        max_files: None,
        max_bytes: None,
        max_file_bytes: None,
        max_commits: None,
        max_commit_files: None,
    }
}

fn make_row(path: &str, lang: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: String::new(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code: 10,
        comments: 0,
        blanks: 0,
        lines: 10,
        bytes: 100,
        tokens: 50,
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

fn build_report_for_code(code: &str, lang: &str, filename: &str) -> HalsteadMetrics {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(filename), code).unwrap();
    let export = make_export(vec![make_row(filename, lang)]);
    let files = vec![PathBuf::from(filename)];
    build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap()
}

// ═══════════════════════════════════════════════════════════════════
// § Ruby-specific tokenization
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ruby_spaceship_operator() {
    let counts = tokenize_for_halstead("a <=> b", "ruby");
    assert!(
        counts.operators.contains_key("<=>"),
        "Ruby should detect spaceship operator <=>"
    );
}

#[test]
fn ruby_regex_match_operator() {
    let counts = tokenize_for_halstead("x =~ pattern", "ruby");
    assert!(
        counts.operators.contains_key("=~"),
        "Ruby should detect regex match operator =~"
    );
}

#[test]
fn ruby_not_match_operator() {
    let counts = tokenize_for_halstead("x !~ pattern", "ruby");
    assert!(
        counts.operators.contains_key("!~"),
        "Ruby should detect negated match operator !~"
    );
}

#[test]
fn ruby_range_operators() {
    let counts = tokenize_for_halstead("a = 1..10\nb = 1...10", "ruby");
    assert!(counts.operators.contains_key(".."));
    assert!(counts.operators.contains_key("..."));
}

#[test]
fn ruby_keywords_as_operators() {
    let code = "def foo\n  yield\n  rescue\n  ensure\nend";
    let counts = tokenize_for_halstead(code, "ruby");
    assert!(counts.operators.contains_key("def"));
    assert!(counts.operators.contains_key("yield"));
    assert!(counts.operators.contains_key("rescue"));
    assert!(counts.operators.contains_key("ensure"));
    assert!(counts.operators.contains_key("end"));
}

// ═══════════════════════════════════════════════════════════════════
// § C-family tokenization
// ═══════════════════════════════════════════════════════════════════

#[test]
fn c_sizeof_operator() {
    let counts = tokenize_for_halstead("int x = sizeof(int);", "c");
    assert!(counts.operators.contains_key("sizeof"));
}

#[test]
fn c_arrow_and_scope_operators() {
    let counts = tokenize_for_halstead("ptr->field\nClass::method", "c++");
    assert!(counts.operators.contains_key("->"));
    assert!(counts.operators.contains_key("::"));
}

// ═══════════════════════════════════════════════════════════════════
// § round_f64 edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn round_f64_zero_decimals() {
    assert_eq!(round_f64(3.7, 0), 4.0);
    assert_eq!(round_f64(3.2, 0), 3.0);
}

#[test]
fn round_f64_negative_values() {
    assert_eq!(round_f64(-1.555, 2), -1.56);
    assert_eq!(round_f64(-0.005, 2), -0.01);
}

#[test]
fn round_f64_large_values() {
    let result = round_f64(123456.789, 2);
    assert!((result - 123456.79).abs() < 0.001);
}

#[test]
fn round_f64_zero() {
    assert_eq!(round_f64(0.0, 5), 0.0);
}

// ═══════════════════════════════════════════════════════════════════
// § is_halstead_lang coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn all_supported_languages() {
    let supported = [
        "rust",
        "Rust",
        "RUST",
        "javascript",
        "JavaScript",
        "typescript",
        "TypeScript",
        "python",
        "Python",
        "go",
        "Go",
        "c",
        "C",
        "c++",
        "C++",
        "java",
        "Java",
        "c#",
        "C#",
        "php",
        "PHP",
        "ruby",
        "Ruby",
    ];
    for lang in &supported {
        assert!(is_halstead_lang(lang), "{lang} should be supported");
    }
}

#[test]
fn unsupported_languages() {
    let unsupported = ["haskell", "erlang", "fortran", "cobol", "lua", ""];
    for lang in &unsupported {
        assert!(!is_halstead_lang(lang), "{lang} should not be supported");
    }
}

// ═══════════════════════════════════════════════════════════════════
// § operators_for_lang returns empty for unsupported
// ═══════════════════════════════════════════════════════════════════

#[test]
fn operators_for_unsupported_lang_empty() {
    assert!(operators_for_lang("haskell").is_empty());
    assert!(operators_for_lang("").is_empty());
    assert!(operators_for_lang("brainfuck").is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// § HalsteadMetrics serde round-trip
// ═══════════════════════════════════════════════════════════════════

#[test]
fn halstead_metrics_serde_round_trip() {
    let m = build_report_for_code("let x = 1\nlet y = x + 2", "Rust", "test.rs");
    let json = serde_json::to_string(&m).unwrap();
    let deserialized: HalsteadMetrics = serde_json::from_str(&json).unwrap();

    assert_eq!(m.distinct_operators, deserialized.distinct_operators);
    assert_eq!(m.distinct_operands, deserialized.distinct_operands);
    assert_eq!(m.total_operators, deserialized.total_operators);
    assert_eq!(m.total_operands, deserialized.total_operands);
    assert_eq!(m.vocabulary, deserialized.vocabulary);
    assert_eq!(m.length, deserialized.length);
    assert!((m.volume - deserialized.volume).abs() < 0.01);
    assert!((m.difficulty - deserialized.difficulty).abs() < 0.01);
    assert!((m.effort - deserialized.effort).abs() < 0.01);
    assert!((m.time_seconds - deserialized.time_seconds).abs() < 0.01);
    assert!((m.estimated_bugs - deserialized.estimated_bugs).abs() < 0.0001);
}

// ═══════════════════════════════════════════════════════════════════
// § HalsteadMetrics JSON shape
// ═══════════════════════════════════════════════════════════════════

#[test]
fn halstead_metrics_json_shape() {
    let m = build_report_for_code("let x = 1;", "Rust", "test.rs");
    let v: serde_json::Value = serde_json::to_value(&m).unwrap();

    assert!(v.is_object());
    let expected_keys = [
        "distinct_operators",
        "distinct_operands",
        "total_operators",
        "total_operands",
        "vocabulary",
        "length",
        "volume",
        "difficulty",
        "effort",
        "time_seconds",
        "estimated_bugs",
    ];
    for key in &expected_keys {
        assert!(v.get(key).is_some(), "JSON should have key '{key}'");
    }
}

// ═══════════════════════════════════════════════════════════════════
// § time_seconds and estimated_bugs formulas
// ═══════════════════════════════════════════════════════════════════

#[test]
fn time_and_bugs_formulas() {
    let m = build_report_for_code(
        "fn compute(a: i32, b: i32) -> i32 { let x = a + b; let y = a * b; x - y }",
        "Rust",
        "f.rs",
    );

    // time = effort / 18
    let expected_time = round_f64(m.effort / 18.0, 2);
    assert!(
        (m.time_seconds - expected_time).abs() < 0.01,
        "time_seconds ({}) should be effort/18 ({})",
        m.time_seconds,
        expected_time
    );

    // bugs = volume / 3000
    let expected_bugs = round_f64(m.volume / 3000.0, 4);
    assert!(
        (m.estimated_bugs - expected_bugs).abs() < 0.001,
        "estimated_bugs ({}) should be volume/3000 ({})",
        m.estimated_bugs,
        expected_bugs
    );
}

// ═══════════════════════════════════════════════════════════════════
// § Comment-only file produces all-zero metrics
// ═══════════════════════════════════════════════════════════════════

#[test]
fn comment_only_file_all_zeros() {
    let code = "// this is a comment\n// another comment\n// yet another";
    let m = build_report_for_code(code, "Rust", "comments.rs");

    assert_eq!(m.distinct_operators, 0);
    assert_eq!(m.distinct_operands, 0);
    assert_eq!(m.total_operators, 0);
    assert_eq!(m.total_operands, 0);
    assert_eq!(m.volume, 0.0);
    assert_eq!(m.difficulty, 0.0);
    assert_eq!(m.effort, 0.0);
}

// ═══════════════════════════════════════════════════════════════════
// § Multi-file aggregation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn multi_file_aggregation_merges_counts() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "let x = 1;").unwrap();
    std::fs::write(dir.path().join("b.rs"), "let y = 2;").unwrap();

    let export = make_export(vec![make_row("a.rs", "Rust"), make_row("b.rs", "Rust")]);
    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    // Each file: let(1) =(1) → 2 operators, operand(2)
    // Merged: let(2) =(2) → total_operators=4, operands: {x,1,y,2}
    assert_eq!(m.total_operators, 4);
    assert_eq!(m.distinct_operators, 2); // let, =
    assert_eq!(m.distinct_operands, 4); // x, 1, y, 2
}

// ═══════════════════════════════════════════════════════════════════
// § Unsupported language files are skipped
// ═══════════════════════════════════════════════════════════════════

#[test]
fn unsupported_language_files_skipped() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("main.hs"),
        "module Main where\nmain = putStrLn \"hello\"",
    )
    .unwrap();

    let export = make_export(vec![make_row("main.hs", "Haskell")]);
    let files = vec![PathBuf::from("main.hs")];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    assert_eq!(m.distinct_operators, 0);
    assert_eq!(m.distinct_operands, 0);
    assert_eq!(m.volume, 0.0);
}

// ═══════════════════════════════════════════════════════════════════
// § FileTokenCounts consistency
// ═══════════════════════════════════════════════════════════════════

#[test]
fn file_token_counts_consistency() {
    let counts = tokenize_for_halstead("let a = 1\nlet b = a + 2", "rust");

    // total_operators should equal sum of all operator counts
    let sum_ops: usize = counts.operators.values().sum();
    assert_eq!(
        counts.total_operators, sum_ops,
        "total_operators should equal sum of individual operator counts"
    );

    // distinct_operators = operators.len()
    assert_eq!(counts.operators.len(), 3); // let, =, +

    // total_operands >= operands.len() (distinct)
    assert!(
        counts.total_operands >= counts.operands.len(),
        "total_operands ({}) should be >= distinct operands ({})",
        counts.total_operands,
        counts.operands.len()
    );
}

// ═══════════════════════════════════════════════════════════════════
// § CRLF line endings handled
// ═══════════════════════════════════════════════════════════════════

#[test]
fn crlf_line_endings_handled() {
    let code = "let x = 1;\r\nlet y = 2;\r\n";
    let counts = tokenize_for_halstead(code, "rust");
    assert_eq!(counts.total_operators, 4); // let, =, let, =
    assert!(counts.operands.contains("x"));
    assert!(counts.operands.contains("y"));
}

// ═══════════════════════════════════════════════════════════════════
// § Logical operators counted correctly
// ═══════════════════════════════════════════════════════════════════

#[test]
fn logical_operators_counted() {
    let code = "if a && b || c { return true; }";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operators.contains_key("&&"));
    assert!(counts.operators.contains_key("||"));
    assert_eq!(*counts.operators.get("&&").unwrap(), 1);
    assert_eq!(*counts.operators.get("||").unwrap(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// § Many distinct operands increase vocabulary
// ═══════════════════════════════════════════════════════════════════

#[test]
fn many_distinct_operands_increase_vocabulary() {
    let code =
        "let a = 1\nlet b = 2\nlet c = 3\nlet d = 4\nlet e = 5\nlet f = 6\nlet g = 7\nlet h = 8";
    let m = build_report_for_code(code, "Rust", "many.rs");
    // 2 distinct operators (let, =), 16 distinct operands (a-h, 1-8)
    assert_eq!(m.distinct_operators, 2);
    assert_eq!(m.distinct_operands, 16);
    assert_eq!(m.vocabulary, 18);
}

// ═══════════════════════════════════════════════════════════════════
// § HalsteadMetrics deserialization from known JSON
// ═══════════════════════════════════════════════════════════════════

#[test]
fn halstead_metrics_deserializes_from_known_json() {
    let json = r#"{
        "distinct_operators": 3,
        "distinct_operands": 4,
        "total_operators": 5,
        "total_operands": 5,
        "vocabulary": 7,
        "length": 10,
        "volume": 28.07,
        "difficulty": 1.88,
        "effort": 52.77,
        "time_seconds": 2.93,
        "estimated_bugs": 0.0094
    }"#;
    let m: HalsteadMetrics = serde_json::from_str(json).unwrap();
    assert_eq!(m.distinct_operators, 3);
    assert_eq!(m.distinct_operands, 4);
    assert_eq!(m.vocabulary, 7);
    assert_eq!(m.length, 10);
    assert!((m.volume - 28.07).abs() < 0.01);
}

// ═══════════════════════════════════════════════════════════════════
// § Python hash comment lines are skipped
// ═══════════════════════════════════════════════════════════════════

#[test]
fn python_hash_comments_skipped() {
    let code = "# this is a comment\nx = 1\n# another comment\ny = 2";
    let counts = tokenize_for_halstead(code, "python");
    // Only lines 2 and 4 are parsed
    assert_eq!(counts.total_operators, 2); // =, =
    assert_eq!(counts.total_operands, 4); // x, 1, y, 2
}

// ═══════════════════════════════════════════════════════════════════
// § Difficulty zero when no operands
// ═══════════════════════════════════════════════════════════════════

#[test]
fn difficulty_zero_when_no_operands() {
    let m = build_report_for_code("return", "Rust", "ret.rs");
    assert_eq!(m.difficulty, 0.0);
}
