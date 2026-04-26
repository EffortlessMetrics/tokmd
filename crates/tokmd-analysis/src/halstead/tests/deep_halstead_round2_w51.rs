//! Deep round-2 tests for `analysis Halstead module` (w51).
//!
//! Focuses on Halstead metrics with known operator/operand counts,
//! volume/difficulty/effort calculations, mathematical relationships,
//! and edge cases (0 operators, 1 operand, multi-language).

use std::path::PathBuf;

use crate::halstead::{is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead};
use tokmd_analysis_types::AnalysisLimits;
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
    crate::halstead::build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap()
}

// ═══════════════════════════════════════════════════════════════════
// § Known operator/operand counts
// ═══════════════════════════════════════════════════════════════════

mod known_counts {
    use super::*;

    #[test]
    fn single_let_binding() {
        let counts = tokenize_for_halstead("let x = 42;", "rust");
        // operators: "let", "=" → 2 distinct, 2 total
        assert_eq!(counts.operators.len(), 2);
        assert_eq!(counts.total_operators, 2);
        // operands: "x", "42" → 2 distinct
        assert!(counts.operands.contains("x"));
        assert!(counts.operands.contains("42"));
    }

    #[test]
    fn repeated_operator_counted_correctly() {
        let code = "let a = 1;\nlet b = 2;\nlet c = 3;";
        let counts = tokenize_for_halstead(code, "rust");
        // "let" appears 3 times, "=" appears 3 times
        assert_eq!(*counts.operators.get("let").unwrap(), 3);
        assert_eq!(*counts.operators.get("=").unwrap(), 3);
        assert_eq!(counts.total_operators, 6);
    }

    #[test]
    fn python_operators_detected() {
        let code = "def add(a, b):\n    return a + b";
        let counts = tokenize_for_halstead(code, "python");
        assert!(counts.operators.contains_key("def"));
        assert!(counts.operators.contains_key("return"));
        assert!(counts.operators.contains_key("+"));
        assert!(counts.total_operators >= 3);
    }

