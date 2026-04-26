//! Deep round-2 tests for `tokmd-analysis-complexity` (w51).
//!
//! Focuses on cyclomatic complexity with varied control flow patterns,
//! cognitive complexity scoring, risk indicator thresholds, and sorting.

use std::fs;
use std::path::PathBuf;

use tokmd_analysis_complexity::{build_complexity_report, generate_complexity_histogram};
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::{ComplexityRisk, FileComplexity};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn make_row(path: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "src".to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 0,
        blanks: 0,
        lines: code,
        bytes: code * 40,
        tokens: code * 8,
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

fn write_temp_files(files: &[(&str, &str)]) -> (tempfile::TempDir, Vec<PathBuf>) {
    let dir = tempfile::tempdir().expect("create tempdir");
    let mut paths = Vec::new();
    for (rel, content) in files {
        let full = dir.path().join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full, content).unwrap();
        paths.push(PathBuf::from(rel));
    }
    (dir, paths)
}

fn analyze(files: &[(&str, &str, &str)], detail: bool) -> tokmd_analysis_types::ComplexityReport {
    let file_entries: Vec<(&str, &str)> = files.iter().map(|(p, _, c)| (*p, *c)).collect();
    let (dir, paths) = write_temp_files(&file_entries);
    let rows: Vec<FileRow> = files
        .iter()
        .map(|(p, lang, c)| make_row(p, lang, c.lines().count()))
        .collect();
    let export = make_export(rows);
    build_complexity_report(
        dir.path(),
        &paths,
        &export,
        &AnalysisLimits::default(),
        detail,
    )
    .unwrap()
}

// ═══════════════════════════════════════════════════════════════════
// § Cyclomatic complexity – control flow patterns
// ═══════════════════════════════════════════════════════════════════

mod cyclomatic_patterns {
    use super::*;

    #[test]
    fn while_loop_adds_one() {
        let code = "fn run() {\n    while true {\n        break;\n    }\n}\n";
        let r = analyze(&[("w.rs", "Rust", code)], false);
        // Base 1 + 1 while = 2
        assert_eq!(r.files[0].cyclomatic_complexity, 2);
    }

    #[test]
    fn loop_keyword_adds_one() {
        let code = "fn run() {\n    loop {\n        break;\n    }\n}\n";
        let r = analyze(&[("l.rs", "Rust", code)], false);
        // Base 1 + 1 loop = 2
        assert_eq!(r.files[0].cyclomatic_complexity, 2);
    }

    #[test]
    fn logical_and_or_adds_complexity() {
        let code = "fn check(a: bool, b: bool, c: bool) -> bool {\n    a && b || c\n}\n";
        let r = analyze(&[("logic.rs", "Rust", code)], false);
        // Base 1 + 1 && + 1 || = 3
        assert_eq!(r.files[0].cyclomatic_complexity, 3);
    }

    #[test]
    fn question_mark_operator_adds_one() {
        let code = "fn try_it() -> Result<(), ()> {\n    let x = ok()?;\n    Ok(())\n}\n";
        let r = analyze(&[("q.rs", "Rust", code)], false);
        // Base 1 + 1 ? = 2
        assert_eq!(r.files[0].cyclomatic_complexity, 2);
    }

