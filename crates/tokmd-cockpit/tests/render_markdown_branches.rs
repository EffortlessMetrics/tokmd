//! Branch-coverage tests for `tokmd_cockpit::render::render_markdown`.
//!
//! `render_markdown` dispatches to several sub-renderers
//! (`evidence_gates`, `risk`, `summary` comparison, …). Existing
//! integration tests build receipts with `evidence.diff_coverage`,
//! `evidence.contracts`, `evidence.supply_chain`, `evidence.determinism`
//! and `evidence.complexity` all set to `None`, so the `Some(_)` arms
//! never run. This file pins each `if let Some(_) = …` branch in
//! `evidence_gates::render` plus the `bus_factor_warnings` branch in
//! `risk::render`.
//!
//! The sub-renderers are private (`pub(super)`); they are exercised
//! through the public `render_markdown` entrypoint.

use tokmd_cockpit::render::render_markdown;
use tokmd_cockpit::*;
use tokmd_types::cockpit::COCKPIT_SCHEMA_VERSION;

fn base_meta() -> GateMeta {
    GateMeta {
        status: GateStatus::Pass,
        source: EvidenceSource::RanLocal,
        commit_match: CommitMatch::Exact,
        scope: ScopeCoverage {
            relevant: vec![],
            tested: vec![],
            ratio: 1.0,
            lines_relevant: None,
            lines_tested: None,
        },
        evidence_commit: None,
        evidence_generated_at_ms: None,
    }
}

fn base_mutation() -> MutationGate {
    MutationGate {
        meta: GateMeta {
            status: GateStatus::Skipped,
            ..base_meta()
        },
        survivors: vec![],
        killed: 0,
        timeout: 0,
        unviable: 0,
    }
}

fn base_receipt() -> CockpitReceipt {
    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 0,
        base_ref: "main".to_string(),
        head_ref: "HEAD".to_string(),
        change_surface: ChangeSurface {
            commits: 1,
            files_changed: 1,
            insertions: 10,
            deletions: 5,
            net_lines: 5,
            churn_velocity: 0.0,
            change_concentration: 0.0,
        },
        composition: Composition {
            code_pct: 1.0,
            test_pct: 0.0,
            docs_pct: 0.0,
            config_pct: 0.0,
            test_ratio: 0.0,
        },
        code_health: CodeHealth {
            score: 95,
            grade: "A".to_string(),
            large_files_touched: 0,
            avg_file_size: 100,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Low,
            score: 10,
        },
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
        evidence: Evidence {
            overall_status: GateStatus::Pass,
            mutation: base_mutation(),
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        },
        review_plan: vec![],
        trend: None,
    }
}

// ---------------------------------------------------------------------------
// Baseline structure
// ---------------------------------------------------------------------------

#[test]
fn markdown_emits_glass_cockpit_header_and_evidence_gates_section() {
    let md = render_markdown(&base_receipt());
    assert!(md.starts_with("## Glass Cockpit\n"));
    assert!(md.contains("### Evidence Gates"));
    assert!(md.contains("- **Overall status**:"));
    assert!(md.contains("- **Mutation**:"));
}

#[test]
fn markdown_omits_optional_gate_lines_when_none() {
    let md = render_markdown(&base_receipt());
    assert!(!md.contains("- **Diff coverage**"));
    assert!(!md.contains("- **Contracts**:"));
    assert!(!md.contains("- **Supply chain**"));
    assert!(!md.contains("- **Determinism**"));
    assert!(!md.contains("- **Complexity**:"));
}

// ---------------------------------------------------------------------------
// evidence_gates::render — each `if let Some(_) = …` arm
// ---------------------------------------------------------------------------

#[test]
fn markdown_emits_diff_coverage_gate_line_when_present() {
    let mut r = base_receipt();
    r.evidence.diff_coverage = Some(DiffCoverageGate {
        meta: base_meta(),
        lines_added: 100,
        lines_covered: 84,
        coverage_pct: 0.842,
        uncovered_hunks: vec![],
    });

    let md = render_markdown(&r);

    assert!(
        md.contains("- **Diff coverage**: Pass (84.2%)"),
        "unexpected diff coverage line in: {md}"
    );
}

