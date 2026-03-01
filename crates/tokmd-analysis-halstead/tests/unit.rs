//! Unit tests for tokmd-analysis-halstead metric computation.

use std::path::PathBuf;

use tokmd_analysis_halstead::{
    build_halstead_report, is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead,
};
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── helpers ──────────────────────────────────────────────────────────

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

// ── 1. Known operator/operand counts ─────────────────────────────────

#[test]
fn known_counts_single_assignment() {
    // `let x = 1;` → operators: let, = → N1=2, n1=2; operands: x, 1 → N2=2, n2=2
    let counts = tokenize_for_halstead("let x = 1;", "rust");
    assert_eq!(*counts.operators.get("let").unwrap(), 1);
    assert_eq!(*counts.operators.get("=").unwrap(), 1);
    assert_eq!(counts.total_operators, 2);
    assert!(counts.operands.contains("x"));
    assert!(counts.operands.contains("1"));
    assert_eq!(counts.total_operands, 2);
}

#[test]
fn known_counts_repeated_operator() {
    // `x + y + z` → operator "+" appears 2 times
    let counts = tokenize_for_halstead("x + y + z", "rust");
    assert_eq!(*counts.operators.get("+").unwrap(), 2);
    assert_eq!(counts.total_operators, 2);
    // distinct operands: x, y, z
    assert_eq!(counts.operands.len(), 3);
    assert_eq!(counts.total_operands, 3);
}

#[test]
fn known_counts_compound_operators() {
    // Verify compound assignment operators are recognized as single tokens
    let counts = tokenize_for_halstead("x += 1", "rust");
    assert!(counts.operators.contains_key("+="));
    assert_eq!(counts.total_operators, 1);
}

// ── 2. Boundary: zero operators ──────────────────────────────────────

#[test]
fn boundary_zero_operators_unknown_lang() {
    // Unknown language → no operators recognized, all tokens become operands
    let counts = tokenize_for_halstead("foo bar baz", "unknown");
    assert_eq!(counts.total_operators, 0);
    assert!(counts.operators.is_empty());
    assert_eq!(counts.total_operands, 3);
    assert_eq!(counts.operands.len(), 3);
}

#[test]
fn boundary_zero_operands() {
    // Only operators, no identifiers/numbers
    let counts = tokenize_for_halstead("return", "rust");
    assert_eq!(counts.total_operators, 1);
    assert_eq!(counts.total_operands, 0);
    assert!(counts.operands.is_empty());
}

#[test]
fn boundary_single_operator() {
    let counts = tokenize_for_halstead("if", "rust");
    assert_eq!(counts.total_operators, 1);
    assert_eq!(counts.operators.len(), 1);
    assert!(counts.operators.contains_key("if"));
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn boundary_empty_string() {
    let counts = tokenize_for_halstead("", "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
    assert!(counts.operators.is_empty());
    assert!(counts.operands.is_empty());
}

// ── 3. Mathematical relationships ────────────────────────────────────

#[test]
fn math_volume_equals_n_times_log2_n() {
    // Build metrics from a known file, then verify volume = length * log2(vocabulary)
    let dir = tempfile::tempdir().unwrap();
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    std::fs::write(dir.path().join("f.rs"), code).unwrap();

    let export = make_export(vec![make_row("f.rs", "Rust")]);
    let files = vec![PathBuf::from("f.rs")];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    let expected_volume = m.length as f64 * (m.vocabulary as f64).log2();
    assert!(
        (m.volume - round_f64(expected_volume, 2)).abs() < 0.01,
        "volume ({}) should equal N*log2(n) ({})",
        m.volume,
        expected_volume
    );
}

#[test]
fn math_difficulty_formula() {
    let dir = tempfile::tempdir().unwrap();
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    std::fs::write(dir.path().join("f.rs"), code).unwrap();

    let export = make_export(vec![make_row("f.rs", "Rust")]);
    let files = vec![PathBuf::from("f.rs")];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    // difficulty = (n1/2) * (N2/n2)
    let expected = (m.distinct_operators as f64 / 2.0)
        * (m.total_operands as f64 / m.distinct_operands as f64);
    assert!(
        (m.difficulty - round_f64(expected, 2)).abs() < 0.01,
        "difficulty ({}) should equal (n1/2)*(N2/n2) ({})",
        m.difficulty,
        expected
    );
}

#[test]
fn math_effort_equals_difficulty_times_volume() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("f.rs"), "let x = 1 + 2 + 3;").unwrap();

    let export = make_export(vec![make_row("f.rs", "Rust")]);
    let files = vec![PathBuf::from("f.rs")];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    let expected_effort = round_f64(m.difficulty * m.volume, 2);
    assert!(
        (m.effort - expected_effort).abs() < 0.01,
        "effort ({}) should equal D*V ({})",
        m.effort,
        expected_effort
    );
}

