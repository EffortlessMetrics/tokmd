use std::collections::BTreeMap;
use std::collections::BTreeSet;
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
use details::{
    detect_fn_spans_c_style, detect_fn_spans_go, detect_fn_spans_js, detect_fn_spans_python,
    detect_fn_spans_rust,
};
use functions::count_functions;
#[cfg(test)]
use functions::{count_python_functions, count_rust_functions, is_rust_fn_start};
#[cfg(test)]
pub(crate) use histogram::generate_complexity_histogram;
pub(crate) use histogram::generate_complexity_histogram_for_files;
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
    let mut per_file_max_cyclomatic: Vec<usize> = Vec::new();
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
        let cyclomatic_analysis =
            crate::content::complexity::estimate_cyclomatic_complexity(&text, lang_mapped);
        let (cyclomatic, max_function_cyclomatic) = if cyclomatic_analysis.function_count > 0 {
            (cyclomatic_analysis.total_cc, cyclomatic_analysis.max_cc)
        } else if function_count > 0 {
            let file_level = estimate_cyclomatic(&row.lang, &text);
            (file_level, file_level)
        } else {
            (0, 0)
        };

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
            max_function_cyclomatic,
            cognitive_complexity,
            max_nesting,
        );

        let functions = if detail_functions {
            Some(extract_function_details(&row.lang, &text))
        } else {
            None
        };

        per_file_max_cyclomatic.push(max_function_cyclomatic);
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
    let summary = summarize_file_complexities(&file_complexities, &per_file_max_cyclomatic);

    // Generate histogram from all files before truncating
    let histogram =
        generate_complexity_histogram_for_files(&file_complexities, &per_file_max_cyclomatic, 5);

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

pub(crate) fn bounded_complexity_warnings(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    limits: &AnalysisLimits,
) -> Vec<String> {
    let mut row_map: BTreeMap<String, &FileRow> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        row_map.insert(normalize_path(&row.path, root), row);
    }

    let scoped_files: BTreeSet<String> = files
        .iter()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect();
    let per_file_limit = limits.max_file_bytes.unwrap_or(DEFAULT_MAX_FILE_BYTES);
    let limit_label = if limits.max_file_bytes.is_some() {
        format!("max_file_bytes={per_file_limit}")
    } else {
        format!("default max_file_bytes={per_file_limit}")
    };

    let mut eligible_files = 0usize;
    let mut clipped_files = 0usize;
    let mut bounded_bytes = 0u64;
    let mut total_estimated_read_bytes = 0u64;

    for (path, row) in row_map {
        if !scoped_files.contains(&path) || !is_complexity_lang(&row.lang) {
            continue;
        }

        eligible_files += 1;
        let row_bytes = row.bytes as u64;
        let read_bytes = row_bytes.min(per_file_limit);
        total_estimated_read_bytes = total_estimated_read_bytes.saturating_add(read_bytes);

        if row_bytes > per_file_limit {
            clipped_files += 1;
            bounded_bytes = bounded_bytes.saturating_add(row_bytes.saturating_sub(per_file_limit));
        }
    }

    let mut warnings = Vec::new();
    if clipped_files > 0 {
        warnings.push(format!(
            "complexity scan bounded: {clipped_files} of {eligible_files} eligible file(s) exceed {limit_label}; complexity metrics are partial"
        ));
    }
    if let Some(max_bytes) = limits.max_bytes
        && total_estimated_read_bytes > max_bytes
    {
        warnings.push(format!(
            "complexity scan bounded: max_bytes={max_bytes} stops before all eligible complexity files are scanned; complexity metrics are partial"
        ));
    }
    if bounded_bytes > 0 {
        warnings.push(format!(
            "complexity scan bounded: at least {bounded_bytes} byte(s) of eligible source were outside the scanned content window"
        ));
    }

    warnings
}
