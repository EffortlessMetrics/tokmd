//! Depth tests for Halstead metric computation (W56).

use tokmd_analysis_halstead::{
    FileTokenCounts, is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead,
};

// ───────────────────── is_halstead_lang ─────────────────────

#[test]
fn supported_languages_recognized() {
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
        assert!(is_halstead_lang(lang), "{lang} should be supported");
    }
}

#[test]
fn unsupported_languages_rejected() {
    for lang in &["Haskell", "Lua", "SQL", "Markdown", "", "COBOL"] {
        assert!(!is_halstead_lang(lang), "{lang} should NOT be supported");
    }
}

#[test]
fn lang_check_is_case_insensitive() {
    assert!(is_halstead_lang("rust"));
    assert!(is_halstead_lang("RUST"));
    assert!(is_halstead_lang("Rust"));
}

// ───────────────────── operators_for_lang ─────────────────────

#[test]
fn rust_operators_non_empty() {
    let ops = operators_for_lang("rust");
    assert!(!ops.is_empty());
    assert!(ops.contains(&"fn"));
    assert!(ops.contains(&"if"));
    assert!(ops.contains(&"=>"));
}

#[test]
fn python_operators_include_walrus() {
    let ops = operators_for_lang("python");
    assert!(ops.contains(&":="));
    assert!(ops.contains(&"def"));
}

#[test]
fn go_operators_include_short_assign() {
    let ops = operators_for_lang("go");
    assert!(ops.contains(&":="));
    assert!(ops.contains(&"func"));
    assert!(ops.contains(&"<-"));
}

#[test]
fn unknown_lang_returns_empty_ops() {
    let ops = operators_for_lang("brainfuck");
    assert!(ops.is_empty());
}

#[test]
fn js_and_ts_share_operators() {
    let js = operators_for_lang("javascript");
    let ts = operators_for_lang("typescript");
    assert_eq!(js, ts);
}

#[test]
fn ruby_operators_include_spaceship() {
    let ops = operators_for_lang("ruby");
    assert!(ops.contains(&"<=>"));
    assert!(ops.contains(&"=~"));
}

// ───────────────────── tokenize_for_halstead – basic ─────────────────────

#[test]
fn empty_input_yields_zero_counts() {
    let c = tokenize_for_halstead("", "rust");
    assert_eq!(c.total_operators, 0);
    assert_eq!(c.total_operands, 0);
    assert!(c.operators.is_empty());
    assert!(c.operands.is_empty());
}

#[test]
fn comment_only_input_yields_zero() {
    let code = "// this is a comment\n// another line\n";
    let c = tokenize_for_halstead(code, "rust");
    assert_eq!(c.total_operators, 0);
    assert_eq!(c.total_operands, 0);
}

#[test]
fn blank_lines_ignored() {
    let code = "\n\n\n   \n\t\n";
    let c = tokenize_for_halstead(code, "rust");
    assert_eq!(c.total_operators, 0);
    assert_eq!(c.total_operands, 0);
}

#[test]
fn hash_comment_lines_skipped() {
    let code = "# comment\n# another\n";
    let c = tokenize_for_halstead(code, "python");
    assert_eq!(c.total_operators, 0);
    assert_eq!(c.total_operands, 0);
}

// ───────────────────── tokenize – Rust ─────────────────────

#[test]
fn rust_fn_counts_operators_and_operands() {
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let c = tokenize_for_halstead(code, "rust");
    assert!(c.total_operators > 0, "should find operators");
    assert!(c.total_operands > 0, "should find operands");
    assert!(c.operators.contains_key("fn"));
    assert!(c.operands.contains("add"));
}

#[test]
fn rust_match_keyword_counted() {
    let code = "match x { 1 => true, _ => false }";
    let c = tokenize_for_halstead(code, "rust");
    assert!(c.operators.contains_key("match"));
    assert!(c.operators.contains_key("=>"));
}

#[test]
fn rust_multiline_function() {
    let code = r#"
fn compute(x: i32, y: i32) -> i32 {
    let sum = x + y;
    if sum > 10 {
        sum * 2
    } else {
        sum - 1
    }
}
"#;
    let c = tokenize_for_halstead(code, "rust");
    assert!(c.operators.contains_key("fn"));
    assert!(c.operators.contains_key("let"));
    assert!(c.operators.contains_key("if"));
    assert!(c.operators.contains_key("else"));
    assert!(c.total_operators >= 6);
    assert!(c.total_operands >= 3);
}

// ───────────────────── tokenize – Python ─────────────────────

#[test]
fn python_def_counted_as_operator() {
    let code = "def foo(x, y): return x + y";
    let c = tokenize_for_halstead(code, "python");
    assert!(c.operators.contains_key("def"));
    assert!(c.operators.contains_key("return"));
    assert!(c.operands.contains("foo"));
}

#[test]
fn python_for_loop_complexity() {
    let code = r#"
def process(items):
    for item in items:
        if item > 0:
            yield item * 2
"#;
    let c = tokenize_for_halstead(code, "python");
    assert!(c.operators.contains_key("for"));
    assert!(c.operators.contains_key("in"));
    assert!(c.operators.contains_key("if"));
    assert!(c.operators.contains_key("yield"));
}

// ───────────────────── tokenize – JavaScript ─────────────────────

#[test]
fn js_arrow_function_detected() {
    let code = "const add = (a, b) => a + b;";
    let c = tokenize_for_halstead(code, "javascript");
    assert!(c.operators.contains_key("const"));
    assert!(c.operators.contains_key("=>"));
}

#[test]
fn js_async_await_counted() {
    let code = "async function fetch() { await getData(); }";
    let c = tokenize_for_halstead(code, "javascript");
    assert!(c.operators.contains_key("async"));
    assert!(c.operators.contains_key("await"));
    assert!(c.operators.contains_key("function"));
}

