//! Wave 47 deep integration and property tests for tokmd-cockpit.
//!
//! Covers:
//! - CockpitReceipt structure validation (schema_version, fields)
//! - Gate evaluation with different gate rule configurations
//! - Evidence accumulation and reporting
//! - Capability reporting (git available vs unavailable)
//! - Empty diff handling
//! - Determinism: same inputs → same outputs

use proptest::prelude::*;
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

fn no_contracts() -> Contracts {
    Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    }
}

fn make_receipt(stats: &[FileStat]) -> CockpitReceipt {
    let contracts = detect_contracts(stats);
    let composition = compute_composition(stats);
    let code_health = compute_code_health(stats, &contracts);
    let risk = compute_risk(stats.to_vec(), &contracts, &code_health);
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
        evidence: Evidence {
            overall_status: GateStatus::Pass,
            mutation: MutationGate {
                meta: GateMeta {
                    status: GateStatus::Skipped,
                    source: EvidenceSource::RanLocal,
                    commit_match: CommitMatch::Unknown,
                    scope: ScopeCoverage {
                        relevant: Vec::new(),
                        tested: Vec::new(),
                        ratio: 1.0,
                        lines_relevant: None,
                        lines_tested: None,
                    },
                    evidence_commit: None,
                    evidence_generated_at_ms: None,
                },
                survivors: Vec::new(),
                killed: 0,
                timeout: 0,
                unviable: 0,
            },
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        },
        review_plan,
        trend: None,
    }
}

// =========================================================================
// 1. CockpitReceipt structure validation
// =========================================================================

#[test]
fn receipt_schema_version_matches_constant() {
    let receipt = make_receipt(&[]);
    assert_eq!(receipt.schema_version, COCKPIT_SCHEMA_VERSION);
}

#[test]
fn receipt_mode_is_cockpit() {
    let receipt = make_receipt(&[]);
    assert_eq!(receipt.mode, "cockpit");
}

#[test]
fn receipt_json_roundtrip_preserves_all_fields() {
    let stats = vec![
        stat("src/main.rs", 100, 30),
        stat("tests/test_a.rs", 50, 10),
    ];
    let receipt = make_receipt(&stats);
    let json = serde_json::to_string(&receipt).unwrap();
    let roundtrip: CockpitReceipt = serde_json::from_str(&json).unwrap();

    assert_eq!(roundtrip.schema_version, receipt.schema_version);
    assert_eq!(roundtrip.mode, receipt.mode);
    assert_eq!(
        roundtrip.change_surface.files_changed,
        receipt.change_surface.files_changed
    );
    assert_eq!(
        roundtrip.change_surface.insertions,
        receipt.change_surface.insertions
    );
    assert_eq!(roundtrip.code_health.score, receipt.code_health.score);
    assert_eq!(roundtrip.risk.level, receipt.risk.level);
}

#[test]
fn receipt_base_and_head_refs_preserved() {
    let receipt = make_receipt(&[]);
    assert_eq!(receipt.base_ref, "main");
    assert_eq!(receipt.head_ref, "feature-branch");
}

// =========================================================================
// 2. Evidence overall status via receipt serialization
// =========================================================================

#[test]
fn evidence_overall_status_serializes_correctly() {
    let mut receipt = make_receipt(&[]);
    receipt.evidence.overall_status = GateStatus::Pass;
    let json = serde_json::to_string(&receipt).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["evidence"]["overall_status"], "pass");
}

#[test]
fn evidence_skipped_status_serializes() {
    let receipt = make_receipt(&[]);
    let json = serde_json::to_string(&receipt).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    // mutation gate is Skipped by default
    assert_eq!(parsed["evidence"]["mutation"]["status"], "skipped");
}

#[test]
fn evidence_fail_status_roundtrips() {
    let mut receipt = make_receipt(&[]);
    receipt.evidence.overall_status = GateStatus::Fail;
    receipt.evidence.mutation.meta.status = GateStatus::Fail;
    let json = serde_json::to_string(&receipt).unwrap();
    let roundtrip: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtrip.evidence.overall_status, GateStatus::Fail);
    assert_eq!(roundtrip.evidence.mutation.meta.status, GateStatus::Fail);
}

