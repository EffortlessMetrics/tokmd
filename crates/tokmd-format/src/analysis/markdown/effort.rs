//! Effort estimate Markdown rendering.
//!
//! This module owns the richer `receipt.effort` renderer and the legacy
//! `derived.cocomo` compatibility fallback used by analysis Markdown output.

use std::fmt::Write;

use super::{fmt_f64, fmt_pct};
use tokmd_analysis_types::{DerivedReport, EffortDriverDirection, EffortEstimateReport};

/// Render the richer `receipt.effort` section to Markdown.
///
/// This path is preferred over legacy `derived.cocomo` rendering because it can
/// expose the newer effort contract: authored-vs-total size basis, confidence,
/// explanatory drivers, assumptions, and optional base/head delta analysis.
///
/// The function is intentionally receipt-driven:
/// - it does not compute estimates,
/// - it does not backfill missing fields,
/// - it does not hide placeholder/scaffold values.
///
/// That keeps formatting honest when the upstream effort engine is only
/// partially implemented.
pub(super) fn render_effort_report(out: &mut String, effort: &EffortEstimateReport) {
    out.push_str("## Effort estimate\n\n");

    out.push_str("### Size basis\n\n");
    let _ = writeln!(
        out,
        "- Model: `{}`\n- Total LOC lines: `{}`\n- Authored LOC lines: `{}`\n- Generated LOC lines: `{}`\n- Vendored LOC lines: `{}`\n- Authoring KLOC: `{}`\n- Total KLOC: `{}`\n- Generated share: `{}`\n- Vendored share: `{}`\n- Classification confidence: `{}`\n",
        effort.model,
        effort.size_basis.total_lines,
        effort.size_basis.authored_lines,
        effort.size_basis.generated_lines,
        effort.size_basis.vendored_lines,
        fmt_f64(effort.size_basis.kloc_authored, 4),
        fmt_f64(effort.size_basis.kloc_total, 4),
        fmt_pct(effort.size_basis.generated_pct),
        fmt_pct(effort.size_basis.vendored_pct),
        effort.size_basis.classification_confidence
    );

    if !effort.size_basis.by_tag.is_empty() {
        out.push_str("### Size by tag\n\n");
        out.push_str("|Tag|Lines|Authored|Share|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for row in &effort.size_basis.by_tag {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.tag,
                row.lines,
                row.authored_lines,
                fmt_pct(row.pct_of_total)
            );
        }
        out.push('\n');
    }

    out.push_str("### Headline\n\n");
    let _ = writeln!(
        out,
        "- Effort p50: `{}` person-months (low `{}` / p80 `{}`)\n- Schedule p50: `{}` months (low `{}` / p80 `{}`)\n- Staff p50: `{}` FTE (low `{}` / p80 `{}`)\n",
        fmt_f64(effort.results.effort_pm_p50, 4),
        fmt_f64(effort.results.effort_pm_low, 4),
        fmt_f64(effort.results.effort_pm_p80, 4),
        fmt_f64(effort.results.schedule_months_p50, 4),
        fmt_f64(effort.results.schedule_months_low, 4),
        fmt_f64(effort.results.schedule_months_p80, 4),
        fmt_f64(effort.results.staff_p50, 4),
        fmt_f64(effort.results.staff_low, 4),
        fmt_f64(effort.results.staff_p80, 4),
    );

    out.push_str("### Why\n\n");
    let _ = writeln!(out, "- Confidence level: `{}`", effort.confidence.level);
    if let Some(coverage) = effort.confidence.data_coverage_pct {
        let _ = writeln!(out, "- Data coverage: `{}`", fmt_pct(coverage));
    }
    if !effort.confidence.reasons.is_empty() {
        out.push_str("- Reasons:\n");
        for reason in &effort.confidence.reasons {
            let _ = writeln!(out, "  - {reason}");
        }
    }
    out.push('\n');

    out.push_str("### Drivers\n\n");
    if effort.drivers.is_empty() {
        out.push_str("- No material drivers were inferred.\n\n");
    } else {
        out.push_str("|Driver|Direction|Weight|Evidence|\n");
        out.push_str("|---|---|---:|---|\n");
        for row in effort.drivers.iter().take(35) {
            let direction = match row.direction {
                EffortDriverDirection::Raises => "raises",
                EffortDriverDirection::Lowers => "lowers",
                EffortDriverDirection::Neutral => "neutral",
            };
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.label,
                direction,
                fmt_f64(row.weight, 4),
                row.evidence
            );
        }
        out.push('\n');
    }

    if !effort.assumptions.notes.is_empty() {
        out.push_str("### Assumptions\n\n");
        for note in &effort.assumptions.notes {
            let _ = writeln!(out, "- {note}");
        }
        out.push('\n');
    }

    if !effort.assumptions.overrides.is_empty() {
        out.push_str("### Assumption overrides\n\n");
        out.push_str("|Setting|Value|\n");
        out.push_str("|---|---|\n");
        for (key, value) in &effort.assumptions.overrides {
            let _ = writeln!(out, "|{key}|{value}|");
        }
        out.push('\n');
    }

    out.push_str("### Delta\n\n");
    if let Some(delta) = &effort.delta {
        let _ = writeln!(
            out,
            "- Reference window: `{}`..`{}`\n- Files changed: `{}`\n- Modules changed: `{}`\n- Languages changed: `{}`\n- Hotspots touched: `{}`\n- Coupled neighbors touched: `{}`\n- Blast radius: `{}`\n- Classification: `{}`\n- Effort p50 impact: `{}`\n- Effort p80 impact: `{}`\n",
            delta.base,
            delta.head,
            delta.files_changed,
            delta.modules_changed,
            delta.langs_changed,
            delta.hotspot_files_touched,
            delta.coupled_neighbors_touched,
            fmt_f64(delta.blast_radius, 4),
            delta.classification,
            fmt_f64(delta.effort_pm_est, 4),
            fmt_f64(delta.effort_pm_high, 4)
        );
        let _ = writeln!(
            out,
            "- Effort low bound (delta): `{}`\n",
            fmt_f64(delta.effort_pm_low, 4),
        );
    } else {
        out.push_str("- Baseline comparison is not available for this receipt.\n\n");
    }
}