#[test]
fn math_time_equals_effort_over_18() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("f.rs"), "let x = 1 + 2 + 3;").unwrap();

    let export = make_export(vec![make_row("f.rs", "Rust")]);
    let files = vec![PathBuf::from("f.rs")];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    let expected_time = round_f64(m.effort / 18.0, 2);
    assert!(
        (m.time_seconds - expected_time).abs() < 0.01,
        "time ({}) should equal E/18 ({})",
        m.time_seconds,
        expected_time
    );
}

#[test]
fn math_bugs_equals_volume_over_3000() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("f.rs"), "let x = 1 + 2 + 3;").unwrap();

    let export = make_export(vec![make_row("f.rs", "Rust")]);
    let files = vec![PathBuf::from("f.rs")];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    let expected_bugs = round_f64(m.volume / 3000.0, 4);
    assert!(
        (m.estimated_bugs - expected_bugs).abs() < 0.0001,
        "bugs ({}) should equal V/3000 ({})",
        m.estimated_bugs,
        expected_bugs
    );
}

#[test]
fn math_vocabulary_equals_n1_plus_n2() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("f.rs"), "fn foo() { let x = 1; }").unwrap();

    let export = make_export(vec![make_row("f.rs", "Rust")]);
    let files = vec![PathBuf::from("f.rs")];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    assert_eq!(
        m.vocabulary,
        m.distinct_operators + m.distinct_operands,
        "vocabulary should equal n1 + n2"
    );
}

#[test]
fn math_length_equals_big_n1_plus_big_n2() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("f.rs"), "fn foo() { let x = 1; }").unwrap();

    let export = make_export(vec![make_row("f.rs", "Rust")]);
    let files = vec![PathBuf::from("f.rs")];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    assert_eq!(
        m.length,
        m.total_operators + m.total_operands,
        "length should equal N1 + N2"
    );
}

// ── 4. Zero-division safety in build_halstead_report ─────────────────

#[test]
fn zero_vocabulary_yields_zero_derived_metrics() {
    let dir = tempfile::tempdir().unwrap();
    let export = make_export(vec![]);
    let files: Vec<PathBuf> = vec![];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    assert_eq!(m.vocabulary, 0);
    assert_eq!(m.length, 0);
    assert_eq!(m.volume, 0.0);
    assert_eq!(m.difficulty, 0.0);
    assert_eq!(m.effort, 0.0);
    assert_eq!(m.time_seconds, 0.0);
    assert_eq!(m.estimated_bugs, 0.0);
}

// ── 5. Language support ──────────────────────────────────────────────

#[test]
fn is_halstead_lang_case_insensitive() {
    for &lang in &["rust", "RUST", "Rust", "rUsT"] {
        assert!(is_halstead_lang(lang), "{lang} should be supported");
    }
}

#[test]
fn unsupported_lang_returns_empty_operators() {
    assert!(operators_for_lang("brainfuck").is_empty());
    assert!(operators_for_lang("").is_empty());
    assert!(operators_for_lang("markdown").is_empty());
}

// ── 6. round_f64 edge cases ─────────────────────────────────────────

#[test]
#[allow(clippy::approx_constant)]
fn round_f64_various_precisions() {
    assert_eq!(round_f64(std::f64::consts::PI, 0), 3.0);
    assert_eq!(round_f64(std::f64::consts::PI, 2), 3.14);
    assert_eq!(round_f64(std::f64::consts::PI, 4), 3.1416);
    assert_eq!(round_f64(0.0, 5), 0.0);
    assert_eq!(round_f64(-1.5, 0), -2.0);
    // Idempotent
    let v = round_f64(1.23456, 3);
    assert_eq!(round_f64(v, 3), v);
}

// ── 7. Tokenizer: string literal handling ───────────────────────────

#[test]
fn string_literal_counted_as_single_operand() {
    let code = r#"let s = "hello world";"#;
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operands.contains("<string>"));
    assert!(counts.operands.contains("s"));
    // The string literal itself counts as one operand occurrence
    assert!(counts.total_operands >= 2);
}

// ── 8. Tokenizer: multi-language consistency ─────────────────────────

#[test]
fn typescript_shares_javascript_operators() {
    let js_ops = operators_for_lang("javascript");
    let ts_ops = operators_for_lang("typescript");
    assert_eq!(
        js_ops, ts_ops,
        "JS and TS should share the same operator table"
    );
}

#[test]
fn c_family_shares_operator_table() {
    let c_ops = operators_for_lang("c");
    let cpp_ops = operators_for_lang("c++");
    let java_ops = operators_for_lang("java");
    let csharp_ops = operators_for_lang("c#");
    let php_ops = operators_for_lang("php");
    assert_eq!(c_ops, cpp_ops);
    assert_eq!(c_ops, java_ops);
    assert_eq!(c_ops, csharp_ops);
    assert_eq!(c_ops, php_ops);
}
