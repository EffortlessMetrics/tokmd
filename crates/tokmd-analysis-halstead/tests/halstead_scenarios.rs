//! Deeper scenario tests for Halstead metric calculations.
//!
//! Covers known expected values, determinism, and edge cases.

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

// ===========================================================================
// Known expected values: simple Rust program
// ===========================================================================

#[test]
fn known_values_simple_rust_assignment() {
    // "let x = 1;" has operators: "let", "=" and operands: "x", "1"
    let counts = tokenize_for_halstead("let x = 1;", "rust");
    assert!(counts.operators.contains_key("let"));
    assert!(counts.operators.contains_key("="));
    assert!(counts.operands.contains("x"));
    assert!(counts.operands.contains("1"));
    // n1=2, n2=2, N1=2, N2=2
    assert_eq!(counts.operators.len(), 2, "distinct operators");
    assert_eq!(counts.operands.len(), 2, "distinct operands");
    assert_eq!(counts.total_operators, 2, "total operators");
    assert_eq!(counts.total_operands, 2, "total operands");
}

#[test]
fn known_values_halstead_formulas_from_counts() {
    // Given: n1=2, n2=2, N1=2, N2=2
    // vocabulary = 4, length = 4
    // volume = 4 * log2(4) = 4 * 2 = 8.0
    // difficulty = (2/2) * (2/2) = 1.0
    // effort = 1.0 * 8.0 = 8.0
    // time = 8.0 / 18.0 ≈ 0.44
    // bugs = 8.0 / 3000.0 ≈ 0.0027
    let n1 = 2usize;
    let n2 = 2usize;
    let total_ops = 2usize;
    let total_opds = 2usize;
    let vocab = n1 + n2;
    let length = total_ops + total_opds;

    assert_eq!(vocab, 4);
    assert_eq!(length, 4);

    let volume = length as f64 * (vocab as f64).log2();
    assert!((volume - 8.0).abs() < 0.001);

    let difficulty = (n1 as f64 / 2.0) * (total_opds as f64 / n2 as f64);
    assert!((difficulty - 1.0).abs() < 0.001);

    let effort = difficulty * volume;
    assert!((effort - 8.0).abs() < 0.001);

    let time = effort / 18.0;
    assert!((time - 0.4444).abs() < 0.01);

    let bugs = volume / 3000.0;
    assert!((bugs - 0.00267).abs() < 0.001);
}

#[test]
fn known_values_larger_program() {
    // A slightly larger program to verify metric ordering
    let code = r#"
fn add(a: i32, b: i32) -> i32 {
    let result = a + b;
    if result > 100 {
        return 0;
    }
    result
}
"#;
    let counts = tokenize_for_halstead(code, "rust");

    // Verify key operators present
    assert!(counts.operators.contains_key("fn"));
    assert!(counts.operators.contains_key("let"));
    assert!(counts.operators.contains_key("if"));
    assert!(counts.operators.contains_key("return"));
    assert!(counts.operators.contains_key("+"));
    assert!(counts.operators.contains_key(">"));
    assert!(counts.operators.contains_key("->"));

    // Verify key operands present
    assert!(counts.operands.contains("add"));
    assert!(counts.operands.contains("a"));
    assert!(counts.operands.contains("b"));
    assert!(counts.operands.contains("i32"));
    assert!(counts.operands.contains("result"));

    // Derived metrics should be positive
    let vocab = counts.operators.len() + counts.operands.len();
    let length = counts.total_operators + counts.total_operands;
    let volume = length as f64 * (vocab as f64).log2();
    assert!(volume > 0.0);
    assert!(vocab > 4, "non-trivial program should have vocab > 4");
    assert!(length > 10, "non-trivial program should have length > 10");
}

// ===========================================================================
// Determinism: same code always produces same results
// ===========================================================================

#[test]
fn determinism_tokenize_identical_across_runs() {
    let code = "fn main() { let x = 1 + 2; let y = x * 3; }";
    let a = tokenize_for_halstead(code, "rust");
    let b = tokenize_for_halstead(code, "rust");
    let c = tokenize_for_halstead(code, "rust");

    assert_eq!(a.total_operators, b.total_operators);
    assert_eq!(a.total_operands, b.total_operands);
    assert_eq!(a.operators, b.operators);
    assert_eq!(a.operands, b.operands);
    assert_eq!(b.total_operators, c.total_operators);
    assert_eq!(b.total_operands, c.total_operands);
    assert_eq!(b.operators, c.operators);
    assert_eq!(b.operands, c.operands);
}