#[test]
fn markdown_emits_contracts_gate_line_when_present_with_failure_count() {
    let mut r = base_receipt();
    r.evidence.contracts = Some(ContractDiffGate {
        meta: GateMeta {
            status: GateStatus::Fail,
            ..base_meta()
        },
        semver: None,
        cli: None,
        schema: None,
        failures: 3,
    });

    let md = render_markdown(&r);

    assert!(
        md.contains("- **Contracts**: Fail (failures: 3)"),
        "unexpected contracts line in: {md}"
    );
}

#[test]
fn markdown_emits_supply_chain_gate_line_when_present_with_vuln_count() {
    let mut r = base_receipt();
    r.evidence.supply_chain = Some(SupplyChainGate {
        meta: GateMeta {
            status: GateStatus::Warn,
            ..base_meta()
        },
        vulnerabilities: vec![
            Vulnerability {
                id: "RUSTSEC-2024-0001".to_string(),
                package: "x".to_string(),
                severity: "low".to_string(),
                title: "advisory".to_string(),
            },
            Vulnerability {
                id: "RUSTSEC-2024-0002".to_string(),
                package: "y".to_string(),
                severity: "high".to_string(),
                title: "advisory".to_string(),
            },
        ],
        denied: vec![],
        advisory_db_version: None,
    });

    let md = render_markdown(&r);

    assert!(
        md.contains("- **Supply chain**: Warn (vulnerabilities: 2)"),
        "unexpected supply chain line in: {md}"
    );
}

#[test]
fn markdown_emits_determinism_gate_line_when_present_with_diff_count() {
    let mut r = base_receipt();
    r.evidence.determinism = Some(DeterminismGate {
        meta: base_meta(),
        expected_hash: None,
        actual_hash: None,
        algo: "blake3".to_string(),
        differences: vec!["a.json".to_string(), "b.json".to_string()],
    });

    let md = render_markdown(&r);

    assert!(
        md.contains("- **Determinism**: Pass (differences: 2)"),
        "unexpected determinism line in: {md}"
    );
}

#[test]
fn markdown_emits_complexity_gate_line_with_avg_and_max() {
    let mut r = base_receipt();
    r.evidence.complexity = Some(ComplexityGate {
        meta: GateMeta {
            status: GateStatus::Warn,
            ..base_meta()
        },
        files_analyzed: 7,
        high_complexity_files: vec![],
        avg_cyclomatic: 8.55,
        max_cyclomatic: 21,
        threshold_exceeded: true,
    });

    let md = render_markdown(&r);

    assert!(
        md.contains("- **Complexity**: Warn (avg cyclomatic: 8.6, max: 21)"),
        "unexpected complexity line in: {md}"
    );
}

#[test]
fn markdown_emits_all_optional_gate_lines_when_all_present() {
    let mut r = base_receipt();
    r.evidence.diff_coverage = Some(DiffCoverageGate {
        meta: base_meta(),
        lines_added: 1,
        lines_covered: 1,
        coverage_pct: 1.0,
        uncovered_hunks: vec![],
    });
    r.evidence.contracts = Some(ContractDiffGate {
        meta: base_meta(),
        semver: None,
        cli: None,
        schema: None,
        failures: 0,
    });
    r.evidence.supply_chain = Some(SupplyChainGate {
        meta: base_meta(),
        vulnerabilities: vec![],
        denied: vec![],
        advisory_db_version: None,
    });
    r.evidence.determinism = Some(DeterminismGate {
        meta: base_meta(),
        expected_hash: None,
        actual_hash: None,
        algo: "blake3".to_string(),
        differences: vec![],
    });
    r.evidence.complexity = Some(ComplexityGate {
        meta: base_meta(),
        files_analyzed: 1,
        high_complexity_files: vec![],
        avg_cyclomatic: 1.0,
        max_cyclomatic: 1,
        threshold_exceeded: false,
    });

    let md = render_markdown(&r);

    for needle in [
        "- **Diff coverage**",
        "- **Contracts**:",
        "- **Supply chain**",
        "- **Determinism**",
        "- **Complexity**:",
    ] {
        assert!(md.contains(needle), "missing {needle:?} in: {md}");
    }
}

// ---------------------------------------------------------------------------
// risk::render — bus_factor_warnings block
// ---------------------------------------------------------------------------

#[test]
fn markdown_omits_bus_factor_warnings_when_empty() {
    let md = render_markdown(&base_receipt());
    assert!(!md.contains("Bus factor warnings"));
}

