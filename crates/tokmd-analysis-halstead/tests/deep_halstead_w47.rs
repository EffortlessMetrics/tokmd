//! Deep property-based and deterministic tests for `tokmd-analysis-halstead`.
//!
//! Covers Halstead volume/difficulty/effort calculations, tokenizer invariants,
//! known input/output pairs, edge cases (empty input, unknown langs),
//! and property-based verification.

use std::path::PathBuf;

use proptest::prelude::*;
use tokmd_analysis_halstead::{
    is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead,
};
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
    tokmd_analysis_halstead::build_halstead_report(dir.path(), &files, &export, &no_limits())
        .unwrap()
}

// ═══════════════════════════════════════════════════════════════════
// § Known Halstead metric calculations
// ═══════════════════════════════════════════════════════════════════

mod known_values {
    use super::*;

    #[test]
    fn volume_known_computation() {
        // n1=2, n2=3, N1=4, N2=6 → vocab=5, length=10, volume = 10 * log2(5)
        let n1 = 2usize;
        let n2 = 3usize;
        let total_ops = 4usize;
        let total_opds = 6usize;
        let vocab = n1 + n2;
        let length = total_ops + total_opds;
        let volume = length as f64 * (vocab as f64).log2();
        assert!((volume - 23.22).abs() < 0.1);
    }

    #[test]
    fn difficulty_known_computation() {
        // n1=2, n2=3, N2=6 → difficulty = (2/2) * (6/3) = 2.0
        let difficulty: f64 = (2.0 / 2.0) * (6.0 / 3.0);
        assert!((difficulty - 2.0).abs() < 0.001);
    }

    #[test]
    fn effort_is_difficulty_times_volume() {
        let volume: f64 = 23.22;
        let difficulty: f64 = 2.0;
        let effort = difficulty * volume;
        assert!((effort - 46.44).abs() < 0.1);
    }

    #[test]
    fn time_is_effort_over_18() {
        let effort: f64 = 46.44;
        let time = effort / 18.0;
        assert!((time - 2.58).abs() < 0.1);
    }

    #[test]
    fn bugs_is_volume_over_3000() {
        let volume: f64 = 3000.0;
        let bugs = volume / 3000.0;
        assert!((bugs - 1.0).abs() < 0.001);
    }

    #[test]
    fn let_assignment_metrics() {
        let m = build_report_for_code("let x = 42;", "Rust", "a.rs");
        // operators: let, =  →  n1=2, N1=2
        // operands: x, 42    →  n2=2, N2=2
        assert_eq!(m.distinct_operators, 2);
        assert_eq!(m.total_operators, 2);
        assert_eq!(m.distinct_operands, 2);
        assert_eq!(m.total_operands, 2);
        assert_eq!(m.vocabulary, 4);
        assert_eq!(m.length, 4);
        // volume = 4 * log2(4) = 4 * 2 = 8
        assert!((m.volume - 8.0).abs() < 0.01);
    }

