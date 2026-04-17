//! Depth tests for tokmd-cockpit (w57).
//!
//! Exercises evidence gate evaluation, cockpit report construction,
//! determinism, schema versioning, missing optional metrics, and serde roundtrips.

use tokmd_cockpit::*;

// ═══════════════════════════════════════════════════════════════════
// Helper builders
// ═══════════════════════════════════════════════════════════════════

fn make_gate_meta(status: GateStatus) -> GateMeta {
    GateMeta {
        status,
        source: EvidenceSource::RanLocal,
        commit_match: CommitMatch::Exact,
        scope: ScopeCoverage {
            relevant: vec!["src/lib.rs".into()],
            tested: vec!["src/lib.rs".into()],
            ratio: 1.0,
            lines_relevant: None,
            lines_tested: None,
        },
        evidence_commit: Some("abc123".into()),
        evidence_generated_at_ms: Some(1_700_000_000_000),
    }
}

fn make_mutation_gate(status: GateStatus) -> MutationGate {
    MutationGate {
        meta: make_gate_meta(status),
        survivors: vec![],
        killed: 10,
        timeout: 0,
        unviable: 0,
    }
}

fn make_evidence_all_pass() -> Evidence {
    Evidence {
        overall_status: GateStatus::Pass,
        mutation: make_mutation_gate(GateStatus::Pass),
        diff_coverage: None,
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: None,
    }
}

fn make_minimal_receipt() -> CockpitReceipt {
    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".into(),
        generated_at_ms: 1_700_000_000_000,
        base_ref: "main".into(),
        head_ref: "feature/x".into(),
        change_surface: ChangeSurface {
            commits: 3,
            files_changed: 5,
            insertions: 100,
            deletions: 20,
            net_lines: 80,
            churn_velocity: 40.0,
            change_concentration: 0.6,
        },
        composition: Composition {
            code_pct: 0.6,
            test_pct: 0.2,
            docs_pct: 0.1,
            config_pct: 0.1,
            test_ratio: 0.33,
        },
        code_health: CodeHealth {
            score: 90,
            grade: "A".into(),
            large_files_touched: 0,
            avg_file_size: 50,
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
        evidence: make_evidence_all_pass(),
        review_plan: vec![],
        trend: None,
    }
}

// ═══════════════════════════════════════════════════════════════════
// 1. compute_composition
// ═══════════════════════════════════════════════════════════════════

#[test]
fn composition_pure_code() {
    let files = vec!["src/main.rs", "src/lib.rs"];
    let c = compute_composition(&files);
    assert!((c.code_pct - 1.0).abs() < 1e-10);
    assert!((c.test_pct).abs() < 1e-10);
}

#[test]
fn composition_empty_files() {
    let files: Vec<&str> = vec![];
    let c = compute_composition(&files);
    assert!((c.code_pct).abs() < 1e-10);
    assert!((c.test_ratio).abs() < 1e-10);
}

#[test]
fn composition_test_files_detected() {
    let files = vec!["src/main.rs", "tests/test_main.rs"];
    let c = compute_composition(&files);
    assert!(c.test_pct > 0.0);
    assert!(c.test_ratio > 0.0);
}

#[test]
fn composition_docs_detected() {
    let files = vec!["README.md", "docs/guide.md"];
    let c = compute_composition(&files);
    assert!((c.docs_pct - 1.0).abs() < 1e-10);
}

#[test]
fn composition_config_detected() {
    let files = vec!["Cargo.toml", "settings.json", "config.yml"];
    let c = compute_composition(&files);
    assert!((c.config_pct - 1.0).abs() < 1e-10);
}

#[test]
fn composition_mixed() {
    let files = vec!["src/lib.rs", "tests/test_a.rs", "README.md", "Cargo.toml"];
    let c = compute_composition(&files);
    assert!((c.code_pct - 0.25).abs() < 1e-10);
    assert!((c.test_pct - 0.25).abs() < 1e-10);
    assert!((c.docs_pct - 0.25).abs() < 1e-10);
    assert!((c.config_pct - 0.25).abs() < 1e-10);
}

// ═══════════════════════════════════════════════════════════════════
// 2. detect_contracts
// ═══════════════════════════════════════════════════════════════════

#[test]
fn contracts_none_detected() {
    let files = vec!["src/main.rs"];
    let c = detect_contracts(&files);
    assert!(!c.api_changed);
    assert!(!c.cli_changed);
    assert!(!c.schema_changed);
    assert_eq!(c.breaking_indicators, 0);
}

