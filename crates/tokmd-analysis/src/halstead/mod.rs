use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::HalsteadMetrics;
use tokmd_types::{ExportData, FileKind, FileRow};

use tokmd_analysis_types::{AnalysisLimits, normalize_path};

const DEFAULT_MAX_FILE_BYTES: u64 = 128 * 1024;

mod operators;
mod tokenizer;

pub(crate) use operators::is_halstead_lang;
#[cfg(test)]
pub(crate) use operators::operators_for_lang;
#[cfg(test)]
pub(crate) use tokenizer::FileTokenCounts;
pub(crate) use tokenizer::tokenize_for_halstead;

#[cfg(test)]
#[path = "tests.rs"]
mod moved_tests;

/// Build aggregated Halstead metrics from source files.
pub(crate) fn build_halstead_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    limits: &AnalysisLimits,
) -> Result<HalsteadMetrics> {
    let mut row_map: BTreeMap<String, &FileRow> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        row_map.insert(normalize_path(&row.path, root), row);
    }

    let mut all_operators: BTreeMap<String, usize> = BTreeMap::new();
    let mut all_operands: BTreeSet<String> = BTreeSet::new();
    let mut total_ops = 0usize;
    let mut total_opds = 0usize;
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
        if !is_halstead_lang(&row.lang) {
            continue;
        }

        let path = root.join(rel);
        let bytes = match crate::content::io::read_head(&path, per_file_limit) {
            Ok(b) => b,
            Err(_) => continue,
        };
        total_bytes += bytes.len() as u64;

        let text = match crate::content::io::as_text(&bytes) {
            Some(t) => t,
            None => continue,
        };
        let lang_lower = row.lang.to_lowercase();
        let counts = tokenize_for_halstead(text, &lang_lower);

        for (op, count) in counts.operators {
            *all_operators.entry(op).or_insert(0) += count;
        }
        all_operands.extend(counts.operands);
        total_ops += counts.total_operators;
        total_opds += counts.total_operands;
    }

    let n1 = all_operators.len(); // distinct operators
    let n2 = all_operands.len(); // distinct operands
    let vocabulary = n1 + n2;
    let length = total_ops + total_opds;

    let volume = if vocabulary > 0 {
        length as f64 * (vocabulary as f64).log2()
    } else {
        0.0
    };

    let difficulty = if n2 > 0 {
        (n1 as f64 / 2.0) * (total_opds as f64 / n2 as f64)
    } else {
        0.0
    };

    let effort = difficulty * volume;
    let time_seconds = effort / 18.0;
    let estimated_bugs = volume / 3000.0;

    Ok(HalsteadMetrics {
        distinct_operators: n1,
        distinct_operands: n2,
        total_operators: total_ops,
        total_operands: total_opds,
        vocabulary,
        length,
        volume: round_f64(volume, 2),
        difficulty: round_f64(difficulty, 2),
        effort: round_f64(effort, 2),
        time_seconds: round_f64(time_seconds, 2),
        estimated_bugs: round_f64(estimated_bugs, 4),
    })
}

/// Round an f64 to a given number of decimal places.
pub(crate) fn round_f64(val: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (val * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_rust() {
        let code = r#"
fn add(a: i32, b: i32) -> i32 {
    if a > b {
        a + b
    } else {
        a - b
    }
}
"#;
        let counts = tokenize_for_halstead(code, "rust");
        assert!(counts.total_operators > 0);
        assert!(counts.total_operands > 0);
        assert!(!counts.operators.is_empty());
        assert!(!counts.operands.is_empty());
    }

    #[test]
    fn test_tokenize_python() {
        let code = r#"
def add(a, b):
    if a > b:
        return a + b
    else:
        return a - b
"#;
        let counts = tokenize_for_halstead(code, "python");
        assert!(counts.total_operators > 0);
        assert!(counts.total_operands > 0);
    }

    #[test]
    fn test_halstead_computation() {
        // Known values: 2 distinct operators, 3 distinct operands
        // n1=2, n2=3, N1=4, N2=6
        // vocabulary = 5, length = 10
        // volume = 10 * log2(5) ≈ 23.22
        // difficulty = (2/2) * (6/3) = 2.0
        // effort = 2.0 * 23.22 ≈ 46.44
        let n1 = 2;
        let n2 = 3;
        let total_ops = 4;
        let total_opds = 6;
        let vocabulary = n1 + n2;
        let length = total_ops + total_opds;
        let volume = length as f64 * (vocabulary as f64).log2();
        let difficulty = (n1 as f64 / 2.0) * (total_opds as f64 / n2 as f64);
        let effort = difficulty * volume;

        assert!((volume - 23.22).abs() < 0.1);
        assert!((difficulty - 2.0).abs() < 0.01);
        assert!((effort - 46.44).abs() < 0.2);
    }

    #[test]
    fn test_empty_input() {
        let counts = tokenize_for_halstead("", "rust");
        assert_eq!(counts.total_operators, 0);
        assert_eq!(counts.total_operands, 0);
    }

    #[test]
    fn tokenize_handles_non_ascii_after_operator() {
        let counts = tokenize_for_halstead(r#"let locale = ("日本", "ja");"#, "rust");
        assert!(counts.total_operators > 0);
    }

    #[test]
    fn test_tokenize_cjk_panic() {
        // Ensure that slicing a multi-byte character does not panic.
        let code = "!你好";
        let counts = tokenize_for_halstead(code, "rust");
        // '!' is a known operator. '你好' should be captured as an operand.
        assert_eq!(counts.total_operators, 1);
        assert!(counts.operators.contains_key("!"));
    }
}