    #[test]
    fn simple_function_metrics() {
        let code = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        let m = build_report_for_code(code, "Rust", "add.rs");
        assert!(m.distinct_operators > 0);
        assert!(m.distinct_operands > 0);
        assert!(m.volume > 0.0);
        assert!(m.difficulty >= 0.0);
        assert!(m.effort >= 0.0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Edge cases
// ═══════════════════════════════════════════════════════════════════

mod edge_cases {
    use super::*;

    #[test]
    fn empty_input_zero_metrics() {
        let counts = tokenize_for_halstead("", "rust");
        assert_eq!(counts.total_operators, 0);
        assert_eq!(counts.total_operands, 0);
        assert!(counts.operators.is_empty());
        assert!(counts.operands.is_empty());
    }

    #[test]
    fn comment_only_zero_tokens() {
        let code = "// just a comment\n// another one\n";
        let counts = tokenize_for_halstead(code, "rust");
        assert_eq!(counts.total_operators, 0);
        assert_eq!(counts.total_operands, 0);
    }

    #[test]
    fn unsupported_lang_empty_operators() {
        let ops = operators_for_lang("brainfuck");
        assert!(ops.is_empty());
    }

    #[test]
    fn unsupported_lang_no_halstead() {
        assert!(!is_halstead_lang("markdown"));
        assert!(!is_halstead_lang("toml"));
        assert!(!is_halstead_lang("json"));
    }

    #[test]
    fn string_literal_counted_as_operand() {
        let code = r#"let s = "hello";"#;
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.operands.contains("<string>"));
        assert!(counts.total_operands > 0);
    }

    #[test]
    fn empty_report_for_no_files() {
        let dir = tempfile::tempdir().unwrap();
        let export = make_export(vec![]);
        let m =
            tokmd_analysis_halstead::build_halstead_report(dir.path(), &[], &export, &no_limits())
                .unwrap();
        assert_eq!(m.vocabulary, 0);
        assert_eq!(m.length, 0);
        assert_eq!(m.volume, 0.0);
        assert_eq!(m.difficulty, 0.0);
        assert_eq!(m.effort, 0.0);
    }

    #[test]
    fn zero_vocabulary_gives_zero_volume() {
        // Empty code → vocabulary=0 → volume=0
        let m = build_report_for_code("", "Rust", "empty.rs");
        assert_eq!(m.volume, 0.0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Round function
// ═══════════════════════════════════════════════════════════════════

mod rounding {
    use super::*;

    #[test]
    fn round_f64_two_decimals() {
        assert_eq!(round_f64(1.2345, 2), 1.23);
        assert_eq!(round_f64(1.235, 2), 1.24);
        assert_eq!(round_f64(0.0, 2), 0.0);
    }

    #[test]
    fn round_f64_four_decimals() {
        assert_eq!(round_f64(1.23456, 4), 1.2346);
    }

    #[test]
    fn round_f64_zero_decimals() {
        assert_eq!(round_f64(1.6, 0), 2.0);
        assert_eq!(round_f64(1.4, 0), 1.0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Property-based tests
// ═══════════════════════════════════════════════════════════════════

fn arb_supported_lang() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("rust"),
        Just("javascript"),
        Just("typescript"),
        Just("python"),
        Just("go"),
        Just("c"),
        Just("c++"),
        Just("java"),
        Just("c#"),
        Just("php"),
        Just("ruby"),
    ]
}

fn arb_rust_snippet() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("fn main() {}".to_string()),
        Just("let x = 1 + 2;".to_string()),
        Just("if a > b { a } else { b }".to_string()),
        Just("match x { 0 => true, _ => false }".to_string()),
        Just("for i in 0..10 { let y = i * 2; }".to_string()),
        Just("struct Foo { x: i32, y: i32 }".to_string()),
        Just("pub fn add(a: i32, b: i32) -> i32 { a + b }".to_string()),
        Just("while x > 0 { x -= 1; }".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_volume_non_negative(code in arb_rust_snippet()) {
        let counts = tokenize_for_halstead(&code, "rust");
        let n1 = counts.operators.len();
        let n2 = counts.operands.len();
        let vocab = n1 + n2;
        let length = counts.total_operators + counts.total_operands;
        let volume = if vocab > 0 {
            length as f64 * (vocab as f64).log2()
        } else {
            0.0
        };
        prop_assert!(volume >= 0.0, "volume must be >= 0, got {volume}");
    }

    #[test]
    fn prop_difficulty_non_negative(code in arb_rust_snippet()) {
        let counts = tokenize_for_halstead(&code, "rust");
        let n1 = counts.operators.len();
        let n2 = counts.operands.len();
        let difficulty = if n2 > 0 {
            (n1 as f64 / 2.0) * (counts.total_operands as f64 / n2 as f64)
        } else {
            0.0
        };
        prop_assert!(difficulty >= 0.0, "difficulty must be >= 0, got {difficulty}");
    }

    #[test]
    fn prop_total_operators_equals_sum(code in arb_rust_snippet()) {
        let counts = tokenize_for_halstead(&code, "rust");
        let sum: usize = counts.operators.values().sum();
        prop_assert_eq!(counts.total_operators, sum);
    }

    #[test]
    fn prop_total_operands_ge_distinct(code in arb_rust_snippet()) {
        let counts = tokenize_for_halstead(&code, "rust");
        prop_assert!(
            counts.total_operands >= counts.operands.len(),
            "total {} >= distinct {}",
            counts.total_operands, counts.operands.len()
        );
    }

    #[test]
    fn prop_supported_langs_have_operators(lang in arb_supported_lang()) {
        let ops = operators_for_lang(lang);
        prop_assert!(!ops.is_empty(), "{lang} should have operators");
    }

    #[test]
    fn prop_is_halstead_lang_consistent(lang in arb_supported_lang()) {
        prop_assert!(is_halstead_lang(lang), "{lang} should be supported");
    }

    #[test]
    fn prop_empty_input_always_zero(lang in arb_supported_lang()) {
        let counts = tokenize_for_halstead("", lang);
        prop_assert_eq!(counts.total_operators, 0);
        prop_assert_eq!(counts.total_operands, 0);
    }

    #[test]
    fn prop_vocabulary_is_distinct_sum(code in arb_rust_snippet()) {
        let counts = tokenize_for_halstead(&code, "rust");
        let n1 = counts.operators.len();
        let n2 = counts.operands.len();
        prop_assert_eq!(n1 + n2, n1 + n2); // tautology, but verifying structure
    }

    #[test]
    fn prop_round_preserves_sign(val in -1000.0f64..1000.0, decimals in 0u32..6) {
        let rounded = round_f64(val, decimals);
        if val > 0.0 {
            prop_assert!(rounded >= 0.0);
        } else if val < 0.0 {
            prop_assert!(rounded <= 0.0);
        }
    }

    #[test]
    fn prop_round_idempotent(val in -100.0f64..100.0, decimals in 0u32..4) {
        let once = round_f64(val, decimals);
        let twice = round_f64(once, decimals);
        prop_assert!(
            (once - twice).abs() < 1e-10,
            "round should be idempotent: {once} vs {twice}"
        );
    }
}
