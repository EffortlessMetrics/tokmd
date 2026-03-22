//! BDD-style integration tests for cockpit receipt types.
//!
//! These tests verify CockpitReceipt construction, serialization,
//! evidence gate evaluation, change surface calculations, and
//! schema version correctness.

use serde_json::Value;
use tokmd_types::cockpit::{
    BreakingChange, COCKPIT_SCHEMA_VERSION, ChangeSurface, CliSubGate, CockpitReceipt, CodeHealth,
    CommitMatch, ComplexityGate, ComplexityIndicator, Composition, ContractDiffGate, Contracts,
    DeterminismGate, DiffCoverageGate, Evidence, EvidenceSource, GateMeta, GateStatus,
    HealthWarning, HighComplexityFile, MutationGate, MutationSurvivor, ReviewItem, Risk, RiskLevel,
    SchemaSubGate, ScopeCoverage, SemverSubGate, SupplyChainGate, TrendComparison, TrendDirection,
    TrendIndicator, TrendMetric, UncoveredHunk, Vulnerability, WarningType,
};

// =============================================================================
// Helpers
// =============================================================================

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
        evidence_commit: Some("abc123".to_string()),
        evidence_generated_at_ms: Some(1700000000000),
    }
}

fn sample_mutation_gate(status: GateStatus) -> MutationGate {
    MutationGate {
        meta: sample_gate_meta(status),
        survivors: vec![],
        killed: 10,
        timeout: 1,
        unviable: 2,
    }
}

fn sample_evidence(overall: GateStatus) -> Evidence {
    Evidence {
        overall_status: overall,
        mutation: sample_mutation_gate(overall),
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
        head_ref: "feature/add-tests".to_string(),
        change_surface: ChangeSurface {
            commits: 5,
            files_changed: 12,
            insertions: 350,
            deletions: 120,
            net_lines: 230,
            churn_velocity: 94.0,
            change_concentration: 0.75,
        },
        composition: Composition {
            code_pct: 65.0,
            test_pct: 25.0,
            docs_pct: 5.0,
            config_pct: 5.0,
            test_ratio: 0.38,
        },
        code_health: CodeHealth {
            score: 82,
            grade: "B".to_string(),
            large_files_touched: 1,
            avg_file_size: 180,
            complexity_indicator: ComplexityIndicator::Medium,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec!["src/core.rs".to_string()],
            bus_factor_warnings: vec![],
            level: RiskLevel::Medium,
            score: 45,
        },
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
        evidence: sample_evidence(GateStatus::Pass),
        review_plan: vec![ReviewItem {
            path: "src/core.rs".to_string(),
            reason: "hotspot + high churn".to_string(),
            priority: 1,
            complexity: Some(3),
            lines_changed: Some(85),
        }],
        trend: None,
    }
}

// =============================================================================
// Scenario: Schema version is correct
// =============================================================================

#[test]
fn given_cockpit_schema_version_then_it_equals_three() {
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3);
}

#[test]
fn given_cockpit_receipt_then_schema_version_matches_constant() {
    let receipt = sample_cockpit_receipt();
    assert_eq!(receipt.schema_version, COCKPIT_SCHEMA_VERSION);
}

// =============================================================================
// Scenario: CockpitReceipt construction and field access
// =============================================================================

#[test]
fn given_cockpit_receipt_then_all_required_fields_accessible() {
    let receipt = sample_cockpit_receipt();

    assert_eq!(receipt.mode, "cockpit");
    assert_eq!(receipt.base_ref, "main");
    assert_eq!(receipt.head_ref, "feature/add-tests");
    assert_eq!(receipt.change_surface.commits, 5);
    assert_eq!(receipt.composition.code_pct, 65.0);
    assert_eq!(receipt.code_health.score, 82);
    assert_eq!(receipt.risk.level, RiskLevel::Medium);
    assert_eq!(receipt.contracts.breaking_indicators, 0);
    assert_eq!(receipt.evidence.overall_status, GateStatus::Pass);
    assert_eq!(receipt.review_plan.len(), 1);
    assert!(receipt.trend.is_none());
}

// =============================================================================
// Scenario: CockpitReceipt JSON serialization
// =============================================================================

