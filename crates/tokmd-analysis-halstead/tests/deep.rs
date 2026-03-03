//! Deep tests for `tokmd-analysis-halstead`.
//!
//! Covers exact operator/operand counts for known snippets, end-to-end
//! metric verification with hand-computed values, cross-language tokenization
//! consistency, mathematical property tests on actual reports, and edge cases.

use std::path::PathBuf;

use tokmd_analysis_halstead::{
    build_halstead_report, is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead,
};
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

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

fn build_report_for_code(
    code: &str,
    lang: &str,
    filename: &str,
) -> tokmd_analysis_types::HalsteadMetrics {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(filename), code).unwrap();
    let export = make_export(vec![make_row(filename, lang)]);
    let files = vec![PathBuf::from(filename)];
    build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap()
}

// ═══════════════════════════════════════════════════════════════════
// § Exact operator/operand counts for known Rust snippets
// ═══════════════════════════════════════════════════════════════════

mod exact_counts {
    use super::*;

    #[test]
    fn single_let_assignment() {
        // `let x = 42;` → operators: {let:1, =:1}, operands: {x, 42}
        let counts = tokenize_for_halstead("let x = 42;", "rust");
        assert_eq!(*counts.operators.get("let").unwrap(), 1);
        assert_eq!(*counts.operators.get("=").unwrap(), 1);
        assert_eq!(counts.operators.len(), 2, "exactly 2 distinct operators");
        assert_eq!(counts.total_operators, 2);
        assert!(counts.operands.contains("x"));
        assert!(counts.operands.contains("42"));
        assert_eq!(counts.operands.len(), 2, "exactly 2 distinct operands");
        assert_eq!(counts.total_operands, 2);
    }

    #[test]
    fn repeated_variable_use() {
        // `let x = x + x;` → operators: {let:1, =:1, +:1}, operands: {x} with total 3
        let counts = tokenize_for_halstead("let x = x + x;", "rust");
        assert_eq!(counts.total_operators, 3); // let, =, +
        assert_eq!(counts.total_operands, 3); // x, x, x
        assert_eq!(counts.operands.len(), 1, "x is the only distinct operand");
    }

    #[test]
    fn if_else_return() {
        // `if x > 0 { return x; } else { return 0; }`
        let counts = tokenize_for_halstead("if x > 0 { return x; } else { return 0; }", "rust");
        assert!(counts.operators.contains_key("if"));
        assert!(counts.operators.contains_key("else"));
        assert!(counts.operators.contains_key("return"));
        assert!(counts.operators.contains_key(">"));
        assert_eq!(
            *counts.operators.get("return").unwrap(),
            2,
            "return appears twice"
        );
        assert!(counts.operands.contains("x"));
        assert!(counts.operands.contains("0"));
    }

    #[test]
    fn for_loop_operators() {
        let counts = tokenize_for_halstead("for i in 0..10 { let x = i + 1; }", "rust");
        assert!(counts.operators.contains_key("for"));
        assert!(counts.operators.contains_key("in"));
        assert!(counts.operators.contains_key("let"));
        assert!(counts.operators.contains_key("="));
        assert!(counts.operators.contains_key("+"));
        assert!(counts.operators.contains_key(".."));
        assert!(counts.operands.contains("i"));
        assert!(counts.operands.contains("x"));
        assert!(counts.operands.contains("0"));
        assert!(counts.operands.contains("10"));
        assert!(counts.operands.contains("1"));
    }

    #[test]
    fn match_expression() {
        let counts = tokenize_for_halstead("match x { 1 => true, _ => false, }", "rust");
        assert!(counts.operators.contains_key("match"));
        assert!(counts.operators.contains_key("=>"));
        assert_eq!(*counts.operators.get("=>").unwrap(), 2, "=> appears twice");
        assert!(counts.operands.contains("x"));
        assert!(counts.operands.contains("1"));
    }

