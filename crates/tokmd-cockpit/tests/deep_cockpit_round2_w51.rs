//! Wave 51 deep round-2 tests for tokmd-cockpit.
//!
//! Covers:
//! - CockpitReceipt construction with all evidence gate variants
//! - Review plan generation with various metric combinations
//! - Health/risk score computation boundaries
//! - Composition percentage calculations (sum to ~100%)
//! - Trend detection (improving/declining/stable)
//! - No git data (capability unavailable) scenario
//! - Determinism: same input → identical cockpit receipt
//! - Schema version in cockpit JSON output

use tokmd_cockpit::*;

// =========================================================================
// Helpers
// =========================================================================

fn stat(path: &str, ins: usize, del: usize) -> FileStat {
    FileStat {
        path: path.to_string(),
        insertions: ins,
        deletions: del,
    }
}

fn make_gate_meta(status: GateStatus) -> GateMeta {
    GateMeta {
        status,
        source: EvidenceSource::RanLocal,
        commit_match: CommitMatch::Exact,
        scope: ScopeCoverage {
            relevant: Vec::new(),
            tested: Vec::new(),
            ratio: 1.0,
            lines_relevant: None,
            lines_tested: None,
        },
        evidence_commit: None,
        evidence_generated_at_ms: None,
    }
}

fn make_evidence_all_gates() -> Evidence {
    Evidence {
        overall_status: GateStatus::Pass,
        mutation: MutationGate {
            meta: make_gate_meta(GateStatus::Pass),
            survivors: vec![MutationSurvivor {
                file: "src/lib.rs".into(),
                line: 42,
                mutation: "replace + with -".into(),
            }],
            killed: 10,
            timeout: 1,
            unviable: 2,
        },
        diff_coverage: Some(DiffCoverageGate {
            meta: make_gate_meta(GateStatus::Pass),
            lines_added: 100,
            lines_covered: 85,
            coverage_pct: 0.85,
            uncovered_hunks: vec![UncoveredHunk {
                file: "src/lib.rs".into(),
                start_line: 50,
                end_line: 65,
            }],
        }),
        contracts: Some(ContractDiffGate {
            meta: make_gate_meta(GateStatus::Warn),
            semver: Some(SemverSubGate {
                status: GateStatus::Warn,
                breaking_changes: vec![BreakingChange {
                    kind: "function_removed".into(),
                    path: "src/api.rs".into(),
                    message: "removed deprecated fn".into(),
                }],
            }),
            cli: Some(CliSubGate {
                status: GateStatus::Pass,
                diff_summary: Some("no changes".into()),
            }),
            schema: Some(SchemaSubGate {
                status: GateStatus::Pass,
                diff_summary: None,
            }),
            failures: 0,
        }),
        supply_chain: Some(SupplyChainGate {
            meta: make_gate_meta(GateStatus::Pass),
            vulnerabilities: vec![Vulnerability {
                id: "RUSTSEC-2024-0001".into(),
                package: "some-crate".into(),
                severity: "low".into(),
                title: "Minor issue".into(),
            }],
            denied: vec![],
            advisory_db_version: Some("2024-01-01".into()),
        }),
        determinism: Some(DeterminismGate {
            meta: make_gate_meta(GateStatus::Pass),
            expected_hash: Some("abc123".into()),
            actual_hash: Some("abc123".into()),
            algo: "blake3".into(),
            differences: vec![],
        }),
        complexity: Some(ComplexityGate {
            meta: make_gate_meta(GateStatus::Pass),
            files_analyzed: 5,
            high_complexity_files: vec![HighComplexityFile {
                path: "src/engine.rs".into(),
                cyclomatic: 20,
                function_count: 8,
                max_function_length: 150,
            }],
            avg_cyclomatic: 8.5,
            max_cyclomatic: 20,
            threshold_exceeded: false,
        }),
    }
}

