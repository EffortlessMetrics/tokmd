//! Technical debt ratio helpers for complexity reports.

use tokmd_analysis_types::{FileComplexity, TechnicalDebtLevel, TechnicalDebtRatio};
use tokmd_types::{ExportData, FileKind};

use super::math::round_f64;

const TECHNICAL_DEBT_LOW_THRESHOLD: f64 = 30.0;
const TECHNICAL_DEBT_MODERATE_THRESHOLD: f64 = 60.0;
const TECHNICAL_DEBT_HIGH_THRESHOLD: f64 = 100.0;

pub(super) fn average_parent_loc(export: &ExportData) -> Option<f64> {
    let total_code: usize = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .map(|r| r.code)
        .sum();
    let parent_count: usize = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .count();

    if parent_count == 0 {
        return None;
    }

    let avg_loc = total_code as f64 / parent_count as f64;
    if avg_loc <= 0.0 {
        return None;
    }
    Some(avg_loc)
}

/// Compute a complexity-to-size heuristic debt ratio.
///
/// Ratio = (sum cyclomatic + cognitive complexity points) / KLOC
pub(super) fn compute_technical_debt_ratio(
    export: &ExportData,
    file_complexities: &[FileComplexity],
) -> Option<TechnicalDebtRatio> {
    if file_complexities.is_empty() {
        return None;
    }

    let total_code: usize = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .map(|r| r.code)
        .sum();
    if total_code == 0 {
        return None;
    }

    let complexity_points: usize = file_complexities
        .iter()
        .map(|f| f.cyclomatic_complexity + f.cognitive_complexity.unwrap_or(0))
        .sum();

    let code_kloc = total_code as f64 / 1000.0;
    let ratio = round_f64(complexity_points as f64 / code_kloc, 2);
    let level = if ratio < TECHNICAL_DEBT_LOW_THRESHOLD {
        TechnicalDebtLevel::Low
    } else if ratio < TECHNICAL_DEBT_MODERATE_THRESHOLD {
        TechnicalDebtLevel::Moderate
    } else if ratio < TECHNICAL_DEBT_HIGH_THRESHOLD {
        TechnicalDebtLevel::High
    } else {
        TechnicalDebtLevel::Critical
    };

    Some(TechnicalDebtRatio {
        ratio,
        complexity_points,
        code_kloc: round_f64(code_kloc, 4),
        level,
    })
}
