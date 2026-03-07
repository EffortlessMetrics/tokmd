use std::cmp::Ordering;

use tokmd_analysis_types::{
    ApiSurfaceReport, ComplexityReport, DerivedReport, DuplicateReport, EffortDriver,
    EffortDriverDirection, EffortSizeBasis, GitReport,
};

pub fn build_drivers(
    basis: &EffortSizeBasis,
    derived: &DerivedReport,
    git: Option<&GitReport>,
    complexity: Option<&ComplexityReport>,
    api_surface: Option<&ApiSurfaceReport>,
    dup: Option<&DuplicateReport>,
) -> Vec<EffortDriver> {
    let mut drivers: Vec<EffortDriver> = Vec::new();

    if basis.generated_pct > 0.30 {
        drivers.push(EffortDriver {
            key: "generated_files".to_string(),
            label: "Large generated surface increases cleanup overhead".to_string(),
            weight: (basis.generated_pct * 0.8).clamp(0.05, 0.75),
            direction: EffortDriverDirection::Raises,
            evidence: format!("{}% generated", round_pct(basis.generated_pct)),
        });
    }

    if basis.vendored_pct > 0.25 {
        drivers.push(EffortDriver {
            key: "vendored_files".to_string(),
            label: "Large vendored surface increases integration friction".to_string(),
            weight: (basis.vendored_pct * 0.9).clamp(0.05, 0.65),
            direction: EffortDriverDirection::Raises,
            evidence: format!("{}% vendored", round_pct(basis.vendored_pct)),
        });
    }

    if let Some(git) = git {
        if git.freshness.stale_pct > 0.30 {
            drivers.push(EffortDriver {
                key: "freshness_staleness".to_string(),
                label: "Stale files may increase rework probability".to_string(),
                weight: (git.freshness.stale_pct * 0.95).clamp(0.06, 0.95),
                direction: EffortDriverDirection::Raises,
                evidence: format!("{}% files stale", round_pct(git.freshness.stale_pct)),
            });
        }

        let coupling_links = git.coupling.len();
        if coupling_links > 0 {
            drivers.push(EffortDriver {
                key: "module_coupling".to_string(),
                label: "High coupling increases integration effort".to_string(),
                weight: (coupling_links as f64 / 120.0).clamp(0.1, 0.7),
                direction: EffortDriverDirection::Raises,
                evidence: format!("{} coupling edges", coupling_links),
            });
        }
    }

    if let Some(complexity) = complexity {
        if complexity.max_cyclomatic > 30 {
            drivers.push(EffortDriver {
                key: "complexity_hotspots".to_string(),
                label: "High cyclomatic complexity raises effort".to_string(),
                weight: ((complexity.max_cyclomatic as f64 - 20.0) / 200.0).clamp(0.15, 0.75),
                direction: EffortDriverDirection::Raises,
                evidence: format!("max cyclomatic {}", complexity.max_cyclomatic),
            });
        }
        if complexity.avg_cyclomatic > 4.0 {
            drivers.push(EffortDriver {
                key: "complexity_breadth".to_string(),
                label: "Average cyclomatic complexity suggests added control flow overhead".to_string(),
                weight: ((complexity.avg_cyclomatic - 1.0) / 30.0).clamp(0.08, 0.45),
                direction: EffortDriverDirection::Raises,
                evidence: format!("avg cyclomatic {:.2}", complexity.avg_cyclomatic),
            });
        }
    }

    if let Some(api) = api_surface {
        if api.public_ratio < 0.20 {
            drivers.push(EffortDriver {
                key: "api_documentation".to_string(),
                label: "Low public/API documentation ratio may slow reuse".to_string(),
                weight: 0.35,
                direction: EffortDriverDirection::Raises,
                evidence: format!("public ratio {:.2}", api.public_ratio),
            });
        }
        if api.documented_ratio < 0.50 {
            drivers.push(EffortDriver {
                key: "api_documented_ratio".to_string(),
                label: "API documentation coverage is low".to_string(),
                weight: 0.20,
                direction: EffortDriverDirection::Raises,
                evidence: format!("documented ratio {:.2}", api.documented_ratio),
            });
        }
    }

    if let Some(dup) = dup {
        if dup.wasted_bytes > 0 {
            drivers.push(EffortDriver {
                key: "duplication".to_string(),
                label: "Duplicate blocks add review and refactor overhead".to_string(),
                weight: (dup.wasted_bytes as f64 / 12_000.0).clamp(0.1, 0.7),
                direction: EffortDriverDirection::Raises,
                evidence: format!("{} bytes wasted by duplication", dup.wasted_bytes),
            });
        }
    }

    if derived.test_density.ratio < 0.10 {
        drivers.push(EffortDriver {
            key: "test_density".to_string(),
            label: "Limited test coverage increases implied verification effort".to_string(),
            weight: 0.55,
            direction: EffortDriverDirection::Raises,
            evidence: format!("test ratio {:.2}", derived.test_density.ratio),
        });
    }

    if derived.polyglot.lang_count > 8 {
        drivers.push(EffortDriver {
            key: "polyglot_spread".to_string(),
            label: "Wide language spread increases onboarding effort".to_string(),
            weight: (derived.polyglot.lang_count as f64 / 20.0).clamp(0.1, 0.45),
            direction: EffortDriverDirection::Raises,
            evidence: format!("{} languages", derived.polyglot.lang_count),
        });
    }

    if !basis.warnings.is_empty() {
        drivers.push(EffortDriver {
            key: "classification_fallback".to_string(),
            label: "Heuristic file classification used for some files".to_string(),
            weight: 0.08,
            direction: EffortDriverDirection::Raises,
            evidence: format!("{} classification warnings", basis.warnings.len()),
        });
    }

    drivers.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.key.cmp(&b.key))
    });
    drivers
}

fn round_pct(value: f64) -> String {
    format!("{:.1}", value * 100.0)
}
