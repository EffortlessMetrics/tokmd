//! Branch-coverage tests for `render::render_comment_md`.
//!
//! Builds `CockpitReceipt` fixtures toggling each conditional in the
//! comment renderer and asserts the output reflects the receipt state.
//! The renderer is pure (`&CockpitReceipt -> String`) so all branches
//! are exercised without git or filesystem fixtures.

use tokmd_cockpit::render::render_comment_md;
use tokmd_cockpit::*;
use tokmd_types::cockpit::COCKPIT_SCHEMA_VERSION;

fn base_meta() -> GateMeta {
    GateMeta {
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
    }
}

fn base_mutation() -> MutationGate {
    MutationGate {
        meta: GateMeta {
            status: GateStatus::Skipped,
            ..base_meta()
        },
        survivors: vec![],
        killed: 0,
        timeout: 0,
        unviable: 0,
    }
}

fn base_receipt() -> CockpitReceipt {
    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 0,
        base_ref: "main".to_string(),
        head_ref: "HEAD".to_string(),
        change_surface: ChangeSurface {
            commits: 1,
            files_changed: 1,
            insertions: 10,
            deletions: 5,
            net_lines: 5,
            churn_velocity: 0.0,
            change_concentration: 0.0,
        },
        composition: Composition {
            code_pct: 1.0,
            test_pct: 0.0,
            docs_pct: 0.0,
            config_pct: 0.0,
            test_ratio: 0.0,
        },
        code_health: CodeHealth {
            score: 95,
            grade: "A".to_string(),
            large_files_touched: 0,
            avg_file_size: 100,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Low,
            score: 10,
        },
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
        evidence: Evidence {
            overall_status: GateStatus::Pass,
            mutation: base_mutation(),
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        },
        review_plan: vec![],
        trend: None,
    }
}

// ---------------------------------------------------------------------------
// Always-emitted header / summary
// ---------------------------------------------------------------------------

#[test]
fn comment_includes_glass_cockpit_header_and_change_surface_line() {
    let mut r = base_receipt();
    r.change_surface.files_changed = 4;
    r.change_surface.insertions = 42;
    r.change_surface.deletions = 7;

    let md = render_comment_md(&r);

    assert!(md.starts_with("## Glass Cockpit Summary\n"));
    assert!(md.contains("**4 files changed**, +42/-7"));
}

#[test]
fn comment_includes_health_and_risk_lines() {
    let mut r = base_receipt();
    r.code_health.score = 73;
    r.code_health.grade = "C".to_string();
    r.risk.level = RiskLevel::Medium;
    r.risk.score = 55;

    let md = render_comment_md(&r);

    assert!(md.contains("**Health**: 73/100 (C)"));
    assert!(md.contains("**Risk**: medium (55/100)"));
}

// ---------------------------------------------------------------------------
// Contract changes block (any of api/cli/schema/breaking)
// ---------------------------------------------------------------------------

#[test]
fn comment_omits_contract_block_when_no_changes() {
    let md = render_comment_md(&base_receipt());
    assert!(!md.contains("**Contract changes**"));
    assert!(!md.contains("API contract changed"));
    assert!(!md.contains("CLI contract changed"));
    assert!(!md.contains("Schema contract changed"));
    assert!(!md.contains("breaking indicator"));
}

#[test]
fn comment_emits_api_only_contract_line() {
    let mut r = base_receipt();
    r.contracts.api_changed = true;

    let md = render_comment_md(&r);

    assert!(md.contains("**Contract changes**"));
    assert!(md.contains("- API contract changed"));
    assert!(!md.contains("- CLI contract changed"));
    assert!(!md.contains("- Schema contract changed"));
}

#[test]
fn comment_emits_cli_only_contract_line() {
    let mut r = base_receipt();
    r.contracts.cli_changed = true;

    let md = render_comment_md(&r);

    assert!(md.contains("- CLI contract changed"));
    assert!(!md.contains("- API contract changed"));
    assert!(!md.contains("- Schema contract changed"));
}

#[test]
fn comment_emits_schema_only_contract_line() {
    let mut r = base_receipt();
    r.contracts.schema_changed = true;

    let md = render_comment_md(&r);

    assert!(md.contains("- Schema contract changed"));
    assert!(!md.contains("- API contract changed"));
    assert!(!md.contains("- CLI contract changed"));
}

#[test]
fn comment_emits_all_three_contract_lines() {
    let mut r = base_receipt();
    r.contracts.api_changed = true;
    r.contracts.cli_changed = true;
    r.contracts.schema_changed = true;

    let md = render_comment_md(&r);

    assert!(md.contains("- API contract changed"));
    assert!(md.contains("- CLI contract changed"));
    assert!(md.contains("- Schema contract changed"));
}

#[test]
fn comment_includes_breaking_indicator_count_when_nonzero() {
    let mut r = base_receipt();
    r.contracts.api_changed = true;
    r.contracts.breaking_indicators = 3;

    let md = render_comment_md(&r);

    assert!(md.contains("- 3 breaking indicator(s)"));
}

