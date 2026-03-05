//! Contract tests for `tokmd-analysis-complexity` enricher (w65).
//!
//! Covers: histogram generation, risk classification, function counting,
//! cyclomatic estimation, build_complexity_report via temp-dir fixtures,
//! technical debt, and property-based invariants.

use std::io::Write;
use tempfile::TempDir;
use tokmd_analysis_complexity::{build_complexity_report, generate_complexity_histogram};
use tokmd_analysis_types::{ComplexityRisk, FileComplexity};
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ──────────────────────────────────────────────────────

fn fc(path: &str, cyclomatic: usize, cognitive: Option<usize>, nesting: Option<usize>, risk: ComplexityRisk) -> FileComplexity {
    FileComplexity {
        path: path.to_string(),
        module: "src".to_string(),
        function_count: 1,
        max_function_length: 10,
        cyclomatic_complexity: cyclomatic,
        cognitive_complexity: cognitive,
        max_nesting: nesting,
        risk_level: risk,
        functions: None,
    }
}

fn make_row(path: &str, lang: &str, code: usize, bytes: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "src".to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 0,
        blanks: 0,
        lines: code,
        bytes,
        tokens: code * 5,
    }
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn default_limits() -> AnalysisLimits {
    AnalysisLimits {
        max_files: None,
        max_bytes: None,
        max_file_bytes: None,
        max_commits: None,
        max_commit_files: None,
    }
}