fn make_receipt(stats: &[FileStat]) -> CockpitReceipt {
    let contracts = detect_contracts(stats);
    let composition = compute_composition(stats);
    let code_health = compute_code_health(stats, &contracts);
    let risk = compute_risk(stats, &contracts, &code_health);
    let review_plan = generate_review_plan(stats, &contracts);

    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 1000,
        base_ref: "main".to_string(),
        head_ref: "feature-branch".to_string(),
        change_surface: ChangeSurface {
            commits: 3,
            files_changed: stats.len(),
            insertions: stats.iter().map(|s| s.insertions).sum(),
            deletions: stats.iter().map(|s| s.deletions).sum(),
            net_lines: stats
                .iter()
                .map(|s| s.insertions as i64 - s.deletions as i64)
                .sum(),
            churn_velocity: 0.0,
            change_concentration: 0.0,
        },
        composition,
        code_health,
        risk,
        contracts,
        evidence: make_evidence_all_gates(),
        review_plan,
        trend: None,
    }
}

// =========================================================================
// 1. Receipt construction with all evidence gate variants
// =========================================================================

#[test]
fn receipt_all_gates_present_roundtrip() {
    let stats = vec![stat("src/lib.rs", 100, 20), stat("tests/foo.rs", 50, 5)];
    let receipt = make_receipt(&stats);
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json).unwrap();

    assert!(back.evidence.diff_coverage.is_some());
    assert!(back.evidence.contracts.is_some());
    assert!(back.evidence.supply_chain.is_some());
    assert!(back.evidence.determinism.is_some());
    assert!(back.evidence.complexity.is_some());
}

#[test]
fn receipt_mutation_survivors_preserved() {
    let receipt = make_receipt(&[stat("src/main.rs", 10, 5)]);
    assert_eq!(receipt.evidence.mutation.survivors.len(), 1);
    assert_eq!(receipt.evidence.mutation.killed, 10);
    assert_eq!(receipt.evidence.mutation.timeout, 1);
    assert_eq!(receipt.evidence.mutation.unviable, 2);
}

#[test]
fn receipt_diff_coverage_fields() {
    let receipt = make_receipt(&[stat("src/main.rs", 10, 5)]);
    let dc = receipt.evidence.diff_coverage.as_ref().unwrap();
    assert_eq!(dc.lines_added, 100);
    assert_eq!(dc.lines_covered, 85);
    assert!((dc.coverage_pct - 0.85).abs() < f64::EPSILON);
    assert_eq!(dc.uncovered_hunks.len(), 1);
}

#[test]
fn receipt_contract_gate_sub_gates() {
    let receipt = make_receipt(&[stat("src/lib.rs", 10, 5)]);
    let cg = receipt.evidence.contracts.as_ref().unwrap();
    assert!(cg.semver.is_some());
    assert!(cg.cli.is_some());
    assert!(cg.schema.is_some());
    assert_eq!(cg.semver.as_ref().unwrap().breaking_changes.len(), 1);
}

#[test]
fn receipt_supply_chain_vulnerability() {
    let receipt = make_receipt(&[stat("src/main.rs", 10, 5)]);
    let sc = receipt.evidence.supply_chain.as_ref().unwrap();
    assert_eq!(sc.vulnerabilities.len(), 1);
    assert_eq!(sc.vulnerabilities[0].id, "RUSTSEC-2024-0001");
    assert!(sc.advisory_db_version.is_some());
}

// =========================================================================
// 2. Review plan generation with various metric combinations
// =========================================================================

#[test]
fn review_plan_priority_ordering() {
    let stats = vec![
        stat("src/big.rs", 300, 100),
        stat("src/small.rs", 10, 5),
        stat("src/medium.rs", 60, 20),
    ];
    let contracts = detect_contracts(&stats);
    let plan = generate_review_plan(&stats, &contracts);

    assert_eq!(plan.len(), 3);
    // Items should be sorted by priority (1 = highest)
    assert!(plan[0].priority <= plan[1].priority);
    assert!(plan[1].priority <= plan[2].priority);
}

#[test]
fn review_plan_complexity_scores() {
    let stats = vec![
        stat("src/huge.rs", 500, 200), // 700 lines -> complexity 5
        stat("src/medium.rs", 80, 40), // 120 lines -> complexity 3
        stat("src/tiny.rs", 5, 2),     // 7 lines   -> complexity 1
    ];
    let contracts = detect_contracts(&stats);
    let plan = generate_review_plan(&stats, &contracts);

    assert_eq!(plan[0].complexity, Some(5));
    let medium = plan.iter().find(|i| i.path == "src/medium.rs").unwrap();
    assert_eq!(medium.complexity, Some(3));
    let tiny = plan.iter().find(|i| i.path == "src/tiny.rs").unwrap();
    assert_eq!(tiny.complexity, Some(1));
}

