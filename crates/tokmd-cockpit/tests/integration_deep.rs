//! Deep integration tests for tokmd-cockpit.
//!
//! 72 tests across 12 categories: composition, contracts, code health, risk,
//! review plan, trend computation, determinism hashing, rendering pipeline,
//! utility helpers, end-to-end workflows, and edge cases.

use tokmd_cockpit::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_file_stat(path: &str, insertions: usize, deletions: usize) -> FileStat {
    FileStat {
        path: path.to_string(),
        insertions,
        deletions,
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

fn make_no_contract() -> Contracts {
    Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    }
}

// ===========================================================================
// 1. Composition
// ===========================================================================

#[test]
fn composition_empty_files() {
    let comp = compute_composition::<String>(&[]);
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(comp.test_pct, 0.0);
    assert_eq!(comp.docs_pct, 0.0);
    assert_eq!(comp.config_pct, 0.0);
    assert_eq!(comp.test_ratio, 0.0);
}

#[test]
fn composition_only_code() {
    let files = ["src/main.rs", "src/lib.rs"];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 1.0);
    assert_eq!(comp.test_pct, 0.0);
    assert_eq!(comp.test_ratio, 0.0);
}

#[test]
fn composition_only_tests() {
    let files = ["tests/unit_test.rs", "src/test_helper.rs"];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(comp.test_pct, 1.0);
    assert_eq!(comp.test_ratio, 1.0);
}

#[test]
fn composition_mixed_types() {
    let files = ["src/main.rs", "tests/test_it.rs", "README.md", "Cargo.toml"];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 0.25);
    assert_eq!(comp.test_pct, 0.25);
    assert_eq!(comp.docs_pct, 0.25);
    assert_eq!(comp.config_pct, 0.25);
    assert_eq!(comp.test_ratio, 1.0);
}

#[test]
fn composition_docs_directory() {
    let files = ["docs/tutorial.md", "docs/guide/intro.txt"];
    let comp = compute_composition(&files);
    assert!(comp.docs_pct > 0.0);
}

#[test]
fn composition_config_extensions() {
    let files = ["config.yml", "settings.yaml", "data.json", "Cargo.toml"];
    let comp = compute_composition(&files);
    assert_eq!(comp.config_pct, 1.0);
}

#[test]
fn composition_spec_files_are_tests() {
    let files = ["src/main_spec.js"];
    let comp = compute_composition(&files);
    assert_eq!(comp.test_pct, 1.0);
    assert_eq!(comp.code_pct, 0.0);
}

#[test]
fn composition_python_and_typescript() {
    let files = ["app.py", "app.ts", "test_app.py", "app.test.ts"];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 0.5);
    assert_eq!(comp.test_pct, 0.5);
    assert_eq!(comp.test_ratio, 1.0);
}

#[test]
fn composition_unrecognized_extensions_ignored() {
    let files = ["image.png", "data.bin", "src/main.rs"];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 1.0);
}

// ===========================================================================
// 2. Contracts
// ===========================================================================

#[test]
fn contracts_no_changes() {
    let files: Vec<String> = vec!["src/utils.rs".to_string()];
    let contracts = detect_contracts(&files);
    assert!(!contracts.api_changed);
    assert!(!contracts.cli_changed);
    assert!(!contracts.schema_changed);
    assert_eq!(contracts.breaking_indicators, 0);
}

#[test]
fn contracts_api_change_lib_rs() {
    let files = ["src/lib.rs"];
    let contracts = detect_contracts(&files);
    assert!(contracts.api_changed);
    assert_eq!(contracts.breaking_indicators, 1);
}

#[test]
fn contracts_api_change_mod_rs() {
    let files = ["src/module/mod.rs"];
    let contracts = detect_contracts(&files);
    assert!(contracts.api_changed);
}

#[test]
fn contracts_cli_change_commands() {
    let files = ["crates/tokmd/src/commands/new_cmd.rs"];
    let contracts = detect_contracts(&files);
    assert!(contracts.cli_changed);
    assert!(!contracts.api_changed);
}

#[test]
fn contracts_cli_change_config() {
    let files = ["crates/tokmd-config/src/lib.rs"];
    let contracts = detect_contracts(&files);
    assert!(contracts.cli_changed);
    assert!(contracts.api_changed);
}

