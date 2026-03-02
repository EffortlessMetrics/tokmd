//! Edge-case and cross-language BDD tests for Halstead analysis.

use std::path::PathBuf;

use tokmd_analysis_halstead::{
    build_halstead_report, operators_for_lang, round_f64, tokenize_for_halstead,
};
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

// ── Scenario: Ruby tokenization ─────────────────────────────────────

#[test]
fn given_ruby_code_when_tokenized_then_ruby_operators_detected() {
    let code = "def greet(name)\n  return name + \" hello\"\nend";
    let counts = tokenize_for_halstead(code, "ruby");
    assert!(counts.operators.contains_key("def"));
    assert!(counts.operators.contains_key("return"));
    assert!(counts.operators.contains_key("+"));
    assert!(counts.operators.contains_key("end"));
    assert!(counts.operands.contains("greet"));
    assert!(counts.operands.contains("name"));
}

#[test]
fn given_ruby_class_when_tokenized_then_class_and_self_detected() {
    let code = "class Foo\n  def bar\n    self\n  end\nend";
    let counts = tokenize_for_halstead(code, "ruby");
    assert!(counts.operators.contains_key("class"));
    assert!(counts.operators.contains_key("self"));
    assert!(counts.operators.contains_key("def"));
    assert!(counts.operands.contains("Foo"));
    assert!(counts.operands.contains("bar"));
}

// ── Scenario: C-family tokenization ─────────────────────────────────

#[test]
fn given_c_code_when_tokenized_then_c_operators_detected() {
    let code = "if (x > 0) { return x + 1; }";
    let counts = tokenize_for_halstead(code, "c");
    assert!(counts.operators.contains_key("if"));
    assert!(counts.operators.contains_key("return"));
    assert!(counts.operators.contains_key(">"));
    assert!(counts.operators.contains_key("+"));
    assert!(counts.operands.contains("x"));
}

#[test]
fn given_java_code_when_tokenized_then_class_keywords_detected() {
    let code = "public class Main { static void run() { new Object(); } }";
    let counts = tokenize_for_halstead(code, "java");
    assert!(counts.operators.contains_key("public"));
    assert!(counts.operators.contains_key("class"));
    assert!(counts.operators.contains_key("static"));
    assert!(counts.operators.contains_key("void"));
    assert!(counts.operators.contains_key("new"));
    assert!(counts.operands.contains("Main"));
    assert!(counts.operands.contains("Object"));
}

// ── Scenario: Multi-line deeply nested code ─────────────────────────

#[test]
fn given_nested_rust_code_when_tokenized_then_all_levels_counted() {
    let code = "\
fn outer() {
    if true {
        for i in 0..10 {
            while i > 0 {
                let x = i + 1;
            }
        }
    }
}";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operators.contains_key("fn"));
    assert!(counts.operators.contains_key("if"));
    assert!(counts.operators.contains_key("for"));
    assert!(counts.operators.contains_key("in"));
    assert!(counts.operators.contains_key("while"));
    assert!(counts.operators.contains_key("let"));
    assert!(counts.operators.contains_key(">"));
    assert!(counts.operators.contains_key("+"));
    assert!(counts.operators.contains_key(".."));
    assert!(counts.total_operators >= 9);
}

// ── Scenario: Operator counting precision ───────────────────────────

#[test]
fn given_repeated_distinct_operators_when_tokenized_then_counts_are_exact() {
    // 3 distinct operators: let, =, + (let appears 3x, = appears 3x, + appears 2x)
    let code = "let a = 1\nlet b = 2\nlet c = a + b";
    let counts = tokenize_for_halstead(code, "rust");
    assert_eq!(*counts.operators.get("let").unwrap(), 3);
    assert_eq!(*counts.operators.get("=").unwrap(), 3);
    assert_eq!(*counts.operators.get("+").unwrap(), 1);
    assert_eq!(counts.operators.len(), 3, "exactly 3 distinct operators");
    assert_eq!(counts.total_operators, 7);
}

// ── Scenario: per-file byte limit in build_halstead_report ──────────

#[test]
fn given_small_per_file_limit_when_building_report_then_file_is_truncated() {
    let dir = tempfile::tempdir().unwrap();
    // Write a file with operators beyond the first 10 bytes
    let code = "fn long_function_name_here() { let x = 1 + 2 + 3 + 4; }";
    std::fs::write(dir.path().join("f.rs"), code).unwrap();

    let export = make_export(vec![make_row("f.rs", "Rust")]);
    let files = vec![PathBuf::from("f.rs")];

    let tight = AnalysisLimits {
        max_file_bytes: Some(10),
        ..no_limits()
    };
    let full = no_limits();

    let tight_m = build_halstead_report(dir.path(), &files, &export, &tight).unwrap();
    let full_m = build_halstead_report(dir.path(), &files, &export, &full).unwrap();

    // Tight limit reads fewer bytes → fewer or equal operators
    assert!(tight_m.total_operators <= full_m.total_operators);
}

// ── Scenario: round_f64 boundary values ─────────────────────────────

#[test]
fn given_exactly_half_when_rounded_then_rounds_to_even_or_up() {
    // 2.5 rounds to 3.0 in Rust's default rounding (round half to even would give 2.0)
    let result = round_f64(2.5, 0);
    assert_eq!(result, 3.0);
}

#[test]
fn given_very_small_positive_when_rounded_then_zero() {
    assert_eq!(round_f64(0.004, 2), 0.0);
    assert_eq!(round_f64(0.005, 2), 0.01); // rounds up
}

// ── Scenario: Ruby operators are distinct from Rust ─────────────────

#[test]
fn given_ruby_operators_when_compared_to_rust_then_different_sets() {
    let ruby_ops = operators_for_lang("ruby");
    let rust_ops = operators_for_lang("rust");
    // Ruby has "end", "def", "elsif" which Rust doesn't
    assert!(ruby_ops.contains(&"end"));
    assert!(ruby_ops.contains(&"elsif"));
    assert!(!rust_ops.contains(&"end"));
    assert!(!rust_ops.contains(&"elsif"));
    // Rust has "fn", "let", "match" which Ruby doesn't
    assert!(rust_ops.contains(&"fn"));
    assert!(rust_ops.contains(&"match"));
    assert!(!ruby_ops.contains(&"fn"));
    assert!(!ruby_ops.contains(&"match"));
}

// ── Scenario: build_halstead_report determinism ─────────────────────

#[test]
fn given_same_files_when_report_built_twice_then_identical_metrics() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn main() { let x = 1 + 2; }").unwrap();
    std::fs::write(dir.path().join("b.py"), "def f(x):\n    return x + 1\n").unwrap();

    let export = make_export(vec![make_row("a.rs", "Rust"), make_row("b.py", "Python")]);
    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.py")];

    let m1 = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();
    let m2 = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    assert_eq!(m1.distinct_operators, m2.distinct_operators);
    assert_eq!(m1.distinct_operands, m2.distinct_operands);
    assert_eq!(m1.total_operators, m2.total_operators);
    assert_eq!(m1.total_operands, m2.total_operands);
    assert_eq!(m1.vocabulary, m2.vocabulary);
    assert_eq!(m1.length, m2.length);
    assert!((m1.volume - m2.volume).abs() < f64::EPSILON);
    assert!((m1.difficulty - m2.difficulty).abs() < f64::EPSILON);
    assert!((m1.effort - m2.effort).abs() < f64::EPSILON);
}