    #[test]
    fn javascript_arrow_function() {
        let code = "const add = (a, b) => a + b;";
        let counts = tokenize_for_halstead(code, "javascript");
        assert!(counts.operators.contains_key("=>"));
        assert!(counts.operators.contains_key("const"));
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Volume, difficulty, effort calculations
// ═══════════════════════════════════════════════════════════════════

mod calculations {
    use super::*;

    #[test]
    fn volume_equals_length_times_log2_vocabulary() {
        let m = build_report_for_code("let x = 42;", "Rust", "a.rs");
        let expected_volume = m.length as f64 * (m.vocabulary as f64).log2();
        assert!(
            (m.volume - round_f64(expected_volume, 2)).abs() < 0.1,
            "volume = length * log2(vocab): expected {expected_volume:.2}, got {}",
            m.volume
        );
    }

    #[test]
    fn difficulty_formula_verified() {
        let m = build_report_for_code("let a = 1;\nlet b = 2;\nlet c = a + b;", "Rust", "d.rs");
        if m.distinct_operands > 0 {
            let expected = (m.distinct_operators as f64 / 2.0)
                * (m.total_operands as f64 / m.distinct_operands as f64);
            assert!(
                (m.difficulty - round_f64(expected, 2)).abs() < 0.1,
                "difficulty = (n1/2)*(N2/n2): expected {expected:.2}, got {}",
                m.difficulty
            );
        }
    }

    #[test]
    fn effort_equals_difficulty_times_volume() {
        let m = build_report_for_code("fn add(a: i32, b: i32) -> i32 { a + b }", "Rust", "e.rs");
        let expected = round_f64(m.difficulty * m.volume, 2);
        assert!(
            (m.effort - expected).abs() < 1.0,
            "effort = D * V: expected {expected:.2}, got {}",
            m.effort
        );
    }

    #[test]
    fn time_equals_effort_over_18() {
        let m = build_report_for_code(
            "fn f(x: i32) -> i32 { if x > 0 { x } else { -x } }",
            "Rust",
            "t.rs",
        );
        let expected = round_f64(m.effort / 18.0, 2);
        assert!(
            (m.time_seconds - expected).abs() < 0.1,
            "time = effort / 18: expected {expected:.2}, got {}",
            m.time_seconds
        );
    }

    #[test]
    fn bugs_equals_volume_over_3000() {
        let m = build_report_for_code(
            "fn f(x: i32) -> i32 { if x > 0 { x } else { -x } }",
            "Rust",
            "b.rs",
        );
        let expected = round_f64(m.volume / 3000.0, 2);
        assert!(
            (m.estimated_bugs - expected).abs() < 0.01,
            "bugs = volume / 3000: expected {expected:.4}, got {}",
            m.estimated_bugs
        );
    }

    #[test]
    fn vocabulary_equals_distinct_sum() {
        let m = build_report_for_code("let x = 1 + 2;", "Rust", "v.rs");
        assert_eq!(
            m.vocabulary,
            m.distinct_operators + m.distinct_operands,
            "vocabulary = n1 + n2"
        );
    }

    #[test]
    fn length_equals_total_sum() {
        let m = build_report_for_code("let x = 1 + 2;", "Rust", "l.rs");
        assert_eq!(
            m.length,
            m.total_operators + m.total_operands,
            "length = N1 + N2"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Edge cases
// ═══════════════════════════════════════════════════════════════════

mod edge_cases {
    use super::*;

    #[test]
    fn only_operators_no_operands() {
        // Keywords only, no identifiers or literals
        let counts = tokenize_for_halstead("return", "rust");
        assert!(counts.operators.contains_key("return"));
        // With zero distinct operands, difficulty should be 0
        if counts.operands.is_empty() {
            let m = build_report_for_code("return", "Rust", "ret.rs");
            assert_eq!(m.difficulty, 0.0, "zero operands → difficulty = 0");
        }
    }

    #[test]
    fn single_operand_no_operators_unknown_lang() {
        // Unknown language → no operator table → everything is operand
        let counts = tokenize_for_halstead("hello", "brainfuck");
        assert_eq!(counts.total_operators, 0);
        assert!(counts.operands.contains("hello"));
    }

    #[test]
    fn comments_excluded_from_counts() {
        let code = "// this is a comment with let and fn keywords\nlet x = 1;";
        let counts = tokenize_for_halstead(code, "rust");
        // "let" should appear once (from second line), not from comment
        assert_eq!(*counts.operators.get("let").unwrap(), 1);
    }

    #[test]
    fn multi_file_aggregation() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "let x = 1;").unwrap();
        std::fs::write(dir.path().join("b.rs"), "let y = 2;").unwrap();

        let rows = vec![make_row("a.rs", "Rust"), make_row("b.rs", "Rust")];
        let export = make_export(rows);
        let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];
        let m = crate::halstead::build_halstead_report(dir.path(), &files, &export, &no_limits())
            .unwrap();

        // Aggregated: "let" appears 2 times across files
        assert!(m.total_operators >= 4, "aggregated operators from 2 files");
        assert!(m.total_operands >= 4, "aggregated operands from 2 files");
    }

    #[test]
    fn string_literal_escaped_quotes() {
        let code = r#"let s = "hello \"world\"";"#;
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.operands.contains("<string>"));
    }

    #[test]
    fn larger_code_has_more_length() {
        let small = build_report_for_code("let x = 1;", "Rust", "s.rs");
        let big = build_report_for_code(
            "fn f(a: i32, b: i32) -> i32 {\n    let c = a + b;\n    let d = c * 2;\n    d\n}\n",
            "Rust",
            "b.rs",
        );
        assert!(
            big.length > small.length,
            "more code → greater length: {} vs {}",
            big.length,
            small.length
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Language coverage
// ═══════════════════════════════════════════════════════════════════

mod lang_coverage {
    use super::*;

    #[test]
    fn all_supported_langs_have_operators() {
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
            assert!(is_halstead_lang(lang), "{lang} should be supported");
            let ops = operators_for_lang(lang);
            assert!(!ops.is_empty(), "{lang} should have operators");
        }
    }

    #[test]
    fn case_insensitive_lang_detection() {
        assert!(is_halstead_lang("Rust"));
        assert!(is_halstead_lang("RUST"));
        assert!(is_halstead_lang("rust"));
        assert!(is_halstead_lang("Python"));
        assert!(is_halstead_lang("PYTHON"));
    }
}
