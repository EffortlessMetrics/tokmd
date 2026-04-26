//! Edge-case and language-coverage tests for `analysis complexity module`.
//!
//! Supplements the existing BDD, integration, and property tests with
//! scenarios for deeply nested code, additional languages, and report
//! aggregate invariants.

use std::fs;
use std::path::PathBuf;

use crate::complexity::build_complexity_report;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::ComplexityRisk;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn make_row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
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

// ── Deeply nested code ──────────────────────────────────────────

mod deeply_nested {
    use super::*;

    #[test]
    fn given_deeply_nested_rust_when_analyzed_then_high_cyclomatic() {
        let code = "\
fn deep(x: i32, y: i32, z: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            if z > 0 {
                if x > y {
                    if y > z {
                        for i in 0..x {
                            if i % 2 == 0 {
                                return i;
                            }
                        }
                    }
                }
            } else {
                match z {
                    0 => return 0,
                    1 => return 1,
                    _ => return -1,
                }
            }
        }
    }
    42
}
";
        let (dir, paths) = write_temp_files(&[("nested.rs", code)]);
        let export = make_export(vec![make_row("nested.rs", "src", "Rust", 25)]);
        let report = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            false,
        )
        .unwrap();

        assert_eq!(report.files.len(), 1);
        assert!(
            report.files[0].cyclomatic_complexity >= 7,
            "deeply nested code should have high cyclomatic, got {}",
            report.files[0].cyclomatic_complexity
        );
        assert!(matches!(
            report.files[0].risk_level,
            ComplexityRisk::Moderate | ComplexityRisk::High | ComplexityRisk::Critical
        ));
    }

    #[test]
    fn given_deeply_nested_rust_when_analyzed_then_nesting_depth_detected() {
        let code = "\
fn deep_nest() {
    if true {
        if true {
            if true {
                if true {
                    println!(\"deep\");
                }
            }
        }
    }
}
";
        let (dir, paths) = write_temp_files(&[("deep.rs", code)]);
        let export = make_export(vec![make_row("deep.rs", "src", "Rust", 12)]);
        let report = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            false,
        )
        .unwrap();

        assert_eq!(report.files.len(), 1);
        if let Some(nesting) = report.files[0].max_nesting {
            assert!(nesting >= 4, "expected nesting depth >= 4, got {nesting}");
        }
    }
}

// ── Ruby and C# language support ────────────────────────────────

mod language_support {
    use super::*;

    #[test]
    fn given_ruby_file_when_analyzed_then_functions_detected() {
        let code = "\
def greet(name)
  if name
    puts \"Hello, #{name}!\"
  else
    puts \"Hello!\"
  end
end

def add(a, b)
  a + b
end
";
        let (dir, paths) = write_temp_files(&[("app.rb", code)]);
        let export = make_export(vec![make_row("app.rb", ".", "Ruby", 12)]);
        let report = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            false,
        )
        .unwrap();

        assert_eq!(report.total_functions, 2);
        assert_eq!(report.files.len(), 1);
    }

    #[test]
    fn given_c_sharp_file_when_analyzed_then_functions_detected() {
        let code = "\
public class Program {
    static void Main(string[] args) {
        if (args.Length > 0) {
            Console.WriteLine(args[0]);
        }
    }

    static int Add(int a, int b) {
        return a + b;
    }
}
";
        let (dir, paths) = write_temp_files(&[("Program.cs", code)]);
        let export = make_export(vec![make_row("Program.cs", ".", "C#", 12)]);
        let report = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            false,
        )
        .unwrap();

        assert!(
            report.total_functions >= 2,
            "C# file should detect functions, got {}",
            report.total_functions
        );
    }

    #[test]
    fn given_typescript_file_when_analyzed_then_functions_detected() {
        let code = "\
function greet(name: string): void {
    if (name) {
        console.log(`Hello, ${name}!`);
    } else {
        console.log('Hello!');
    }
}

function add(a: number, b: number): number {
    return a + b;
}
";
        let (dir, paths) = write_temp_files(&[("app.ts", code)]);
        let export = make_export(vec![make_row("app.ts", ".", "TypeScript", 12)]);
        let report = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            false,
        )
        .unwrap();

        assert_eq!(report.total_functions, 2);
        assert!(report.avg_cyclomatic > 0.0);
    }
}