#[test]
fn contracts_schema_change() {
    let files = ["docs/schema.json"];
    let contracts = detect_contracts(&files);
    assert!(contracts.schema_changed);
    assert_eq!(contracts.breaking_indicators, 1);
}

#[test]
fn contracts_schema_md_change() {
    let files = ["docs/SCHEMA.md"];
    let contracts = detect_contracts(&files);
    assert!(contracts.schema_changed);
}

#[test]
fn contracts_all_changes() {
    let files = [
        "src/lib.rs",
        "crates/tokmd/src/commands/run.rs",
        "docs/schema.json",
    ];
    let contracts = detect_contracts(&files);
    assert!(contracts.api_changed);
    assert!(contracts.cli_changed);
    assert!(contracts.schema_changed);
    assert_eq!(contracts.breaking_indicators, 2);
}

// ===========================================================================
// 3. Code Health
// ===========================================================================

#[test]
fn health_no_files() {
    let health = compute_code_health(&[], &make_no_contract());
    assert_eq!(health.score, 100);
    assert_eq!(health.grade, "A");
    assert_eq!(health.large_files_touched, 0);
    assert_eq!(health.avg_file_size, 0);
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Low);
    assert!(health.warnings.is_empty());
}

#[test]
fn health_small_files_perfect_score() {
    let stats = vec![
        make_file_stat("a.rs", 50, 10),
        make_file_stat("b.rs", 30, 20),
    ];
    let health = compute_code_health(&stats, &make_no_contract());
    assert_eq!(health.score, 100);
    assert_eq!(health.grade, "A");
    assert_eq!(health.large_files_touched, 0);
}

#[test]
fn health_one_large_file_penalty() {
    let stats = vec![make_file_stat("big.rs", 300, 250)];
    let health = compute_code_health(&stats, &make_no_contract());
    assert_eq!(health.large_files_touched, 1);
    assert_eq!(health.score, 90);
    assert_eq!(health.grade, "A");
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Medium);
}

#[test]
fn health_two_large_files() {
    let stats = vec![
        make_file_stat("a.rs", 300, 250),
        make_file_stat("b.rs", 400, 200),
    ];
    let health = compute_code_health(&stats, &make_no_contract());
    assert_eq!(health.large_files_touched, 2);
    assert_eq!(health.score, 80);
    assert_eq!(health.grade, "B");
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Medium);
}

#[test]
fn health_three_large_files_high_complexity() {
    let stats: Vec<FileStat> = (0..3)
        .map(|i| make_file_stat(&format!("f{i}.rs"), 300, 250))
        .collect();
    let health = compute_code_health(&stats, &make_no_contract());
    assert_eq!(health.large_files_touched, 3);
    assert_eq!(health.complexity_indicator, ComplexityIndicator::High);
}

#[test]
fn health_six_large_files_critical() {
    let stats: Vec<FileStat> = (0..6)
        .map(|i| make_file_stat(&format!("f{i}.rs"), 300, 250))
        .collect();
    let health = compute_code_health(&stats, &make_no_contract());
    assert_eq!(health.large_files_touched, 6);
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Critical);
}

#[test]
fn health_breaking_contract_penalty() {
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 1,
    };
    let health = compute_code_health(&[], &contracts);
    assert_eq!(health.score, 80);
    assert_eq!(health.grade, "B");
}

#[test]
fn health_breaking_plus_large_file() {
    let stats = vec![make_file_stat("big.rs", 300, 250)];
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: true,
        breaking_indicators: 2,
    };
    let health = compute_code_health(&stats, &contracts);
    assert_eq!(health.score, 70);
    assert_eq!(health.grade, "C");
}

#[test]
fn health_grade_boundary_d() {
    let stats: Vec<FileStat> = (0..2)
        .map(|i| make_file_stat(&format!("f{i}.rs"), 300, 250))
        .collect();
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 1,
    };
    let health = compute_code_health(&stats, &contracts);
    assert_eq!(health.score, 60);
    assert_eq!(health.grade, "D");
}

#[test]
fn health_grade_f() {
    let stats: Vec<FileStat> = (0..4)
        .map(|i| make_file_stat(&format!("f{i}.rs"), 300, 250))
        .collect();
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: true,
        breaking_indicators: 2,
    };
    let health = compute_code_health(&stats, &contracts);
    assert_eq!(health.score, 40);
    assert_eq!(health.grade, "F");
}

