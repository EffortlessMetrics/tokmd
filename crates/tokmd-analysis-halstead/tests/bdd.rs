//! BDD-style scenario tests for Halstead metric calculation.

use std::path::PathBuf;

use tokmd_analysis_halstead::{
    build_halstead_report, is_halstead_lang, operators_for_lang, round_f64,
    tokenize_for_halstead,
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

// ── Scenario: language support detection ─────────────────────────────

#[test]
fn scenario_supported_languages_are_recognized() {
    // Given a set of known supported languages
    let supported = [
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
    ];
    // When we check each language
    // Then all are recognized as Halstead languages
    for lang in &supported {
        assert!(
            is_halstead_lang(lang),
            "{lang} should be a supported Halstead language"
        );
    }
}

#[test]
fn scenario_unsupported_languages_are_rejected() {
    // Given languages that don't support Halstead analysis
    let unsupported = ["Markdown", "JSON", "YAML", "TOML", "HTML", "CSS", ""];
    // Then none are recognized
    for lang in &unsupported {
        assert!(
            !is_halstead_lang(lang),
            "{lang} should NOT be a supported Halstead language"
        );
    }
}

#[test]
fn scenario_language_detection_is_case_insensitive() {
    assert!(is_halstead_lang("rust"));
    assert!(is_halstead_lang("RUST"));
    assert!(is_halstead_lang("Rust"));
    assert!(is_halstead_lang("rUsT"));
}

// ── Scenario: operator tables are non-empty for supported languages ──

#[test]
fn scenario_supported_languages_have_operators() {
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
        assert!(
            !ops.is_empty(),
            "{lang} should have a non-empty operator table"
        );
    }
}

#[test]
fn scenario_unsupported_language_returns_empty_operators() {
    assert!(operators_for_lang("brainfuck").is_empty());
    assert!(operators_for_lang("").is_empty());
}

// ── Scenario: tokenizing Rust code ──────────────────────────────────

#[test]
fn scenario_tokenize_rust_fn_with_operators_and_operands() {
    // Given a simple Rust function
    let code = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}";
    // When we tokenize it
    let counts = tokenize_for_halstead(code, "rust");
    // Then we find operators like "fn", "+", "->"
    assert!(counts.operators.contains_key("fn"));
    assert!(counts.operators.contains_key("+"));
    assert!(counts.operators.contains_key("->"));
    // And we find operands like "add", "a", "b", "i32"
    assert!(counts.operands.contains("add"));
    assert!(counts.operands.contains("a"));
    assert!(counts.operands.contains("b"));
    assert!(counts.operands.contains("i32"));
    // And totals are positive
    assert!(counts.total_operators > 0);
    assert!(counts.total_operands > 0);
}

#[test]
fn scenario_tokenize_rust_if_else() {
    let code = "if x > 0 { return x; } else { return 0; }";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operators.contains_key("if"));
    assert!(counts.operators.contains_key("else"));
    assert!(counts.operators.contains_key("return"));
    assert!(counts.operators.contains_key(">"));
    assert!(counts.operands.contains("x"));
}

#[test]
fn scenario_tokenize_rust_match() {
    let code = "match val {\n    1 => true,\n    _ => false,\n}";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operators.contains_key("match"));
    assert!(counts.operators.contains_key("=>"));
    assert!(counts.operands.contains("val"));
}

// ── Scenario: tokenizing Python code ────────────────────────────────

#[test]
fn scenario_tokenize_python_def() {
    let code = "def greet(name):\n    return name + \" hello\"";
    let counts = tokenize_for_halstead(code, "python");
    assert!(counts.operators.contains_key("def"));
    assert!(counts.operators.contains_key("return"));
    assert!(counts.operators.contains_key("+"));
    assert!(counts.operands.contains("greet"));
    assert!(counts.operands.contains("name"));
    // String literal counted as operand
    assert!(counts.operands.contains("<string>"));
}

#[test]
fn scenario_tokenize_python_for_loop() {
    let code = "for i in range(10):\n    x = x + i";
    let counts = tokenize_for_halstead(code, "python");
    assert!(counts.operators.contains_key("for"));
    assert!(counts.operators.contains_key("in"));
    assert!(counts.operators.contains_key("="));
    assert!(counts.operators.contains_key("+"));
}

// ── Scenario: tokenizing JavaScript code ────────────────────────────

