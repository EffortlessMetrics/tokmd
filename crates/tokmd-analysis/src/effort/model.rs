use std::path::Path;

use anyhow::Result;
use tokmd_analysis_types::{ApiSurfaceReport, ComplexityReport, DuplicateReport, GitReport};
use tokmd_analysis_types::{
    DerivedReport, EffortAssumptions, EffortDriver, EffortEstimateReport, EffortModel,
    EffortResults, EffortSizeBasis,
};
use tokmd_types::ExportData;

use super::cocomo2::cocomo2_baseline;
use super::cocomo81::cocomo81_baseline;
use super::confidence::build_confidence;
use super::delta::build_delta;
use super::drivers::build_drivers;
use super::monte_carlo::apply_monte_carlo;
use super::request::{EffortModelKind, EffortRequest};
use super::size_basis::SizeBasisResult;
use super::size_basis::build_size_basis;

fn has_host_root(root: &Path) -> bool {
    !root.as_os_str().is_empty()
}
use super::uncertainty::apply_uncertainty;

/// Build an effort estimate from exported code inventory plus optional enrichers.
///
/// The builder is intentionally staged:
///
/// 1. classify authored/generated/vendored surface,
/// 2. compute a deterministic baseline from authored KLOC,
/// 3. extract explanatory drivers from available signals,
/// 4. derive confidence from signal coverage and classification quality,
/// 5. attach delta/blast-radius output when base/head refs are provided.
///
/// The function is conservative:
///
/// - it does not require git/complexity/docs/dup inputs to be present,
/// - it degrades confidence rather than failing when signals are missing,
/// - it keeps the estimate deterministic unless the request explicitly opts
///   into probabilistic follow-up behavior.
///
/// Callers should prefer passing the richest available context instead of
/// backfilling values later in the formatting layer.
#[allow(clippy::too_many_arguments)]
pub fn build_effort_report(
    root: &Path,
    export: &ExportData,
    derived: &DerivedReport,
    git: Option<&GitReport>,
    complexity: Option<&ComplexityReport>,
    api_surface: Option<&ApiSurfaceReport>,
    dup: Option<&DuplicateReport>,
    req: &EffortRequest,
) -> Result<EffortEstimateReport> {
    let SizeBasisResult {
        basis: size_basis,
        source_confidence: basis_confidence,
    } = build_size_basis(root, export);

    let drivers: Vec<EffortDriver> =
        build_drivers(&size_basis, derived, git, complexity, api_surface, dup);

    let delta = match (&req.base_ref, &req.head_ref) {
        (Some(base), Some(head)) if has_host_root(root) => {
            build_delta(root, export, git, base.as_str(), head.as_str()).ok()
        }
        _ => None,
    };

    let (mut effort_model_result, model_name) = match req.model {
        EffortModelKind::Cocomo81Basic => (
            cocomo81_baseline(size_basis.kloc_authored),
            EffortModel::Cocomo81Basic,
        ),
        EffortModelKind::Cocomo2Early => (
            cocomo2_baseline(size_basis.kloc_authored),
            EffortModel::Cocomo2Early,
        ),
        EffortModelKind::Ensemble => {
            let c1 = cocomo81_baseline(size_basis.kloc_authored);
            let c2 = cocomo2_baseline(size_basis.kloc_authored);
            (avg_models(&c1, &c2), EffortModel::Ensemble)
        }
    };

    let has_delta = delta.is_some();
    let (confidence, confidence_score) = build_confidence(
        &size_basis,
        derived,
        git,
        complexity,
        api_surface,
        dup,
        has_delta,
    );

    if req.monte_carlo && req.mc_iterations > 0 {
        effort_model_result = apply_monte_carlo(
            &effort_model_result,
            &drivers,
            &confidence,
            basis_confidence,
            req.mc_iterations,
            req.mc_seed,
        );
    } else {
        effort_model_result =
            apply_uncertainty(effort_model_result, &confidence, basis_confidence, &drivers);
    }

    let results = effort_model_result;

    let mut assumptions = assumptions_summary(&size_basis, confidence_score, req);

    if req.base_ref.is_some() || req.head_ref.is_some() {
        assumptions
            .notes
            .push("Delta path requested via base/head references".to_string());
    }
    if req.monte_carlo {
        assumptions.notes.push(format!(
            "Monte Carlo requested (iterations: {}, seed: {:?})",
            req.mc_iterations, req.mc_seed
        ));
    }

    Ok(EffortEstimateReport {
        model: model_name,
        size_basis,
        results,
        confidence,
        drivers,
        assumptions,
        delta,
    })
}

fn assumptions_summary(
    size_basis: &EffortSizeBasis,
    confidence: f64,
    req: &EffortRequest,
) -> EffortAssumptions {
    let mut notes: Vec<String> = Vec::new();

    notes.push(format!("Effort layer requested: {}", req.layer.as_str()));

    if req.base_ref.is_some() {
        notes.push("Base/head inputs requested for delta context".to_string());
    }

    if req.monte_carlo {
        notes.push(format!(
            "Monte Carlo enabled: {} iterations{}",
            req.mc_iterations,
            req.mc_seed
                .map(|seed| format!(", seed {}", seed))
                .unwrap_or_else(|| ", deterministic seed".to_string())
        ));
    }

    if size_basis.generated_lines > 0 {
        notes.push(format!(
            "{}% of lines treated as generated or vendored",
            ((size_basis.generated_pct + size_basis.vendored_pct) * 100.0).round()
        ));
    }

    let mut overrides = std::collections::BTreeMap::new();
    if req.monte_carlo {
        overrides.insert(
            "monte_carlo".to_string(),
            format!("iterations={} seed={:?}", req.mc_iterations, req.mc_seed),
        );
    }

    if confidence < 0.6 {
        notes.push("Confidence lowered due to missing optional inputs".to_string());
    }

    EffortAssumptions { notes, overrides }
}

fn avg_models(a: &EffortResults, b: &EffortResults) -> EffortResults {
    let mut out = a.clone();
    out.effort_pm_p50 = (a.effort_pm_p50 + b.effort_pm_p50) * 0.5;
    out.schedule_months_p50 = (a.schedule_months_p50 + b.schedule_months_p50) * 0.5;
    out.staff_p50 = (a.staff_p50 + b.staff_p50) * 0.5;
    out.effort_pm_low = (a.effort_pm_low + b.effort_pm_low) * 0.5;
    out.effort_pm_p80 = (a.effort_pm_p80 + b.effort_pm_p80) * 0.5;
    out.schedule_months_low = (a.schedule_months_low + b.schedule_months_low) * 0.5;
    out.schedule_months_p80 = (a.schedule_months_p80 + b.schedule_months_p80) * 0.5;
    out.staff_low = (a.staff_low + b.staff_low) * 0.5;
    out.staff_p80 = (a.staff_p80 + b.staff_p80) * 0.5;
    out
}

#[allow(dead_code)]
struct SizeContext {
    size_basis: EffortSizeBasis,
    basis_confidence: f64,
}