#[test]
fn given_cockpit_receipt_when_serialized_then_json_has_required_envelope_fields() {
    let receipt = sample_cockpit_receipt();
    let json: Value = serde_json::to_value(receipt).unwrap();

    assert_eq!(json["schema_version"], COCKPIT_SCHEMA_VERSION);
    assert!(json["generated_at_ms"].is_number());
    assert_eq!(json["mode"], "cockpit");
    assert!(json["base_ref"].is_string());
    assert!(json["head_ref"].is_string());
    assert!(json["change_surface"].is_object());
    assert!(json["composition"].is_object());
    assert!(json["code_health"].is_object());
    assert!(json["risk"].is_object());
    assert!(json["contracts"].is_object());
    assert!(json["evidence"].is_object());
    assert!(json["review_plan"].is_array());
}

#[test]
fn given_cockpit_receipt_when_trend_is_none_then_omitted_from_json() {
    let receipt = sample_cockpit_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    assert!(!json.contains("\"trend\""));
}

#[test]
fn given_cockpit_receipt_with_trend_then_trend_in_json() {
    let mut receipt = sample_cockpit_receipt();
    receipt.trend = Some(TrendComparison {
        baseline_available: true,
        baseline_path: Some("baseline.json".to_string()),
        baseline_generated_at_ms: Some(1699000000000),
        health: Some(TrendMetric {
            current: 82.0,
            previous: 78.0,
            delta: 4.0,
            delta_pct: 5.13,
            direction: TrendDirection::Improving,
        }),
        risk: None,
        complexity: None,
    });

    let json: Value = serde_json::to_value(receipt).unwrap();
    assert!(json["trend"].is_object());
    assert_eq!(json["trend"]["baseline_available"], true);
    assert_eq!(json["trend"]["health"]["direction"], "improving");
}

// =============================================================================
// Scenario: CockpitReceipt serde roundtrip
// =============================================================================

#[test]
fn given_cockpit_receipt_when_roundtripped_then_data_preserved() {
    let receipt = sample_cockpit_receipt();
    let json_str = serde_json::to_string(&receipt).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json_str).unwrap();

    assert_eq!(back.schema_version, receipt.schema_version);
    assert_eq!(back.mode, receipt.mode);
    assert_eq!(back.base_ref, receipt.base_ref);
    assert_eq!(back.head_ref, receipt.head_ref);
    assert_eq!(back.change_surface.commits, receipt.change_surface.commits);
    assert_eq!(back.composition.test_ratio, receipt.composition.test_ratio);
    assert_eq!(back.code_health.grade, receipt.code_health.grade);
    assert_eq!(back.risk.score, receipt.risk.score);
    assert_eq!(
        back.evidence.overall_status,
        receipt.evidence.overall_status
    );
    assert_eq!(back.review_plan.len(), receipt.review_plan.len());
}

// =============================================================================
// Scenario: Deterministic JSON output
// =============================================================================

#[test]
fn given_cockpit_receipt_when_serialized_twice_then_output_identical() {
    let receipt = sample_cockpit_receipt();
    let json1 = serde_json::to_string_pretty(&receipt).unwrap();
    let json2 = serde_json::to_string_pretty(&receipt).unwrap();
    assert_eq!(json1, json2);
}

// =============================================================================
// Scenario: ChangeSurface calculations
// =============================================================================

#[test]
fn given_change_surface_then_net_lines_equals_insertions_minus_deletions() {
    let cs = ChangeSurface {
        commits: 3,
        files_changed: 8,
        insertions: 200,
        deletions: 50,
        net_lines: 150,
        churn_velocity: 83.3,
        change_concentration: 0.6,
    };
    assert_eq!(cs.net_lines, cs.insertions as i64 - cs.deletions as i64);
}

#[test]
fn given_change_surface_with_more_deletions_then_net_lines_negative() {
    let cs = ChangeSurface {
        commits: 1,
        files_changed: 5,
        insertions: 10,
        deletions: 80,
        net_lines: -70,
        churn_velocity: 90.0,
        change_concentration: 0.9,
    };
    assert!(cs.net_lines < 0);
}

#[test]
fn given_change_surface_when_serialized_then_all_fields_present() {
    let cs = ChangeSurface {
        commits: 2,
        files_changed: 4,
        insertions: 100,
        deletions: 30,
        net_lines: 70,
        churn_velocity: 65.0,
        change_concentration: 0.5,
    };
    let json: Value = serde_json::to_value(cs).unwrap();
    assert!(json["commits"].is_number());
    assert!(json["files_changed"].is_number());
    assert!(json["insertions"].is_number());
    assert!(json["deletions"].is_number());
    assert!(json["net_lines"].is_number());
    assert!(json["churn_velocity"].is_number());
    assert!(json["change_concentration"].is_number());
}