#[test]
fn determinism_across_all_supported_languages() {
    let samples = [
        ("fn main() { let x = 1 + 2; }", "rust"),
        ("def add(a, b):\n    return a + b", "python"),
        ("const add = (a, b) => a + b;", "javascript"),
        ("const add = (a: number, b: number) => a + b;", "typescript"),
        ("func main() { x := 1 + 2 }", "go"),
        ("int add(int a, int b) { return a + b; }", "c"),
        ("int add(int a, int b) { return a + b; }", "java"),
        ("def add(a, b)\n  a + b\nend", "ruby"),
    ];

    for (code, lang) in &samples {
        let a = tokenize_for_halstead(code, lang);
        let b = tokenize_for_halstead(code, lang);
        assert_eq!(
            a.total_operators, b.total_operators,
            "{lang}: operators differ"
        );
        assert_eq!(
            a.total_operands, b.total_operands,
            "{lang}: operands differ"
        );
        assert_eq!(a.operators, b.operators, "{lang}: operator map differs");
        assert_eq!(a.operands, b.operands, "{lang}: operand set differs");
    }
}

#[test]
fn determinism_build_report_produces_same_metrics() {
    let dir = tempfile::tempdir().unwrap();
    let code = "fn main() {\n    let x = 1 + 2;\n    let y = x * 3;\n}\n";
    std::fs::write(dir.path().join("main.rs"), code).unwrap();

    let export = make_export(vec![make_row("main.rs", "Rust")]);
    let files = vec![PathBuf::from("main.rs")];

    let m1 = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();
    let m2 = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    assert_eq!(m1.distinct_operators, m2.distinct_operators);
    assert_eq!(m1.distinct_operands, m2.distinct_operands);
    assert_eq!(m1.total_operators, m2.total_operators);
    assert_eq!(m1.total_operands, m2.total_operands);
    assert_eq!(m1.vocabulary, m2.vocabulary);
    assert_eq!(m1.length, m2.length);
    assert_eq!(m1.volume, m2.volume);
    assert_eq!(m1.difficulty, m2.difficulty);
    assert_eq!(m1.effort, m2.effort);
    assert_eq!(m1.time_seconds, m2.time_seconds);
    assert_eq!(m1.estimated_bugs, m2.estimated_bugs);
}

// ===========================================================================
// Edge case: empty input
// ===========================================================================

#[test]
fn edge_empty_input_all_languages() {
    let langs = [
        "rust",
        "javascript",
        "typescript",
        "python",
        "go",
        "c",
        "c++",
        "java",
        "c#",
        "php",
        "ruby",
    ];
    for lang in &langs {
        let counts = tokenize_for_halstead("", lang);
        assert_eq!(counts.total_operators, 0, "{lang}: empty");
        assert_eq!(counts.total_operands, 0, "{lang}: empty");
        assert!(counts.operators.is_empty(), "{lang}: empty");
        assert!(counts.operands.is_empty(), "{lang}: empty");
    }
}

// ===========================================================================
// Edge case: single operator only
// ===========================================================================

#[test]
fn edge_single_operator_no_operands() {
    let counts = tokenize_for_halstead("return", "rust");
    assert_eq!(counts.total_operators, 1);
    assert_eq!(counts.total_operands, 0);
    assert!(counts.operators.contains_key("return"));
}

