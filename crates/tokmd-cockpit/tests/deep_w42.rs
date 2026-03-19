//! Wave 42 deep tests for tokmd-cockpit.
//!
//! Covers:
//! - Composition computation with various file mixes
//! - Contract detection edge cases
//! - Code health scoring and grade boundaries
//! - Risk computation with hotspot thresholds
//! - Review plan generation and ordering
//! - Metric trend computation (higher/lower is better)
//! - Complexity trend indicators
//! - Sparkline rendering edge cases
//! - Utility functions (round_pct, format_signed_f64)
//! - Determinism: same input → same output
//! - Schema version compliance

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

// =========================================================================
// 1. Composition
// =========================================================================

#[test]
fn composition_all_categories() {
    let files = vec![
        "src/main.rs",
        "tests/test_a.rs",
        "docs/guide.md",
        "config.toml",
    ];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 0.25);
    assert_eq!(comp.test_pct, 0.25);
    assert_eq!(comp.docs_pct, 0.25);
    assert_eq!(comp.config_pct, 0.25);
}

#[test]
fn composition_docs_folder_path() {
    let files = vec!["project/docs/internal.txt"];
    let comp = compute_composition(&files);
    assert_eq!(comp.docs_pct, 1.0);
}

#[test]
fn composition_yaml_config() {
    let files = vec!["ci.yml", "config.yaml"];
    let comp = compute_composition(&files);
    assert_eq!(comp.config_pct, 1.0);
}

#[test]
fn composition_js_and_ts_counted_as_code() {
    let files = vec!["index.js", "app.ts"];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 1.0);
    assert_eq!(comp.test_ratio, 0.0);
}

#[test]
fn composition_spec_files_counted_as_test() {
    let files = vec!["app_spec.ts", "helper_spec.js"];
    let comp = compute_composition(&files);
    assert_eq!(comp.test_pct, 1.0);
}

#[test]
fn composition_py_test_files() {
    let files = vec!["test_main.py", "src/app.py"];
    let comp = compute_composition(&files);
    assert!(comp.test_pct > 0.0);
    assert!(comp.code_pct > 0.0);
    assert_eq!(comp.test_ratio, 1.0); // 1 test / 1 code
}

#[test]
fn composition_unrecognized_files_ignored() {
    let files = vec!["image.png", "binary.so", "data.dat"];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(comp.test_pct, 0.0);
    assert_eq!(comp.docs_pct, 0.0);
    assert_eq!(comp.config_pct, 0.0);
    assert_eq!(comp.test_ratio, 0.0);
}

#[test]
fn composition_json_as_config() {
    let files = vec!["package.json"];
    let comp = compute_composition(&files);
    assert_eq!(comp.config_pct, 1.0);
}

// =========================================================================
// 2. Contract detection
// =========================================================================

