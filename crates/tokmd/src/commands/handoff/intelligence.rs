//! Handoff intelligence artifact construction.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use tokmd_scan::normalize_slashes as normalize_path;
use tokmd_types::{
    CapabilityState, CapabilityStatus, ExportData, FileKind, FileRow, HandoffComplexity,
    HandoffDerived, HandoffHotspot, HandoffIntelligence,
};

use crate::cli;

use super::capabilities::{capability_reason, capability_state};
use super::{DEFAULT_TREE_DEPTH, round_f64};

/// Build intelligence data for the handoff.
pub(super) fn build_intelligence(
    export: &ExportData,
    args: &cli::HandoffArgs,
    capabilities: &[CapabilityStatus],
    git_scores: Option<&tokmd_core::context_git::GitScores>,
) -> HandoffIntelligence {
    let mut warnings = Vec::new();

    // Build tree (always included)
    let tree = Some(tokmd_format::render_handoff_tree(
        export,
        DEFAULT_TREE_DEPTH,
    ));
    let tree_depth = tree.as_ref().map(|_| DEFAULT_TREE_DEPTH);

    // Build hotspots (Risk/Deep presets)
    let wants_hotspots = matches!(
        args.preset,
        cli::HandoffPreset::Risk | cli::HandoffPreset::Deep
    );
    let hotspots = if wants_hotspots {
        match git_scores {
            Some(scores) if !scores.hotspots.is_empty() => {
                let mut hotspot_rows: Vec<HandoffHotspot> = scores
                    .hotspots
                    .iter()
                    .map(|(path, &score)| {
                        let commits = scores.commit_counts.get(path).copied().unwrap_or(0);
                        let lines = export
                            .rows
                            .iter()
                            .find(|r| normalize_path(&r.path) == *path)
                            .map(|r| r.lines)
                            .unwrap_or(0);
                        HandoffHotspot {
                            path: path.clone(),
                            commits,
                            lines,
                            score,
                        }
                    })
                    .collect();
                // Sort by score descending, then by path
                hotspot_rows
                    .sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));
                // Limit to top 20
                hotspot_rows.truncate(20);
                Some(hotspot_rows)
            }
            _ => {
                let state = capability_state(capabilities, "git_history");
                if wants_hotspots {
                    let reason = capability_reason(capabilities, "git_history");
                    match state {
                        Some(CapabilityState::Available) => {
                            warnings.push("hotspots unavailable: no git history found".to_string());
                        }
                        Some(CapabilityState::Skipped) => {
                            let msg = if let Some(r) = reason {
                                format!("hotspots unavailable: git history skipped ({})", r)
                            } else {
                                "hotspots unavailable: git history skipped".to_string()
                            };
                            warnings.push(msg);
                        }
                        Some(CapabilityState::Unavailable) => {
                            let msg = if let Some(r) = reason {
                                format!("hotspots unavailable: git history unavailable ({})", r)
                            } else {
                                "hotspots unavailable: git history unavailable".to_string()
                            };
                            warnings.push(msg);
                        }
                        None => {}
                    }
                }
                None
            }
        }
    } else {
        None
    };

    // Build complexity (Standard/Risk/Deep presets)
    let complexity = if matches!(
        args.preset,
        cli::HandoffPreset::Standard | cli::HandoffPreset::Risk | cli::HandoffPreset::Deep
    ) {
        Some(build_simple_complexity(export))
    } else {
        None
    };

    // Build derived (Standard/Risk/Deep presets)
    let derived = if matches!(
        args.preset,
        cli::HandoffPreset::Standard | cli::HandoffPreset::Risk | cli::HandoffPreset::Deep
    ) {
        Some(build_simple_derived(export))
    } else {
        None
    };

    HandoffIntelligence {
        tree,
        tree_depth,
        hotspots,
        complexity,
        derived,
        warnings,
    }
}

/// Maximum number of files to analyze for complexity.
const MAX_COMPLEXITY_FILES: usize = 50;
/// Maximum bytes to read per file for complexity analysis.
const MAX_COMPLEXITY_BYTES: usize = 128 * 1024;

