use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::maintainability::compute_maintainability_index;
use anyhow::Result;
#[cfg(test)]
use tokmd_analysis_types::ComplexityRisk;
#[cfg(test)]
use tokmd_analysis_types::TechnicalDebtLevel;
use tokmd_analysis_types::{ComplexityReport, FileComplexity};
use tokmd_types::{ExportData, FileKind, FileRow};

use tokmd_analysis_types::{AnalysisLimits, normalize_path};

mod debt;
mod details;
mod functions;
mod histogram;
mod language;
mod math;
mod risk;
mod summary;

use debt::{average_parent_loc, compute_technical_debt_ratio};
use details::extract_function_details;
#[cfg(test)]
use details::{detect_fn_spans_c_style, detect_fn_spans_python, detect_fn_spans_rust};
use functions::count_functions;
#[cfg(test)]
use functions::{count_python_functions, count_rust_functions, is_rust_fn_start};
pub(crate) use histogram::generate_complexity_histogram;
use language::{is_complexity_lang, map_language_for_complexity};
use risk::{classify_risk_extended, estimate_cyclomatic};
use summary::summarize_file_complexities;

const DEFAULT_MAX_FILE_BYTES: u64 = 128 * 1024;
const MAX_COMPLEXITY_FILES: usize = 100;

#[cfg(test)]
#[path = "tests.rs"]
mod moved_tests;
#[cfg(test)]
#[path = "tests/unit.rs"]
mod unit_tests;

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
    let summary = summarize_file_complexities(&file_complexities);

    // Generate histogram from all files before truncating
    let histogram = generate_complexity_histogram(&file_complexities, 5);

    // Compute maintainability index
    let maintainability_index = if file_complexities.is_empty() {
        None
    } else {
        average_parent_loc(export).and_then(|avg_loc| {
            compute_maintainability_index(summary.avg_cyclomatic, avg_loc, None)
        })
    };
    let technical_debt = compute_technical_debt_ratio(export, &file_complexities);

    // Only keep top files by complexity
    file_complexities.truncate(MAX_COMPLEXITY_FILES);

    Ok(ComplexityReport {
        total_functions: summary.total_functions,
        avg_function_length: summary.avg_function_length,
        max_function_length: summary.max_function_length,
        avg_cyclomatic: summary.avg_cyclomatic,
        max_cyclomatic: summary.max_cyclomatic,
        avg_cognitive: summary.avg_cognitive,
        max_cognitive: summary.max_cognitive,
        avg_nesting_depth: summary.avg_nesting_depth,
        max_nesting_depth: summary.max_nesting_depth,
        high_risk_files: summary.high_risk_files,
        histogram: Some(histogram),
        halstead: None, // Populated when halstead feature is enabled
        maintainability_index,
        technical_debt,
        files: file_complexities,
    })
}