    #[test]
    fn compound_assignment_operators() {
        let code = "x += 1\ny -= 2\nz *= 3\nw /= 4\nm %= 5";
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.operators.contains_key("+="));
        assert!(counts.operators.contains_key("-="));
        assert!(counts.operators.contains_key("*="));
        assert!(counts.operators.contains_key("/="));
        assert!(counts.operators.contains_key("%="));
        assert_eq!(counts.operators.len(), 5);
        assert_eq!(counts.total_operators, 5);
    }

    #[test]
    fn comparison_operators() {
        let code = "a == b\nc != d\ne < f\ng > h\ni <= j\nk >= l";
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.operators.contains_key("=="));
        assert!(counts.operators.contains_key("!="));
        assert!(counts.operators.contains_key("<"));
        assert!(counts.operators.contains_key(">"));
        assert!(counts.operators.contains_key("<="));
        assert!(counts.operators.contains_key(">="));
        assert_eq!(counts.operators.len(), 6);
    }

    #[test]
    fn bitwise_operators() {
        let code = "a & b | c ^ d << 1 >> 2";
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.operators.contains_key("&"));
        assert!(counts.operators.contains_key("|"));
        assert!(counts.operators.contains_key("^"));
        assert!(counts.operators.contains_key("<<"));
        assert!(counts.operators.contains_key(">>"));
    }

    #[test]
    fn question_mark_operator() {
        let code = "let v = x?;";
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.operators.contains_key("?"));
        assert_eq!(*counts.operators.get("?").unwrap(), 1);
    }

    #[test]
    fn range_operators() {
        let code = "let a = 0..10;\nlet b = 0..=9;";
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.operators.contains_key(".."));
        assert!(counts.operators.contains_key("..="));
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Python-specific tokenization
// ═══════════════════════════════════════════════════════════════════

mod python_counts {
    use super::*;

    #[test]
    fn python_walrus_operator() {
        let counts = tokenize_for_halstead("if (n := 10) > 5:", "python");
        assert!(counts.operators.contains_key(":="));
    }

    #[test]
    fn python_floor_division() {
        let counts = tokenize_for_halstead("x = 10 // 3", "python");
        assert!(counts.operators.contains_key("//"));
        assert!(counts.operators.contains_key("="));
    }

    #[test]
    fn python_power_operator() {
        let counts = tokenize_for_halstead("x = 2 ** 10", "python");
        assert!(counts.operators.contains_key("**"));
    }

    #[test]
    fn python_lambda() {
        let counts = tokenize_for_halstead("f = lambda x: x + 1", "python");
        assert!(counts.operators.contains_key("lambda"));
        assert!(counts.operators.contains_key("="));
        assert!(counts.operators.contains_key("+"));
    }

    #[test]
    fn python_comprehension_operators() {
        let counts = tokenize_for_halstead("result = [x for x in range(10) if x > 5]", "python");
        assert!(counts.operators.contains_key("for"));
        assert!(counts.operators.contains_key("in"));
        assert!(counts.operators.contains_key("if"));
        assert!(counts.operators.contains_key(">"));
        assert!(counts.operators.contains_key("="));
    }

    #[test]
    fn python_with_statement() {
        let counts = tokenize_for_halstead("with open('f') as fp:", "python");
        assert!(counts.operators.contains_key("with"));
        assert!(counts.operators.contains_key("as"));
    }
}

// ═══════════════════════════════════════════════════════════════════
// § JavaScript-specific tokenization
// ═══════════════════════════════════════════════════════════════════

mod javascript_counts {
    use super::*;

    #[test]
    fn js_strict_equality() {
        let counts = tokenize_for_halstead("a === b", "javascript");
        assert!(counts.operators.contains_key("==="));
    }

    #[test]
    fn js_strict_inequality() {
        let counts = tokenize_for_halstead("a !== b", "javascript");
        assert!(counts.operators.contains_key("!=="));
    }

    #[test]
    fn js_nullish_coalescing() {
        let counts = tokenize_for_halstead("let x = a ?? b", "javascript");
        assert!(counts.operators.contains_key("??"));
    }

    #[test]
    fn js_optional_chaining() {
        let counts = tokenize_for_halstead("let x = a?.b", "javascript");
        assert!(counts.operators.contains_key("?."));
    }

    #[test]
    fn js_spread_operator() {
        let counts = tokenize_for_halstead("let x = [...a]", "javascript");
        assert!(counts.operators.contains_key("..."));
    }

    #[test]
    fn js_arrow_function() {
        let counts = tokenize_for_halstead("const f = (x) => x + 1", "javascript");
        assert!(counts.operators.contains_key("const"));
        assert!(counts.operators.contains_key("="));
        assert!(counts.operators.contains_key("=>"));
        assert!(counts.operators.contains_key("+"));
    }