#[test]
fn review_plan_lines_changed_recorded() {
    let stats = vec![stat("src/lib.rs", 40, 10)];
    let contracts = detect_contracts(&stats);
    let plan = generate_review_plan(&stats, &contracts);

    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].lines_changed, Some(50));
}

#[test]
fn review_plan_empty_input() {
    let contracts = detect_contracts::<FileStat>(&[]);
    let plan = generate_review_plan(&[], &contracts);
    assert!(plan.is_empty());
}

// =========================================================================
// 3. Health/risk score computation boundaries
// =========================================================================

#[test]
fn health_score_perfect_no_large_files() {
    let stats = vec![stat("src/a.rs", 10, 5), stat("src/b.rs", 20, 10)];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);
    assert_eq!(health.score, 100);
    assert_eq!(health.grade, "A");
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Low);
}

#[test]
fn health_score_decreases_with_large_files() {
    let stats = vec![
        stat("src/a.rs", 300, 300),
        stat("src/b.rs", 400, 200),
        stat("src/c.rs", 500, 100),
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);
    assert!(health.score < 100);
    assert!(health.large_files_touched > 0);
}

#[test]
fn health_score_penalized_for_breaking_changes() {
    let stats = vec![stat("src/lib.rs", 10, 5)];
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: true,
        breaking_indicators: 2,
    };
    let health = compute_code_health(&stats, &contracts);
    // Breaking indicators subtract 20 from score
    assert!(health.score <= 80);
}

#[test]
fn health_score_never_below_zero() {
    // Many large files + breaking changes
    let stats: Vec<FileStat> = (0..20)
        .map(|i| stat(&format!("src/file_{i}.rs"), 600, 600))
        .collect();
    let contracts = Contracts {
        api_changed: true,
        cli_changed: true,
        schema_changed: true,
        breaking_indicators: 3,
    };
    let health = compute_code_health(&stats, &contracts);
    // u32 saturating_sub ensures no underflow
    assert!(health.score <= 100); // always valid
}

#[test]
fn risk_score_low_for_small_changes() {
    let stats = vec![stat("src/a.rs", 10, 5)];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);
    let risk = compute_risk(&stats, &contracts, &health);
    assert_eq!(risk.level, RiskLevel::Low);
    assert!(risk.hotspots_touched.is_empty());
}

#[test]
fn risk_score_high_for_many_hotspots() {
    let stats: Vec<FileStat> = (0..5)
        .map(|i| stat(&format!("src/hot_{i}.rs"), 400, 200))
        .collect();
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);
    let risk = compute_risk(&stats, &contracts, &health);
    assert!(!risk.hotspots_touched.is_empty());
    assert!(risk.score > 20);
}

#[test]
fn risk_score_capped_at_100() {
    let stats: Vec<FileStat> = (0..20)
        .map(|i| stat(&format!("src/massive_{i}.rs"), 1000, 500))
        .collect();
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 1,
    };
    let health = compute_code_health(&stats, &contracts);
    let risk = compute_risk(&stats, &contracts, &health);
    assert!(risk.score <= 100);
}

// =========================================================================
// 4. Composition percentage calculations
// =========================================================================

#[test]
fn composition_percentages_sum_to_one() {
    let stats = vec![
        stat("src/main.rs", 100, 50),
        stat("tests/test_a.rs", 50, 10),
        stat("README.md", 20, 5),
        stat("Cargo.toml", 10, 2),
    ];
    let comp = compute_composition(&stats);
    let total = comp.code_pct + comp.test_pct + comp.docs_pct + comp.config_pct;
    assert!(
        (total - 1.0).abs() < 0.01,
        "Composition should sum to ~100%, got {total}"
    );
}

#[test]
fn composition_all_code() {
    let stats = vec![stat("src/a.rs", 10, 5), stat("src/b.rs", 20, 10)];
    let comp = compute_composition(&stats);
    assert!((comp.code_pct - 1.0).abs() < f64::EPSILON);
    assert!((comp.test_pct - 0.0).abs() < f64::EPSILON);
}