#[test]
fn health_score_saturates_at_zero() {
    let stats: Vec<FileStat> = (0..12)
        .map(|i| make_file_stat(&format!("f{i}.rs"), 300, 250))
        .collect();
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: true,
        breaking_indicators: 2,
    };
    let health = compute_code_health(&stats, &contracts);
    assert_eq!(health.score, 0);
    assert_eq!(health.grade, "F");
}

#[test]
fn health_warnings_for_large_files() {
    let stats = vec![
        make_file_stat("big.rs", 300, 250),
        make_file_stat("small.rs", 10, 5),
        make_file_stat("huge.rs", 1000, 500),
    ];
    let health = compute_code_health(&stats, &make_no_contract());
    assert_eq!(health.warnings.len(), 2);
    assert!(
        health
            .warnings
            .iter()
            .all(|w| w.warning_type == WarningType::LargeFile)
    );
}

#[test]
fn health_avg_file_size() {
    let stats = vec![
        make_file_stat("a.rs", 100, 50),
        make_file_stat("b.rs", 200, 100),
    ];
    let health = compute_code_health(&stats, &make_no_contract());
    assert_eq!(health.avg_file_size, 225);
}

// ===========================================================================
// 4. Risk
// ===========================================================================

#[test]
fn risk_no_files() {
    let health = compute_code_health(&[], &make_no_contract());
    let risk = compute_risk(&[], &make_no_contract(), &health);
    assert_eq!(risk.score, 0);
    assert_eq!(risk.level, RiskLevel::Low);
    assert!(risk.hotspots_touched.is_empty());
}

#[test]
fn risk_small_files_low() {
    let stats = vec![make_file_stat("a.rs", 50, 10)];
    let health = compute_code_health(&stats, &make_no_contract());
    let risk = compute_risk(&stats, &make_no_contract(), &health);
    assert_eq!(risk.level, RiskLevel::Low);
    assert!(risk.hotspots_touched.is_empty());
}

#[test]
fn risk_hotspot_detection() {
    let stats = vec![make_file_stat("hot.rs", 200, 150)];
    let health = compute_code_health(&stats, &make_no_contract());
    let risk = compute_risk(&stats, &make_no_contract(), &health);
    assert_eq!(risk.hotspots_touched.len(), 1);
    assert_eq!(risk.hotspots_touched[0], "hot.rs");
}

#[test]
fn risk_score_formula() {
    let stats = vec![make_file_stat("hot.rs", 200, 150)];
    let health = compute_code_health(&stats, &make_no_contract());
    let risk = compute_risk(&stats, &make_no_contract(), &health);
    assert_eq!(risk.score, 15);
    assert_eq!(risk.level, RiskLevel::Low);
}

#[test]
fn risk_medium_level() {
    let stats = vec![
        make_file_stat("a.rs", 200, 150),
        make_file_stat("b.rs", 200, 150),
    ];
    let health = compute_code_health(&stats, &make_no_contract());
    let risk = compute_risk(&stats, &make_no_contract(), &health);
    assert_eq!(risk.score, 30);
    assert_eq!(risk.level, RiskLevel::Medium);
}

#[test]
fn risk_high_level() {
    let stats: Vec<FileStat> = (0..4)
        .map(|i| make_file_stat(&format!("f{i}.rs"), 200, 150))
        .collect();
    let health = compute_code_health(&stats, &make_no_contract());
    let risk = compute_risk(&stats, &make_no_contract(), &health);
    assert_eq!(risk.score, 60);
    assert_eq!(risk.level, RiskLevel::High);
}

#[test]
fn risk_critical_level() {
    let stats: Vec<FileStat> = (0..6)
        .map(|i| make_file_stat(&format!("f{i}.rs"), 300, 250))
        .collect();
    let health = compute_code_health(&stats, &make_no_contract());
    let risk = compute_risk(&stats, &make_no_contract(), &health);
    assert!(risk.score > 80);
    assert_eq!(risk.level, RiskLevel::Critical);
}

#[test]
fn risk_score_capped_at_100() {
    let stats: Vec<FileStat> = (0..10)
        .map(|i| make_file_stat(&format!("f{i}.rs"), 300, 250))
        .collect();
    let health = compute_code_health(&stats, &make_no_contract());
    let risk = compute_risk(&stats, &make_no_contract(), &health);
    assert_eq!(risk.score, 100);
}