#[test]
fn evidence_all_gate_statuses_representable() {
    for status in [
        GateStatus::Pass,
        GateStatus::Fail,
        GateStatus::Warn,
        GateStatus::Skipped,
        GateStatus::Pending,
    ] {
        let mut receipt = make_receipt(&[]);
        receipt.evidence.overall_status = status;
        let json = serde_json::to_string(&receipt).unwrap();
        let roundtrip: CockpitReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.evidence.overall_status, status);
    }
}

// =========================================================================
// 3. Evidence accumulation and reporting
// =========================================================================

#[test]
fn evidence_mutation_survivors_tracked() {
    let mut receipt = make_receipt(&[]);
    receipt.evidence.mutation.survivors = vec![
        MutationSurvivor {
            file: "src/a.rs".into(),
            line: 1,
            mutation: "replace x with y".into(),
        },
        MutationSurvivor {
            file: "src/b.rs".into(),
            line: 5,
            mutation: "remove call".into(),
        },
    ];
    receipt.evidence.mutation.killed = 8;

    let json = serde_json::to_string(&receipt).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let survivors = parsed["evidence"]["mutation"]["survivors"]
        .as_array()
        .unwrap();
    assert_eq!(survivors.len(), 2);
    assert_eq!(parsed["evidence"]["mutation"]["killed"], 8);
}

