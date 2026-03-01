//! Integration tests for full cockpit workflows.
//!
//! These tests exercise end-to-end flows: computing metrics, rendering output,
//! writing artifacts, and verifying cross-module consistency.

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

// ===========================================================================
// Integration: Empty PR workflow
// ===========================================================================

#[test]
fn integration_empty_pr_workflow() {
    // A PR with no changed files
    let stats: Vec<FileStat> = Vec::new();
    let receipt = make_receipt(&stats);

    assert_eq!(receipt.change_surface.files_changed, 0);
    assert_eq!(receipt.change_surface.insertions, 0);
    assert_eq!(receipt.change_surface.deletions, 0);
    assert_eq!(receipt.change_surface.net_lines, 0);
    assert_eq!(receipt.composition.code_pct, 0.0);
    assert_eq!(receipt.code_health.score, 100);
    assert_eq!(receipt.risk.level, RiskLevel::Low);
    assert!(receipt.review_plan.is_empty());

    // Should serialize fine
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let roundtrip: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtrip.schema_version, COCKPIT_SCHEMA_VERSION);
}

// ===========================================================================
// Integration: Single file PR workflow
// ===========================================================================

#[test]
fn integration_single_file_pr() {
    let stats = vec![make_file_stat("src/main.rs", 50, 10)];
    let receipt = make_receipt(&stats);

    assert_eq!(receipt.change_surface.files_changed, 1);
    assert_eq!(receipt.change_surface.insertions, 50);
    assert_eq!(receipt.change_surface.deletions, 10);
    assert_eq!(receipt.change_surface.net_lines, 40);

    // Single code file -> 100% code
    assert_eq!(receipt.composition.code_pct, 1.0);
    assert_eq!(receipt.composition.test_ratio, 0.0);

    // Small file -> healthy
    assert_eq!(receipt.code_health.score, 100);
    assert_eq!(receipt.code_health.grade, "A");

    // One review item
    assert_eq!(receipt.review_plan.len(), 1);
    assert_eq!(receipt.review_plan[0].path, "src/main.rs");
}

// ===========================================================================
// Integration: Multi-file PR with mixed types
// ===========================================================================

#[test]
fn integration_multi_file_mixed_pr() {
    let stats = vec![
        make_file_stat("src/lib.rs", 100, 30),
        make_file_stat("tests/integration_test.rs", 50, 10),
        make_file_stat("README.md", 20, 5),
        make_file_stat("Cargo.toml", 5, 2),
        make_file_stat("crates/tokmd/src/commands/new_cmd.rs", 200, 50),
    ];
    let receipt = make_receipt(&stats);

    assert_eq!(receipt.change_surface.files_changed, 5);

    // Contract detection
    assert!(receipt.contracts.api_changed); // lib.rs
    assert!(receipt.contracts.cli_changed); // commands/
    assert!(!receipt.contracts.schema_changed);

    // Composition has code, test, docs, config
    assert!(receipt.composition.code_pct > 0.0);
    assert!(receipt.composition.test_pct > 0.0);
    assert!(receipt.composition.docs_pct > 0.0);
    assert!(receipt.composition.config_pct > 0.0);

    // Review plan sorted by priority
    for window in receipt.review_plan.windows(2) {
        assert!(window[0].priority <= window[1].priority);
    }
}

// ===========================================================================
// Integration: Large file PR degrades health and increases risk
// ===========================================================================

#[test]
fn integration_large_file_pr() {
    let stats = vec![
        make_file_stat("src/mega.rs", 400, 200), // 600 total > 500 threshold
        make_file_stat("src/another_big.rs", 300, 250), // 550 total > 500
    ];
    let receipt = make_receipt(&stats);

    // Health degraded
    assert!(receipt.code_health.score < 100);
    assert_eq!(receipt.code_health.large_files_touched, 2);
    assert!(!receipt.code_health.warnings.is_empty());

    // Risk increased
    assert!(!receipt.risk.hotspots_touched.is_empty());
    assert!(receipt.risk.score > 0);
}

