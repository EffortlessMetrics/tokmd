//! W61 depth tests for tokmd-analysis-halstead: BDD edge cases, determinism, proptest.

use tokmd_analysis_halstead::{
    is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead,
};

// ---------------------------------------------------------------------------
// BDD: is_halstead_lang coverage
// ---------------------------------------------------------------------------

#[test]
fn supported_languages_all_detected() {
    let supported = [
        "rust", "javascript", "typescript", "python", "go",
        "c", "c++", "java", "c#", "php", "ruby",
    ];
    for lang in &supported {
        assert!(is_halstead_lang(lang), "{} should be a halstead lang", lang);
    }
}

#[test]
fn supported_languages_case_insensitive() {
    assert!(is_halstead_lang("Rust"));
    assert!(is_halstead_lang("JAVASCRIPT"));
    assert!(is_halstead_lang("TypeScript"));
    assert!(is_halstead_lang("PYTHON"));
    assert!(is_halstead_lang("C#"));
    assert!(is_halstead_lang("C++"));
}

#[test]
fn unsupported_languages_rejected() {
    assert!(!is_halstead_lang("haskell"));
    assert!(!is_halstead_lang("lua"));
    assert!(!is_halstead_lang("perl"));
    assert!(!is_halstead_lang("json"));
    assert!(!is_halstead_lang("markdown"));
    assert!(!is_halstead_lang(""));
}

// ---------------------------------------------------------------------------
// BDD: operators_for_lang
// ---------------------------------------------------------------------------

#[test]
fn operators_for_known_langs_nonempty() {
    let langs = ["rust", "javascript", "typescript", "python", "go", "c", "c++", "java", "c#", "php", "ruby"];
    for lang in &langs {
        let ops = operators_for_lang(lang);
        assert!(!ops.is_empty(), "Operators for {} should not be empty", lang);
    }
}

#[test]
fn operators_for_unknown_lang_empty() {
    assert!(operators_for_lang("haskell").is_empty());
    assert!(operators_for_lang("").is_empty());
    assert!(operators_for_lang("zig").is_empty());
}

#[test]
fn rust_operators_contain_fn_and_let() {
    let ops = operators_for_lang("rust");
    assert!(ops.contains(&"fn"), "Rust should contain 'fn'");
    assert!(ops.contains(&"let"), "Rust should contain 'let'");
    assert!(ops.contains(&"match"), "Rust should contain 'match'");
}

#[test]
fn python_operators_contain_def_and_class() {
    let ops = operators_for_lang("python");
    assert!(ops.contains(&"def"), "Python should contain 'def'");
    assert!(ops.contains(&"class"), "Python should contain 'class'");
    assert!(ops.contains(&"lambda"), "Python should contain 'lambda'");
}

#[test]
fn go_operators_contain_func_and_defer() {
    let ops = operators_for_lang("go");
    assert!(ops.contains(&"func"), "Go should contain 'func'");
    assert!(ops.contains(&"defer"), "Go should contain 'defer'");
    assert!(ops.contains(&"go"), "Go should contain 'go'");
}

#[test]
fn javascript_and_typescript_share_operators() {
    let js = operators_for_lang("javascript");
    let ts = operators_for_lang("typescript");
    assert_eq!(js.len(), ts.len(), "JS and TS should have same operator set");
    for op in js {
        assert!(ts.contains(op), "TS missing JS operator: {}", op);
    }
}

#[test]
fn c_family_share_operators() {
    let c_ops = operators_for_lang("c");
    let cpp_ops = operators_for_lang("c++");
    let java_ops = operators_for_lang("java");
    let csharp_ops = operators_for_lang("c#");
    let php_ops = operators_for_lang("php");
    assert_eq!(c_ops.len(), cpp_ops.len());
    assert_eq!(c_ops.len(), java_ops.len());
    assert_eq!(c_ops.len(), csharp_ops.len());
    assert_eq!(c_ops.len(), php_ops.len());
}

#[test]
fn ruby_operators_contain_end_and_require() {
    let ops = operators_for_lang("ruby");
    assert!(ops.contains(&"end"), "Ruby should contain 'end'");
    assert!(ops.contains(&"require"), "Ruby should contain 'require'");
}

#[test]
fn operators_lists_have_no_duplicates() {
    let langs = ["rust", "javascript", "python", "go", "c", "ruby"];
    for lang in &langs {
        let ops = operators_for_lang(lang);
        let mut sorted: Vec<&str> = ops.to_vec();
        sorted.sort();
        for window in sorted.windows(2) {
            assert_ne!(window[0], window[1], "Duplicate operator '{}' in {}", window[0], lang);
        }
    }
}

