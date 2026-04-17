//! Deep tests for tokmd-cockpit: composition, contracts, health, risk,
//! review plan, sparkline, trend, overall status, and serde.

use tokmd_cockpit::*;

// =============================================================================
// Helper
// =============================================================================

fn make_stat(path: &str, ins: usize, del: usize) -> FileStat {
    FileStat {
        path: path.to_string(),
        insertions: ins,
        deletions: del,
    }
}

fn no_contracts() -> Contracts {
    Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    }
}

fn make_gate_meta(status: GateStatus) -> GateMeta {
    GateMeta {
        status,
        source: EvidenceSource::RanLocal,
        commit_match: CommitMatch::Unknown,
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

fn make_mutation_gate(status: GateStatus) -> MutationGate {
    MutationGate {
        meta: make_gate_meta(status),
        survivors: vec![],
        killed: 0,
        timeout: 0,
        unviable: 0,
    }
}

// =============================================================================
// compute_composition
// =============================================================================

#[test]
fn composition_all_code() {
    let files = vec!["src/lib.rs", "src/main.rs", "src/utils.rs"];
    let c = compute_composition(&files);
    assert_eq!(c.code_pct, 1.0);
    assert_eq!(c.test_pct, 0.0);
    assert_eq!(c.docs_pct, 0.0);
    assert_eq!(c.config_pct, 0.0);
    assert_eq!(c.test_ratio, 0.0);
}

#[test]
fn composition_all_tests() {
    let files = vec!["tests/test_a.rs", "tests/test_b.rs"];
    let c = compute_composition(&files);
    assert_eq!(c.code_pct, 0.0);
    assert_eq!(c.test_pct, 1.0);
    assert_eq!(c.test_ratio, 1.0);
}

#[test]
fn composition_mixed() {
    let files = vec!["src/lib.rs", "tests/test_lib.rs", "README.md", "Cargo.toml"];
    let c = compute_composition(&files);
    assert!((c.code_pct - 0.25).abs() < 1e-10);
    assert!((c.test_pct - 0.25).abs() < 1e-10);
    assert!((c.docs_pct - 0.25).abs() < 1e-10);
    assert!((c.config_pct - 0.25).abs() < 1e-10);
    assert_eq!(c.test_ratio, 1.0); // 1 test / 1 code
}

#[test]
fn composition_empty() {
    let files: Vec<&str> = vec![];
    let c = compute_composition(&files);
    assert_eq!(c.code_pct, 0.0);
    assert_eq!(c.test_ratio, 0.0);
}

#[test]
fn composition_unrecognized_extensions() {
    let files = vec!["logo.png", "audio.mp3"];
    let c = compute_composition(&files);
    // png/mp3 don't match any category
    assert_eq!(c.code_pct, 0.0);
    assert_eq!(c.test_pct, 0.0);
}

// =============================================================================
// detect_contracts
// =============================================================================

#[test]
fn contracts_api_from_lib_rs() {
    let files = vec!["crates/tokmd-types/src/lib.rs"];
    let c = detect_contracts(&files);
    assert!(c.api_changed);
    assert!(!c.cli_changed);
    assert!(!c.schema_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_cli_from_commands() {
    let files = vec!["crates/tokmd/src/commands/analyze.rs"];
    let c = detect_contracts(&files);
    assert!(c.cli_changed);
    assert!(!c.api_changed);
}

#[test]
fn contracts_schema_from_schema_json() {
    let files = vec!["docs/schema.json"];
    let c = detect_contracts(&files);
    assert!(c.schema_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_schema_md() {
    let files = vec!["docs/SCHEMA.md"];
    let c = detect_contracts(&files);
    assert!(c.schema_changed);
}

#[test]
fn contracts_none() {
    let files = vec!["src/helpers.rs", "README.md"];
    let c = detect_contracts(&files);
    assert!(!c.api_changed);
    assert!(!c.cli_changed);
    assert!(!c.schema_changed);
    assert_eq!(c.breaking_indicators, 0);
}

#[test]
fn contracts_all_types() {
    let files = vec![
        "crates/tokmd-core/src/lib.rs",
        "crates/tokmd/src/commands/export.rs",
        "docs/schema.json",
    ];
    let c = detect_contracts(&files);
    assert!(c.api_changed);
    assert!(c.cli_changed);
    assert!(c.schema_changed);
    assert_eq!(c.breaking_indicators, 2);
}

// =============================================================================
// compute_code_health
// =============================================================================

#[test]
fn code_health_perfect_no_large_files() {
    let stats = vec![make_stat("src/a.rs", 10, 5)];
    let h = compute_code_health(&stats, &no_contracts());
    assert_eq!(h.score, 100);
    assert_eq!(h.grade, "A");
    assert_eq!(h.large_files_touched, 0);
    assert_eq!(h.complexity_indicator, ComplexityIndicator::Low);
    assert!(h.warnings.is_empty());
}

#[test]
fn code_health_large_file_penalty() {
    let stats = vec![make_stat("src/huge.rs", 300, 250)]; // 550 > 500
    let h = compute_code_health(&stats, &no_contracts());
    assert!(h.score < 100);
    assert_eq!(h.large_files_touched, 1);
    assert_eq!(h.complexity_indicator, ComplexityIndicator::Medium);
    assert!(!h.warnings.is_empty());
    assert_eq!(h.warnings[0].warning_type, WarningType::LargeFile);
}

#[test]
fn code_health_breaking_changes_reduce_score() {
    let stats = vec![make_stat("src/lib.rs", 5, 3)];
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 1,
    };
    let h = compute_code_health(&stats, &contracts);
    assert_eq!(h.score, 80); // 100 - 20 for breaking
    assert_eq!(h.grade, "B");
}

#[test]
fn code_health_critical_complexity() {
    // >5 large files → Critical
    let stats: Vec<_> = (0..6)
        .map(|i| make_stat(&format!("src/f{i}.rs"), 300, 250))
        .collect();
    let h = compute_code_health(&stats, &no_contracts());
    assert_eq!(h.complexity_indicator, ComplexityIndicator::Critical);
    assert!(h.score < 50);
}

// =============================================================================
// compute_risk
// =============================================================================

#[test]
fn risk_low_for_small_changes() {
    let stats = vec![make_stat("src/a.rs", 10, 5)];
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    let r = compute_risk(&stats, &contracts, &health);
    assert_eq!(r.level, RiskLevel::Low);
    assert!(r.hotspots_touched.is_empty());
}

#[test]
fn risk_hotspot_detection() {
    let stats = vec![make_stat("src/core.rs", 200, 150)]; // >300 total
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    let r = compute_risk(&stats, &contracts, &health);
    assert!(!r.hotspots_touched.is_empty());
    assert!(r.hotspots_touched.contains(&"src/core.rs".to_string()));
}

#[test]
fn risk_score_bounded_at_100() {
    // Many large changes
    let stats: Vec<_> = (0..20)
        .map(|i| make_stat(&format!("src/f{i}.rs"), 300, 250))
        .collect();
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    let r = compute_risk(&stats, &contracts, &health);
    assert!(r.score <= 100);
}

// =============================================================================
// generate_review_plan
// =============================================================================

#[test]
fn review_plan_sorted_by_priority() {
    let stats = vec![
        make_stat("src/small.rs", 5, 5),     // 10 lines → priority 3
        make_stat("src/medium.rs", 40, 20),  // 60 lines → priority 2
        make_stat("src/large.rs", 150, 100), // 250 lines → priority 1
    ];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan.len(), 3);
    assert_eq!(plan[0].priority, 1);
    assert_eq!(plan[1].priority, 2);
    assert_eq!(plan[2].priority, 3);
    assert_eq!(plan[0].path, "src/large.rs");
}

#[test]
fn review_plan_complexity_scores() {
    let stats = vec![
        make_stat("src/tiny.rs", 5, 5),     // 10 → complexity 1
        make_stat("src/mid.rs", 60, 50),    // 110 → complexity 3
        make_stat("src/huge.rs", 200, 150), // 350 → complexity 5
    ];
    let plan = generate_review_plan(&stats, &no_contracts());
    // Sort by priority first
    assert_eq!(plan[0].complexity, Some(5)); // huge → priority 1
    assert_eq!(plan[1].complexity, Some(3)); // mid → priority 2
    assert_eq!(plan[2].complexity, Some(1)); // tiny → priority 3
}

#[test]
fn review_plan_empty_input() {
    let stats: Vec<FileStat> = vec![];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert!(plan.is_empty());
}

// =============================================================================
// sparkline
// =============================================================================

#[test]
fn sparkline_empty_returns_empty() {
    assert_eq!(sparkline(&[]), "");
}

#[test]
fn sparkline_single_value() {
    let s = sparkline(&[42.0]);
    assert_eq!(s.chars().count(), 1);
}

#[test]
fn sparkline_ascending_has_low_to_high() {
    let s = sparkline(&[0.0, 50.0, 100.0]);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars[0], '\u{2581}'); // lowest
    assert_eq!(chars[2], '\u{2588}'); // highest
}

#[test]
fn sparkline_constant_values_uniform() {
    let s = sparkline(&[10.0, 10.0, 10.0, 10.0]);
    let chars: Vec<char> = s.chars().collect();
    assert!(chars.iter().all(|&c| c == chars[0]));
}

#[test]
fn sparkline_all_infinity_returns_empty() {
    // All infinite values → min/max both infinite → not finite → empty
    assert_eq!(sparkline(&[f64::INFINITY]), "");
    assert_eq!(sparkline(&[f64::NEG_INFINITY, f64::INFINITY]), "");
}

#[test]
fn sparkline_negative_values() {
    let s = sparkline(&[-100.0, -50.0, 0.0, 50.0, 100.0]);
    assert_eq!(s.chars().count(), 5);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars[0], '\u{2581}');
    assert_eq!(chars[4], '\u{2588}');
}

// =============================================================================
// compute_metric_trend
// =============================================================================

#[test]
fn trend_improving_higher_is_better() {
    let t = compute_metric_trend(90.0, 80.0, true);
    assert_eq!(t.direction, TrendDirection::Improving);
    assert_eq!(t.delta, 10.0);
    assert!(t.delta_pct > 0.0);
}

#[test]
fn trend_degrading_higher_is_better() {
    let t = compute_metric_trend(70.0, 85.0, true);
    assert_eq!(t.direction, TrendDirection::Degrading);
    assert!(t.delta < 0.0);
}

#[test]
fn trend_stable_within_threshold() {
    let t = compute_metric_trend(80.0, 80.5, true);
    assert_eq!(t.direction, TrendDirection::Stable);
}

#[test]
fn trend_improving_lower_is_better() {
    // Risk: lower is better. Going from 50 to 30 is improving.
    let t = compute_metric_trend(30.0, 50.0, false);
    assert_eq!(t.direction, TrendDirection::Improving);
}

#[test]
fn trend_degrading_lower_is_better() {
    let t = compute_metric_trend(60.0, 30.0, false);
    assert_eq!(t.direction, TrendDirection::Degrading);
}

#[test]
fn trend_from_zero_pct() {
    let t = compute_metric_trend(10.0, 0.0, true);
    assert_eq!(t.delta_pct, 100.0);
}

#[test]
fn trend_both_zero() {
    let t = compute_metric_trend(0.0, 0.0, true);
    assert_eq!(t.delta_pct, 0.0);
    assert_eq!(t.direction, TrendDirection::Stable);
}

// =============================================================================
// round_pct
// =============================================================================

#[test]
fn round_pct_basic() {
    assert_eq!(round_pct(0.123456), 0.12);
    assert_eq!(round_pct(0.999), 1.0);
    assert_eq!(round_pct(0.0), 0.0);
}

#[test]
fn round_pct_negative() {
    assert_eq!(round_pct(-0.567), -0.57);
}

// =============================================================================
// format_signed_f64
// =============================================================================

#[test]
#[allow(clippy::approx_constant)]
fn format_signed_positive() {
    assert_eq!(format_signed_f64(3.14), "+3.14");
}

#[test]
fn format_signed_negative() {
    assert_eq!(format_signed_f64(-2.5), "-2.50");
}

#[test]
fn format_signed_zero() {
    assert_eq!(format_signed_f64(0.0), "0.00");
}

// =============================================================================
// trend_direction_label
// =============================================================================

#[test]
fn trend_labels() {
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

// =============================================================================
// CockpitReceipt JSON serialization
// =============================================================================

#[test]
fn cockpit_receipt_json_roundtrip() {
    let receipt = CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 100,
        base_ref: "main".to_string(),
        head_ref: "HEAD".to_string(),
        change_surface: ChangeSurface {
            commits: 1,
            files_changed: 2,
            insertions: 10,
            deletions: 5,
            net_lines: 5,
            churn_velocity: 15.0,
            change_concentration: 0.8,
        },
        composition: Composition {
            code_pct: 1.0,
            test_pct: 0.0,
            docs_pct: 0.0,
            config_pct: 0.0,
            test_ratio: 0.0,
        },
        code_health: CodeHealth {
            score: 100,
            grade: "A".to_string(),
            large_files_touched: 0,
            avg_file_size: 15,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Low,
            score: 0,
        },
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
        evidence: Evidence {
            overall_status: GateStatus::Skipped,
            mutation: make_mutation_gate(GateStatus::Skipped),
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        },
        review_plan: vec![],
        trend: None,
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(back.change_surface.commits, 1);
    assert_eq!(back.code_health.grade, "A");
    // trend should be absent
    assert!(back.trend.is_none());
}

#[test]
fn cockpit_receipt_trend_serializes_when_present() {
    let receipt = CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 200,
        base_ref: "v1".to_string(),
        head_ref: "v2".to_string(),
        change_surface: ChangeSurface {
            commits: 0,
            files_changed: 0,
            insertions: 0,
            deletions: 0,
            net_lines: 0,
            churn_velocity: 0.0,
            change_concentration: 0.0,
        },
        composition: Composition {
            code_pct: 0.0,
            test_pct: 0.0,
            docs_pct: 0.0,
            config_pct: 0.0,
            test_ratio: 0.0,
        },
        code_health: CodeHealth {
            score: 50,
            grade: "D".to_string(),
            large_files_touched: 0,
            avg_file_size: 0,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Low,
            score: 0,
        },
        contracts: no_contracts(),
        evidence: Evidence {
            overall_status: GateStatus::Skipped,
            mutation: make_mutation_gate(GateStatus::Skipped),
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        },
        review_plan: vec![],
        trend: Some(TrendComparison {
            baseline_available: true,
            baseline_path: Some("b.json".to_string()),
            baseline_generated_at_ms: Some(100),
            health: Some(TrendMetric {
                current: 50.0,
                previous: 70.0,
                delta: -20.0,
                delta_pct: -28.57,
                direction: TrendDirection::Degrading,
            }),
            risk: None,
            complexity: None,
        }),
    };

    let json = serde_json::to_string(&receipt).unwrap();
    assert!(json.contains("\"trend\""));
    let back: CockpitReceipt = serde_json::from_str(&json).unwrap();
    let trend = back.trend.unwrap();
    assert!(trend.baseline_available);
    assert_eq!(trend.health.unwrap().direction, TrendDirection::Degrading);
}

// =============================================================================
// now_iso8601 format check
// =============================================================================

#[test]
fn now_iso8601_format() {
    let ts = now_iso8601();
    // Should look like "2024-01-15T12:34:56Z"
    assert!(ts.ends_with('Z'));
    assert_eq!(ts.len(), 20);
    assert!(ts.contains('T'));
}

// =============================================================================
// is_relevant_rust_source (tested via composition indirectly)
// =============================================================================

#[test]
fn filestat_as_ref_returns_path() {
    let s = make_stat("src/foo.rs", 1, 2);
    let r: &str = s.as_ref();
    assert_eq!(r, "src/foo.rs");
}
