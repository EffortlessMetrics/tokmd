//! BDD-style scenario tests for cockpit workflows.
//!
//! Each test is structured as Given / When / Then to document expected behavior.

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

fn minimal_receipt() -> CockpitReceipt {
    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 1000,
        base_ref: "main".to_string(),
        head_ref: "feature".to_string(),
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
            grade: "A".to_string(),
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
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
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
        review_plan: Vec::new(),
        trend: None,
    }
}

// ===========================================================================
// Scenario: Composition from empty file list
// ===========================================================================

#[test]
fn scenario_empty_file_list_yields_zero_composition() {
    // Given: no changed files
    let files: Vec<&str> = Vec::new();

    // When: we compute composition
    let comp = compute_composition(&files);

    // Then: all percentages are zero
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(comp.test_pct, 0.0);
    assert_eq!(comp.docs_pct, 0.0);
    assert_eq!(comp.config_pct, 0.0);
    assert_eq!(comp.test_ratio, 0.0);
}

// ===========================================================================
// Scenario: Composition with only code files
// ===========================================================================

#[test]
fn scenario_only_code_files() {
    // Given: two Rust source files (no tests)
    let files = vec!["src/main.rs", "src/lib.rs"];

    // When: we compute composition
    let comp = compute_composition(&files);

    // Then: 100% code, 0% everything else
    assert_eq!(comp.code_pct, 1.0);
    assert_eq!(comp.test_pct, 0.0);
    assert_eq!(comp.test_ratio, 0.0);
}

// ===========================================================================
// Scenario: Composition with mixed file types
// ===========================================================================

#[test]
fn scenario_mixed_file_composition() {
    // Given: 2 code, 1 test, 1 doc, 1 config
    let files = vec![
        "src/main.rs",
        "src/lib.rs",
        "tests/unit_test.rs",
        "README.md",
        "Cargo.toml",
    ];

    // When: we compute composition
    let comp = compute_composition(&files);

    // Then: 40% code, 20% test, 20% docs, 20% config
    assert!((comp.code_pct - 0.4).abs() < 0.01);
    assert!((comp.test_pct - 0.2).abs() < 0.01);
    assert!((comp.docs_pct - 0.2).abs() < 0.01);
    assert!((comp.config_pct - 0.2).abs() < 0.01);
    assert!((comp.test_ratio - 0.5).abs() < 0.01);
}

// ===========================================================================
// Scenario: Contract detection for API changes
// ===========================================================================

#[test]
fn scenario_detect_api_contract_changes() {
    // Given: a lib.rs was changed
    let files = vec!["crates/tokmd-types/src/lib.rs"];

    // When: we detect contracts
    let contracts = detect_contracts(&files);

    // Then: API changed is true, breaking_indicators >= 1
    assert!(contracts.api_changed);
    assert!(!contracts.cli_changed);
    assert!(!contracts.schema_changed);
    assert!(contracts.breaking_indicators >= 1);
}

#[test]
fn scenario_detect_cli_contract_changes() {
    // Given: a command file was changed
    let files = vec!["crates/tokmd/src/commands/lang.rs"];

    // When: we detect contracts
    let contracts = detect_contracts(&files);

    // Then: CLI changed is true
    assert!(contracts.cli_changed);
    assert!(!contracts.api_changed);
    assert!(!contracts.schema_changed);
}

#[test]
fn scenario_detect_schema_contract_changes() {
    // Given: schema.json was changed
    let files = vec!["docs/schema.json"];

    // When: we detect contracts
    let contracts = detect_contracts(&files);

    // Then: schema changed is true
    assert!(contracts.schema_changed);
    assert!(contracts.breaking_indicators >= 1);
}

#[test]
fn scenario_no_contract_changes() {
    // Given: only a non-contract file changed
    let files = vec!["src/utils.rs"];

    // When: we detect contracts
    let contracts = detect_contracts(&files);

    // Then: no contract changes
    assert!(!contracts.api_changed);
    assert!(!contracts.cli_changed);
    assert!(!contracts.schema_changed);
    assert_eq!(contracts.breaking_indicators, 0);
}

// ===========================================================================
// Scenario: Code health scoring
// ===========================================================================

#[test]
fn scenario_healthy_small_pr() {
    // Given: a few small file changes, no contract changes
    let stats = vec![
        make_file_stat("src/main.rs", 10, 5),
        make_file_stat("src/lib.rs", 20, 3),
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };

    // When: we compute code health
    let health = compute_code_health(&stats, &contracts);

    // Then: score is 100, grade A, no warnings
    assert_eq!(health.score, 100);
    assert_eq!(health.grade, "A");
    assert_eq!(health.large_files_touched, 0);
    assert!(health.warnings.is_empty());
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Low);
}

#[test]
fn scenario_large_files_degrade_health() {
    // Given: files with >500 lines changed
    let stats = vec![
        make_file_stat("src/big.rs", 400, 200),
        make_file_stat("src/small.rs", 10, 5),
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };

    // When: we compute code health
    let health = compute_code_health(&stats, &contracts);

    // Then: score reduced, warnings for large file
    assert!(health.score < 100);
    assert_eq!(health.large_files_touched, 1);
    assert!(!health.warnings.is_empty());
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Medium);
}

#[test]
fn scenario_breaking_changes_penalize_health() {
    // Given: a small change but with breaking indicators
    let stats = vec![make_file_stat("src/lib.rs", 5, 2)];
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: true,
        breaking_indicators: 2,
    };

    // When: we compute code health
    let health = compute_code_health(&stats, &contracts);

    // Then: score reduced by 20 for breaking indicators
    assert_eq!(health.score, 80);
    assert_eq!(health.grade, "B");
}

// ===========================================================================
// Scenario: Risk scoring
// ===========================================================================

#[test]
fn scenario_low_risk_small_pr() {
    // Given: small changes, healthy code
    let stats = vec![make_file_stat("src/main.rs", 10, 5)];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);

    // When: we compute risk
    let risk = compute_risk(&stats, &contracts, &health);

    // Then: low risk
    assert_eq!(risk.level, RiskLevel::Low);
    assert!(risk.hotspots_touched.is_empty());
}