#[test]
fn composition_all_tests() {
    let stats = vec![
        stat("tests/test_a.rs", 10, 5),
        stat("tests/test_b.rs", 20, 10),
    ];
    let comp = compute_composition(&stats);
    assert!((comp.test_pct - 1.0).abs() < f64::EPSILON);
    assert!((comp.code_pct - 0.0).abs() < f64::EPSILON);
}

#[test]
fn composition_empty_input_all_zero() {
    let comp = compute_composition::<FileStat>(&[]);
    assert!((comp.code_pct - 0.0).abs() < f64::EPSILON);
    assert!((comp.test_pct - 0.0).abs() < f64::EPSILON);
    assert!((comp.docs_pct - 0.0).abs() < f64::EPSILON);
    assert!((comp.config_pct - 0.0).abs() < f64::EPSILON);
    assert!((comp.test_ratio - 0.0).abs() < f64::EPSILON);
}

#[test]
fn composition_test_ratio_computed() {
    let stats = vec![
        stat("src/main.rs", 10, 5),
        stat("src/lib.rs", 10, 5),
        stat("tests/test_main.rs", 10, 5),
    ];
    let comp = compute_composition(&stats);
    // 1 test file / 2 code files = 0.5
    assert!((comp.test_ratio - 0.5).abs() < f64::EPSILON);
}

// =========================================================================
// 5. Trend detection (improving / declining / stable)
// =========================================================================

#[test]
fn trend_improving_when_health_increases() {
    let trend = compute_metric_trend(90.0, 80.0, true);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert!(trend.delta > 0.0);
}

#[test]
fn trend_degrading_when_health_decreases() {
    let trend = compute_metric_trend(60.0, 80.0, true);
    assert_eq!(trend.direction, TrendDirection::Degrading);
    assert!(trend.delta < 0.0);
}

#[test]
fn trend_stable_when_delta_small() {
    let trend = compute_metric_trend(80.0, 80.5, true);
    assert_eq!(trend.direction, TrendDirection::Stable);
}

#[test]
fn trend_risk_lower_is_better() {
    // Risk: lower is better (higher_is_better=false)
    let trend = compute_metric_trend(20.0, 40.0, false);
    assert_eq!(trend.direction, TrendDirection::Improving);

    let trend = compute_metric_trend(50.0, 30.0, false);
    assert_eq!(trend.direction, TrendDirection::Degrading);
}

#[test]
fn trend_delta_pct_from_zero_baseline() {
    let trend = compute_metric_trend(10.0, 0.0, true);
    assert!((trend.delta_pct - 100.0).abs() < f64::EPSILON);
}

#[test]
fn trend_both_zero_stable() {
    let trend = compute_metric_trend(0.0, 0.0, true);
    assert_eq!(trend.direction, TrendDirection::Stable);
    assert!((trend.delta_pct - 0.0).abs() < f64::EPSILON);
}

#[test]
fn complexity_trend_stable_when_same() {
    let receipt = make_receipt(&[stat("src/a.rs", 10, 5)]);
    let indicator = compute_complexity_trend(&receipt, &receipt);
    assert_eq!(indicator.direction, TrendDirection::Stable);
    assert!(indicator.summary.contains("stable"));
}

// =========================================================================
// 6. No git data (capability unavailable)
// =========================================================================

#[test]
fn receipt_without_trend_serializes_cleanly() {
    let receipt = make_receipt(&[stat("src/a.rs", 10, 5)]);
    assert!(receipt.trend.is_none());
    let json = serde_json::to_string(&receipt).unwrap();
    // trend field should be absent (skip_serializing_if = "Option::is_none")
    assert!(!json.contains("\"trend\""));
}

#[test]
fn evidence_optional_gates_none_serialize_cleanly() {
    let evidence = Evidence {
        overall_status: GateStatus::Skipped,
        mutation: MutationGate {
            meta: make_gate_meta(GateStatus::Skipped),
            survivors: vec![],
            killed: 0,
            timeout: 0,
            unviable: 0,
        },
        diff_coverage: None,
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: None,
    };
    let json = serde_json::to_string(&evidence).unwrap();
    assert!(!json.contains("\"diff_coverage\""));
    assert!(!json.contains("\"contracts\""));
    assert!(!json.contains("\"supply_chain\""));
    assert!(!json.contains("\"determinism\""));
    assert!(!json.contains("\"complexity\""));
}