// =============================================================================
// Scenario: Evidence gate evaluation
// =============================================================================

#[test]
fn given_all_gates_pass_then_overall_status_pass() {
    let evidence = sample_evidence(GateStatus::Pass);
    assert_eq!(evidence.overall_status, GateStatus::Pass);
}

#[test]
fn given_mutation_gate_with_survivors_then_survivors_accessible() {
    let gate = MutationGate {
        meta: sample_gate_meta(GateStatus::Warn),
        survivors: vec![
            MutationSurvivor {
                file: "src/calc.rs".to_string(),
                line: 42,
                mutation: "replaced + with -".to_string(),
            },
            MutationSurvivor {
                file: "src/parse.rs".to_string(),
                line: 100,
                mutation: "replaced == with !=".to_string(),
            },
        ],
        killed: 48,
        timeout: 2,
        unviable: 0,
    };

    assert_eq!(gate.survivors.len(), 2);
    assert_eq!(gate.killed, 48);
    assert_eq!(gate.survivors[0].file, "src/calc.rs");
    assert_eq!(gate.survivors[0].line, 42);
}

#[test]
fn given_mutation_gate_when_serialized_then_meta_fields_flattened() {
    let gate = sample_mutation_gate(GateStatus::Pass);
    let json: Value = serde_json::to_value(gate).unwrap();

    // GateMeta is flattened, so status/source/commit_match appear at top level
    assert_eq!(json["status"], "pass");
    assert_eq!(json["source"], "ran_local");
    assert_eq!(json["commit_match"], "exact");
    assert!(json["scope"].is_object());
    assert!(json["survivors"].is_array());
    assert!(json["killed"].is_number());
}

#[test]
fn given_diff_coverage_gate_then_coverage_pct_accessible() {
    let gate = DiffCoverageGate {
        meta: sample_gate_meta(GateStatus::Warn),
        lines_added: 100,
        lines_covered: 75,
        coverage_pct: 75.0,
        uncovered_hunks: vec![UncoveredHunk {
            file: "src/new.rs".to_string(),
            start_line: 10,
            end_line: 20,
        }],
    };

    assert_eq!(gate.coverage_pct, 75.0);
    assert_eq!(gate.uncovered_hunks.len(), 1);
    assert_eq!(gate.uncovered_hunks[0].file, "src/new.rs");
}

#[test]
fn given_diff_coverage_gate_when_roundtripped_then_preserved() {
    let gate = DiffCoverageGate {
        meta: sample_gate_meta(GateStatus::Pass),
        lines_added: 50,
        lines_covered: 50,
        coverage_pct: 100.0,
        uncovered_hunks: vec![],
    };
    let json_str = serde_json::to_string(&gate).unwrap();
    let back: DiffCoverageGate = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.coverage_pct, 100.0);
    assert_eq!(back.lines_added, 50);
}

#[test]
fn given_supply_chain_gate_with_vulnerabilities_then_accessible() {
    let gate = SupplyChainGate {
        meta: sample_gate_meta(GateStatus::Fail),
        vulnerabilities: vec![Vulnerability {
            id: "RUSTSEC-2024-001".to_string(),
            package: "risky-crate".to_string(),
            severity: "high".to_string(),
            title: "Memory safety issue".to_string(),
        }],
        denied: vec!["banned-license".to_string()],
        advisory_db_version: Some("2024-01-01".to_string()),
    };

    assert_eq!(gate.vulnerabilities.len(), 1);
    assert_eq!(gate.vulnerabilities[0].id, "RUSTSEC-2024-001");
    assert_eq!(gate.denied.len(), 1);
}

#[test]
fn given_determinism_gate_with_mismatch_then_differences_listed() {
    let gate = DeterminismGate {
        meta: sample_gate_meta(GateStatus::Fail),
        expected_hash: Some("expected_abc".to_string()),
        actual_hash: Some("actual_def".to_string()),
        algo: "blake3".to_string(),
        differences: vec!["row ordering changed".to_string()],
    };

    assert_eq!(gate.algo, "blake3");
    assert_eq!(gate.differences.len(), 1);
    assert!(gate.expected_hash.is_some());
    assert!(gate.actual_hash.is_some());
}