// ===========================================================================
// 5. Review Plan
// ===========================================================================

#[test]
fn review_plan_empty() {
    let plan = generate_review_plan(&[], &make_no_contract());
    assert!(plan.is_empty());
}

#[test]
fn review_plan_priority_p1() {
    let stats = vec![make_file_stat("big.rs", 150, 60)];
    let plan = generate_review_plan(&stats, &make_no_contract());
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].priority, 1);
    assert_eq!(plan[0].lines_changed, Some(210));
}

#[test]
fn review_plan_priority_p2() {
    let stats = vec![make_file_stat("mid.rs", 40, 20)];
    let plan = generate_review_plan(&stats, &make_no_contract());
    assert_eq!(plan[0].priority, 2);
}

#[test]
fn review_plan_priority_p3() {
    let stats = vec![make_file_stat("small.rs", 20, 10)];
    let plan = generate_review_plan(&stats, &make_no_contract());
    assert_eq!(plan[0].priority, 3);
}

#[test]
fn review_plan_complexity_scores() {
    let stats = vec![
        make_file_stat("huge.rs", 200, 150),
        make_file_stat("mid.rs", 80, 40),
        make_file_stat("small.rs", 20, 10),
    ];
    let plan = generate_review_plan(&stats, &make_no_contract());
    assert_eq!(plan.len(), 3);
    let huge = plan.iter().find(|i| i.path == "huge.rs").unwrap();
    let mid = plan.iter().find(|i| i.path == "mid.rs").unwrap();
    let small = plan.iter().find(|i| i.path == "small.rs").unwrap();
    assert_eq!(huge.complexity, Some(5));
    assert_eq!(mid.complexity, Some(3));
    assert_eq!(small.complexity, Some(1));
}

#[test]
fn review_plan_sorted_by_priority() {
    let stats = vec![
        make_file_stat("small.rs", 10, 5),
        make_file_stat("big.rs", 200, 100),
        make_file_stat("mid.rs", 40, 20),
    ];
    let plan = generate_review_plan(&stats, &make_no_contract());
    for window in plan.windows(2) {
        assert!(window[0].priority <= window[1].priority);
    }
}

#[test]
fn review_plan_reason_contains_lines() {
    let stats = vec![make_file_stat("a.rs", 42, 13)];
    let plan = generate_review_plan(&stats, &make_no_contract());
    assert!(plan[0].reason.contains("55"));
}

#[test]
fn review_plan_boundary_200_lines() {
    let stats = vec![make_file_stat("edge.rs", 150, 50)];
    let plan = generate_review_plan(&stats, &make_no_contract());
    assert_eq!(plan[0].priority, 2);
}

#[test]
fn review_plan_boundary_201_lines() {
    let stats = vec![make_file_stat("edge.rs", 151, 50)];
    let plan = generate_review_plan(&stats, &make_no_contract());
    assert_eq!(plan[0].priority, 1);
}

#[test]
fn review_plan_boundary_50_lines() {
    let stats = vec![make_file_stat("edge.rs", 30, 20)];
    let plan = generate_review_plan(&stats, &make_no_contract());
    assert_eq!(plan[0].priority, 3);
}

#[test]
fn review_plan_boundary_51_lines() {
    let stats = vec![make_file_stat("edge.rs", 31, 20)];
    let plan = generate_review_plan(&stats, &make_no_contract());
    assert_eq!(plan[0].priority, 2);
}

// ===========================================================================
// 6. Trend Computation
// ===========================================================================

#[test]
fn trend_metric_improving_higher_is_better() {
    let t = compute_metric_trend(90.0, 70.0, true);
    assert_eq!(t.direction, TrendDirection::Improving);
    assert_eq!(t.current, 90.0);
    assert_eq!(t.previous, 70.0);
    assert!((t.delta - 20.0).abs() < f64::EPSILON);
}

#[test]
fn trend_metric_degrading_higher_is_better() {
    let t = compute_metric_trend(60.0, 80.0, true);
    assert_eq!(t.direction, TrendDirection::Degrading);
}

#[test]
fn trend_metric_improving_lower_is_better() {
    let t = compute_metric_trend(10.0, 30.0, false);
    assert_eq!(t.direction, TrendDirection::Improving);
}

