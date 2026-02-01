use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::{ComplexityReport, ComplexityRisk, FileComplexity};
use tokmd_types::{ExportData, FileKind, FileRow};

use crate::analysis::AnalysisLimits;
use crate::util::normalize_path;

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
        let bytes = match tokmd_content::read_head(&path, per_file_limit) {
            Ok(b) => b,
            Err(_) => continue,
        };
        total_bytes += bytes.len() as u64;

        if !tokmd_content::is_text_like(&bytes) {
            continue;
        }

        let text = String::from_utf8_lossy(&bytes);
        let lang_mapped = map_language_for_complexity(&row.lang);
        let (function_count, max_function_length) = count_functions(&row.lang, &text);
        let cyclomatic = estimate_cyclomatic(&row.lang, &text);

        // Compute cognitive complexity and nesting depth
        let cognitive_result =
            tokmd_content::complexity::estimate_cognitive_complexity(&text, lang_mapped);
        let nesting_result =
            tokmd_content::complexity::analyze_nesting_depth(&text, lang_mapped);

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

        file_complexities.push(FileComplexity {
            path: rel_str,
            module: row.module.clone(),
            function_count,
            max_function_length,
            cyclomatic_complexity: cyclomatic,
            cognitive_complexity,
            max_nesting,
            risk_level,
            functions: None, // Will be populated when --detail-functions is used
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
        files: file_complexities,
    })
}

/// Count functions and estimate max function length in lines.
fn count_functions(lang: &str, text: &str) -> (usize, usize) {
    let lines: Vec<&str> = text.lines().collect();
    match lang.to_lowercase().as_str() {
        "rust" => count_rust_functions(&lines),
        "javascript" | "typescript" => count_js_functions(&lines),
        "python" => count_python_functions(&lines),
        "go" => count_go_functions(&lines),
        "c" | "c++" | "java" | "c#" | "php" => count_c_style_functions(&lines),
        "ruby" => count_ruby_functions(&lines),
        _ => (0, 0),
    }
}

