use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_maintainability::compute_maintainability_index;
use tokmd_analysis_types::{
    ComplexityHistogram, ComplexityReport, ComplexityRisk, FileComplexity,
    FunctionComplexityDetail, TechnicalDebtLevel, TechnicalDebtRatio,
};
use tokmd_types::{ExportData, FileKind, FileRow};

use tokmd_analysis_util::{AnalysisLimits, normalize_path};

const DEFAULT_MAX_FILE_BYTES: u64 = 128 * 1024;
const MAX_COMPLEXITY_FILES: usize = 100;
const TECHNICAL_DEBT_LOW_THRESHOLD: f64 = 30.0;
const TECHNICAL_DEBT_MODERATE_THRESHOLD: f64 = 60.0;
const TECHNICAL_DEBT_HIGH_THRESHOLD: f64 = 100.0;

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
pub fn build_complexity_report(
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
        let nesting_result = tokmd_content::complexity::analyze_nesting_depth(&text, lang_mapped);

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

/// Generate a histogram of cyclomatic complexity distribution.
///
/// Buckets files by cyclomatic complexity: 0-4, 5-9, 10-14, 15-19, 20-24, 25-29, 30+.
///
/// # Arguments
/// * `files` - Slice of file complexity data
/// * `bucket_size` - Size of each bucket (default 5)
///
/// # Returns
/// A `ComplexityHistogram` with counts for each bucket
///
/// # Note
/// This function is planned for integration in v1.6.0.
pub fn generate_complexity_histogram(
    files: &[FileComplexity],
    bucket_size: u32,
) -> ComplexityHistogram {
    // 7 buckets: 0-4, 5-9, 10-14, 15-19, 20-24, 25-29, 30+
    let num_buckets = 7;
    let mut counts = vec![0u32; num_buckets];

    for file in files {
        let complexity = file.cyclomatic_complexity as u32;
        let bucket = (complexity / bucket_size).min((num_buckets - 1) as u32) as usize;
        counts[bucket] += 1;
    }

    ComplexityHistogram {
        buckets: (0..num_buckets).map(|i| (i as u32) * bucket_size).collect(),
        counts,
        total: files.len() as u32,
    }
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

/// Check if a trimmed line starts a Rust function definition.
///
/// Handles all visibility qualifiers including `pub(in path::here)`,
/// optional `async`, `unsafe`, `const`, and `extern "ABI"` modifiers.
fn is_rust_fn_start(trimmed: &str) -> bool {
    // Fast path: find "fn " in the line
    let Some(fn_pos) = trimmed.find("fn ") else {
        return false;
    };

    // Everything before "fn " must be valid qualifiers
    let prefix = trimmed[..fn_pos].trim();
    if prefix.is_empty() {
        return true; // bare "fn name"
    }

    // Parse prefix: valid tokens are pub/pub(...), async, unsafe, const, extern "..."
    let mut rest = prefix;
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with("pub(") {
            // Skip pub(...) with arbitrary content (e.g., pub(in crate::foo))
            if let Some(close) = rest.find(')') {
                rest = &rest[close + 1..];
            } else {
                return false; // Unclosed paren
            }
        } else if let Some(r) = rest.strip_prefix("pub") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("async") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("unsafe") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("const") {
            rest = r;
        } else if rest.starts_with("extern") {
            // extern "ABI" - skip the ABI string
            rest = rest["extern".len()..].trim_start();
            if rest.starts_with('"') {
                if let Some(close) = rest[1..].find('"') {
                    rest = &rest[close + 2..];
                } else {
                    return false; // Unclosed string
                }
            }
        } else {
            return false; // Unknown token
        }
    }

    true
}

fn count_rust_functions(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth: i32 = 0;
    let mut in_string = false;
    let mut in_block_comment = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Detect function start
        if !in_fn && is_rust_fn_start(trimmed) {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }

        if in_fn {
            let chars: Vec<char> = line.chars().collect();
            let mut j = 0;
            while j < chars.len() {
                let c = chars[j];
                let next = chars.get(j + 1).copied();

                if in_block_comment {
                    if c == '*' && next == Some('/') {
                        in_block_comment = false;
                        j += 2;
                        continue;
                    }
                    j += 1;
                    continue;
                }

                if c == '/' && next == Some('/') {
                    break; // Line comment
                }

                if c == '/' && next == Some('*') {
                    in_block_comment = true;
                    j += 2;
                    continue;
                }

                if c == '"' && (j == 0 || chars[j - 1] != '\\') {
                    in_string = !in_string;
                    j += 1;
                    continue;
                }

                if !in_string && !in_block_comment {
                    if c == '{' {
                        brace_depth += 1;
                    } else if c == '}' {
                        brace_depth = brace_depth.saturating_sub(1);
                        if brace_depth == 0 {
                            let fn_len = i - fn_start + 1;
                            max_len = max_len.max(fn_len);
                            in_fn = false;
                            break;
                        }
                    }
                }
                j += 1;
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
                || trimmed.starts_with("while ")
                || trimmed.starts_with("until ")
                || trimmed.starts_with("for ")
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
        "rust" => &["if ", "match ", "while ", "for ", "loop ", "?", "&&", "||"],
        "javascript" | "typescript" => {
            &["if ", "case ", "while ", "for ", "?", "&&", "||", "catch "]
        }
        "python" => &["if ", "elif ", "while ", "for ", "except ", " and ", " or "],
        "go" => &["if ", "case ", "for ", "select ", "&&", "||"],
        "c" | "c++" | "java" | "c#" | "php" => {
            &["if ", "case ", "while ", "for ", "?", "&&", "||", "catch "]
        }
        "ruby" => &[
            "if ", "elsif ", "unless ", "while ", "until ", "for ", "when ", "rescue ", " and ",
            " or ",
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

fn average_parent_loc(export: &ExportData) -> Option<f64> {
    let total_code: usize = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .map(|r| r.code)
        .sum();
    let parent_count: usize = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .count();

    if parent_count == 0 {
        return None;
    }

    let avg_loc = total_code as f64 / parent_count as f64;
    if avg_loc <= 0.0 {
        return None;
    }
    Some(avg_loc)
}

/// Compute a complexity-to-size heuristic debt ratio.
///
/// Ratio = (sum cyclomatic + cognitive complexity points) / KLOC
fn compute_technical_debt_ratio(
    export: &ExportData,
    file_complexities: &[FileComplexity],
) -> Option<TechnicalDebtRatio> {
    if file_complexities.is_empty() {
        return None;
    }

    let total_code: usize = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .map(|r| r.code)
        .sum();
    if total_code == 0 {
        return None;
    }

    let complexity_points: usize = file_complexities
        .iter()
        .map(|f| f.cyclomatic_complexity + f.cognitive_complexity.unwrap_or(0))
        .sum();

    let code_kloc = total_code as f64 / 1000.0;
    let ratio = round_f64(complexity_points as f64 / code_kloc, 2);
    let level = if ratio < TECHNICAL_DEBT_LOW_THRESHOLD {
        TechnicalDebtLevel::Low
    } else if ratio < TECHNICAL_DEBT_MODERATE_THRESHOLD {
        TechnicalDebtLevel::Moderate
    } else if ratio < TECHNICAL_DEBT_HIGH_THRESHOLD {
        TechnicalDebtLevel::High
    } else {
        TechnicalDebtLevel::Critical
    };

    Some(TechnicalDebtRatio {
        ratio,
        complexity_points,
        code_kloc: round_f64(code_kloc, 4),
        level,
    })
}

/// Extract function-level complexity details for a source file.
fn extract_function_details(lang: &str, text: &str) -> Vec<FunctionComplexityDetail> {
    let lines: Vec<&str> = text.lines().collect();
    let mapped_lang = map_language_for_complexity(lang);

    let fn_spans: Vec<(usize, usize, String)> = match lang.to_lowercase().as_str() {
        "rust" => detect_fn_spans_rust(&lines),
        "javascript" | "typescript" => detect_fn_spans_js(&lines),
        "python" => detect_fn_spans_python(&lines),
        "go" => detect_fn_spans_go(&lines),
        "c" | "c++" | "java" | "c#" | "php" => detect_fn_spans_c_style(&lines),
        _ => Vec::new(),
    };

    fn_spans
        .into_iter()
        .map(|(start, end, name)| {
            let length = end.saturating_sub(start) + 1;
            let fn_text = lines[start..=end.min(lines.len() - 1)].join("\n");
            let cyclomatic = estimate_cyclomatic_inline(mapped_lang, &fn_text);

            let cognitive_result =
                tokmd_content::complexity::estimate_cognitive_complexity(&fn_text, mapped_lang);
            let cognitive = Some(cognitive_result.total);

            let nesting_result =
                tokmd_content::complexity::analyze_nesting_depth(&fn_text, mapped_lang);
            let max_nesting = if nesting_result.max_depth > 0 {
                Some(nesting_result.max_depth)
            } else {
                None
            };

            let param_count = count_params(lines.get(start).unwrap_or(&""));

            FunctionComplexityDetail {
                name,
                line_start: start + 1,
                line_end: end + 1,
                length,
                cyclomatic,
                cognitive,
                max_nesting,
                param_count: if param_count > 0 {
                    Some(param_count)
                } else {
                    None
                },
            }
        })
        .collect()
}

/// Detect Rust function spans: (start_line, end_line, name).
fn detect_fn_spans_rust(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if is_rust_fn_start(trimmed) {
            let name = extract_rust_fn_name(trimmed);
            let start = i;
            if let Some(end) = find_brace_end_at(lines, i) {
                spans.push((start, end, name));
                i = end + 1;
            } else {
                // No body found (trait sig, abstract, extern) — skip
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    spans
}

/// Detect JS/TS function spans.
fn detect_fn_spans_js(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let is_fn = trimmed.starts_with("function ")
            || trimmed.starts_with("async function ")
            || trimmed.starts_with("export function ")
            || trimmed.starts_with("export async function ")
            || trimmed.contains("=> {");
        if is_fn && !trimmed.starts_with("//") {
            let name = extract_js_fn_name(trimmed);
            let start = i;
            if let Some(end) = find_brace_end_at(lines, i) {
                spans.push((start, end, name));
                i = end + 1;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    spans
}

/// Detect Python function spans.
fn detect_fn_spans_python(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            let name = extract_python_fn_name(trimmed);
            let base_indent = lines[i].len() - lines[i].trim_start().len();

            // Walk upward to include decorator lines (@...)
            let mut start = i;
            {
                let mut k = i;
                while k > 0 {
                    let prev_line = lines[k - 1];
                    let prev_trimmed = prev_line.trim();

                    if prev_trimmed.is_empty() {
                        k -= 1;
                        continue;
                    }

                    if prev_trimmed.starts_with('#') {
                        k -= 1;
                        continue;
                    }

                    let prev_indent = prev_line.len() - prev_line.trim_start().len();
                    if prev_indent == base_indent && prev_trimmed.starts_with('@') {
                        start = k - 1;
                        k -= 1;
                    } else {
                        break;
                    }
                }
            }
            let mut end = i;
            let mut j = i + 1;
            while j < lines.len() {
                let lt = lines[j].trim();
                if lt.is_empty() || lt.starts_with('#') {
                    j += 1;
                    continue;
                }
                let indent = lines[j].len() - lines[j].trim_start().len();
                if indent <= base_indent {
                    break;
                }
                end = j;
                j += 1;
            }
            spans.push((start, end, name));
            i = end + 1;
        } else {
            i += 1;
        }
    }
    spans
}

/// Detect Go function spans.
fn detect_fn_spans_go(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("func ") {
            let name = extract_go_fn_name(trimmed);
            let start = i;
            if let Some(end) = find_brace_end_at(lines, i) {
                spans.push((start, end, name));
                i = end + 1;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    spans
}

/// Detect C-style function spans.
fn detect_fn_spans_c_style(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let looks_like_fn = (trimmed.ends_with(") {") || trimmed.ends_with("){"))
            && !trimmed.starts_with("if ")
            && !trimmed.starts_with("if(")
            && !trimmed.starts_with("while ")
            && !trimmed.starts_with("for ")
            && !trimmed.starts_with("switch ")
            && !trimmed.starts_with("//")
            && !trimmed.starts_with('#');
        if looks_like_fn {
            let name = extract_c_fn_name(trimmed);
            let start = i;
            if let Some(end) = find_brace_end_at(lines, i) {
                spans.push((start, end, name));
                i = end + 1;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    spans
}

/// Find closing brace for a block starting at `start_line`.
///
/// Returns `None` if no opening brace is found (e.g., trait method
/// signatures, extern declarations, abstract methods).
fn find_brace_end_at(lines: &[&str], start_line: usize) -> Option<usize> {
    let mut depth: usize = 0;
    let mut found_open = false;
    for (i, line) in lines.iter().enumerate().skip(start_line) {
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
                found_open = true;
            } else if ch == '}' {
                depth = depth.saturating_sub(1);
                if found_open && depth == 0 {
                    return Some(i);
                }
            }
        }
    }
    // Both cases (no open brace, or unclosed braces) → None
    None
}

/// Extract Rust function name from a line containing "fn ".
fn extract_rust_fn_name(line: &str) -> String {
    if let Some(idx) = line.find("fn ") {
        let after = &line[idx + 3..];
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "<unknown>".to_string()
}

fn extract_js_fn_name(line: &str) -> String {
    if let Some(idx) = line.find("function ") {
        let after = &line[idx + 9..];
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    if let Some(paren_idx) = line.find('(') {
        let before = line[..paren_idx].trim();
        let name: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "<anonymous>".to_string()
}

fn extract_python_fn_name(line: &str) -> String {
    let keyword = if line.contains("async def ") {
        "async def "
    } else {
        "def "
    };
    if let Some(idx) = line.find(keyword) {
        let after = &line[idx + keyword.len()..];
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "<unknown>".to_string()
}

fn extract_go_fn_name(line: &str) -> String {
    if let Some(idx) = line.find("func ") {
        let after = &line[idx + 5..];
        let after = if after.starts_with('(') {
            if let Some(close) = after.find(')') {
                after[close + 1..].trim_start()
            } else {
                after
            }
        } else {
            after
        };
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "<unknown>".to_string()
}

fn extract_c_fn_name(line: &str) -> String {
    if let Some(paren_idx) = line.find('(') {
        let before = line[..paren_idx].trim();
        let name: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "<unknown>".to_string()
}

/// Count function parameters from a line.
fn count_params(line: &str) -> usize {
    if let Some(open) = line.find('(')
        && let Some(close) = line.find(')')
    {
        let params = line[open + 1..close].trim();
        if params.is_empty() {
            return 0;
        }
        return params.split(',').count();
    }
    0
}

/// Estimate cyclomatic complexity for a function body.
fn estimate_cyclomatic_inline(lang: &str, text: &str) -> usize {
    let mut complexity = 1usize;
    let keywords: &[&str] = match lang {
        "rust" => &["if ", "match ", "while ", "for ", "loop ", "?", "&&", "||"],
        "javascript" | "typescript" => {
            &["if ", "case ", "while ", "for ", "?", "&&", "||", "catch "]
        }
        "python" => &["if ", "elif ", "while ", "for ", "except ", " and ", " or "],
        "go" => &["if ", "case ", "for ", "select ", "&&", "||"],
        "c" | "c++" | "java" | "c#" | "php" => {
            &["if ", "case ", "while ", "for ", "?", "&&", "||", "catch "]
        }
        _ => &[],
    };
    let lower = text.to_lowercase();
    for keyword in keywords {
        complexity += lower.matches(keyword).count();
    }
    complexity
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