#[test]
fn contracts_mod_rs_is_api() {
    let files = vec!["crates/tokmd-types/src/mod.rs"];
    let c = detect_contracts(&files);
    assert!(c.api_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_schema_md_detected() {
    let files = vec!["docs/SCHEMA.md"];
    let c = detect_contracts(&files);
    assert!(c.schema_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_config_crate_is_cli() {
    let files = vec!["crates/tokmd-config/src/lib.rs"];
    let c = detect_contracts(&files);
    assert!(c.cli_changed);
    assert!(c.api_changed); // also lib.rs → api_changed
}

#[test]
fn contracts_breaking_indicators_count_api_plus_schema() {
    let files = vec!["src/lib.rs", "docs/schema.json"];
    let c = detect_contracts(&files);
    assert_eq!(c.breaking_indicators, 2);
}

// =========================================================================
// 3. Code health scoring
// =========================================================================

#[test]
fn health_grade_boundaries() {
    let c = no_contracts();

    // Score 100 → A
    let h = compute_code_health(&[stat("a.rs", 10, 5)], &c);
    assert_eq!(h.grade, "A");

    // 3 large files → score = 100 - 30 = 70 → C
    let stats: Vec<_> = (0..3)
        .map(|i| stat(&format!("big{i}.rs"), 300, 300))
        .collect();
    let h = compute_code_health(&stats, &c);
    assert_eq!(h.score, 70);
    assert_eq!(h.grade, "C");
}

#[test]
fn health_complexity_high_with_three_large_files() {
    let c = no_contracts();
    let stats: Vec<_> = (0..3)
        .map(|i| stat(&format!("f{i}.rs"), 300, 300))
        .collect();
    let h = compute_code_health(&stats, &c);
    assert_eq!(h.complexity_indicator, ComplexityIndicator::High);
}

#[test]
fn health_complexity_critical_with_six_large_files() {
    let c = no_contracts();
    let stats: Vec<_> = (0..6)
        .map(|i| stat(&format!("f{i}.rs"), 300, 300))
        .collect();
    let h = compute_code_health(&stats, &c);
    assert_eq!(h.complexity_indicator, ComplexityIndicator::Critical);
}

#[test]
fn health_saturating_sub_does_not_underflow() {
    let c = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: true,
        breaking_indicators: 2,
    };
    // 11 large files → 110 penalty, but saturating_sub from 100 → 0, then -20 → 0
    let stats: Vec<_> = (0..11)
        .map(|i| stat(&format!("f{i}.rs"), 300, 300))
        .collect();
    let h = compute_code_health(&stats, &c);
    assert_eq!(h.score, 0);
    assert_eq!(h.grade, "F");
}

#[test]
fn health_avg_file_size() {
    let c = no_contracts();
    let stats = vec![stat("a.rs", 100, 0), stat("b.rs", 200, 0)];
    let h = compute_code_health(&stats, &c);
    // (100 + 200) / 2 = 150
    assert_eq!(h.avg_file_size, 150);
}

// =========================================================================
// 4. Risk
// =========================================================================

#[test]
fn risk_low_with_small_changes() {
    let c = no_contracts();
    let stats = vec![stat("a.rs", 10, 5)];
    let h = compute_code_health(&stats, &c);
    let r = compute_risk(&stats, &c, &h);
    assert_eq!(r.level, RiskLevel::Low);
    assert!(r.hotspots_touched.is_empty());
}

#[test]
fn risk_hotspots_over_300_lines() {
    let c = no_contracts();
    let stats = vec![stat("big.rs", 200, 200)]; // 400 lines > 300
    let h = compute_code_health(&stats, &c);
    let r = compute_risk(&stats, &c, &h);
    assert_eq!(r.hotspots_touched.len(), 1);
    assert_eq!(r.hotspots_touched[0], "big.rs");
}

#[test]
fn risk_score_capped_at_100() {
    let c = no_contracts();
    let stats: Vec<_> = (0..20)
        .map(|i| stat(&format!("f{i}.rs"), 300, 300))
        .collect();
    let h = compute_code_health(&stats, &c);
    let r = compute_risk(&stats, &c, &h);
    assert!(r.score <= 100);
}

// =========================================================================
// 5. Review plan
// =========================================================================

#[test]
fn review_plan_priority_and_complexity_assignment() {
    let c = no_contracts();
    let stats = vec![
        stat("huge.rs", 250, 250), // 500 lines: priority 1, complexity 5
        stat("mid.rs", 60, 60),    // 120 lines: priority 2, complexity 3
        stat("tiny.rs", 5, 5),     // 10 lines: priority 3, complexity 1
    ];
    let plan = generate_review_plan(&stats, &c);
    assert_eq!(plan.len(), 3);
    assert_eq!(plan[0].priority, 1);
    assert_eq!(plan[0].complexity, Some(5));
    assert_eq!(plan[1].priority, 2);
    assert_eq!(plan[1].complexity, Some(3));
    assert_eq!(plan[2].priority, 3);
    assert_eq!(plan[2].complexity, Some(1));
}

#[test]
fn review_plan_lines_changed_populated() {
    let c = no_contracts();
    let stats = vec![stat("a.rs", 30, 20)];
    let plan = generate_review_plan(&stats, &c);
    assert_eq!(plan[0].lines_changed, Some(50));
}

#[test]
fn review_plan_reason_contains_line_count() {
    let c = no_contracts();
    let stats = vec![stat("a.rs", 10, 5)];
    let plan = generate_review_plan(&stats, &c);
    assert!(plan[0].reason.contains("15 lines changed"));
}

// =========================================================================
// 6. Metric trend
// =========================================================================

#[test]
fn trend_stable_within_one_unit() {
    let t = compute_metric_trend(80.0, 80.5, true);
    assert_eq!(t.direction, TrendDirection::Stable);
}

#[test]
fn trend_lower_is_better_improving() {
    let t = compute_metric_trend(20.0, 40.0, false);
    assert_eq!(t.direction, TrendDirection::Improving);
    assert!(t.delta < 0.0);
}

#[test]
fn trend_lower_is_better_degrading() {
    let t = compute_metric_trend(50.0, 30.0, false);
    assert_eq!(t.direction, TrendDirection::Degrading);
}

// =========================================================================
// 7. Complexity trend
// =========================================================================

#[test]
fn complexity_trend_stable_below_threshold() {
    let current = make_receipt_with_complexity(5.0);
    let baseline = make_receipt_with_complexity(5.2);
    let indicator = compute_complexity_trend(&current, &baseline);
    assert_eq!(indicator.direction, TrendDirection::Stable);
}

#[test]
fn complexity_trend_degrading() {
    let current = make_receipt_with_complexity(10.0);
    let baseline = make_receipt_with_complexity(5.0);
    let indicator = compute_complexity_trend(&current, &baseline);
    assert_eq!(indicator.direction, TrendDirection::Degrading);
    assert!(indicator.summary.contains("increased"));
}

#[test]
fn complexity_trend_improving() {
    let current = make_receipt_with_complexity(3.0);
    let baseline = make_receipt_with_complexity(8.0);
    let indicator = compute_complexity_trend(&current, &baseline);
    assert_eq!(indicator.direction, TrendDirection::Improving);
    assert!(indicator.summary.contains("decreased"));
}

fn make_receipt_with_complexity(avg_cyc: f64) -> CockpitReceipt {
    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 0,
        base_ref: "main".to_string(),
        head_ref: "feature".to_string(),
        change_surface: ChangeSurface {
            commits: 1,
            files_changed: 1,
            insertions: 10,
            deletions: 5,
            net_lines: 5,
            churn_velocity: 15.0,
            change_concentration: 1.0,
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
        contracts: no_contracts(),
        evidence: Evidence {
            overall_status: GateStatus::Pass,
            mutation: MutationGate {
                meta: GateMeta {
                    status: GateStatus::Skipped,
                    source: EvidenceSource::RanLocal,
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
                },
                survivors: vec![],
                killed: 0,
                timeout: 0,
                unviable: 0,
            },
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: Some(ComplexityGate {
                meta: GateMeta {
                    status: GateStatus::Pass,
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
                },
                files_analyzed: 1,
                high_complexity_files: vec![],
                avg_cyclomatic: avg_cyc,
                max_cyclomatic: avg_cyc as u32,
                threshold_exceeded: false,
            }),
        },
        review_plan: vec![],
        trend: None,
    }
}

// =========================================================================
// 8. Sparkline edge cases
// =========================================================================

#[test]
fn sparkline_all_same_values() {
    let s = sparkline(&[5.0, 5.0, 5.0, 5.0]);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars.len(), 4);
    assert!(chars.iter().all(|c| *c == chars[0]));
}

#[test]
fn sparkline_two_values_min_max() {
    let s = sparkline(&[0.0, 100.0]);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars.len(), 2);
    assert_eq!(chars[0], '\u{2581}'); // lowest
    assert_eq!(chars[1], '\u{2588}'); // highest
}

// =========================================================================
// 9. Utility functions
// =========================================================================

#[test]
fn round_pct_precision() {
    assert_eq!(round_pct(0.1234), 0.12);
    assert_eq!(round_pct(0.1250), 0.13);
    assert_eq!(round_pct(1.0), 1.0);
    assert_eq!(round_pct(0.0), 0.0);
}

#[test]
fn format_signed_f64_positive_negative_zero() {
    assert!(format_signed_f64(3.15).starts_with('+'));
    assert!(format_signed_f64(-2.5).starts_with('-'));
    assert_eq!(format_signed_f64(0.0), "0.00");
}

#[test]
fn trend_direction_label_all_variants() {
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
// 10. Schema version compliance
// =========================================================================

#[test]
fn cockpit_schema_version_is_current() {
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3);
}

#[test]
fn complexity_threshold_is_fifteen() {
    assert_eq!(COMPLEXITY_THRESHOLD, 15);
}

// =========================================================================
// 11. Determinism: same input → same output
// =========================================================================

#[test]
fn composition_deterministic() {
    let files = vec!["src/a.rs", "tests/b.rs", "README.md", "Cargo.toml"];
    let a = compute_composition(&files);
    let b = compute_composition(&files);
    assert_eq!(a.code_pct, b.code_pct);
    assert_eq!(a.test_pct, b.test_pct);
    assert_eq!(a.docs_pct, b.docs_pct);
    assert_eq!(a.config_pct, b.config_pct);
    assert_eq!(a.test_ratio, b.test_ratio);
}

#[test]
fn code_health_deterministic() {
    let c = no_contracts();
    let stats = vec![stat("a.rs", 300, 300), stat("b.rs", 10, 5)];
    let h1 = compute_code_health(&stats, &c);
    let h2 = compute_code_health(&stats, &c);
    assert_eq!(h1.score, h2.score);
    assert_eq!(h1.grade, h2.grade);
    assert_eq!(h1.large_files_touched, h2.large_files_touched);
}

#[test]
fn review_plan_deterministic() {
    let c = no_contracts();
    let stats = vec![
        stat("a.rs", 100, 100),
        stat("b.rs", 10, 5),
        stat("c.rs", 50, 50),
    ];
    let p1 = generate_review_plan(&stats, &c);
    let p2 = generate_review_plan(&stats, &c);
    assert_eq!(p1.len(), p2.len());
    for (a, b) in p1.iter().zip(p2.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.priority, b.priority);
        assert_eq!(a.complexity, b.complexity);
    }
}