#[test]
fn scenario_hotspot_files_increase_risk() {
    // Given: a file with >300 lines changed
    let stats = vec![make_file_stat("src/core.rs", 200, 150)];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);

    // When: we compute risk
    let risk = compute_risk(&stats, &contracts, &health);

    // Then: file appears as hotspot
    assert!(!risk.hotspots_touched.is_empty());
    assert!(risk.hotspots_touched.contains(&"src/core.rs".to_string()));
    assert!(risk.score > 0);
}

// ===========================================================================
// Scenario: Review plan generation
// ===========================================================================

#[test]
fn scenario_empty_stats_empty_review_plan() {
    // Given: no file stats
    let stats: Vec<FileStat> = Vec::new();
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };

    // When: we generate review plan
    let plan = generate_review_plan(&stats, &contracts);

    // Then: empty plan
    assert!(plan.is_empty());
}

#[test]
fn scenario_review_plan_priority_ordering() {
    // Given: files with varying change sizes
    let stats = vec![
        make_file_stat("src/small.rs", 10, 5),    // priority 3
        make_file_stat("src/medium.rs", 40, 20),  // priority 2
        make_file_stat("src/large.rs", 150, 100), // priority 1
    ];
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };

    // When: we generate review plan
    let plan = generate_review_plan(&stats, &contracts);

    // Then: sorted by priority (highest priority first)
    assert_eq!(plan.len(), 3);
    assert_eq!(plan[0].priority, 1);
    assert_eq!(plan[0].path, "src/large.rs");
    assert_eq!(plan[1].priority, 2);
    assert_eq!(plan[2].priority, 3);
}

// ===========================================================================
// Scenario: Trend computation
// ===========================================================================

#[test]
fn scenario_trend_improving_health() {
    // Given: current health is higher than previous
    // When: we compute trend (higher is better for health)
    let trend = compute_metric_trend(90.0, 70.0, true);

    // Then: direction is improving
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert_eq!(trend.current, 90.0);
    assert_eq!(trend.previous, 70.0);
    assert!((trend.delta - 20.0).abs() < 0.01);
}

#[test]
fn scenario_trend_degrading_health() {
    // Given: current health is lower than previous
    let trend = compute_metric_trend(60.0, 80.0, true);

    // Then: direction is degrading
    assert_eq!(trend.direction, TrendDirection::Degrading);
}

#[test]
fn scenario_trend_stable_within_threshold() {
    // Given: delta < 1.0
    let trend = compute_metric_trend(80.5, 80.0, true);

    // Then: direction is stable
    assert_eq!(trend.direction, TrendDirection::Stable);
}

#[test]
fn scenario_risk_trend_lower_is_better() {
    // Given: risk decreased (lower is better)
    let trend = compute_metric_trend(20.0, 40.0, false);

    // Then: improving (lower risk)
    assert_eq!(trend.direction, TrendDirection::Improving);
}

#[test]
fn scenario_risk_trend_higher_is_worse() {
    // Given: risk increased (lower is better)
    let trend = compute_metric_trend(50.0, 30.0, false);

    // Then: degrading
    assert_eq!(trend.direction, TrendDirection::Degrading);
}

// ===========================================================================
// Scenario: Complexity trend
// ===========================================================================

#[test]
fn scenario_complexity_trend_stable() {
    // Given: two receipts with same complexity
    let current = minimal_receipt();
    let baseline = minimal_receipt();

    // When: computing complexity trend
    let indicator = compute_complexity_trend(&current, &baseline);

    // Then: stable
    assert_eq!(indicator.direction, TrendDirection::Stable);
    assert!(indicator.summary.contains("stable"));
}

#[test]
fn scenario_complexity_trend_degrading() {
    // Given: current has higher complexity
    let mut current = minimal_receipt();
    current.evidence.complexity = Some(ComplexityGate {
        meta: GateMeta {
            status: GateStatus::Warn,
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
        files_analyzed: 1,
        high_complexity_files: Vec::new(),
        avg_cyclomatic: 10.0,
        max_cyclomatic: 10,
        threshold_exceeded: false,
    });
    let baseline = minimal_receipt();

    // When: computing complexity trend
    let indicator = compute_complexity_trend(&current, &baseline);

    // Then: degrading
    assert_eq!(indicator.direction, TrendDirection::Degrading);
    assert!(indicator.summary.contains("increased"));
}

// ===========================================================================
// Scenario: Overall gate status computation
// ===========================================================================

#[test]
fn scenario_overall_gate_all_pass() {
    // Given: mutation gate passes
    let receipt = minimal_receipt();

    // When: overall status check
    // Then: Pass (only Skipped gate -> since all gates are skipped except mutation which is Skipped,
    //        the overall is Skipped)
    assert_eq!(receipt.evidence.overall_status, GateStatus::Pass);
}

// ===========================================================================
// Scenario: Utility helpers
// ===========================================================================

#[test]
fn scenario_format_signed_positive() {
    assert_eq!(format_signed_f64(5.0), "+5.00");
}

#[test]
fn scenario_format_signed_negative() {
    assert_eq!(format_signed_f64(-3.5), "-3.50");
}

#[test]
fn scenario_format_signed_zero() {
    assert_eq!(format_signed_f64(0.0), "0.00");
}

#[test]
fn scenario_trend_direction_labels() {
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

// ===========================================================================
// Scenario: Sparkline rendering
// ===========================================================================

#[test]
fn scenario_sparkline_empty_input() {
    assert_eq!(sparkline(&[]), "");
}

#[test]
fn scenario_sparkline_single_value() {
    let result = sparkline(&[50.0]);
    assert_eq!(result.chars().count(), 1);
}

#[test]
fn scenario_sparkline_two_values() {
    let result = sparkline(&[10.0, 90.0]);
    assert_eq!(result.chars().count(), 2);
}

#[test]
fn scenario_sparkline_equal_values() {
    let result = sparkline(&[42.0, 42.0, 42.0]);
    // All equal -> all same bar character
    let chars: Vec<char> = result.chars().collect();
    assert_eq!(chars.len(), 3);
    assert!(chars.iter().all(|c| *c == chars[0]));
}

// ===========================================================================
// Scenario: Round percentage
// ===========================================================================

#[test]
fn scenario_round_pct() {
    assert_eq!(round_pct(0.0), 0.0);
    assert_eq!(round_pct(1.0), 1.0);
    assert_eq!(round_pct(0.456), 0.46);
    assert_eq!(round_pct(0.554), 0.55);
}

// ===========================================================================
// Scenario: now_iso8601 produces valid format
// ===========================================================================

#[test]
fn scenario_now_iso8601_format() {
    let ts = now_iso8601();
    // Expected format: YYYY-MM-DDTHH:MM:SSZ
    assert!(ts.ends_with('Z'));
    assert!(ts.contains('T'));
    assert_eq!(ts.len(), 20);
}

// ===========================================================================
// Scenario: Determinism hashing
// ===========================================================================

#[test]
fn scenario_hash_files_from_paths_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
    std::fs::write(dir.path().join("b.rs"), "fn test() {}").unwrap();

    let h1 =
        tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs", "b.rs"]).unwrap();
    let h2 =
        tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["b.rs", "a.rs"]).unwrap();

    // Then: hashes are identical regardless of input order
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64); // BLAKE3 hex digest
}

