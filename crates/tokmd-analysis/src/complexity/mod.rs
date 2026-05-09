use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::maintainability::compute_maintainability_index;
use anyhow::Result;
#[cfg(test)]
use tokmd_analysis_types::TechnicalDebtLevel;
use tokmd_analysis_types::{ComplexityReport, ComplexityRisk, FileComplexity};
use tokmd_types::{ExportData, FileKind, FileRow};

use tokmd_analysis_types::{AnalysisLimits, normalize_path};

mod debt;
mod details;
mod functions;
mod histogram;
mod risk;

use debt::{average_parent_loc, compute_technical_debt_ratio};
use details::extract_function_details;
#[cfg(test)]
use details::{detect_fn_spans_c_style, detect_fn_spans_python, detect_fn_spans_rust};
use functions::count_functions;
#[cfg(test)]
use functions::{count_python_functions, count_rust_functions, is_rust_fn_start};
pub(crate) use histogram::generate_complexity_histogram;
use risk::{classify_risk_extended, estimate_cyclomatic};

const DEFAULT_MAX_FILE_BYTES: u64 = 128 * 1024;
const MAX_COMPLEXITY_FILES: usize = 100;

/// Map language strings to complexity-compatible names.
fn map_language_for_complexity(lang: &str) -> &str {
    match lang.to_lowercase().as_str() {
        "rust" => "rust",
        "javascript" | "jsx" => "javascript",
        "typescript" | "tsx" => "typescript",
        "python" => "python",
        "go" => "go",
        "c" => "c",
        "c++" | "cpp" => "c++",
        "java" => "java",
        "c#" | "csharp" => "c#",
        "php" => "php",
        "ruby" => "ruby",
        _ => lang,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod moved_tests;

/// Languages that support complexity analysis.
fn is_complexity_lang(lang: &str) -> bool {
    matches!(
        lang.to_lowercase().as_str(),
        "rust"
            | "javascript"
            | "typescript"
            | "python"
            | "go"
            | "c"
            | "c++"
            | "java"
            | "c#"
            | "php"
            | "ruby"
    )
}

/// Build a complexity report by analyzing function counts, lengths, cyclomatic and cognitive complexity.
pub(crate) fn build_complexity_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    limits: &AnalysisLimits,
    detail_functions: bool,
) -> Result<ComplexityReport> {
    let mut row_map: BTreeMap<String, &FileRow> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        row_map.insert(normalize_path(&row.path, root), row);
    }

    let mut file_complexities: Vec<FileComplexity> = Vec::new();
    let mut total_bytes = 0u64;
    let max_total = limits.max_bytes;
    let per_file_limit = limits.max_file_bytes.unwrap_or(DEFAULT_MAX_FILE_BYTES) as usize;

    for rel in files {
        if max_total.is_some_and(|limit| total_bytes >= limit) {
            break;
        }
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let row = match row_map.get(&rel_str) {
            Some(r) => *r,
            None => continue,
        };
        if !is_complexity_lang(&row.lang) {
            continue;
        }

        let path = root.join(rel);
        let bytes = match crate::content::io::read_head(&path, per_file_limit) {
            Ok(b) => b,
            Err(_) => continue,
        };
        total_bytes += bytes.len() as u64;

        if !crate::content::io::is_text_like(&bytes) {
            continue;
        }

        let text = String::from_utf8_lossy(&bytes);
        let lang_mapped = map_language_for_complexity(&row.lang);
        let (function_count, max_function_length) = count_functions(&row.lang, &text);
        let cyclomatic = estimate_cyclomatic(&row.lang, &text);

        // Compute cognitive complexity and nesting depth
        let cognitive_result =
            crate::content::complexity::estimate_cognitive_complexity(&text, lang_mapped);
        let nesting_result = crate::content::complexity::analyze_nesting_depth(&text, lang_mapped);

        let cognitive_complexity = if cognitive_result.function_count > 0 {
            Some(cognitive_result.total)
        } else {
            None
        };
        let max_nesting = if nesting_result.max_depth > 0 {
            Some(nesting_result.max_depth)
        } else {
            None
        };

        let risk_level = classify_risk_extended(
            function_count,
            max_function_length,
            cyclomatic,
            cognitive_complexity,
            max_nesting,
        );

        let functions = if detail_functions {
            Some(extract_function_details(&row.lang, &text))
        } else {
            None
        };

        file_complexities.push(FileComplexity {
            path: rel_str,
            module: row.module.clone(),
            function_count,
            max_function_length,
            cyclomatic_complexity: cyclomatic,
            cognitive_complexity,
            max_nesting,
            risk_level,
            functions,
        });
    }

    // Sort by cyclomatic complexity descending, then by path
    file_complexities.sort_by(|a, b| {
        b.cyclomatic_complexity
            .cmp(&a.cyclomatic_complexity)
            .then_with(|| a.path.cmp(&b.path))
    });

    // Compute aggregates before truncating
    let total_functions: usize = file_complexities.iter().map(|f| f.function_count).sum();
    let file_count = file_complexities.len();

    let avg_function_length = if total_functions == 0 {
        0.0
    } else {
        let total_max_len: usize = file_complexities
            .iter()
            .map(|f| f.max_function_length)
            .sum();
        round_f64(total_max_len as f64 / file_count as f64, 2)
    };

    let max_function_length = file_complexities
        .iter()
        .map(|f| f.max_function_length)
        .max()
        .unwrap_or(0);

    let avg_cyclomatic = if file_count == 0 {
        0.0
    } else {
        let total_cyclo: usize = file_complexities
            .iter()
            .map(|f| f.cyclomatic_complexity)
            .sum();
        round_f64(total_cyclo as f64 / file_count as f64, 2)
    };

    let max_cyclomatic = file_complexities
        .iter()
        .map(|f| f.cyclomatic_complexity)
        .max()
        .unwrap_or(0);

    // Compute cognitive complexity aggregates
    let cognitive_values: Vec<usize> = file_complexities
        .iter()
        .filter_map(|f| f.cognitive_complexity)
        .collect();
    let (avg_cognitive, max_cognitive) = if cognitive_values.is_empty() {
        (None, None)
    } else {
        let total: usize = cognitive_values.iter().sum();
        let max = cognitive_values.iter().copied().max().unwrap_or(0);
        (
            Some(round_f64(total as f64 / cognitive_values.len() as f64, 2)),
            Some(max),
        )
    };

    // Compute nesting depth aggregates
    let nesting_values: Vec<usize> = file_complexities
        .iter()
        .filter_map(|f| f.max_nesting)
        .collect();
    let (avg_nesting_depth, max_nesting_depth) = if nesting_values.is_empty() {
        (None, None)
    } else {
        let total: usize = nesting_values.iter().sum();
        let max = nesting_values.iter().copied().max().unwrap_or(0);
        (
            Some(round_f64(total as f64 / nesting_values.len() as f64, 2)),
            Some(max),
        )
    };

    let high_risk_files = file_complexities
        .iter()
        .filter(|f| {
            matches!(
                f.risk_level,
                ComplexityRisk::High | ComplexityRisk::Critical
            )
        })
        .count();

    // Generate histogram from all files before truncating
    let histogram = generate_complexity_histogram(&file_complexities, 5);

    // Compute maintainability index
    let maintainability_index = if file_count == 0 {
        None
    } else {
        average_parent_loc(export)
            .and_then(|avg_loc| compute_maintainability_index(avg_cyclomatic, avg_loc, None))
    };
    let technical_debt = compute_technical_debt_ratio(export, &file_complexities);

    // Only keep top files by complexity
    file_complexities.truncate(MAX_COMPLEXITY_FILES);

    Ok(ComplexityReport {
        total_functions,
        avg_function_length,
        max_function_length,
        avg_cyclomatic,
        max_cyclomatic,
        avg_cognitive,
        max_cognitive,
        avg_nesting_depth,
        max_nesting_depth,
        high_risk_files,
        histogram: Some(histogram),
        halstead: None, // Populated when halstead feature is enabled
        maintainability_index,
        technical_debt,
        files: file_complexities,
    })
}

