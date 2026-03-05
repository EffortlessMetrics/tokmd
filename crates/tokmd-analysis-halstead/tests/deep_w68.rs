//! Deep tests for tokmd-analysis-halstead (w68).

use tokmd_analysis_halstead::{
    is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead,
};

// ---------------------------------------------------------------------------
// Language support
// ---------------------------------------------------------------------------

#[test]
fn supported_languages() {
    for lang in &[
        "Rust",
        "JavaScript",
        "TypeScript",
        "Python",
        "Go",
        "C",
        "C++",
        "Java",
        "C#",
        "PHP",
        "Ruby",
    ] {
        assert!(is_halstead_lang(lang), "expected {lang} to be supported");
    }
}

#[test]
fn unsupported_languages() {
    for lang in &["Haskell", "Elixir", "COBOL", ""] {
        assert!(!is_halstead_lang(lang), "expected {lang} to be unsupported");
    }
}

// ---------------------------------------------------------------------------
// Operator tables
// ---------------------------------------------------------------------------

#[test]
fn rust_operators_contain_fn_and_arrow() {
    let ops = operators_for_lang("rust");
    assert!(ops.contains(&"fn"));
    assert!(ops.contains(&"->"));
    assert!(ops.contains(&"=>"));
    assert!(ops.contains(&"::"));
}

#[test]
fn python_operators_contain_def_and_walrus() {
    let ops = operators_for_lang("python");
    assert!(ops.contains(&"def"));
    assert!(ops.contains(&":="));
    assert!(ops.contains(&"lambda"));
}

#[test]
fn javascript_operators_contain_arrow_and_spread() {
    let ops = operators_for_lang("javascript");
    assert!(ops.contains(&"=>"));
    assert!(ops.contains(&"..."));
    assert!(ops.contains(&"==="));
}

#[test]
fn go_operators_contain_short_decl() {
    let ops = operators_for_lang("go");
    assert!(ops.contains(&":="));
    assert!(ops.contains(&"<-"));
    assert!(ops.contains(&"func"));
}

#[test]
fn unknown_lang_returns_empty_ops() {
    let ops = operators_for_lang("brainfuck");
    assert!(ops.is_empty());
}

// ---------------------------------------------------------------------------
// Tokenizer – Rust
// ---------------------------------------------------------------------------

#[test]
fn tokenize_rust_simple_fn() {
    let code = "fn main() { let x = 42; }";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operators.contains_key("fn"));
    assert!(counts.operators.contains_key("let"));
    assert!(counts.operators.contains_key("="));
    assert!(counts.operands.contains("main"));
    assert!(counts.operands.contains("x"));
    assert!(counts.total_operators > 0);
    assert!(counts.total_operands > 0);
}

#[test]
fn tokenize_rust_if_else() {
    let code = "if a > b { a + b } else { a - b }";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operators.contains_key("if"));
    assert!(counts.operators.contains_key("else"));
    assert!(counts.operators.contains_key(">"));
    assert!(counts.operators.contains_key("+"));
    assert!(counts.operators.contains_key("-"));
}

// ---------------------------------------------------------------------------
// Tokenizer – Python
// ---------------------------------------------------------------------------

#[test]
fn tokenize_python_function() {
    let code = "def greet(name):\n    return name";
    let counts = tokenize_for_halstead(code, "python");
    assert!(counts.operators.contains_key("def"));
    assert!(counts.operators.contains_key("return"));
    assert!(counts.operands.contains("greet"));
    assert!(counts.operands.contains("name"));
}

// ---------------------------------------------------------------------------
// Tokenizer – JavaScript
// ---------------------------------------------------------------------------