#[test]
fn markdown_emits_bus_factor_warnings_with_bullet_per_entry() {
    let mut r = base_receipt();
    r.risk.bus_factor_warnings = vec!["alice owns 80% of src/x.rs".to_string()];

    let md = render_markdown(&r);

    assert!(
        md.contains("- **Bus factor warnings**:"),
        "missing header in: {md}"
    );
    assert!(
        md.contains("  - alice owns 80% of src/x.rs"),
        "missing bullet in: {md}"
    );
}

#[test]
fn markdown_emits_multiple_bus_factor_warnings() {
    let mut r = base_receipt();
    r.risk.bus_factor_warnings = vec![
        "bob owns 90% of a.rs".to_string(),
        "carol owns 85% of b.rs".to_string(),
    ];

    let md = render_markdown(&r);

    assert!(md.contains("  - bob owns 90% of a.rs"));
    assert!(md.contains("  - carol owns 85% of b.rs"));
}

#[test]
fn markdown_emits_hotspots_touched_alongside_bus_factor_warnings() {
    let mut r = base_receipt();
    r.risk.hotspots_touched = vec!["src/hot.rs".to_string()];
    r.risk.bus_factor_warnings = vec!["dave owns hot.rs".to_string()];

    let md = render_markdown(&r);

    assert!(md.contains("- **Hotspots touched**:"));
    assert!(md.contains("  - src/hot.rs"));
    assert!(md.contains("- **Bus factor warnings**:"));
    assert!(md.contains("  - dave owns hot.rs"));
}

// ---------------------------------------------------------------------------
// summary::render_comparison — trend baseline path branch
// ---------------------------------------------------------------------------

#[test]
fn markdown_omits_summary_comparison_when_no_trend() {
    let md = render_markdown(&base_receipt());
    assert!(!md.contains("### Summary Comparison"));
}

#[test]
fn markdown_omits_summary_comparison_when_baseline_unavailable() {
    let mut r = base_receipt();
    r.trend = Some(TrendComparison {
        baseline_available: false,
        baseline_path: None,
        baseline_generated_at_ms: None,
        health: None,
        risk: None,
        complexity: None,
    });

    let md = render_markdown(&r);

    assert!(!md.contains("### Summary Comparison"));
}

#[test]
fn markdown_emits_summary_comparison_health_and_risk_rows_when_trend_available() {
    let mut r = base_receipt();
    r.trend = Some(TrendComparison {
        baseline_available: true,
        baseline_path: Some(".tokmd/baseline.json".to_string()),
        baseline_generated_at_ms: Some(1000),
        health: Some(TrendMetric {
            current: 90.0,
            previous: 80.0,
            delta: 10.0,
            delta_pct: 12.5,
            direction: TrendDirection::Improving,
        }),
        risk: Some(TrendMetric {
            current: 25.0,
            previous: 30.0,
            delta: -5.0,
            delta_pct: -16.7,
            direction: TrendDirection::Improving,
        }),
        complexity: Some(TrendIndicator {
            direction: TrendDirection::Stable,
            summary: "stable".to_string(),
            files_increased: 0,
            files_decreased: 0,
            avg_cyclomatic_delta: Some(0.1),
            avg_cognitive_delta: None,
        }),
    });

    let md = render_markdown(&r);

    assert!(md.contains("### Summary Comparison"));
    // Health row with previous/current/delta/direction
    assert!(md.contains("|Health Score|80.0|90.0|+10.00|improving|"));
    assert!(md.contains("|Risk Score|30.0|25.0|-5.00|improving|"));
    // Complexity row: delta is signed, direction label
    assert!(md.contains("|Avg Cyclomatic|n/a|n/a|+0.10|stable|"));
    // Baseline path line
    assert!(md.contains("Baseline: `.tokmd/baseline.json`"));
}

#[test]
fn markdown_summary_comparison_avg_cyclomatic_falls_back_when_delta_missing() {
    let mut r = base_receipt();
    r.trend = Some(TrendComparison {
        baseline_available: true,
        baseline_path: None,
        baseline_generated_at_ms: Some(1000),
        health: None,
        risk: None,
        complexity: Some(TrendIndicator {
            direction: TrendDirection::Stable,
            summary: "stable".to_string(),
            files_increased: 0,
            files_decreased: 0,
            avg_cyclomatic_delta: None,
            avg_cognitive_delta: None,
        }),
    });

    let md = render_markdown(&r);

    assert!(md.contains("### Summary Comparison"));
    assert!(md.contains("|Avg Cyclomatic|n/a|n/a|n/a|stable|"));
}
