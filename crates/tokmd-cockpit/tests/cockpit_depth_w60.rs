//! Depth tests for tokmd-cockpit (w60).
//!
//! Exercises cockpit report construction, evidence gate evaluation,
//! change surface edge cases, review plan ranking, determinism,
//! capability reporting, and proptest invariants.

use proptest::prelude::*;
use tokmd_cockpit::render;
use tokmd_cockpit::*;

// ═══════════════════════════════════════════════════════════════════
// Helper builders
// ═══════════════════════════════════════════════════════════════════

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

fn gate_meta(status: GateStatus) -> GateMeta {
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
        evidence_commit: Some("abc1234".into()),
        evidence_generated_at_ms: Some(1_700_000_000_000),
    }
}

fn mutation_gate(status: GateStatus) -> MutationGate {
    MutationGate {
        meta: gate_meta(status),
        survivors: Vec::new(),
        killed: 5,
        timeout: 0,
        unviable: 0,
    }
}

fn evidence_all_skipped() -> Evidence {
    Evidence {
        overall_status: GateStatus::Skipped,
        mutation: mutation_gate(GateStatus::Skipped),
        diff_coverage: None,
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: None,
    }
}

fn evidence_pass() -> Evidence {
    Evidence {
        overall_status: GateStatus::Pass,
        mutation: mutation_gate(GateStatus::Pass),
        diff_coverage: None,
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: None,
    }
}

fn minimal_receipt() -> CockpitReceipt {
    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".into(),
        generated_at_ms: 1_700_000_000_000,
        base_ref: "main".into(),
        head_ref: "feature/w60".into(),
        change_surface: ChangeSurface {
            commits: 1,
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
            score: 100,
            grade: "A".into(),
            large_files_touched: 0,
            avg_file_size: 0,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: Vec::new(),
        },
        risk: Risk {
            hotspots_touched: Vec::new(),
            bus_factor_warnings: Vec::new(),
            level: RiskLevel::Low,
            score: 0,
        },
        contracts: no_contracts(),
        evidence: evidence_all_skipped(),
        review_plan: Vec::new(),
        trend: None,
    }
}

fn receipt_with_evidence(evidence: Evidence) -> CockpitReceipt {
    let mut r = minimal_receipt();
    r.evidence = evidence;
    r
}

