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
    let mut receipt = make_receipt(&stats);
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