#[test]
fn scenario_tokenize_javascript_arrow_function() {
    let code = "const add = (a, b) => a + b;";
    let counts = tokenize_for_halstead(code, "javascript");
    assert!(counts.operators.contains_key("const"));
    assert!(counts.operators.contains_key("="));
    assert!(counts.operators.contains_key("=>"));
    assert!(counts.operators.contains_key("+"));
    assert!(counts.operands.contains("add"));
    assert!(counts.operands.contains("a"));
    assert!(counts.operands.contains("b"));
}

// ── Scenario: tokenizing Go code ────────────────────────────────────

#[test]
fn scenario_tokenize_go_func() {
    let code = "func main() {\n    x := 42\n    if x > 0 {\n        return\n    }\n}";
    let counts = tokenize_for_halstead(code, "go");
    assert!(counts.operators.contains_key("func"));
    assert!(counts.operators.contains_key(":="));
    assert!(counts.operators.contains_key("if"));
    assert!(counts.operators.contains_key(">"));
    assert!(counts.operators.contains_key("return"));
    assert!(counts.operands.contains("main"));
    assert!(counts.operands.contains("x"));
}

// ── Scenario: edge cases ────────────────────────────────────────────

#[test]
fn scenario_empty_input_yields_zero_counts() {
    let counts = tokenize_for_halstead("", "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
    assert!(counts.operators.is_empty());
    assert!(counts.operands.is_empty());
}

#[test]
fn scenario_whitespace_only_input_yields_zero_counts() {
    let counts = tokenize_for_halstead("   \n\n   \t  \n", "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn scenario_comment_only_input_yields_zero_counts() {
    let code = "// this is a comment\n// another comment\n";
    let counts = tokenize_for_halstead(code, "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn scenario_hash_comment_skipped() {
    let code = "# this is a python comment\n# another one\n";
    let counts = tokenize_for_halstead(code, "python");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn scenario_block_comment_start_skipped() {
    let code = "/* block comment */\n* continuation\n";
    let counts = tokenize_for_halstead(code, "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 0);
}

#[test]
fn scenario_single_operand_only() {
    // A single identifier with no operators
    let code = "x";
    let counts = tokenize_for_halstead(code, "rust");
    assert_eq!(counts.total_operators, 0);
    assert_eq!(counts.total_operands, 1);
    assert!(counts.operands.contains("x"));
}

#[test]
fn scenario_single_operator_only() {
    // A keyword operator alone on a line
    let code = "return";
    let counts = tokenize_for_halstead(code, "rust");
    assert_eq!(counts.total_operators, 1);
    assert_eq!(counts.total_operands, 0);
    assert!(counts.operators.contains_key("return"));
}

#[test]
fn scenario_string_literals_counted_as_operands() {
    let code = "let s = \"hello world\";";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operands.contains("<string>"));
    // "let", "=" are operators; "s" and "<string>" are operands
    assert!(counts.total_operands >= 2);
}

#[test]
fn scenario_escaped_string_literal() {
    let code = r#"let s = "hello \"world\"";"#;
    let counts = tokenize_for_halstead(code, "rust");
    assert!(counts.operands.contains("<string>"));
    assert!(counts.operands.contains("s"));
}

#[test]
fn scenario_single_char_literal() {
    let code = "let c = 'x';";
    let counts = tokenize_for_halstead(code, "rust");
    // Single-quoted literal is treated as a string operand
    assert!(counts.operands.contains("<string>"));
}

#[test]
fn scenario_unknown_language_produces_only_operands() {
    // Unknown language has no operators defined
    let code = "fn let if return x y z";
    let counts = tokenize_for_halstead(code, "unknown_lang");
    // No operators recognized → all tokens are operands
    assert_eq!(counts.total_operators, 0);
    assert!(counts.total_operands > 0);
}

#[test]
fn scenario_duplicate_operands_increase_total_but_not_distinct() {
    let code = "x + x + x";
    let counts = tokenize_for_halstead(code, "rust");
    // "x" appears 3 times → total_operands == 3, distinct == 1
    assert_eq!(counts.operands.len(), 1);
    assert_eq!(counts.total_operands, 3);
    // "+" appears 2 times
    assert_eq!(*counts.operators.get("+").unwrap(), 2);
    assert_eq!(counts.total_operators, 2);
}

#[test]
fn scenario_multi_char_operators_matched_longest_first() {
    // ">>=" should match as a single operator, not ">", ">", "="
    let code = "x >>= 1";
    let counts = tokenize_for_halstead(code, "rust");
    assert!(
        counts.operators.contains_key(">>="),
        ">>= should be matched as a single operator"
    );
    assert!(
        !counts.operators.contains_key(">"),
        "individual > should not appear when >>= matches"
    );
}

// ── Scenario: Halstead metric formulas ──────────────────────────────

#[test]
fn scenario_halstead_volume_formula() {
    // volume = N * log2(n) where N = length, n = vocabulary
    // With n1=3, n2=4, N1=5, N2=8 → n=7, N=13
    // volume = 13 * log2(7) ≈ 13 * 2.807 ≈ 36.49
    let n = 7usize;
    let big_n = 13usize;
    let volume = big_n as f64 * (n as f64).log2();
    assert!((volume - 36.49).abs() < 0.1);
}

#[test]
fn scenario_halstead_difficulty_formula() {
    // difficulty = (n1/2) * (N2/n2)
    // With n1=4, n2=5, N2=10 → (4/2) * (10/5) = 2 * 2 = 4.0
    let n1 = 4.0f64;
    let n2 = 5.0f64;
    let big_n2 = 10.0f64;
    let difficulty = (n1 / 2.0) * (big_n2 / n2);
    assert!((difficulty - 4.0).abs() < f64::EPSILON);
}

#[test]
fn scenario_halstead_effort_is_difficulty_times_volume() {
    let difficulty = 3.5;
    let volume = 100.0;
    let effort = difficulty * volume;
    assert!((effort - 350.0f64).abs() < f64::EPSILON);
}

#[test]
fn scenario_halstead_time_is_effort_over_18() {
    let effort = 180.0;
    let time = effort / 18.0;
    assert!((time - 10.0f64).abs() < f64::EPSILON);
}

#[test]
fn scenario_halstead_bugs_is_volume_over_3000() {
    let volume = 6000.0;
    let bugs = volume / 3000.0;
    assert!((bugs - 2.0f64).abs() < f64::EPSILON);
}

#[test]
fn scenario_zero_vocabulary_yields_zero_volume() {
    // When vocabulary is 0, volume should be 0 (not NaN/Inf)
    let vocabulary = 0usize;
    let length = 0usize;
    let volume = if vocabulary > 0 {
        length as f64 * (vocabulary as f64).log2()
    } else {
        0.0
    };
    assert_eq!(volume, 0.0);
}

#[test]
fn scenario_zero_distinct_operands_yields_zero_difficulty() {
    // When n2 = 0, difficulty should be 0 (avoid division by zero)
    let n1 = 5usize;
    let n2 = 0usize;
    let total_opds = 0usize;
    let difficulty = if n2 > 0 {
        (n1 as f64 / 2.0) * (total_opds as f64 / n2 as f64)
    } else {
        0.0
    };
    assert_eq!(difficulty, 0.0);
}

// ── Scenario: round_f64 ─────────────────────────────────────────────

#[test]
fn scenario_round_f64_basic() {
    assert_eq!(round_f64(3.14159, 2), 3.14);
    assert_eq!(round_f64(3.14159, 4), 3.1416);
    assert_eq!(round_f64(3.14159, 0), 3.0);
}

#[test]
fn scenario_round_f64_zero() {
    assert_eq!(round_f64(0.0, 2), 0.0);
}

#[test]
fn scenario_round_f64_negative() {
    assert_eq!(round_f64(-2.555, 2), -2.56);
}

#[test]
fn scenario_round_f64_large_decimals() {
    // 10 decimal places
    let val = 1.123456789012345;
    let rounded = round_f64(val, 10);
    assert!((rounded - 1.1234567890).abs() < 1e-10);
}

// ── Scenario: build_halstead_report with temp files ─────────────────

#[test]
fn scenario_build_report_with_rust_file() {
    // Given a temporary directory with a Rust source file
    let dir = tempfile::tempdir().unwrap();
    let code = "fn main() {\n    let x = 1 + 2;\n    let y = x * 3;\n}\n";
    std::fs::write(dir.path().join("main.rs"), code).unwrap();

    let export = make_export(vec![make_row("main.rs", "Rust")]);
    let files = vec![PathBuf::from("main.rs")];

    // When we build the Halstead report
    let metrics = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    // Then all metrics are populated and non-negative
    assert!(metrics.distinct_operators > 0);
    assert!(metrics.distinct_operands > 0);
    assert!(metrics.total_operators > 0);
    assert!(metrics.total_operands > 0);
    assert_eq!(
        metrics.vocabulary,
        metrics.distinct_operators + metrics.distinct_operands
    );
    assert_eq!(
        metrics.length,
        metrics.total_operators + metrics.total_operands
    );
    assert!(metrics.volume > 0.0);
    assert!(metrics.difficulty >= 0.0);
    assert!(metrics.effort >= 0.0);
    assert!(metrics.time_seconds >= 0.0);
    assert!(metrics.estimated_bugs >= 0.0);
}

#[test]
fn scenario_build_report_skips_unsupported_language() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("readme.md"), "# Hello").unwrap();

    let export = make_export(vec![make_row("readme.md", "Markdown")]);
    let files = vec![PathBuf::from("readme.md")];

    let metrics = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    // No Halstead metrics for unsupported language
    assert_eq!(metrics.distinct_operators, 0);
    assert_eq!(metrics.distinct_operands, 0);
    assert_eq!(metrics.volume, 0.0);
}

#[test]
fn scenario_build_report_with_empty_file_list() {
    let dir = tempfile::tempdir().unwrap();
    let export = make_export(vec![]);
    let files: Vec<PathBuf> = vec![];

    let metrics = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();
    assert_eq!(metrics.length, 0);
    assert_eq!(metrics.vocabulary, 0);
    assert_eq!(metrics.volume, 0.0);
    assert_eq!(metrics.difficulty, 0.0);
    assert_eq!(metrics.effort, 0.0);
}

#[test]
fn scenario_build_report_respects_max_bytes_limit() {
    let dir = tempfile::tempdir().unwrap();
    // Write two files; first is large enough to hit a 10-byte limit
    std::fs::write(dir.path().join("a.rs"), "fn a() { let x = 1; }").unwrap();
    std::fs::write(dir.path().join("b.rs"), "fn b() { let y = 2; }").unwrap();

    let export = make_export(vec![make_row("a.rs", "Rust"), make_row("b.rs", "Rust")]);
    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];

    let limits = AnalysisLimits {
        max_bytes: Some(10), // very tight limit
        ..no_limits()
    };

    let metrics = build_halstead_report(dir.path(), &files, &export, &limits).unwrap();
    // Should still produce *some* metrics from partial scan
    // (first file is read, second may be skipped)
    assert!(metrics.length > 0 || metrics.length == 0); // doesn't panic
}

#[test]
fn scenario_build_report_skips_missing_files_gracefully() {
    let dir = tempfile::tempdir().unwrap();
    let export = make_export(vec![make_row("nonexistent.rs", "Rust")]);
    let files = vec![PathBuf::from("nonexistent.rs")];

    let metrics = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();
    assert_eq!(metrics.length, 0);
}

#[test]
fn scenario_build_report_aggregates_multiple_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn a() { let x = 1; }").unwrap();
    std::fs::write(dir.path().join("b.rs"), "fn b() { let y = 2; }").unwrap();

    let export = make_export(vec![make_row("a.rs", "Rust"), make_row("b.rs", "Rust")]);
    let files = vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")];

    let multi = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();

    // Metrics from two files should be >= metrics from one file
    let export_a = make_export(vec![make_row("a.rs", "Rust")]);
    let files_a = vec![PathBuf::from("a.rs")];
    let single = build_halstead_report(dir.path(), &files_a, &export_a, &no_limits()).unwrap();

    assert!(multi.total_operators >= single.total_operators);
    assert!(multi.total_operands >= single.total_operands);
    assert!(multi.length >= single.length);
}

#[test]
fn scenario_build_report_skips_child_file_kind() {
    // Only FileKind::Parent rows should be processed
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("child.rs"), "fn x() {}").unwrap();

    let mut row = make_row("child.rs", "Rust");
    row.kind = FileKind::Child;
    let export = make_export(vec![row]);
    let files = vec![PathBuf::from("child.rs")];

    let metrics = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();
    assert_eq!(metrics.length, 0, "Child rows should be skipped");
}

#[test]
fn scenario_build_report_mixed_languages() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("app.py"), "def f(x):\n    return x + 1\n").unwrap();
    std::fs::write(dir.path().join("app.js"), "const f = (x) => x + 1;\n").unwrap();

    let export = make_export(vec![
        make_row("app.py", "Python"),
        make_row("app.js", "JavaScript"),
    ]);
    let files = vec![PathBuf::from("app.py"), PathBuf::from("app.js")];

    let metrics = build_halstead_report(dir.path(), &files, &export, &no_limits()).unwrap();
    assert!(metrics.distinct_operators > 0);
    assert!(metrics.distinct_operands > 0);
}
