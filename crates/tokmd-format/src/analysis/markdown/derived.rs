//! Derived analysis Markdown rendering.
//!
//! This module owns totals, ratios, distribution, top-offender tables, density,
//! context-window, legacy COCOMO fallback, and integrity sections.

use std::fmt::Write;

use super::{effort, fmt_f64, fmt_pct};
use tokmd_analysis_types::{DerivedReport, EffortEstimateReport, FileStatRow};

pub(super) fn render_derived_report(
    out: &mut String,
    derived: &DerivedReport,
    effort_report: Option<&EffortEstimateReport>,
) {
    out.push_str("## Totals\n\n");
    out.push_str("|Files|Code|Comments|Blanks|Lines|Bytes|Tokens|\n");
    out.push_str("|---:|---:|---:|---:|---:|---:|---:|\n");
    let _ = writeln!(
        out,
        "|{}|{}|{}|{}|{}|{}|{}|\n",
        derived.totals.files,
        derived.totals.code,
        derived.totals.comments,
        derived.totals.blanks,
        derived.totals.lines,
        derived.totals.bytes,
        derived.totals.tokens
    );

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

    out.push_str("## Distribution\n\n");
    out.push_str("|Count|Min|Max|Mean|Median|P90|P99|Gini|\n");
    out.push_str("|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    let _ = writeln!(
        out,
        "|{}|{}|{}|{}|{}|{}|{}|{}|\n",
        derived.distribution.count,
        derived.distribution.min,
        derived.distribution.max,
        fmt_f64(derived.distribution.mean, 2),
        fmt_f64(derived.distribution.median, 2),
        fmt_f64(derived.distribution.p90, 2),
        fmt_f64(derived.distribution.p99, 2),
        fmt_f64(derived.distribution.gini, 4)
    );

    out.push_str("## File size histogram\n\n");
    out.push_str("|Bucket|Min|Max|Files|Pct|\n");
    out.push_str("|---|---:|---:|---:|---:|\n");
    for bucket in &derived.histogram {
        let max = bucket
            .max
            .map(|v| v.to_string())
            .unwrap_or_else(|| "∞".to_string());
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|{}|",
            bucket.label,
            bucket.min,
            max,
            bucket.files,
            fmt_pct(bucket.pct)
        );
    }
    out.push('\n');

    out.push_str("## Top offenders\n\n");

    out.push_str("### Largest files by lines\n\n");
    out.push_str(&render_file_table(&derived.top.largest_lines));
    out.push('\n');

    out.push_str("### Largest files by tokens\n\n");
    out.push_str(&render_file_table(&derived.top.largest_tokens));
    out.push('\n');

    out.push_str("### Largest files by bytes\n\n");
    out.push_str(&render_file_table(&derived.top.largest_bytes));
    out.push('\n');

    out.push_str("### Least documented (min LOC)\n\n");
    out.push_str(&render_file_table(&derived.top.least_documented));
    out.push('\n');

    out.push_str("### Most dense (bytes/line)\n\n");
    out.push_str(&render_file_table(&derived.top.most_dense));
    out.push('\n');

    out.push_str("## Structure\n\n");
    let _ = writeln!(
        out,
        "- Max depth: `{}`\n- Avg depth: `{}`\n",
        derived.nesting.max,
        fmt_f64(derived.nesting.avg, 2)
    );

    out.push_str("## Test density\n\n");
    let _ = writeln!(
        out,
        "- Test lines: `{}`\n- Prod lines: `{}`\n- Test ratio: `{}`\n",
        derived.test_density.test_lines,
        derived.test_density.prod_lines,
        fmt_pct(derived.test_density.ratio)
    );

    if let Some(todo) = &derived.todo {
        out.push_str("## TODOs\n\n");
        let _ = writeln!(
            out,
            "- Total: `{}`\n- Density (per KLOC): `{}`\n",
            todo.total,
            fmt_f64(todo.density_per_kloc, 2)
        );
        out.push_str("|Tag|Count|\n");
        out.push_str("|---|---:|\n");
        for tag in &todo.tags {
            let _ = writeln!(out, "|{}|{}|", tag.tag, tag.count);
        }
        out.push('\n');
    }

    out.push_str("## Boilerplate ratio\n\n");
    let _ = writeln!(
        out,
        "- Infra lines: `{}`\n- Logic lines: `{}`\n- Infra ratio: `{}`\n",
        derived.boilerplate.infra_lines,
        derived.boilerplate.logic_lines,
        fmt_pct(derived.boilerplate.ratio)
    );

    out.push_str("## Polyglot\n\n");
    let _ = writeln!(
        out,
        "- Languages: `{}`\n- Dominant: `{}` ({})\n- Entropy: `{}`\n",
        derived.polyglot.lang_count,
        derived.polyglot.dominant_lang,
        fmt_pct(derived.polyglot.dominant_pct),
        fmt_f64(derived.polyglot.entropy, 4)
    );

    out.push_str("## Reading time\n\n");
    let _ = writeln!(
        out,
        "- Minutes: `{}` ({} lines/min)\n",
        fmt_f64(derived.reading_time.minutes, 2),
        derived.reading_time.lines_per_minute
    );

    if let Some(context) = &derived.context_window {
        out.push_str("## Context window\n\n");
        let _ = writeln!(
            out,
            "- Window tokens: `{}`\n- Total tokens: `{}`\n- Utilization: `{}`\n- Fits: `{}`\n",
            context.window_tokens,
            context.total_tokens,
            fmt_pct(context.pct),
            context.fits
        );
    }

    // Prefer the richer top-level effort contract when present; fall back to
    // legacy derived COCOMO output for older receipts.
    if let Some(effort_report) = effort_report {
        effort::render_effort_report(out, effort_report);
    } else if let Some(cocomo) = &derived.cocomo {
        effort::render_legacy_cocomo_report(out, derived, cocomo);
    }

    out.push_str("## Integrity\n\n");
    let _ = writeln!(
        out,
        "- Hash: `{}` (`{}`)\n- Entries: `{}`\n",
        derived.integrity.hash, derived.integrity.algo, derived.integrity.entries
    );
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
