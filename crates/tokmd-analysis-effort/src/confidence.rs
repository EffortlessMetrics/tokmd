use tokmd_analysis_types::{
    ComplexityReport, DerivedReport, DuplicateReport, EffortConfidence, EffortConfidenceLevel,
    EffortDriver, EffortDriverDirection, EffortSizeBasis, GitReport, ApiSurfaceReport,
};

pub fn build_confidence(
    basis: &EffortSizeBasis,
    derived: &DerivedReport,
    git: Option<&GitReport>,
    complexity: Option<&ComplexityReport>,
    api_surface: Option<&ApiSurfaceReport>,
    dup: Option<&DuplicateReport>,
    delta_present: bool,
) -> (EffortConfidence, f64) {
    let mut reasons: Vec<String> = Vec::new();
    let mut score: f64 = 0.25;

    if !basis.warnings.is_empty() {
        reasons.push("classification heuristics touched unknown files".to_string());
    } else {
        score += 0.10;
    }

    if basis.classification_confidence == EffortConfidenceLevel::High {
        score += 0.25;
    } else if basis.classification_confidence == EffortConfidenceLevel::Medium {
        score += 0.15;
    } else {
        reasons.push("classification used fallback classification".to_string());
    }

    if basis.generated_pct > 0.50 {
        reasons.push("majority of scan appears generated".to_string());
    }
    if basis.vendored_pct > 0.50 {
        reasons.push("majority of scan appears vendored".to_string());
    }

    if let Some(git) = git {
        score += 0.12;
        if git.freshness.stale_pct > 0.0 {
            score += 0.02;
        }
        if !git.hotspots.is_empty() {
            score += 0.03;
        }
    } else {
        reasons.push("git data missing".to_string());
    }

    if let Some(complexity) = complexity {
        score += 0.10;
        if complexity.avg_cyclomatic > 0.0 {
            score += 0.04;
        }
        if complexity.max_cyclomatic > 30 {
            score += 0.02;
        }
    } else {
        reasons.push("complexity report missing".to_string());
    }

    if api_surface.is_some() {
        score += 0.08;
    } else {
        reasons.push("api surface data missing".to_string());
    }

    if dup.is_some() {
        score += 0.05;
    } else {
        reasons.push("duplication report missing".to_string());
    }

    if derived.test_density.test_lines + derived.test_density.prod_lines > 0 {
        score += 0.10;
    } else {
        reasons.push("test density unavailable".to_string());
    }

    if delta_present {
        score += 0.05;
    }

    if derived.polyglot.lang_count == 0 {
        reasons.push("polyglot signal missing".to_string());
    }

    let score = (score - (basis.generated_pct + basis.vendored_pct) * 0.20).clamp(0.0, 1.0);
    let level = if score >= 0.72 {
        EffortConfidenceLevel::High
    } else if score >= 0.45 {
        EffortConfidenceLevel::Medium
    } else {
        EffortConfidenceLevel::Low
    };

    if level == EffortConfidenceLevel::High {
        reasons.clear();
    }

    (
        EffortConfidence {
            level,
            reasons,
            data_coverage_pct: Some(score),
        },
        score,
    )
}
