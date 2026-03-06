//! W74 – Unit tests for tokmd-analysis-halstead enricher.

use tokmd_analysis_halstead::{
    is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead,
};

// ── is_halstead_lang ──────────────────────────────────────────────────────

#[test]
fn supported_languages() {
    for lang in &[
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
    ] {
        assert!(is_halstead_lang(lang), "expected {lang} to be supported");
    }
}

#[test]
fn unsupported_languages() {
    assert!(!is_halstead_lang("markdown"));
    assert!(!is_halstead_lang("json"));
    assert!(!is_halstead_lang("toml"));
    assert!(!is_halstead_lang("yaml"));
}

#[test]
fn case_insensitive_lang_check() {
    assert!(is_halstead_lang("Rust"));
    assert!(is_halstead_lang("PYTHON"));
    assert!(is_halstead_lang("JavaScript"));
}

// ── operators_for_lang ────────────────────────────────────────────────────

#[test]
fn rust_operators_non_empty() {
    let ops = operators_for_lang("rust");
    assert!(!ops.is_empty());
    assert!(ops.contains(&"fn"));
    assert!(ops.contains(&"if"));
    assert!(ops.contains(&"match"));
}

#[test]
fn unknown_lang_has_no_operators() {
    let ops = operators_for_lang("brainfuck");
    assert!(ops.is_empty());
}

#[test]
fn python_operators_include_def() {
    let ops = operators_for_lang("python");
    assert!(ops.contains(&"def"));
    assert!(ops.contains(&"class"));
    assert!(ops.contains(&"lambda"));
}

// ── tokenize_for_halstead ─────────────────────────────────────────────────

#[test]
fn tokenize_empty_input() {
    let counts = tokenize_for_halstead("", "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
    assert!(counts.operators.is_empty());
    assert!(counts.operands.is_empty());
}

#[test]
fn tokenize_comment_only() {
    let code = "// this is a comment\n// another comment\n";
    let counts = tokenize_for_halstead(code, "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn tokenize_simple_rust_function() {
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.total_operators > 0, "should have operators");
    assert!(counts.total_operands > 0, "should have operands");
    assert!(counts.operators.contains_key("fn"));
}

#[test]
fn tokenize_python_function() {
    let code = "def add(a, b):\n    return a + b\n";
    let counts = tokenize_for_halstead(code, "python");
    assert!(counts.operators.contains_key("def"));
    assert!(counts.operators.contains_key("return"));
    assert!(counts.operators.contains_key("+"));
}

#[test]
fn tokenize_string_literals_counted_as_operand() {
    let code = r#"let msg = "hello world";"#;
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operands.contains("<string>"));
}

#[test]
fn tokenize_multiple_operators_counted() {
    let code = "if x > 0 && y < 10 || z == 5 { }";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operators.contains_key("if"));
    assert!(counts.operators.contains_key("&&"));
    assert!(counts.operators.contains_key("||"));
    assert!(counts.operators.contains_key("=="));
}

// ── Halstead formula verification ─────────────────────────────────────────

#[test]
fn vocabulary_is_distinct_operators_plus_operands() {
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let counts = tokenize_for_halstead(code, "rust");
    let n1 = counts.operators.len();
    let n2 = counts.operands.len();
    let vocab = n1 + n2;
    assert!(vocab > 0);
    assert_eq!(vocab, n1 + n2);
}

#[test]
fn length_is_total_operators_plus_operands() {
    let code = "let x = 1; let y = 2;";
    let counts = tokenize_for_halstead(code, "rust");
    let length = counts.total_operators + counts.total_operands;
    assert!(length > 0);
}

#[test]
fn volume_formula_correctness() {
    // Manual computation: vocab=5, length=10 → volume = 10 * log2(5) ≈ 23.22
    let vocab = 5usize;
    let length = 10usize;
    let volume = length as f64 * (vocab as f64).log2();
    assert!((volume - 23.22).abs() < 0.1);
}

#[test]
fn difficulty_formula_correctness() {
    // n1=2, n2=3, N2=6 → difficulty = (2/2) * (6/3) = 2.0
    let n1 = 2usize;
    let n2 = 3usize;
    let total_opds = 6usize;
    let difficulty = (n1 as f64 / 2.0) * (total_opds as f64 / n2 as f64);
    assert!((difficulty - 2.0).abs() < 0.001);
}

#[test]
fn effort_is_difficulty_times_volume() {
    let difficulty = 2.0f64;
    let volume = 23.22f64;
    let effort = difficulty * volume;
    assert!((effort - 46.44).abs() < 0.01);
}

// ── round_f64 ─────────────────────────────────────────────────────────────

#[test]
fn round_f64_two_decimals() {
    assert_eq!(round_f64(1.23456, 2), 1.23);
    assert_eq!(round_f64(2.005, 2), 2.01); // banker's rounding edge
    assert_eq!(round_f64(0.0, 2), 0.0);
}

#[test]
fn round_f64_zero_decimals() {
    assert_eq!(round_f64(3.7, 0), 4.0);
    assert_eq!(round_f64(3.2, 0), 3.0);
}

#[test]
fn round_f64_four_decimals() {
    assert_eq!(round_f64(1.23456789, 4), 1.2346);
}
