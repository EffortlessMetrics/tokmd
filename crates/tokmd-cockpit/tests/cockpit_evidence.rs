//! Deeper tests for cockpit evidence gate evaluation, metric collection,
//! and deterministic ordering.

use tokmd_cockpit::*;

// ============================================================================
// Helpers
// ============================================================================

fn make_stat(path: &str, ins: usize, del: usize) -> FileStat {
    FileStat {
        path: path.to_string(),
        insertions: ins,
        deletions: del,
    }
}

fn minimal_evidence() -> Evidence {
    Evidence {
        overall_status: GateStatus::Pass,
        mutation: MutationGate {
            meta: GateMeta {
                status: GateStatus::Pass,
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
        complexity: None,
    }
}

fn minimal_receipt() -> CockpitReceipt {
    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 1000,
        base_ref: "main".to_string(),
        head_ref: "HEAD".to_string(),
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
            score: 100,
            grade: "A".to_string(),
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
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
        evidence: minimal_evidence(),
        review_plan: vec![],
        trend: None,
    }
}

// ============================================================================
// Evidence gate overall status computation
// ============================================================================

#[test]
fn overall_status_all_pass() {
    let evidence = minimal_evidence();
    let status = compute_overall_gate_status(&evidence);
    assert_eq!(status, GateStatus::Pass);
}

#[test]
fn overall_status_mutation_fail_propagates() {
    let mut evidence = minimal_evidence();
    evidence.mutation.meta.status = GateStatus::Fail;
    let status = compute_overall_gate_status(&evidence);
    assert_eq!(status, GateStatus::Fail);
}

#[test]
fn overall_status_any_fail_beats_pass() {
    let mut evidence = minimal_evidence();
    evidence.complexity = Some(ComplexityGate {
        meta: GateMeta {
            status: GateStatus::Fail,
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
        },
        files_analyzed: 1,
        high_complexity_files: vec![],
        avg_cyclomatic: 20.0,
        max_cyclomatic: 30,
        threshold_exceeded: true,
    });
    let status = compute_overall_gate_status(&evidence);
    assert_eq!(status, GateStatus::Fail);
}

#[test]
fn overall_status_pending_without_fail() {
    let mut evidence = minimal_evidence();
    evidence.mutation.meta.status = GateStatus::Pending;
    let status = compute_overall_gate_status(&evidence);
    assert_eq!(status, GateStatus::Pending);
}

#[test]
fn overall_status_warn_without_fail_or_pending() {
    let mut evidence = minimal_evidence();
    evidence.mutation.meta.status = GateStatus::Warn;
    let status = compute_overall_gate_status(&evidence);
    assert_eq!(status, GateStatus::Warn);
}

#[test]
fn overall_status_all_skipped() {
    let mut evidence = minimal_evidence();
    evidence.mutation.meta.status = GateStatus::Skipped;
    let status = compute_overall_gate_status(&evidence);
    assert_eq!(status, GateStatus::Skipped);
}

#[test]
fn overall_status_fail_trumps_pending_and_warn() {
    let mut evidence = minimal_evidence();
    evidence.mutation.meta.status = GateStatus::Pending;
    evidence.diff_coverage = Some(DiffCoverageGate {
        meta: GateMeta {
            status: GateStatus::Fail,
            source: EvidenceSource::CiArtifact,
            commit_match: CommitMatch::Exact,
            scope: ScopeCoverage {
                relevant: vec![],
                tested: vec![],
                ratio: 0.0,
                lines_relevant: Some(100),
                lines_tested: Some(10),
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        lines_added: 100,
        lines_covered: 10,
        coverage_pct: 0.1,
        uncovered_hunks: vec![],
    });
    let status = compute_overall_gate_status(&evidence);
    assert_eq!(status, GateStatus::Fail);
}

// ============================================================================
// Composition computation edge cases
// ============================================================================

#[test]
fn composition_empty_files() {
    let files: Vec<&str> = vec![];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(comp.test_pct, 0.0);
    assert_eq!(comp.docs_pct, 0.0);
    assert_eq!(comp.config_pct, 0.0);
    assert_eq!(comp.test_ratio, 0.0);
}

#[test]
fn composition_only_tests() {
    let files = vec!["tests/test_foo.rs", "tests/test_bar.rs"];
    let comp = compute_composition(&files);
    assert_eq!(comp.test_pct, 1.0);
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(
        comp.test_ratio, 1.0,
        "test_ratio should be 1.0 when only tests"
    );
}

#[test]
fn composition_mixed_files() {
    let files = vec![
        "src/lib.rs",
        "src/main.rs",
        "tests/test_lib.rs",
        "README.md",
        "Cargo.toml",
    ];
    let comp = compute_composition(&files);
    // 2 code, 1 test, 1 docs, 1 config = 5 total
    assert!((comp.code_pct - 0.4).abs() < 0.01);
    assert!((comp.test_pct - 0.2).abs() < 0.01);
    assert!((comp.docs_pct - 0.2).abs() < 0.01);
    assert!((comp.config_pct - 0.2).abs() < 0.01);
    assert!((comp.test_ratio - 0.5).abs() < 0.01);
}

#[test]
fn composition_unrecognized_extensions_excluded() {
    let files = vec!["image.png", "binary.exe", "data.bin"];
    let comp = compute_composition(&files);
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(comp.test_ratio, 0.0);
}

// ============================================================================
// Contract detection
// ============================================================================

#[test]
fn contracts_no_relevant_files() {
    let files = vec!["src/util.rs", "crates/tokmd-types/src/util.rs"];
    let contracts = detect_contracts(&files);
    assert!(!contracts.api_changed);
    assert!(!contracts.cli_changed);
    assert!(!contracts.schema_changed);
    assert_eq!(contracts.breaking_indicators, 0);
}

#[test]
fn contracts_lib_rs_triggers_api_changed() {
    let files = vec!["crates/tokmd-core/src/lib.rs"];
    let contracts = detect_contracts(&files);
    assert!(contracts.api_changed);
    assert_eq!(contracts.breaking_indicators, 1);
}

#[test]
fn contracts_schema_json_triggers_schema_changed() {
    let files = vec!["docs/schema.json"];
    let contracts = detect_contracts(&files);
    assert!(contracts.schema_changed);
    assert_eq!(contracts.breaking_indicators, 1);
}

#[test]
fn contracts_all_flags_set() {
    let files = vec![
        "crates/tokmd/src/lib.rs",
        "crates/tokmd/src/commands/lang.rs",
        "docs/schema.json",
    ];
    let contracts = detect_contracts(&files);
    assert!(contracts.api_changed);
    assert!(contracts.cli_changed);
    assert!(contracts.schema_changed);
    assert_eq!(contracts.breaking_indicators, 2); // api + schema
}

// ============================================================================
// Code health computation
// ============================================================================

#[test]
fn code_health_no_files() {
    let stats: Vec<FileStat> = vec![];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);
    assert_eq!(health.score, 100);
    assert_eq!(health.grade, "A");
    assert_eq!(health.large_files_touched, 0);
    assert_eq!(health.avg_file_size, 0);
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Low);
    assert!(health.warnings.is_empty());
}

#[test]
fn code_health_large_files_reduce_score() {
    let stats = vec![
        make_stat("big.rs", 400, 200), // 600 lines > 500
        make_stat("small.rs", 10, 5),  // 15 lines
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);
    assert_eq!(health.large_files_touched, 1);
    assert_eq!(health.score, 90); // 100 - 10
    assert_eq!(health.grade, "A");
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Medium);
    assert_eq!(health.warnings.len(), 1);
    assert_eq!(health.warnings[0].warning_type, WarningType::LargeFile);
}

