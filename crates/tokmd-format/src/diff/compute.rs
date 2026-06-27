//! Diff row and total computation.
//!
//! This module owns the deterministic transformation from two language
//! reports into language-level diff rows and aggregate totals. Rendering stays
//! in the parent diff module.

use tokmd_types::{DiffRow, DiffTotals, LangReport};

/// Compute diff rows between two language reports.
///
/// Each row captures the delta between old and new values for a language.
/// Languages with no change are omitted.
///
/// # Examples
///
/// ```
/// use tokmd_types::{LangReport, LangRow, Totals, ChildrenMode};
/// use tokmd_format::compute_diff_rows;
///
/// let from = LangReport {
///     rows: vec![LangRow {
///         lang: "Rust".into(), code: 100, lines: 150,
///         files: 5, bytes: 4000, tokens: 1000, avg_lines: 30,
///     }],
///     total: Totals { code: 100, lines: 150, files: 5, bytes: 4000, tokens: 1000, avg_lines: 30 },
///     with_files: true, children: ChildrenMode::Collapse, top: 0,
/// };
/// let to = LangReport {
///     rows: vec![LangRow {
///         lang: "Rust".into(), code: 200, lines: 300,
///         files: 8, bytes: 8000, tokens: 2000, avg_lines: 38,
///     }],
///     total: Totals { code: 200, lines: 300, files: 8, bytes: 8000, tokens: 2000, avg_lines: 38 },
///     with_files: true, children: ChildrenMode::Collapse, top: 0,
/// };
///
/// let rows = compute_diff_rows(&from, &to);
/// assert_eq!(rows.len(), 1);
/// assert_eq!(rows[0].delta_code, 100);
/// ```
pub fn compute_diff_rows(from_report: &LangReport, to_report: &LangReport) -> Vec<DiffRow> {
    // Collect all languages from both reports
    let mut all_langs: Vec<&str> = from_report
        .rows
        .iter()
        .chain(to_report.rows.iter())
        .map(|r| r.lang.as_str())
        .collect();
    all_langs.sort();
    all_langs.dedup();

    all_langs
        .into_iter()
        .filter_map(|lang_name| {
            let old_row = from_report.rows.iter().find(|r| r.lang == lang_name);
            let new_row = to_report.rows.iter().find(|r| r.lang == lang_name);

            let old_code = old_row.map(|r| r.code).unwrap_or(0);
            let old_lines = old_row.map(|r| r.lines).unwrap_or(0);
            let old_files = old_row.map(|r| r.files).unwrap_or(0);
            let old_bytes = old_row.map(|r| r.bytes).unwrap_or(0);
            let old_tokens = old_row.map(|r| r.tokens).unwrap_or(0);

            let new_code = new_row.map(|r| r.code).unwrap_or(0);
            let new_lines = new_row.map(|r| r.lines).unwrap_or(0);
            let new_files = new_row.map(|r| r.files).unwrap_or(0);
            let new_bytes = new_row.map(|r| r.bytes).unwrap_or(0);
            let new_tokens = new_row.map(|r| r.tokens).unwrap_or(0);

            // Skip if no change
            if old_code == new_code
                && old_lines == new_lines
                && old_files == new_files
                && old_bytes == new_bytes
                && old_tokens == new_tokens
            {
                return None;
            }

            Some(DiffRow {
                lang: lang_name.to_string(),
                old_code,
                new_code,
                delta_code: new_code as i64 - old_code as i64,
                old_lines,
                new_lines,
                delta_lines: new_lines as i64 - old_lines as i64,
                old_files,
                new_files,
                delta_files: new_files as i64 - old_files as i64,
                old_bytes,
                new_bytes,
                delta_bytes: new_bytes as i64 - old_bytes as i64,
                old_tokens,
                new_tokens,
                delta_tokens: new_tokens as i64 - old_tokens as i64,
            })
        })
        .collect()
}

/// Compute totals from diff rows.
///
/// # Examples
///
/// ```
/// use tokmd_types::DiffRow;
/// use tokmd_format::compute_diff_totals;
///
/// let rows = vec![DiffRow {
///     lang: "Rust".into(),
///     old_code: 100, new_code: 200, delta_code: 100,
///     old_lines: 150, new_lines: 300, delta_lines: 150,
///     old_files: 5, new_files: 8, delta_files: 3,
///     old_bytes: 4000, new_bytes: 8000, delta_bytes: 4000,
///     old_tokens: 1000, new_tokens: 2000, delta_tokens: 1000,
/// }];
///
/// let totals = compute_diff_totals(&rows);
/// assert_eq!(totals.delta_code, 100);
/// assert_eq!(totals.delta_tokens, 1000);
/// ```
pub fn compute_diff_totals(rows: &[DiffRow]) -> DiffTotals {
    let mut totals = DiffTotals {
        old_code: 0,
        new_code: 0,
        delta_code: 0,
        old_lines: 0,
        new_lines: 0,
        delta_lines: 0,
        old_files: 0,
        new_files: 0,
        delta_files: 0,
        old_bytes: 0,
        new_bytes: 0,
        delta_bytes: 0,
        old_tokens: 0,
        new_tokens: 0,
        delta_tokens: 0,
    };

    for row in rows {
        totals.old_code += row.old_code;
        totals.new_code += row.new_code;
        totals.delta_code += row.delta_code;
        totals.old_lines += row.old_lines;
        totals.new_lines += row.new_lines;
        totals.delta_lines += row.delta_lines;
        totals.old_files += row.old_files;
        totals.new_files += row.new_files;
        totals.delta_files += row.delta_files;
        totals.old_bytes += row.old_bytes;
        totals.new_bytes += row.new_bytes;
        totals.delta_bytes += row.delta_bytes;
        totals.old_tokens += row.old_tokens;
        totals.new_tokens += row.new_tokens;
        totals.delta_tokens += row.delta_tokens;
    }

    totals
}