// =========================================================================
// 12. CockpitReceipt JSON round-trip
// =========================================================================

#[test]
fn cockpit_receipt_json_roundtrip() {
    let receipt = make_receipt_with_complexity(5.0);
    let json = serde_json::to_string(&receipt).unwrap();
    let parsed: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(parsed.mode, "cockpit");
    assert_eq!(parsed.base_ref, "main");
    assert_eq!(parsed.head_ref, "feature");
}

// =========================================================================
// 13. GateStatus serialization
// =========================================================================

#[test]
fn gate_status_json_serde() {
    for (status, expected) in [
        (GateStatus::Pass, "\"pass\""),
        (GateStatus::Warn, "\"warn\""),
        (GateStatus::Fail, "\"fail\""),
        (GateStatus::Skipped, "\"skipped\""),
        (GateStatus::Pending, "\"pending\""),
    ] {
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, expected);
        let parsed: GateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, status);
    }
}

// =========================================================================
// 14. FileStat AsRef<str>
// =========================================================================

#[test]
fn file_stat_as_ref_str() {
    let s = stat("src/main.rs", 10, 5);
    let r: &str = s.as_ref();
    assert_eq!(r, "src/main.rs");
}

// =========================================================================
// 15. now_iso8601 format
// =========================================================================

#[test]
fn now_iso8601_valid_format() {
    let ts = now_iso8601();
    assert!(ts.ends_with('Z'));
    assert!(ts.contains('T'));
    assert_eq!(ts.len(), 20);
    // Verify parseable year
    let year: u32 = ts[..4].parse().unwrap();
    assert!(year >= 2024);
}