// ── Aggregate invariants ────────────────────────────────────────

mod aggregate_invariants {
    use super::*;

    #[test]
    fn given_multiple_files_when_analyzed_then_avg_cyclomatic_between_min_and_max() {
        let code_simple = "fn simple() { let x = 1; }\n";
        let code_branchy = "\
fn branchy(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            for i in 0..x {
                if i % 2 == 0 { return i; }
            }
        }
    }
    0
}
";
        let (dir, paths) =
            write_temp_files(&[("simple.rs", code_simple), ("branchy.rs", code_branchy)]);
        let export = make_export(vec![
            make_row("simple.rs", "src", "Rust", 1),
            make_row("branchy.rs", "src", "Rust", 12),
        ]);
        let report = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            false,
        )
        .unwrap();

        let min_cyclo = report
            .files
            .iter()
            .map(|f| f.cyclomatic_complexity)
            .min()
            .unwrap_or(0);
        let max_cyclo = report.max_cyclomatic;
        assert!(
            report.avg_cyclomatic >= min_cyclo as f64,
            "avg {} should be >= min {}",
            report.avg_cyclomatic,
            min_cyclo
        );
        assert!(
            report.avg_cyclomatic <= max_cyclo as f64,
            "avg {} should be <= max {}",
            report.avg_cyclomatic,
            max_cyclo
        );
    }

    #[test]
    fn given_files_when_analyzed_then_high_risk_count_le_total_files() {
        let code = "\
fn moderate(x: i32) -> i32 {
    if x > 0 { if x > 10 { 42 } else { x } } else { 0 }
}
";
        let (dir, paths) = write_temp_files(&[("a.rs", code), ("b.rs", code)]);
        let export = make_export(vec![
            make_row("a.rs", "src", "Rust", 3),
            make_row("b.rs", "src", "Rust", 3),
        ]);
        let report = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            false,
        )
        .unwrap();

        assert!(report.high_risk_files <= report.files.len());
    }
}

// ── Determinism ─────────────────────────────────────────────────

mod determinism {
    use super::*;

    #[test]
    fn given_same_input_when_analyzed_twice_then_output_is_identical() {
        let code = "\
fn branchy(x: i32) -> i32 {
    if x > 0 { if x > 10 { 42 } else { x } } else { 0 }
}
fn simple() { let y = 1; }
";
        let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
        let export = make_export(vec![make_row("lib.rs", "src", "Rust", 5)]);

        let r1 = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            true,
        )
        .unwrap();
        let r2 = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            true,
        )
        .unwrap();

        assert_eq!(r1.total_functions, r2.total_functions);
        assert_eq!(r1.max_cyclomatic, r2.max_cyclomatic);
        assert_eq!(r1.avg_cyclomatic, r2.avg_cyclomatic);
        assert_eq!(r1.files.len(), r2.files.len());
        for (f1, f2) in r1.files.iter().zip(r2.files.iter()) {
            assert_eq!(f1.path, f2.path);
            assert_eq!(f1.cyclomatic_complexity, f2.cyclomatic_complexity);
            assert_eq!(f1.function_count, f2.function_count);
        }
    }
}

// ── Empty / minimal ─────────────────────────────────────────────

mod empty_minimal {
    use super::*;

    #[test]
    fn given_file_with_no_functions_when_analyzed_then_zero_function_count() {
        let code = "// Just a comment, no functions\nlet x = 42;\n";
        let (dir, paths) = write_temp_files(&[("nofn.rs", code)]);
        let export = make_export(vec![make_row("nofn.rs", "src", "Rust", 2)]);
        let report = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            false,
        )
        .unwrap();

        // File is analyzed but has no functions
        if !report.files.is_empty() {
            assert_eq!(report.files[0].function_count, 0);
        }
    }

    #[test]
    fn given_empty_file_when_analyzed_then_no_crash() {
        let (dir, paths) = write_temp_files(&[("empty.rs", "")]);
        let export = make_export(vec![make_row("empty.rs", "src", "Rust", 0)]);
        let report = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            false,
        )
        .unwrap();

        // Should not panic; may or may not include the file
        assert!(report.total_functions == 0);
    }
}