fn count_rust_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Detect function start
        if !in_fn
            && (trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub(crate) fn ")
                || trimmed.starts_with("pub(super) fn ")
                || trimmed.starts_with("async fn ")
                || trimmed.starts_with("pub async fn "))
        {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }

        if in_fn {
            brace_depth += line.chars().filter(|&c| c == '{').count();
            brace_depth = brace_depth.saturating_sub(line.chars().filter(|&c| c == '}').count());

            if brace_depth == 0 && line.contains('}') {
                let fn_len = i - fn_start + 1;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    (count, max_len)
}

fn count_js_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Detect function declarations
        let is_fn_start = trimmed.starts_with("function ")
            || trimmed.starts_with("async function ")
            || trimmed.contains("=> {")
            || (trimmed.contains("(")
                && trimmed.contains(") {")
                && !trimmed.starts_with("if ")
                && !trimmed.starts_with("while ")
                && !trimmed.starts_with("for ")
                && !trimmed.starts_with("switch "));

        if !in_fn && is_fn_start {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }

        if in_fn {
            brace_depth += line.chars().filter(|&c| c == '{').count();
            brace_depth = brace_depth.saturating_sub(line.chars().filter(|&c| c == '}').count());

            if brace_depth == 0 && line.contains('}') {
                let fn_len = i - fn_start + 1;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    (count, max_len)
}

fn count_python_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut fn_start = 0;
    let mut fn_indent = 0;
    let mut in_fn = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            if in_fn {
                // Previous function ended
                let fn_len = i - fn_start;
                max_len = max_len.max(fn_len);
            }
            count += 1;
            in_fn = true;
            fn_start = i;
            fn_indent = line.len() - line.trim_start().len();
        } else if in_fn && !trimmed.is_empty() && !trimmed.starts_with('#') {
            let current_indent = line.len() - line.trim_start().len();
            if current_indent <= fn_indent
                && !trimmed.starts_with("def ")
                && !trimmed.starts_with("async def ")
            {
                let fn_len = i - fn_start;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    if in_fn {
        let fn_len = lines.len() - fn_start;
        max_len = max_len.max(fn_len);
    }

    (count, max_len)
}

fn count_go_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if !in_fn && trimmed.starts_with("func ") {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }

        if in_fn {
            brace_depth += line.chars().filter(|&c| c == '{').count();
            brace_depth = brace_depth.saturating_sub(line.chars().filter(|&c| c == '}').count());

            if brace_depth == 0 && line.contains('}') {
                let fn_len = i - fn_start + 1;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    (count, max_len)
}

fn count_c_style_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Heuristic: line ends with ) { or ) followed by { on next line
        let looks_like_fn = trimmed.ends_with(") {")
            || (trimmed.ends_with(')') && i + 1 < lines.len() && lines[i + 1].trim() == "{");

        // Exclude control structures
        let is_control = trimmed.starts_with("if ")
            || trimmed.starts_with("if(")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("while(")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("for(")
            || trimmed.starts_with("switch ")
            || trimmed.starts_with("switch(");

        if !in_fn && looks_like_fn && !is_control {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }

        if in_fn {
            brace_depth += line.chars().filter(|&c| c == '{').count();
            brace_depth = brace_depth.saturating_sub(line.chars().filter(|&c| c == '}').count());

            if brace_depth == 0 && line.contains('}') {
                let fn_len = i - fn_start + 1;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    (count, max_len)
}

fn count_ruby_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut fn_start = 0;
    let mut in_fn = false;
    let mut depth = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("def ") {
            if !in_fn {
                count += 1;
                in_fn = true;
                fn_start = i;
                depth = 1;
            } else {
                depth += 1;
            }
        } else if in_fn {
            // Count nested blocks
            if trimmed.starts_with("do")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("module ")
                || trimmed.starts_with("begin")
                || trimmed.starts_with("if ")
                || trimmed.starts_with("unless ")
                || trimmed.starts_with("case ")
            {
                depth += 1;
            }
            if trimmed == "end" || trimmed.starts_with("end ") {
                depth -= 1;
                if depth == 0 {
                    let fn_len = i - fn_start + 1;
                    max_len = max_len.max(fn_len);
                    in_fn = false;
                }
            }
        }
    }

    (count, max_len)
}

/// Estimate cyclomatic complexity by counting branching keywords.
fn estimate_cyclomatic(lang: &str, text: &str) -> usize {
    // Base complexity is 1
    let mut complexity = 1usize;

    let keywords: &[&str] = match lang.to_lowercase().as_str() {
        "rust" => &[
            "if ", "else if ", "match ", "while ", "for ", "loop ", "?", "&&", "||",
        ],
        "javascript" | "typescript" => &[
            "if ", "else if ", "switch ", "case ", "while ", "for ", "?", "&&", "||", "catch ",
        ],
        "python" => &[
            "if ", "elif ", "while ", "for ", "except ", " and ", " or ", " if ",
        ],
        "go" => &[
            "if ", "else if ", "switch ", "case ", "for ", "select ", "&&", "||",
        ],
        "c" | "c++" | "java" | "c#" | "php" => &[
            "if ", "else if ", "switch ", "case ", "while ", "for ", "?", "&&", "||", "catch ",
        ],
        "ruby" => &[
            "if ", "elsif ", "unless ", "while ", "until ", "for ", "case ", "when ", "rescue ",
            " and ", " or ",
        ],
        _ => &[],
    };

    let lower = text.to_lowercase();
    for keyword in keywords {
        complexity += lower.matches(keyword).count();
    }

    complexity
}

/// Classify risk based on complexity metrics including cognitive and nesting.
fn classify_risk_extended(
    function_count: usize,
    max_function_length: usize,
    cyclomatic: usize,
    cognitive: Option<usize>,
    max_nesting: Option<usize>,
) -> ComplexityRisk {
    // Risk factors
    let mut score = 0;

    // Function count risk
    if function_count > 50 {
        score += 2;
    } else if function_count > 20 {
        score += 1;
    }

    // Function length risk (long functions are harder to maintain)
    if max_function_length > 100 {
        score += 3;
    } else if max_function_length > 50 {
        score += 2;
    } else if max_function_length > 25 {
        score += 1;
    }

    // Cyclomatic complexity risk
    if cyclomatic > 50 {
        score += 3;
    } else if cyclomatic > 20 {
        score += 2;
    } else if cyclomatic > 10 {
        score += 1;
    }

    // Cognitive complexity risk (higher thresholds than cyclomatic)
    if let Some(cog) = cognitive {
        if cog > 100 {
            score += 3;
        } else if cog > 50 {
            score += 2;
        } else if cog > 25 {
            score += 1;
        }
    }

    // Nesting depth risk
    if let Some(nesting) = max_nesting {
        if nesting > 8 {
            score += 3;
        } else if nesting > 5 {
            score += 2;
        } else if nesting > 4 {
            score += 1;
        }
    }

    match score {
        0..=1 => ComplexityRisk::Low,
        2..=4 => ComplexityRisk::Moderate,
        5..=7 => ComplexityRisk::High,
        _ => ComplexityRisk::Critical,
    }
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
        assert!(cyclo >= 4);
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
}
