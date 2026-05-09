//! API surface Markdown rendering.
//!
//! This module owns the top-level API metrics table and its language, module,
//! and exporter detail tables for analysis Markdown output.

use std::fmt::Write;

use super::fmt_pct;
use tokmd_analysis_types::ApiSurfaceReport;

pub(super) fn render_api_surface_report(out: &mut String, api: &ApiSurfaceReport) {
    out.push_str("## API surface\n\n");
    out.push_str("|Metric|Value|\n");
    out.push_str("|---|---:|\n");
    let _ = writeln!(out, "|Total items|{}|", api.total_items);
    let _ = writeln!(out, "|Public items|{}|", api.public_items);
    let _ = writeln!(out, "|Internal items|{}|", api.internal_items);
    let _ = writeln!(out, "|Public ratio|{}|", fmt_pct(api.public_ratio));
    let _ = writeln!(
        out,
        "|Documented ratio|{}|\n",
        fmt_pct(api.documented_ratio)
    );

    if !api.by_language.is_empty() {
        out.push_str("### By language\n\n");
        out.push_str("|Language|Total|Public|Internal|Public%|\n");
        out.push_str("|---|---:|---:|---:|---:|\n");
        for (lang, data) in &api.by_language {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|{}|",
                lang,
                data.total_items,
                data.public_items,
                data.internal_items,
                fmt_pct(data.public_ratio)
            );
        }
        out.push('\n');
    }

    if !api.by_module.is_empty() {
        out.push_str("### By module\n\n");
        out.push_str("|Module|Total|Public|Public%|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for row in api.by_module.iter().take(20) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.module,
                row.total_items,
                row.public_items,
                fmt_pct(row.public_ratio)
            );
        }
        out.push('\n');
    }

    if !api.top_exporters.is_empty() {
        out.push_str("### Top exporters\n\n");
        out.push_str("|Path|Language|Public|Total|\n");
        out.push_str("|---|---|---:|---:|\n");
        for item in api.top_exporters.iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                item.path, item.lang, item.public_items, item.total_items
            );
        }
        out.push('\n');
    }
}