// ═══════════════════════════════════════════════════════════════════
// BDD: Cockpit report construction
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bdd_minimal_receipt_has_correct_schema_version() {
    let r = minimal_receipt();
    assert_eq!(r.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(r.mode, "cockpit");
}

#[test]
fn bdd_receipt_preserves_ref_names() {
    let r = minimal_receipt();
    assert_eq!(r.base_ref, "main");
    assert_eq!(r.head_ref, "feature/w60");
}

#[test]
fn bdd_receipt_with_zero_change_surface() {
    let r = minimal_receipt();
    assert_eq!(r.change_surface.files_changed, 0);
    assert_eq!(r.change_surface.insertions, 0);
    assert_eq!(r.change_surface.deletions, 0);
    assert_eq!(r.change_surface.net_lines, 0);
    assert_eq!(r.change_surface.churn_velocity, 0.0);
}

#[test]
fn bdd_receipt_default_health_is_perfect() {
    let r = minimal_receipt();
    assert_eq!(r.code_health.score, 100);
    assert_eq!(r.code_health.grade, "A");
    assert!(r.code_health.warnings.is_empty());
}

#[test]
fn bdd_receipt_default_risk_is_low() {
    let r = minimal_receipt();
    assert_eq!(r.risk.level, RiskLevel::Low);
    assert_eq!(r.risk.score, 0);
    assert!(r.risk.hotspots_touched.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// BDD: Evidence gate evaluation — pass/fail/skip scenarios
// ═══════════════════════════════════════════════════════════════════

#[test]
fn gate_all_skipped_yields_skipped_overall() {
    let r = receipt_with_evidence(evidence_all_skipped());
    assert_eq!(r.evidence.overall_status, GateStatus::Skipped);
}

#[test]
fn gate_mutation_pass_with_no_optional_gates() {
    let r = receipt_with_evidence(evidence_pass());
    assert_eq!(r.evidence.overall_status, GateStatus::Pass);
    assert_eq!(r.evidence.mutation.meta.status, GateStatus::Pass);
}

#[test]
fn gate_mutation_survivors_listed() {
    let mut ev = evidence_pass();
    ev.mutation.survivors = vec![MutationSurvivor {
        file: "src/lib.rs".into(),
        line: 42,
        mutation: "replace == with !=".into(),
    }];
    ev.overall_status = GateStatus::Warn;
    let r = receipt_with_evidence(ev);
    assert_eq!(r.evidence.mutation.survivors.len(), 1);
    assert_eq!(r.evidence.mutation.survivors[0].line, 42);
}

#[test]
fn gate_diff_coverage_pass_at_80_pct() {
    let mut ev = evidence_pass();
    ev.diff_coverage = Some(DiffCoverageGate {
        meta: gate_meta(GateStatus::Pass),
        lines_added: 100,
        lines_covered: 80,
        coverage_pct: 0.80,
        uncovered_hunks: Vec::new(),
    });
    let r = receipt_with_evidence(ev);
    let dc = r.evidence.diff_coverage.as_ref().unwrap();
    assert_eq!(dc.meta.status, GateStatus::Pass);
    assert_eq!(dc.coverage_pct, 0.80);
}

#[test]
fn gate_diff_coverage_warn_at_50_pct() {
    let mut ev = evidence_pass();
    ev.diff_coverage = Some(DiffCoverageGate {
        meta: gate_meta(GateStatus::Warn),
        lines_added: 100,
        lines_covered: 50,
        coverage_pct: 0.50,
        uncovered_hunks: vec![UncoveredHunk {
            file: "src/main.rs".into(),
            start_line: 10,
            end_line: 20,
        }],
    });
    let r = receipt_with_evidence(ev);
    let dc = r.evidence.diff_coverage.as_ref().unwrap();
    assert_eq!(dc.meta.status, GateStatus::Warn);
    assert_eq!(dc.uncovered_hunks.len(), 1);
}

#[test]
fn gate_diff_coverage_fail_below_50_pct() {
    let mut ev = evidence_pass();
    ev.diff_coverage = Some(DiffCoverageGate {
        meta: gate_meta(GateStatus::Fail),
        lines_added: 100,
        lines_covered: 10,
        coverage_pct: 0.10,
        uncovered_hunks: Vec::new(),
    });
    ev.overall_status = GateStatus::Fail;
    let r = receipt_with_evidence(ev);
    assert_eq!(r.evidence.overall_status, GateStatus::Fail);
}

#[test]
fn gate_complexity_pass_no_high_files() {
    let mut ev = evidence_pass();
    ev.complexity = Some(ComplexityGate {
        meta: gate_meta(GateStatus::Pass),
        files_analyzed: 5,
        high_complexity_files: Vec::new(),
        avg_cyclomatic: 3.0,
        max_cyclomatic: 8,
        threshold_exceeded: false,
    });
    let r = receipt_with_evidence(ev);
    let cx = r.evidence.complexity.as_ref().unwrap();
    assert_eq!(cx.meta.status, GateStatus::Pass);
    assert!(!cx.threshold_exceeded);
}

#[test]
fn gate_complexity_warn_with_high_files() {
    let mut ev = evidence_pass();
    ev.complexity = Some(ComplexityGate {
        meta: gate_meta(GateStatus::Warn),
        files_analyzed: 3,
        high_complexity_files: vec![HighComplexityFile {
            path: "src/parser.rs".into(),
            cyclomatic: 20,
            function_count: 5,
            max_function_length: 80,
        }],
        avg_cyclomatic: 12.0,
        max_cyclomatic: 20,
        threshold_exceeded: true,
    });
    let r = receipt_with_evidence(ev);
    let cx = r.evidence.complexity.as_ref().unwrap();
    assert!(cx.threshold_exceeded);
    assert_eq!(cx.high_complexity_files.len(), 1);
    assert_eq!(cx.max_cyclomatic, 20);
}

#[test]
fn gate_supply_chain_with_vulnerabilities() {
    let mut ev = evidence_pass();
    ev.supply_chain = Some(SupplyChainGate {
        meta: gate_meta(GateStatus::Fail),
        vulnerabilities: vec![Vulnerability {
            id: "RUSTSEC-2024-001".into(),
            package: "unsafe-crate".into(),
            severity: "high".into(),
            title: "Memory safety issue".into(),
        }],
        denied: Vec::new(),
        advisory_db_version: Some("2024-01-01".into()),
    });
    ev.overall_status = GateStatus::Fail;
    let r = receipt_with_evidence(ev);
    let sc = r.evidence.supply_chain.as_ref().unwrap();
    assert_eq!(sc.vulnerabilities.len(), 1);
    assert_eq!(sc.vulnerabilities[0].severity, "high");
}

#[test]
fn gate_contracts_with_semver_failure() {
    let mut ev = evidence_pass();
    ev.contracts = Some(ContractDiffGate {
        meta: gate_meta(GateStatus::Fail),
        semver: Some(SemverSubGate {
            status: GateStatus::Fail,
            breaking_changes: vec![BreakingChange {
                kind: "semver".into(),
                path: "crates/tokmd-types/src/lib.rs".into(),
                message: "removed public function".into(),
            }],
        }),
        cli: None,
        schema: None,
        failures: 1,
    });
    ev.overall_status = GateStatus::Fail;
    let r = receipt_with_evidence(ev);
    let c = r.evidence.contracts.as_ref().unwrap();
    assert_eq!(c.failures, 1);
    assert!(c.semver.as_ref().unwrap().status == GateStatus::Fail);
}

#[test]
fn gate_determinism_pass_hashes_match() {
    let mut ev = evidence_pass();
    ev.determinism = Some(DeterminismGate {
        meta: gate_meta(GateStatus::Pass),
        expected_hash: Some("aabbccdd".into()),
        actual_hash: Some("aabbccdd".into()),
        algo: "blake3".into(),
        differences: Vec::new(),
    });
    let r = receipt_with_evidence(ev);
    let det = r.evidence.determinism.as_ref().unwrap();
    assert_eq!(det.meta.status, GateStatus::Pass);
    assert!(det.differences.is_empty());
}

#[test]
fn gate_determinism_warn_hashes_differ() {
    let mut ev = evidence_pass();
    ev.determinism = Some(DeterminismGate {
        meta: gate_meta(GateStatus::Warn),
        expected_hash: Some("aabbccdd".into()),
        actual_hash: Some("eeff0011".into()),
        algo: "blake3".into(),
        differences: vec!["source hash mismatch".into()],
    });
    ev.overall_status = GateStatus::Warn;
    let r = receipt_with_evidence(ev);
    let det = r.evidence.determinism.as_ref().unwrap();
    assert_eq!(det.differences.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// Change surface computation edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn change_surface_empty_files_all_zero() {
    let files: Vec<&str> = Vec::new();
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(comp.test_pct, 0.0);
    assert_eq!(comp.docs_pct, 0.0);
    assert_eq!(comp.config_pct, 0.0);
    assert_eq!(comp.test_ratio, 0.0);
}

#[test]
fn change_surface_single_code_file() {
    let files = vec!["src/main.rs"];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 1.0);
    assert_eq!(comp.test_ratio, 0.0);
}

#[test]
fn change_surface_only_test_files() {
    let files = vec!["tests/integration_test.rs", "src/test_utils.rs"];
    let comp = compute_composition(&files);
    assert_eq!(comp.test_pct, 1.0);
    assert_eq!(comp.code_pct, 0.0);
    // test_ratio: when code==0 and test>0, should be 1.0
    assert_eq!(comp.test_ratio, 1.0);
}

#[test]
fn change_surface_only_docs() {
    let files = vec!["README.md", "CHANGELOG.md"];
    let comp = compute_composition(&files);
    assert_eq!(comp.docs_pct, 1.0);
    assert_eq!(comp.code_pct, 0.0);
}

#[test]
fn change_surface_only_config() {
    let files = vec!["Cargo.toml", "config.yaml"];
    let comp = compute_composition(&files);
    assert_eq!(comp.config_pct, 1.0);
    assert_eq!(comp.code_pct, 0.0);
}

#[test]
fn change_surface_unrecognized_extensions_ignored() {
    // Files with unrecognized extensions should not be counted
    let files = vec!["image.png", "data.bin", "src/main.rs"];
    let comp = compute_composition(&files);
    // Only main.rs is recognized as code
    assert_eq!(comp.code_pct, 1.0);
}

#[test]
fn change_surface_net_lines_negative() {
    // Deletions exceed insertions
    let stats = vec![stat("src/cleanup.rs", 5, 50)];
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    let risk = compute_risk(stats.clone(), &contracts, &health);
    // Net lines is computed at higher level; verify health/risk don't panic
    assert!(health.score > 0);
    assert_eq!(risk.level, RiskLevel::Low);
}

#[test]
fn change_surface_massive_diff_many_files() {
    let stats: Vec<FileStat> = (0..200)
        .map(|i| stat(&format!("src/mod_{i}.rs"), 100, 50))
        .collect();
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    // 200 files * 150 lines each = many large files
    assert!(health.large_files_touched == 0); // 150 < 500 threshold
    assert_eq!(health.score, 100);
}

#[test]
fn change_surface_all_files_above_500_lines() {
    let stats = vec![
        stat("src/a.rs", 300, 250),
        stat("src/b.rs", 400, 200),
        stat("src/c.rs", 500, 100),
    ];
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    assert_eq!(health.large_files_touched, 3);
    assert_eq!(health.complexity_indicator, ComplexityIndicator::High);
    // Score: 100 - 3*10 = 70
    assert_eq!(health.score, 70);
    assert_eq!(health.grade, "C");
}

// ═══════════════════════════════════════════════════════════════════
// Review plan generation and ranking
// ═══════════════════════════════════════════════════════════════════

#[test]
fn review_plan_empty_stats_empty_plan() {
    let plan = generate_review_plan(&[], &no_contracts());
    assert!(plan.is_empty());
}

#[test]
fn review_plan_single_small_file() {
    let stats = vec![stat("src/utils.rs", 10, 5)];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].priority, 3);
    assert_eq!(plan[0].path, "src/utils.rs");
    assert_eq!(plan[0].lines_changed, Some(15));
}

#[test]
fn review_plan_medium_file_priority_2() {
    let stats = vec![stat("src/parser.rs", 40, 20)];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan[0].priority, 2);
}

