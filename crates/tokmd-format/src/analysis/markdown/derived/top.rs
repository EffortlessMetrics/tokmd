//! Top-offender table rendering for derived analysis reports.

use std::fmt::Write;

use tokmd_analysis_types::{DerivedReport, FileStatRow};

use crate::analysis::markdown::{fmt_f64, fmt_pct};

pub(super) fn render_top_offenders(out: &mut String, derived: &DerivedReport) {
    out.push_str("## Top offenders\n\n");

    render_top_table(out, "Largest files by lines", &derived.top.largest_lines);
    render_top_table(out, "Largest files by tokens", &derived.top.largest_tokens);
    render_top_table(out, "Largest files by bytes", &derived.top.largest_bytes);
    render_top_table(
        out,
        "Least documented (min LOC)",
        &derived.top.least_documented,
    );
    render_top_table(out, "Most dense (bytes/line)", &derived.top.most_dense);
}

fn render_top_table(out: &mut String, title: &str, rows: &[FileStatRow]) {
    let _ = writeln!(out, "### {title}\n");
    out.push_str(&render_file_table(rows));
    out.push('\n');
}

fn render_file_table(rows: &[FileStatRow]) -> String {
    let mut out = String::with_capacity((rows.len() + 3) * 80);
    out.push_str("|Path|Lang|Lines|Code|Bytes|Tokens|Doc%|B/Line|\n");
    out.push_str("|---|---|---:|---:|---:|---:|---:|---:|\n");
    for row in rows {
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|{}|{}|{}|{}|",
            row.path,
            row.lang,
            row.lines,
            row.code,
            row.bytes,
            row.tokens,
            row.doc_pct.map(fmt_pct).unwrap_or_else(|| "-".to_string()),
            row.bytes_per_line
                .map(|v| fmt_f64(v, 2))
                .unwrap_or_else(|| "-".to_string())
        );
    }
    out
}