    #[test]
    fn js_increment_decrement() {
        let counts = tokenize_for_halstead("x++\ny--", "javascript");
        assert!(counts.operators.contains_key("++"));
        assert!(counts.operators.contains_key("--"));
    }

    #[test]
    fn js_exponent_operator() {
        let counts = tokenize_for_halstead("let x = 2 ** 10", "javascript");
        assert!(counts.operators.contains_key("**"));
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Go-specific tokenization
// ═══════════════════════════════════════════════════════════════════

mod go_counts {
    use super::*;

    #[test]
    fn go_short_var_declaration() {
        let counts = tokenize_for_halstead("x := 42", "go");
        assert!(counts.operators.contains_key(":="));
    }

    #[test]
    fn go_channel_operator() {
        let counts = tokenize_for_halstead("ch <- val", "go");
        assert!(counts.operators.contains_key("<-"));
    }

    #[test]
    fn go_defer_statement() {
        let counts = tokenize_for_halstead("defer close(f)", "go");
        assert!(counts.operators.contains_key("defer"));
    }

    #[test]
    fn go_goroutine() {
        let counts = tokenize_for_halstead("go process(x)", "go");
        assert!(counts.operators.contains_key("go"));
    }

    #[test]
    fn go_bit_clear_operator() {
        let counts = tokenize_for_halstead("x = a &^ b", "go");
        assert!(counts.operators.contains_key("&^"));
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Hand-computed end-to-end metric verification
// ═══════════════════════════════════════════════════════════════════

mod end_to_end_metrics {
    use super::*;

    #[test]
    fn known_snippet_metrics() {
        // Code: `let a = 1\nlet b = a + 2`
        // Operators: let(2), =(2), +(1) → n1=3, N1=5
        // Operands: a(2), 1(1), b(1), 2(1) → n2=4, N2=5
        // Vocabulary = 3 + 4 = 7
        // Length = 5 + 5 = 10
        // Volume = 10 * log2(7) ≈ 10 * 2.8074 ≈ 28.07
        // Difficulty = (3/2) * (5/4) = 1.5 * 1.25 = 1.875
        // Effort = 1.875 * 28.07 ≈ 52.63
        let m = build_report_for_code("let a = 1\nlet b = a + 2", "Rust", "test.rs");

        assert_eq!(m.distinct_operators, 3, "n1 should be 3");
        assert_eq!(m.total_operators, 5, "N1 should be 5");
        assert_eq!(m.distinct_operands, 4, "n2 should be 4");
        assert_eq!(m.total_operands, 5, "N2 should be 5");
        assert_eq!(m.vocabulary, 7);
        assert_eq!(m.length, 10);

        let expected_volume = 10.0 * (7.0_f64).log2();
        assert!(
            (m.volume - round_f64(expected_volume, 2)).abs() < 0.01,
            "volume: expected ~{}, got {}",
            round_f64(expected_volume, 2),
            m.volume
        );

        let expected_difficulty = (3.0 / 2.0) * (5.0 / 4.0);
        assert!(
            (m.difficulty - round_f64(expected_difficulty, 2)).abs() < 0.01,
            "difficulty: expected ~{}, got {}",
            round_f64(expected_difficulty, 2),
            m.difficulty
        );

        let expected_effort = expected_difficulty * expected_volume;
        assert!(
            (m.effort - round_f64(expected_effort, 2)).abs() < 0.1,
            "effort: expected ~{}, got {}",
            round_f64(expected_effort, 2),
            m.effort
        );
    }

    #[test]
    fn single_operator_no_operands() {
        // `return` → n1=1, N1=1, n2=0, N2=0
        // vocabulary=1, length=1, volume=1*log2(1)=0, difficulty=0, effort=0
        let m = build_report_for_code("return", "Rust", "test.rs");
        assert_eq!(m.distinct_operators, 1);
        assert_eq!(m.total_operators, 1);
        assert_eq!(m.distinct_operands, 0);
        assert_eq!(m.total_operands, 0);
        assert_eq!(m.vocabulary, 1);
        assert_eq!(m.length, 1);
        // log2(1) = 0, so volume = 1 * 0 = 0
        assert_eq!(m.volume, 0.0, "volume should be 0 when vocabulary is 1");
        assert_eq!(m.difficulty, 0.0, "difficulty should be 0 with no operands");
        assert_eq!(m.effort, 0.0);
        assert_eq!(m.time_seconds, 0.0);
        assert_eq!(m.estimated_bugs, 0.0);
    }

    #[test]
    fn single_operand_no_operators() {
        // `foo_bar` → n1=0, N1=0, n2=1, N2=1
        // vocabulary=1, length=1, volume=0
        let m = build_report_for_code("foo_bar", "Rust", "test.rs");
        assert_eq!(m.distinct_operators, 0);
        assert_eq!(m.total_operators, 0);
        assert_eq!(m.distinct_operands, 1);
        assert_eq!(m.total_operands, 1);
        assert_eq!(m.vocabulary, 1);
        assert_eq!(m.length, 1);
        assert_eq!(m.volume, 0.0, "volume should be 0 when vocabulary is 1");
    }

    #[test]
    fn empty_file_all_zeros() {
        let m = build_report_for_code("", "Rust", "empty.rs");
        assert_eq!(m.distinct_operators, 0);
        assert_eq!(m.distinct_operands, 0);
        assert_eq!(m.total_operators, 0);
        assert_eq!(m.total_operands, 0);
        assert_eq!(m.vocabulary, 0);
        assert_eq!(m.length, 0);
        assert_eq!(m.volume, 0.0);
        assert_eq!(m.difficulty, 0.0);
        assert_eq!(m.effort, 0.0);
        assert_eq!(m.time_seconds, 0.0);
        assert_eq!(m.estimated_bugs, 0.0);
    }

    #[test]
    fn volume_positive_for_nontrivial_code() {
        let m = build_report_for_code("fn add(a: i32, b: i32) -> i32 { a + b }", "Rust", "f.rs");
        assert!(
            m.volume > 0.0,
            "volume should be positive for code with multiple distinct tokens"
        );
    }

    #[test]
    fn metrics_increase_with_larger_code() {
        let small = build_report_for_code("let x = 1;", "Rust", "small.rs");
        let large_code = "\
fn compute(a: i32, b: i32, c: i32) -> i32 {
    let x = a + b;
    let y = b * c;
    let z = x - y;
    if z > 0 {
        return z;
    } else {
        return x + y + z;
    }
}
";
        let large = build_report_for_code(large_code, "Rust", "large.rs");
        assert!(
            large.length > small.length,
            "larger code should have greater length"
        );
        assert!(
            large.volume > small.volume,
            "larger code should have greater volume"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Mathematical invariants on actual reports
// ═══════════════════════════════════════════════════════════════════

mod math_invariants {
    use super::*;

    fn verify_invariants(m: &tokmd_analysis_types::HalsteadMetrics) {
        // vocabulary = n1 + n2
        assert_eq!(
            m.vocabulary,
            m.distinct_operators + m.distinct_operands,
            "vocabulary should be n1 + n2"
        );
        // length = N1 + N2
        assert_eq!(
            m.length,
            m.total_operators + m.total_operands,
            "length should be N1 + N2"
        );
        // length >= vocabulary (pigeonhole)
        assert!(
            m.length >= m.vocabulary || m.length == 0,
            "length ({}) should be >= vocabulary ({})",
            m.length,
            m.vocabulary
        );
        // volume >= 0
        assert!(m.volume >= 0.0, "volume should be non-negative");
        // difficulty >= 0
        assert!(m.difficulty >= 0.0, "difficulty should be non-negative");
        // effort >= 0
        assert!(m.effort >= 0.0, "effort should be non-negative");
        // time_seconds >= 0
        assert!(m.time_seconds >= 0.0, "time should be non-negative");
        // estimated_bugs >= 0
        assert!(m.estimated_bugs >= 0.0, "bugs should be non-negative");

        // effort = difficulty * volume (within rounding tolerance)
        // Note: compound rounding at each step (D, V, E each rounded) causes drift
        let expected_effort = round_f64(m.difficulty * m.volume, 2);
        assert!(
            (m.effort - expected_effort).abs() < 1.0,
            "effort ({}) should be close to D*V ({})",
            m.effort,
            expected_effort
        );

        // time = effort / 18
        let expected_time = round_f64(m.effort / 18.0, 2);
        assert!(
            (m.time_seconds - expected_time).abs() < 0.1,
            "time ({}) should be close to E/18 ({})",
            m.time_seconds,
            expected_time
        );

        // bugs = volume / 3000
        let expected_bugs = round_f64(m.volume / 3000.0, 4);
        assert!(
            (m.estimated_bugs - expected_bugs).abs() < 0.01,
            "bugs ({}) should be close to V/3000 ({})",
            m.estimated_bugs,
            expected_bugs
        );
    }

    #[test]
    fn rust_invariants() {
        let m = build_report_for_code(
            "fn f(x: i32) -> i32 { if x > 0 { x * 2 } else { x + 1 } }",
            "Rust",
            "f.rs",
        );
        verify_invariants(&m);
    }

    #[test]
    fn python_invariants() {
        let m = build_report_for_code(
            "def f(x):\n    if x > 0:\n        return x * 2\n    return x + 1\n",
            "Python",
            "f.py",
        );
        verify_invariants(&m);
    }

    #[test]
    fn javascript_invariants() {
        let m = build_report_for_code(
            "function f(x) { if (x > 0) { return x * 2; } return x + 1; }",
            "JavaScript",
            "f.js",
        );
        verify_invariants(&m);
    }

    #[test]
    fn go_invariants() {
        let m = build_report_for_code(
            "func f(x int) int {\n    if x > 0 {\n        return x * 2\n    }\n    return x + 1\n}",
            "Go",
            "f.go",
        );
        verify_invariants(&m);
    }

    #[test]
    fn ruby_invariants() {
        let m = build_report_for_code(
            "def f(x)\n  if x > 0\n    return x * 2\n  end\n  x + 1\nend\n",
            "Ruby",
            "f.rb",
        );
        verify_invariants(&m);
    }

    #[test]
    fn empty_invariants() {
        let m = build_report_for_code("", "Rust", "empty.rs");
        verify_invariants(&m);
    }

    #[test]
    fn multi_file_invariants() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn a() { let x = 1 + 2; }").unwrap();
        std::fs::write(
            dir.path().join("b.rs"),
            "fn b(y: i32) { if y > 0 { return y; } }",
        )
        .unwrap();
        std::fs::write(dir.path().join("c.py"), "def c(z):\n    return z * 2\n").unwrap();

        let export = make_export(vec![
            make_row("a.rs", "Rust"),
            make_row("b.rs", "Rust"),
            make_row("c.py", "Python"),
        ]);
        let files = vec![
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            PathBuf::from("c.py"),
        ];
        let m = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();
        verify_invariants(&m);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Edge cases for tokenizer
// ═══════════════════════════════════════════════════════════════════

mod tokenizer_edge_cases {
    use super::*;

    #[test]
    fn numeric_literals_are_operands() {
        let counts = tokenize_for_halstead("let x = 123456789;", "rust");
        assert!(counts.operands.contains("123456789"));
    }

    #[test]
    fn underscore_in_identifiers() {
        let counts = tokenize_for_halstead("let my_var_name = 1;", "rust");
        assert!(counts.operands.contains("my_var_name"));
    }

    #[test]
    fn multiple_string_literals_one_distinct_operand() {
        let code = r#"let a = "hello"; let b = "world";"#;
        let counts = tokenize_for_halstead(code, "rust");
        // Both strings map to <string>
        assert!(counts.operands.contains("<string>"));
        // Total operands includes both string occurrences + a + b
        assert!(counts.total_operands >= 4);
    }

    #[test]
    fn escaped_quote_in_string() {
        let code = r#"let s = "he said \"hi\"";"#;
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.operands.contains("<string>"));
        assert!(counts.operands.contains("s"));
    }

    #[test]
    fn single_quoted_char() {
        let code = "let c = 'x';";
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.operands.contains("<string>"));
    }

    #[test]
    fn block_comment_start_line_skipped() {
        let code = "/* this is a block comment */\nlet x = 1;";
        let counts = tokenize_for_halstead(code, "rust");
        // First line starts with /* so it's skipped
        // Second line: let, =, x, 1
        assert!(counts.operators.contains_key("let"));
        assert!(counts.operators.contains_key("="));
    }

    #[test]
    fn star_continuation_line_skipped() {
        let code = "* this is a continuation\nlet x = 1;";
        let counts = tokenize_for_halstead(code, "rust");
        // First line starts with * so it's skipped
        assert!(counts.operators.contains_key("let"));
    }

    #[test]
    fn line_comment_skipped() {
        let code = "// comment\nlet x = 1;";
        let counts = tokenize_for_halstead(code, "rust");
        // Comment line skipped, second line parsed
        assert_eq!(counts.total_operators, 2);
        assert_eq!(counts.total_operands, 2);
    }

    #[test]
    fn tabs_and_spaces_handled() {
        let code = "\t\tlet\t\tx\t=\t1;";
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.operators.contains_key("let"));
        assert!(counts.operators.contains_key("="));
        assert!(counts.operands.contains("x"));
        assert!(counts.operands.contains("1"));
    }

    #[test]
    fn mixed_operators_on_single_line() {
        let code = "x += y -= z *= w /= v %= u";
        let counts = tokenize_for_halstead(code, "rust");
        assert_eq!(counts.operators.len(), 5);
        assert_eq!(counts.total_operators, 5);
    }

    #[test]
    fn longest_operator_match_precedence() {
        // `<<=` should match as one operator, not `<<` + `=` or `<` + `<=`
        let counts = tokenize_for_halstead("x <<= 1", "rust");
        assert!(
            counts.operators.contains_key("<<="),
            "<<= should be matched as single operator"
        );
        assert_eq!(counts.total_operators, 1);
    }

    #[test]
    fn triple_char_operator() {
        // `>>=` is a 3-char operator in Rust
        let counts = tokenize_for_halstead("x >>= 2", "rust");
        assert!(counts.operators.contains_key(">>="));
        assert_eq!(counts.total_operators, 1);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Cross-language consistency
// ═══════════════════════════════════════════════════════════════════

mod cross_language {
    use super::*;

    #[test]
    fn all_supported_languages_have_common_arithmetic_operators() {
        let langs = ["rust", "javascript", "python", "go", "c", "ruby"];
        for lang in &langs {
            let ops = operators_for_lang(lang);
            assert!(ops.contains(&"+"), "{lang} should have + operator");
            assert!(ops.contains(&"-"), "{lang} should have - operator");
            assert!(ops.contains(&"*"), "{lang} should have * operator");
            assert!(ops.contains(&"/"), "{lang} should have / operator");
            assert!(ops.contains(&"="), "{lang} should have = operator");
        }
    }

    #[test]
    fn all_supported_languages_have_comparison_operators() {
        let langs = ["rust", "javascript", "python", "go", "c", "ruby"];
        for lang in &langs {
            let ops = operators_for_lang(lang);
            assert!(ops.contains(&"=="), "{lang} should have == operator");
            assert!(ops.contains(&"!="), "{lang} should have != operator");
            assert!(ops.contains(&"<"), "{lang} should have < operator");
            assert!(ops.contains(&">"), "{lang} should have > operator");
        }
    }

    #[test]
    fn c_family_all_share_operators() {
        let c_ops = operators_for_lang("c");
        let cpp_ops = operators_for_lang("c++");
        let java_ops = operators_for_lang("java");
        let csharp_ops = operators_for_lang("c#");
        let php_ops = operators_for_lang("php");
        assert_eq!(c_ops, cpp_ops, "C and C++ should share operators");
        assert_eq!(c_ops, java_ops, "C and Java should share operators");
        assert_eq!(c_ops, csharp_ops, "C and C# should share operators");
        assert_eq!(c_ops, php_ops, "C and PHP should share operators");
    }

    #[test]
    fn js_and_ts_share_operators() {
        let js = operators_for_lang("javascript");
        let ts = operators_for_lang("typescript");
        assert_eq!(js, ts, "JavaScript and TypeScript should share operators");
    }

    #[test]
    fn equivalent_code_different_languages() {
        // Same logic in multiple languages should produce similar (not identical) token counts
        let rust_counts = tokenize_for_halstead("if x > 0 { return x + 1; }", "rust");
        let py_counts = tokenize_for_halstead("if x > 0:\n    return x + 1", "python");
        let js_counts = tokenize_for_halstead("if (x > 0) { return x + 1; }", "javascript");

        // All should find operators and operands
        assert!(rust_counts.total_operators > 0);
        assert!(py_counts.total_operators > 0);
        assert!(js_counts.total_operators > 0);
        assert!(rust_counts.total_operands > 0);
        assert!(py_counts.total_operands > 0);
        assert!(js_counts.total_operands > 0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Determinism
// ═══════════════════════════════════════════════════════════════════

mod determinism {
    use super::*;

    #[test]
    fn ten_runs_identical_tokenization() {
        let code = "fn compute(a: i32, b: i32) -> i32 { if a > b { a * 2 } else { b + 1 } }";
        let first = tokenize_for_halstead(code, "rust");
        for _ in 0..9 {
            let run = tokenize_for_halstead(code, "rust");
            assert_eq!(first.total_operators, run.total_operators);
            assert_eq!(first.total_operands, run.total_operands);
            assert_eq!(first.operators, run.operators);
            assert_eq!(first.operands, run.operands);
        }
    }

    #[test]
    fn ten_runs_identical_report() {
        let code = "fn f(x: i32) -> i32 { let y = x + 1; if y > 10 { y * 2 } else { y - 1 } }";
        let first = build_report_for_code(code, "Rust", "f.rs");
        for _ in 0..9 {
            let run = build_report_for_code(code, "Rust", "f.rs");
            assert_eq!(first.distinct_operators, run.distinct_operators);
            assert_eq!(first.distinct_operands, run.distinct_operands);
            assert_eq!(first.total_operators, run.total_operators);
            assert_eq!(first.total_operands, run.total_operands);
            assert_eq!(first.vocabulary, run.vocabulary);
            assert_eq!(first.length, run.length);
            assert!((first.volume - run.volume).abs() < f64::EPSILON);
            assert!((first.difficulty - run.difficulty).abs() < f64::EPSILON);
            assert!((first.effort - run.effort).abs() < f64::EPSILON);
            assert!((first.time_seconds - run.time_seconds).abs() < f64::EPSILON);
            assert!((first.estimated_bugs - run.estimated_bugs).abs() < f64::EPSILON);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § round_f64 deep tests
// ═══════════════════════════════════════════════════════════════════

mod round_deep {
    use super::*;

    #[test]
    fn round_preserves_zero() {
        assert_eq!(round_f64(0.0, 0), 0.0);
        assert_eq!(round_f64(0.0, 5), 0.0);
        assert_eq!(round_f64(0.0, 10), 0.0);
    }

    #[test]
    fn round_negative_values() {
        assert_eq!(round_f64(-1.234, 2), -1.23);
        assert_eq!(round_f64(-1.235, 2), -1.24);
        assert_eq!(round_f64(-0.5, 0), -1.0);
    }

    #[test]
    fn round_very_large_values() {
        let val = 1_000_000.123456;
        let rounded = round_f64(val, 2);
        assert!((rounded - 1_000_000.12).abs() < 0.01);
    }

    #[test]
    fn round_with_many_decimal_places() {
        let val = 1.0 / 3.0; // 0.33333...
        assert_eq!(round_f64(val, 2), 0.33);
        assert_eq!(round_f64(val, 4), 0.3333);
        assert_eq!(round_f64(val, 6), 0.333333);
    }

    #[test]
    fn round_idempotent() {
        let val = 3.14159;
        let once = round_f64(val, 3);
        let twice = round_f64(once, 3);
        assert_eq!(once, twice, "round_f64 should be idempotent");
    }

    #[test]
    fn round_boundary_half_up() {
        // Rust rounds 0.5 away from zero
        assert_eq!(round_f64(0.5, 0), 1.0);
        assert_eq!(round_f64(1.5, 0), 2.0);
        assert_eq!(round_f64(2.5, 0), 3.0);
        assert_eq!(round_f64(0.15, 1), 0.2);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Language support edge cases
// ═══════════════════════════════════════════════════════════════════

mod lang_support {
    use super::*;

    #[test]
    fn is_halstead_lang_all_supported() {
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
    fn is_halstead_lang_unsupported() {
        let unsupported = [
            "", "Markdown", "JSON", "YAML", "TOML", "HTML", "CSS", "Shell", "Bash", "SQL",
            "Haskell", "Erlang", "Elixir", "Kotlin", "Swift", "Scala", "Perl", "Lua",
        ];
        for lang in &unsupported {
            assert!(!is_halstead_lang(lang), "{lang} should NOT be supported");
        }
    }

    #[test]
    fn operators_for_unknown_lang_is_empty() {
        assert!(operators_for_lang("fortran").is_empty());
        assert!(operators_for_lang("cobol").is_empty());
        assert!(operators_for_lang("").is_empty());
    }

    #[test]
    fn operator_tables_have_no_empty_strings() {
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
            let ops = operators_for_lang(lang);
            for op in ops {
                assert!(!op.is_empty(), "{lang} has an empty operator string");
            }
        }
    }
}