#[test]
fn comment_omits_breaking_indicator_count_when_zero() {
    let mut r = base_receipt();
    r.contracts.api_changed = true;
    r.contracts.breaking_indicators = 0;

    let md = render_comment_md(&r);

    assert!(!md.contains("breaking indicator"));
}

// ---------------------------------------------------------------------------
// Evidence gate detail lines
// ---------------------------------------------------------------------------

#[test]
fn comment_omits_mutation_line_when_no_survivors() {
    let md = render_comment_md(&base_receipt());
    assert!(!md.contains("Mutation:"));
    assert!(!md.contains("survivors detected"));
}

#[test]
fn comment_emits_mutation_survivors_count() {
    let mut r = base_receipt();
    r.evidence.mutation.survivors = vec![
        MutationSurvivor {
            file: "src/lib.rs".to_string(),
            line: 12,
            mutation: "replace `+` with `-`".to_string(),
        },
        MutationSurvivor {
            file: "src/lib.rs".to_string(),
            line: 30,
            mutation: "delete `&& x`".to_string(),
        },
    ];

    let md = render_comment_md(&r);

    assert!(md.contains("- Mutation: 2 survivors detected"));
}

#[test]
fn comment_omits_diff_coverage_line_when_none() {
    let md = render_comment_md(&base_receipt());
    assert!(!md.contains("Diff coverage"));
}

#[test]
fn comment_emits_diff_coverage_percent_with_one_decimal() {
    let mut r = base_receipt();
    r.evidence.diff_coverage = Some(DiffCoverageGate {
        meta: base_meta(),
        lines_added: 10,
        lines_covered: 7,
        coverage_pct: 0.732,
        uncovered_hunks: vec![],
    });

    let md = render_comment_md(&r);

    assert!(
        md.contains("- Diff coverage: 73.2%"),
        "unexpected diff coverage line in: {md}"
    );
}

#[test]
fn comment_omits_contract_failure_line_when_no_failures() {
    let mut r = base_receipt();
    r.evidence.contracts = Some(ContractDiffGate {
        meta: base_meta(),
        semver: None,
        cli: None,
        schema: None,
        failures: 0,
    });

    let md = render_comment_md(&r);

    assert!(!md.contains("Contracts:"));
    assert!(!md.contains("sub-gate(s) failed"));
}

#[test]
fn comment_emits_contract_failure_count_when_nonzero() {
    let mut r = base_receipt();
    r.evidence.contracts = Some(ContractDiffGate {
        meta: base_meta(),
        semver: None,
        cli: None,
        schema: None,
        failures: 2,
    });

    let md = render_comment_md(&r);

    assert!(md.contains("- Contracts: 2 sub-gate(s) failed"));
}

#[test]
fn comment_omits_supply_chain_line_when_no_vulnerabilities() {
    let mut r = base_receipt();
    r.evidence.supply_chain = Some(SupplyChainGate {
        meta: base_meta(),
        vulnerabilities: vec![],
        denied: vec![],
        advisory_db_version: None,
    });

    let md = render_comment_md(&r);

    assert!(!md.contains("Supply chain:"));
}

#[test]
fn comment_emits_supply_chain_vuln_count() {
    let mut r = base_receipt();
    r.evidence.supply_chain = Some(SupplyChainGate {
        meta: base_meta(),
        vulnerabilities: vec![Vulnerability {
            id: "RUSTSEC-2024-0001".to_string(),
            package: "openssl".to_string(),
            severity: "high".to_string(),
            title: "buffer overflow".to_string(),
        }],
        denied: vec![],
        advisory_db_version: None,
    });

    let md = render_comment_md(&r);

    assert!(md.contains("- Supply chain: 1 vulnerability/vulnerabilities"));
}

#[test]
fn comment_omits_complexity_line_when_threshold_not_exceeded() {
    let mut r = base_receipt();
    r.evidence.complexity = Some(ComplexityGate {
        meta: base_meta(),
        files_analyzed: 5,
        high_complexity_files: vec![],
        avg_cyclomatic: 4.0,
        max_cyclomatic: 9,
        threshold_exceeded: false,
    });

    let md = render_comment_md(&r);

    assert!(!md.contains("Complexity:"));
    assert!(!md.contains("threshold exceeded"));
}

#[test]
fn comment_emits_complexity_threshold_warning_with_max_cyclomatic() {
    let mut r = base_receipt();
    r.evidence.complexity = Some(ComplexityGate {
        meta: base_meta(),
        files_analyzed: 5,
        high_complexity_files: vec![],
        avg_cyclomatic: 12.0,
        max_cyclomatic: 27,
        threshold_exceeded: true,
    });

    let md = render_comment_md(&r);

    assert!(md.contains("- Complexity: threshold exceeded (max cyclomatic: 27)"));
}

// ---------------------------------------------------------------------------
// Next-steps section: GateStatus arms
// ---------------------------------------------------------------------------

#[test]
fn next_steps_fail_status_asks_to_address_gates() {
    let mut r = base_receipt();
    r.evidence.overall_status = GateStatus::Fail;
    let md = render_comment_md(&r);
    assert!(md.contains("Address failing evidence gates before merge"));
    assert!(!md.contains("Proceed with reviewer sign-off"));
}