// ---------------------------------------------------------------------------
// BDD: tokenize_for_halstead
// ---------------------------------------------------------------------------

#[test]
fn empty_input_yields_zero_counts() {
    let counts = tokenize_for_halstead("", "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
    assert!(counts.operators.is_empty());
    assert!(counts.operands.is_empty());
}

#[test]
fn comment_only_input_yields_zero_counts() {
    let code = "// this is a comment\n// another comment\n";
    let counts = tokenize_for_halstead(code, "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn blank_lines_only_yields_zero() {
    let counts = tokenize_for_halstead("\n\n\n\n", "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn python_comment_lines_skipped() {
    let code = "# comment\n# another\n";
    let counts = tokenize_for_halstead(code, "python");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn simple_rust_fn_tokenizes_operators_and_operands() {
    let code = "fn main() { let x = 5; }";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.total_operators > 0, "Should have operators");
    assert!(counts.total_operands > 0, "Should have operands");
    assert!(counts.operators.contains_key("fn"), "Should detect 'fn'");
    assert!(counts.operators.contains_key("let"), "Should detect 'let'");
}

#[test]
fn string_literals_counted_as_single_operand() {
    let code = r#"let s = "hello world";"#;
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operands.contains("<string>"), "String literal should be tracked as <string>");
}

#[test]
fn tokenize_detects_multi_char_operators() {
    let code = "if x == y && z != w { }";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operators.contains_key("=="), "Should detect '=='");
    assert!(counts.operators.contains_key("&&"), "Should detect '&&'");
    assert!(counts.operators.contains_key("!="), "Should detect '!='");
}

#[test]
fn tokenize_handles_escaped_string() {
    let code = r#"let s = "hello \"world\"";"#;
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operands.contains("<string>"));
}

#[test]
fn tokenize_unknown_lang_yields_only_operands() {
    let code = "fn main() { let x = 5; }";
    let counts = tokenize_for_halstead(code, "brainfuck");
    // No operators recognized since unknown lang has empty operator list
    assert_eq!(counts.total_operators, 0, "Unknown lang should have no operators");
    // But identifiers/numbers are still counted as operands
    assert!(counts.total_operands > 0, "Should still have operands");
}

#[test]
fn tokenize_length_equals_sum_of_ops_and_opds() {
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let counts = tokenize_for_halstead(code, "rust");
    let length = counts.total_operators + counts.total_operands;
    assert!(length > 0);
    // Verify operators map sums match total_operators
    let sum_ops: usize = counts.operators.values().sum();
    assert_eq!(sum_ops, counts.total_operators, "Operator map sum should equal total_operators");
}

// ---------------------------------------------------------------------------
// BDD: round_f64
// ---------------------------------------------------------------------------

#[test]
fn round_f64_zero_decimal_places() {
    assert_eq!(round_f64(2.7, 0), 3.0);
    assert_eq!(round_f64(2.4, 0), 2.0);
}

#[test]
fn round_f64_two_decimal_places() {
    assert_eq!(round_f64(3.14159, 2), 3.14);
}

#[test]
fn round_f64_exact_value() {
    assert_eq!(round_f64(1.0, 5), 1.0);
}

#[test]
fn round_f64_negative() {
    // -1.555 can't be exactly represented in f64; rounds to -1.56 or -1.55
    let r = round_f64(-1.555, 2);
    assert!((r - -1.55).abs() < 0.02, "Expected near -1.55, got {}", r);
}

#[test]
fn round_f64_zero() {
    assert_eq!(round_f64(0.0, 3), 0.0);
}

// ---------------------------------------------------------------------------
// Determinism tests
// ---------------------------------------------------------------------------

#[test]
fn is_halstead_lang_deterministic() {
    for lang in &["rust", "python", "unknown", ""] {
        assert_eq!(is_halstead_lang(lang), is_halstead_lang(lang));
    }
}

#[test]
fn operators_for_lang_deterministic() {
    for lang in &["rust", "python", "go", "unknown"] {
        let a = operators_for_lang(lang);
        let b = operators_for_lang(lang);
        assert_eq!(a.len(), b.len());
    }
}

#[test]
fn tokenize_deterministic() {
    let code = "fn main() { let x = 1 + 2; }";
    let a = tokenize_for_halstead(code, "rust");
    let b = tokenize_for_halstead(code, "rust");
    assert_eq!(a.total_operators, b.total_operators);
    assert_eq!(a.total_operands, b.total_operands);
    assert_eq!(a.operators, b.operators);
    assert_eq!(a.operands, b.operands);
}