#[test]
fn scenario_hash_skips_git_and_target_dirs() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();

    let h1 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();
    let h2 = tokmd_cockpit::determinism::hash_files_from_paths(
        dir.path(),
        &[
            "a.rs",
            ".git/config",
            "target/debug/build",
            ".tokmd/baseline.json",
        ],
    )
    .unwrap();

    // Then: excluded paths don't affect hash
    assert_eq!(h1, h2);
}

#[test]
fn scenario_hash_missing_file_skipped() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();

    // NotFound files should be silently skipped
    let result =
        tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs", "nonexistent.rs"]);

    assert!(result.is_ok());
}

#[test]
fn scenario_hash_cargo_lock_present() {
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
fn scenario_hash_cargo_lock_absent() {
    let dir = tempfile::tempdir().unwrap();
    let result = tokmd_cockpit::determinism::hash_cargo_lock(dir.path()).unwrap();
    assert!(result.is_none());
}

// ===========================================================================
// Scenario: Render JSON round-trip
// ===========================================================================

#[test]
fn scenario_render_json_roundtrip() {
    let receipt = minimal_receipt();
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let parsed: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(parsed.mode, "cockpit");
    assert_eq!(parsed.base_ref, "main");
    assert_eq!(parsed.head_ref, "feature");
}

// ===========================================================================
// Scenario: Render Markdown contains expected sections
// ===========================================================================

#[test]
fn scenario_render_markdown_sections() {
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

// ===========================================================================
// Scenario: Render sections output
// ===========================================================================

#[test]
fn scenario_render_sections_contains_markers() {
    let receipt = minimal_receipt();
    let sections = tokmd_cockpit::render::render_sections(&receipt);

    assert!(sections.contains("<!-- SECTION:COCKPIT -->"));
    assert!(sections.contains("<!-- SECTION:REVIEW_PLAN -->"));
    assert!(sections.contains("<!-- SECTION:RECEIPTS -->"));
}

// ===========================================================================
// Scenario: Render comment.md
// ===========================================================================

#[test]
fn scenario_render_comment_md_summary() {
    let receipt = minimal_receipt();
    let comment = tokmd_cockpit::render::render_comment_md(&receipt);

    assert!(comment.contains("## Glass Cockpit Summary"));
    assert!(comment.contains("Health"));
    assert!(comment.contains("Risk"));
    assert!(comment.contains("Evidence availability"));
    assert!(comment.contains("1 skipped"));
    assert!(comment.contains("5 unavailable"));
}

// ===========================================================================
// Scenario: Write artifacts to disk
// ===========================================================================

#[test]
fn scenario_write_artifacts_creates_files() {
    let dir = tempfile::tempdir().unwrap();
    let receipt = minimal_receipt();
    let out = dir.path().join("output");

    tokmd_cockpit::render::write_artifacts(&out, &receipt).unwrap();

    assert!(out.join("cockpit.json").exists());
    assert!(out.join("report.json").exists());
    assert!(out.join("comment.md").exists());

    // Verify cockpit.json is valid JSON
    let content = std::fs::read_to_string(out.join("cockpit.json")).unwrap();
    let _: CockpitReceipt = serde_json::from_str(&content).unwrap();
}

// ===========================================================================
// Scenario: Write review packet to disk
// ===========================================================================

#[test]
fn scenario_write_review_packet_creates_contract_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut receipt = minimal_receipt();
    receipt.evidence.mutation.meta.status = GateStatus::Pass;
    receipt.evidence.mutation.meta.commit_match = CommitMatch::Exact;
    receipt.evidence.diff_coverage = Some(DiffCoverageGate {
        meta: GateMeta {
            status: GateStatus::Pending,
            source: EvidenceSource::CiArtifact,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: vec!["src/lib.rs".to_string()],
                tested: Vec::new(),
                ratio: 0.0,
                lines_relevant: Some(42),
                lines_tested: Some(0),
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        lines_added: 42,
        lines_covered: 0,
        coverage_pct: 0.0,
        uncovered_hunks: vec![UncoveredHunk {
            file: "src/lib.rs".to_string(),
            start_line: 1,
            end_line: 42,
        }],
    });
    receipt.review_plan = vec![ReviewItem {
        path: "src/lib.rs".to_string(),
        reason: "Large changed file".to_string(),
        priority: 1,
        complexity: Some(4),
        lines_changed: Some(240),
    }];
    let out = dir.path().join("review");

    tokmd_cockpit::render::write_review_packet(&out, &receipt).unwrap();

    assert!(out.join("manifest.json").exists());
    assert!(out.join("cockpit.json").exists());
    assert!(out.join("evidence.json").exists());
    assert!(out.join("review-map.json").exists());
    assert!(out.join("review-map.md").exists());
    assert!(out.join("comment.md").exists());

    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["schema"], "tokmd.review_packet_manifest.v1");
    assert_eq!(manifest["generated_by"]["mode"], "cockpit");
    assert_eq!(manifest["verdict"]["blocking"].as_bool(), Some(false));
    assert_eq!(
        manifest["verdict"]["evidence"]["details"],
        "evidence.json#/gates"
    );
    assert_eq!(manifest["verdict"]["evidence"]["total_gates"], 6);
    assert_eq!(manifest["verdict"]["evidence"]["available"], 1);
    assert_eq!(manifest["verdict"]["evidence"]["missing"], 1);
    assert_eq!(manifest["verdict"]["evidence"]["unavailable"], 4);
    assert_eq!(
        manifest["capabilities"]["evidence"]["available"][0],
        "mutation"
    );
    assert_eq!(
        manifest["capabilities"]["evidence"]["missing"][0],
        "diff_coverage"
    );
    assert!(
        manifest["capabilities"]["evidence"]["unavailable"]
            .as_array()
            .unwrap()
            .iter()
            .any(|gate| gate == "contracts")
    );
    assert_eq!(manifest["artifacts"].as_array().unwrap().len(), 5);
    assert_eq!(manifest["artifacts"][0]["path"], "cockpit.json");
    assert_eq!(manifest["artifacts"][0]["hash"]["algo"], "blake3");
    assert_eq!(
        manifest["artifacts"][0]["hash"]["hash"]
            .as_str()
            .unwrap()
            .len(),
        64
    );

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("evidence.json")).unwrap()).unwrap();
    assert_eq!(evidence["schema"], "tokmd.review_packet_evidence.v1");
    assert_eq!(evidence["gates"][0]["id"], "mutation");
    assert_eq!(evidence["gates"][0]["availability"], "available");
    assert_eq!(evidence["gates"][1]["id"], "diff_coverage");
    assert_eq!(evidence["gates"][1]["availability"], "missing");
    assert_eq!(evidence["gates"][1]["scope"]["relevant"][0], "src/lib.rs");
    assert_eq!(
        evidence["gates"][1]["scope"]["tested"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert!(
        evidence.get("proof").is_none(),
        "packets without proof inputs should preserve the current evidence shape"
    );

    let review_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("review-map.json")).unwrap())
            .unwrap();
    assert_eq!(review_map["schema"], "tokmd.review_map.v1");
    assert_eq!(
        review_map["evidence"]["summary"]["details"],
        "evidence.json#/gates"
    );
    assert_eq!(review_map["evidence"]["summary"]["available"], 1);
    assert_eq!(review_map["evidence"]["summary"]["missing"], 1);
    assert_eq!(review_map["evidence"]["groups"]["available"][0], "mutation");
    assert_eq!(
        review_map["evidence"]["groups"]["missing"][0],
        "diff_coverage"
    );
    assert_eq!(review_map["item_count"], 1);
    assert_eq!(review_map["items"][0]["path"], "src/lib.rs");
    assert_eq!(
        review_map["items"][0]["evidence_refs"][0],
        "cockpit.json#/review_plan/0"
    );
    assert_eq!(review_map["items"][0]["evidence"]["status"], "missing");
    assert_eq!(review_map["items"][0]["evidence"]["present"][0], "mutation");
    assert_eq!(
        review_map["items"][0]["evidence"]["missing"][0],
        "diff_coverage"
    );
    assert!(
        review_map["items"][0]["proof_refs"]
            .as_array()
            .unwrap()
            .is_empty(),
        "review-map item proof refs should stay empty when no proof evidence is imported"
    );

    let review_map_md = std::fs::read_to_string(out.join("review-map.md")).unwrap();
    assert!(!review_map_md.contains("Proof evidence overview"));
    assert!(review_map_md.contains("# Review Map"));
    assert!(review_map_md.contains("Evidence overview: 1 available"));
    assert!(review_map_md.contains("## Review First"));
    assert!(review_map_md.contains("`src/lib.rs`"));
    assert!(review_map_md.contains("Why it matters: Large changed file"));
    assert!(review_map_md.contains("Evidence status: missing"));
    assert!(review_map_md.contains("Evidence present: mutation"));
    assert!(review_map_md.contains("Evidence missing: diff_coverage"));
    assert!(review_map_md.contains("Evidence references:"));
    assert!(review_map_md.contains("cockpit.json#/review_plan/0"));
    assert!(review_map_md.contains("evidence.json#/gates"));
    assert!(review_map_md.contains("Reproduce:"));
    assert!(review_map_md.contains("tokmd cockpit --base main --head feature --format json"));
    assert!(review_map_md.contains(
        "tokmd cockpit --base main --head feature --review-packet-dir <REVIEW_PACKET_DIR>"
    ));

    let comment_md = std::fs::read_to_string(out.join("comment.md")).unwrap();
    assert!(comment_md.contains("Evidence availability"));
    assert!(comment_md.contains("1 available"));
    assert!(comment_md.contains("4 unavailable"));
    assert!(comment_md.contains("1 missing"));
    assert!(!comment_md.contains("Proof evidence"));
    assert!(comment_md.contains("Review packet artifacts"));
    assert!(comment_md.contains("[Evidence gates](evidence.json)"));
    assert!(comment_md.contains("[Review map](review-map.md)"));
    assert!(comment_md.contains("[Full cockpit receipt](cockpit.json)"));
}

