//! Language summary table rendering.
//!
//! This module owns Markdown and TSV rendering for `LangReport`. The parent
//! summary module keeps command dispatch and public helper exports.

use std::fmt::Write as FmtWrite;

use tokmd_types::LangReport;

pub(super) fn render_lang_md(report: &LangReport) -> String {
    // Heuristic: (rows + 3) * 80 chars per row
    let mut s = String::with_capacity((report.rows.len() + 3) * 80);

    if report.with_files {
        s.push_str("|Lang|Code|Lines|Files|Bytes|Tokens|Avg|\n");
        s.push_str("|---|---:|---:|---:|---:|---:|---:|\n");
        for r in &report.rows {
            let _ = writeln!(
                s,
                "|{}|{}|{}|{}|{}|{}|{}|",
                r.lang, r.code, r.lines, r.files, r.bytes, r.tokens, r.avg_lines
            );
        }
        let _ = writeln!(
            s,
            "|**Total**|{}|{}|{}|{}|{}|{}|",
            report.total.code,
            report.total.lines,
            report.total.files,
            report.total.bytes,
            report.total.tokens,
            report.total.avg_lines
        );
    } else {
        s.push_str("|Lang|Code|Lines|Bytes|Tokens|\n");
        s.push_str("|---|---:|---:|---:|---:|\n");
        for r in &report.rows {
            let _ = writeln!(
                s,
                "|{}|{}|{}|{}|{}|",
                r.lang, r.code, r.lines, r.bytes, r.tokens
            );
        }
        let _ = writeln!(
            s,
            "|**Total**|{}|{}|{}|{}|",
            report.total.code, report.total.lines, report.total.bytes, report.total.tokens
        );
    }

    s
}

pub(super) fn render_lang_tsv(report: &LangReport) -> String {
    // Heuristic: (rows + 2) * 64 chars per row
    let mut s = String::with_capacity((report.rows.len() + 2) * 64);

    if report.with_files {
        s.push_str("Lang\tCode\tLines\tFiles\tBytes\tTokens\tAvg\n");
        for r in &report.rows {
            let _ = writeln!(
                s,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                r.lang, r.code, r.lines, r.files, r.bytes, r.tokens, r.avg_lines
            );
        }
        let _ = writeln!(
            s,
            "Total\t{}\t{}\t{}\t{}\t{}\t{}",
            report.total.code,
            report.total.lines,
            report.total.files,
            report.total.bytes,
            report.total.tokens,
            report.total.avg_lines
        );
    } else {
        s.push_str("Lang\tCode\tLines\tBytes\tTokens\n");
        for r in &report.rows {
            let _ = writeln!(
                s,
                "{}\t{}\t{}\t{}\t{}",
                r.lang, r.code, r.lines, r.bytes, r.tokens
            );
        }
        let _ = writeln!(
            s,
            "Total\t{}\t{}\t{}\t{}",
            report.total.code, report.total.lines, report.total.bytes, report.total.tokens
        );
    }

    s
}