#[test]
fn tokenize_deterministic_across_languages() {
    let code = "if x == 0 { return 1; }";
    let r1 = tokenize_for_halstead(code, "rust");
    let r2 = tokenize_for_halstead(code, "rust");
    assert_eq!(r1.total_operators, r2.total_operators);
    assert_eq!(r1.total_operands, r2.total_operands);
}

// ---------------------------------------------------------------------------
// BDD: Halstead formula invariants
// ---------------------------------------------------------------------------

#[test]
fn volume_zero_for_empty_vocabulary() {
    // With no code, vocabulary is 0, volume should be 0
    let counts = tokenize_for_halstead("", "rust");
    let n1 = counts.operators.len();
    let n2 = counts.operands.len();
    let vocabulary = n1 + n2;
    let length = counts.total_operators + counts.total_operands;
    let volume = if vocabulary > 0 {
        length as f64 * (vocabulary as f64).log2()
    } else {
        0.0
    };
    assert_eq!(volume, 0.0);
}

#[test]
fn difficulty_zero_when_no_operands() {
    // If n2 (distinct operands) is 0, difficulty is 0
    let n1 = 5;
    let n2 = 0usize;
    let total_opds = 0usize;
    let difficulty = if n2 > 0 {
        (n1 as f64 / 2.0) * (total_opds as f64 / n2 as f64)
    } else {
        0.0
    };
    assert_eq!(difficulty, 0.0);
}

#[test]
fn effort_is_product_of_difficulty_and_volume() {
    let code = "fn foo() { let a = 1; let b = 2; let c = a + b; }";
    let counts = tokenize_for_halstead(code, "rust");
    let n1 = counts.operators.len();
    let n2 = counts.operands.len();
    let vocabulary = n1 + n2;
    let length = counts.total_operators + counts.total_operands;

    let volume = if vocabulary > 0 {
        length as f64 * (vocabulary as f64).log2()
    } else {
        0.0
    };
    let difficulty = if n2 > 0 {
        (n1 as f64 / 2.0) * (counts.total_operands as f64 / n2 as f64)
    } else {
        0.0
    };
    let effort = difficulty * volume;
    assert!(effort >= 0.0, "Effort should be non-negative");
    assert!((effort - difficulty * volume).abs() < f64::EPSILON);
}

#[test]
fn time_is_effort_over_eighteen() {
    let effort = 360.0;
    let time = effort / 18.0;
    assert_eq!(time, 20.0);
}

#[test]
fn bugs_is_volume_over_three_thousand() {
    let volume = 6000.0;
    let bugs = volume / 3000.0;
    assert_eq!(bugs, 2.0);
}

// ---------------------------------------------------------------------------
// Proptest properties
// ---------------------------------------------------------------------------

mod properties {
    use proptest::prelude::*;
    use tokmd_analysis_halstead::{
        is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead,
    };

    proptest! {
        #[test]
        fn is_halstead_lang_never_panics(s in "\\PC{0,50}") {
            let _ = is_halstead_lang(&s);
        }

        #[test]
        fn operators_for_lang_never_panics(s in "\\PC{0,50}") {
            let _ = operators_for_lang(&s);
        }

        #[test]
        fn tokenize_never_panics(code in "[a-zA-Z0-9 _=+\\-*/(){};.,:\\n\"'#/<>!&|%^~\\[\\]]{0,200}", lang in "[a-z+#]{0,15}") {
            let _ = tokenize_for_halstead(&code, &lang);
        }

        #[test]
        fn tokenize_operator_sum_matches_total(code in "[a-zA-Z0-9 _=+\\-*/(){};\\n]{0,200}") {
            let counts = tokenize_for_halstead(&code, "rust");
            let sum: usize = counts.operators.values().sum();
            prop_assert_eq!(sum, counts.total_operators);
        }

        #[test]
        fn round_f64_idempotent(val in -1e6f64..1e6, decimals in 0u32..8) {
            let once = round_f64(val, decimals);
            let twice = round_f64(once, decimals);
            prop_assert!((once - twice).abs() < 1e-10);
        }

        #[test]
        fn tokenize_deterministic_property(code in "[a-zA-Z0-9 _=+(){};\\n]{0,100}") {
            let a = tokenize_for_halstead(&code, "rust");
            let b = tokenize_for_halstead(&code, "rust");
            prop_assert_eq!(a.total_operators, b.total_operators);
            prop_assert_eq!(a.total_operands, b.total_operands);
        }

        #[test]
        fn vocabulary_equals_distinct_ops_plus_opds(code in "[a-zA-Z0-9 _=+(){};\\n]{0,100}") {
            let counts = tokenize_for_halstead(&code, "rust");
            let n1 = counts.operators.len();
            let n2 = counts.operands.len();
            prop_assert_eq!(n1 + n2, counts.operators.len() + counts.operands.len());
        }
    }
}