/// Render the legacy `derived.cocomo` estimate block.
///
/// This is a compatibility fallback for receipts that do not yet populate the
/// top-level `effort` section. It renders classic derived COCOMO values using
/// `derived.totals` for line-count context and `derived.cocomo` for the model
/// coefficients and headline estimate.
///
/// Newer receipts should prefer [`render_effort_report`], which can represent
/// authored size basis, confidence, drivers, assumptions, and delta data.
pub(super) fn render_legacy_cocomo_report(
    out: &mut String,
    derived: &DerivedReport,
    cocomo: &tokmd_analysis_types::CocomoReport,
) {
    out.push_str("## Effort estimate\n\n");

    out.push_str("### Size basis\n\n");
    let _ = writeln!(
        out,
        "- Source lines: `{}`\n- Total lines: `{}`\n- KLOC: `{}`\n",
        derived.totals.code,
        derived.totals.lines,
        fmt_f64(cocomo.kloc, 4)
    );

    out.push_str("### Headline\n\n");
    let _ = writeln!(
        out,
        "- Effort: `{}` person-months\n- Duration: `{}` months\n- Staff: `{}`\n",
        fmt_f64(cocomo.effort_pm, 2),
        fmt_f64(cocomo.duration_months, 2),
        fmt_f64(cocomo.staff, 2)
    );

    out.push_str("### Why\n\n");
    let _ = writeln!(
        out,
        "- Model: `COCOMO` (`{}` mode)\n- Formula: `E = a * KLOC^b`\n- Coefficients: `a={}`, `b={}`, `c={}`, `d={}`\n",
        cocomo.mode,
        fmt_f64(cocomo.a, 2),
        fmt_f64(cocomo.b, 2),
        fmt_f64(cocomo.c, 2),
        fmt_f64(cocomo.d, 2)
    );

    out.push_str("### Delta\n\n");
    out.push_str("- Baseline comparison is not available for this receipt.\n\n");
}