/// Build complexity metrics by reading source files and counting functions/branching.
fn build_simple_complexity(export: &ExportData) -> HandoffComplexity {
    let mut parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .filter(|r| is_analyzable_lang(&r.lang))
        .collect();

    if parents.is_empty() {
        return HandoffComplexity {
            total_functions: 0,
            avg_function_length: 0.0,
            max_function_length: 0,
            avg_cyclomatic: 0.0,
            max_cyclomatic: 0,
            high_risk_files: 0,
        };
    }

    // Sort by code lines descending, take top files
    parents.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));
    parents.truncate(MAX_COMPLEXITY_FILES);

    let mut total_functions: usize = 0;
    let mut all_function_lengths: Vec<usize> = Vec::new();
    let mut max_function_length: usize = 0;
    let mut file_cyclomatic: Vec<usize> = Vec::new();
    let mut max_cyclomatic: usize = 0;
    let mut high_risk_files: usize = 0;

    for row in &parents {
        let path = PathBuf::from(&row.path);
        let content = match read_file_capped(&path, MAX_COMPLEXITY_BYTES) {
            Some(c) => c,
            None => continue,
        };

        let (fn_count, fn_max_len) = count_functions_simple(&row.lang, &content);
        let cyclomatic = estimate_cyclomatic_simple(&row.lang, &content);

        total_functions += fn_count;
        if fn_max_len > 0 {
            all_function_lengths.push(fn_max_len);
        }
        max_function_length = max_function_length.max(fn_max_len);
        file_cyclomatic.push(cyclomatic);
        max_cyclomatic = max_cyclomatic.max(cyclomatic);

        // High risk: high cyclomatic OR very long functions
        if cyclomatic > 20 || fn_max_len > 100 {
            high_risk_files += 1;
        }
    }

    let avg_function_length = if total_functions == 0 {
        0.0
    } else {
        let total_len: usize = all_function_lengths.iter().sum();
        total_len as f64 / all_function_lengths.len().max(1) as f64
    };

    let avg_cyclomatic = if file_cyclomatic.is_empty() {
        0.0
    } else {
        let total: usize = file_cyclomatic.iter().sum();
        total as f64 / file_cyclomatic.len() as f64
    };

    HandoffComplexity {
        total_functions,
        avg_function_length: round_f64(avg_function_length, 2),
        max_function_length,
        avg_cyclomatic: round_f64(avg_cyclomatic, 2),
        max_cyclomatic,
        high_risk_files,
    }
}

/// Check if a language is analyzable for complexity.
fn is_analyzable_lang(lang: &str) -> bool {
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

/// Read file contents up to a byte cap. Returns None if unreadable.
fn read_file_capped(path: &Path, max_bytes: usize) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut buf = vec![0u8; max_bytes];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);
    String::from_utf8(buf).ok()
}

/// Count functions and estimate max function length in lines.
/// Simplified inline version that avoids heavy dependencies.
fn count_functions_simple(lang: &str, text: &str) -> (usize, usize) {
    let lines: Vec<&str> = text.lines().collect();
    match lang.to_lowercase().as_str() {
        "rust" => count_brace_functions(&lines, is_rust_fn_start_simple),
        "go" => count_brace_functions(&lines, |t| t.starts_with("func ")),
        "javascript" | "typescript" => count_brace_functions(&lines, |t| {
            t.starts_with("function ")
                || t.starts_with("async function ")
                || t.starts_with("export function ")
                || t.starts_with("export async function ")
                || (t.contains("=> {") && !t.starts_with("//"))
        }),
        "c" | "c++" | "java" | "c#" | "php" => count_brace_functions(&lines, |t| {
            (t.ends_with(") {") || t.ends_with("){"))
                && !t.starts_with("if ")
                && !t.starts_with("if(")
                && !t.starts_with("while ")
                && !t.starts_with("while(")
                && !t.starts_with("for ")
                && !t.starts_with("for(")
                && !t.starts_with("switch ")
                && !t.starts_with("//")
        }),
        "python" => count_python_functions_simple(&lines),
        "ruby" => count_ruby_functions_simple(&lines),
        _ => (0, 0),
    }
}

/// Check if a trimmed line starts a Rust function definition.
/// Handles all visibility qualifiers including `pub(in path)`, extern "ABI", etc.
fn is_rust_fn_start_simple(trimmed: &str) -> bool {
    let Some(fn_pos) = trimmed.find("fn ") else {
        return false;
    };
    let prefix = trimmed[..fn_pos].trim();
    if prefix.is_empty() {
        return true;
    }
    let mut rest = prefix;
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with("pub(") {
            if let Some(close) = rest.find(')') {
                rest = &rest[close + 1..];
            } else {
                return false;
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
            rest = rest["extern".len()..].trim_start();
            if rest.starts_with('"') {
                if let Some(close) = rest[1..].find('"') {
                    rest = &rest[close + 2..];
                } else {
                    return false;
                }
            }
        } else {
            return false;
        }
    }
    true
}

/// Count functions in brace-delimited languages.
fn count_brace_functions(lines: &[&str], is_fn_start: impl Fn(&str) -> bool) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !in_fn && is_fn_start(trimmed) {
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

/// Count functions in Python (indentation-based).
fn count_python_functions_simple(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut fn_start = 0;
    let mut fn_indent = 0;
    let mut in_fn = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            if in_fn {
                max_len = max_len.max(i - fn_start);
            }
            count += 1;
            in_fn = true;
            fn_start = i;
            fn_indent = line.len() - line.trim_start().len();
        } else if in_fn && !trimmed.is_empty() && !trimmed.starts_with('#') {
            let indent = line.len() - line.trim_start().len();
            if indent <= fn_indent
                && !trimmed.starts_with("def ")
                && !trimmed.starts_with("async def ")
            {
                max_len = max_len.max(i - fn_start);
                in_fn = false;
            }
        }
    }
    if in_fn {
        max_len = max_len.max(lines.len() - fn_start);
    }

    (count, max_len)
}