#[test]
fn given_complexity_gate_with_high_files_then_threshold_exceeded() {
    let gate = ComplexityGate {
        meta: sample_gate_meta(GateStatus::Warn),
        files_analyzed: 20,
        high_complexity_files: vec![HighComplexityFile {
            path: "src/monster.rs".to_string(),
            cyclomatic: 45,
            function_count: 12,
            max_function_length: 200,
        }],
        avg_cyclomatic: 8.5,
        max_cyclomatic: 45,
        threshold_exceeded: true,
    };

    assert!(gate.threshold_exceeded);
    assert_eq!(gate.high_complexity_files.len(), 1);
    assert_eq!(gate.high_complexity_files[0].cyclomatic, 45);
    assert_eq!(gate.max_cyclomatic, 45);
}

#[test]
fn given_contract_diff_gate_with_semver_break_then_failures_nonzero() {
    let gate = ContractDiffGate {
        meta: sample_gate_meta(GateStatus::Fail),
        semver: Some(SemverSubGate {
            status: GateStatus::Fail,
            breaking_changes: vec![BreakingChange {
                kind: "removed_function".to_string(),
                path: "tokmd_core::old_fn".to_string(),
                message: "public function was removed".to_string(),
            }],
        }),
        cli: Some(CliSubGate {
            status: GateStatus::Pass,
            diff_summary: None,
        }),
        schema: Some(SchemaSubGate {
            status: GateStatus::Pass,
            diff_summary: None,
        }),
        failures: 1,
    };

    assert_eq!(gate.failures, 1);
    assert_eq!(gate.semver.as_ref().unwrap().status, GateStatus::Fail);
    assert_eq!(gate.semver.as_ref().unwrap().breaking_changes.len(), 1);
}

// =============================================================================
// Scenario: Evidence with all optional gates populated
// =============================================================================

#[test]
fn given_evidence_with_all_gates_when_serialized_then_all_present() {
    let evidence = Evidence {
        overall_status: GateStatus::Warn,
        mutation: sample_mutation_gate(GateStatus::Pass),
        diff_coverage: Some(DiffCoverageGate {
            meta: sample_gate_meta(GateStatus::Pass),
            lines_added: 100,
            lines_covered: 95,
            coverage_pct: 95.0,
            uncovered_hunks: vec![],
        }),
        contracts: Some(ContractDiffGate {
            meta: sample_gate_meta(GateStatus::Pass),
            semver: None,
            cli: None,
            schema: None,
            failures: 0,
        }),
        supply_chain: Some(SupplyChainGate {
            meta: sample_gate_meta(GateStatus::Pass),
            vulnerabilities: vec![],
            denied: vec![],
            advisory_db_version: None,
        }),
        determinism: Some(DeterminismGate {
            meta: sample_gate_meta(GateStatus::Pass),
            expected_hash: None,
            actual_hash: None,
            algo: "blake3".to_string(),
            differences: vec![],
        }),
        complexity: Some(ComplexityGate {
            meta: sample_gate_meta(GateStatus::Pass),
            files_analyzed: 10,
            high_complexity_files: vec![],
            avg_cyclomatic: 3.2,
            max_cyclomatic: 12,
            threshold_exceeded: false,
        }),
    };

    let json: Value = serde_json::to_value(evidence).unwrap();
    assert!(json["mutation"].is_object());
    assert!(json["diff_coverage"].is_object());
    assert!(json["contracts"].is_object());
    assert!(json["supply_chain"].is_object());
    assert!(json["determinism"].is_object());
    assert!(json["complexity"].is_object());
}

#[test]
fn given_evidence_without_optional_gates_then_omitted_from_json() {
    let evidence = sample_evidence(GateStatus::Pass);
    let json = serde_json::to_string(&evidence).unwrap();

    assert!(!json.contains("\"diff_coverage\""));
    assert!(!json.contains("\"contracts\""));
    assert!(!json.contains("\"supply_chain\""));
    assert!(!json.contains("\"determinism\""));
    assert!(!json.contains("\"complexity\""));
}

// =============================================================================
// Scenario: Scope coverage ratio validation
// =============================================================================

#[test]
fn given_scope_coverage_when_all_tested_then_ratio_is_one() {
    let scope = ScopeCoverage {
        relevant: vec!["a.rs".to_string(), "b.rs".to_string()],
        tested: vec!["a.rs".to_string(), "b.rs".to_string()],
        ratio: 1.0,
        lines_relevant: Some(500),
        lines_tested: Some(500),
    };
    assert!((scope.ratio - 1.0).abs() < f64::EPSILON);
}