// ===========================================================================
// Integration: Full render pipeline (JSON -> Markdown -> Sections -> Comment)
// ===========================================================================

#[test]
fn integration_full_render_pipeline() {
    let stats = vec![
        make_file_stat("src/main.rs", 50, 10),
        make_file_stat("tests/test_it.rs", 30, 5),
        make_file_stat("Cargo.toml", 3, 1),
    ];
    let receipt = make_receipt(&stats);

    // JSON
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    assert!(json.contains("cockpit"));
    assert!(json.contains("schema_version"));

    // JSON roundtrip
    let parsed: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.change_surface.files_changed, 3);

    // Markdown
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    assert!(md.contains("## Glass Cockpit"));
    assert!(md.contains("Files Changed"));
    assert!(md.contains("3"));

    // Sections
    let sections = tokmd_cockpit::render::render_sections(&receipt);
    assert!(sections.contains("## Glass Cockpit"));
    assert!(sections.contains("## Review Plan"));

    // Comment
    let comment = tokmd_cockpit::render::render_comment_md(&receipt);
    assert!(comment.contains("## Glass Cockpit Summary"));
}

// ===========================================================================
// Integration: Write artifacts and verify contents
// ===========================================================================

#[test]
fn integration_write_and_read_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let stats = vec![make_file_stat("src/lib.rs", 20, 5)];
    let receipt = make_receipt(&stats);
    let out = dir.path().join("cockpit-output");

    // Write
    tokmd_cockpit::render::write_artifacts(&out, &receipt).unwrap();

    // Read and verify cockpit.json
    let cockpit_json = std::fs::read_to_string(out.join("cockpit.json")).unwrap();
    let parsed: CockpitReceipt = serde_json::from_str(&cockpit_json).unwrap();
    assert_eq!(parsed.mode, "cockpit");
    assert_eq!(parsed.change_surface.files_changed, 1);

    // Read and verify report.json (sensor envelope)
    let report_json = std::fs::read_to_string(out.join("report.json")).unwrap();
    let report: serde_json::Value = serde_json::from_str(&report_json).unwrap();
    assert!(report.get("tool").is_some());
    assert!(report.get("verdict").is_some());

    // Read and verify comment.md
    let comment = std::fs::read_to_string(out.join("comment.md")).unwrap();
    assert!(comment.contains("Glass Cockpit Summary"));
}

// ===========================================================================
// Integration: Trend comparison end-to-end
// ===========================================================================

#[test]
fn integration_trend_comparison_e2e() {
    let dir = tempfile::tempdir().unwrap();

    // Create baseline receipt
    let baseline_stats = vec![make_file_stat("src/lib.rs", 100, 50)];
    let mut baseline = make_receipt(&baseline_stats);
    baseline.code_health.score = 75;
    baseline.risk.score = 40;

    // Write baseline to disk
    let baseline_path = dir.path().join("baseline.json");
    let baseline_json = serde_json::to_string_pretty(&baseline).unwrap();
    std::fs::write(&baseline_path, &baseline_json).unwrap();

    // Create current receipt with better metrics
    let current_stats = vec![make_file_stat("src/lib.rs", 20, 5)];
    let mut current = make_receipt(&current_stats);
    current.code_health.score = 95;
    current.risk.score = 10;

    // Compute trend
    let trend = load_and_compute_trend(&baseline_path, &current).unwrap();

    assert!(trend.baseline_available);
    assert!(trend.baseline_path.is_some());

    // Health improved (95 vs 75)
    let health = trend.health.unwrap();
    assert_eq!(health.direction, TrendDirection::Improving);
    assert_eq!(health.current, 95.0);
    assert_eq!(health.previous, 75.0);

    // Risk improved (10 vs 40, lower is better)
    let risk = trend.risk.unwrap();
    assert_eq!(risk.direction, TrendDirection::Improving);

    // Complexity trend should exist
    assert!(trend.complexity.is_some());
}