// =========================================================================
// 7. Determinism: same input → identical cockpit receipt
// =========================================================================

#[test]
fn determinism_same_input_same_output() {
    let stats = vec![
        stat("src/main.rs", 100, 50),
        stat("src/lib.rs", 200, 80),
        stat("tests/test_a.rs", 50, 10),
        stat("Cargo.toml", 5, 2),
        stat("README.md", 30, 10),
    ];
    let r1 = make_receipt(&stats);
    let r2 = make_receipt(&stats);

    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2, "Same input must produce identical JSON");
}

#[test]
fn determinism_composition_stable() {
    let stats = vec![
        stat("src/a.rs", 10, 5),
        stat("tests/t.rs", 5, 1),
        stat("docs/readme.md", 3, 0),
    ];
    let c1 = compute_composition(&stats);
    let c2 = compute_composition(&stats);
    assert!((c1.code_pct - c2.code_pct).abs() < f64::EPSILON);
    assert!((c1.test_pct - c2.test_pct).abs() < f64::EPSILON);
    assert!((c1.docs_pct - c2.docs_pct).abs() < f64::EPSILON);
}

// =========================================================================
// 8. Schema version in cockpit JSON output
// =========================================================================

#[test]
fn schema_version_in_json() {
    let receipt = make_receipt(&[stat("src/main.rs", 10, 5)]);
    let json: serde_json::Value = serde_json::to_value(&receipt).unwrap();
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        COCKPIT_SCHEMA_VERSION as u64
    );
}

#[test]
fn schema_version_constant_is_3() {
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3);
}

// =========================================================================
// 9. Additional edge cases
// =========================================================================

#[test]
fn detect_contracts_api_changed() {
    let files = vec![
        stat("crates/tokmd/src/lib.rs", 10, 5),
        stat("crates/tokmd/src/cli/mod.rs", 5, 2),
    ];
    let contracts = detect_contracts(&files);
    assert!(contracts.api_changed);
    assert!(contracts.cli_changed);
    assert!(!contracts.schema_changed);
}

#[test]
fn overall_status_all_pass() {
    let receipt = make_receipt(&[stat("src/a.rs", 10, 5)]);
    // We constructed all gates as Pass
    assert_eq!(receipt.evidence.overall_status, GateStatus::Pass);
}

#[test]
fn gate_status_serde_roundtrip() {
    for status in [
        GateStatus::Pass,
        GateStatus::Warn,
        GateStatus::Fail,
        GateStatus::Skipped,
        GateStatus::Pending,
    ] {
        let json = serde_json::to_string(&status).unwrap();
        let back: GateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, status);
    }
}

#[test]
fn utility_round_pct() {
    assert!((round_pct(0.8567) - 0.86).abs() < f64::EPSILON);
    assert!((round_pct(0.0) - 0.0).abs() < f64::EPSILON);
    assert!((round_pct(1.0) - 1.0).abs() < f64::EPSILON);
}

#[test]
fn utility_format_signed_f64() {
    assert_eq!(format_signed_f64(5.0), "+5.00");
    assert_eq!(format_signed_f64(-3.5), "-3.50");
    assert_eq!(format_signed_f64(0.0), "0.00");
}

#[test]
fn utility_trend_direction_label() {
    assert_eq!(
        trend_direction_label(TrendDirection::Improving),
        "improving"
    );
    assert_eq!(trend_direction_label(TrendDirection::Stable), "stable");
    assert_eq!(
        trend_direction_label(TrendDirection::Degrading),
        "degrading"
    );
}

#[test]
fn sparkline_non_empty() {
    let s = sparkline(&[1.0, 5.0, 3.0, 8.0, 2.0]);
    assert!(!s.is_empty());
    assert_eq!(s.chars().count(), 5);
}

#[test]
fn sparkline_empty_input() {
    assert!(sparkline(&[]).is_empty());
}