#[test]
fn review_plan_large_file_priority_1() {
    let stats = vec![stat("src/engine.rs", 201, 100)];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan[0].priority, 1);
    assert_eq!(plan[0].complexity, Some(5));
}

#[test]
fn review_plan_sorted_by_priority() {
    let stats = vec![
        stat("src/tiny.rs", 5, 2),
        stat("src/huge.rs", 200, 100),
        stat("src/mid.rs", 30, 25),
    ];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan.len(), 3);
    assert_eq!(plan[0].priority, 1);
    assert_eq!(plan[0].path, "src/huge.rs");
    assert_eq!(plan[1].priority, 2);
    assert_eq!(plan[2].priority, 3);
}

#[test]
fn review_plan_complexity_thresholds() {
    // lines <= 100 => complexity 1
    let plan_small = generate_review_plan(&[stat("a.rs", 40, 10)], &no_contracts());
    assert_eq!(plan_small[0].complexity, Some(1));

    // 100 < lines <= 300 => complexity 3
    let plan_mid = generate_review_plan(&[stat("b.rs", 100, 50)], &no_contracts());
    assert_eq!(plan_mid[0].complexity, Some(3));

    // lines > 300 => complexity 5
    let plan_big = generate_review_plan(&[stat("c.rs", 200, 150)], &no_contracts());
    assert_eq!(plan_big[0].complexity, Some(5));
}