// ===========================================================================
// Integration: Determinism hashing workflow
// ===========================================================================

#[test]
fn integration_determinism_hashing_workflow() {
    let dir = tempfile::tempdir().unwrap();

    // Create a mini project
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
    std::fs::write(
        dir.path().join("Cargo.lock"),
        "[[package]]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    // Hash using explicit paths
    let h1 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["main.rs", "lib.rs"])
        .unwrap();

    // Same files, different order -> same hash
    let h2 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["lib.rs", "main.rs"])
        .unwrap();
    assert_eq!(h1, h2);

    // Duplicate entries -> same hash (dedup)
    let h3 = tokmd_cockpit::determinism::hash_files_from_paths(
        dir.path(),
        &["main.rs", "lib.rs", "main.rs"],
    )
    .unwrap();
    assert_eq!(h1, h3);

    // Cargo.lock hash
    let lock_hash = tokmd_cockpit::determinism::hash_cargo_lock(dir.path()).unwrap();
    assert!(lock_hash.is_some());

    // Modify a file -> hash changes
    std::fs::write(
        dir.path().join("main.rs"),
        "fn main() { println!(\"world\"); }",
    )
    .unwrap();
    let h4 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["main.rs", "lib.rs"])
        .unwrap();
    assert_ne!(h1, h4);
}

// ===========================================================================
// Integration: Determinism hash error propagation (non-NotFound)
// ===========================================================================

#[test]
fn integration_determinism_not_found_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn a() {}").unwrap();

    // NotFound files are silently skipped
    let result =
        tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs", "missing.rs"]);
    assert!(result.is_ok());

    // Hash should equal hash of just a.rs
    let just_a = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();
    assert_eq!(result.unwrap(), just_a);
}

// ===========================================================================
// Integration: JSON schema version consistency
// ===========================================================================

#[test]
fn integration_schema_version_in_json() {
    let receipt = make_receipt(&[]);
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(
        value["schema_version"].as_u64().unwrap(),
        COCKPIT_SCHEMA_VERSION as u64,
    );
}

// ===========================================================================
// Integration: Multiple large files → Critical complexity indicator
// ===========================================================================

#[test]
fn integration_many_large_files_critical_complexity() {
    let stats: Vec<FileStat> = (0..6)
        .map(|i| make_file_stat(&format!("src/big_{}.rs", i), 300, 250))
        .collect();
    let contracts = Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    };
    let health = compute_code_health(&stats, &contracts);

    assert_eq!(health.complexity_indicator, ComplexityIndicator::Critical);
    assert_eq!(health.large_files_touched, 6);
}

// ===========================================================================
// Integration: Comment.md includes contract changes when present
// ===========================================================================

#[test]
fn integration_comment_md_contract_section() {
    let stats = vec![
        make_file_stat("src/lib.rs", 10, 5),
        make_file_stat("docs/schema.json", 20, 10),
    ];
    let receipt = make_receipt(&stats);
    // Ensure contracts are set correctly
    assert!(receipt.contracts.api_changed);
    assert!(receipt.contracts.schema_changed);

    let comment = tokmd_cockpit::render::render_comment_md(&receipt);
    assert!(comment.contains("Contract changes"));
    assert!(comment.contains("API contract changed"));
    assert!(comment.contains("Schema contract changed"));
}

// ===========================================================================
// Integration: Comment.md omits contract section when none changed
// ===========================================================================

#[test]
fn integration_comment_md_no_contracts() {
    let stats = vec![make_file_stat("src/utils.rs", 10, 5)];
    let receipt = make_receipt(&stats);

    let comment = tokmd_cockpit::render::render_comment_md(&receipt);
    assert!(!comment.contains("Contract changes"));
}