#[test]
fn trend_metric_degrading_lower_is_better() {
    let t = compute_metric_trend(50.0, 20.0, false);
    assert_eq!(t.direction, TrendDirection::Degrading);
}

#[test]
fn trend_metric_stable_small_delta() {
    let t = compute_metric_trend(50.0, 50.5, true);
    assert_eq!(t.direction, TrendDirection::Stable);
}

#[test]
fn trend_metric_stable_exact_equal() {
    let t = compute_metric_trend(42.0, 42.0, true);
    assert_eq!(t.direction, TrendDirection::Stable);
    assert_eq!(t.delta, 0.0);
    assert_eq!(t.delta_pct, 0.0);
}

#[test]
fn trend_metric_pct_from_zero() {
    let t = compute_metric_trend(10.0, 0.0, true);
    assert_eq!(t.delta_pct, 100.0);
}

#[test]
fn trend_metric_both_zero() {
    let t = compute_metric_trend(0.0, 0.0, true);
    assert_eq!(t.direction, TrendDirection::Stable);
    assert_eq!(t.delta_pct, 0.0);
}

#[test]
fn complexity_trend_stable() {
    let r1 = make_receipt(&[]);
    let r2 = make_receipt(&[]);
    let trend = compute_complexity_trend(&r1, &r2);
    assert_eq!(trend.direction, TrendDirection::Stable);
    assert!(trend.summary.contains("stable"));
}

#[test]
fn complexity_trend_with_complexity_gate() {
    let mut current = make_receipt(&[]);
    let mut baseline = make_receipt(&[]);
    current.evidence.complexity = Some(ComplexityGate {
        meta: GateMeta {
            status: GateStatus::Pass,
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
        },
        files_analyzed: 5,
        high_complexity_files: Vec::new(),
        avg_cyclomatic: 8.0,
        max_cyclomatic: 15,
        threshold_exceeded: false,
    });
    baseline.evidence.complexity = Some(ComplexityGate {
        meta: GateMeta {
            status: GateStatus::Pass,
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
        },
        files_analyzed: 5,
        high_complexity_files: Vec::new(),
        avg_cyclomatic: 12.0,
        max_cyclomatic: 20,
        threshold_exceeded: false,
    });
    let trend = compute_complexity_trend(&current, &baseline);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert!(trend.summary.contains("decreased"));
}

// ===========================================================================
// 7. Trend Loading
// ===========================================================================

#[test]
fn trend_load_missing_baseline() {
    let current = make_receipt(&[]);
    let result =
        load_and_compute_trend(std::path::Path::new("/nonexistent/baseline.json"), &current)
            .unwrap();
    assert!(!result.baseline_available);
    assert!(result.baseline_path.is_some());
}

#[test]
fn trend_load_invalid_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    std::fs::write(&path, "not valid json").unwrap();
    let current = make_receipt(&[]);
    let result = load_and_compute_trend(&path, &current).unwrap();
    assert!(!result.baseline_available);
}

#[test]
fn trend_load_valid_baseline() {
    let dir = tempfile::tempdir().unwrap();
    let baseline_path = dir.path().join("baseline.json");
    let mut baseline = make_receipt(&[]);
    baseline.code_health.score = 75;
    baseline.risk.score = 40;
    let json = serde_json::to_string_pretty(&baseline).unwrap();
    std::fs::write(&baseline_path, &json).unwrap();

    let mut current = make_receipt(&[]);
    current.code_health.score = 95;
    current.risk.score = 10;
    let result = load_and_compute_trend(&baseline_path, &current).unwrap();
    assert!(result.baseline_available);
    let health = result.health.unwrap();
    assert_eq!(health.direction, TrendDirection::Improving);
    assert_eq!(health.current, 95.0);
    assert_eq!(health.previous, 75.0);
    let risk = result.risk.unwrap();
    assert_eq!(risk.direction, TrendDirection::Improving);
    assert!(result.complexity.is_some());
}

// ===========================================================================
// 8. Determinism Hashing
// ===========================================================================

#[test]
fn hash_order_independent() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn a() {}").unwrap();
    std::fs::write(dir.path().join("b.rs"), "fn b() {}").unwrap();
    let h1 =
        tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs", "b.rs"]).unwrap();
    let h2 =
        tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["b.rs", "a.rs"]).unwrap();
    assert_eq!(h1, h2);
}