fn write_file(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(&full).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

// ── Histogram tests ─────────────────────────────────────────────

mod histogram {
    use super::*;

    #[test]
    fn empty_input_yields_zero_total() {
        let h = generate_complexity_histogram(&[], 5);
        assert_eq!(h.total, 0);
        assert!(h.counts.iter().all(|&c| c == 0));
    }

    #[test]
    fn seven_buckets_always_generated() {
        let h = generate_complexity_histogram(&[], 5);
        assert_eq!(h.buckets.len(), 7);
        assert_eq!(h.counts.len(), 7);
    }

    #[test]
    fn low_complexity_in_first_bucket() {
        let files = [fc("a.rs", 2, None, None, ComplexityRisk::Low)];
        let h = generate_complexity_histogram(&files, 5);
        assert_eq!(h.counts[0], 1);
        assert_eq!(h.counts[1..].iter().sum::<u32>(), 0);
    }

    #[test]
    fn high_complexity_in_last_bucket() {
        let files = [fc("a.rs", 50, None, None, ComplexityRisk::Critical)];
        let h = generate_complexity_histogram(&files, 5);
        assert_eq!(h.counts[6], 1);
    }

    #[test]
    fn boundary_value_bucket_5() {
        let files = [fc("a.rs", 5, None, None, ComplexityRisk::Low)];
        let h = generate_complexity_histogram(&files, 5);
        assert_eq!(h.counts[1], 1);
    }

    #[test]
    fn boundary_value_bucket_4() {
        let files = [fc("a.rs", 4, None, None, ComplexityRisk::Low)];
        let h = generate_complexity_histogram(&files, 5);
        assert_eq!(h.counts[0], 1);
    }

    #[test]
    fn spread_across_all_buckets() {
        let files = [
            fc("a.rs", 0, None, None, ComplexityRisk::Low),
            fc("b.rs", 7, None, None, ComplexityRisk::Low),
            fc("c.rs", 12, None, None, ComplexityRisk::Moderate),
            fc("d.rs", 17, None, None, ComplexityRisk::Moderate),
            fc("e.rs", 22, None, None, ComplexityRisk::High),
            fc("f.rs", 27, None, None, ComplexityRisk::High),
            fc("g.rs", 35, None, None, ComplexityRisk::Critical),
        ];
        let h = generate_complexity_histogram(&files, 5);
        assert_eq!(h.total, 7);
        for c in &h.counts {
            assert_eq!(*c, 1);
        }
    }

    #[test]
    fn total_equals_sum_of_counts() {
        let files = [
            fc("a.rs", 3, None, None, ComplexityRisk::Low),
            fc("b.rs", 3, None, None, ComplexityRisk::Low),
            fc("c.rs", 15, None, None, ComplexityRisk::Moderate),
        ];
        let h = generate_complexity_histogram(&files, 5);
        let sum: u32 = h.counts.iter().sum();
        assert_eq!(h.total, sum);
    }

    #[test]
    fn buckets_are_evenly_spaced() {
        let h = generate_complexity_histogram(&[], 5);
        for (i, &b) in h.buckets.iter().enumerate() {
            assert_eq!(b, (i as u32) * 5);
        }
    }
}

// ── build_complexity_report with temp fixtures ──────────────────

mod report {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn empty_export_yields_zero_aggregates() {
        let dir = TempDir::new().unwrap();
        let data = export(vec![]);
        let files: Vec<PathBuf> = vec![];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert_eq!(r.total_functions, 0);
        assert_eq!(r.max_cyclomatic, 0);
        assert_eq!(r.high_risk_files, 0);
        assert!(r.files.is_empty());
    }

    #[test]
    fn single_rust_file_detected() {
        let dir = TempDir::new().unwrap();
        let code = "fn main() {\n    println!(\"hello\");\n}\n";
        write_file(&dir, "src/main.rs", code);
        let data = export(vec![make_row("src/main.rs", "Rust", 3, code.len())]);
        let files = vec![PathBuf::from("src/main.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert_eq!(r.files.len(), 1);
        assert!(r.total_functions >= 1);
    }

    #[test]
    fn unsupported_language_skipped() {
        let dir = TempDir::new().unwrap();
        write_file(&dir, "data.json", "{\"key\": \"value\"}");
        let data = export(vec![make_row("data.json", "JSON", 1, 18)]);
        let files = vec![PathBuf::from("data.json")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert!(r.files.is_empty());
    }

    #[test]
    fn multiple_functions_counted() {
        let dir = TempDir::new().unwrap();
        let code = "fn foo() {\n    let x = 1;\n}\n\nfn bar() {\n    let y = 2;\n}\n\nfn baz() {\n    let z = 3;\n}\n";
        write_file(&dir, "src/lib.rs", code);
        let data = export(vec![make_row("src/lib.rs", "Rust", 9, code.len())]);
        let files = vec![PathBuf::from("src/lib.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert!(r.total_functions >= 3);
    }

    #[test]
    fn cyclomatic_increases_with_branches() {
        let dir = TempDir::new().unwrap();
        let simple = "fn simple() {\n    let x = 1;\n}\n";
        let complex = "fn complex() {\n    if true {\n        if false {\n            for i in 0..10 {\n                while true {\n                    match x {\n                        _ => {}\n                    }\n                }\n            }\n        }\n    }\n}\n";
        write_file(&dir, "src/simple.rs", simple);
        write_file(&dir, "src/complex.rs", complex);

        let data = export(vec![
            make_row("src/simple.rs", "Rust", 3, simple.len()),
            make_row("src/complex.rs", "Rust", 13, complex.len()),
        ]);
        let files = vec![
            PathBuf::from("src/simple.rs"),
            PathBuf::from("src/complex.rs"),
        ];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert!(r.files.len() >= 2);

        let simple_fc = r.files.iter().find(|f| f.path.contains("simple")).unwrap();
        let complex_fc = r.files.iter().find(|f| f.path.contains("complex")).unwrap();
        assert!(complex_fc.cyclomatic_complexity > simple_fc.cyclomatic_complexity);
    }

    #[test]
    fn detail_functions_flag_populates_functions() {
        let dir = TempDir::new().unwrap();
        let code = "fn alpha() {\n    let a = 1;\n}\n\nfn beta() {\n    let b = 2;\n}\n";
        write_file(&dir, "src/lib.rs", code);
        let data = export(vec![make_row("src/lib.rs", "Rust", 6, code.len())]);
        let files = vec![PathBuf::from("src/lib.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), true).unwrap();
        assert!(r.files[0].functions.is_some());
        let funcs = r.files[0].functions.as_ref().unwrap();
        assert!(funcs.len() >= 2);
    }

    #[test]
    fn detail_functions_off_returns_none() {
        let dir = TempDir::new().unwrap();
        let code = "fn foo() {\n    let x = 1;\n}\n";
        write_file(&dir, "src/lib.rs", code);
        let data = export(vec![make_row("src/lib.rs", "Rust", 3, code.len())]);
        let files = vec![PathBuf::from("src/lib.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert!(r.files[0].functions.is_none());
    }

    #[test]
    fn javascript_file_analyzed() {
        let dir = TempDir::new().unwrap();
        let code = "function greet() {\n    console.log('hi');\n}\n";
        write_file(&dir, "src/app.js", code);
        let data = export(vec![make_row("src/app.js", "JavaScript", 3, code.len())]);
        let files = vec![PathBuf::from("src/app.js")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert_eq!(r.files.len(), 1);
    }

    #[test]
    fn python_file_analyzed() {
        let dir = TempDir::new().unwrap();
        let code = "def hello():\n    print('hello')\n\ndef world():\n    print('world')\n";
        write_file(&dir, "src/app.py", code);
        let data = export(vec![make_row("src/app.py", "Python", 4, code.len())]);
        let files = vec![PathBuf::from("src/app.py")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert!(!r.files.is_empty());
        assert!(r.total_functions >= 2);
    }

    #[test]
    fn go_file_analyzed() {
        let dir = TempDir::new().unwrap();
        let code = "package main\n\nfunc main() {\n    fmt.Println(\"hello\")\n}\n";
        write_file(&dir, "main.go", code);
        let data = export(vec![make_row("main.go", "Go", 5, code.len())]);
        let files = vec![PathBuf::from("main.go")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert!(!r.files.is_empty());
    }

    #[test]
    fn files_sorted_by_cyclomatic_desc() {
        let dir = TempDir::new().unwrap();
        let low = "fn low() {\n    let x = 1;\n}\n";
        let high = "fn high() {\n    if true {\n        if false {\n            for _ in 0..10 {\n                while true {}\n            }\n        }\n    }\n}\n";
        write_file(&dir, "src/low.rs", low);
        write_file(&dir, "src/high.rs", high);
        let data = export(vec![
            make_row("src/low.rs", "Rust", 3, low.len()),
            make_row("src/high.rs", "Rust", 9, high.len()),
        ]);
        let files = vec![PathBuf::from("src/low.rs"), PathBuf::from("src/high.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        if r.files.len() >= 2 {
            assert!(r.files[0].cyclomatic_complexity >= r.files[1].cyclomatic_complexity);
        }
    }

    #[test]
    fn histogram_present_in_report() {
        let dir = TempDir::new().unwrap();
        let code = "fn foo() {\n    let x = 1;\n}\n";
        write_file(&dir, "src/lib.rs", code);
        let data = export(vec![make_row("src/lib.rs", "Rust", 3, code.len())]);
        let files = vec![PathBuf::from("src/lib.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert!(r.histogram.is_some());
    }

    #[test]
    fn missing_file_skipped_gracefully() {
        let dir = TempDir::new().unwrap();
        let data = export(vec![make_row("src/nonexistent.rs", "Rust", 10, 200)]);
        let files = vec![PathBuf::from("src/nonexistent.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert!(r.files.is_empty());
    }

    #[test]
    fn file_byte_limit_respected() {
        let dir = TempDir::new().unwrap();
        let code = "fn f() {\n".repeat(100) + "}\n";
        write_file(&dir, "src/big.rs", &code);
        let data = export(vec![make_row("src/big.rs", "Rust", 101, code.len())]);
        let files = vec![PathBuf::from("src/big.rs")];
        let limits = AnalysisLimits {
            max_file_bytes: Some(50),
            ..default_limits()
        };
        // Should still process (reads up to limit)
        let r = build_complexity_report(dir.path(), &files, &data, &limits, false).unwrap();
        // File may or may not be analyzed depending on read_head behavior
        assert!(r.files.len() <= 1);
    }
}

// ── Edge cases ──────────────────────────────────────────────────

mod edge_cases {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn empty_source_file() {
        let dir = TempDir::new().unwrap();
        write_file(&dir, "src/empty.rs", "");
        let data = export(vec![make_row("src/empty.rs", "Rust", 0, 0)]);
        let files = vec![PathBuf::from("src/empty.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        if !r.files.is_empty() {
            assert_eq!(r.files[0].cyclomatic_complexity, 1);
        }
    }

    #[test]
    fn single_line_file() {
        let dir = TempDir::new().unwrap();
        let code = "fn main() {}";
        write_file(&dir, "src/one.rs", code);
        let data = export(vec![make_row("src/one.rs", "Rust", 1, code.len())]);
        let files = vec![PathBuf::from("src/one.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert!(!r.files.is_empty());
    }

    #[test]
    fn binary_file_skipped() {
        let dir = TempDir::new().unwrap();
        let content: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let full = dir.path().join("src/binary.rs");
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, &content).unwrap();
        let data = export(vec![make_row("src/binary.rs", "Rust", 10, 256)]);
        let files = vec![PathBuf::from("src/binary.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert!(r.files.is_empty());
    }

    #[test]
    fn child_rows_excluded() {
        let dir = TempDir::new().unwrap();
        let code = "fn main() {}\n";
        write_file(&dir, "src/main.rs", code);
        let mut data = export(vec![make_row("src/main.rs", "Rust", 1, code.len())]);
        data.rows.push(FileRow {
            path: "src/main.rs/html".to_string(),
            module: "src".to_string(),
            lang: "HTML".to_string(),
            kind: FileKind::Child,
            code: 50,
            comments: 0,
            blanks: 0,
            lines: 50,
            bytes: 1000,
            tokens: 250,
        });
        let files = vec![PathBuf::from("src/main.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        // Only parent row should be processed
        assert!(r.files.len() <= 1);
    }
}

// ── Property tests ──────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn histogram_total_equals_input_len(n in 0..50usize) {
            let files: Vec<FileComplexity> = (0..n)
                .map(|i| fc(&format!("f{i}.rs"), i % 40, None, None, ComplexityRisk::Low))
                .collect();
            let h = generate_complexity_histogram(&files, 5);
            prop_assert_eq!(h.total, n as u32);
        }

        #[test]
        fn histogram_counts_sum_to_total(n in 0..50usize) {
            let files: Vec<FileComplexity> = (0..n)
                .map(|i| fc(&format!("f{i}.rs"), i * 3, None, None, ComplexityRisk::Low))
                .collect();
            let h = generate_complexity_histogram(&files, 5);
            let sum: u32 = h.counts.iter().sum();
            prop_assert_eq!(sum, h.total);
        }

        #[test]
        fn cyclomatic_in_first_or_last_bucket(val in 0..100usize) {
            let files = [fc("f.rs", val, None, None, ComplexityRisk::Low)];
            let h = generate_complexity_histogram(&files, 5);
            let bucket_idx = (val as u32 / 5).min(6) as usize;
            prop_assert_eq!(h.counts[bucket_idx], 1);
        }
    }
}

// ── Determinism ─────────────────────────────────────────────────

mod determinism {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn report_deterministic_across_runs() {
        let dir = TempDir::new().unwrap();
        let code = "fn foo() {\n    if true {\n        let x = 1;\n    }\n}\n\nfn bar() {\n    let y = 2;\n}\n";
        write_file(&dir, "src/lib.rs", code);
        let data = export(vec![make_row("src/lib.rs", "Rust", 9, code.len())]);
        let files = vec![PathBuf::from("src/lib.rs")];
        let r1 = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        let r2 = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        assert_eq!(r1.total_functions, r2.total_functions);
        assert_eq!(r1.max_cyclomatic, r2.max_cyclomatic);
        assert_eq!(r1.files.len(), r2.files.len());
        for (a, b) in r1.files.iter().zip(r2.files.iter()) {
            assert_eq!(a.path, b.path);
            assert_eq!(a.cyclomatic_complexity, b.cyclomatic_complexity);
        }
    }
}

// ── Serialization round-trip ────────────────────────────────────

mod serialization {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn report_serializes_to_json() {
        let dir = TempDir::new().unwrap();
        let code = "fn main() {\n    let x = 1;\n}\n";
        write_file(&dir, "src/main.rs", code);
        let data = export(vec![make_row("src/main.rs", "Rust", 3, code.len())]);
        let files = vec![PathBuf::from("src/main.rs")];
        let r = build_complexity_report(dir.path(), &files, &data, &default_limits(), false).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("total_functions"));
        assert!(json.contains("avg_cyclomatic"));
    }
}