#[test]
fn review_plan_reason_contains_lines_changed() {
    let plan = generate_review_plan(&[stat("x.rs", 20, 10)], &no_contracts());
    assert!(plan[0].reason.contains("30"));
    assert!(plan[0].reason.contains("lines changed"));
}

// ═══════════════════════════════════════════════════════════════════
// Determinism: same inputs → same reports
// ═══════════════════════════════════════════════════════════════════

#[test]
fn determinism_same_inputs_same_composition() {
    let files = vec!["src/a.rs", "tests/b.rs", "README.md", "Cargo.toml"];
    let c1 = compute_composition(&files);
    let c2 = compute_composition(&files);
    assert_eq!(c1.code_pct, c2.code_pct);
    assert_eq!(c1.test_pct, c2.test_pct);
    assert_eq!(c1.docs_pct, c2.docs_pct);
    assert_eq!(c1.config_pct, c2.config_pct);
    assert_eq!(c1.test_ratio, c2.test_ratio);
}

#[test]
fn determinism_same_inputs_same_contracts() {
    let files = vec!["crates/tokmd-types/src/lib.rs", "docs/schema.json"];
    let c1 = detect_contracts(&files);
    let c2 = detect_contracts(&files);
    assert_eq!(c1.api_changed, c2.api_changed);
    assert_eq!(c1.cli_changed, c2.cli_changed);
    assert_eq!(c1.schema_changed, c2.schema_changed);
    assert_eq!(c1.breaking_indicators, c2.breaking_indicators);
}

#[test]
fn determinism_same_inputs_same_health() {
    let stats = vec![stat("src/a.rs", 100, 50), stat("src/b.rs", 600, 10)];
    let contracts = no_contracts();
    let h1 = compute_code_health(&stats, &contracts);
    let h2 = compute_code_health(&stats, &contracts);
    assert_eq!(h1.score, h2.score);
    assert_eq!(h1.grade, h2.grade);
    assert_eq!(h1.large_files_touched, h2.large_files_touched);
    assert_eq!(h1.complexity_indicator, h2.complexity_indicator);
}

#[test]
fn determinism_same_inputs_same_risk() {
    let stats = vec![stat("src/big.rs", 200, 150)];
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    let r1 = compute_risk(stats.clone(), &contracts, &health);
    let r2 = compute_risk(stats.clone(), &contracts, &health);
    assert_eq!(r1.score, r2.score);
    assert_eq!(r1.level, r2.level);
    assert_eq!(r1.hotspots_touched, r2.hotspots_touched);
}

#[test]
fn determinism_same_inputs_same_review_plan() {
    let stats = vec![
        stat("src/x.rs", 200, 100),
        stat("src/y.rs", 10, 5),
        stat("src/z.rs", 60, 20),
    ];
    let p1 = generate_review_plan(&stats, &no_contracts());
    let p2 = generate_review_plan(&stats, &no_contracts());
    assert_eq!(p1.len(), p2.len());
    for (a, b) in p1.iter().zip(p2.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.priority, b.priority);
        assert_eq!(a.complexity, b.complexity);
    }
}

#[test]
fn determinism_receipt_json_roundtrip() {
    let r = minimal_receipt();
    let json = serde_json::to_string_pretty(&r).unwrap();
    let r2: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(r.schema_version, r2.schema_version);
    assert_eq!(r.mode, r2.mode);
    assert_eq!(r.base_ref, r2.base_ref);
    assert_eq!(r.head_ref, r2.head_ref);
    assert_eq!(
        r.change_surface.files_changed,
        r2.change_surface.files_changed
    );
    assert_eq!(r.code_health.score, r2.code_health.score);
    assert_eq!(r.risk.score, r2.risk.score);
}

#[test]
fn determinism_json_roundtrip_with_all_gates() {
    let mut r = minimal_receipt();
    r.evidence = evidence_pass();
    r.evidence.diff_coverage = Some(DiffCoverageGate {
        meta: gate_meta(GateStatus::Pass),
        lines_added: 50,
        lines_covered: 45,
        coverage_pct: 0.90,
        uncovered_hunks: Vec::new(),
    });
    r.evidence.complexity = Some(ComplexityGate {
        meta: gate_meta(GateStatus::Pass),
        files_analyzed: 2,
        high_complexity_files: Vec::new(),
        avg_cyclomatic: 4.0,
        max_cyclomatic: 8,
        threshold_exceeded: false,
    });
    let json = serde_json::to_string(&r).unwrap();
    let r2: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(
        r.evidence.diff_coverage.as_ref().unwrap().coverage_pct,
        r2.evidence.diff_coverage.as_ref().unwrap().coverage_pct
    );
}