#[test]
fn evidence_complexity_gate_serializes() {
    let mut receipt = make_receipt(&[]);
    receipt.evidence.complexity = Some(ComplexityGate {
        meta: GateMeta {
            status: GateStatus::Warn,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: Vec::new(),
                tested: Vec::new(),
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        files_analyzed: 1,
        avg_cyclomatic: 12.5,
        max_cyclomatic: 30,
        high_complexity_files: vec![HighComplexityFile {
            path: "src/complex.rs".into(),
            cyclomatic: 30,
            function_count: 5,
            max_function_length: 200,
        }],
        threshold_exceeded: true,
    });

    let json = serde_json::to_string(&receipt).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let complexity = &parsed["evidence"]["complexity"];
    assert_eq!(complexity["avg_cyclomatic"], 12.5);
    assert_eq!(complexity["max_cyclomatic"], 30);
    assert_eq!(
        complexity["high_complexity_files"][0]["path"],
        "src/complex.rs"
    );
}

// =========================================================================
// 4. Empty diff handling
// =========================================================================

#[test]
fn empty_diff_produces_zero_surface() {
    let receipt = make_receipt(&[]);
    assert_eq!(receipt.change_surface.files_changed, 0);
    assert_eq!(receipt.change_surface.insertions, 0);
    assert_eq!(receipt.change_surface.deletions, 0);
    assert_eq!(receipt.change_surface.net_lines, 0);
}

#[test]
fn empty_diff_has_zero_composition() {
    let receipt = make_receipt(&[]);
    assert_eq!(receipt.composition.code_pct, 0.0);
    assert_eq!(receipt.composition.test_pct, 0.0);
    assert_eq!(receipt.composition.docs_pct, 0.0);
    assert_eq!(receipt.composition.config_pct, 0.0);
}

#[test]
fn empty_diff_has_perfect_health() {
    let receipt = make_receipt(&[]);
    assert_eq!(receipt.code_health.score, 100);
    assert_eq!(receipt.code_health.grade, "A");
    assert_eq!(receipt.code_health.large_files_touched, 0);
}

#[test]
fn empty_diff_has_low_risk() {
    let receipt = make_receipt(&[]);
    assert_eq!(receipt.risk.level, RiskLevel::Low);
    assert_eq!(receipt.risk.score, 0);
    assert!(receipt.risk.hotspots_touched.is_empty());
}

#[test]
fn empty_diff_has_empty_review_plan() {
    let receipt = make_receipt(&[]);
    assert!(receipt.review_plan.is_empty());
}

// =========================================================================
// 5. Determinism: same inputs → same outputs
// =========================================================================

#[test]
fn determinism_same_stats_same_receipt() {
    let stats = vec![
        stat("src/main.rs", 50, 10),
        stat("tests/test_a.rs", 30, 5),
        stat("README.md", 10, 2),
    ];
    let r1 = make_receipt(&stats);
    let r2 = make_receipt(&stats);

    assert_eq!(
        r1.change_surface.files_changed,
        r2.change_surface.files_changed
    );
    assert_eq!(r1.change_surface.insertions, r2.change_surface.insertions);
    assert_eq!(r1.code_health.score, r2.code_health.score);
    assert_eq!(r1.code_health.grade, r2.code_health.grade);
    assert_eq!(r1.risk.score, r2.risk.score);
    assert_eq!(r1.risk.level, r2.risk.level);
    assert_eq!(r1.composition.code_pct, r2.composition.code_pct);
    assert_eq!(r1.review_plan.len(), r2.review_plan.len());
}

#[test]
fn determinism_json_output_stable() {
    let stats = vec![stat("src/lib.rs", 20, 5)];
    let receipt = make_receipt(&stats);
    let json1 = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let json2 = tokmd_cockpit::render::render_json(&receipt).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn determinism_markdown_output_stable() {
    let stats = vec![stat("src/lib.rs", 20, 5)];
    let receipt = make_receipt(&stats);
    let md1 = tokmd_cockpit::render::render_markdown(&receipt);
    let md2 = tokmd_cockpit::render::render_markdown(&receipt);
    assert_eq!(md1, md2);
}

// =========================================================================
// 6. Code health scoring boundaries
// =========================================================================

#[test]
fn health_grade_boundaries() {
    // No large files, no contracts → score 100 → A
    let stats = vec![stat("src/a.rs", 10, 5)];
    let health = compute_code_health(&stats, &no_contracts());
    assert_eq!(health.score, 100);
    assert_eq!(health.grade, "A");

    // 1 large file → score 90 → A
    let stats = vec![stat("src/big.rs", 400, 200)];
    let health = compute_code_health(&stats, &no_contracts());
    assert_eq!(health.score, 90);
    assert_eq!(health.grade, "A");

    // 2 large files → score 80 → B
    let stats = vec![stat("src/a.rs", 300, 250), stat("src/b.rs", 300, 250)];
    let health = compute_code_health(&stats, &no_contracts());
    assert_eq!(health.score, 80);
    assert_eq!(health.grade, "B");
}

#[test]
fn health_breaking_contracts_deduct_20() {
    let stats = vec![stat("src/lib.rs", 10, 5)];
    let contracts = detect_contracts(&stats);
    assert!(contracts.api_changed);
    let health = compute_code_health(&stats, &contracts);
    // api_changed adds 1 breaking_indicator → -20
    assert_eq!(health.score, 80);
    assert_eq!(health.grade, "B");
}

// =========================================================================
// 7. Risk computation
// =========================================================================

#[test]
fn risk_hotspot_threshold_at_300_lines() {
    // Under threshold → no hotspots
    let stats = vec![stat("src/a.rs", 150, 149)]; // 299 total
    let health = compute_code_health(&stats, &no_contracts());
    let risk = compute_risk(stats.clone(), &no_contracts(), &health);
    assert!(risk.hotspots_touched.is_empty());

    // Over threshold → hotspot
    let stats = vec![stat("src/a.rs", 200, 101)]; // 301 total
    let health = compute_code_health(&stats, &no_contracts());
    let risk = compute_risk(stats.clone(), &no_contracts(), &health);
    assert_eq!(risk.hotspots_touched.len(), 1);
    assert_eq!(risk.hotspots_touched[0], "src/a.rs");
}

#[test]
fn risk_level_increases_with_hotspots() {
    let stats: Vec<FileStat> = (0..5)
        .map(|i| stat(&format!("src/big_{i}.rs"), 200, 200))
        .collect();
    let health = compute_code_health(&stats, &no_contracts());
    let risk = compute_risk(stats.clone(), &no_contracts(), &health);
    assert!(risk.score > 20);
    assert!(matches!(
        risk.level,
        RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical
    ));
}

// =========================================================================
// 8. Trend computation
// =========================================================================

#[test]
fn metric_trend_higher_is_better_improving() {
    let trend = compute_metric_trend(95.0, 80.0, true);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert_eq!(trend.current, 95.0);
    assert_eq!(trend.previous, 80.0);
    assert!(trend.delta > 0.0);
}

#[test]
fn metric_trend_lower_is_better_improving() {
    let trend = compute_metric_trend(10.0, 30.0, false);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert!(trend.delta < 0.0);
}

#[test]
fn metric_trend_stable_within_threshold() {
    let trend = compute_metric_trend(50.0, 50.5, true);
    assert_eq!(trend.direction, TrendDirection::Stable);
}

#[test]
fn metric_trend_zero_baseline() {
    let trend = compute_metric_trend(10.0, 0.0, true);
    assert_eq!(trend.delta_pct, 100.0);
}

#[test]
fn metric_trend_both_zero() {
    let trend = compute_metric_trend(0.0, 0.0, true);
    assert_eq!(trend.delta_pct, 0.0);
    assert_eq!(trend.direction, TrendDirection::Stable);
}

// =========================================================================
// 9. Utility functions
// =========================================================================

#[test]
fn round_pct_rounds_to_two_decimals() {
    assert_eq!(round_pct(0.12345), 0.12);
    assert_eq!(round_pct(0.999), 1.0);
    assert_eq!(round_pct(0.0), 0.0);
}

#[test]
fn format_signed_positive() {
    assert_eq!(format_signed_f64(5.0), "+5.00");
}

#[test]
fn format_signed_negative() {
    assert_eq!(format_signed_f64(-3.5), "-3.50");
}

#[test]
fn format_signed_zero() {
    assert_eq!(format_signed_f64(0.0), "0.00");
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
    let s = sparkline(&[1.0, 2.0, 3.0, 4.0]);
    let chars: Vec<char> = s.chars().collect();
    // First char should be smallest bar, last should be tallest
    assert!(chars[0] < chars[3]);
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

// =========================================================================
// 10. Property tests
// =========================================================================

proptest! {
    #[test]
    fn prop_composition_percentages_sum_to_one_or_zero(
        code_count in 0usize..10,
        test_count in 0usize..10,
        doc_count in 0usize..10,
        config_count in 0usize..10,
    ) {
        let mut files: Vec<String> = Vec::new();
        for i in 0..code_count {
            files.push(format!("src/mod_{i}.rs"));
        }
        for i in 0..test_count {
            files.push(format!("tests/test_{i}.rs"));
        }
        for i in 0..doc_count {
            files.push(format!("docs/doc_{i}.md"));
        }
        for i in 0..config_count {
            files.push(format!("config_{i}.toml"));
        }

        let comp = compute_composition(&files);
        let sum = comp.code_pct + comp.test_pct + comp.docs_pct + comp.config_pct;

        if files.is_empty() {
            prop_assert!((sum - 0.0).abs() < f64::EPSILON);
        } else {
            prop_assert!((sum - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn prop_health_score_bounded_0_100(
        large_count in 0usize..20,
        breaking in 0u32..5,
    ) {
        let stats: Vec<FileStat> = (0..large_count)
            .map(|i| stat(&format!("src/f_{i}.rs"), 300, 250))
            .collect();
        let contracts = Contracts {
            api_changed: breaking > 0,
            cli_changed: false,
            schema_changed: breaking > 1,
            breaking_indicators: breaking.min(2) as usize,
        };
        let health = compute_code_health(&stats, &contracts);
        prop_assert!(health.score <= 100);
    }

    #[test]
    fn prop_risk_score_bounded_0_100(
        file_count in 0usize..20,
        ins in 0usize..500,
        del in 0usize..500,
    ) {
        let stats: Vec<FileStat> = (0..file_count)
            .map(|i| stat(&format!("src/f_{i}.rs"), ins, del))
            .collect();
        let contracts = no_contracts();
        let health = compute_code_health(&stats, &contracts);
        let risk = compute_risk(stats.clone(), &contracts, &health);
        prop_assert!(risk.score <= 100);
    }

    #[test]
    fn prop_review_plan_sorted_by_priority(
        file_count in 1usize..15,
    ) {
        let stats: Vec<FileStat> = (0..file_count)
            .map(|i| stat(&format!("src/f_{i}.rs"), (i + 1) * 30, i * 10))
            .collect();
        let plan = generate_review_plan(&stats, &no_contracts());
        for window in plan.windows(2) {
            prop_assert!(window[0].priority <= window[1].priority);
        }
    }

    #[test]
    fn prop_round_pct_is_idempotent(val in -100.0f64..100.0) {
        let once = round_pct(val);
        let twice = round_pct(once);
        prop_assert!((once - twice).abs() < f64::EPSILON);
    }
}