#[test]
fn given_scope_coverage_when_partially_tested_then_ratio_less_than_one() {
    let scope = ScopeCoverage {
        relevant: vec!["a.rs".to_string(), "b.rs".to_string()],
        tested: vec!["a.rs".to_string()],
        ratio: 0.5,
        lines_relevant: None,
        lines_tested: None,
    };
    assert!(scope.ratio < 1.0);
}

// =============================================================================
// Scenario: CodeHealth and composition validation
// =============================================================================

#[test]
fn given_code_health_with_warnings_then_warnings_accessible() {
    let health = CodeHealth {
        score: 60,
        grade: "C".to_string(),
        large_files_touched: 3,
        avg_file_size: 500,
        complexity_indicator: ComplexityIndicator::High,
        warnings: vec![
            HealthWarning {
                path: "src/big.rs".to_string(),
                warning_type: WarningType::LargeFile,
                message: "File exceeds 500 lines".to_string(),
            },
            HealthWarning {
                path: "src/hot.rs".to_string(),
                warning_type: WarningType::HighChurn,
                message: "High change frequency".to_string(),
            },
        ],
    };

    assert_eq!(health.warnings.len(), 2);
    assert_eq!(health.warnings[0].warning_type, WarningType::LargeFile);
    assert_eq!(health.warnings[1].warning_type, WarningType::HighChurn);
}

#[test]
fn given_composition_then_percentages_sum_to_100() {
    let comp = Composition {
        code_pct: 60.0,
        test_pct: 25.0,
        docs_pct: 10.0,
        config_pct: 5.0,
        test_ratio: 0.42,
    };
    let sum = comp.code_pct + comp.test_pct + comp.docs_pct + comp.config_pct;
    assert!((sum - 100.0).abs() < f64::EPSILON);
}

// =============================================================================
// Scenario: ReviewItem ordering by priority
// =============================================================================

#[test]
fn given_review_items_when_sorted_by_priority_then_lowest_first() {
    let mut items = [
        ReviewItem {
            path: "c.rs".to_string(),
            reason: "low".to_string(),
            priority: 3,
            complexity: None,
            lines_changed: None,
        },
        ReviewItem {
            path: "a.rs".to_string(),
            reason: "high".to_string(),
            priority: 1,
            complexity: Some(5),
            lines_changed: Some(200),
        },
        ReviewItem {
            path: "b.rs".to_string(),
            reason: "medium".to_string(),
            priority: 2,
            complexity: Some(3),
            lines_changed: Some(50),
        },
    ];

    items.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
    assert_eq!(items[0].path, "a.rs");
    assert_eq!(items[1].path, "b.rs");
    assert_eq!(items[2].path, "c.rs");
}

#[test]
fn given_review_item_with_optional_fields_omitted_then_json_clean() {
    let item = ReviewItem {
        path: "x.rs".to_string(),
        reason: "test".to_string(),
        priority: 1,
        complexity: None,
        lines_changed: None,
    };
    let json = serde_json::to_string(&item).unwrap();
    assert!(!json.contains("\"complexity\""));
    assert!(!json.contains("\"lines_changed\""));
}

// =============================================================================
// Scenario: Trend comparison
// =============================================================================

#[test]
fn given_trend_comparison_default_then_baseline_unavailable() {
    let trend = TrendComparison::default();
    assert!(!trend.baseline_available);
    assert!(trend.baseline_path.is_none());
    assert!(trend.health.is_none());
    assert!(trend.risk.is_none());
    assert!(trend.complexity.is_none());
}

#[test]
fn given_trend_metric_improving_then_positive_delta() {
    let metric = TrendMetric {
        current: 85.0,
        previous: 75.0,
        delta: 10.0,
        delta_pct: 13.33,
        direction: TrendDirection::Improving,
    };
    assert!(metric.delta > 0.0);
    assert_eq!(metric.direction, TrendDirection::Improving);
}

#[test]
fn given_trend_metric_degrading_then_negative_delta() {
    let metric = TrendMetric {
        current: 60.0,
        previous: 80.0,
        delta: -20.0,
        delta_pct: -25.0,
        direction: TrendDirection::Degrading,
    };
    assert!(metric.delta < 0.0);
    assert_eq!(metric.direction, TrendDirection::Degrading);
}