// ═══════════════════════════════════════════════════════════════════
// Trend computation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn trend_improving_health_higher_is_better() {
    let t = compute_metric_trend(95.0, 80.0, true);
    assert_eq!(t.direction, TrendDirection::Improving);
    assert!((t.delta - 15.0).abs() < 0.01);
}

#[test]
fn trend_degrading_health() {
    let t = compute_metric_trend(60.0, 90.0, true);
    assert_eq!(t.direction, TrendDirection::Degrading);
}

#[test]
fn trend_stable_small_delta() {
    let t = compute_metric_trend(80.3, 80.0, true);
    assert_eq!(t.direction, TrendDirection::Stable);
}

#[test]
fn trend_risk_lower_is_better_improving() {
    let t = compute_metric_trend(10.0, 50.0, false);
    assert_eq!(t.direction, TrendDirection::Improving);
}

#[test]
fn trend_risk_lower_is_better_degrading() {
    let t = compute_metric_trend(60.0, 20.0, false);
    assert_eq!(t.direction, TrendDirection::Degrading);
}

#[test]
fn trend_zero_previous_nonzero_current() {
    let t = compute_metric_trend(50.0, 0.0, true);
    assert_eq!(t.delta_pct, 100.0);
}

#[test]
fn trend_both_zero_stable() {
    let t = compute_metric_trend(0.0, 0.0, true);
    assert_eq!(t.direction, TrendDirection::Stable);
    assert_eq!(t.delta_pct, 0.0);
}

#[test]
fn complexity_trend_stable_no_complexity() {
    let c = minimal_receipt();
    let b = minimal_receipt();
    let ind = compute_complexity_trend(&c, &b);
    assert_eq!(ind.direction, TrendDirection::Stable);
    assert!(ind.summary.contains("stable"));
}

#[test]
fn complexity_trend_degrading() {
    let mut current = minimal_receipt();
    current.evidence.complexity = Some(ComplexityGate {
        meta: gate_meta(GateStatus::Warn),
        files_analyzed: 3,
        high_complexity_files: Vec::new(),
        avg_cyclomatic: 8.0,
        max_cyclomatic: 15,
        threshold_exceeded: false,
    });
    let baseline = minimal_receipt();
    let ind = compute_complexity_trend(&current, &baseline);
    assert_eq!(ind.direction, TrendDirection::Degrading);
    assert!(ind.summary.contains("increased"));
}

#[test]
fn complexity_trend_improving() {
    let current = minimal_receipt();
    let mut baseline = minimal_receipt();
    baseline.evidence.complexity = Some(ComplexityGate {
        meta: gate_meta(GateStatus::Pass),
        files_analyzed: 3,
        high_complexity_files: Vec::new(),
        avg_cyclomatic: 10.0,
        max_cyclomatic: 15,
        threshold_exceeded: false,
    });
    let ind = compute_complexity_trend(&current, &baseline);
    assert_eq!(ind.direction, TrendDirection::Improving);
    assert!(ind.summary.contains("decreased"));
}

// ═══════════════════════════════════════════════════════════════════
// Contract detection edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn contracts_no_contract_files() {
    let files = vec!["src/utils.rs", "src/helpers.rs"];
    let c = detect_contracts(&files);
    assert!(!c.api_changed);
    assert!(!c.cli_changed);
    assert!(!c.schema_changed);
    assert_eq!(c.breaking_indicators, 0);
}

#[test]
fn contracts_api_and_schema_both_changed() {
    let files = vec!["crates/tokmd-types/src/lib.rs", "docs/schema.json"];
    let c = detect_contracts(&files);
    assert!(c.api_changed);
    assert!(c.schema_changed);
    assert_eq!(c.breaking_indicators, 2);
}

#[test]
fn contracts_cli_changed_via_commands() {
    let files = vec!["crates/tokmd/src/commands/gate.rs"];
    let c = detect_contracts(&files);
    assert!(c.cli_changed);
    assert!(!c.api_changed);
}

#[test]
fn contracts_cli_changed_via_config() {
    let files = vec!["crates/tokmd-config/src/lib.rs"];
    let c = detect_contracts(&files);
    assert!(c.cli_changed);
    assert!(c.api_changed); // lib.rs also triggers API
}

#[test]
fn contracts_mod_rs_triggers_api() {
    let files = vec!["crates/tokmd-format/src/json/mod.rs"];
    let c = detect_contracts(&files);
    assert!(c.api_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_schema_md_triggers_schema() {
    let files = vec!["docs/SCHEMA.md"];
    let c = detect_contracts(&files);
    assert!(c.schema_changed);
}

// ═══════════════════════════════════════════════════════════════════
// Code health edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn health_empty_stats_perfect_score() {
    let h = compute_code_health(&[], &no_contracts());
    assert_eq!(h.score, 100);
    assert_eq!(h.grade, "A");
    assert_eq!(h.avg_file_size, 0);
}

#[test]
fn health_many_large_files_cap_at_zero() {
    let stats: Vec<FileStat> = (0..15)
        .map(|i| stat(&format!("src/f{i}.rs"), 400, 200))
        .collect();
    let h = compute_code_health(&stats, &no_contracts());
    // 15 large files * 10 = 150 penalty, saturating at 0
    assert_eq!(h.score, 0);
    assert_eq!(h.grade, "F");
}

#[test]
fn health_breaking_indicators_subtract_20() {
    let stats = vec![stat("src/lib.rs", 10, 5)];
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 1,
    };
    let h = compute_code_health(&stats, &contracts);
    assert_eq!(h.score, 80);
    assert_eq!(h.grade, "B");
}