#[test]
fn code_health_breaking_contracts_reduce_score() {
    let stats = vec![make_stat("src/lib.rs", 10, 5)];
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: true,
        breaking_indicators: 2,
    };
    let health = compute_code_health(&stats, &contracts);
    // 100 - 0 (no large files) - 20 (breaking) = 80
    assert_eq!(health.score, 80);
    assert_eq!(health.grade, "B");
}

#[test]
fn code_health_massive_change_critical_complexity() {
    let stats: Vec<FileStat> = (0..6)
        .map(|i| make_stat(&format!("big_{i}.rs"), 300, 300))
        .collect();
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);
    assert_eq!(health.large_files_touched, 6);
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Critical);
    // 100 - 60 = 40
    assert_eq!(health.score, 40);
    assert_eq!(health.grade, "F");
}

// ============================================================================
// Risk computation
// ============================================================================

#[test]
fn risk_no_files() {
    let stats: Vec<FileStat> = vec![];
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
fn risk_hotspot_detection() {
    let stats = vec![
        make_stat("core.rs", 200, 200), // 400 > 300 → hotspot
        make_stat("util.rs", 5, 5),     // 10 < 300 → not hotspot
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);
    let risk = compute_risk(&stats, &contracts, &health);
    assert_eq!(risk.hotspots_touched.len(), 1);
    assert_eq!(risk.hotspots_touched[0], "core.rs");
}

#[test]
fn risk_score_capped_at_100() {
    let stats: Vec<FileStat> = (0..20)
        .map(|i| make_stat(&format!("hotspot_{i}.rs"), 200, 200))
        .collect();
    let contracts = Contracts {
        api_changed: true,
        cli_changed: true,
        schema_changed: true,
        breaking_indicators: 2,
    };
    let health = compute_code_health(&stats, &contracts);
    let risk = compute_risk(&stats, &contracts, &health);
    assert!(risk.score <= 100);
    assert_eq!(risk.level, RiskLevel::Critical);
}

// ============================================================================
// Review plan generation
// ============================================================================

#[test]
fn review_plan_empty() {
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

#[test]
fn review_plan_sorted_by_priority() {
    let stats = vec![
        make_stat("small.rs", 10, 5),   // 15 lines → priority 3
        make_stat("medium.rs", 40, 20), // 60 lines → priority 2
        make_stat("big.rs", 150, 100),  // 250 lines → priority 1
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let plan = generate_review_plan(&stats, &contracts);
    assert_eq!(plan.len(), 3);
    assert_eq!(plan[0].priority, 1);
    assert_eq!(plan[0].path, "big.rs");
    assert_eq!(plan[1].priority, 2);
    assert_eq!(plan[2].priority, 3);
}

#[test]
fn review_plan_complexity_scales() {
    let stats = vec![
        make_stat("tiny.rs", 5, 5),     // 10 lines → complexity 1
        make_stat("mid.rs", 60, 50),    // 110 lines → complexity 3
        make_stat("huge.rs", 200, 200), // 400 lines → complexity 5
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let plan = generate_review_plan(&stats, &contracts);
    let complexities: Vec<u8> = plan.iter().filter_map(|i| i.complexity).collect();
    assert!(complexities.contains(&1));
    assert!(complexities.contains(&3));
    assert!(complexities.contains(&5));
}

// ============================================================================
// Trend computation
// ============================================================================

#[test]
fn metric_trend_improving_health() {
    let trend = compute_metric_trend(90.0, 80.0, true);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert!((trend.delta - 10.0).abs() < 0.01);
    assert!((trend.delta_pct - 12.5).abs() < 0.1);
}

#[test]
fn metric_trend_degrading_health() {
    let trend = compute_metric_trend(70.0, 85.0, true);
    assert_eq!(trend.direction, TrendDirection::Degrading);
    assert!(trend.delta < 0.0);
}

#[test]
fn metric_trend_stable_within_threshold() {
    let trend = compute_metric_trend(80.0, 80.5, true);
    assert_eq!(trend.direction, TrendDirection::Stable);
}

#[test]
fn metric_trend_risk_lower_is_better() {
    // Risk: lower is better, so a decrease is improving
    let trend = compute_metric_trend(20.0, 30.0, false);
    assert_eq!(trend.direction, TrendDirection::Improving);

    // Risk: increase is degrading
    let trend = compute_metric_trend(40.0, 25.0, false);
    assert_eq!(trend.direction, TrendDirection::Degrading);
}

#[test]
fn metric_trend_zero_baseline() {
    let trend = compute_metric_trend(10.0, 0.0, true);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert!((trend.delta_pct - 100.0).abs() < 0.01);
}

#[test]
fn metric_trend_both_zero() {
    let trend = compute_metric_trend(0.0, 0.0, true);
    assert_eq!(trend.direction, TrendDirection::Stable);
    assert!((trend.delta_pct).abs() < 0.01);
}

#[test]
fn complexity_trend_stable() {
    let current = minimal_receipt();
    let baseline = minimal_receipt();
    let trend = compute_complexity_trend(&current, &baseline);
    assert_eq!(trend.direction, TrendDirection::Stable);
    assert!(trend.summary.contains("stable"));
}

#[test]
fn complexity_trend_improving() {
    let mut current = minimal_receipt();
    let mut baseline = minimal_receipt();
    baseline.evidence.complexity = Some(ComplexityGate {
        meta: GateMeta {
            status: GateStatus::Pass,
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
        },
        files_analyzed: 5,
        high_complexity_files: vec![],
        avg_cyclomatic: 10.0,
        max_cyclomatic: 15,
        threshold_exceeded: false,
    });
    current.evidence.complexity = Some(ComplexityGate {
        meta: GateMeta {
            status: GateStatus::Pass,
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
        },
        files_analyzed: 5,
        high_complexity_files: vec![],
        avg_cyclomatic: 5.0,
        max_cyclomatic: 10,
        threshold_exceeded: false,
    });
    let trend = compute_complexity_trend(&current, &baseline);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert!(trend.summary.contains("decreased"));
}

// ============================================================================
// Utility helpers
// ============================================================================

#[test]
fn format_signed_positive() {
    assert_eq!(format_signed_f64(3.14), "+3.14");
}

#[test]
fn format_signed_negative() {
    assert_eq!(format_signed_f64(-2.71), "-2.71");
}

#[test]
fn format_signed_zero() {
    assert_eq!(format_signed_f64(0.0), "0.00");
}

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
    // First should be lowest bar, last should be highest
    assert!(chars[0] < chars[2]);
}

#[test]
fn round_pct_basic() {
    assert!((round_pct(0.333) - 0.33).abs() < 0.001);
    assert!((round_pct(0.666) - 0.67).abs() < 0.001);
    assert!((round_pct(1.0) - 1.0).abs() < 0.001);
}

// ============================================================================
// Rendering: JSON round-trip
// ============================================================================

#[test]
fn render_json_roundtrip() {
    let receipt = minimal_receipt();
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(back.mode, "cockpit");
    assert_eq!(back.evidence.overall_status, GateStatus::Pass);
}

#[test]
fn render_json_deterministic() {
    let receipt = minimal_receipt();
    let json1 = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let json2 = tokmd_cockpit::render::render_json(&receipt).unwrap();
    assert_eq!(json1, json2, "JSON rendering must be deterministic");
}

// ============================================================================
// Rendering: Markdown contains expected sections
// ============================================================================

#[test]
fn render_markdown_contains_sections() {
    let receipt = minimal_receipt();
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    assert!(md.contains("## Glass Cockpit"));
    assert!(md.contains("### Summary"));
    assert!(md.contains("### Change Surface"));
    assert!(md.contains("### Composition"));
    assert!(md.contains("### Contracts"));
    assert!(md.contains("### Code Health"));
    assert!(md.contains("### Risk"));
    assert!(md.contains("### Evidence Gates"));
    assert!(md.contains("### Review Plan"));
}

#[test]
fn render_markdown_with_trend() {
    let mut receipt = minimal_receipt();
    receipt.trend = Some(TrendComparison {
        baseline_available: true,
        baseline_path: Some("baseline.json".to_string()),
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
        complexity: None,
    });
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    assert!(md.contains("### Summary Comparison"));
    assert!(md.contains("### Trend"));
    assert!(md.contains("baseline.json"));
}

// ============================================================================
// Rendering: Sections format
// ============================================================================

#[test]
fn render_sections_contains_markers() {
    let receipt = minimal_receipt();
    let s = tokmd_cockpit::render::render_sections(&receipt);
    assert!(s.contains("<!-- SECTION:COCKPIT -->"));
    assert!(s.contains("<!-- SECTION:REVIEW_PLAN -->"));
    assert!(s.contains("<!-- SECTION:RECEIPTS -->"));
}

// ============================================================================
// Rendering: Comment markdown
// ============================================================================

#[test]
fn render_comment_md_basic() {
    let receipt = minimal_receipt();
    let md = tokmd_cockpit::render::render_comment_md(&receipt);
    assert!(md.contains("## Glass Cockpit Summary"));
    assert!(md.contains("Evidence gates"));
}

#[test]
fn render_comment_md_with_contract_changes() {
    let mut receipt = minimal_receipt();
    receipt.contracts.api_changed = true;
    receipt.contracts.breaking_indicators = 1;
    let md = tokmd_cockpit::render::render_comment_md(&receipt);
    assert!(md.contains("Contract changes"));
    assert!(md.contains("API contract changed"));
}

#[test]
fn render_comment_md_priority_items() {
    let mut receipt = minimal_receipt();
    receipt.review_plan = vec![
        ReviewItem {
            path: "important.rs".to_string(),
            reason: "many lines changed".to_string(),
            priority: 1,
            complexity: Some(5),
            lines_changed: Some(500),
        },
        ReviewItem {
            path: "trivial.rs".to_string(),
            reason: "minor change".to_string(),
            priority: 3,
            complexity: Some(1),
            lines_changed: Some(5),
        },
    ];
    let md = tokmd_cockpit::render::render_comment_md(&receipt);
    assert!(md.contains("Priority review items"));
    assert!(md.contains("important.rs"));
    // priority 3 should NOT appear in priority items (filter is <= 2)
    assert!(!md.contains("trivial.rs"));
}

// ============================================================================
// Artifact writing
// ============================================================================

#[test]
fn write_artifacts_creates_files() {
    let dir = tempfile::tempdir().unwrap();
    let receipt = minimal_receipt();
    tokmd_cockpit::render::write_artifacts(dir.path(), &receipt).unwrap();

    assert!(dir.path().join("cockpit.json").exists());
    assert!(dir.path().join("report.json").exists());
    assert!(dir.path().join("comment.md").exists());

    // Verify cockpit.json is valid JSON
    let json_str = std::fs::read_to_string(dir.path().join("cockpit.json")).unwrap();
    let _: CockpitReceipt = serde_json::from_str(&json_str).unwrap();

    // Verify report.json is valid sensor report envelope
    let report_str = std::fs::read_to_string(dir.path().join("report.json")).unwrap();
    let report: serde_json::Value = serde_json::from_str(&report_str).unwrap();
    assert!(report.get("verdict").is_some());
}

// ============================================================================
// Deterministic ordering invariants
// ============================================================================

#[test]
fn review_plan_ordering_is_deterministic() {
    let stats = vec![
        make_stat("z_file.rs", 100, 50),
        make_stat("a_file.rs", 100, 50),
        make_stat("m_file.rs", 100, 50),
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let plan1 = generate_review_plan(&stats, &contracts);
    let plan2 = generate_review_plan(&stats, &contracts);
    let paths1: Vec<&str> = plan1.iter().map(|i| i.path.as_str()).collect();
    let paths2: Vec<&str> = plan2.iter().map(|i| i.path.as_str()).collect();
    assert_eq!(paths1, paths2, "Review plan order must be deterministic");
}

#[test]
fn json_output_deterministic_across_calls() {
    let receipt = minimal_receipt();
    let j1 = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let j2 = tokmd_cockpit::render::render_json(&receipt).unwrap();
    assert_eq!(j1, j2);
}