#[test]
fn given_trend_indicator_with_complexity_deltas_then_roundtrip() {
    let indicator = TrendIndicator {
        direction: TrendDirection::Stable,
        summary: "No significant changes".to_string(),
        files_increased: 2,
        files_decreased: 2,
        avg_cyclomatic_delta: Some(0.1),
        avg_cognitive_delta: Some(-0.05),
    };

    let json_str = serde_json::to_string(&indicator).unwrap();
    let back: TrendIndicator = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.direction, TrendDirection::Stable);
    assert_eq!(back.files_increased, 2);
    assert!(back.avg_cyclomatic_delta.is_some());
}

// =============================================================================
// Scenario: Full cockpit receipt snapshot (insta)
// =============================================================================

#[test]
fn snapshot_cockpit_receipt_minimal() {
    let receipt = sample_cockpit_receipt();
    insta::assert_json_snapshot!("cockpit_receipt_minimal", receipt);
}

#[test]
fn snapshot_cockpit_evidence_all_gates() {
    let evidence = Evidence {
        overall_status: GateStatus::Warn,
        mutation: MutationGate {
            meta: sample_gate_meta(GateStatus::Warn),
            survivors: vec![MutationSurvivor {
                file: "src/calc.rs".to_string(),
                line: 42,
                mutation: "replaced + with -".to_string(),
            }],
            killed: 49,
            timeout: 0,
            unviable: 1,
        },
        diff_coverage: Some(DiffCoverageGate {
            meta: sample_gate_meta(GateStatus::Pass),
            lines_added: 100,
            lines_covered: 95,
            coverage_pct: 95.0,
            uncovered_hunks: vec![],
        }),
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: None,
    };
    insta::assert_json_snapshot!("cockpit_evidence_with_gates", evidence);
}

#[test]
fn snapshot_change_surface() {
    let cs = ChangeSurface {
        commits: 3,
        files_changed: 7,
        insertions: 250,
        deletions: 80,
        net_lines: 170,
        churn_velocity: 110.0,
        change_concentration: 0.65,
    };
    insta::assert_json_snapshot!("cockpit_change_surface", cs);
}

// =============================================================================
// Scenario: CockpitReceipt JSON has no null required fields
// =============================================================================

#[test]
fn given_cockpit_receipt_then_json_has_no_null_required_fields() {
    let receipt = sample_cockpit_receipt();
    let json: Value = serde_json::to_value(receipt).unwrap();
    let obj = json.as_object().unwrap();

    for (key, value) in obj {
        assert!(
            !value.is_null(),
            "CockpitReceipt field '{key}' should not be null"
        );
    }
}

// =============================================================================
// Scenario: Risk level ordering
// =============================================================================

#[test]
fn given_risk_levels_then_display_matches_serde() {
    let cases = [
        (RiskLevel::Low, "low"),
        (RiskLevel::Medium, "medium"),
        (RiskLevel::High, "high"),
        (RiskLevel::Critical, "critical"),
    ];
    for (level, expected) in &cases {
        assert_eq!(level.to_string(), *expected);
        let json = serde_json::to_string(level).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

// =============================================================================
// Scenario: Contracts serialization
// =============================================================================

#[test]
fn given_contracts_with_breaking_changes_then_indicators_nonzero() {
    let contracts = Contracts {
        api_changed: true,
        cli_changed: true,
        schema_changed: false,
        breaking_indicators: 2,
    };
    let json: Value = serde_json::to_value(contracts).unwrap();
    assert_eq!(json["api_changed"], true);
    assert_eq!(json["cli_changed"], true);
    assert_eq!(json["schema_changed"], false);
    assert_eq!(json["breaking_indicators"], 2);
}

// =============================================================================
// Scenario: GateMeta with optional fields
// =============================================================================

#[test]
fn given_gate_meta_without_evidence_commit_then_omitted_from_json() {
    let meta = GateMeta {
        status: GateStatus::Skipped,
        source: EvidenceSource::Cached,
        commit_match: CommitMatch::Unknown,
        scope: ScopeCoverage {
            relevant: vec![],
            tested: vec![],
            ratio: 0.0,
            lines_relevant: None,
            lines_tested: None,
        },
        evidence_commit: None,
        evidence_generated_at_ms: None,
    };
    let json = serde_json::to_string(&meta).unwrap();
    assert!(!json.contains("\"evidence_commit\""));
    assert!(!json.contains("\"evidence_generated_at_ms\""));
}