#[test]
fn contracts_api_change() {
    let files = vec!["crates/tokmd-core/src/lib.rs"];
    let c = detect_contracts(&files);
    assert!(c.api_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_schema_change() {
    let files = vec!["docs/schema.json"];
    let c = detect_contracts(&files);
    assert!(c.schema_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_cli_change() {
    let files = vec!["crates/tokmd/src/commands/lang.rs"];
    let c = detect_contracts(&files);
    assert!(c.cli_changed);
    assert_eq!(c.breaking_indicators, 0); // CLI changes alone don't count as breaking
}

// ═══════════════════════════════════════════════════════════════════
// 3. compute_code_health
// ═══════════════════════════════════════════════════════════════════

#[test]
fn code_health_perfect() {
    let stats = vec![FileStat {
        path: "src/lib.rs".into(),
        insertions: 10,
        deletions: 5,
    }];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let h = compute_code_health(&stats, &contracts);
    assert_eq!(h.score, 100);
    assert_eq!(h.grade, "A");
    assert_eq!(h.complexity_indicator, ComplexityIndicator::Low);
}

#[test]
fn code_health_large_files_degrade_score() {
    let stats = vec![
        FileStat {
            path: "big.rs".into(),
            insertions: 400,
            deletions: 200,
        },
        FileStat {
            path: "big2.rs".into(),
            insertions: 300,
            deletions: 300,
        },
        FileStat {
            path: "big3.rs".into(),
            insertions: 501,
            deletions: 0,
        },
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let h = compute_code_health(&stats, &contracts);
    assert!(h.score < 100);
    assert!(h.large_files_touched >= 2);
    assert!(matches!(
        h.complexity_indicator,
        ComplexityIndicator::Medium | ComplexityIndicator::High
    ));
}

#[test]
fn code_health_breaking_contracts_reduce_score() {
    let stats = vec![FileStat {
        path: "src/lib.rs".into(),
        insertions: 10,
        deletions: 5,
    }];
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: true,
        breaking_indicators: 2,
    };
    let h = compute_code_health(&stats, &contracts);
    assert!(h.score <= 80);
}

#[test]
fn code_health_empty_stats() {
    let stats: Vec<FileStat> = vec![];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let h = compute_code_health(&stats, &contracts);
    assert_eq!(h.score, 100);
    assert_eq!(h.avg_file_size, 0);
}

// ═══════════════════════════════════════════════════════════════════
// 4. compute_risk
// ═══════════════════════════════════════════════════════════════════

#[test]
fn risk_low_for_small_changes() {
    let stats = vec![FileStat {
        path: "src/lib.rs".into(),
        insertions: 5,
        deletions: 3,
    }];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);
    let r = compute_risk(&stats, &contracts, &health);
    assert_eq!(r.level, RiskLevel::Low);
    assert!(r.hotspots_touched.is_empty());
}

#[test]
fn risk_increases_with_hotspots() {
    let stats = vec![FileStat {
        path: "hot.rs".into(),
        insertions: 200,
        deletions: 200,
    }];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);
    let r = compute_risk(&stats, &contracts, &health);
    assert!(!r.hotspots_touched.is_empty());
    assert!(r.score > 0);
}

// ═══════════════════════════════════════════════════════════════════
// 5. generate_review_plan
// ═══════════════════════════════════════════════════════════════════

#[test]
fn review_plan_priority_ordering() {
    let stats = vec![
        FileStat {
            path: "small.rs".into(),
            insertions: 10,
            deletions: 5,
        },
        FileStat {
            path: "big.rs".into(),
            insertions: 200,
            deletions: 100,
        },
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let plan = generate_review_plan(&stats, &contracts);
    assert_eq!(plan.len(), 2);
    // Higher priority (lower number) items should come first
    assert!(plan[0].priority <= plan[1].priority);
}

#[test]
fn review_plan_empty_stats() {
    let stats: Vec<FileStat> = vec![];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let plan = generate_review_plan(&stats, &contracts);
    assert!(plan.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// 6. compute_metric_trend
// ═══════════════════════════════════════════════════════════════════

#[test]
fn trend_improving_higher_is_better() {
    let t = compute_metric_trend(90.0, 80.0, true);
    assert_eq!(t.direction, TrendDirection::Improving);
    assert!((t.delta - 10.0).abs() < 1e-10);
}

#[test]
fn trend_degrading_higher_is_better() {
    let t = compute_metric_trend(70.0, 80.0, true);
    assert_eq!(t.direction, TrendDirection::Degrading);
}

#[test]
fn trend_stable_small_delta() {
    let t = compute_metric_trend(80.0, 80.5, true);
    assert_eq!(t.direction, TrendDirection::Stable);
}

#[test]
fn trend_improving_lower_is_better() {
    let t = compute_metric_trend(5.0, 15.0, false);
    assert_eq!(t.direction, TrendDirection::Improving);
}

#[test]
fn trend_zero_previous() {
    let t = compute_metric_trend(10.0, 0.0, true);
    assert!((t.delta_pct - 100.0).abs() < 1e-10);
}

#[test]
fn trend_both_zero() {
    let t = compute_metric_trend(0.0, 0.0, true);
    assert_eq!(t.direction, TrendDirection::Stable);
    assert!((t.delta_pct).abs() < 1e-10);
}

// ═══════════════════════════════════════════════════════════════════
// 7. Schema version correctness
// ═══════════════════════════════════════════════════════════════════

#[test]
fn schema_version_is_current() {
    let receipt = make_minimal_receipt();
    assert_eq!(receipt.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(receipt.schema_version, 3);
}

// ═══════════════════════════════════════════════════════════════════
// 8. Determinism
// ═══════════════════════════════════════════════════════════════════

#[test]
fn composition_deterministic() {
    let files = vec!["a.rs", "b.rs", "test_c.rs", "README.md"];
    let c1 = compute_composition(&files);
    let c2 = compute_composition(&files);
    assert!((c1.code_pct - c2.code_pct).abs() < 1e-10);
    assert!((c1.test_pct - c2.test_pct).abs() < 1e-10);
    assert!((c1.docs_pct - c2.docs_pct).abs() < 1e-10);
}

#[test]
fn contracts_deterministic() {
    let files = vec!["src/lib.rs", "docs/schema.json"];
    let c1 = detect_contracts(&files);
    let c2 = detect_contracts(&files);
    assert_eq!(c1.api_changed, c2.api_changed);
    assert_eq!(c1.schema_changed, c2.schema_changed);
    assert_eq!(c1.breaking_indicators, c2.breaking_indicators);
}

// ═══════════════════════════════════════════════════════════════════
// 9. Serde roundtrips
// ═══════════════════════════════════════════════════════════════════

#[test]
fn cockpit_receipt_serde_roundtrip() {
    let receipt = make_minimal_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let parsed: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.schema_version, receipt.schema_version);
    assert_eq!(parsed.mode, receipt.mode);
    assert_eq!(parsed.base_ref, receipt.base_ref);
    assert_eq!(parsed.head_ref, receipt.head_ref);
    assert_eq!(
        parsed.change_surface.commits,
        receipt.change_surface.commits
    );
    assert_eq!(parsed.risk.score, receipt.risk.score);
}

#[test]
fn evidence_serde_roundtrip() {
    let ev = make_evidence_all_pass();
    let json = serde_json::to_string(&ev).unwrap();
    let parsed: Evidence = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.overall_status, GateStatus::Pass);
    assert_eq!(parsed.mutation.killed, 10);
}

#[test]
fn gate_status_serde_variants() {
    for status in &[
        GateStatus::Pass,
        GateStatus::Warn,
        GateStatus::Fail,
        GateStatus::Skipped,
        GateStatus::Pending,
    ] {
        let json = serde_json::to_value(status).unwrap();
        let parsed: GateStatus = serde_json::from_value(json).unwrap();
        assert_eq!(&parsed, status);
    }
}

#[test]
fn trend_direction_serde_variants() {
    for dir in &[
        TrendDirection::Improving,
        TrendDirection::Stable,
        TrendDirection::Degrading,
    ] {
        let json = serde_json::to_value(dir).unwrap();
        let parsed: TrendDirection = serde_json::from_value(json).unwrap();
        assert_eq!(&parsed, dir);
    }
}

#[test]
fn risk_level_serde_variants() {
    for level in &[
        RiskLevel::Low,
        RiskLevel::Medium,
        RiskLevel::High,
        RiskLevel::Critical,
    ] {
        let json = serde_json::to_value(level).unwrap();
        let parsed: RiskLevel = serde_json::from_value(json).unwrap();
        assert_eq!(&parsed, level);
    }
}

#[test]
fn complexity_indicator_serde_variants() {
    for ci in &[
        ComplexityIndicator::Low,
        ComplexityIndicator::Medium,
        ComplexityIndicator::High,
        ComplexityIndicator::Critical,
    ] {
        let json = serde_json::to_value(ci).unwrap();
        let parsed: ComplexityIndicator = serde_json::from_value(json).unwrap();
        assert_eq!(&parsed, ci);
    }
}

// ═══════════════════════════════════════════════════════════════════
// 10. Missing optional metrics
// ═══════════════════════════════════════════════════════════════════

#[test]
fn receipt_no_trend() {
    let receipt = make_minimal_receipt();
    assert!(receipt.trend.is_none());
    let json = serde_json::to_string(&receipt).unwrap();
    assert!(!json.contains("\"trend\""));
}

#[test]
fn evidence_no_optional_gates() {
    let ev = make_evidence_all_pass();
    assert!(ev.diff_coverage.is_none());
    assert!(ev.contracts.is_none());
    assert!(ev.supply_chain.is_none());
    assert!(ev.determinism.is_none());
    assert!(ev.complexity.is_none());
}

#[test]
fn receipt_with_all_optional_evidence_roundtrips() {
    let mut receipt = make_minimal_receipt();
    receipt.evidence.complexity = Some(ComplexityGate {
        meta: make_gate_meta(GateStatus::Pass),
        files_analyzed: 10,
        high_complexity_files: vec![],
        avg_cyclomatic: 3.5,
        max_cyclomatic: 8,
        threshold_exceeded: false,
    });
    receipt.evidence.determinism = Some(DeterminismGate {
        meta: make_gate_meta(GateStatus::Pass),
        expected_hash: Some("abc".into()),
        actual_hash: Some("abc".into()),
        algo: "blake3".into(),
        differences: vec![],
    });

    let json = serde_json::to_string(&receipt).unwrap();
    let parsed: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert!(parsed.evidence.complexity.is_some());
    assert!(parsed.evidence.determinism.is_some());
}

// ═══════════════════════════════════════════════════════════════════
// 11. Utility helpers
// ═══════════════════════════════════════════════════════════════════

#[test]
fn round_pct_basic() {
    assert!((round_pct(0.8567) - 0.86).abs() < 0.005);
    assert!((round_pct(1.0) - 1.0).abs() < 1e-10);
    assert!((round_pct(0.0) - 0.0).abs() < 1e-10);
}

#[test]
fn format_signed_positive() {
    assert_eq!(format_signed_f64(1.5), "+1.50");
}

#[test]
fn format_signed_negative() {
    assert_eq!(format_signed_f64(-2.3), "-2.30");
}

#[test]
fn format_signed_zero() {
    assert_eq!(format_signed_f64(0.0), "0.00");
}

#[test]
fn trend_direction_labels() {
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
fn sparkline_empty() {
    assert_eq!(sparkline(&[]), "");
}

#[test]
fn sparkline_single_value() {
    let s = sparkline(&[5.0]);
    assert_eq!(s.chars().count(), 1);
}

#[test]
fn sparkline_ascending() {
    let s = sparkline(&[0.0, 50.0, 100.0]);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars.len(), 3);
    assert!(chars[0] < chars[2]);
}

// ═══════════════════════════════════════════════════════════════════
// 12. Complexity trend
// ═══════════════════════════════════════════════════════════════════

#[test]
fn complexity_trend_stable_when_equal() {
    let current = make_minimal_receipt();
    let baseline = make_minimal_receipt();
    let trend = compute_complexity_trend(&current, &baseline);
    assert_eq!(trend.direction, TrendDirection::Stable);
}

#[test]
fn complexity_trend_degrading_when_increased() {
    let mut current = make_minimal_receipt();
    current.evidence.complexity = Some(ComplexityGate {
        meta: make_gate_meta(GateStatus::Warn),
        files_analyzed: 5,
        high_complexity_files: vec![],
        avg_cyclomatic: 20.0,
        max_cyclomatic: 30,
        threshold_exceeded: true,
    });
    let baseline = make_minimal_receipt();
    let trend = compute_complexity_trend(&current, &baseline);
    assert_eq!(trend.direction, TrendDirection::Degrading);
}
