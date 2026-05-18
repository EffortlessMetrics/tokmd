//! Ratio table rendering for derived analysis reports.

use std::fmt::Write;

use tokmd_analysis_types::DerivedReport;

use crate::analysis::markdown::{fmt_f64, fmt_pct};

pub(super) fn render_ratios(out: &mut String, derived: &DerivedReport) {
    out.push_str("## Ratios\n\n");
    out.push_str("|Metric|Value|\n");
    out.push_str("|---|---:|\n");
    let _ = writeln!(
        out,
        "|Doc density|{}|",
        fmt_pct(derived.doc_density.total.ratio)
    );
    let _ = writeln!(
        out,
        "|Whitespace ratio|{}|",
        fmt_pct(derived.whitespace.total.ratio)
    );
    let _ = writeln!(
        out,
        "|Bytes per line|{}|\n",
        fmt_f64(derived.verbosity.total.rate, 2)
    );

    render_doc_density_by_language(out, derived);
    render_whitespace_by_language(out, derived);
    render_verbosity_by_language(out, derived);
}

fn render_doc_density_by_language(out: &mut String, derived: &DerivedReport) {
    out.push_str("### Doc density by language\n\n");
    out.push_str("|Lang|Doc%|Comments|Code|\n");
    out.push_str("|---|---:|---:|---:|\n");
    for row in derived.doc_density.by_lang.iter().take(10) {
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|",
            row.key,
            fmt_pct(row.ratio),
            row.numerator,
            row.denominator.saturating_sub(row.numerator)
        );
    }
    out.push('\n');
}

fn render_whitespace_by_language(out: &mut String, derived: &DerivedReport) {
    out.push_str("### Whitespace ratio by language\n\n");
    out.push_str("|Lang|Blank%|Blanks|Code+Comments|\n");
    out.push_str("|---|---:|---:|---:|\n");
    for row in derived.whitespace.by_lang.iter().take(10) {
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|",
            row.key,
            fmt_pct(row.ratio),
            row.numerator,
            row.denominator
        );
    }
    out.push('\n');
}

fn render_verbosity_by_language(out: &mut String, derived: &DerivedReport) {
    out.push_str("### Verbosity by language\n\n");
    out.push_str("|Lang|Bytes/Line|Bytes|Lines|\n");
    out.push_str("|---|---:|---:|---:|\n");
    for row in derived.verbosity.by_lang.iter().take(10) {
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|",
            row.key,
            fmt_f64(row.rate, 2),
            row.numerator,
            row.denominator
        );
    }
    out.push('\n');
}