fn round_f64(val: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (val * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_rust_functions() {
        let code = r#"
fn simple() {
    println!("hello");
}

pub fn public_fn() {
    let x = 1;
    let y = 2;
}

pub async fn async_fn() {
    todo!()
}
"#;
        let lines: Vec<&str> = code.lines().collect();
        let (count, _max_len) = count_rust_functions(&lines);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_python_functions() {
        let code = r#"
def foo():
    pass

async def bar():
    await something()

def baz():
    x = 1
    y = 2
    return x + y
"#;
        let lines: Vec<&str> = code.lines().collect();
        let (count, _max_len) = count_python_functions(&lines);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_estimate_cyclomatic_rust() {
        let code = r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            x * 2
        } else {
            x + 1
        }
    } else {
        match x {
            -1 => 0,
            _ => x.abs(),
        }
    }
}
"#;
        let cyclo = estimate_cyclomatic("rust", code);
        // Base 1 + 2 ifs + 1 match = 4
        assert_eq!(cyclo, 4);
    }

    #[test]
    fn test_estimate_cyclomatic_rust_no_else_if_double_count() {
        // "else if" should only count once (as "if"), not as both "if" and "else if"
        let code = r#"
fn branchy(x: i32) -> i32 {
    if x > 0 {
        1
    } else if x < 0 {
        -1
    } else if x == 0 {
        0
    } else {
        42
    }
}
"#;
        let cyclo = estimate_cyclomatic("rust", code);
        // Base 1 + 3 ifs (the initial "if" + 2 "else if" each matched by "if ")
        assert_eq!(cyclo, 4);
    }

    #[test]
    fn test_estimate_cyclomatic_js_no_switch_double_count() {
        // "switch" removed; only "case" contributes
        let code = r#"
function classify(x) {
    switch (x) {
        case 1: return "one";
        case 2: return "two";
        case 3: return "three";
        default: return "other";
    }
}
"#;
        let cyclo = estimate_cyclomatic("javascript", code);
        // Base 1 + 3 cases = 4
        assert_eq!(cyclo, 4);
    }

    #[test]
    fn test_classify_risk() {
        assert_eq!(
            classify_risk_extended(5, 10, 5, None, None),
            ComplexityRisk::Low
        );
        assert_eq!(
            classify_risk_extended(25, 30, 15, None, None),
            ComplexityRisk::Moderate
        );
        assert_eq!(
            classify_risk_extended(30, 60, 25, None, None),
            ComplexityRisk::High
        );
        assert_eq!(
            classify_risk_extended(60, 120, 60, None, None),
            ComplexityRisk::Critical
        );
    }

    #[test]
    fn test_classify_risk_with_cognitive() {
        // Low cognitive should not change low risk
        assert_eq!(
            classify_risk_extended(5, 10, 5, Some(10), Some(2)),
            ComplexityRisk::Low
        );
        // High cognitive should increase risk
        assert!(matches!(
            classify_risk_extended(5, 10, 5, Some(60), Some(6)),
            ComplexityRisk::Moderate | ComplexityRisk::High
        ));
        // High nesting should increase risk
        assert!(matches!(
            classify_risk_extended(5, 10, 5, Some(10), Some(9)),
            ComplexityRisk::Moderate | ComplexityRisk::High
        ));
    }

    #[test]
    fn test_is_complexity_lang() {
        assert!(is_complexity_lang("Rust"));
        assert!(is_complexity_lang("javascript"));
        assert!(is_complexity_lang("Python"));
        assert!(!is_complexity_lang("Markdown"));
        assert!(!is_complexity_lang("JSON"));
    }

    #[test]
    fn test_is_rust_fn_start_extended() {
        // Standard cases
        assert!(is_rust_fn_start("fn foo()"));
        assert!(is_rust_fn_start("pub fn foo()"));
        assert!(is_rust_fn_start("pub(crate) fn foo()"));
        assert!(is_rust_fn_start("pub(super) fn foo()"));
        assert!(is_rust_fn_start("async fn foo()"));
        assert!(is_rust_fn_start("pub async fn foo()"));
        assert!(is_rust_fn_start("unsafe fn foo()"));
        assert!(is_rust_fn_start("const fn foo()"));

        // Extended: pub(in path) visibility
        assert!(is_rust_fn_start("pub(in crate::foo) fn bar()"));
        assert!(is_rust_fn_start("pub(in crate::foo::bar) fn baz()"));

        // Extended: extern "ABI" functions
        assert!(is_rust_fn_start(r#"extern "C" fn callback()"#));
        assert!(is_rust_fn_start(r#"pub extern "C" fn callback()"#));
        assert!(is_rust_fn_start(r#"pub unsafe extern "C" fn callback()"#));

        // Extended: multi-qualifier combos
        assert!(is_rust_fn_start("pub(crate) unsafe async fn baz()"));
        assert!(is_rust_fn_start("pub(super) const fn helper()"));

        // Negative cases
        assert!(!is_rust_fn_start("let fn_name = 5;"));
        assert!(!is_rust_fn_start("// fn foo()"));
        assert!(!is_rust_fn_start("struct Foo {"));
    }

    #[test]
    fn test_detect_fn_rust_qualifiers() {
        let code = r#"
pub(crate) async fn crate_async() {
    todo!()
}

pub(super) async fn super_async() {
    todo!()
}

pub(crate) unsafe fn crate_unsafe() {
    todo!()
}

pub unsafe fn public_unsafe() {
    todo!()
}

pub(crate) const fn crate_const() -> u32 {
    42
}

pub const fn public_const() -> u32 {
    0
}
"#;
        let lines: Vec<&str> = code.lines().collect();
        let spans = detect_fn_spans_rust(&lines);
        let names: Vec<&str> = spans.iter().map(|(_, _, n)| n.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "crate_async",
                "super_async",
                "crate_unsafe",
                "public_unsafe",
                "crate_const",
                "public_const",
            ]
        );

        // Also verify count_rust_functions picks them all up
        let (count, _) = count_rust_functions(&lines);
        assert_eq!(count, 6);
    }

    #[test]
    fn test_detect_fn_python_decorators() {
        let code = r#"
@staticmethod
def plain_static():
    pass

@app.route("/")
@login_required
def index():
    return "hello"

def no_decorator():
    pass
"#;
        let lines: Vec<&str> = code.lines().collect();
        let spans = detect_fn_spans_python(&lines);
        assert_eq!(spans.len(), 3);

        // First function: @staticmethod + def plain_static
        let (start, _end, ref name) = spans[0];
        assert_eq!(name, "plain_static");
        // The span should start at the decorator line
        assert!(lines[start].trim().starts_with('@'));

        // Second function: two decorators + def index
        let (start2, _end2, ref name2) = spans[1];
        assert_eq!(name2, "index");
        assert!(lines[start2].trim().starts_with('@'));

        // Third function: no decorator
        let (start3, _end3, ref name3) = spans[2];
        assert_eq!(name3, "no_decorator");
        assert!(lines[start3].trim().starts_with("def "));
    }

    #[test]
    fn test_detect_fn_c_style_no_preprocessor() {
        let code = r#"
#define THING(x) { }
#define MACRO(a, b) { a + b; }

int main(int argc, char** argv) {
    return 0;
}
"#;
        let lines: Vec<&str> = code.lines().collect();
        let spans = detect_fn_spans_c_style(&lines);
        // Should only detect main, not #define macros
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].2, "main");
    }

    #[test]
    fn test_compute_technical_debt_ratio() {
        let export = ExportData {
            rows: vec![FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 1000,
                comments: 0,
                blanks: 0,
                lines: 1000,
                bytes: 1000,
                tokens: 250,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: tokmd_types::ChildIncludeMode::Separate,
        };

        let files = vec![FileComplexity {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            function_count: 3,
            max_function_length: 20,
            cyclomatic_complexity: 12,
            cognitive_complexity: Some(8),
            max_nesting: Some(2),
            risk_level: ComplexityRisk::Moderate,
            functions: None,
        }];

        let debt = compute_technical_debt_ratio(&export, &files).expect("debt ratio");
        assert_eq!(debt.complexity_points, 20);
        assert!((debt.ratio - 20.0).abs() < f64::EPSILON);
        assert!((debt.code_kloc - 1.0).abs() < f64::EPSILON);
        assert_eq!(debt.level, TechnicalDebtLevel::Low);
    }

    #[test]
    fn test_compute_technical_debt_ratio_none_for_zero_code() {
        let export = ExportData {
            rows: vec![FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 0,
                comments: 0,
                blanks: 0,
                lines: 0,
                bytes: 0,
                tokens: 0,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: tokmd_types::ChildIncludeMode::Separate,
        };

        let files = vec![FileComplexity {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            function_count: 1,
            max_function_length: 1,
            cyclomatic_complexity: 1,
            cognitive_complexity: Some(1),
            max_nesting: Some(1),
            risk_level: ComplexityRisk::Low,
            functions: None,
        }];

        assert!(compute_technical_debt_ratio(&export, &files).is_none());
    }

    #[test]
    fn test_detect_fn_python_decorators_extended() {
        let code = r#"
@app.route("/")
# This is a comment between decorators
@login_required

# Another comment
def index():
    return "hello"

@nested_decorator
# Indented comment
def nested():
    pass
"#;
        let lines: Vec<&str> = code.lines().collect();
        let spans = detect_fn_spans_python(&lines);
        assert_eq!(spans.len(), 2);

        // First function: index
        let (start, _end, ref name) = spans[0];
        assert_eq!(name, "index");
        // Should start at @app.route
        assert!(lines[start].trim().starts_with("@app.route"));

        // Second function: nested
        let (start2, _end2, ref name2) = spans[1];
        assert_eq!(name2, "nested");
        // Should start at @nested_decorator
        assert!(lines[start2].trim().starts_with("@nested_decorator"));
    }
}