#[test]
fn health_complexity_indicator_critical() {
    let stats: Vec<FileStat> = (0..6)
        .map(|i| stat(&format!("src/big{i}.rs"), 400, 200))
        .collect();
    let h = compute_code_health(&stats, &no_contracts());
    assert_eq!(h.complexity_indicator, ComplexityIndicator::Critical);
}

#[test]
fn health_warnings_for_each_large_file() {
    let stats = vec![stat("src/a.rs", 300, 250), stat("src/b.rs", 400, 200)];
    let h = compute_code_health(&stats, &no_contracts());
    assert_eq!(h.warnings.len(), 2);
    assert!(
        h.warnings
            .iter()
            .all(|w| w.warning_type == WarningType::LargeFile)
    );
}

// ═══════════════════════════════════════════════════════════════════
// Risk computation edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn risk_no_hotspots_low() {
    let stats = vec![stat("src/small.rs", 10, 5)];
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    let r = compute_risk(stats.clone(), &contracts, &health);
    assert_eq!(r.level, RiskLevel::Low);
    assert!(r.hotspots_touched.is_empty());
}

#[test]
fn risk_hotspot_above_300_lines() {
    let stats = vec![stat("src/core.rs", 200, 150)];
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    let r = compute_risk(stats.clone(), &contracts, &health);
    assert!(r.hotspots_touched.contains(&"src/core.rs".to_string()));
    assert!(r.score > 0);
}

#[test]
fn risk_score_capped_at_100() {
    let stats: Vec<FileStat> = (0..20)
        .map(|i| stat(&format!("src/big{i}.rs"), 300, 200))
        .collect();
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    let r = compute_risk(stats.clone(), &contracts, &health);
    assert!(r.score <= 100);
}

#[test]
fn risk_many_hotspots_high_or_critical() {
    let stats: Vec<FileStat> = (0..5)
        .map(|i| stat(&format!("src/hot{i}.rs"), 200, 200))
        .collect();
    let contracts = no_contracts();
    let health = compute_code_health(&stats, &contracts);
    let r = compute_risk(stats.clone(), &contracts, &health);
    assert!(matches!(r.level, RiskLevel::High | RiskLevel::Critical));
}

// ═══════════════════════════════════════════════════════════════════
// Utility function tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn format_signed_positive() {
    assert_eq!(format_signed_f64(3.15), "+3.15");
}

#[test]
fn format_signed_negative() {
    assert_eq!(format_signed_f64(-2.5), "-2.50");
}

#[test]
fn format_signed_zero() {
    assert_eq!(format_signed_f64(0.0), "0.00");
}

#[test]
fn trend_label_all_directions() {
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
fn sparkline_single() {
    assert_eq!(sparkline(&[50.0]).chars().count(), 1);
}

#[test]
fn sparkline_ascending() {
    let s = sparkline(&[0.0, 50.0, 100.0]);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars.len(), 3);
    // First char should be lowest bar, last should be highest
    assert!(chars[0] < chars[2]);
}

#[test]
fn sparkline_equal_values_same_bars() {
    let s = sparkline(&[42.0, 42.0, 42.0]);
    let chars: Vec<char> = s.chars().collect();
    assert!(chars.iter().all(|c| *c == chars[0]));
}

#[test]
fn round_pct_basic() {
    assert_eq!(round_pct(0.0), 0.0);
    assert_eq!(round_pct(1.0), 1.0);
    assert_eq!(round_pct(0.456), 0.46);
    assert_eq!(round_pct(0.554), 0.55);
}

#[test]
fn now_iso8601_format() {
    let ts = now_iso8601();
    assert!(ts.ends_with('Z'));
    assert!(ts.contains('T'));
    assert_eq!(ts.len(), 20);
}

// ═══════════════════════════════════════════════════════════════════
// Render functions
// ═══════════════════════════════════════════════════════════════════

#[test]
fn render_json_valid_json() {
    let r = minimal_receipt();
    let json = render::render_json(&r).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["mode"], "cockpit");
    assert_eq!(parsed["schema_version"], COCKPIT_SCHEMA_VERSION);
}

#[test]
fn render_markdown_contains_sections() {
    let r = minimal_receipt();
    let md = render::render_markdown(&r);
    assert!(md.contains("## Glass Cockpit"));
    assert!(md.contains("### Change Surface"));
    assert!(md.contains("### Composition"));
    assert!(md.contains("### Contracts"));
    assert!(md.contains("### Code Health"));
    assert!(md.contains("### Risk"));
    assert!(md.contains("### Evidence Gates"));
    assert!(md.contains("### Review Plan"));
}