#[test]
fn hash_dedup() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn a() {}").unwrap();
    let h1 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();
    let h2 =
        tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs", "a.rs"]).unwrap();
    assert_eq!(h1, h2);
}

#[test]
fn hash_changes_on_modification() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn a() {}").unwrap();
    let h1 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn a() { panic!() }").unwrap();
    let h2 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn hash_missing_file_skipped() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn a() {}").unwrap();
    let h1 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();
    let h2 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs", "missing.rs"])
        .unwrap();
    assert_eq!(h1, h2);
}

#[test]
fn hash_cargo_lock_present() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("Cargo.lock"),
        "[[package]]\nname = \"test\"",
    )
    .unwrap();
    let result = tokmd_cockpit::determinism::hash_cargo_lock(dir.path()).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().len(), 64);
}

#[test]
fn hash_cargo_lock_absent() {
    let dir = tempfile::tempdir().unwrap();
    let result = tokmd_cockpit::determinism::hash_cargo_lock(dir.path()).unwrap();
    assert!(result.is_none());
}

#[test]
fn hash_hex_length_64() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("x.rs"), "fn x() {}").unwrap();
    let h = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["x.rs"]).unwrap();
    assert_eq!(h.len(), 64);
}

#[test]
fn hash_skips_tokmd_dir() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".tokmd")).unwrap();
    std::fs::write(dir.path().join(".tokmd/baseline.json"), "{}").unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn a() {}").unwrap();
    let h1 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();
    let h2 = tokmd_cockpit::determinism::hash_files_from_paths(
        dir.path(),
        &["a.rs", ".tokmd/baseline.json"],
    )
    .unwrap();
    assert_eq!(h1, h2);
}

// ===========================================================================
// 9. Rendering Pipeline
// ===========================================================================

#[test]
fn render_json_roundtrip() {
    let receipt = make_receipt(&[make_file_stat("src/main.rs", 50, 10)]);
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let parsed: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(parsed.change_surface.files_changed, 1);
}

#[test]
fn render_markdown_contains_sections() {
    let receipt = make_receipt(&[make_file_stat("src/main.rs", 50, 10)]);
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    assert!(md.contains("## Glass Cockpit"));
    assert!(md.contains("Files Changed"));
    assert!(md.contains("Code Health Score"));
    assert!(md.contains("Risk Score"));
}

#[test]
fn render_sections_contains_review_plan() {
    let receipt = make_receipt(&[make_file_stat("src/main.rs", 50, 10)]);
    let sections = tokmd_cockpit::render::render_sections(&receipt);
    assert!(sections.contains("## Review Plan"));
}

#[test]
fn render_comment_md_summary() {
    let receipt = make_receipt(&[make_file_stat("src/main.rs", 50, 10)]);
    let comment = tokmd_cockpit::render::render_comment_md(&receipt);
    assert!(comment.contains("## Glass Cockpit Summary"));
}

#[test]
fn render_comment_with_contracts() {
    let stats = vec![
        make_file_stat("src/lib.rs", 10, 5),
        make_file_stat("docs/schema.json", 20, 10),
    ];
    let receipt = make_receipt(&stats);
    let comment = tokmd_cockpit::render::render_comment_md(&receipt);
    assert!(comment.contains("Contract changes"));
    assert!(comment.contains("API contract changed"));
    assert!(comment.contains("Schema contract changed"));
}

#[test]
fn render_comment_no_contracts() {
    let receipt = make_receipt(&[make_file_stat("src/utils.rs", 10, 5)]);
    let comment = tokmd_cockpit::render::render_comment_md(&receipt);
    assert!(!comment.contains("Contract changes"));
}

#[test]
fn render_write_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let receipt = make_receipt(&[make_file_stat("src/lib.rs", 20, 5)]);
    let out = dir.path().join("cockpit-output");
    tokmd_cockpit::render::write_artifacts(&out, &receipt).unwrap();
    assert!(out.join("cockpit.json").exists());
    assert!(out.join("report.json").exists());
    assert!(out.join("comment.md").exists());
}