    #[test]
    fn deeply_nested_ifs_accumulate() {
        let code = r#"fn deep(x: i32) {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                if x > 1000 {
                    println!("huge");
                }
            }
        }
    }
}
"#;
        let r = analyze(&[("deep.rs", "Rust", code)], false);
        // Base 1 + 4 ifs = 5
        assert_eq!(r.files[0].cyclomatic_complexity, 5);
    }

    #[test]
    fn python_for_while_if() {
        let code = "def f(items):\n    for item in items:\n        while item > 0:\n            if item % 2 == 0:\n                item -= 1\n";
        let r = analyze(&[("f.py", "Python", code)], false);
        // Base 1 + 1 for + 1 while + 1 if = 4
        assert_eq!(r.files[0].cyclomatic_complexity, 4);
    }

    #[test]
    fn go_select_case() {
        let code = r#"func main() {
    select {
    case msg := <-ch1:
        fmt.Println(msg)
    case msg := <-ch2:
        fmt.Println(msg)
    }
}
"#;
        let r = analyze(&[("main.go", "Go", code)], false);
        // Base 1 + select + 2 case = depends on implementation
        assert!(r.files[0].cyclomatic_complexity >= 2);
    }

    #[test]
    fn javascript_ternary_adds_one() {
        let code = "function f(x) {\n    return x > 0 ? 'pos' : 'neg';\n}\n";
        let r = analyze(&[("f.js", "JavaScript", code)], false);
        // Base 1 + 1 ? (ternary)
        assert!(
            r.files[0].cyclomatic_complexity >= 2,
            "ternary should increase complexity"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Cognitive complexity
// ═══════════════════════════════════════════════════════════════════

mod cognitive_scoring {
    use super::*;

    #[test]
    fn linear_code_low_cognitive() {
        let code = "fn linear() {\n    let a = 1;\n    let b = 2;\n    let c = a + b;\n}\n";
        let r = analyze(&[("lin.rs", "Rust", code)], false);
        if let Some(cog) = r.files[0].cognitive_complexity {
            assert!(cog <= 1, "linear code → cognitive ≤ 1, got {cog}");
        }
    }

    #[test]
    fn nested_code_higher_cognitive_than_flat() {
        let flat = r#"fn flat(x: i32) {
    if x > 0 { println!("a"); }
    if x > 10 { println!("b"); }
    if x > 100 { println!("c"); }
}
"#;
        let nested = r#"fn nested(x: i32) {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                println!("deep");
            }
        }
    }
}
"#;
        let r_flat = analyze(&[("flat.rs", "Rust", flat)], false);
        let r_nested = analyze(&[("nested.rs", "Rust", nested)], false);

        let cog_flat = r_flat.files[0].cognitive_complexity.unwrap_or(0);
        let cog_nested = r_nested.files[0].cognitive_complexity.unwrap_or(0);
        assert!(
            cog_nested >= cog_flat,
            "nested ({cog_nested}) should be >= flat ({cog_flat})"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Risk level thresholds
// ═══════════════════════════════════════════════════════════════════

mod risk_thresholds {
    use super::*;

    #[test]
    fn simple_code_is_low_risk() {
        let code = "fn simple() {\n    let x = 1;\n}\n";
        let r = analyze(&[("simple.rs", "Rust", code)], false);
        assert_eq!(
            r.files[0].risk_level,
            ComplexityRisk::Low,
            "trivial code → low risk"
        );
    }

    #[test]
    fn high_cyclomatic_elevates_risk() {
        // Generate code with many branches to push cyclomatic complexity high
        let mut code = String::from("fn complex(x: i32) {\n");
        for i in 0..30 {
            code.push_str(&format!("    if x > {i} {{ println!(\"{i}\"); }}\n"));
        }
        code.push_str("}\n");
        let r = analyze(&[("complex.rs", "Rust", &code)], false);
        assert!(
            r.files[0].cyclomatic_complexity > 20,
            "should have high cyclomatic"
        );
        assert!(
            r.files[0].risk_level != ComplexityRisk::Low,
            "high cyclomatic → not low risk"
        );
    }

    #[test]
    fn high_risk_files_count_matches() {
        let simple = "fn a() {}\n";
        let complex_code = {
            let mut s = String::from("fn b(x: i32) {\n");
            for i in 0..60 {
                s.push_str(&format!("    if x > {i} {{ println!(\"{i}\"); }}\n"));
            }
            s.push_str("}\n");
            s
        };
        let r = analyze(
            &[
                ("simple.rs", "Rust", simple),
                ("complex.rs", "Rust", &complex_code),
            ],
            false,
        );
        let actual_high = r
            .files
            .iter()
            .filter(|f| {
                matches!(
                    f.risk_level,
                    ComplexityRisk::High | ComplexityRisk::Critical
                )
            })
            .count();
        assert_eq!(r.high_risk_files, actual_high);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Sorting: highest cyclomatic first
// ═══════════════════════════════════════════════════════════════════

mod sorting {
    use super::*;

    #[test]
    fn files_sorted_by_cyclomatic_descending() {
        let low = "fn a() { let x = 1; }\n";
        let mid = "fn b(x: i32) {\n    if x > 0 { println!(\"a\"); }\n    if x > 1 { println!(\"b\"); }\n}\n";
        let high = {
            let mut s = String::from("fn c(x: i32) {\n");
            for i in 0..10 {
                s.push_str(&format!("    if x > {i} {{ println!(\"{i}\"); }}\n"));
            }
            s.push_str("}\n");
            s
        };
        let r = analyze(
            &[
                ("low.rs", "Rust", low),
                ("high.rs", "Rust", &high),
                ("mid.rs", "Rust", mid),
            ],
            false,
        );
        for w in r.files.windows(2) {
            assert!(
                w[0].cyclomatic_complexity >= w[1].cyclomatic_complexity,
                "files not sorted descending: {} < {}",
                w[0].cyclomatic_complexity,
                w[1].cyclomatic_complexity
            );
        }
    }

    #[test]
    fn tie_breaking_is_deterministic() {
        let code = "fn f() { let x = 1; }\n";
        let r1 = analyze(
            &[
                ("a.rs", "Rust", code),
                ("b.rs", "Rust", code),
                ("c.rs", "Rust", code),
            ],
            false,
        );
        let r2 = analyze(
            &[
                ("c.rs", "Rust", code),
                ("a.rs", "Rust", code),
                ("b.rs", "Rust", code),
            ],
            false,
        );
        let paths1: Vec<&str> = r1.files.iter().map(|f| f.path.as_str()).collect();
        let paths2: Vec<&str> = r2.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths1, paths2, "same complexity → deterministic path order");
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Histogram placement
// ═══════════════════════════════════════════════════════════════════

mod histogram_round2 {
    use super::*;

    #[test]
    fn all_low_complexity_in_first_bucket() {
        let files: Vec<FileComplexity> = (0..5)
            .map(|i| FileComplexity {
                path: format!("f{i}.rs"),
                module: "src".to_string(),
                function_count: 1,
                max_function_length: 5,
                cyclomatic_complexity: 2,
                cognitive_complexity: None,
                max_nesting: None,
                risk_level: ComplexityRisk::Low,
                functions: None,
            })
            .collect();
        let h = generate_complexity_histogram(&files, 5);
        assert_eq!(h.counts[0], 5, "all low complexity in first bucket");
        assert_eq!(h.total, 5);
    }

    #[test]
    fn high_complexity_clamped_to_last_bucket() {
        let files = vec![FileComplexity {
            path: "huge.rs".to_string(),
            module: "src".to_string(),
            function_count: 100,
            max_function_length: 500,
            cyclomatic_complexity: 999,
            cognitive_complexity: Some(500),
            max_nesting: Some(15),
            risk_level: ComplexityRisk::Critical,
            functions: None,
        }];
        let h = generate_complexity_histogram(&files, 5);
        assert_eq!(
            h.counts.last().copied().unwrap_or(0),
            1,
            "huge complexity in last bucket"
        );
    }
}
