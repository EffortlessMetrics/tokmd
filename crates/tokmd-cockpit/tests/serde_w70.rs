//! W70: Comprehensive serde roundtrip tests for tokmd-cockpit receipt types.
//!
//! Validates JSON roundtrip for CockpitReceipt and all nested types,
//! schema_version presence, enum variants, and deterministic output.

use proptest::prelude::*;
use serde_json;
use tokmd_types::cockpit::*;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn sample_gate_meta(status: GateStatus) -> GateMeta {
    GateMeta {
        status,
        source: EvidenceSource::RanLocal,
        commit_match: CommitMatch::Exact,
        scope: ScopeCoverage {
            relevant: vec!["src/lib.rs".to_string()],
            tested: vec!["src/lib.rs".to_string()],
            ratio: 1.0,
            lines_relevant: None,
            lines_tested: None,
        },
        evidence_commit: None,
        evidence_generated_at_ms: None,
    }
}

fn sample_evidence() -> Evidence {
    Evidence {
        overall_status: GateStatus::Pass,
        mutation: MutationGate {
            meta: sample_gate_meta(GateStatus::Pass),
            survivors: vec![],
            killed: 10,
            timeout: 1,
            unviable: 0,
        },
        diff_coverage: None,
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: None,
    }
}

fn sample_cockpit_receipt() -> CockpitReceipt {
    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 1700000000000,
        base_ref: "main".to_string(),
        head_ref: "feature/test".to_string(),
        change_surface: ChangeSurface {
            commits: 5,
            files_changed: 12,
            insertions: 200,
            deletions: 50,
            net_lines: 150,
            churn_velocity: 50.0,
            change_concentration: 0.65,
        },
        composition: Composition {
            code_pct: 70.0,
            test_pct: 20.0,
            docs_pct: 5.0,
            config_pct: 5.0,
            test_ratio: 0.29,
        },
        code_health: CodeHealth {
            score: 85,
            grade: "B".to_string(),
            large_files_touched: 1,
            avg_file_size: 120,
            complexity_indicator: ComplexityIndicator::Medium,
            warnings: vec![HealthWarning {
                path: "src/big.rs".to_string(),
                warning_type: WarningType::LargeFile,
                message: "File exceeds 500 lines".to_string(),
            }],
        },
        risk: Risk {
            hotspots_touched: vec!["src/core.rs".to_string()],
            bus_factor_warnings: vec![],
            level: RiskLevel::Medium,
            score: 35,
        },
        contracts: Contracts {
            api_changed: true,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 1,
        },
        evidence: sample_evidence(),
        review_plan: vec![ReviewItem {
            path: "src/core.rs".to_string(),
            reason: "hotspot + high churn".to_string(),
            priority: 1,
            complexity: Some(3),
            lines_changed: Some(80),
        }],
        trend: None,
    }
}

// ─── 1. Full CockpitReceipt roundtrip ──────────────────────────────────────

#[test]
fn cockpit_receipt_full_roundtrip() {
    let receipt = sample_cockpit_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(back.mode, "cockpit");
    assert_eq!(back.change_surface.commits, 5);
    assert_eq!(back.composition.code_pct, 70.0);
    assert_eq!(back.code_health.score, 85);
    assert_eq!(back.risk.level, RiskLevel::Medium);
    assert_eq!(back.review_plan.len(), 1);
}

// ─── 2. Schema version present in JSON output ──────────────────────────────

#[test]
fn schema_version_in_json_output() {
    let receipt = sample_cockpit_receipt();
    let val: serde_json::Value = serde_json::to_value(&receipt).unwrap();
    assert_eq!(val["schema_version"], COCKPIT_SCHEMA_VERSION);
    assert_eq!(val["mode"], "cockpit");
}

// ─── 3. GateStatus all variants roundtrip ───────────────────────────────────