#[test]
fn render_sections_contains_markers() {
    let r = minimal_receipt();
    let s = render::render_sections(&r);
    assert!(s.contains("<!-- SECTION:COCKPIT -->"));
    assert!(s.contains("<!-- SECTION:REVIEW_PLAN -->"));
    assert!(s.contains("<!-- SECTION:RECEIPTS -->"));
}

#[test]
fn render_comment_md_basic() {
    let r = minimal_receipt();
    let c = render::render_comment_md(&r);
    assert!(c.contains("## Glass Cockpit Summary"));
    assert!(c.contains("Health"));
    assert!(c.contains("Risk"));
}

#[test]
fn render_comment_md_shows_contract_changes() {
    let mut r = minimal_receipt();
    r.contracts.api_changed = true;
    r.contracts.cli_changed = true;
    let c = render::render_comment_md(&r);
    assert!(c.contains("Contract changes"));
    assert!(c.contains("API contract changed"));
    assert!(c.contains("CLI contract changed"));
}

#[test]
fn render_comment_md_shows_priority_review_items() {
    let mut r = minimal_receipt();
    r.review_plan = vec![
        ReviewItem {
            path: "src/important.rs".into(),
            reason: "critical change".into(),
            priority: 1,
            complexity: Some(5),
            lines_changed: Some(300),
        },
        ReviewItem {
            path: "src/trivial.rs".into(),
            reason: "minor tweak".into(),
            priority: 3,
            complexity: Some(1),
            lines_changed: Some(5),
        },
    ];
    let c = render::render_comment_md(&r);
    assert!(c.contains("Priority review items"));
    assert!(c.contains("src/important.rs"));
    // Priority 3 should NOT appear in priority review items
    assert!(!c.contains("src/trivial.rs"));
}

#[test]
fn render_markdown_with_trend() {
    let mut r = minimal_receipt();
    r.trend = Some(TrendComparison {
        baseline_available: true,
        baseline_path: Some("/tmp/baseline.json".into()),
        baseline_generated_at_ms: Some(1_600_000_000_000),
        health: Some(TrendMetric {
            current: 90.0,
            previous: 80.0,
            delta: 10.0,
            delta_pct: 12.5,
            direction: TrendDirection::Improving,
        }),
        risk: None,
        complexity: None,
    });
    let md = render::render_markdown(&r);
    assert!(md.contains("### Trend"));
    assert!(md.contains("/tmp/baseline.json"));
}

#[test]
fn write_artifacts_creates_files() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("cockpit-out");
    let r = minimal_receipt();
    render::write_artifacts(&out, &r).unwrap();

    assert!(out.join("cockpit.json").exists());
    assert!(out.join("report.json").exists());
    assert!(out.join("comment.md").exists());

    // Verify cockpit.json is valid
    let content = std::fs::read_to_string(out.join("cockpit.json")).unwrap();
    let parsed: CockpitReceipt = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.mode, "cockpit");
}

// ═══════════════════════════════════════════════════════════════════
// Determinism hashing (unit tests for the determinism module)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn hash_files_deterministic_same_content() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
    std::fs::write(dir.path().join("b.rs"), "fn test() {}").unwrap();

    let h1 = determinism::hash_files_from_paths(dir.path(), &["a.rs", "b.rs"]).unwrap();
    let h2 = determinism::hash_files_from_paths(dir.path(), &["a.rs", "b.rs"]).unwrap();
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64);
}

#[test]
fn hash_files_order_independent() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("x.rs"), "let x = 1;").unwrap();
    std::fs::write(dir.path().join("y.rs"), "let y = 2;").unwrap();

    let h1 = determinism::hash_files_from_paths(dir.path(), &["x.rs", "y.rs"]).unwrap();
    let h2 = determinism::hash_files_from_paths(dir.path(), &["y.rs", "x.rs"]).unwrap();
    assert_eq!(h1, h2);
}

#[test]
fn hash_files_content_change_changes_hash() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "v1").unwrap();
    let h1 = determinism::hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();

    std::fs::write(dir.path().join("a.rs"), "v2").unwrap();
    let h2 = determinism::hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn hash_cargo_lock_present() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.lock"), "[[package]]").unwrap();
    let h = determinism::hash_cargo_lock(dir.path()).unwrap();
    assert!(h.is_some());
    assert_eq!(h.unwrap().len(), 64);
}

#[test]
fn hash_cargo_lock_absent() {
    let dir = tempfile::tempdir().unwrap();
    let h = determinism::hash_cargo_lock(dir.path()).unwrap();
    assert!(h.is_none());
}

#[test]
fn hash_skips_target_and_git_dirs() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "code").unwrap();
    std::fs::create_dir_all(dir.path().join("target")).unwrap();
    std::fs::write(dir.path().join("target/debug.rs"), "debug").unwrap();

    // Hashing with target path should produce same result as without it
    let h1 = determinism::hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();
    let h2 = determinism::hash_files_from_paths(dir.path(), &["a.rs", "target/debug.rs"]).unwrap();
    assert_eq!(h1, h2);
}

// ═══════════════════════════════════════════════════════════════════
// FileStat AsRef<str> impl
// ═══════════════════════════════════════════════════════════════════

