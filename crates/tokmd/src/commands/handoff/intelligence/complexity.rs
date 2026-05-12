//! Lightweight source complexity estimates for handoff intelligence.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use tokmd_types::{ExportData, FileKind, FileRow, HandoffComplexity};

use super::super::round_f64;

#[path = "complexity/language.rs"]
mod language;

use language::{count_functions_simple, estimate_cyclomatic_simple, is_analyzable_lang};

/// Maximum number of files to analyze for complexity.
const MAX_COMPLEXITY_FILES: usize = 50;
/// Maximum bytes to read per file for complexity analysis.
const MAX_COMPLEXITY_BYTES: usize = 128 * 1024;

/// Build complexity metrics by reading source files and counting functions/branching.
pub(super) fn build_simple_complexity(export: &ExportData) -> HandoffComplexity {
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

/// Read file contents up to a byte cap. Returns None if unreadable.
fn read_file_capped(path: &Path, max_bytes: usize) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut buf = vec![0u8; max_bytes];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);
    String::from_utf8(buf).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