#[test]
fn render_write_artifacts_nested() {
    let dir = tempfile::tempdir().unwrap();
    let deep = dir.path().join("a").join("b").join("c");
    let receipt = make_receipt(&[]);
    tokmd_cockpit::render::write_artifacts(&deep, &receipt).unwrap();
    assert!(deep.join("cockpit.json").exists());
}

#[test]
fn render_markdown_with_trend() {
    let mut receipt = make_receipt(&[make_file_stat("src/main.rs", 20, 5)]);
    receipt.trend = Some(TrendComparison {
        baseline_available: true,
        baseline_path: Some("/path/baseline.json".to_string()),
        baseline_generated_at_ms: Some(500),
        health: Some(TrendMetric {
            current: 90.0,
            previous: 80.0,
            delta: 10.0,
            delta_pct: 12.5,
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
            direction: TrendDirection::Stable,
            summary: "Complexity stable".to_string(),
            files_increased: 0,
            files_decreased: 0,
            avg_cyclomatic_delta: Some(0.0),
            avg_cognitive_delta: None,
        }),
    });
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    assert!(md.contains("### Trend"));
    assert!(md.contains("### Summary Comparison"));
}

#[test]
fn render_markdown_no_trend() {
    let receipt = make_receipt(&[]);
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    assert!(!md.contains("### Trend"));
}

#[test]
fn render_evidence_gates_in_markdown() {
    let mut receipt = make_receipt(&[]);
    receipt.evidence.overall_status = GateStatus::Fail;
    receipt.evidence.mutation.meta.status = GateStatus::Fail;
    receipt.evidence.mutation.survivors = vec![MutationSurvivor {
        file: "src/lib.rs".to_string(),
        line: 42,
        mutation: "replace + with -".to_string(),
    }];
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    assert!(md.contains("Evidence Gates"));
    assert!(md.contains("Mutation"));
    assert!(md.contains("survivors: 1"));
}

#[test]
fn render_report_json_envelope() {
    let dir = tempfile::tempdir().unwrap();
    let receipt = make_receipt(&[make_file_stat("src/lib.rs", 20, 5)]);
    let out = dir.path().join("out");
    tokmd_cockpit::render::write_artifacts(&out, &receipt).unwrap();
    let report_json = std::fs::read_to_string(out.join("report.json")).unwrap();
    let report: serde_json::Value = serde_json::from_str(&report_json).unwrap();
    assert!(report.get("tool").is_some());
    assert!(report.get("verdict").is_some());
}

// ===========================================================================
// 10. Utility Helpers
// ===========================================================================

#[test]
fn format_signed_positive() {
    assert_eq!(format_signed_f64(10.5), "+10.50");
}

#[test]
fn format_signed_negative() {
    assert_eq!(format_signed_f64(-3.14), "-3.14");
}

#[test]
fn format_signed_zero() {
    assert_eq!(format_signed_f64(0.0), "0.00");
}

#[test]
fn trend_label_improving() {
    assert_eq!(
        trend_direction_label(TrendDirection::Improving),
        "improving"
    );
}

#[test]
fn trend_label_stable() {
    assert_eq!(trend_direction_label(TrendDirection::Stable), "stable");
}

#[test]
fn trend_label_degrading() {
    assert_eq!(
        trend_direction_label(TrendDirection::Degrading),
        "degrading"
    );
}

#[test]
fn round_pct_basic() {
    assert!((round_pct(3.14159) - 3.14).abs() < 0.01);
}

#[test]
fn round_pct_zero() {
    assert_eq!(round_pct(0.0), 0.0);
}

#[test]
fn round_pct_negative() {
    assert!((round_pct(-2.567) - (-2.57)).abs() < 0.01);
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
    let s = sparkline(&[0.0, 25.0, 50.0, 75.0, 100.0]);
    assert_eq!(s.chars().count(), 5);
    let chars: Vec<char> = s.chars().collect();
    assert!(chars[0] < chars[4]);
}

#[test]
fn sparkline_all_equal() {
    let s = sparkline(&[42.0, 42.0, 42.0]);
    assert_eq!(s.chars().count(), 3);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars[0], chars[1]);
    assert_eq!(chars[1], chars[2]);
}

#[test]
fn now_iso8601_format() {
    let ts = now_iso8601();
    assert!(ts.ends_with('Z'));
    assert!(ts.contains('T'));
    assert_eq!(ts.len(), 20);
}