#[test]
fn file_stat_as_ref_str() {
    let fs = stat("src/lib.rs", 10, 5);
    let s: &str = fs.as_ref();
    assert_eq!(s, "src/lib.rs");
}

// ═══════════════════════════════════════════════════════════════════
// COMPLEXITY_THRESHOLD constant
// ═══════════════════════════════════════════════════════════════════

#[test]
fn complexity_threshold_value() {
    assert_eq!(COMPLEXITY_THRESHOLD, 15);
}

// ═══════════════════════════════════════════════════════════════════
// Proptest: property-based tests
// ═══════════════════════════════════════════════════════════════════

fn file_ext_strat() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(".rs".to_string()),
        Just(".js".to_string()),
        Just(".ts".to_string()),
        Just(".py".to_string()),
        Just(".md".to_string()),
        Just(".toml".to_string()),
        Just(".json".to_string()),
        Just(".yaml".to_string()),
    ]
}

fn file_path_strat() -> impl Strategy<Value = String> {
    (
        prop_oneof![
            Just("src/".to_string()),
            Just("tests/".to_string()),
            Just("docs/".to_string()),
            Just("crates/tokmd/src/commands/".to_string()),
            Just("".to_string()),
        ],
        "[a-z]{1,8}",
        file_ext_strat(),
    )
        .prop_map(|(dir, name, ext)| format!("{dir}{name}{ext}"))
}

fn file_stat_strat() -> impl Strategy<Value = FileStat> {
    (file_path_strat(), 0..2000usize, 0..2000usize).prop_map(|(path, ins, del)| FileStat {
        path,
        insertions: ins,
        deletions: del,
    })
}

proptest! {
    #[test]
    fn prop_composition_pcts_sum_leq_one(
        paths in prop::collection::vec(file_path_strat(), 0..30)
    ) {
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let comp = compute_composition(&refs);
        let sum = comp.code_pct + comp.test_pct + comp.docs_pct + comp.config_pct;
        prop_assert!(sum <= 1.001, "sum={sum}");
        prop_assert!((0.0..=1.0).contains(&comp.code_pct));
        prop_assert!((0.0..=1.0).contains(&comp.test_pct));
        prop_assert!((0.0..=1.0).contains(&comp.docs_pct));
        prop_assert!((0.0..=1.0).contains(&comp.config_pct));
    }

    #[test]
    fn prop_health_score_in_range(
        stats in prop::collection::vec(file_stat_strat(), 0..20),
        api in proptest::bool::ANY,
        schema in proptest::bool::ANY,
    ) {
        let mut breaking = 0usize;
        if api { breaking += 1; }
        if schema { breaking += 1; }
        let contracts = Contracts {
            api_changed: api,
            cli_changed: false,
            schema_changed: schema,
            breaking_indicators: breaking,
        };
        let h = compute_code_health(&stats, &contracts);
        prop_assert!(h.score <= 100);
        prop_assert!(matches!(
            h.grade.as_str(),
            "A" | "B" | "C" | "D" | "F"
        ));
    }

    #[test]
    fn prop_risk_score_capped(
        stats in prop::collection::vec(file_stat_strat(), 0..20),
    ) {
        let contracts = no_contracts();
        let health = compute_code_health(&stats, &contracts);
        let r = compute_risk(stats.clone(), &contracts, &health);
        prop_assert!(r.score <= 100);
        prop_assert!(matches!(
            r.level,
            RiskLevel::Low | RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical
        ));
    }

    #[test]
    fn prop_review_plan_sorted_by_priority(
        stats in prop::collection::vec(file_stat_strat(), 0..15),
    ) {
        let plan = generate_review_plan(&stats, &no_contracts());
        for window in plan.windows(2) {
            prop_assert!(window[0].priority <= window[1].priority);
        }
    }

    #[test]
    fn prop_trend_direction_consistent(
        current in 0.0f64..200.0,
        previous in 0.0f64..200.0,
    ) {
        let t = compute_metric_trend(current, previous, true);
        let delta = current - previous;
        if delta.abs() < 1.0 {
            prop_assert_eq!(t.direction, TrendDirection::Stable);
        } else if delta > 0.0 {
            prop_assert_eq!(t.direction, TrendDirection::Improving);
        } else {
            prop_assert_eq!(t.direction, TrendDirection::Degrading);
        }
    }

    #[test]
    fn prop_round_pct_finite(val in -1000.0f64..1000.0) {
        let rounded = round_pct(val);
        prop_assert!(rounded.is_finite());
    }

    #[test]
    fn prop_sparkline_length_matches_input(
        values in prop::collection::vec(0.0f64..100.0, 0..20)
    ) {
        let s = sparkline(&values);
        prop_assert_eq!(s.chars().count(), values.len());
    }

    #[test]
    fn prop_composition_deterministic(
        paths in prop::collection::vec(file_path_strat(), 0..20)
    ) {
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let c1 = compute_composition(&refs);
        let c2 = compute_composition(&refs);
        prop_assert_eq!(c1.code_pct.to_bits(), c2.code_pct.to_bits());
        prop_assert_eq!(c1.test_pct.to_bits(), c2.test_pct.to_bits());
    }
}