#[test]
fn gate_status_all_variants_roundtrip() {
    for variant in [
        GateStatus::Pass,
        GateStatus::Warn,
        GateStatus::Fail,
        GateStatus::Skipped,
        GateStatus::Pending,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: GateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ─── 4. RiskLevel all variants roundtrip ────────────────────────────────────

#[test]
fn risk_level_all_variants_roundtrip() {
    for variant in [
        RiskLevel::Low,
        RiskLevel::Medium,
        RiskLevel::High,
        RiskLevel::Critical,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: RiskLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ─── 5. ComplexityIndicator all variants ────────────────────────────────────

#[test]
fn complexity_indicator_all_variants_roundtrip() {
    for variant in [
        ComplexityIndicator::Low,
        ComplexityIndicator::Medium,
        ComplexityIndicator::High,
        ComplexityIndicator::Critical,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ComplexityIndicator = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ─── 6. WarningType all variants ────────────────────────────────────────────

#[test]
fn warning_type_all_variants_roundtrip() {
    for variant in [
        WarningType::LargeFile,
        WarningType::HighChurn,
        WarningType::LowTestCoverage,
        WarningType::ComplexChange,
        WarningType::BusFactor,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: WarningType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ─── 7. TrendDirection all variants ─────────────────────────────────────────

#[test]
fn trend_direction_all_variants_roundtrip() {
    for variant in [
        TrendDirection::Improving,
        TrendDirection::Stable,
        TrendDirection::Degrading,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: TrendDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ─── 8. EvidenceSource and CommitMatch variants ─────────────────────────────

#[test]
fn evidence_source_all_variants_roundtrip() {
    for variant in [
        EvidenceSource::CiArtifact,
        EvidenceSource::Cached,
        EvidenceSource::RanLocal,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: EvidenceSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn commit_match_all_variants_roundtrip() {
    for variant in [
        CommitMatch::Exact,
        CommitMatch::Partial,
        CommitMatch::Stale,
        CommitMatch::Unknown,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CommitMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ─── 9. Evidence with all gates populated ───────────────────────────────────

#[test]
fn evidence_full_roundtrip() {
    let evidence = Evidence {
        overall_status: GateStatus::Warn,
        mutation: MutationGate {
            meta: sample_gate_meta(GateStatus::Fail),
            survivors: vec![MutationSurvivor {
                file: "src/lib.rs".to_string(),
                line: 42,
                mutation: "replaced + with -".to_string(),
            }],
            killed: 8,
            timeout: 2,
            unviable: 1,
        },
        diff_coverage: Some(DiffCoverageGate {
            meta: sample_gate_meta(GateStatus::Pass),
            lines_added: 50,
            lines_covered: 45,
            coverage_pct: 90.0,
            uncovered_hunks: vec![UncoveredHunk {
                file: "src/new.rs".to_string(),
                start_line: 10,
                end_line: 14,
            }],
        }),
        contracts: Some(ContractDiffGate {
            meta: sample_gate_meta(GateStatus::Warn),
            semver: Some(SemverSubGate {
                status: GateStatus::Fail,
                breaking_changes: vec![BreakingChange {
                    kind: "removed".to_string(),
                    path: "crate::foo".to_string(),
                    message: "Function removed".to_string(),
                }],
            }),
            cli: Some(CliSubGate {
                status: GateStatus::Pass,
                diff_summary: None,
            }),
            schema: Some(SchemaSubGate {
                status: GateStatus::Pass,
                diff_summary: Some("field added".to_string()),
            }),
            failures: 1,
        }),
        supply_chain: Some(SupplyChainGate {
            meta: sample_gate_meta(GateStatus::Pass),
            vulnerabilities: vec![Vulnerability {
                id: "RUSTSEC-2024-0001".to_string(),
                package: "serde".to_string(),
                severity: "low".to_string(),
                title: "Test vuln".to_string(),
            }],
            denied: vec!["evil-crate".to_string()],
            advisory_db_version: Some("2024-01-01".to_string()),
        }),
        determinism: Some(DeterminismGate {
            meta: sample_gate_meta(GateStatus::Pass),
            expected_hash: Some("abc123".to_string()),
            actual_hash: Some("abc123".to_string()),
            algo: "blake3".to_string(),
            differences: vec![],
        }),
        complexity: Some(ComplexityGate {
            meta: sample_gate_meta(GateStatus::Warn),
            files_analyzed: 25,
            high_complexity_files: vec![HighComplexityFile {
                path: "src/complex.rs".to_string(),
                cyclomatic: 20,
                function_count: 5,
                max_function_length: 150,
            }],
            avg_cyclomatic: 8.5,
            max_cyclomatic: 20,
            threshold_exceeded: true,
        }),
    };

    let json = serde_json::to_string(&evidence).unwrap();
    let back: Evidence = serde_json::from_str(&json).unwrap();
    assert_eq!(back.overall_status, GateStatus::Warn);
    assert_eq!(back.mutation.survivors.len(), 1);
    assert!(back.diff_coverage.is_some());
    assert!(back.contracts.is_some());
    assert!(back.supply_chain.is_some());
    assert!(back.determinism.is_some());
    assert!(back.complexity.is_some());
}

// ─── 10. TrendComparison roundtrip ──────────────────────────────────────────

#[test]
fn trend_comparison_full_roundtrip() {
    let trend = TrendComparison {
        baseline_available: true,
        baseline_path: Some("/path/to/baseline.json".to_string()),
        baseline_generated_at_ms: Some(1699000000000),
        health: Some(TrendMetric {
            current: 90.0,
            previous: 85.0,
            delta: 5.0,
            delta_pct: 5.88,
            direction: TrendDirection::Improving,
        }),
        risk: Some(TrendMetric {
            current: 20.0,
            previous: 30.0,
            delta: -10.0,
            delta_pct: -33.33,
            direction: TrendDirection::Improving,
        }),
        complexity: Some(TrendIndicator {
            direction: TrendDirection::Degrading,
            summary: "Complexity increased in 3 files".to_string(),
            files_increased: 3,
            files_decreased: 1,
            avg_cyclomatic_delta: Some(2.5),
            avg_cognitive_delta: Some(1.8),
        }),
    };

    let json = serde_json::to_string(&trend).unwrap();
    let back: TrendComparison = serde_json::from_str(&json).unwrap();
    assert!(back.baseline_available);
    let health = back.health.unwrap();
    assert_eq!(health.direction, TrendDirection::Improving);
    assert_eq!(health.delta, 5.0);
    let complexity = back.complexity.unwrap();
    assert_eq!(complexity.files_increased, 3);
}

// ─── 11. CockpitReceipt with trend populated ───────────────────────────────

#[test]
fn cockpit_receipt_with_trend_roundtrip() {
    let mut receipt = sample_cockpit_receipt();
    receipt.trend = Some(TrendComparison {
        baseline_available: true,
        baseline_path: Some("baseline.json".to_string()),
        baseline_generated_at_ms: Some(1699000000000),
        health: Some(TrendMetric {
            current: 85.0,
            previous: 80.0,
            delta: 5.0,
            delta_pct: 6.25,
            direction: TrendDirection::Improving,
        }),
        risk: None,
        complexity: None,
    });

    let json = serde_json::to_string(&receipt).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert!(back.trend.is_some());
    let trend = back.trend.unwrap();
    assert!(trend.baseline_available);
}

// ─── 12. Deterministic JSON output ──────────────────────────────────────────

#[test]
fn deterministic_cockpit_json() {
    let r1 = sample_cockpit_receipt();
    let r2 = sample_cockpit_receipt();
    let json1 = serde_json::to_string(&r1).unwrap();
    let json2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(json1, json2, "JSON output must be deterministic");
}

// ─── 13. Optional evidence fields omitted when None ─────────────────────────

#[test]
fn optional_evidence_fields_omitted_when_none() {
    let evidence = sample_evidence();
    let json = serde_json::to_string(&evidence).unwrap();
    assert!(!json.contains("\"diff_coverage\""));
    assert!(!json.contains("\"supply_chain\""));
    assert!(!json.contains("\"determinism\""));
    assert!(!json.contains("\"complexity\""));
}

// ─── 14. ReviewItem roundtrip ───────────────────────────────────────────────

#[test]
fn review_item_roundtrip() {
    let items = vec![
        ReviewItem {
            path: "src/api.rs".to_string(),
            reason: "contract change".to_string(),
            priority: 1,
            complexity: Some(4),
            lines_changed: Some(120),
        },
        ReviewItem {
            path: "tests/test.rs".to_string(),
            reason: "test coverage".to_string(),
            priority: 3,
            complexity: None,
            lines_changed: None,
        },
    ];

    for item in &items {
        let json = serde_json::to_string(item).unwrap();
        let back: ReviewItem = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, item.path);
        assert_eq!(back.priority, item.priority);
        assert_eq!(back.complexity, item.complexity);
        assert_eq!(back.lines_changed, item.lines_changed);
    }
}

// ─── 15. Property: health score and risk score survive roundtrip ────────────

proptest! {
    #[test]
    fn prop_health_score_roundtrip(score in 0u32..101) {
        let health = CodeHealth {
            score,
            grade: "A".to_string(),
            large_files_touched: 0,
            avg_file_size: 100,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: vec![],
        };
        let json = serde_json::to_string(&health).unwrap();
        let back: CodeHealth = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.score, score);
    }

    #[test]
    fn prop_risk_score_roundtrip(score in 0u32..101) {
        let risk = Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Low,
            score,
        };
        let json = serde_json::to_string(&risk).unwrap();
        let back: Risk = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.score, score);
    }

    #[test]
    fn prop_change_surface_roundtrip(
        commits in 0usize..1000,
        files in 0usize..500,
        ins in 0usize..50000,
        del in 0usize..50000,
    ) {
        let net = ins as i64 - del as i64;
        let velocity = if commits > 0 { (ins + del) as f64 / commits as f64 } else { 0.0 };
        let surface = ChangeSurface {
            commits,
            files_changed: files,
            insertions: ins,
            deletions: del,
            net_lines: net,
            churn_velocity: velocity,
            change_concentration: 0.5,
        };
        let json = serde_json::to_string(&surface).unwrap();
        let back: ChangeSurface = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.commits, commits);
        prop_assert_eq!(back.files_changed, files);
        prop_assert_eq!(back.net_lines, net);
    }
}
