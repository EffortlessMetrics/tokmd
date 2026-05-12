//! Markdown rendering for diff receipts.
//!
//! This module owns human diff table rendering and color formatting. The parent
//! diff module owns row/totals computation and JSON receipt construction.

use std::fmt::Write as FmtWrite;

use tokmd_types::{DiffRow, DiffTotals};

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

#[cfg(test)]
mod tests {
    use super::format_delta;

    #[test]
    fn test_format_delta() {
        // Kills mutants in format_delta function
        assert_eq!(format_delta(5), "+5");
        assert_eq!(format_delta(0), "0");
        assert_eq!(format_delta(-3), "-3");
    }
}