// ───────────────────── tokenize – Go ─────────────────────

#[test]
fn go_func_and_channel() {
    let code = "func send(ch chan int) { ch <- 42 }";
    let c = tokenize_for_halstead(code, "go");
    assert!(c.operators.contains_key("func"));
    assert!(c.operators.contains_key("chan"));
    assert!(c.operators.contains_key("<-"));
}

// ───────────────────── string literals ─────────────────────

#[test]
fn string_literal_counted_as_operand() {
    let code = r#"let msg = "hello world";"#;
    let c = tokenize_for_halstead(code, "rust");
    assert!(c.operands.contains("<string>"));
    assert!(c.total_operands >= 2); // msg + string literal
}

#[test]
fn escaped_string_handled() {
    let code = r#"let msg = "hello \"world\"";"#;
    let c = tokenize_for_halstead(code, "rust");
    assert!(c.operands.contains("<string>"));
}

// ───────────────────── single-operator edge case ─────────────────────

#[test]
fn single_operator_only() {
    let code = "return";
    let c = tokenize_for_halstead(code, "rust");
    assert_eq!(c.total_operators, 1);
    assert_eq!(c.total_operands, 0);
}

#[test]
fn single_operand_only() {
    let code = "x";
    let c = tokenize_for_halstead(code, "rust");
    assert_eq!(c.total_operators, 0);
    assert_eq!(c.total_operands, 1);
}

// ───────────────────── Halstead formula validation ─────────────────────

fn halstead_metrics(c: &FileTokenCounts) -> (f64, f64, f64, f64, f64) {
    let n1 = c.operators.len();
    let n2 = c.operands.len();
    let vocabulary = n1 + n2;
    let length = c.total_operators + c.total_operands;
    let volume = if vocabulary > 0 {
        length as f64 * (vocabulary as f64).log2()
    } else {
        0.0
    };
    let difficulty = if n2 > 0 {
        (n1 as f64 / 2.0) * (c.total_operands as f64 / n2 as f64)
    } else {
        0.0
    };
    let effort = difficulty * volume;
    let time_seconds = effort / 18.0;
    let estimated_bugs = volume / 3000.0;
    (volume, difficulty, effort, time_seconds, estimated_bugs)
}

#[test]
fn volume_zero_for_empty_input() {
    let c = tokenize_for_halstead("", "rust");
    let (volume, _, _, _, _) = halstead_metrics(&c);
    assert_eq!(volume, 0.0);
}

#[test]
fn difficulty_zero_when_no_operands() {
    let code = "return";
    let c = tokenize_for_halstead(code, "rust");
    let (_, difficulty, _, _, _) = halstead_metrics(&c);
    assert_eq!(difficulty, 0.0);
}

#[test]
fn effort_equals_difficulty_times_volume() {
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let c = tokenize_for_halstead(code, "rust");
    let (volume, difficulty, effort, _, _) = halstead_metrics(&c);
    let expected = difficulty * volume;
    assert!((effort - expected).abs() < 1e-10);
}

#[test]
fn time_equals_effort_div_18() {
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let c = tokenize_for_halstead(code, "rust");
    let (_, _, effort, time, _) = halstead_metrics(&c);
    let expected = effort / 18.0;
    assert!((time - expected).abs() < 1e-10);
}

#[test]
fn bugs_equals_volume_div_3000() {
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let c = tokenize_for_halstead(code, "rust");
    let (volume, _, _, _, bugs) = halstead_metrics(&c);
    let expected = volume / 3000.0;
    assert!((bugs - expected).abs() < 1e-10);
}

// ───────────────────── round_f64 ─────────────────────

#[test]
fn round_zero_decimals() {
    assert_eq!(round_f64(3.456, 0), 3.0);
}

#[test]
fn round_two_decimals() {
    assert_eq!(round_f64(3.456, 2), 3.46);
}

#[test]
fn round_four_decimals() {
    assert_eq!(round_f64(0.12345, 4), 0.1235);
}

#[test]
fn round_negative_value() {
    assert_eq!(round_f64(-2.555, 2), -2.56);
}

// ───────────────────── determinism ─────────────────────

#[test]
fn tokenize_is_deterministic() {
    let code = r#"
fn main() {
    let x = 1 + 2;
    if x > 2 { println!("big"); }
}
"#;
    let a = tokenize_for_halstead(code, "rust");
    let b = tokenize_for_halstead(code, "rust");
    assert_eq!(a.total_operators, b.total_operators);
    assert_eq!(a.total_operands, b.total_operands);
    assert_eq!(a.operators, b.operators);
    assert_eq!(a.operands, b.operands);
}

#[test]
fn deterministic_across_repeated_calls() {
    let code = "const f = (a, b) => a + b;";
    let results: Vec<_> = (0..5)
        .map(|_| {
            let c = tokenize_for_halstead(code, "javascript");
            (c.total_operators, c.total_operands)
        })
        .collect();
    let first = results[0];
    for r in &results {
        assert_eq!(*r, first);
    }
}

// ───────────────────── massive input ─────────────────────

#[test]
fn large_input_does_not_panic() {
    let line = "let x = y + z * w - q / r;\n";
    let code: String = line.repeat(10_000);
    let c = tokenize_for_halstead(&code, "rust");
    assert!(c.total_operators > 0);
    assert!(c.total_operands > 0);
}

// ───────────────────── unsupported lang ─────────────────────

#[test]
fn unsupported_lang_yields_only_operands() {
    let code = "foo bar baz + qux";
    let c = tokenize_for_halstead(code, "brainfuck");
    // No operators defined, so everything identifier-like is an operand
    assert_eq!(c.total_operators, 0);
    assert!(c.total_operands > 0);
}