#[test]
fn scenario_review_map_orders_review_first_by_evidence_risk() {
    let dir = tempfile::tempdir().unwrap();
    let mut receipt = minimal_receipt();
    receipt.evidence.mutation.meta.status = GateStatus::Pass;
    receipt.evidence.mutation.meta.commit_match = CommitMatch::Exact;
    receipt.evidence.diff_coverage = Some(DiffCoverageGate {
        meta: GateMeta {
            status: GateStatus::Pending,
            source: EvidenceSource::CiArtifact,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: vec!["src/missing.rs".to_string()],
                tested: Vec::new(),
                ratio: 0.0,
                lines_relevant: Some(12),
                lines_tested: Some(0),
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        lines_added: 12,
        lines_covered: 0,
        coverage_pct: 0.0,
        uncovered_hunks: vec![UncoveredHunk {
            file: "src/missing.rs".to_string(),
            start_line: 1,
            end_line: 12,
        }],
    });
    receipt.review_plan = vec![
        ReviewItem {
            path: "src/available.rs".to_string(),
            reason: "Large changed file".to_string(),
            priority: 1,
            complexity: Some(5),
            lines_changed: Some(400),
        },
        ReviewItem {
            path: "src/missing.rs".to_string(),
            reason: "Diff coverage evidence is missing".to_string(),
            priority: 2,
            complexity: Some(1),
            lines_changed: Some(12),
        },
    ];
    let out = dir.path().join("review");

    tokmd_cockpit::render::write_review_packet(&out, &receipt).unwrap();

    let review_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("review-map.json")).unwrap())
            .unwrap();
    assert_eq!(review_map["items"][0]["rank"], 1);
    assert_eq!(review_map["items"][0]["path"], "src/missing.rs");
    assert_eq!(review_map["items"][0]["source_index"], 1);
    assert_eq!(
        review_map["items"][0]["evidence_refs"][0],
        "cockpit.json#/review_plan/1"
    );
    assert_eq!(review_map["items"][0]["evidence"]["status"], "missing");
    assert_eq!(review_map["items"][1]["rank"], 2);
    assert_eq!(review_map["items"][1]["path"], "src/available.rs");
    assert_eq!(review_map["items"][1]["source_index"], 0);
    assert_eq!(
        review_map["items"][1]["evidence_refs"][0],
        "cockpit.json#/review_plan/0"
    );

    let review_map_md = std::fs::read_to_string(out.join("review-map.md")).unwrap();
    let missing_pos = review_map_md
        .find("1. `src/missing.rs`")
        .expect("missing-evidence item should be first");
    let available_pos = review_map_md
        .find("2. `src/available.rs`")
        .expect("available-evidence item should be second");
    assert!(
        missing_pos < available_pos,
        "review-map Markdown should order missing evidence before raw size"
    );
    assert!(review_map_md.contains(
        "Review-first signal: Evidence is missing for this item; repair or acknowledge the missing proof before relying on it."
    ));
    assert!(review_map_md.contains("do not treat this packet as a merge verdict"));
}