// ===========================================================================
// 11. End-to-End Workflows
// ===========================================================================

#[test]
fn e2e_empty_pr() {
    let receipt = make_receipt(&[]);
    assert_eq!(receipt.change_surface.files_changed, 0);
    assert_eq!(receipt.code_health.score, 100);
    assert_eq!(receipt.risk.level, RiskLevel::Low);
    assert!(receipt.review_plan.is_empty());
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let roundtrip: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtrip.schema_version, COCKPIT_SCHEMA_VERSION);
}

#[test]
fn e2e_large_mixed_pr() {
    let stats = vec![
        make_file_stat("src/lib.rs", 100, 30),
        make_file_stat("crates/tokmd/src/commands/new.rs", 200, 50),
        make_file_stat("tests/integration_test.rs", 50, 10),
        make_file_stat("docs/schema.json", 20, 5),
        make_file_stat("README.md", 10, 2),
        make_file_stat("Cargo.toml", 5, 1),
        make_file_stat("src/mega.rs", 400, 200),
    ];
    let receipt = make_receipt(&stats);
    assert!(receipt.contracts.api_changed);
    assert!(receipt.contracts.cli_changed);
    assert!(receipt.contracts.schema_changed);
    assert!(receipt.code_health.score < 100);
    assert_eq!(receipt.code_health.large_files_touched, 1);
    assert!(!receipt.risk.hotspots_touched.is_empty());
    for window in receipt.review_plan.windows(2) {
        assert!(window[0].priority <= window[1].priority);
    }
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let _: CockpitReceipt = serde_json::from_str(&json).unwrap();
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    assert!(md.contains("## Glass Cockpit"));
}

#[test]
fn e2e_schema_version_in_json() {
    let receipt = make_receipt(&[]);
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        value["schema_version"].as_u64().unwrap(),
        COCKPIT_SCHEMA_VERSION as u64
    );
}

#[test]
fn e2e_file_stat_as_ref() {
    let stat = make_file_stat("src/main.rs", 10, 5);
    let path: &str = stat.as_ref();
    assert_eq!(path, "src/main.rs");
}

// ===========================================================================
// 12. Edge Cases
// ===========================================================================

#[test]
fn edge_zero_line_files() {
    let stats = vec![
        make_file_stat("empty.rs", 0, 0),
        make_file_stat("also_empty.rs", 0, 0),
    ];
    let receipt = make_receipt(&stats);
    assert_eq!(receipt.code_health.score, 100);
    assert_eq!(receipt.code_health.avg_file_size, 0);
    assert_eq!(receipt.risk.score, 0);
}

#[test]
fn edge_many_small_files() {
    let stats: Vec<FileStat> = (0..100)
        .map(|i| make_file_stat(&format!("file_{i}.rs"), 5, 2))
        .collect();
    let receipt = make_receipt(&stats);
    assert_eq!(receipt.change_surface.files_changed, 100);
    assert_eq!(receipt.code_health.score, 100);
    assert_eq!(receipt.code_health.large_files_touched, 0);
    assert_eq!(receipt.risk.level, RiskLevel::Low);
    assert_eq!(receipt.review_plan.len(), 100);
}

#[test]
fn edge_exact_500_lines_not_large() {
    let stats = vec![make_file_stat("edge.rs", 300, 200)];
    let health = compute_code_health(&stats, &make_no_contract());
    assert_eq!(health.large_files_touched, 0);
}

#[test]
fn edge_exact_501_lines_is_large() {
    let stats = vec![make_file_stat("edge.rs", 301, 200)];
    let health = compute_code_health(&stats, &make_no_contract());
    assert_eq!(health.large_files_touched, 1);
}

#[test]
fn edge_exact_300_lines_not_hotspot() {
    let stats = vec![make_file_stat("edge.rs", 200, 100)];
    let health = compute_code_health(&stats, &make_no_contract());
    let risk = compute_risk(&stats, &make_no_contract(), &health);
    assert!(risk.hotspots_touched.is_empty());
}

#[test]
fn edge_exact_301_lines_is_hotspot() {
    let stats = vec![make_file_stat("edge.rs", 201, 100)];
    let health = compute_code_health(&stats, &make_no_contract());
    let risk = compute_risk(&stats, &make_no_contract(), &health);
    assert_eq!(risk.hotspots_touched.len(), 1);
}
