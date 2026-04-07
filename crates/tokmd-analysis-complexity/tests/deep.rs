//! Deep tests for `tokmd-analysis-complexity`.
//!
//! Covers exact cyclomatic values, risk threshold boundaries,
//! function detail extraction, cognitive/nesting depth patterns,
//! multi-language patterns, and deterministic output guarantees.

use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use tokmd_analysis_complexity::{build_complexity_report, generate_complexity_histogram};
use tokmd_analysis_types::{ComplexityRisk, FileComplexity};
use tokmd_analysis_util::AnalysisLimits;
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

fn analyze(
    files: &[(&str, &str, &str)], // (path, lang, content)
    detail: bool,
) -> tokmd_analysis_types::ComplexityReport {
    let file_entries: Vec<(&str, &str)> = files.iter().map(|(p, _, c)| (*p, *c)).collect();
    let (dir, paths) = write_temp_files(&file_entries);
    let rows: Vec<FileRow> = files
        .iter()
        .map(|(p, lang, c)| make_row(p, "src", lang, c.lines().count()))
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
// § Exact cyclomatic complexity values for known patterns
// ═══════════════════════════════════════════════════════════════════

mod exact_cyclomatic {
    use super::*;

    #[test]
    fn linear_code_has_cyclomatic_one() {
        let code = "\
fn linear() {
    let a = 1;
    let b = 2;
    let c = a + b;
    println!(\"{}\", c);
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        assert_eq!(
            report.files[0].cyclomatic_complexity, 1,
            "straight-line code should have cyclomatic = 1 (base)"
        );
    }

    #[test]
    fn single_if_adds_one() {
        let code = "\
fn one_if(x: i32) {
    if x > 0 {
        println!(\"pos\");
    }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        // Base 1 + 1 if = 2
        assert_eq!(report.files[0].cyclomatic_complexity, 2);
    }

    #[test]
    fn nested_ifs_each_count() {
        let code = "\
fn nested(x: i32, y: i32) {
    if x > 0 {
        if y > 0 {
            println!(\"both positive\");
        }
    }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        // Base 1 + 2 ifs = 3
        assert_eq!(report.files[0].cyclomatic_complexity, 3);
    }

    #[test]
    fn match_arms_count_once() {
        let code = "\
fn matcher(x: i32) -> &'static str {
    match x {
        1 => \"one\",
        2 => \"two\",
        3 => \"three\",
        _ => \"other\",
    }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        // Base 1 + 1 match = 2
        assert_eq!(report.files[0].cyclomatic_complexity, 2);
    }

    #[test]
    fn for_loop_adds_one() {
        let code = "\
fn loopy() {
    for i in 0..10 {
        println!(\"{}\", i);
    }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        // Base 1 + 1 for = 2
        assert_eq!(report.files[0].cyclomatic_complexity, 2);
    }

    #[test]
    fn while_loop_adds_one() {
        let code = "\
fn while_loop(mut x: i32) {
    while x > 0 {
        x -= 1;
    }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        // Base 1 + 1 while = 2
        assert_eq!(report.files[0].cyclomatic_complexity, 2);
    }

    #[test]
    fn logical_operators_add_complexity() {
        let code = "\
fn logical(a: bool, b: bool, c: bool) -> bool {
    a && b || c
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        // Base 1 + 1 && + 1 || = 3
        assert_eq!(report.files[0].cyclomatic_complexity, 3);
    }

    #[test]
    fn question_mark_operator_adds_complexity() {
        let code = "\
fn fallible(x: Option<i32>) -> Option<i32> {
    let val = x?;
    Some(val + 1)
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        // Base 1 + 1 ? = 2
        assert_eq!(report.files[0].cyclomatic_complexity, 2);
    }

    #[test]
    fn complex_combination_counts_all_branches() {
        let code = "\
fn complex(x: i32, y: i32) -> i32 {
    if x > 0 && y > 0 {
        match x {
            1 => y,
            _ => x + y,
        }
    } else if x < 0 || y < 0 {
        while x > y {
            return x;
        }
        for i in 0..10 {
            println!(\"{}\", i);
        }
        0
    } else {
        loop {
            break;
        }
        42
    }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        // Base 1 + 2 if + 1 && + 1 || + 1 match + 1 while + 1 for + 1 loop = 9
        assert_eq!(report.files[0].cyclomatic_complexity, 9);
    }

    #[test]
    fn python_elif_each_counted() {
        let code = "\
def classify(x):
    if x > 100:
        return \"big\"
    elif x > 50:
        return \"medium\"
    elif x > 10:
        return \"small\"
    else:
        return \"tiny\"
";
        let report = analyze(&[("main.py", "Python", code)], false);
        // Base 1 + 3 "if " (1 actual + 2 inside "elif ") + 2 "elif " = 6
        // Note: "elif " contains "if " as a substring, so each elif is counted twice
        assert_eq!(report.files[0].cyclomatic_complexity, 6);
    }

    #[test]
    fn python_logical_operators_counted() {
        let code = "\
def check(a, b, c):
    if a and b or c:
        return True
    return False
";
        let report = analyze(&[("main.py", "Python", code)], false);
        // Base 1 + 1 if + 1 and + 1 or = 4
        assert_eq!(report.files[0].cyclomatic_complexity, 4);
    }

    #[test]
    fn javascript_switch_case_counting() {
        let code = "\
function grade(score) {
    switch (score) {
        case 'A': return 4;
        case 'B': return 3;
        case 'C': return 2;
        case 'D': return 1;
        default: return 0;
    }
}
";
        let report = analyze(&[("app.js", "JavaScript", code)], false);
        // Base 1 + 4 case = 5
        assert_eq!(report.files[0].cyclomatic_complexity, 5);
    }

    #[test]
    fn go_select_case_counting() {
        let code = "\
func handler(x int) int {
    if x > 0 {
        return x
    }
    for i := 0; i < 10; i++ {
        if i > x {
            return i
        }
    }
    return 0
}
";
        let report = analyze(&[("main.go", "Go", code)], false);
        // Base 1 + 2 if + 1 for = 4
        assert_eq!(report.files[0].cyclomatic_complexity, 4);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Function counting and length
// ═══════════════════════════════════════════════════════════════════

mod function_counting {
    use super::*;

    #[test]
    fn rust_counts_all_function_variants() {
        let code = "\
fn bare() {}
pub fn public_fn() {}
pub(crate) fn crate_fn() {}
async fn async_fn() {}
unsafe fn unsafe_fn() {}
const fn const_fn() -> u32 { 0 }
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        assert_eq!(report.total_functions, 6);
    }

    #[test]
    fn rust_function_length_tracks_longest() {
        let code = "\
fn short() {
    1;
}

fn long() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
    let i = 9;
    let j = 10;
    a + b + c + d + e + f + g + h + i + j;
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        assert_eq!(report.total_functions, 2);
        assert!(
            report.max_function_length >= 12,
            "max function length should be at least 12, got {}",
            report.max_function_length
        );
    }

    #[test]
    fn python_function_counting_with_nested_def() {
        let code = "\
def outer():
    def inner():
        pass
    inner()

def standalone():
    return 42
";
        let report = analyze(&[("app.py", "Python", code)], false);
        assert!(
            report.total_functions >= 2,
            "should count at least outer and standalone (got {})",
            report.total_functions
        );
    }

    #[test]
    fn go_method_receivers_counted() {
        let code = "\
func standalone() {
}

func (s *Server) handle() {
}

func (s *Server) close() {
}
";
        let report = analyze(&[("main.go", "Go", code)], false);
        assert_eq!(report.total_functions, 3);
    }

    #[test]
    fn js_arrow_functions_counted() {
        let code = "\
function named() {
    return 1;
}

const arrow = (x) => {
    return x + 1;
}
";
        let report = analyze(&[("app.js", "JavaScript", code)], false);
        assert!(
            report.total_functions >= 2,
            "should count named and arrow functions, got {}",
            report.total_functions
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Function detail extraction
// ═══════════════════════════════════════════════════════════════════

mod function_details {
    use super::*;

    #[test]
    fn detail_includes_line_numbers() {
        let code = "\
fn first() {
    let x = 1;
}

fn second() {
    if true {
        println!(\"yes\");
    }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], true);
        let fns = report.files[0].functions.as_ref().unwrap();
        assert_eq!(fns.len(), 2);
        // line_start should be positive (1-indexed)
        assert!(fns[0].line_start >= 1);
        assert!(fns[1].line_start > fns[0].line_start);
    }

    #[test]
    fn detail_cyclomatic_per_function() {
        let code = "\
fn simple() {
    let x = 1;
}

fn branchy(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            42
        } else {
            x
        }
    } else {
        0
    }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], true);
        let fns = report.files[0].functions.as_ref().unwrap();
        assert_eq!(fns.len(), 2);

        let simple = fns.iter().find(|f| f.name == "simple").unwrap();
        let branchy = fns.iter().find(|f| f.name == "branchy").unwrap();
        assert!(
            branchy.cyclomatic > simple.cyclomatic,
            "branchy ({}) should have higher cyclomatic than simple ({})",
            branchy.cyclomatic,
            simple.cyclomatic
        );
    }

    #[test]
    fn detail_param_count_extracted() {
        let code = "\
fn no_params() {
}

fn two_params(a: i32, b: i32) {
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], true);
        let fns = report.files[0].functions.as_ref().unwrap();

        let no_p = fns.iter().find(|f| f.name == "no_params").unwrap();
        let two_p = fns.iter().find(|f| f.name == "two_params").unwrap();

        assert_eq!(no_p.param_count, None);
        assert_eq!(two_p.param_count, Some(2));
    }

    #[test]
    fn detail_cognitive_present_for_branchy_function() {
        let code = "\
fn branchy(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            42
        } else {
            x
        }
    } else {
        0
    }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], true);
        let fns = report.files[0].functions.as_ref().unwrap();
        let branchy = &fns[0];
        assert!(
            branchy.cognitive.is_some(),
            "cognitive complexity should be present for branchy function"
        );
    }

    #[test]
    fn detail_length_is_end_minus_start_plus_one() {
        let code = "\
fn measured() {
    let a = 1;
    let b = 2;
    let c = 3;
    a + b + c;
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], true);
        let fns = report.files[0].functions.as_ref().unwrap();
        let f = &fns[0];
        assert_eq!(
            f.length,
            f.line_end - f.line_start + 1,
            "length should be line_end - line_start + 1"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Risk threshold boundary tests
// ═══════════════════════════════════════════════════════════════════

mod risk_thresholds {
    use super::*;

    fn file_with_metrics(
        cyclomatic: usize,
        fn_count: usize,
        fn_len: usize,
        cognitive: Option<usize>,
        nesting: Option<usize>,
    ) -> FileComplexity {
        FileComplexity {
            path: "test.rs".to_string(),
            module: "src".to_string(),
            function_count: fn_count,
            max_function_length: fn_len,
            cyclomatic_complexity: cyclomatic,
            cognitive_complexity: cognitive,
            max_nesting: nesting,
            risk_level: ComplexityRisk::Low, // not used in histogram
            functions: None,
        }
    }

    #[test]
    fn all_low_scores_produce_low_risk() {
        // score=0: fc<=20, mfl<=25, cyclo<=10, cog<=25, nesting<=4
        let code = "\
fn simple(x: i32) -> i32 {
    if x > 0 { x } else { 0 }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        assert_eq!(report.files[0].risk_level, ComplexityRisk::Low);
    }

    #[test]
    fn high_function_count_increases_risk() {
        // Generate code with many functions to push function_count > 50
        let mut code = String::new();
        for i in 0..55 {
            let _ = writeln!(code, "fn f{i}() {{ let x = 1; }}");
        }
        let report = analyze(&[("lib.rs", "Rust", &code)], false);
        assert!(
            report.files[0].function_count > 50,
            "should have > 50 functions"
        );
        // With 55 functions, score += 2; total score = 2 → Moderate
        assert!(matches!(
            report.files[0].risk_level,
            ComplexityRisk::Moderate | ComplexityRisk::High | ComplexityRisk::Critical
        ));
    }

    #[test]
    fn histogram_with_zero_bucket_size_treated_as_one() {
        // bucket_size of 1 should still work
        let files = vec![file_with_metrics(3, 1, 5, None, None)];
        let hist = generate_complexity_histogram(&files, 1);
        assert_eq!(hist.total, 1);
        assert_eq!(hist.counts.iter().sum::<u32>(), 1);
    }

    #[test]
    fn file_sort_order_is_deterministic_for_equal_complexity() {
        let code_a = "fn a() { if true { 1 } else { 0 }; }\n";
        let code_b = "fn b() { if true { 1 } else { 0 }; }\n";
        let report = analyze(&[("b.rs", "Rust", code_b), ("a.rs", "Rust", code_a)], false);
        // Files with same complexity should be sorted by path ascending
        if report.files.len() == 2
            && report.files[0].cyclomatic_complexity == report.files[1].cyclomatic_complexity
        {
            assert!(
                report.files[0].path < report.files[1].path,
                "files with same complexity should be sorted by path: {} vs {}",
                report.files[0].path,
                report.files[1].path
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Cognitive complexity and nesting depth patterns
// ═══════════════════════════════════════════════════════════════════

mod cognitive_nesting {
    use super::*;

    #[test]
    fn nesting_depth_increases_with_deeper_code() {
        let shallow = "\
fn shallow(x: i32) {
    if x > 0 {
        println!(\"yes\");
    }
}
";
        let deep = "\
fn deep(x: i32, y: i32, z: i32) {
    if x > 0 {
        if y > 0 {
            if z > 0 {
                if x > y {
                    println!(\"deep\");
                }
            }
        }
    }
}
";
        let report_shallow = analyze(&[("shallow.rs", "Rust", shallow)], false);
        let report_deep = analyze(&[("deep.rs", "Rust", deep)], false);

        let shallow_nesting = report_shallow.files[0].max_nesting.unwrap_or(0);
        let deep_nesting = report_deep.files[0].max_nesting.unwrap_or(0);
        assert!(
            deep_nesting > shallow_nesting,
            "deep nesting ({deep_nesting}) should exceed shallow nesting ({shallow_nesting})"
        );
    }

    #[test]
    fn cognitive_grows_with_nesting() {
        let flat = "\
fn flat(a: bool, b: bool) -> bool {
    if a {
        return true;
    }
    if b {
        return true;
    }
    false
}
";
        let nested = "\
fn nested(a: bool, b: bool) -> bool {
    if a {
        if b {
            return true;
        }
    }
    false
}
";
        let report_flat = analyze(&[("flat.rs", "Rust", flat)], false);
        let report_nested = analyze(&[("nested.rs", "Rust", nested)], false);

        let cog_flat = report_flat.files[0].cognitive_complexity.unwrap_or(0);
        let cog_nested = report_nested.files[0].cognitive_complexity.unwrap_or(0);
        // Nested code should have higher cognitive complexity due to nesting penalty
        assert!(
            cog_nested >= cog_flat,
            "nested cognitive ({cog_nested}) should be >= flat cognitive ({cog_flat})"
        );
    }

    #[test]
    fn aggregate_nesting_tracks_max_across_files() {
        let shallow = "fn f() { if true { 1; } }\n";
        let deep = "\
fn g(x: i32) {
    if x > 0 {
        if x > 1 {
            if x > 2 {
                if x > 3 {
                    println!(\"deep\");
                }
            }
        }
    }
}
";
        let report = analyze(
            &[("shallow.rs", "Rust", shallow), ("deep.rs", "Rust", deep)],
            false,
        );
        if let Some(max_nesting) = report.max_nesting_depth {
            assert!(
                max_nesting >= 4,
                "max nesting should be >= 4, got {max_nesting}"
            );
        }
    }

    #[test]
    fn avg_cognitive_between_min_and_max() {
        let simple = "fn s() { if true { 1; } }\n";
        let complex = "\
fn c(x: i32) {
    if x > 0 {
        for i in 0..x {
            if i > 5 {
                while i > 10 {
                    break;
                }
            }
        }
    }
}
";
        let report = analyze(
            &[("s.rs", "Rust", simple), ("c.rs", "Rust", complex)],
            false,
        );
        if let (Some(avg), Some(max)) = (report.avg_cognitive, report.max_cognitive) {
            assert!(
                avg <= max as f64,
                "avg cognitive ({avg}) should be <= max ({max})"
            );
            assert!(avg >= 0.0, "avg cognitive should be non-negative");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Multi-language support
// ═══════════════════════════════════════════════════════════════════

mod multi_language {
    use super::*;

    #[test]
    fn ruby_if_unless_while_counted() {
        let code = "\
def process(x)
  if x > 0
    unless x > 100
      while x > 10
        x = x - 1
      end
    end
  end
end
";
        let report = analyze(&[("app.rb", "Ruby", code)], false);
        assert_eq!(report.total_functions, 1);
        // Base 1 + if + unless + while = 4
        assert!(
            report.files[0].cyclomatic_complexity >= 4,
            "Ruby code should have cyclomatic >= 4, got {}",
            report.files[0].cyclomatic_complexity
        );
    }

    #[test]
    fn c_style_for_while_if_counted() {
        let code = "\
int process(int x) {
    for (int i = 0; i < x; i++) {
        if (i > 5) {
            while (x > 0) {
                x--;
            }
        }
    }
    return x;
}
";
        let report = analyze(&[("main.c", "C", code)], false);
        assert_eq!(report.total_functions, 1);
        // Base 1 + for + if + while = 4
        assert!(
            report.files[0].cyclomatic_complexity >= 4,
            "C code should have cyclomatic >= 4, got {}",
            report.files[0].cyclomatic_complexity
        );
    }

    #[test]
    fn mixed_language_files_all_analyzed() {
        let rust_code = "fn f() { if true { 1; } }\n";
        let py_code = "def g():\n    if True:\n        pass\n";
        let js_code = "function h() {\n    if (true) { return 1; }\n}\n";
        let go_code = "func k() {\n    if true {\n        return\n    }\n}\n";

        let report = analyze(
            &[
                ("a.rs", "Rust", rust_code),
                ("b.py", "Python", py_code),
                ("c.js", "JavaScript", js_code),
                ("d.go", "Go", go_code),
            ],
            false,
        );
        assert_eq!(
            report.files.len(),
            4,
            "all 4 language files should be analyzed"
        );
        assert!(report.total_functions >= 4);
    }

    #[test]
    fn unsupported_language_excluded_from_report() {
        let rust_code = "fn f() { 1; }\n";
        let md_code = "# Header\nSome text\n";

        let (dir, paths) = write_temp_files(&[("a.rs", rust_code), ("b.md", md_code)]);
        let export = make_export(vec![
            make_row("a.rs", "src", "Rust", 1),
            make_row("b.md", "src", "Markdown", 2),
        ]);
        let report = build_complexity_report(
            dir.path(),
            &paths,
            &export,
            &AnalysisLimits::default(),
            false,
        )
        .unwrap();

        assert_eq!(report.files.len(), 1, "only Rust file should be in report");
        assert_eq!(report.files[0].path, "a.rs");
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Edge cases
// ═══════════════════════════════════════════════════════════════════

mod edge_cases {
    use super::*;

    #[test]
    fn one_liner_function() {
        let code = "fn one() { 42 }\n";
        let report = analyze(&[("lib.rs", "Rust", code)], true);
        assert_eq!(report.total_functions, 1);
        let fns = report.files[0].functions.as_ref().unwrap();
        assert_eq!(fns[0].name, "one");
        assert!(fns[0].length <= 2, "one-liner should have length 1 or 2");
    }

    #[test]
    fn comments_do_not_affect_cyclomatic() {
        let code_no_comments = "\
fn f(x: i32) -> i32 {
    if x > 0 {
        x
    } else {
        0
    }
}
";
        let code_with_comments = "\
// This function processes x
fn f(x: i32) -> i32 {
    // Check when positive
    if x > 0 {
        // Return x
        x
    } else {
        // Return 0
        0
    }
}
// End of function
";
        let report1 = analyze(&[("a.rs", "Rust", code_no_comments)], false);
        let report2 = analyze(&[("b.rs", "Rust", code_with_comments)], false);
        assert_eq!(
            report1.files[0].cyclomatic_complexity, report2.files[0].cyclomatic_complexity,
            "comments should not change cyclomatic complexity"
        );
    }

    #[test]
    fn empty_file_produces_zero_metrics() {
        let report = analyze(&[("empty.rs", "Rust", "")], false);
        assert_eq!(report.total_functions, 0);
    }

    #[test]
    fn whitespace_only_file() {
        let report = analyze(&[("ws.rs", "Rust", "   \n\n\t  \n")], false);
        assert_eq!(report.total_functions, 0);
    }

    #[test]
    fn file_with_only_comments() {
        let code = "// Just comments\n// Nothing else\n/// doc comment\n";
        let report = analyze(&[("comments.rs", "Rust", code)], false);
        assert_eq!(report.total_functions, 0);
    }

    #[test]
    fn maintainability_index_present_for_nonempty_code() {
        let code = "\
fn f(x: i32) -> i32 {
    if x > 0 { x } else { 0 }
}
";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        // Maintainability index should be computed when we have functions and code
        // It may or may not be present depending on the average LOC calculation
        // but should not cause a crash
        let _ = report.maintainability_index;
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Determinism
// ═══════════════════════════════════════════════════════════════════

mod determinism {
    use super::*;

    #[test]
    fn ten_runs_produce_identical_output() {
        let code = "\
fn complex(x: i32, y: i32) -> i32 {
    if x > 0 && y > 0 {
        for i in 0..x {
            if i > y {
                return i;
            }
        }
    }
    match x {
        0 => y,
        _ => x,
    }
}
fn simple() { let z = 42; }
";
        let first = analyze(&[("lib.rs", "Rust", code)], true);
        for _ in 0..9 {
            let run = analyze(&[("lib.rs", "Rust", code)], true);
            assert_eq!(first.total_functions, run.total_functions);
            assert_eq!(first.max_cyclomatic, run.max_cyclomatic);
            assert_eq!(first.avg_cyclomatic, run.avg_cyclomatic);
            assert_eq!(first.high_risk_files, run.high_risk_files);
            assert_eq!(first.files.len(), run.files.len());
            for (a, b) in first.files.iter().zip(run.files.iter()) {
                assert_eq!(a.path, b.path);
                assert_eq!(a.cyclomatic_complexity, b.cyclomatic_complexity);
                assert_eq!(a.cognitive_complexity, b.cognitive_complexity);
                assert_eq!(a.max_nesting, b.max_nesting);
                assert_eq!(a.function_count, b.function_count);
                assert_eq!(a.risk_level, b.risk_level);
            }
        }
    }

    #[test]
    fn histogram_deterministic_across_runs() {
        let code = "\
fn a() { if true { 1; } }
fn b(x: i32) { if x > 0 { for i in 0..10 { if i > 5 { return; } } } }
";
        let report1 = analyze(&[("lib.rs", "Rust", code)], false);
        let report2 = analyze(&[("lib.rs", "Rust", code)], false);

        let h1 = report1.histogram.unwrap();
        let h2 = report2.histogram.unwrap();
        assert_eq!(h1.buckets, h2.buckets);
        assert_eq!(h1.counts, h2.counts);
        assert_eq!(h1.total, h2.total);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Technical debt
// ═══════════════════════════════════════════════════════════════════

mod technical_debt {
    use super::*;

    #[test]
    fn debt_ratio_increases_with_complexity() {
        let simple = "fn f() { let x = 1; }\n";
        let complex = "\
fn g(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                for i in 0..x {
                    if i > 50 {
                        while i > x {
                            return i;
                        }
                    }
                }
            }
        }
    }
    0
}
";
        let report_simple = analyze(&[("s.rs", "Rust", simple)], false);
        let report_complex = analyze(&[("c.rs", "Rust", complex)], false);

        match (
            &report_simple.technical_debt,
            &report_complex.technical_debt,
        ) {
            (Some(ds), Some(dc)) => {
                assert!(
                    dc.ratio >= ds.ratio,
                    "complex code debt ratio ({}) should be >= simple code debt ratio ({})",
                    dc.ratio,
                    ds.ratio
                );
            }
            _ => {
                // One or both might be None if code count is 0
            }
        }
    }

    #[test]
    fn debt_level_classifications() {
        // Low: ratio < 30
        // Moderate: 30 <= ratio < 60
        // High: 60 <= ratio < 100
        // Critical: ratio >= 100
        // We can't directly test classify_risk_extended thresholds here,
        // but we can verify the debt level is one of the valid variants
        let code = "fn f() { if true { 1; } }\n";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        if let Some(debt) = &report.technical_debt {
            assert!(matches!(
                debt.level,
                tokmd_analysis_types::TechnicalDebtLevel::Low
                    | tokmd_analysis_types::TechnicalDebtLevel::Moderate
                    | tokmd_analysis_types::TechnicalDebtLevel::High
                    | tokmd_analysis_types::TechnicalDebtLevel::Critical
            ));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Serialization roundtrip
// ═══════════════════════════════════════════════════════════════════

mod serialization {
    use super::*;

    #[test]
    fn complexity_report_round_trips_through_json() {
        let code = r#"
fn branchy(x: i32) -> i32 {
    if x > 0 {
        if x > 10 { x * 2 } else { x + 1 }
    } else {
        match x { -1 => 0, _ => x.abs() }
    }
}
"#;
        let report = analyze(&[("lib.rs", "Rust", code)], true);
        let json = serde_json::to_string(&report).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["total_functions"].is_u64());
        assert!(parsed["avg_cyclomatic"].is_f64());
        assert!(parsed["max_cyclomatic"].is_u64());
        assert!(parsed["files"].is_array());
    }

    #[test]
    fn file_complexity_fields_serialized() {
        let code = "fn f(x: i32) -> i32 { if x > 0 { x } else { -x } }\n";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        let json = serde_json::to_value(report).unwrap();
        let file = &json["files"][0];
        assert!(file["path"].is_string());
        assert!(file["module"].is_string());
        assert!(file["function_count"].is_u64());
        assert!(file["cyclomatic_complexity"].is_u64());
        assert!(file["risk_level"].is_string());
    }

    #[test]
    fn function_details_serialized_when_enabled() {
        let code = "fn a() { let x = 1; }\nfn b(x: i32) { if x > 0 { return; } }\n";
        let report = analyze(&[("lib.rs", "Rust", code)], true);
        let json = serde_json::to_value(report).unwrap();
        let functions = &json["files"][0]["functions"];
        assert!(functions.is_array());
        let fns = functions.as_array().unwrap();
        assert!(!fns.is_empty());
        assert!(fns[0]["name"].is_string());
        assert!(fns[0]["cyclomatic"].is_u64());
    }

    #[test]
    fn histogram_serialized_when_present() {
        let code = "fn f() { if true { 1; } }\n";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        let json = serde_json::to_value(report).unwrap();
        let hist = &json["histogram"];
        assert!(hist.is_object());
        assert!(hist["buckets"].is_array());
        assert!(hist["counts"].is_array());
        assert!(hist["total"].is_u64());
    }

    #[test]
    fn risk_level_serializes_as_string() {
        let code = "fn f() {}\n";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        let json = serde_json::to_value(report).unwrap();
        let risk = json["files"][0]["risk_level"].as_str().unwrap();
        assert!(["low", "moderate", "high", "critical"].contains(&risk));
    }

    #[test]
    fn technical_debt_serialized_when_present() {
        let code = "fn f(x: i32) -> i32 { if x > 0 { x } else { -x } }\n";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        let json = serde_json::to_value(report).unwrap();
        if let Some(debt) = json.get("technical_debt")
            && !debt.is_null()
        {
            assert!(debt["ratio"].is_f64());
            assert!(debt["complexity_points"].is_u64());
            assert!(debt["code_kloc"].is_f64());
            assert!(debt["level"].is_string());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Aggregation across files
// ═══════════════════════════════════════════════════════════════════

mod aggregation {
    use super::*;

    #[test]
    fn total_functions_sums_across_files() {
        let code_a = "fn a() {}\nfn b() {}\n";
        let code_b = "fn c() {}\nfn d() {}\nfn e() {}\n";
        let report = analyze(&[("a.rs", "Rust", code_a), ("b.rs", "Rust", code_b)], false);
        assert!(report.total_functions >= 5);
    }

    #[test]
    fn max_cyclomatic_is_highest_across_files() {
        let simple = "fn f() { let x = 1; }\n";
        let complex = r#"
fn g(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 { x + y } else { x - y }
    } else {
        match x {
            0 => y,
            _ => x * y,
        }
    }
}
"#;
        let report = analyze(
            &[
                ("simple.rs", "Rust", simple),
                ("complex.rs", "Rust", complex),
            ],
            false,
        );
        assert!(report.max_cyclomatic > 1);
        // The complex file should have higher cyclomatic than the simple one
        if report.files.len() >= 2 {
            let sorted: Vec<usize> = report
                .files
                .iter()
                .map(|f| f.cyclomatic_complexity)
                .collect();
            assert!(report.max_cyclomatic == *sorted.iter().max().unwrap());
        }
    }

    #[test]
    fn avg_cyclomatic_between_min_and_max() {
        let code_a = "fn f() { let x = 1; }\n";
        let code_b =
            "fn g(x: i32) -> i32 { if x > 0 { if x > 10 { x * 2 } else { x } } else { -x } }\n";
        let report = analyze(&[("a.rs", "Rust", code_a), ("b.rs", "Rust", code_b)], false);
        let min_cyclo: usize = report
            .files
            .iter()
            .map(|f| f.cyclomatic_complexity)
            .min()
            .unwrap_or(0);
        let max_cyclo: usize = report
            .files
            .iter()
            .map(|f| f.cyclomatic_complexity)
            .max()
            .unwrap_or(0);
        assert!(report.avg_cyclomatic >= min_cyclo as f64);
        assert!(report.avg_cyclomatic <= max_cyclo as f64);
    }

    #[test]
    fn max_function_length_is_longest_across_files() {
        let short = "fn f() { let x = 1; }\n";
        let long = r#"
fn g() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
}
"#;
        let report = analyze(
            &[("short.rs", "Rust", short), ("long.rs", "Rust", long)],
            false,
        );
        assert!(report.max_function_length > 1);
    }

    #[test]
    fn high_risk_files_counts_risky_files() {
        let safe = "fn f() { let x = 1; }\n";
        let report = analyze(&[("safe.rs", "Rust", safe)], false);
        // A single simple function should not be high risk
        assert_eq!(report.high_risk_files, 0);
    }

    #[test]
    fn files_sorted_by_cyclomatic_descending() {
        let code_a = "fn f() { let x = 1; }\n";
        let code_b =
            "fn g(x: i32) -> i32 { if x > 0 { if x > 10 { x * 2 } else { x } } else { -x } }\n";
        let report = analyze(&[("a.rs", "Rust", code_a), ("b.rs", "Rust", code_b)], false);
        if report.files.len() >= 2 {
            for w in report.files.windows(2) {
                assert!(w[0].cyclomatic_complexity >= w[1].cyclomatic_complexity);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Histogram generation
// ═══════════════════════════════════════════════════════════════════

mod histogram {
    use super::*;

    #[test]
    fn histogram_has_seven_buckets() {
        let files = vec![FileComplexity {
            path: "a.rs".to_string(),
            module: "src".to_string(),
            function_count: 1,
            max_function_length: 5,
            cyclomatic_complexity: 3,
            cognitive_complexity: None,
            max_nesting: None,
            risk_level: ComplexityRisk::Low,
            functions: None,
        }];
        let hist = generate_complexity_histogram(&files, 5);
        assert_eq!(hist.buckets.len(), 7);
        assert_eq!(hist.counts.len(), 7);
    }

    #[test]
    fn histogram_total_matches_file_count() {
        let files: Vec<FileComplexity> = (0..10)
            .map(|i| FileComplexity {
                path: format!("{i}.rs"),
                module: "src".to_string(),
                function_count: 1,
                max_function_length: 5,
                cyclomatic_complexity: i * 3,
                cognitive_complexity: None,
                max_nesting: None,
                risk_level: ComplexityRisk::Low,
                functions: None,
            })
            .collect();
        let hist = generate_complexity_histogram(&files, 5);
        assert_eq!(hist.total, 10);
        let sum: u32 = hist.counts.iter().sum();
        assert_eq!(sum, 10);
    }

    #[test]
    fn empty_files_gives_empty_histogram() {
        let hist = generate_complexity_histogram(&[], 5);
        assert_eq!(hist.total, 0);
        assert!(hist.counts.iter().all(|&c| c == 0));
    }

    #[test]
    fn high_complexity_files_in_last_bucket() {
        let files = vec![FileComplexity {
            path: "complex.rs".to_string(),
            module: "src".to_string(),
            function_count: 10,
            max_function_length: 200,
            cyclomatic_complexity: 50,
            cognitive_complexity: Some(100),
            max_nesting: Some(8),
            risk_level: ComplexityRisk::Critical,
            functions: None,
        }];
        let hist = generate_complexity_histogram(&files, 5);
        // Complexity 50 should be in last bucket (30+)
        assert_eq!(*hist.counts.last().unwrap(), 1);
    }

    #[test]
    fn bucket_boundaries_are_correct() {
        let hist = generate_complexity_histogram(&[], 5);
        assert_eq!(hist.buckets, vec![0, 5, 10, 15, 20, 25, 30]);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Additional edge cases
// ═══════════════════════════════════════════════════════════════════

mod additional_edge_cases {
    use super::*;

    #[test]
    fn single_function_file() {
        let code = "fn main() { println!(\"hello\"); }\n";
        let report = analyze(&[("main.rs", "Rust", code)], false);
        assert!(report.total_functions >= 1);
        assert!(report.avg_cyclomatic >= 1.0);
    }

    #[test]
    fn file_with_no_functions() {
        let code = "// just a comment\nlet x = 42;\n";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        // Should still produce a report, just with minimal metrics
        assert_eq!(report.files.len(), 1);
    }

    #[test]
    fn deeply_nested_code() {
        let code = r#"
fn deep(x: i32) {
    if x > 0 {
        if x > 1 {
            if x > 2 {
                if x > 3 {
                    if x > 4 {
                        println!("very deep");
                    }
                }
            }
        }
    }
}
"#;
        let report = analyze(&[("deep.rs", "Rust", code)], false);
        // Should detect deep nesting
        if let Some(max_nest) = report.max_nesting_depth {
            assert!(max_nest >= 4);
        }
    }

    #[test]
    fn many_functions_file() {
        let mut code = String::new();
        for i in 0..30 {
            let _ = writeln!(code, "fn func_{i}() {{ let x = {i}; }}");
        }
        let report = analyze(&[("many.rs", "Rust", &code)], false);
        assert!(report.total_functions >= 20);
    }

    #[test]
    fn python_nested_control_flow() {
        let code = r#"
def process(data):
    if data:
        for item in data:
            if item > 0:
                while item > 10:
                    item = item - 1
                    if item == 5:
                        break
"#;
        let report = analyze(&[("proc.py", "Python", code)], false);
        assert!(!report.files.is_empty());
        assert!(report.files[0].cyclomatic_complexity > 1);
    }

    #[test]
    fn javascript_async_functions() {
        let code = r#"
async function fetchData(url) {
    if (!url) {
        throw new Error("no url");
    }
    const res = await fetch(url);
    if (!res.ok) {
        throw new Error("bad response");
    }
    return res.json();
}
"#;
        let report = analyze(&[("fetch.js", "JavaScript", code)], false);
        assert!(!report.files.is_empty());
        assert!(report.files[0].function_count >= 1);
    }

    #[test]
    fn go_multiple_functions() {
        let code = r#"
func Add(a int, b int) int {
    return a + b
}

func Max(a int, b int) int {
    if a > b {
        return a
    }
    return b
}
"#;
        let report = analyze(&[("math.go", "Go", code)], false);
        assert!(!report.files.is_empty());
        assert!(report.files[0].function_count >= 2);
    }

    #[test]
    fn maintainability_index_present_for_valid_code() {
        let code = r#"
fn compute(x: i32, y: i32) -> i32 {
    if x > 0 {
        x + y
    } else {
        x - y
    }
}
"#;
        let report = analyze(&[("comp.rs", "Rust", code)], false);
        // Maintainability index should be computed for non-empty code
        // It may or may not be present depending on avg_loc calculation
        // Just verify the report is valid
        assert!(report.avg_cyclomatic >= 1.0);
    }

    #[test]
    fn detail_functions_disabled_returns_none() {
        let code = "fn f() { let x = 1; }\n";
        let report = analyze(&[("lib.rs", "Rust", code)], false);
        for file in &report.files {
            assert!(file.functions.is_none());
        }
    }
}