#[test]
fn edge_single_operand_no_operators() {
    let counts = tokenize_for_halstead("x", "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 1);
    assert!(counts.operands.contains("x"));
}

// ===========================================================================
// Edge case: whitespace-only and comment-only
// ===========================================================================

#[test]
fn edge_whitespace_only() {
    let counts = tokenize_for_halstead("   \n\t\n   \n", "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn edge_comments_only_rust() {
    let counts = tokenize_for_halstead("// comment\n// another\n", "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn edge_comments_only_python() {
    let counts = tokenize_for_halstead("# comment\n# another\n", "python");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

// ===========================================================================
// Edge case: string literals counted as operands
// ===========================================================================

#[test]
fn edge_string_literal_counted_as_operand() {
    let counts = tokenize_for_halstead(r#"let s = "hello";"#, "rust");
    assert!(counts.operands.contains("<string>"));
    assert!(counts.operands.contains("s"));
    assert!(counts.operators.contains_key("let"));
    assert!(counts.operators.contains_key("="));
}

// ===========================================================================
// Edge case: unknown language produces only operands
// ===========================================================================

#[test]
fn edge_unknown_language_no_operators() {
    let counts = tokenize_for_halstead("fn let if return x y z", "brainfuck");
    assert_eq!(counts.total_operators, 0);
    assert!(counts.total_operands > 0);
}

// ===========================================================================
// Vocabulary and length invariants
// ===========================================================================

#[test]
fn invariant_vocabulary_equals_distinct_sum() {
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let counts = tokenize_for_halstead(code, "rust");
    let vocab = counts.operators.len() + counts.operands.len();
    let length = counts.total_operators + counts.total_operands;

    // length >= vocabulary (repetition means length >= distinct)
    assert!(
        length >= vocab,
        "length ({length}) should be >= vocabulary ({vocab})"
    );
}

#[test]
fn invariant_total_operators_equals_individual_sum() {
    let code = "if x > 0 { return x + 1; } else { return x - 1; }";
    let counts = tokenize_for_halstead(code, "rust");
    let sum: usize = counts.operators.values().sum();
    assert_eq!(counts.total_operators, sum);
}

// ===========================================================================
// Relative ordering: more complex code has higher metrics
// ===========================================================================

#[test]
fn ordering_complex_code_has_higher_volume() {
    let simple = tokenize_for_halstead("let x = 1;", "rust");
    let complex = tokenize_for_halstead(
        r#"
fn process(items: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    for item in items {
        if item > 0 {
            let doubled = item * 2;
            result.push(doubled);
        } else {
            result.push(item);
        }
    }
    return result;
}
"#,
        "rust",
    );

    let simple_len = simple.total_operators + simple.total_operands;
    let complex_len = complex.total_operators + complex.total_operands;
    assert!(complex_len > simple_len);
    assert!(complex.operators.len() >= simple.operators.len());
}

// ===========================================================================
// Multi-char operators: longest match first
// ===========================================================================

#[test]
fn multi_char_operator_longest_match() {
    let counts = tokenize_for_halstead("x >>= 1", "rust");
    assert!(
        counts.operators.contains_key(">>="),
        ">>= should match as single operator"
    );
}

#[test]
fn multi_char_operator_arrow() {
    let counts = tokenize_for_halstead("fn f() -> i32 { 0 }", "rust");
    assert!(counts.operators.contains_key("->"));
}

#[test]
fn multi_char_operator_fat_arrow_js() {
    let counts = tokenize_for_halstead("const f = () => 1;", "javascript");
    assert!(counts.operators.contains_key("=>"));
}

// ===========================================================================
// Duplicate operands: total > distinct
// ===========================================================================

#[test]
fn duplicate_operands_total_exceeds_distinct() {
    let counts = tokenize_for_halstead("x + x + x + x", "rust");
    assert_eq!(counts.operands.len(), 1, "one distinct operand: x");
    assert_eq!(counts.total_operands, 4, "x appears 4 times");
    assert_eq!(*counts.operators.get("+").unwrap(), 3, "+ appears 3 times");
}

// ===========================================================================
// round_f64: additional edge cases
// ===========================================================================

#[test]
fn round_f64_half_up() {
    assert_eq!(round_f64(2.5, 0), 3.0);
    assert_eq!(round_f64(2.45, 1), 2.5);
}

#[test]
fn round_f64_very_small_value() {
    let val = 0.000001;
    let rounded = round_f64(val, 2);
    assert_eq!(rounded, 0.0);
}

#[test]
fn round_f64_large_value() {
    let val = 123456.789;
    assert_eq!(round_f64(val, 0), 123457.0);
    assert_eq!(round_f64(val, 1), 123456.8);
    assert_eq!(round_f64(val, 2), 123456.79);
}

// ===========================================================================
// build_halstead_report: metric identity invariants
// ===========================================================================

#[test]
fn build_report_vocabulary_and_length_identities() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() { let x = 1 + 2; }").unwrap();

    let export = make_export(vec![make_row("main.rs", "Rust")]);
    let files = vec![PathBuf::from("main.rs")];
    let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    assert_eq!(m.vocabulary, m.distinct_operators + m.distinct_operands);
    assert_eq!(m.length, m.total_operators + m.total_operands);
    assert!(m.volume >= 0.0);
    assert!(m.difficulty >= 0.0);
    assert!(m.effort >= 0.0);
    assert!(m.time_seconds >= 0.0);
    assert!(m.estimated_bugs >= 0.0);
}

// ===========================================================================
// is_halstead_lang: case insensitivity
// ===========================================================================

#[test]
fn is_halstead_lang_case_insensitive() {
    assert!(is_halstead_lang("rust"));
    assert!(is_halstead_lang("RUST"));
    assert!(is_halstead_lang("Rust"));
    assert!(is_halstead_lang("rUsT"));
    assert!(!is_halstead_lang(""));
    assert!(!is_halstead_lang("Markdown"));
}

// ===========================================================================
// operators_for_lang: unsupported returns empty
// ===========================================================================

#[test]
fn operators_for_unsupported_lang_empty() {
    assert!(operators_for_lang("").is_empty());
    assert!(operators_for_lang("xml").is_empty());
    assert!(operators_for_lang("css").is_empty());
}
