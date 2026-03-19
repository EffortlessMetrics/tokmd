//! Snapshot tests for tokmd-cockpit render output – wave 45.
//!
//! Locks cockpit Markdown, JSON, sections, and comment output with insta
//! snapshots for regression detection.

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

// ── Markdown: empty PR ────────────────────────────────────────────────

#[test]
fn snapshot_cockpit_md_empty_pr() {
    let receipt = make_receipt(&[]);
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    insta::assert_snapshot!(md);
}

// ── Markdown: single file PR ──────────────────────────────────────────

#[test]
fn snapshot_cockpit_md_single_file() {
    let stats = vec![make_file_stat("src/main.rs", 50, 10)];
    let receipt = make_receipt(&stats);
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    insta::assert_snapshot!(md);
}

// ── Markdown: multi-file mixed PR ─────────────────────────────────────

#[test]
fn snapshot_cockpit_md_multi_file() {
    let stats = vec![
        make_file_stat("src/lib.rs", 100, 30),
        make_file_stat("tests/integration_test.rs", 50, 10),
        make_file_stat("README.md", 20, 5),
        make_file_stat("Cargo.toml", 5, 2),
    ];
    let receipt = make_receipt(&stats);
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    insta::assert_snapshot!(md);
}

// ── JSON: empty PR ────────────────────────────────────────────────────

#[test]
fn snapshot_cockpit_json_empty_pr() {
    let receipt = make_receipt(&[]);
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["generated_at_ms"] = serde_json::json!(0);
    insta::assert_json_snapshot!(v);
}

// ── JSON: single file PR ──────────────────────────────────────────────

#[test]
fn snapshot_cockpit_json_single_file() {
    let stats = vec![make_file_stat("src/main.rs", 50, 10)];
    let receipt = make_receipt(&stats);
    let json = tokmd_cockpit::render::render_json(&receipt).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["generated_at_ms"] = serde_json::json!(0);
    insta::assert_json_snapshot!(v);
}

// ── Comment: empty PR ─────────────────────────────────────────────────

#[test]
fn snapshot_cockpit_comment_empty_pr() {
    let receipt = make_receipt(&[]);
    let comment = tokmd_cockpit::render::render_comment_md(&receipt);
    insta::assert_snapshot!(comment);
}

// ── Comment: PR with contract changes ─────────────────────────────────

#[test]
fn snapshot_cockpit_comment_with_contracts() {
    let stats = vec![
        make_file_stat("src/lib.rs", 10, 5),
        make_file_stat("docs/schema.json", 20, 10),
        make_file_stat("crates/tokmd/src/commands/new_cmd.rs", 80, 20),
    ];
    let receipt = make_receipt(&stats);
    let comment = tokmd_cockpit::render::render_comment_md(&receipt);
    insta::assert_snapshot!(comment);
}

// ── Sections: multi-file PR ───────────────────────────────────────────

#[test]
fn snapshot_cockpit_sections_multi_file() {
    let stats = vec![
        make_file_stat("src/lib.rs", 100, 30),
        make_file_stat("tests/test_it.rs", 50, 10),
        make_file_stat("Cargo.toml", 3, 1),
    ];
    let receipt = make_receipt(&stats);
    let sections = tokmd_cockpit::render::render_sections(&receipt);
    insta::assert_snapshot!(sections);
}

// ── Markdown: with trend data ─────────────────────────────────────────

#[test]
fn snapshot_cockpit_md_with_trend() {
    let stats = vec![make_file_stat("src/main.rs", 20, 5)];
    let mut receipt = make_receipt(&stats);
    receipt.trend = Some(TrendComparison {
        baseline_available: true,
        baseline_path: Some("/path/to/baseline.json".to_string()),
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

    let md = tokmd_cockpit::render::render_markdown(&receipt);
    insta::assert_snapshot!(md);
}

// ── Markdown: with evidence gate failures ─────────────────────────────

#[test]
fn snapshot_cockpit_md_with_failed_gates() {
    let mut receipt = make_receipt(&[make_file_stat("src/lib.rs", 50, 10)]);
    receipt.evidence.overall_status = GateStatus::Fail;
    receipt.evidence.mutation.meta.status = GateStatus::Fail;
    receipt.evidence.mutation.survivors = vec![
        MutationSurvivor {
            file: "src/lib.rs".to_string(),
            line: 42,
            mutation: "replace + with -".to_string(),
        },
        MutationSurvivor {
            file: "src/lib.rs".to_string(),
            line: 87,
            mutation: "replace true with false".to_string(),
        },
    ];
    receipt.evidence.mutation.killed = 15;

    let md = tokmd_cockpit::render::render_markdown(&receipt);
    insta::assert_snapshot!(md);
}

// ── Markdown: large file PR with degraded health ──────────────────────

#[test]
fn snapshot_cockpit_md_large_files() {
    let stats = vec![
        make_file_stat("src/mega.rs", 400, 200),
        make_file_stat("src/another_big.rs", 300, 250),
        make_file_stat("src/small.rs", 10, 2),
    ];
    let receipt = make_receipt(&stats);
    let md = tokmd_cockpit::render::render_markdown(&receipt);
    insta::assert_snapshot!(md);
}