/// Count functions in Ruby (end-delimited).
fn count_ruby_functions_simple(lines: &[&str]) -> (usize, usize) {
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
                    max_len = max_len.max(i - fn_start + 1);
                    in_fn = false;
                }
            }
        }
    }

    (count, max_len)
}

/// Estimate file-level cyclomatic complexity by counting branching keywords.
fn estimate_cyclomatic_simple(lang: &str, text: &str) -> usize {
    let mut complexity: usize = 1; // base

    let keywords: &[&str] = match lang.to_lowercase().as_str() {
        "rust" => &[
            "if ", "else if ", "match ", "while ", "for ", "loop ", "&&", "||",
        ],
        "javascript" | "typescript" => &[
            "if ", "else if ", "switch ", "case ", "while ", "for ", "&&", "||", "catch ",
        ],
        "python" => &["if ", "elif ", "while ", "for ", "except ", " and ", " or "],
        "go" => &[
            "if ", "else if ", "switch ", "case ", "for ", "select ", "&&", "||",
        ],
        "c" | "c++" | "java" | "c#" | "php" => &[
            "if ", "else if ", "switch ", "case ", "while ", "for ", "&&", "||", "catch ",
        ],
        "ruby" => &[
            "if ", "elsif ", "unless ", "while ", "until ", "for ", "case ", "when ", "rescue ",
        ],
        _ => return 1,
    };

    for line in text.lines() {
        let trimmed = line.trim();
        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
            continue;
        }
        for keyword in keywords {
            complexity += trimmed.matches(keyword).count();
        }
    }

    complexity
}

/// Build simple derived metrics from export data.
fn build_simple_derived(export: &ExportData) -> HandoffDerived {
    let parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .collect();

    let total_files = parents.len();
    let total_code: usize = parents.iter().map(|r| r.code).sum();
    let total_lines: usize = parents.iter().map(|r| r.lines).sum();
    let total_tokens: usize = parents.iter().map(|r| r.tokens).sum();

    // Count languages
    let mut lang_counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in &parents {
        *lang_counts.entry(row.lang.clone()).or_insert(0) += row.code;
    }
    let lang_count = lang_counts.len();

    // Find dominant language
    let (dominant_lang, dominant_code) = lang_counts
        .iter()
        .max_by_key(|(_, code)| *code)
        .map(|(lang, code)| (lang.clone(), *code))
        .unwrap_or_else(|| ("Unknown".to_string(), 0));

    let dominant_pct = if total_code > 0 {
        (dominant_code as f64 / total_code as f64) * 100.0
    } else {
        0.0
    };

    HandoffDerived {
        total_files,
        total_code,
        total_lines,
        total_tokens,
        lang_count,
        dominant_lang,
        dominant_pct: round_f64(dominant_pct, 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_simple_derived_empty() {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 2,
            children: tokmd_types::ChildIncludeMode::ParentsOnly,
        };
        let derived = build_simple_derived(&export);
        assert_eq!(derived.total_files, 0);
        assert_eq!(derived.total_code, 0);
        assert_eq!(derived.lang_count, 0);
    }

    #[test]
    fn test_build_simple_complexity_empty() {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 2,
            children: tokmd_types::ChildIncludeMode::ParentsOnly,
        };
        let complexity = build_simple_complexity(&export);
        assert_eq!(complexity.total_functions, 0);
        assert_eq!(complexity.high_risk_files, 0);
    }

    #[test]
    fn test_count_functions_simple_rust() {
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
        let (count, max_len) = count_functions_simple("Rust", code);
        assert_eq!(count, 3);
        assert!(max_len >= 3);
    }

    #[test]
    fn test_count_functions_simple_python() {
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
        let (count, _max_len) = count_functions_simple("Python", code);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_estimate_cyclomatic_simple_rust() {
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
        let cyclo = estimate_cyclomatic_simple("Rust", code);
        // Base 1 + 2 ifs + 1 else if (none here) + 1 match = 4+
        assert!(cyclo >= 4, "Expected cyclomatic >= 4, got {}", cyclo);
    }

    #[test]
    fn test_is_analyzable_lang() {
        assert!(is_analyzable_lang("Rust"));
        assert!(is_analyzable_lang("javascript"));
        assert!(is_analyzable_lang("Python"));
        assert!(!is_analyzable_lang("Markdown"));
        assert!(!is_analyzable_lang("JSON"));
        assert!(!is_analyzable_lang("TOML"));
    }
}
