use std::fmt::Write as FmtWrite;

use crate::now_ms;
// -----------------
// Diff output
// -----------------

use tokmd_types::{DiffReceipt, DiffRow, DiffTotals, LangReport, LangRow, ToolInfo};

/// Compute diff rows from two lang reports.
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

fn format_delta(delta: i64) -> String {
    if delta > 0 {
        format!("+{}", delta)
    } else {
        delta.to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffColorMode {
    Off,
    Ansi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffRenderOptions {
    pub compact: bool,
    pub color: DiffColorMode,
}

impl Default for DiffRenderOptions {
    fn default() -> Self {
        Self {
            compact: false,
            color: DiffColorMode::Off,
        }
    }
}

fn format_delta_colored(delta: i64, mode: DiffColorMode) -> String {
    let raw = format_delta(delta);
    if mode == DiffColorMode::Off {
        return raw;
    }
    if delta > 0 {
        format!("\x1b[32m{}\x1b[0m", raw)
    } else if delta < 0 {
        format!("\x1b[31m{}\x1b[0m", raw)
    } else {
        format!("\x1b[33m{}\x1b[0m", raw)
    }
}

fn format_pct_delta_colored(delta_pct: f64, mode: DiffColorMode) -> String {
    let raw = format!("{:+.1}%", delta_pct);
    if mode == DiffColorMode::Off {
        return raw;
    }
    if delta_pct > 0.0 {
        format!("\x1b[32m{}\x1b[0m", raw)
    } else if delta_pct < 0.0 {
        format!("\x1b[31m{}\x1b[0m", raw)
    } else {
        format!("\x1b[33m{}\x1b[0m", raw)
    }
}

fn percent_change(old: usize, new: usize) -> f64 {
    if old > 0 {
        ((new as f64 - old as f64) / old as f64) * 100.0
    } else if new > 0 {
        100.0
    } else {
        0.0
    }
}

/// Render diff as Markdown table with optional compact/color behavior.
pub fn render_diff_md_with_options(
    from_source: &str,
    to_source: &str,
    rows: &[DiffRow],
    totals: &DiffTotals,
    options: DiffRenderOptions,
) -> String {
    // Heuristic: (rows + 20) * 80 chars per row
    let mut s = String::with_capacity((rows.len() + 20) * 80);

    let _ = writeln!(s, "## Diff: {} → {}", from_source, to_source);
    s.push('\n');

    let languages_added = rows
        .iter()
        .filter(|r| r.old_code == 0 && r.new_code > 0)
        .count();
    let languages_removed = rows
        .iter()
        .filter(|r| r.old_code > 0 && r.new_code == 0)
        .count();
    let languages_modified = rows
        .len()
        .saturating_sub(languages_added + languages_removed);

    if options.compact {
        s.push_str("### Summary\n\n");
        s.push_str("|Metric|Value|\n");
        s.push_str("|---|---:|\n");
        let _ = writeln!(s, "|From LOC|{}|", totals.old_code);
        let _ = writeln!(s, "|To LOC|{}|", totals.new_code);
        let _ = writeln!(
            s,
            "|Delta LOC|{}|",
            format_delta_colored(totals.delta_code, options.color)
        );
        let _ = writeln!(
            s,
            "|LOC Change|{}|",
            format_pct_delta_colored(
                percent_change(totals.old_code, totals.new_code),
                options.color
            )
        );
        let _ = writeln!(
            s,
            "|Delta Lines|{}|",
            format_delta_colored(totals.delta_lines, options.color)
        );
        let _ = writeln!(
            s,
            "|Delta Files|{}|",
            format_delta_colored(totals.delta_files, options.color)
        );
        let _ = writeln!(
            s,
            "|Delta Bytes|{}|",
            format_delta_colored(totals.delta_bytes, options.color)
        );
        let _ = writeln!(
            s,
            "|Delta Tokens|{}|",
            format_delta_colored(totals.delta_tokens, options.color)
        );
        let _ = writeln!(s, "|Languages changed|{}|", rows.len());
        let _ = writeln!(s, "|Languages added|{}|", languages_added);
        let _ = writeln!(s, "|Languages removed|{}|", languages_removed);
        let _ = writeln!(s, "|Languages modified|{}|", languages_modified);
        return s;
    }

    // Summary comparison table
    s.push_str("### Summary\n\n");
    s.push_str("|Metric|From|To|Delta|Change|\n");
    s.push_str("|---|---:|---:|---:|---:|\n");

    let _ = writeln!(
        s,
        "|LOC|{}|{}|{}|{}|",
        totals.old_code,
        totals.new_code,
        format_delta_colored(totals.delta_code, options.color),
        format_pct_delta_colored(
            percent_change(totals.old_code, totals.new_code),
            options.color
        )
    );
    let _ = writeln!(
        s,
        "|Lines|{}|{}|{}|{}|",
        totals.old_lines,
        totals.new_lines,
        format_delta_colored(totals.delta_lines, options.color),
        format_pct_delta_colored(
            percent_change(totals.old_lines, totals.new_lines),
            options.color
        )
    );
    let _ = writeln!(
        s,
        "|Files|{}|{}|{}|{}|",
        totals.old_files,
        totals.new_files,
        format_delta_colored(totals.delta_files, options.color),
        format_pct_delta_colored(
            percent_change(totals.old_files, totals.new_files),
            options.color
        )
    );
    let _ = writeln!(
        s,
        "|Bytes|{}|{}|{}|{}|",
        totals.old_bytes,
        totals.new_bytes,
        format_delta_colored(totals.delta_bytes, options.color),
        format_pct_delta_colored(
            percent_change(totals.old_bytes, totals.new_bytes),
            options.color
        )
    );
    let _ = writeln!(
        s,
        "|Tokens|{}|{}|{}|{}|",
        totals.old_tokens,
        totals.new_tokens,
        format_delta_colored(totals.delta_tokens, options.color),
        format_pct_delta_colored(
            percent_change(totals.old_tokens, totals.new_tokens),
            options.color
        )
    );
    s.push('\n');

    s.push_str("### Language Movement\n\n");
    s.push_str("|Type|Count|\n");
    s.push_str("|---|---:|\n");
    let _ = writeln!(s, "|Changed|{}|", rows.len());
    let _ = writeln!(s, "|Added|{}|", languages_added);
    let _ = writeln!(s, "|Removed|{}|", languages_removed);
    let _ = writeln!(s, "|Modified|{}|", languages_modified);
    s.push('\n');

    // Detailed language breakdown
    s.push_str("### Language Breakdown\n\n");
    s.push_str("|Language|Old LOC|New LOC|Delta|\n");
    s.push_str("|---|---:|---:|---:|\n");

    for row in rows {
        let _ = writeln!(
            s,
            "|{}|{}|{}|{}|",
            row.lang,
            row.old_code,
            row.new_code,
            format_delta_colored(row.delta_code, options.color)
        );
    }

    let _ = writeln!(
        s,
        "|**Total**|{}|{}|{}|",
        totals.old_code,
        totals.new_code,
        format_delta_colored(totals.delta_code, options.color)
    );

    s
}

/// Render diff as Markdown table.
pub fn render_diff_md(
    from_source: &str,
    to_source: &str,
    rows: &[DiffRow],
    totals: &DiffTotals,
) -> String {
    render_diff_md_with_options(
        from_source,
        to_source,
        rows,
        totals,
        DiffRenderOptions::default(),
    )
}

/// Create a DiffReceipt for JSON output.
pub fn create_diff_receipt(
    from_source: &str,
    to_source: &str,
    rows: Vec<DiffRow>,
    totals: DiffTotals,
) -> DiffReceipt {
    DiffReceipt {
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "diff".to_string(),
        from_source: from_source.to_string(),
        to_source: to_source.to_string(),
        diff_rows: rows,
        totals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_settings::ChildrenMode;
    use tokmd_types::{LangReport, LangRow, Totals};
    #[test]
    fn test_render_diff_md_smoke() {
        // Kills mutants: render_diff_md -> String::new() / "xyzzy".into()
        let from = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 10,
                lines: 10,
                files: 1,
                bytes: 100,
                tokens: 20,
                avg_lines: 10,
            }],
            total: Totals {
                code: 10,
                lines: 10,
                files: 1,
                bytes: 100,
                tokens: 20,
                avg_lines: 10,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 12,
                lines: 12,
                files: 1,
                bytes: 120,
                tokens: 24,
                avg_lines: 12,
            }],
            total: Totals {
                code: 12,
                lines: 12,
                files: 1,
                bytes: 120,
                tokens: 24,
                avg_lines: 12,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lang, "Rust");
        assert_eq!(rows[0].delta_code, 2);

        let totals = compute_diff_totals(&rows);
        assert_eq!(totals.delta_code, 2);

        let md = render_diff_md("from", "to", &rows, &totals);

        assert!(!md.trim().is_empty(), "diff markdown must not be empty");
        assert!(md.contains("from"));
        assert!(md.contains("to"));
        assert!(md.contains("Rust"));
        assert!(md.contains("|LOC|"));
        assert!(md.contains("|Lines|"));
        assert!(md.contains("|Files|"));
        assert!(md.contains("|Bytes|"));
        assert!(md.contains("|Tokens|"));
        assert!(md.contains("### Language Movement"));
    }

    #[test]
    fn test_render_diff_md_compact_includes_movement_counts() {
        let from = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 10,
                lines: 10,
                files: 1,
                bytes: 100,
                tokens: 20,
                avg_lines: 10,
            }],
            total: Totals {
                code: 10,
                lines: 10,
                files: 1,
                bytes: 100,
                tokens: 20,
                avg_lines: 10,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };
        let to = LangReport {
            rows: vec![
                LangRow {
                    lang: "Rust".to_string(),
                    code: 12,
                    lines: 12,
                    files: 1,
                    bytes: 120,
                    tokens: 24,
                    avg_lines: 12,
                },
                LangRow {
                    lang: "Python".to_string(),
                    code: 8,
                    lines: 8,
                    files: 1,
                    bytes: 80,
                    tokens: 16,
                    avg_lines: 8,
                },
            ],
            total: Totals {
                code: 20,
                lines: 20,
                files: 2,
                bytes: 200,
                tokens: 40,
                avg_lines: 10,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };
        let rows = compute_diff_rows(&from, &to);
        let totals = compute_diff_totals(&rows);
        let md = render_diff_md_with_options(
            "from",
            "to",
            &rows,
            &totals,
            DiffRenderOptions {
                compact: true,
                color: DiffColorMode::Off,
            },
        );

        assert!(md.contains("|Delta Lines|"));
        assert!(md.contains("|Delta Files|"));
        assert!(md.contains("|Delta Bytes|"));
        assert!(md.contains("|Delta Tokens|"));
        assert!(md.contains("|Languages added|1|"));
        assert!(md.contains("|Languages modified|1|"));
    }

    #[test]
    fn test_compute_diff_rows_language_added() {
        // Tests language being added (was 0, now has code)
        let from = LangReport {
            rows: vec![],
            total: Totals {
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
            rows: vec![LangRow {
                lang: "Python".to_string(),
                code: 100,
                lines: 120,
                files: 5,
                bytes: 5000,
                tokens: 250,
                avg_lines: 24,
            }],
            total: Totals {
                code: 100,
                lines: 120,
                files: 5,
                bytes: 5000,
                tokens: 250,
                avg_lines: 24,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lang, "Python");
        assert_eq!(rows[0].old_code, 0);
        assert_eq!(rows[0].new_code, 100);
        assert_eq!(rows[0].delta_code, 100);
    }

    #[test]
    fn test_compute_diff_rows_language_removed() {
        // Tests language being removed (had code, now 0)
        let from = LangReport {
            rows: vec![LangRow {
                lang: "Go".to_string(),
                code: 50,
                lines: 60,
                files: 2,
                bytes: 2000,
                tokens: 125,
                avg_lines: 30,
            }],
            total: Totals {
                code: 50,
                lines: 60,
                files: 2,
                bytes: 2000,
                tokens: 125,
                avg_lines: 30,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let to = LangReport {
            rows: vec![],
            total: Totals {
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&from, &to);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].lang, "Go");
        assert_eq!(rows[0].old_code, 50);
        assert_eq!(rows[0].new_code, 0);
        assert_eq!(rows[0].delta_code, -50);
    }

    #[test]
    fn test_compute_diff_rows_unchanged_excluded() {
        // Tests that unchanged languages are excluded from diff
        let report = LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 100,
                lines: 100,
                files: 1,
                bytes: 1000,
                tokens: 250,
                avg_lines: 100,
            }],
            total: Totals {
                code: 100,
                lines: 100,
                files: 1,
                bytes: 1000,
                tokens: 250,
                avg_lines: 100,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };

        let rows = compute_diff_rows(&report, &report);
        assert!(rows.is_empty(), "unchanged languages should be excluded");
    }

    #[test]
    fn test_format_delta() {
        // Kills mutants in format_delta function
        assert_eq!(format_delta(5), "+5");
        assert_eq!(format_delta(0), "0");
        assert_eq!(format_delta(-3), "-3");
    }
}
