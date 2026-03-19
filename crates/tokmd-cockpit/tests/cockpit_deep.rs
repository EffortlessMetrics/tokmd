//! Deep tests for cockpit receipt construction, evidence gates, sparklines,
//! trend comparison, change surface, review plan ranking, and rendering.

use serde_json::Value;
use tokmd_cockpit::*;

// =============================================================================
// Helpers
// =============================================================================

fn make_file_stat(path: &str, insertions: usize, deletions: usize) -> FileStat {
    FileStat {
        path: path.to_string(),
        insertions,
        deletions,
    }
}

fn minimal_gate_meta(status: GateStatus) -> GateMeta {
    GateMeta {
        status,
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

fn minimal_evidence() -> Evidence {
    Evidence {
        overall_status: GateStatus::Pass,
        mutation: MutationGate {
            meta: minimal_gate_meta(GateStatus::Pass),
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
    }
}

fn build_receipt(stats: &[FileStat]) -> CockpitReceipt {
    let contracts = detect_contracts(stats);
    let composition = compute_composition(stats);
    let code_health = compute_code_health(stats, &contracts);
    let risk = compute_risk(stats.to_vec(), &contracts, &code_health);
    let review_plan = generate_review_plan(stats, &contracts);

    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 1700000000000,
        base_ref: "main".to_string(),
        head_ref: "feature".to_string(),
        change_surface: ChangeSurface {
            commits: 1,
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
        evidence: minimal_evidence(),
        review_plan,
        trend: None,
    }
}

// =============================================================================
// Receipt construction
// =============================================================================

#[test]
fn cockpit_receipt_schema_version_matches_constant() {
    let r = build_receipt(&[make_file_stat("src/lib.rs", 10, 5)]);
    assert_eq!(r.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(r.schema_version, 3);
}

#[test]
fn cockpit_receipt_mode_field_is_cockpit() {
    let r = build_receipt(&[]);
    assert_eq!(r.mode, "cockpit");
}

#[test]
fn cockpit_receipt_empty_files_produces_valid_receipt() {
    let r = build_receipt(&[]);
    assert_eq!(r.change_surface.files_changed, 0);
    assert_eq!(r.change_surface.insertions, 0);
    assert_eq!(r.change_surface.deletions, 0);
    assert_eq!(r.change_surface.net_lines, 0);
}

// =============================================================================
// Serialization roundtrip
// =============================================================================

#[test]
fn cockpit_receipt_json_roundtrip_preserves_all_fields() {
    let stats = vec![
        make_file_stat("src/lib.rs", 100, 20),
        make_file_stat("tests/test.rs", 50, 10),
    ];
    let r = build_receipt(&stats);
    let json_str = serde_json::to_string_pretty(&r).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json_str).unwrap();

    assert_eq!(back.schema_version, r.schema_version);
    assert_eq!(back.mode, r.mode);
    assert_eq!(back.base_ref, r.base_ref);
    assert_eq!(back.head_ref, r.head_ref);
    assert_eq!(back.change_surface.commits, r.change_surface.commits);
    assert_eq!(
        back.change_surface.files_changed,
        r.change_surface.files_changed
    );
    assert_eq!(back.review_plan.len(), r.review_plan.len());
    assert_eq!(back.code_health.score, r.code_health.score);
}

#[test]
fn cockpit_receipt_serialized_twice_is_deterministic() {
    let r = build_receipt(&[make_file_stat("src/main.rs", 50, 10)]);
    let json1 = serde_json::to_string(&r).unwrap();
    let json2 = serde_json::to_string(&r).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn cockpit_receipt_json_has_required_envelope_keys() {
    let r = build_receipt(&[make_file_stat("src/lib.rs", 10, 0)]);
    let val: Value = serde_json::to_value(r).unwrap();
    for key in &[
        "schema_version",
        "mode",
        "generated_at_ms",
        "base_ref",
        "head_ref",
        "change_surface",
        "composition",
        "code_health",
        "risk",
        "contracts",
        "evidence",
        "review_plan",
    ] {
        assert!(val.get(key).is_some(), "missing key: {key}");
    }
}

// =============================================================================
// Change surface computation
// =============================================================================

#[test]
fn change_surface_net_lines_positive_when_more_insertions() {
    let stats = vec![make_file_stat("a.rs", 100, 20)];
    let r = build_receipt(&stats);
    assert_eq!(r.change_surface.net_lines, 80);
}

#[test]
fn change_surface_net_lines_negative_when_more_deletions() {
    let stats = vec![make_file_stat("a.rs", 10, 50)];
    let r = build_receipt(&stats);
    assert_eq!(r.change_surface.net_lines, -40);
}

#[test]
fn change_surface_files_changed_counts_all_stats() {
    let stats: Vec<FileStat> = (0..7)
        .map(|i| make_file_stat(&format!("f{i}.rs"), 1, 0))
        .collect();
    let r = build_receipt(&stats);
    assert_eq!(r.change_surface.files_changed, 7);
}

// =============================================================================
// Composition
// =============================================================================

#[test]
fn composition_all_code_files() {
    let stats = vec![
        make_file_stat("src/a.rs", 10, 0),
        make_file_stat("src/b.rs", 20, 0),
    ];
    let comp = compute_composition(&stats);
    assert!(comp.code_pct > 0.0);
}

#[test]
fn composition_test_files_detected() {
    let stats = vec![
        make_file_stat("src/lib.rs", 10, 0),
        make_file_stat("tests/test_lib.rs", 10, 0),
    ];
    let comp = compute_composition(&stats);
    assert!(comp.test_pct > 0.0);
}

#[test]
fn composition_docs_detected() {
    let stats = vec![
        make_file_stat("README.md", 10, 0),
        make_file_stat("docs/guide.md", 10, 0),
    ];
    let comp = compute_composition(&stats);
    assert!(comp.docs_pct > 0.0);
}

#[test]
fn composition_config_detected() {
    let stats = vec![
        make_file_stat("Cargo.toml", 10, 0),
        make_file_stat(".github/ci.yml", 5, 0),
    ];
    let comp = compute_composition(&stats);
    assert!(comp.config_pct > 0.0);
}

#[test]
fn composition_empty_files_all_zero() {
    let comp = compute_composition::<FileStat>(&[]);
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(comp.test_pct, 0.0);
    assert_eq!(comp.docs_pct, 0.0);
    assert_eq!(comp.config_pct, 0.0);
    assert_eq!(comp.test_ratio, 0.0);
}

// =============================================================================
// Code health
// =============================================================================

#[test]
fn code_health_small_change_high_score() {
    let stats = vec![make_file_stat("src/lib.rs", 5, 2)];
    let contracts = detect_contracts(&stats);
    let health = compute_code_health(&stats, &contracts);
    assert!(health.score >= 70, "small change should have good health");
}

#[test]
fn code_health_large_file_detected() {
    let stats = vec![make_file_stat("src/big.rs", 400, 200)];
    let contracts = detect_contracts(&stats);
    let health = compute_code_health(&stats, &contracts);
    assert!(health.large_files_touched >= 1);
}

#[test]
fn code_health_empty_stats_max_score() {
    let contracts = detect_contracts::<FileStat>(&[]);
    let health = compute_code_health(&[], &contracts);
    assert!(health.score >= 90);
}

// =============================================================================
// Risk
// =============================================================================

#[test]
fn risk_low_for_small_change() {
    let stats = vec![make_file_stat("src/lib.rs", 5, 2)];
    let contracts = detect_contracts(&stats);
    let health = compute_code_health(&stats, &contracts);
    let risk = compute_risk(stats.clone(), &contracts, &health);
    assert!(risk.score <= 30, "small change should be low risk");
}

#[test]
fn risk_hotspot_detected_for_large_file() {
    let stats = vec![make_file_stat("src/core.rs", 200, 150)];
    let contracts = detect_contracts(&stats);
    let health = compute_code_health(&stats, &contracts);
    let risk = compute_risk(stats.clone(), &contracts, &health);
    assert!(
        risk.hotspots_touched.contains(&"src/core.rs".to_string()),
        "files with >300 total lines should be hotspots"
    );
}

// =============================================================================
// Contracts
// =============================================================================

#[test]
fn contracts_api_change_detected_for_lib_rs() {
    let stats = vec![make_file_stat("crates/foo/src/lib.rs", 10, 5)];
    let contracts = detect_contracts(&stats);
    assert!(contracts.api_changed);
}

#[test]
fn contracts_cli_change_detected_for_commands() {
    let stats = vec![make_file_stat("crates/tokmd/src/commands/run.rs", 10, 5)];
    let contracts = detect_contracts(&stats);
    assert!(contracts.cli_changed);
}

#[test]
fn contracts_schema_change_detected() {
    let stats = vec![make_file_stat("docs/schema.json", 50, 20)];
    let contracts = detect_contracts(&stats);
    assert!(contracts.schema_changed);
}

#[test]
fn contracts_no_changes_for_normal_files() {
    let stats = vec![make_file_stat("src/utils.rs", 10, 5)];
    let contracts = detect_contracts(&stats);
    assert!(!contracts.api_changed);
    assert!(!contracts.cli_changed);
    assert!(!contracts.schema_changed);
    assert_eq!(contracts.breaking_indicators, 0);
}

// =============================================================================
// Evidence gates
// =============================================================================

#[test]
fn evidence_mutation_gate_survivors_accessible() {
    let gate = MutationGate {
        meta: minimal_gate_meta(GateStatus::Warn),
        survivors: vec![MutationSurvivor {
            file: "src/calc.rs".to_string(),
            line: 42,
            mutation: "replaced + with -".to_string(),
        }],
        killed: 49,
        timeout: 1,
        unviable: 0,
    };
    assert_eq!(gate.survivors.len(), 1);
    assert_eq!(gate.killed, 49);
}

#[test]
fn evidence_diff_coverage_roundtrip() {
    let gate = DiffCoverageGate {
        meta: minimal_gate_meta(GateStatus::Pass),
        lines_added: 100,
        lines_covered: 95,
        coverage_pct: 95.0,
        uncovered_hunks: vec![],
    };
    let json = serde_json::to_string(&gate).unwrap();
    let back: DiffCoverageGate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.coverage_pct, 95.0);
    assert_eq!(back.lines_covered, 95);
}

#[test]
fn evidence_complexity_gate_threshold() {
    let gate = ComplexityGate {
        meta: minimal_gate_meta(GateStatus::Warn),
        files_analyzed: 10,
        high_complexity_files: vec![HighComplexityFile {
            path: "src/monster.rs".to_string(),
            cyclomatic: 50,
            function_count: 20,
            max_function_length: 300,
        }],
        avg_cyclomatic: 12.0,
        max_cyclomatic: 50,
        threshold_exceeded: true,
    };
    assert!(gate.threshold_exceeded);
    assert_eq!(gate.max_cyclomatic, 50);
}

#[test]
fn evidence_all_optional_gates_serialized_when_present() {
    let evidence = Evidence {
        overall_status: GateStatus::Warn,
        mutation: MutationGate {
            meta: minimal_gate_meta(GateStatus::Pass),
            survivors: vec![],
            killed: 10,
            timeout: 0,
            unviable: 0,
        },
        diff_coverage: Some(DiffCoverageGate {
            meta: minimal_gate_meta(GateStatus::Pass),
            lines_added: 50,
            lines_covered: 50,
            coverage_pct: 100.0,
            uncovered_hunks: vec![],
        }),
        contracts: Some(ContractDiffGate {
            meta: minimal_gate_meta(GateStatus::Pass),
            semver: None,
            cli: None,
            schema: None,
            failures: 0,
        }),
        supply_chain: Some(SupplyChainGate {
            meta: minimal_gate_meta(GateStatus::Pass),
            vulnerabilities: vec![],
            denied: vec![],
            advisory_db_version: None,
        }),
        determinism: Some(DeterminismGate {
            meta: minimal_gate_meta(GateStatus::Pass),
            expected_hash: None,
            actual_hash: None,
            algo: "blake3".to_string(),
            differences: vec![],
        }),
        complexity: Some(ComplexityGate {
            meta: minimal_gate_meta(GateStatus::Pass),
            files_analyzed: 5,
            high_complexity_files: vec![],
            avg_cyclomatic: 3.0,
            max_cyclomatic: 8,
            threshold_exceeded: false,
        }),
    };
    let val: Value = serde_json::to_value(evidence).unwrap();
    assert!(val["diff_coverage"].is_object());
    assert!(val["contracts"].is_object());
    assert!(val["supply_chain"].is_object());
    assert!(val["determinism"].is_object());
    assert!(val["complexity"].is_object());
}

#[test]
fn evidence_optional_gates_omitted_when_none() {
    let evidence = minimal_evidence();
    let json = serde_json::to_string(&evidence).unwrap();
    assert!(!json.contains("\"diff_coverage\""));
    assert!(!json.contains("\"contracts\""));
    assert!(!json.contains("\"supply_chain\""));
    assert!(!json.contains("\"determinism\""));
    assert!(!json.contains("\"complexity\""));
}

// =============================================================================
// Review plan ranking
// =============================================================================

#[test]
fn review_plan_large_file_gets_priority_one() {
    let stats = vec![make_file_stat("src/big.rs", 250, 50)];
    let contracts = detect_contracts(&stats);
    let plan = generate_review_plan(&stats, &contracts);
    assert_eq!(plan.len(), 1);
    assert_eq!(
        plan[0].priority, 1,
        "files > 200 lines should be priority 1"
    );
}

#[test]
fn review_plan_medium_file_gets_priority_two() {
    let stats = vec![make_file_stat("src/mid.rs", 60, 10)];
    let contracts = detect_contracts(&stats);
    let plan = generate_review_plan(&stats, &contracts);
    assert_eq!(
        plan[0].priority, 2,
        "files 51-200 lines should be priority 2"
    );
}

#[test]
fn review_plan_small_file_gets_priority_three() {
    let stats = vec![make_file_stat("src/tiny.rs", 5, 2)];
    let contracts = detect_contracts(&stats);
    let plan = generate_review_plan(&stats, &contracts);
    assert_eq!(
        plan[0].priority, 3,
        "files <= 50 lines should be priority 3"
    );
}

#[test]
fn review_plan_sorted_by_priority() {
    let stats = vec![
        make_file_stat("src/a.rs", 5, 0),   // priority 3
        make_file_stat("src/b.rs", 250, 0), // priority 1
        make_file_stat("src/c.rs", 80, 0),  // priority 2
    ];
    let contracts = detect_contracts(&stats);
    let plan = generate_review_plan(&stats, &contracts);
    let priorities: Vec<u32> = plan.iter().map(|r| r.priority).collect();
    let mut sorted = priorities.clone();
    sorted.sort();
    assert_eq!(
        priorities, sorted,
        "review plan should be sorted by priority"
    );
}

#[test]
fn review_plan_complexity_increases_with_size() {
    let stats = vec![
        make_file_stat("src/small.rs", 10, 0),
        make_file_stat("src/medium.rs", 120, 0),
        make_file_stat("src/large.rs", 400, 0),
    ];
    let contracts = detect_contracts(&stats);
    let plan = generate_review_plan(&stats, &contracts);
    // Sort by path to align with the stat order, since plan is sorted by priority
    let mut by_path: Vec<_> = plan.iter().collect();
    by_path.sort_by_key(|r| &r.path);
    let complexities: Vec<u8> = by_path.iter().filter_map(|r| r.complexity).collect();
    assert!(complexities.len() == 3);
    // large.rs < medium.rs < small.rs alphabetically, so:
    // large (5) >= medium (3) >= small (1)
    assert!(complexities[0] >= complexities[1]);
    assert!(complexities[1] >= complexities[2]);
}

// =============================================================================
// Sparkline generation
// =============================================================================

#[test]
fn sparkline_empty_returns_empty_string() {
    assert_eq!(sparkline(&[]), "");
}

#[test]
fn sparkline_single_value_returns_middle_bar() {
    let s = sparkline(&[50.0]);
    assert_eq!(s.chars().count(), 1);
}

#[test]
fn sparkline_ascending_values() {
    let s = sparkline(&[0.0, 25.0, 50.0, 75.0, 100.0]);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars.len(), 5);
    // First should be lowest bar, last should be highest bar
    assert!(chars[0] <= chars[4]);
}

#[test]
fn sparkline_equal_values_all_same_char() {
    let s = sparkline(&[42.0, 42.0, 42.0]);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars[0], chars[1]);
    assert_eq!(chars[1], chars[2]);
}

#[test]
fn sparkline_two_values_shows_direction() {
    let up = sparkline(&[10.0, 90.0]);
    let chars: Vec<char> = up.chars().collect();
    assert!(
        chars[0] < chars[1],
        "ascending sparkline should show direction"
    );
}

// =============================================================================
// Trend comparison
// =============================================================================

#[test]
fn trend_comparison_default_is_unavailable() {
    let trend = TrendComparison::default();
    assert!(!trend.baseline_available);
    assert!(trend.health.is_none());
    assert!(trend.risk.is_none());
    assert!(trend.complexity.is_none());
}

#[test]
fn trend_metric_improving_has_positive_delta() {
    let metric = TrendMetric {
        current: 90.0,
        previous: 75.0,
        delta: 15.0,
        delta_pct: 20.0,
        direction: TrendDirection::Improving,
    };
    assert!(metric.delta > 0.0);
    assert_eq!(metric.direction, TrendDirection::Improving);
}

#[test]
fn trend_metric_degrading_has_negative_delta() {
    let metric = TrendMetric {
        current: 60.0,
        previous: 80.0,
        delta: -20.0,
        delta_pct: -25.0,
        direction: TrendDirection::Degrading,
    };
    assert!(metric.delta < 0.0);
}

#[test]
fn trend_comparison_with_all_metrics_roundtrips() {
    let trend = TrendComparison {
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
        risk: Some(TrendMetric {
            current: 20.0,
            previous: 30.0,
            delta: -10.0,
            delta_pct: -33.3,
            direction: TrendDirection::Improving,
        }),
        complexity: Some(TrendIndicator {
            direction: TrendDirection::Stable,
            summary: "Stable complexity".to_string(),
            files_increased: 1,
            files_decreased: 1,
            avg_cyclomatic_delta: Some(0.0),
            avg_cognitive_delta: Some(0.0),
        }),
    };
    let json_str = serde_json::to_string(&trend).unwrap();
    let back: TrendComparison = serde_json::from_str(&json_str).unwrap();
    assert!(back.baseline_available);
    assert_eq!(back.health.unwrap().direction, TrendDirection::Improving);
}

// =============================================================================
// Format helpers
// =============================================================================

#[test]
fn format_signed_f64_positive() {
    assert_eq!(format_signed_f64(5.0), "+5.00");
}

#[test]
fn format_signed_f64_negative() {
    assert_eq!(format_signed_f64(-3.5), "-3.50");
}

#[test]
fn format_signed_f64_zero() {
    assert_eq!(format_signed_f64(0.0), "0.00");
}

#[test]
fn trend_direction_labels_correct() {
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
fn round_pct_rounds_correctly() {
    assert_eq!(round_pct(0.125), 0.13);
    assert_eq!(round_pct(0.0), 0.0);
    assert_eq!(round_pct(0.999), 1.0);
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn render_json_produces_valid_json() {
    let r = build_receipt(&[make_file_stat("src/lib.rs", 10, 5)]);
    let json_str = render::render_json(&r).unwrap();
    let _val: Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn render_markdown_contains_glass_cockpit_header() {
    let r = build_receipt(&[make_file_stat("src/lib.rs", 10, 5)]);
    let md = render::render_markdown(&r);
    assert!(md.contains("## Glass Cockpit"));
}

#[test]
fn render_markdown_contains_change_surface_section() {
    let r = build_receipt(&[make_file_stat("src/lib.rs", 10, 5)]);
    let md = render::render_markdown(&r);
    assert!(md.contains("### Change Surface"));
}

#[test]
fn render_sections_contains_section_markers() {
    let r = build_receipt(&[make_file_stat("src/lib.rs", 10, 5)]);
    let sections = render::render_sections(&r);
    assert!(sections.contains("<!-- SECTION:COCKPIT -->"));
    assert!(sections.contains("<!-- SECTION:REVIEW_PLAN -->"));
    assert!(sections.contains("<!-- SECTION:RECEIPTS -->"));
}

#[test]
fn render_comment_md_contains_summary() {
    let r = build_receipt(&[make_file_stat("src/lib.rs", 10, 5)]);
    let comment = render::render_comment_md(&r);
    assert!(comment.contains("## Glass Cockpit Summary"));
}

// =============================================================================
// GateStatus serde
// =============================================================================

#[test]
fn gate_status_all_variants_roundtrip() {
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
fn risk_level_all_variants_display() {
    assert_eq!(RiskLevel::Low.to_string(), "low");
    assert_eq!(RiskLevel::Medium.to_string(), "medium");
    assert_eq!(RiskLevel::High.to_string(), "high");
    assert_eq!(RiskLevel::Critical.to_string(), "critical");
}
