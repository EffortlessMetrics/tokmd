//! Diff row and total computation.
//!
//! This module owns the deterministic transformation from two language
//! reports into language-level diff rows and aggregate totals. Rendering stays
//! in the parent diff module.

use tokmd_types::{DiffRow, DiffTotals, LangReport, LangRow};

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
    let mut all_langs: Vec<String> = from_report
        .rows
        .iter()
        .chain(to_report.rows.iter())
        .map(|r| r.lang.clone())
        .collect();
    all_langs.sort();
    all_langs.dedup();

    all_langs
        .into_iter()
        .filter_map(|lang_name| {
            let old_row = from_report.rows.iter().find(|r| r.lang == lang_name);
            let new_row = to_report.rows.iter().find(|r| r.lang == lang_name);

            let old = old_row.cloned().unwrap_or_else(|| LangRow {
                lang: lang_name.clone(),
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            });
            let new = new_row.cloned().unwrap_or_else(|| LangRow {
                lang: lang_name.clone(),
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            });

            // Skip if no change
            if old.code == new.code
                && old.lines == new.lines
                && old.files == new.files
                && old.bytes == new.bytes
                && old.tokens == new.tokens
            {
                return None;
            }

            Some(DiffRow {
                lang: lang_name,
                old_code: old.code,
                new_code: new.code,
                delta_code: new.code as i64 - old.code as i64,
                old_lines: old.lines,
                new_lines: new.lines,
                delta_lines: new.lines as i64 - old.lines as i64,
                old_files: old.files,
                new_files: new.files,
                delta_files: new.files as i64 - old.files as i64,
                old_bytes: old.bytes,
                new_bytes: new.bytes,
                delta_bytes: new.bytes as i64 - old.bytes as i64,
                old_tokens: old.tokens,
                new_tokens: new.tokens,
                delta_tokens: new.tokens as i64 - old.tokens as i64,
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tokmd_types::DiffRow;

    fn arb_diff_row() -> impl Strategy<Value = DiffRow> {
        (
            0usize..10000,
            0usize..10000,
            0usize..10000,
            0usize..10000,
            0usize..1000,
            0usize..1000,
            0usize..1000000,
            0usize..1000000,
            0usize..100000,
            0usize..100000,
        )
            .prop_map(
                |(
                    old_code,
                    new_code,
                    old_lines,
                    new_lines,
                    old_files,
                    new_files,
                    old_bytes,
                    new_bytes,
                    old_tokens,
                    new_tokens,
                )| {
                    DiffRow {
                        lang: "TestLang".into(),
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
                    }
                },
            )
    }

    proptest! {
        #[test]
        fn diff_totals_preserves_row_sums(rows in prop::collection::vec(arb_diff_row(), 0..10)) {
            let totals = compute_diff_totals(&rows);

            let sum_old_code: usize = rows.iter().map(|r| r.old_code).sum();
            let sum_new_code: usize = rows.iter().map(|r| r.new_code).sum();
            let sum_delta_code: i64 = rows.iter().map(|r| r.delta_code).sum();

            let sum_old_lines: usize = rows.iter().map(|r| r.old_lines).sum();
            let sum_new_lines: usize = rows.iter().map(|r| r.new_lines).sum();
            let sum_delta_lines: i64 = rows.iter().map(|r| r.delta_lines).sum();

            let sum_old_files: usize = rows.iter().map(|r| r.old_files).sum();
            let sum_new_files: usize = rows.iter().map(|r| r.new_files).sum();
            let sum_delta_files: i64 = rows.iter().map(|r| r.delta_files).sum();

            let sum_old_bytes: usize = rows.iter().map(|r| r.old_bytes).sum();
            let sum_new_bytes: usize = rows.iter().map(|r| r.new_bytes).sum();
            let sum_delta_bytes: i64 = rows.iter().map(|r| r.delta_bytes).sum();

            let sum_old_tokens: usize = rows.iter().map(|r| r.old_tokens).sum();
            let sum_new_tokens: usize = rows.iter().map(|r| r.new_tokens).sum();
            let sum_delta_tokens: i64 = rows.iter().map(|r| r.delta_tokens).sum();

            prop_assert_eq!(totals.old_code, sum_old_code);
            prop_assert_eq!(totals.new_code, sum_new_code);
            prop_assert_eq!(totals.delta_code, sum_delta_code);

            prop_assert_eq!(totals.old_lines, sum_old_lines);
            prop_assert_eq!(totals.new_lines, sum_new_lines);
            prop_assert_eq!(totals.delta_lines, sum_delta_lines);

            prop_assert_eq!(totals.old_files, sum_old_files);
            prop_assert_eq!(totals.new_files, sum_new_files);
            prop_assert_eq!(totals.delta_files, sum_delta_files);

            prop_assert_eq!(totals.old_bytes, sum_old_bytes);
            prop_assert_eq!(totals.new_bytes, sum_new_bytes);
            prop_assert_eq!(totals.delta_bytes, sum_delta_bytes);

            prop_assert_eq!(totals.old_tokens, sum_old_tokens);
            prop_assert_eq!(totals.new_tokens, sum_new_tokens);
            prop_assert_eq!(totals.delta_tokens, sum_delta_tokens);
        }

        #[test]
        fn diff_totals_maintains_delta_invariants(rows in prop::collection::vec(arb_diff_row(), 0..10)) {
            let totals = compute_diff_totals(&rows);

            prop_assert_eq!(totals.delta_code, totals.new_code as i64 - totals.old_code as i64);
            prop_assert_eq!(totals.delta_lines, totals.new_lines as i64 - totals.old_lines as i64);
            prop_assert_eq!(totals.delta_files, totals.new_files as i64 - totals.old_files as i64);
            prop_assert_eq!(totals.delta_bytes, totals.new_bytes as i64 - totals.old_bytes as i64);
            prop_assert_eq!(totals.delta_tokens, totals.new_tokens as i64 - totals.old_tokens as i64);
        }

        #[test]
        fn diff_totals_empty_is_zero(_dummy in 0..1u8) {
            let totals = compute_diff_totals(&[]);
            let zero = DiffTotals::default();
            prop_assert_eq!(totals, zero);
        }
    }
}
