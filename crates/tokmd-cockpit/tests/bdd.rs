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
// Scenario: Zero diff stats → all evidence gates pass
// ===========================================================================

#[test]
fn scenario_zero_diff_stats_all_gates_pass() {
    // Given: a cockpit receipt with zero diff stats (no files changed)
    let receipt = minimal_receipt();

    // When: evaluating the evidence section
    let evidence = &receipt.evidence;

    // Then: overall status should be Pass (no failures),
    //       mutation gate is Skipped (no files to mutate),
    //       and optional gates are None (nothing to evaluate)
    assert_eq!(evidence.overall_status, GateStatus::Pass);
    assert_eq!(evidence.mutation.meta.status, GateStatus::Skipped);
    assert!(evidence.diff_coverage.is_none());
    assert!(evidence.contracts.is_none());
    assert!(evidence.supply_chain.is_none());
    assert!(evidence.determinism.is_none());
    assert!(evidence.complexity.is_none());
}

// ===========================================================================
// Scenario: High complexity receipt → complexity gate fails
// ===========================================================================

#[test]
fn scenario_high_complexity_gate_fails() {
    // Given: a receipt whose complexity gate reports >3 high-complexity files
    let mut receipt = minimal_receipt();
    receipt.evidence.complexity = Some(ComplexityGate {
        meta: GateMeta {
            status: GateStatus::Fail,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Exact,
            scope: ScopeCoverage {
                relevant: vec![
                    "src/a.rs".into(),
                    "src/b.rs".into(),
                    "src/c.rs".into(),
                    "src/d.rs".into(),
                ],
                tested: vec![
                    "src/a.rs".into(),
                    "src/b.rs".into(),
                    "src/c.rs".into(),
                    "src/d.rs".into(),
                ],
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        files_analyzed: 4,
        high_complexity_files: vec![
            HighComplexityFile {
                path: "src/a.rs".into(),
                cyclomatic: 25,
                function_count: 5,
                max_function_length: 200,
            },
            HighComplexityFile {
                path: "src/b.rs".into(),
                cyclomatic: 20,
                function_count: 3,
                max_function_length: 150,
            },
            HighComplexityFile {
                path: "src/c.rs".into(),
                cyclomatic: 18,
                function_count: 4,
                max_function_length: 180,
            },
            HighComplexityFile {
                path: "src/d.rs".into(),
                cyclomatic: 16,
                function_count: 2,
                max_function_length: 120,
            },
        ],
        avg_cyclomatic: 19.75,
        max_cyclomatic: 25,
        threshold_exceeded: true,
    });

    // When: evaluating the complexity gate
    let complexity = receipt.evidence.complexity.as_ref().unwrap();

    // Then: the gate status is Fail, threshold is exceeded,
    //       and all four files are flagged as high-complexity
    assert_eq!(complexity.meta.status, GateStatus::Fail);
    assert!(complexity.threshold_exceeded);
    assert_eq!(complexity.high_complexity_files.len(), 4);
    assert!(complexity.max_cyclomatic > COMPLEXITY_THRESHOLD);
}

// ===========================================================================
// Scenario: Determinism hash file comparison produces deterministic result
// ===========================================================================

#[test]
fn scenario_determinism_hash_comparison() {
    // Given: a temporary directory with source files and a determinism hash
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("main.rs"),
        "fn main() { println!(\"hello\"); }",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
    )
    .unwrap();

    let hash1 =
        tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["main.rs", "lib.rs"])
            .unwrap();

    // When: comparing a freshly computed hash against the stored hash
    let hash2 =
        tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["main.rs", "lib.rs"])
            .unwrap();

    // Then: matching hashes produce a deterministic (equal) result
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64); // BLAKE3 hex digest length

    // And: modifying a file causes the hashes to diverge
    std::fs::write(
        dir.path().join("main.rs"),
        "fn main() { println!(\"changed\"); }",
    )
    .unwrap();
    let hash3 =
        tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["main.rs", "lib.rs"])
            .unwrap();
    assert_ne!(hash1, hash3);
}

// ===========================================================================
// Scenario: Two receipts with different schemas → diff detected
// ===========================================================================

#[test]
fn scenario_different_schema_versions_detected_in_diff() {
    // Given: two cockpit receipts with different schema_version values
    let mut receipt_a = minimal_receipt();
    receipt_a.schema_version = 2;
    receipt_a.code_health.score = 80;
    receipt_a.risk.score = 30;

    let mut receipt_b = minimal_receipt();
    receipt_b.schema_version = 3;
    receipt_b.code_health.score = 90;
    receipt_b.risk.score = 10;

    // When: serializing both and comparing their JSON representations
    let json_a = serde_json::to_value(&receipt_a).unwrap();
    let json_b = serde_json::to_value(&receipt_b).unwrap();

    // Then: the schema_version field differs between the two
    assert_ne!(
        json_a["schema_version"], json_b["schema_version"],
        "schema_version should differ between receipts"
    );

    // And: a trend comparison detects the metric differences
    let dir = tempfile::tempdir().unwrap();
    let baseline_path = dir.path().join("baseline.json");
    std::fs::write(
        &baseline_path,
        serde_json::to_string_pretty(&receipt_a).unwrap(),
    )
    .unwrap();

    let trend = load_and_compute_trend(&baseline_path, &receipt_b).unwrap();
    assert!(trend.baseline_available);

    let health = trend.health.unwrap();
    assert_eq!(health.direction, TrendDirection::Improving);
    assert_eq!(health.current, 90.0);
    assert_eq!(health.previous, 80.0);
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