// ===========================================================================
// Integration: Markdown trend section renders when available
// ===========================================================================

#[test]
fn integration_markdown_with_trend() {
    let stats = vec![make_file_stat("src/main.rs", 20, 5)];
    let mut receipt = make_receipt(&stats);
    receipt.trend = Some(TrendComparison {
        baseline_available: true,
        baseline_path: Some("/path/to/baseline.json".to_string()),
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
    assert!(md.contains("Baseline"));
    assert!(md.contains("### Summary Comparison"));
}

// ===========================================================================
// Integration: Markdown without trend section
// ===========================================================================

#[test]
fn integration_markdown_without_trend() {
    let receipt = make_receipt(&[]);
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    assert!(!md.contains("### Trend"));
}

// ===========================================================================
// Integration: Evidence gate rendering in markdown
// ===========================================================================

#[test]
fn integration_evidence_gates_in_markdown() {
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
    assert!(md.contains("killed: 0"));
    assert!(md.contains("survivors: 1"));
}

// ===========================================================================
// Integration: FileStat AsRef<str>
// ===========================================================================

#[test]
fn integration_file_stat_as_ref() {
    let stat = make_file_stat("src/main.rs", 10, 5);
    let path: &str = stat.as_ref();
    assert_eq!(path, "src/main.rs");
}

// ===========================================================================
// Integration: Write artifacts to nested directory
// ===========================================================================

#[test]
fn integration_write_artifacts_nested_dir() {
    let dir = tempfile::tempdir().unwrap();
    let deep = dir.path().join("a").join("b").join("c");
    let receipt = make_receipt(&[]);

    tokmd_cockpit::render::write_artifacts(&deep, &receipt).unwrap();
    assert!(deep.join("cockpit.json").exists());
    assert!(deep.join("report.json").exists());
    assert!(deep.join("comment.md").exists());
}

// ===========================================================================
// Integration: Empty diff produces valid minimal receipt with correct defaults
// ===========================================================================

#[test]
fn integration_empty_diff_valid_minimal_receipt() {
    let stats: Vec<FileStat> = Vec::new();
    let receipt = make_receipt(&stats);

    // Change surface zeroed
    assert_eq!(receipt.change_surface.files_changed, 0);
    assert_eq!(receipt.change_surface.insertions, 0);
    assert_eq!(receipt.change_surface.deletions, 0);
    assert_eq!(receipt.change_surface.net_lines, 0);

    // Composition zeroed
    assert_eq!(receipt.composition.code_pct, 0.0);
    assert_eq!(receipt.composition.test_pct, 0.0);
    assert_eq!(receipt.composition.docs_pct, 0.0);
    assert_eq!(receipt.composition.config_pct, 0.0);
    assert_eq!(receipt.composition.test_ratio, 0.0);

    // Health at maximum
    assert_eq!(receipt.code_health.score, 100);
    assert_eq!(receipt.code_health.grade, "A");
    assert_eq!(receipt.code_health.large_files_touched, 0);
    assert_eq!(receipt.code_health.avg_file_size, 0);
    assert_eq!(
        receipt.code_health.complexity_indicator,
        ComplexityIndicator::Low
    );
    assert!(receipt.code_health.warnings.is_empty());

    // Risk at minimum
    assert_eq!(receipt.risk.score, 0);
    assert_eq!(receipt.risk.level, RiskLevel::Low);
    assert!(receipt.risk.hotspots_touched.is_empty());
    assert!(receipt.risk.bus_factor_warnings.is_empty());

    // No contract changes
    assert!(!receipt.contracts.api_changed);
    assert!(!receipt.contracts.cli_changed);
    assert!(!receipt.contracts.schema_changed);
    assert_eq!(receipt.contracts.breaking_indicators, 0);

    // Review plan empty
    assert!(receipt.review_plan.is_empty());

    // Schema version correct
    assert_eq!(receipt.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(receipt.mode, "cockpit");

    // JSON serialization succeeds
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    assert!(!json.is_empty());
    let _: CockpitReceipt = serde_json::from_str(&json).unwrap();
}

// ===========================================================================
// Integration: Known changes produce expected exact metrics
// ===========================================================================

#[test]
fn integration_known_changes_exact_metrics() {
    // 2 code, 1 test, 1 doc, 1 config = 5 files
    let stats = vec![
        make_file_stat("src/main.rs", 80, 20),       // code, 100 lines
        make_file_stat("src/utils.rs", 30, 10),      // code, 40 lines
        make_file_stat("tests/test_main.rs", 25, 5), // test, 30 lines
        make_file_stat("README.md", 15, 5),          // docs, 20 lines
        make_file_stat("Cargo.toml", 5, 2),          // config, 7 lines
    ];
    let receipt = make_receipt(&stats);

    // Change surface
    assert_eq!(receipt.change_surface.files_changed, 5);
    assert_eq!(receipt.change_surface.insertions, 155);
    assert_eq!(receipt.change_surface.deletions, 42);
    assert_eq!(receipt.change_surface.net_lines, 113);

    // Composition: 2 code, 1 test, 1 docs, 1 config = 5 total
    assert!((receipt.composition.code_pct - 0.4).abs() < 0.01);
    assert!((receipt.composition.test_pct - 0.2).abs() < 0.01);
    assert!((receipt.composition.docs_pct - 0.2).abs() < 0.01);
    assert!((receipt.composition.config_pct - 0.2).abs() < 0.01);
    // test_ratio = 1 test / 2 code = 0.5
    assert!((receipt.composition.test_ratio - 0.5).abs() < 0.01);

    // No large files → health=100, grade A
    assert_eq!(receipt.code_health.score, 100);
    assert_eq!(receipt.code_health.grade, "A");
    assert_eq!(receipt.code_health.large_files_touched, 0);

    // No file > 300 lines → no hotspots
    assert!(receipt.risk.hotspots_touched.is_empty());
    assert_eq!(receipt.risk.level, RiskLevel::Low);

    // Review plan has 5 items, sorted by priority
    assert_eq!(receipt.review_plan.len(), 5);
    for w in receipt.review_plan.windows(2) {
        assert!(w[0].priority <= w[1].priority);
    }
}

// ===========================================================================
// Integration: Evidence gate — overall Fail when any sub-gate fails
// ===========================================================================

#[test]
fn integration_evidence_gate_any_fail_yields_overall_fail() {
    let mut receipt = make_receipt(&[]);

    // Set mutation gate to Fail
    receipt.evidence.mutation.meta.status = GateStatus::Fail;
    receipt.evidence.mutation.survivors = vec![MutationSurvivor {
        file: "src/lib.rs".to_string(),
        line: 10,
        mutation: "replace == with !=".to_string(),
    }];

    // Set complexity gate to Pass
    receipt.evidence.complexity = Some(ComplexityGate {
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
        files_analyzed: 1,
        high_complexity_files: Vec::new(),
        avg_cyclomatic: 2.0,
        max_cyclomatic: 5,
        threshold_exceeded: false,
    });

    // Recompute overall: any Fail → overall Fail
    receipt.evidence.overall_status = compute_overall_gate_status(&receipt.evidence);
    assert_eq!(receipt.evidence.overall_status, GateStatus::Fail);
}

// ===========================================================================
// Integration: Evidence gate — overall Pass when all sub-gates pass
// ===========================================================================

#[test]
fn integration_evidence_gate_all_pass_yields_overall_pass() {
    let mut receipt = make_receipt(&[]);

    receipt.evidence.mutation.meta.status = GateStatus::Pass;
    receipt.evidence.diff_coverage = Some(DiffCoverageGate {
        meta: GateMeta {
            status: GateStatus::Pass,
            source: EvidenceSource::CiArtifact,
            commit_match: CommitMatch::Exact,
            scope: ScopeCoverage {
                relevant: vec!["src/lib.rs".into()],
                tested: vec!["src/lib.rs".into()],
                ratio: 0.95,
                lines_relevant: Some(100),
                lines_tested: Some(95),
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        lines_added: 100,
        lines_covered: 95,
        coverage_pct: 0.95,
        uncovered_hunks: Vec::new(),
    });

    receipt.evidence.overall_status = compute_overall_gate_status(&receipt.evidence);
    assert_eq!(receipt.evidence.overall_status, GateStatus::Pass);
}

// ===========================================================================
// Integration: Evidence gate — Warn propagates when no Fail present
// ===========================================================================

#[test]
fn integration_evidence_gate_warn_propagates() {
    let mut receipt = make_receipt(&[]);

    receipt.evidence.mutation.meta.status = GateStatus::Pass;
    receipt.evidence.complexity = Some(ComplexityGate {
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
        files_analyzed: 2,
        high_complexity_files: Vec::new(),
        avg_cyclomatic: 12.0,
        max_cyclomatic: 14,
        threshold_exceeded: false,
    });

    receipt.evidence.overall_status = compute_overall_gate_status(&receipt.evidence);
    assert_eq!(receipt.evidence.overall_status, GateStatus::Warn);
}

// ===========================================================================
// Integration: Evidence gate — Pending propagates when no Fail present
// ===========================================================================

#[test]
fn integration_evidence_gate_pending_propagates() {
    let mut receipt = make_receipt(&[]);

    receipt.evidence.mutation.meta.status = GateStatus::Pass;
    receipt.evidence.supply_chain = Some(SupplyChainGate {
        meta: GateMeta {
            status: GateStatus::Pending,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
            scope: ScopeCoverage {
                relevant: vec!["Cargo.lock".into()],
                tested: Vec::new(),
                ratio: 0.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        vulnerabilities: Vec::new(),
        denied: Vec::new(),
        advisory_db_version: None,
    });

    receipt.evidence.overall_status = compute_overall_gate_status(&receipt.evidence);
    assert_eq!(receipt.evidence.overall_status, GateStatus::Pending);
}

// ===========================================================================
// Integration: Evidence gate — all Skipped yields Skipped
// ===========================================================================

#[test]
fn integration_evidence_gate_all_skipped_yields_skipped() {
    let mut receipt = make_receipt(&[]);
    receipt.evidence.mutation.meta.status = GateStatus::Skipped;
    // No other gates set

    receipt.evidence.overall_status = compute_overall_gate_status(&receipt.evidence);
    assert_eq!(receipt.evidence.overall_status, GateStatus::Skipped);
}

// ===========================================================================
// Integration: Rich round-trip serialization with all fields populated
// ===========================================================================

#[test]
fn integration_rich_round_trip_serialization() {
    let stats = vec![
        make_file_stat("src/lib.rs", 100, 30),
        make_file_stat("tests/test_it.rs", 50, 10),
    ];
    let mut receipt = make_receipt(&stats);

    // Populate evidence gates
    receipt.evidence.mutation.meta.status = GateStatus::Warn;
    receipt.evidence.mutation.killed = 5;
    receipt.evidence.mutation.survivors = vec![MutationSurvivor {
        file: "src/lib.rs".to_string(),
        line: 42,
        mutation: "replace + with -".to_string(),
    }];

    receipt.evidence.diff_coverage = Some(DiffCoverageGate {
        meta: GateMeta {
            status: GateStatus::Pass,
            source: EvidenceSource::CiArtifact,
            commit_match: CommitMatch::Exact,
            scope: ScopeCoverage {
                relevant: vec!["src/lib.rs".into()],
                tested: vec!["src/lib.rs".into()],
                ratio: 0.85,
                lines_relevant: Some(100),
                lines_tested: Some(85),
            },
            evidence_commit: Some("abc123".to_string()),
            evidence_generated_at_ms: Some(999),
        },
        lines_added: 100,
        lines_covered: 85,
        coverage_pct: 0.85,
        uncovered_hunks: vec![UncoveredHunk {
            file: "src/lib.rs".to_string(),
            start_line: 50,
            end_line: 55,
        }],
    });

    receipt.evidence.complexity = Some(ComplexityGate {
        meta: GateMeta {
            status: GateStatus::Pass,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Exact,
            scope: ScopeCoverage {
                relevant: vec!["src/lib.rs".into()],
                tested: vec!["src/lib.rs".into()],
                ratio: 1.0,
                lines_relevant: None,
                lines_tested: None,
            },
            evidence_commit: None,
            evidence_generated_at_ms: None,
        },
        files_analyzed: 1,
        high_complexity_files: vec![HighComplexityFile {
            path: "src/lib.rs".into(),
            cyclomatic: 12,
            function_count: 3,
            max_function_length: 80,
        }],
        avg_cyclomatic: 12.0,
        max_cyclomatic: 12,
        threshold_exceeded: false,
    });

    // Populate trend
    receipt.trend = Some(TrendComparison {
        baseline_available: true,
        baseline_path: Some("/baseline.json".to_string()),
        baseline_generated_at_ms: Some(500),
        health: Some(TrendMetric {
            current: 95.0,
            previous: 80.0,
            delta: 15.0,
            delta_pct: 18.75,
            direction: TrendDirection::Improving,
        }),
        risk: Some(TrendMetric {
            current: 10.0,
            previous: 30.0,
            delta: -20.0,
            delta_pct: -66.67,
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

    // Serialize
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();

    // Deserialize
    let roundtrip: CockpitReceipt = serde_json::from_str(&json).unwrap();

    // Verify all top-level fields survive
    assert_eq!(roundtrip.schema_version, receipt.schema_version);
    assert_eq!(roundtrip.mode, receipt.mode);
    assert_eq!(roundtrip.base_ref, receipt.base_ref);
    assert_eq!(roundtrip.head_ref, receipt.head_ref);

    // Change surface
    assert_eq!(
        roundtrip.change_surface.files_changed,
        receipt.change_surface.files_changed
    );
    assert_eq!(
        roundtrip.change_surface.insertions,
        receipt.change_surface.insertions
    );

    // Evidence
    assert_eq!(
        roundtrip.evidence.mutation.killed,
        receipt.evidence.mutation.killed
    );
    assert_eq!(roundtrip.evidence.mutation.survivors.len(), 1);
    assert!(roundtrip.evidence.diff_coverage.is_some());
    let dc = roundtrip.evidence.diff_coverage.unwrap();
    assert_eq!(dc.lines_added, 100);
    assert_eq!(dc.coverage_pct, 0.85);
    assert_eq!(dc.uncovered_hunks.len(), 1);

    assert!(roundtrip.evidence.complexity.is_some());
    let cx = roundtrip.evidence.complexity.unwrap();
    assert_eq!(cx.high_complexity_files.len(), 1);
    assert_eq!(cx.avg_cyclomatic, 12.0);

    // Trend
    let trend = roundtrip.trend.unwrap();
    assert!(trend.baseline_available);
    assert_eq!(trend.health.unwrap().direction, TrendDirection::Improving);
    assert_eq!(trend.risk.unwrap().direction, TrendDirection::Improving);
    assert_eq!(trend.complexity.unwrap().direction, TrendDirection::Stable);
}

// ===========================================================================
// Integration: Determinism — same inputs always produce identical receipt
// ===========================================================================

#[test]
fn integration_determinism_identical_inputs_identical_receipt() {
    let stats = vec![
        make_file_stat("src/main.rs", 50, 10),
        make_file_stat("tests/test_a.rs", 30, 5),
        make_file_stat("README.md", 10, 2),
        make_file_stat("Cargo.toml", 3, 1),
    ];

    let receipt_a = make_receipt(&stats);
    let receipt_b = make_receipt(&stats);

    // Serialize both
    let json_a = serde_json::to_string(&receipt_a).unwrap();
    let json_b = serde_json::to_string(&receipt_b).unwrap();

    // Byte-for-byte identical
    assert_eq!(
        json_a, json_b,
        "identical inputs must produce identical JSON"
    );

    // Also verify individual computed sections match
    assert_eq!(
        receipt_a.composition.code_pct,
        receipt_b.composition.code_pct
    );
    assert_eq!(
        receipt_a.composition.test_ratio,
        receipt_b.composition.test_ratio
    );
    assert_eq!(receipt_a.code_health.score, receipt_b.code_health.score);
    assert_eq!(receipt_a.code_health.grade, receipt_b.code_health.grade);
    assert_eq!(receipt_a.risk.score, receipt_b.risk.score);
    assert_eq!(receipt_a.risk.level, receipt_b.risk.level);
    assert_eq!(receipt_a.review_plan.len(), receipt_b.review_plan.len());
    for (a, b) in receipt_a
        .review_plan
        .iter()
        .zip(receipt_b.review_plan.iter())
    {
        assert_eq!(a.path, b.path);
        assert_eq!(a.priority, b.priority);
    }
}

// ===========================================================================
// Integration: Schema version constant equals expected value
// ===========================================================================

#[test]
fn integration_schema_version_constant_is_correct() {
    assert_eq!(
        COCKPIT_SCHEMA_VERSION, 3,
        "COCKPIT_SCHEMA_VERSION must be 3"
    );

    // Also verify it appears in serialized receipt
    let receipt = make_receipt(&[]);
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        value["schema_version"].as_u64().unwrap(),
        3,
        "JSON schema_version must be 3"
    );
}

// ===========================================================================
// Integration: Determinism — reordered file stats produce same metrics
// ===========================================================================

#[test]
fn integration_determinism_reordered_inputs() {
    let stats_a = vec![
        make_file_stat("src/main.rs", 50, 10),
        make_file_stat("src/lib.rs", 30, 5),
        make_file_stat("tests/test_a.rs", 20, 5),
    ];
    let stats_b = vec![
        make_file_stat("tests/test_a.rs", 20, 5),
        make_file_stat("src/lib.rs", 30, 5),
        make_file_stat("src/main.rs", 50, 10),
    ];

    let receipt_a = make_receipt(&stats_a);
    let receipt_b = make_receipt(&stats_b);

    // Composition depends only on file set, not order
    assert_eq!(
        receipt_a.composition.code_pct,
        receipt_b.composition.code_pct
    );
    assert_eq!(
        receipt_a.composition.test_pct,
        receipt_b.composition.test_pct
    );
    assert_eq!(
        receipt_a.composition.test_ratio,
        receipt_b.composition.test_ratio
    );

    // Health and risk depend on same aggregated data
    assert_eq!(receipt_a.code_health.score, receipt_b.code_health.score);
    assert_eq!(receipt_a.risk.score, receipt_b.risk.score);
}

// ===========================================================================
// Integration: Evidence gate — Fail overrides Warn
// ===========================================================================

#[test]
fn integration_evidence_gate_fail_overrides_warn() {
    let mut receipt = make_receipt(&[]);

    // Mutation = Fail
    receipt.evidence.mutation.meta.status = GateStatus::Fail;
    // Complexity = Warn
    receipt.evidence.complexity = Some(ComplexityGate {
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
        avg_cyclomatic: 12.0,
        max_cyclomatic: 14,
        threshold_exceeded: false,
    });

    receipt.evidence.overall_status = compute_overall_gate_status(&receipt.evidence);
    assert_eq!(
        receipt.evidence.overall_status,
        GateStatus::Fail,
        "Fail must override Warn"
    );
}