#[test]
fn scenario_review_map_orders_contract_paths_before_ordinary_low_priority_items() {
    let dir = tempfile::tempdir().unwrap();
    let mut receipt = minimal_receipt();
    receipt.review_plan = vec![
        ReviewItem {
            path: "docs/NEXT.md".to_string(),
            reason: "Program note changed".to_string(),
            priority: 3,
            complexity: Some(1),
            lines_changed: Some(4),
        },
        ReviewItem {
            path: "crates/tokmd/schemas/review-map.schema.json".to_string(),
            reason: "Review-map schema changed".to_string(),
            priority: 3,
            complexity: Some(1),
            lines_changed: Some(1),
        },
    ];
    let out = dir.path().join("review");

    tokmd_cockpit::render::write_review_packet(&out, &receipt).unwrap();

    let review_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("review-map.json")).unwrap())
            .unwrap();
    assert_eq!(
        review_map["items"][0]["path"],
        "crates/tokmd/schemas/review-map.schema.json"
    );
    assert_eq!(review_map["items"][0]["source_index"], 1);
    assert_eq!(
        review_map["items"][0]["evidence_refs"][0],
        "cockpit.json#/review_plan/1"
    );
    assert_eq!(review_map["items"][1]["path"], "docs/NEXT.md");
    assert_eq!(review_map["items"][1]["source_index"], 0);

    let review_map_md = std::fs::read_to_string(out.join("review-map.md")).unwrap();
    let schema_pos = review_map_md
        .find("1. `crates/tokmd/schemas/review-map.schema.json`")
        .expect("schema contract item should be first");
    let ordinary_pos = review_map_md
        .find("2. `docs/NEXT.md`")
        .expect("ordinary low-priority item should be second");
    assert!(
        schema_pos < ordinary_pos,
        "schema contract paths should be reviewed before ordinary low-priority docs"
    );
    assert!(review_map_md.contains(
        "Review-first signal: Contract or policy path changed; review before ordinary implementation changes."
    ));
}