#[test]
fn tokenize_js_arrow() {
    let code = "const add = (a, b) => a + b;";
    let counts = tokenize_for_halstead(code, "javascript");
    assert!(counts.operators.contains_key("const"));
    assert!(counts.operators.contains_key("=>"));
    assert!(counts.operators.contains_key("="));
    assert!(counts.operators.contains_key("+"));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_source_yields_zero_counts() {
    let counts = tokenize_for_halstead("", "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
    assert!(counts.operators.is_empty());
    assert!(counts.operands.is_empty());
}

#[test]
fn only_comments_yields_zero() {
    let code = "// this is a comment\n// another comment";
    let counts = tokenize_for_halstead(code, "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn only_blank_lines_yields_zero() {
    let code = "\n\n\n   \n\t\n";
    let counts = tokenize_for_halstead(code, "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn string_literal_counted_as_operand() {
    let code = r#"let s = "hello world";"#;
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operands.contains("<string>"));
    assert!(counts.total_operands >= 1);
}

#[test]
fn unknown_lang_counts_only_operands() {
    let code = "fn main() { let x = 1; }";
    let counts = tokenize_for_halstead(code, "unknown_lang");
    // All words become operands since there are no known operators
    assert_eq!(counts.total_operators, 0);
    assert!(counts.total_operands > 0);
}

// ---------------------------------------------------------------------------
// Halstead formulas
// ---------------------------------------------------------------------------

#[test]
fn volume_computation_manual() {
    // n1=3, n2=4, N1=5, N2=8 => vocab=7, length=13
    // volume = 13 * log2(7) ≈ 36.5
    let vocab: f64 = 7.0;
    let length: f64 = 13.0;
    let volume = length * vocab.log2();
    assert!((volume - 36.5).abs() < 0.5);
}

#[test]
fn difficulty_computation_manual() {
    // n1=4, n2=5, N2=10 => difficulty = (4/2) * (10/5) = 4.0
    let n1 = 4.0_f64;
    let n2 = 5.0_f64;
    let total_opds = 10.0_f64;
    let difficulty = (n1 / 2.0) * (total_opds / n2);
    assert!((difficulty - 4.0).abs() < 1e-10);
}

#[test]
fn effort_is_difficulty_times_volume() {
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let counts = tokenize_for_halstead(code, "rust");
    let n1 = counts.operators.len();
    let n2 = counts.operands.len();
    let vocab = n1 + n2;
    let length = counts.total_operators + counts.total_operands;
    let volume = if vocab > 0 {
        length as f64 * (vocab as f64).log2()
    } else {
        0.0
    };
    let difficulty = if n2 > 0 {
        (n1 as f64 / 2.0) * (counts.total_operands as f64 / n2 as f64)
    } else {
        0.0
    };
    let effort = difficulty * volume;
    assert!(effort >= 0.0);
    assert_eq!(effort, difficulty * volume);
}

#[test]
fn zero_operands_yields_zero_difficulty() {
    // Hypothetical: only operators, no operands => difficulty should be 0
    let n2 = 0usize;
    let difficulty = if n2 > 0 { 1.0 } else { 0.0 };
    assert_eq!(difficulty, 0.0);
}

#[test]
fn zero_vocabulary_yields_zero_volume() {
    let vocab = 0usize;
    let length = 0usize;
    let volume = if vocab > 0 {
        length as f64 * (vocab as f64).log2()
    } else {
        0.0
    };
    assert_eq!(volume, 0.0);
}

// ---------------------------------------------------------------------------
// round_f64
// ---------------------------------------------------------------------------

#[test]
fn round_f64_basic() {
    assert_eq!(round_f64(3.14159, 2), 3.14);
    assert_eq!(round_f64(2.005, 2), 2.01); // banker's rounding in IEEE 754
    assert_eq!(round_f64(0.0, 4), 0.0);
    assert_eq!(round_f64(1.23456789, 4), 1.2346);
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn tokenize_deterministic_across_runs() {
    let code = "fn main() {\n    let x = 1;\n    let y = x + 2;\n    if x > y { return; }\n}";
    let c1 = tokenize_for_halstead(code, "rust");
    let c2 = tokenize_for_halstead(code, "rust");
    assert_eq!(c1.total_operators, c2.total_operators);
    assert_eq!(c1.total_operands, c2.total_operands);
    assert_eq!(c1.operators, c2.operators);
    assert_eq!(c1.operands, c2.operands);
}