#[test]
fn next_steps_warn_status_asks_to_capture_risk_acceptance() {
    let mut r = base_receipt();
    r.evidence.overall_status = GateStatus::Warn;
    let md = render_comment_md(&r);
    assert!(md.contains("Review warning evidence gates and capture risk acceptance"));
}

#[test]
fn next_steps_pass_status_asks_for_signoff() {
    let mut r = base_receipt();
    r.evidence.overall_status = GateStatus::Pass;
    let md = render_comment_md(&r);
    assert!(md.contains("Proceed with reviewer sign-off"));
}

#[test]
fn next_steps_skipped_status_asks_to_capture_evidence() {
    let mut r = base_receipt();
    r.evidence.overall_status = GateStatus::Skipped;
    let md = render_comment_md(&r);
    assert!(md.contains("Capture missing or pending evidence before relying on this packet"));
}

#[test]
fn next_steps_pending_status_asks_to_capture_evidence() {
    let mut r = base_receipt();
    r.evidence.overall_status = GateStatus::Pending;
    let md = render_comment_md(&r);
    assert!(md.contains("Capture missing or pending evidence before relying on this packet"));
}

// ---------------------------------------------------------------------------
// Next-steps section: breaking-changes / risk-level reviewer steps
// ---------------------------------------------------------------------------

#[test]
fn next_steps_includes_breaking_changes_step_when_indicators_present() {
    let mut r = base_receipt();
    r.contracts.breaking_indicators = 1;
    let md = render_comment_md(&r);
    assert!(md.contains("Confirm breaking changes are documented"));
}

#[test]
fn next_steps_omits_breaking_changes_step_when_no_indicators() {
    let md = render_comment_md(&base_receipt());
    assert!(!md.contains("Confirm breaking changes are documented"));
}

#[test]
fn next_steps_includes_domain_reviewer_for_high_risk() {
    let mut r = base_receipt();
    r.risk.level = RiskLevel::High;
    let md = render_comment_md(&r);
    assert!(md.contains("Add a domain reviewer for high-risk files"));
}

#[test]
fn next_steps_includes_domain_reviewer_for_critical_risk() {
    let mut r = base_receipt();
    r.risk.level = RiskLevel::Critical;
    let md = render_comment_md(&r);
    assert!(md.contains("Add a domain reviewer for high-risk files"));
}

#[test]
fn next_steps_omits_domain_reviewer_for_low_risk() {
    let mut r = base_receipt();
    r.risk.level = RiskLevel::Low;
    let md = render_comment_md(&r);
    assert!(!md.contains("Add a domain reviewer for high-risk files"));
}

#[test]
fn next_steps_omits_domain_reviewer_for_medium_risk() {
    let mut r = base_receipt();
    r.risk.level = RiskLevel::Medium;
    let md = render_comment_md(&r);
    assert!(!md.contains("Add a domain reviewer for high-risk files"));
}

// ---------------------------------------------------------------------------
// Priority review items section
// ---------------------------------------------------------------------------

#[test]
fn priority_items_section_omitted_when_no_high_priority_items() {
    let mut r = base_receipt();
    r.review_plan = vec![ReviewItem {
        path: "src/low.rs".to_string(),
        reason: "small change".to_string(),
        priority: 3,
        complexity: None,
        lines_changed: None,
    }];

    let md = render_comment_md(&r);

    assert!(!md.contains("Priority review items"));
    assert!(!md.contains("src/low.rs"));
}

#[test]
fn priority_items_section_includes_priority_one_and_two() {
    let mut r = base_receipt();
    r.review_plan = vec![
        ReviewItem {
            path: "src/p1.rs".to_string(),
            reason: "API surface".to_string(),
            priority: 1,
            complexity: None,
            lines_changed: None,
        },
        ReviewItem {
            path: "src/p2.rs".to_string(),
            reason: "hotspot".to_string(),
            priority: 2,
            complexity: None,
            lines_changed: None,
        },
        ReviewItem {
            path: "src/p3.rs".to_string(),
            reason: "minor".to_string(),
            priority: 3,
            complexity: None,
            lines_changed: None,
        },
    ];

    let md = render_comment_md(&r);

    assert!(md.contains("**Priority review items**"));
    assert!(md.contains("- src/p1.rs (API surface)"));
    assert!(md.contains("- src/p2.rs (hotspot)"));
    assert!(!md.contains("src/p3.rs"));
}

// ---------------------------------------------------------------------------
// Evidence availability summary line is always emitted
// ---------------------------------------------------------------------------

#[test]
fn comment_always_emits_evidence_availability_counts_line() {
    let md = render_comment_md(&base_receipt());
    assert!(md.contains("**Evidence availability**"));
    assert!(md.contains("available"));
    assert!(md.contains("degraded"));
    assert!(md.contains("stale"));
    assert!(md.contains("skipped"));
    assert!(md.contains("unavailable"));
    assert!(md.contains("missing"));
}