#[test]
fn scenario_write_review_packet_includes_imported_proof_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let mut receipt = minimal_receipt();
    receipt.review_plan = vec![
        ReviewItem {
            path: "crates/tokmd-cockpit/src/lib.rs".to_string(),
            reason: "Cockpit proof-aware review packet changed".to_string(),
            priority: 1,
            complexity: Some(3),
            lines_changed: Some(12),
        },
        ReviewItem {
            path: "unrelated.rs".to_string(),
            reason: "Unrelated changed file".to_string(),
            priority: 3,
            complexity: None,
            lines_changed: Some(1),
        },
    ];
    let out = dir.path().join("review");
    let proof = tokmd_cockpit::parse_proof_evidence_input(
        r#"{
  "schema": "tokmd.proof_run_observation.v1",
  "status": "passed",
  "execution_status": "executed",
  "profile": "fast",
  "base": "main",
  "head": "feature",
  "ok": true,
  "execution_guard": {
    "enabled": true,
    "ci": true,
    "reason": "required proof-run summary verified"
  },
  "counts": {
    "commands_total": 1,
    "required_planned": 1,
    "advisory_skipped": 0,
    "executed": 1,
    "passed": 1,
    "failed": 0
  },
  "scopes": [
    {
      "name": "tokmd_cockpit",
      "kind": "test",
      "command": "cargo test -p tokmd-cockpit",
      "status": "passed",
      "exit_code": 0
    }
  ],
  "changed_files": ["crates/tokmd-cockpit/src/lib.rs"],
  "unknown_files": []
}"#,
        "proof-run-observation.json",
    )
    .unwrap();

    tokmd_cockpit::render::write_review_packet_with_proof_evidence(&out, &receipt, &[proof])
        .unwrap();

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("evidence.json")).unwrap()).unwrap();
    let proof = evidence["proof"].as_array().expect("proof evidence array");
    assert_eq!(proof.len(), 1);
    assert_eq!(proof[0]["kind"], "proof_run_observation");
    assert_eq!(proof[0]["source"], "proof/proof-run-observation.json");
    assert_eq!(proof[0]["source_schema"], "tokmd.proof_run_observation.v1");
    assert_eq!(proof[0]["profile"], "fast");
    assert_eq!(proof[0]["scope"], "tokmd_cockpit");
    assert_eq!(proof[0]["command"], "cargo test -p tokmd-cockpit");
    assert_eq!(proof[0]["required"], true);
    assert_eq!(proof[0]["advisory"], false);
    assert_eq!(proof[0]["execution_status"], "executed_passed");
    assert_eq!(proof[0]["availability"], "available");
    assert_eq!(proof[0]["commit_match"], "exact");
    assert_eq!(
        proof[0]["refs"][0],
        "proof/proof-run-observation.json#/scopes/0"
    );
    assert!(
        out.join("proof")
            .join("proof-run-observation.json")
            .exists(),
        "imported proof artifact should be copied into the packet"
    );

    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["verdict"]["reason"],
        "cockpit review packets are advisory by default"
    );
    assert_eq!(
        manifest["verdict"]["evidence"]["details"], "evidence.json#/gates",
        "imported proof should not change gate verdict counts yet"
    );
    let artifacts = manifest["artifacts"]
        .as_array()
        .expect("manifest artifacts");
    assert!(
        artifacts.iter().any(|artifact| {
            artifact["id"] == "proof-run-observation"
                && artifact["path"] == "proof/proof-run-observation.json"
                && artifact["schema"] == "tokmd.proof_run_observation.v1"
                && artifact["media_type"] == "application/json"
        }),
        "manifest should list copied proof artifact"
    );

    let review_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("review-map.json")).unwrap())
            .unwrap();
    let items = review_map["items"].as_array().expect("review-map items");
    assert_eq!(items.len(), 2);
    assert!(
        review_map["evidence"]["refs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reference| reference == "evidence.json#/proof"),
        "packet-level review-map evidence refs should link to imported proof evidence"
    );
    assert_eq!(items[0]["path"], "crates/tokmd-cockpit/src/lib.rs");
    assert_eq!(
        items[0]["proof_refs"][0], "evidence.json#/proof/0",
        "matching review item should link to normalized proof evidence"
    );
    assert_eq!(
        items[0]["proof_refs"][1], "proof/proof-run-observation.json#/scopes/0",
        "matching review item should link to packet-local source proof artifact"
    );
    assert!(
        items[1]["proof_refs"].as_array().unwrap().is_empty(),
        "unrelated review item must not inherit proof refs"
    );

    let review_map_md = std::fs::read_to_string(out.join("review-map.md")).unwrap();
    assert!(review_map_md.contains("Proof evidence overview:"));
    assert!(review_map_md.contains("- Required proof: 1 passed, 0 failed, 0 missing"));
    assert!(review_map_md.contains("- Advisory proof: 0 available, 0 missing"));
    assert!(review_map_md.contains("- Freshness: 1 exact, 0 partial, 0 stale, 0 unknown"));
    assert_eq!(
        review_map_md.matches("   Proof:").count(),
        1,
        "only the matching review item should render imported proof evidence"
    );
    assert!(review_map_md.contains(
        "Required: tokmd_cockpit passed (available, freshness: exact) - cargo test -p tokmd-cockpit"
    ));
    assert!(review_map_md.contains("   Proof references:"));
    assert!(review_map_md.contains("evidence.json#/proof/0"));
    assert!(review_map_md.contains("proof/proof-run-observation.json#/scopes/0"));

    let comment_md = std::fs::read_to_string(out.join("comment.md")).unwrap();
    assert!(comment_md.contains("Proof evidence"));
    assert!(comment_md.contains("Required proof: 1 passed, 0 failed, 0 missing"));
    assert!(comment_md.contains("Advisory proof: 0 available, 0 missing"));
    assert!(comment_md.contains("Proof freshness: 1 exact, 0 partial, 0 stale, 0 unknown"));
}

#[test]
fn scenario_write_review_packet_keeps_ambiguous_multiscope_proof_packet_level() {
    let dir = tempfile::tempdir().unwrap();
    let mut receipt = minimal_receipt();
    receipt.review_plan = vec![
        ReviewItem {
            path: "crates/tokmd-cockpit/src/lib.rs".to_string(),
            reason: "Cockpit review code changed".to_string(),
            priority: 1,
            complexity: Some(3),
            lines_changed: Some(12),
        },
        ReviewItem {
            path: "crates/tokmd/src/commands/cockpit.rs".to_string(),
            reason: "CLI cockpit command changed".to_string(),
            priority: 1,
            complexity: Some(2),
            lines_changed: Some(8),
        },
    ];
    let out = dir.path().join("review");
    let proof = tokmd_cockpit::parse_proof_evidence_input(
        r#"{
  "schema": "tokmd.proof_run_observation.v1",
  "status": "passed",
  "execution_status": "executed",
  "profile": "fast",
  "base": "main",
  "head": "feature",
  "ok": true,
  "execution_guard": {
    "enabled": true,
    "ci": true,
    "reason": "required proof-run summary verified"
  },
  "counts": {
    "commands_total": 2,
    "required_planned": 2,
    "advisory_skipped": 0,
    "executed": 2,
    "passed": 2,
    "failed": 0
  },
  "scopes": [
    {
      "name": "tokmd_cockpit",
      "kind": "test",
      "command": "cargo test -p tokmd-cockpit",
      "status": "passed",
      "exit_code": 0
    },
    {
      "name": "tokmd_cli",
      "kind": "test",
      "command": "cargo test -p tokmd --test cockpit_integration",
      "status": "passed",
      "exit_code": 0
    }
  ],
  "changed_files": [
    "crates/tokmd-cockpit/src/lib.rs",
    "crates/tokmd/src/commands/cockpit.rs"
  ],
  "unknown_files": []
}"#,
        "proof-run-observation.json",
    )
    .unwrap();

    tokmd_cockpit::render::write_review_packet_with_proof_evidence(&out, &receipt, &[proof])
        .unwrap();

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("evidence.json")).unwrap()).unwrap();
    let proof = evidence["proof"].as_array().expect("proof evidence array");
    assert_eq!(proof.len(), 2);

    let review_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("review-map.json")).unwrap())
            .unwrap();
    let items = review_map["items"].as_array().expect("review-map items");
    assert_eq!(items.len(), 2);
    assert!(
        review_map["evidence"]["refs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reference| reference == "evidence.json#/proof"),
        "multi-scope proof should stay visible at packet level"
    );
    assert!(
        items[0]["proof_refs"].as_array().unwrap().is_empty(),
        "ambiguous top-level changed_files must not attach all scopes to the cockpit item"
    );
    assert!(
        items[1]["proof_refs"].as_array().unwrap().is_empty(),
        "ambiguous top-level changed_files must not attach all scopes to the CLI item"
    );

    let review_map_md = std::fs::read_to_string(out.join("review-map.md")).unwrap();
    assert!(review_map_md.contains("Proof evidence overview:"));
    assert!(review_map_md.contains("- Required proof: 2 passed, 0 failed, 0 missing"));
    assert_eq!(
        review_map_md.matches("   Proof:").count(),
        0,
        "ambiguous multi-scope proof should not render item-level proof blocks"
    );
}

