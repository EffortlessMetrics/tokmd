//! Derived analysis Markdown rendering.
//!
//! This module coordinates single-responsibility renderers for totals, ratios,
//! distributions, top-offender tables, density, context-window, legacy COCOMO
//! fallback, and integrity sections.

use super::effort;
use tokmd_analysis_types::{DerivedReport, EffortEstimateReport};

mod context;
mod density;
mod distribution;
mod integrity;
mod ratios;
mod structure;
mod top;
mod totals;

pub(super) fn render_derived_report(
    out: &mut String,
    derived: &DerivedReport,
    effort_report: Option<&EffortEstimateReport>,
) {
    totals::render_totals(out, derived);
    ratios::render_ratios(out, derived);
    distribution::render_distribution(out, derived);
    top::render_top_offenders(out, derived);
    structure::render_structure(out, derived);
    density::render_density_sections(out, derived);
    context::render_context_sections(out, derived);

    // Prefer the richer top-level effort contract when present; fall back to
    // legacy derived COCOMO output for older receipts.
    if let Some(effort_report) = effort_report {
        effort::render_effort_report(out, effort_report);
    } else if let Some(cocomo) = &derived.cocomo {
        effort::render_legacy_cocomo_report(out, derived, cocomo);
    }

    integrity::render_integrity(out, derived);
}