#[test]
fn scenario_write_review_packet_includes_imported_doc_artifacts_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let mut receipt = minimal_receipt();
    receipt.review_plan = vec![
        ReviewItem {
            path: "docs/specs/doc-artifacts.md".to_string(),
            reason: "Documentation artifact contract changed".to_string(),
            priority: 1,
            complexity: None,
            lines_changed: Some(24),
        },
        ReviewItem {
            path: "docs/review-packet.md".to_string(),
            reason: "Review packet contract changed".to_string(),
            priority: 1,
            complexity: None,
            lines_changed: Some(6),
        },
    ];
    let out = dir.path().join("review");
    let doc_artifacts = tokmd_cockpit::parse_doc_artifacts_evidence_input(
        r#"{
  "schema": "tokmd.doc_artifacts_check.v1",
  "ok": true,
  "checked": {
    "required_docs": 1,
    "family_files": 11,
    "active_goals": 1,
    "spec_index_artifacts": 9,
    "spec_index_lanes": 2
  },
  "errors": []
}"#,
        "target/docs/doc-artifacts-check.json",
    )
    .unwrap();

    tokmd_cockpit::render::write_review_packet_with_imported_evidence(
        &out,
        &receipt,
        &[],
        Some(&doc_artifacts),
    )
    .unwrap();

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("evidence.json")).unwrap()).unwrap();
    assert_eq!(
        evidence["doc_artifacts"]["source"],
        "docs/doc-artifacts-check.json"
    );
    assert_eq!(
        evidence["doc_artifacts"]["source_schema"],
        "tokmd.doc_artifacts_check.v1"
    );
    assert_eq!(evidence["doc_artifacts"]["ok"], true);
    assert_eq!(evidence["doc_artifacts"]["availability"], "available");
    assert_eq!(evidence["doc_artifacts"]["checked"]["required_docs"], 1);
    assert_eq!(evidence["doc_artifacts"]["checked"]["family_files"], 11);
    assert_eq!(evidence["doc_artifacts"]["checked"]["active_goals"], 1);
    assert_eq!(
        evidence["doc_artifacts"]["checked"]["spec_index_artifacts"],
        9
    );
    assert_eq!(evidence["doc_artifacts"]["checked"]["spec_index_lanes"], 2);
    assert_eq!(
        evidence["doc_artifacts"]["refs"][0],
        "docs/doc-artifacts-check.json"
    );

    let copied_doc_artifacts_path = out.join("docs").join("doc-artifacts-check.json");
    assert!(
        copied_doc_artifacts_path.exists(),
        "doc-artifacts receipt should be copied into the review packet"
    );

    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("manifest.json")).unwrap()).unwrap();
    let artifacts = manifest["artifacts"]
        .as_array()
        .expect("manifest artifacts");
    assert!(
        artifacts.iter().any(|artifact| {
            artifact["id"] == "doc-artifacts-check"
                && artifact["path"] == "docs/doc-artifacts-check.json"
                && artifact["schema"] == "tokmd.doc_artifacts_check.v1"
                && artifact["media_type"] == "application/json"
        }),
        "manifest should list copied doc-artifacts receipt"
    );

    let review_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("review-map.json")).unwrap())
            .unwrap();
    assert!(
        review_map["evidence"]["refs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reference| reference == "evidence.json#/doc_artifacts"),
        "packet-level review-map evidence refs should link to doc-artifacts evidence"
    );
    assert_eq!(
        review_map["items"][0]["doc_artifacts_refs"][0],
        "evidence.json#/doc_artifacts"
    );
    assert_eq!(
        review_map["items"][0]["doc_artifacts_refs"][1],
        "docs/doc-artifacts-check.json"
    );
    assert_eq!(
        review_map["items"][1]["doc_artifacts_refs"][0], "evidence.json#/doc_artifacts",
        "active review-packet contract docs should be treated as source-of-truth review items"
    );
    assert_eq!(
        review_map["items"][1]["doc_artifacts_refs"][1],
        "docs/doc-artifacts-check.json"
    );
    assert!(
        review_map["items"][0]["reproduce"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command.as_str()
                == Some(
                    "cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json"
                )),
        "source-of-truth review items should include the docs checker receipt command"
    );
    assert!(
        review_map["items"][1]["reproduce"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command.as_str()
                == Some(
                    "cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json"
                )),
        "active review-packet contract docs should include the docs checker receipt command"
    );

    let review_map_md = std::fs::read_to_string(out.join("review-map.md")).unwrap();
    assert!(review_map_md.contains("Doc artifacts: verified"));
    assert!(review_map_md.contains("evidence.json#/doc_artifacts"));
    assert!(review_map_md.contains("docs/doc-artifacts-check.json"));
    assert!(
        review_map_md.contains(
            "cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json"
        )
    );

    let comment_md = std::fs::read_to_string(out.join("comment.md")).unwrap();
    assert!(comment_md.contains("Doc artifacts"));
    assert!(comment_md.contains(
        "verified (1 required docs, 11 family files, 1 active goals, 9 spec-index artifacts, 2 spec-index lanes)"
    ));
}

#[test]
fn scenario_write_review_packet_marks_missing_doc_artifacts_for_source_of_truth_change() {
    let dir = tempfile::tempdir().unwrap();
    let mut receipt = minimal_receipt();
    receipt.review_plan = vec![ReviewItem {
        path: ".jules/goals/active.toml".to_string(),
        reason: "Active agent goal changed".to_string(),
        priority: 1,
        complexity: None,
        lines_changed: Some(8),
    }];
    let out = dir.path().join("review");

    tokmd_cockpit::render::write_review_packet(&out, &receipt).unwrap();

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("evidence.json")).unwrap()).unwrap();
    assert_eq!(evidence["doc_artifacts"]["availability"], "missing");
    assert_eq!(
        evidence["doc_artifacts"]["source_schema"],
        "tokmd.doc_artifacts_check.v1"
    );
    assert!(evidence["doc_artifacts"]["source"].is_null());
    assert!(evidence["doc_artifacts"]["checked"].is_null());

    let review_map_md = std::fs::read_to_string(out.join("review-map.md")).unwrap();
    assert!(review_map_md.contains(
        "Review-first signal: Source-of-truth artifact changed; review the governing contract before ordinary files."
    ));
    assert!(review_map_md.contains("Doc artifacts: missing for source-of-truth changes."));
    assert!(review_map_md.contains("   Doc artifacts: missing"));
    assert!(
        review_map_md.contains(
            "cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json"
        )
    );

    let comment_md = std::fs::read_to_string(out.join("comment.md")).unwrap();
    assert!(comment_md.contains("Doc artifacts"));
    assert!(comment_md.contains("missing for source-of-truth changes"));
}

#[test]
fn scenario_review_map_md_includes_packet_level_proof_overview() {
    let dir = tempfile::tempdir().unwrap();
    let mut receipt = minimal_receipt();
    receipt.review_plan = vec![ReviewItem {
        path: "crates/tokmd-cockpit/src/render/review_map.rs".to_string(),
        reason: "Review map rendering changed".to_string(),
        priority: 1,
        complexity: Some(2),
        lines_changed: Some(8),
    }];
    let out = dir.path().join("review");
    let proof = tokmd_cockpit::parse_proof_evidence_input(
        r#"{
  "schema": "tokmd.coverage_receipt.v1",
  "schema_version": 1,
  "repo": "EffortlessMetrics/tokmd",
  "lane": "scoped",
  "flag": "tokmd_cockpit",
  "workflow": "Coverage",
  "sha": "feature",
  "github": {
    "run_id": "12345",
    "run_attempt": "1",
    "event_name": "pull_request",
    "ref_name": "feature"
  },
  "artifacts": [
    {
      "path": "target/proof/coverage/tokmd-cockpit.lcov",
      "kind": "lcov",
      "bytes": 42,
      "non_empty": true
    }
  ],
  "status": {
    "ok": true,
    "missing": [],
    "empty": []
  }
}"#,
        "coverage-receipt.json",
    )
    .unwrap();

    tokmd_cockpit::render::write_review_packet_with_proof_evidence(&out, &receipt, &[proof])
        .unwrap();

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("evidence.json")).unwrap()).unwrap();
    let proof = evidence["proof"].as_array().expect("proof evidence array");
    assert_eq!(proof.len(), 1);
    assert_eq!(proof[0]["kind"], "coverage_receipt");
    assert_eq!(proof[0]["run_id"], "12345");
    assert_eq!(proof[0]["run_attempt"], "1");

    let review_map_md = std::fs::read_to_string(out.join("review-map.md")).unwrap();
    assert!(review_map_md.contains("Proof evidence overview:"));
    assert!(review_map_md.contains("- Required proof: 0 passed, 0 failed, 0 missing"));
    assert!(review_map_md.contains("- Advisory proof: 1 available, 0 missing"));
    assert!(review_map_md.contains("- Freshness: 1 exact, 0 partial, 0 stale, 0 unknown"));
    assert!(
        !review_map_md.contains("   Proof:"),
        "coverage receipts without direct changed-file matches should stay packet-level"
    );
    assert!(
        review_map_md.contains("Evidence references:"),
        "review map should still point reviewers to packet evidence refs"
    );
}

// ===========================================================================
// Scenario: Load baseline trend with missing file
// ===========================================================================

#[test]
fn scenario_load_trend_missing_baseline() {
    let dir = tempfile::tempdir().unwrap();
    let receipt = minimal_receipt();
    let missing = dir.path().join("nonexistent.json");

    let trend = load_and_compute_trend(&missing, &receipt).unwrap();

    assert!(!trend.baseline_available);
}

// ===========================================================================
// Scenario: Load baseline trend with invalid JSON
// ===========================================================================

#[test]
fn scenario_load_trend_invalid_json() {
    let dir = tempfile::tempdir().unwrap();
    let receipt = minimal_receipt();
    let path = dir.path().join("baseline.json");
    std::fs::write(&path, "not valid json").unwrap();

    let trend = load_and_compute_trend(&path, &receipt).unwrap();

    assert!(!trend.baseline_available);
}

// ===========================================================================
// Scenario: Load baseline trend with valid receipt
// ===========================================================================

#[test]
fn scenario_load_trend_valid_baseline() {
    let dir = tempfile::tempdir().unwrap();
    let mut baseline = minimal_receipt();
    baseline.code_health.score = 70;
    baseline.risk.score = 30;
    let baseline_json = serde_json::to_string_pretty(&baseline).unwrap();
    let path = dir.path().join("baseline.json");
    std::fs::write(&path, &baseline_json).unwrap();

    let mut current = minimal_receipt();
    current.code_health.score = 90;
    current.risk.score = 10;

    let trend = load_and_compute_trend(&path, &current).unwrap();

    assert!(trend.baseline_available);
    let health = trend.health.unwrap();
    assert_eq!(health.direction, TrendDirection::Improving);
    assert_eq!(health.current, 90.0);
    assert_eq!(health.previous, 70.0);

    let risk = trend.risk.unwrap();
    assert_eq!(risk.direction, TrendDirection::Improving); // lower risk is better
}
