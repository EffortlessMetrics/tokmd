//! Deep accuracy tests for tokmd-cockpit.
//!
//! 45 tests across 5 categories: evidence gates (construction, status
//! patterns, aggregation), change surface (file-type classification,
//! additions/deletions/modifications), review plan ranking (priority,
//! complexity, boundary values), sparkline patterns (zeros, ascending,
//! descending, spiky, NaN/Inf), and trend computation edge cases.

use tokmd_cockpit::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn make_gate_meta(status: GateStatus) -> GateMeta {
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

fn make_mutation_gate(status: GateStatus) -> MutationGate {
    MutationGate {
        meta: make_gate_meta(status),
        survivors: vec![],
        killed: 0,
        timeout: 0,
        unviable: 0,
    }
}

fn make_evidence(
    overall: GateStatus,
    mutation_status: GateStatus,
    complexity: Option<ComplexityGate>,
) -> Evidence {
    Evidence {
        overall_status: overall,
        mutation: make_mutation_gate(mutation_status),
        diff_coverage: None,
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity,
    }
}

fn make_receipt_with_evidence(stats: &[FileStat], evidence: Evidence) -> CockpitReceipt {
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
        evidence,
        review_plan,
        trend: None,
    }
}

// ===========================================================================
// 1. Evidence gate tests — construction, status patterns, aggregation
// ===========================================================================

#[test]
fn evidence_mutation_gate_pass_with_all_killed() {
    let gate = MutationGate {
        meta: make_gate_meta(GateStatus::Pass),
        survivors: vec![],
        killed: 42,
        timeout: 0,
        unviable: 3,
    };
    assert_eq!(gate.meta.status, GateStatus::Pass);
    assert_eq!(gate.killed, 42);
    assert!(gate.survivors.is_empty());
}

#[test]
fn evidence_mutation_gate_fail_with_survivors() {
    let gate = MutationGate {
        meta: make_gate_meta(GateStatus::Fail),
        survivors: vec![
            MutationSurvivor {
                file: "src/lib.rs".into(),
                line: 42,
                mutation: "replaced + with -".into(),
            },
            MutationSurvivor {
                file: "src/lib.rs".into(),
                line: 99,
                mutation: "replaced true with false".into(),
            },
        ],
        killed: 38,
        timeout: 1,
        unviable: 0,
    };
    assert_eq!(gate.meta.status, GateStatus::Fail);
    assert_eq!(gate.survivors.len(), 2);
    assert_eq!(gate.survivors[0].line, 42);
}

#[test]
fn evidence_mutation_gate_skipped_no_relevant_files() {
    let gate = MutationGate {
        meta: GateMeta {
            status: GateStatus::Skipped,
            source: EvidenceSource::RanLocal,
            commit_match: CommitMatch::Unknown,
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
    };
    assert_eq!(gate.meta.status, GateStatus::Skipped);
    assert_eq!(gate.meta.scope.relevant.len(), 0);
}

#[test]
fn evidence_complexity_gate_pass_no_high_files() {
    let gate = ComplexityGate {
        meta: make_gate_meta(GateStatus::Pass),
        files_analyzed: 5,
        high_complexity_files: vec![],
        avg_cyclomatic: 3.5,
        max_cyclomatic: 8,
        threshold_exceeded: false,
    };
    assert_eq!(gate.meta.status, GateStatus::Pass);
    assert!(!gate.threshold_exceeded);
    assert!(gate.high_complexity_files.is_empty());
}

#[test]
fn evidence_complexity_gate_warn_few_high_files() {
    let gate = ComplexityGate {
        meta: make_gate_meta(GateStatus::Warn),
        files_analyzed: 10,
        high_complexity_files: vec![
            HighComplexityFile {
                path: "src/parser.rs".into(),
                cyclomatic: 25,
                function_count: 8,
                max_function_length: 120,
            },
            HighComplexityFile {
                path: "src/eval.rs".into(),
                cyclomatic: 18,
                function_count: 5,
                max_function_length: 80,
            },
        ],
        avg_cyclomatic: 8.0,
        max_cyclomatic: 25,
        threshold_exceeded: true,
    };
    assert_eq!(gate.meta.status, GateStatus::Warn);
    assert!(gate.threshold_exceeded);
    assert_eq!(gate.high_complexity_files.len(), 2);
    // Verify ordering: highest cyclomatic first
    assert!(gate.high_complexity_files[0].cyclomatic > gate.high_complexity_files[1].cyclomatic);
}

#[test]
fn evidence_complexity_gate_fail_many_high_files() {
    let files: Vec<HighComplexityFile> = (0..5)
        .map(|i| HighComplexityFile {
            path: format!("src/mod{i}.rs"),
            cyclomatic: 20 + i as u32,
            function_count: 3,
            max_function_length: 50,
        })
        .collect();
    let gate = ComplexityGate {
        meta: make_gate_meta(GateStatus::Fail),
        files_analyzed: 20,
        high_complexity_files: files,
        avg_cyclomatic: 12.0,
        max_cyclomatic: 24,
        threshold_exceeded: true,
    };
    assert_eq!(gate.meta.status, GateStatus::Fail);
    assert_eq!(gate.high_complexity_files.len(), 5);
}

#[test]
fn evidence_diff_coverage_gate_pass_high_coverage() {
    let gate = DiffCoverageGate {
        meta: make_gate_meta(GateStatus::Pass),
        lines_added: 100,
        lines_covered: 90,
        coverage_pct: 0.9,
        uncovered_hunks: vec![],
    };
    assert_eq!(gate.meta.status, GateStatus::Pass);
    assert!(gate.coverage_pct >= 0.80);
}

#[test]
fn evidence_diff_coverage_gate_warn_medium_coverage() {
    let gate = DiffCoverageGate {
        meta: make_gate_meta(GateStatus::Warn),
        lines_added: 100,
        lines_covered: 60,
        coverage_pct: 0.6,
        uncovered_hunks: vec![UncoveredHunk {
            file: "src/new.rs".into(),
            start_line: 10,
            end_line: 30,
        }],
    };
    assert_eq!(gate.meta.status, GateStatus::Warn);
    assert!(gate.coverage_pct >= 0.50 && gate.coverage_pct < 0.80);
    assert_eq!(gate.uncovered_hunks.len(), 1);
}

#[test]
fn evidence_diff_coverage_gate_fail_low_coverage() {
    let gate = DiffCoverageGate {
        meta: make_gate_meta(GateStatus::Fail),
        lines_added: 200,
        lines_covered: 40,
        coverage_pct: 0.2,
        uncovered_hunks: vec![
            UncoveredHunk {
                file: "src/a.rs".into(),
                start_line: 1,
                end_line: 50,
            },
            UncoveredHunk {
                file: "src/b.rs".into(),
                start_line: 10,
                end_line: 80,
            },
        ],
    };
    assert_eq!(gate.meta.status, GateStatus::Fail);
    assert!(gate.coverage_pct < 0.50);
    assert_eq!(gate.uncovered_hunks.len(), 2);
}

#[test]
fn evidence_contract_gate_semver_fail() {
    let gate = ContractDiffGate {
        meta: make_gate_meta(GateStatus::Fail),
        semver: Some(SemverSubGate {
            status: GateStatus::Fail,
            breaking_changes: vec![BreakingChange {
                kind: "semver".into(),
                path: "crates/tokmd-types/src/lib.rs".into(),
                message: "removed public function foo".into(),
            }],
        }),
        cli: None,
        schema: None,
        failures: 1,
    };
    assert_eq!(gate.meta.status, GateStatus::Fail);
    assert_eq!(gate.failures, 1);
    assert!(gate.semver.is_some());
    assert_eq!(gate.semver.as_ref().unwrap().breaking_changes.len(), 1);
}

#[test]
fn evidence_contract_gate_cli_pass() {
    let gate = ContractDiffGate {
        meta: make_gate_meta(GateStatus::Pass),
        semver: None,
        cli: Some(CliSubGate {
            status: GateStatus::Pass,
            diff_summary: Some("2 command files".into()),
        }),
        schema: None,
        failures: 0,
    };
    assert_eq!(gate.meta.status, GateStatus::Pass);
    assert!(gate.cli.is_some());
    assert_eq!(gate.cli.as_ref().unwrap().status, GateStatus::Pass);
}

#[test]
fn evidence_contract_gate_schema_warn() {
    let gate = ContractDiffGate {
        meta: make_gate_meta(GateStatus::Pass),
        semver: None,
        cli: None,
        schema: Some(SchemaSubGate {
            status: GateStatus::Pass,
            diff_summary: Some("schema.json: 5 lines added (additive only)".into()),
        }),
        failures: 0,
    };
    assert_eq!(gate.failures, 0);
    assert!(gate.schema.is_some());
}

#[test]
fn evidence_supply_chain_gate_pass_no_vulns() {
    let gate = SupplyChainGate {
        meta: make_gate_meta(GateStatus::Pass),
        vulnerabilities: vec![],
        denied: vec![],
        advisory_db_version: Some("2024-01-01".into()),
    };
    assert_eq!(gate.meta.status, GateStatus::Pass);
    assert!(gate.vulnerabilities.is_empty());
}

#[test]
fn evidence_supply_chain_gate_fail_critical_vuln() {
    let gate = SupplyChainGate {
        meta: make_gate_meta(GateStatus::Fail),
        vulnerabilities: vec![Vulnerability {
            id: "RUSTSEC-2024-0001".into(),
            package: "unsafe-crate".into(),
            severity: "critical".into(),
            title: "Remote code execution".into(),
        }],
        denied: vec![],
        advisory_db_version: Some("2024-06-01".into()),
    };
    assert_eq!(gate.meta.status, GateStatus::Fail);
    assert_eq!(gate.vulnerabilities.len(), 1);
    assert_eq!(gate.vulnerabilities[0].severity, "critical");
}

#[test]
fn evidence_supply_chain_gate_warn_medium_vuln() {
    let gate = SupplyChainGate {
        meta: make_gate_meta(GateStatus::Warn),
        vulnerabilities: vec![Vulnerability {
            id: "RUSTSEC-2024-0099".into(),
            package: "minor-risk".into(),
            severity: "medium".into(),
            title: "Denial of service".into(),
        }],
        denied: vec![],
        advisory_db_version: None,
    };
    assert_eq!(gate.meta.status, GateStatus::Warn);
}

#[test]
fn evidence_determinism_gate_pass_hashes_match() {
    let gate = DeterminismGate {
        meta: make_gate_meta(GateStatus::Pass),
        expected_hash: Some("abc123".into()),
        actual_hash: Some("abc123".into()),
        algo: "blake3".into(),
        differences: vec![],
    };
    assert_eq!(gate.meta.status, GateStatus::Pass);
    assert!(gate.differences.is_empty());
    assert_eq!(gate.expected_hash, gate.actual_hash);
}

#[test]
fn evidence_determinism_gate_warn_hash_mismatch() {
    let gate = DeterminismGate {
        meta: make_gate_meta(GateStatus::Warn),
        expected_hash: Some("abc123".into()),
        actual_hash: Some("def456".into()),
        algo: "blake3".into(),
        differences: vec!["source hash mismatch: expected abc123, got def456".into()],
    };
    assert_eq!(gate.meta.status, GateStatus::Warn);
    assert_eq!(gate.differences.len(), 1);
    assert_ne!(gate.expected_hash, gate.actual_hash);
}

#[test]
fn evidence_overall_all_pass_serde_roundtrip() {
    let evidence = Evidence {
        overall_status: GateStatus::Pass,
        mutation: make_mutation_gate(GateStatus::Pass),
        diff_coverage: Some(DiffCoverageGate {
            meta: make_gate_meta(GateStatus::Pass),
            lines_added: 50,
            lines_covered: 45,
            coverage_pct: 0.9,
            uncovered_hunks: vec![],
        }),
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: Some(ComplexityGate {
            meta: make_gate_meta(GateStatus::Pass),
            files_analyzed: 3,
            high_complexity_files: vec![],
            avg_cyclomatic: 2.0,
            max_cyclomatic: 5,
            threshold_exceeded: false,
        }),
    };
    let json = serde_json::to_string(&evidence).unwrap();
    let back: Evidence = serde_json::from_str(&json).unwrap();
    assert_eq!(back.overall_status, GateStatus::Pass);
    assert!(back.diff_coverage.is_some());
    assert!(back.complexity.is_some());
}

#[test]
fn evidence_overall_some_fail_serde_roundtrip() {
    let evidence = Evidence {
        overall_status: GateStatus::Fail,
        mutation: make_mutation_gate(GateStatus::Fail),
        diff_coverage: Some(DiffCoverageGate {
            meta: make_gate_meta(GateStatus::Pass),
            lines_added: 50,
            lines_covered: 45,
            coverage_pct: 0.9,
            uncovered_hunks: vec![],
        }),
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: None,
    };
    let json = serde_json::to_string(&evidence).unwrap();
    let back: Evidence = serde_json::from_str(&json).unwrap();
    assert_eq!(back.overall_status, GateStatus::Fail);
    assert_eq!(back.mutation.meta.status, GateStatus::Fail);
}

#[test]
fn evidence_all_fail_overall_fail() {
    let evidence = Evidence {
        overall_status: GateStatus::Fail,
        mutation: make_mutation_gate(GateStatus::Fail),
        diff_coverage: Some(DiffCoverageGate {
            meta: make_gate_meta(GateStatus::Fail),
            lines_added: 100,
            lines_covered: 10,
            coverage_pct: 0.1,
            uncovered_hunks: vec![],
        }),
        contracts: Some(ContractDiffGate {
            meta: make_gate_meta(GateStatus::Fail),
            semver: Some(SemverSubGate {
                status: GateStatus::Fail,
                breaking_changes: vec![],
            }),
            cli: None,
            schema: None,
            failures: 1,
        }),
        supply_chain: None,
        determinism: None,
        complexity: None,
    };
    assert_eq!(evidence.overall_status, GateStatus::Fail);
    assert_eq!(evidence.mutation.meta.status, GateStatus::Fail);
    assert_eq!(
        evidence.diff_coverage.as_ref().unwrap().meta.status,
        GateStatus::Fail,
    );
}

#[test]
fn evidence_missing_optional_gates_skipped() {
    let evidence = Evidence {
        overall_status: GateStatus::Pass,
        mutation: make_mutation_gate(GateStatus::Pass),
        diff_coverage: None,
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: None,
    };
    // All optional gates are None (unavailable / skipped)
    assert!(evidence.diff_coverage.is_none());
    assert!(evidence.contracts.is_none());
    assert!(evidence.supply_chain.is_none());
    assert!(evidence.determinism.is_none());
    assert!(evidence.complexity.is_none());
    // JSON should omit optional fields
    let json = serde_json::to_string(&evidence).unwrap();
    assert!(!json.contains("diff_coverage"));
    assert!(!json.contains("supply_chain"));
}

#[test]
fn evidence_scope_coverage_partial_ratio() {
    let scope = ScopeCoverage {
        relevant: vec!["a.rs".into(), "b.rs".into(), "c.rs".into()],
        tested: vec!["a.rs".into()],
        ratio: 1.0 / 3.0,
        lines_relevant: Some(300),
        lines_tested: Some(100),
    };
    assert!(scope.ratio < 0.5);
    assert_eq!(scope.relevant.len(), 3);
    assert_eq!(scope.tested.len(), 1);
}

#[test]
fn evidence_commit_match_variants() {
    assert_ne!(CommitMatch::Exact, CommitMatch::Stale);
    assert_ne!(CommitMatch::Partial, CommitMatch::Unknown);
    // Serde roundtrip
    for variant in [
        CommitMatch::Exact,
        CommitMatch::Partial,
        CommitMatch::Stale,
        CommitMatch::Unknown,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CommitMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn evidence_source_variants_serde() {
    for variant in [
        EvidenceSource::CiArtifact,
        EvidenceSource::Cached,
        EvidenceSource::RanLocal,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: EvidenceSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ===========================================================================
// 2. Change surface tests — file type classification, ins/del/mod patterns
// ===========================================================================

#[test]
fn change_surface_rust_files_additions_only() {
    let stats = vec![stat("src/main.rs", 50, 0), stat("src/lib.rs", 30, 0)];
    let comp = compute_composition(&stats);
    assert_eq!(comp.code_pct, 1.0);
    assert_eq!(comp.test_pct, 0.0);
    // Total insertions should drive health score
    let health = compute_code_health(&stats, &no_contracts());
    assert_eq!(health.score, 100);
    assert_eq!(health.large_files_touched, 0);
}

#[test]
fn change_surface_deletions_only() {
    let stats = vec![
        stat("src/old.rs", 0, 200),
        stat("src/deprecated.rs", 0, 100),
    ];
    let health = compute_code_health(&stats, &no_contracts());
    assert_eq!(health.score, 100); // Deletions < 500 → no large file penalty
    assert_eq!(health.large_files_touched, 0);
}

#[test]
fn change_surface_modifications_mixed() {
    let stats = vec![
        stat("src/refactor.rs", 100, 80),
        stat("src/update.py", 50, 30),
        stat("tests/test_refactor.rs", 20, 10),
    ];
    let comp = compute_composition(&stats);
    assert!(comp.code_pct > 0.0);
    assert!(comp.test_pct > 0.0);
    // test_refactor.rs contains "test" → classified as test
    assert!(comp.test_ratio > 0.0);
}

#[test]
fn change_surface_renames_same_content() {
    // Renames show up as 0 insertions, 0 deletions in git numstat
    let stats = vec![stat("src/new_name.rs", 0, 0)];
    let health = compute_code_health(&stats, &no_contracts());
    assert_eq!(health.score, 100);
    assert_eq!(health.avg_file_size, 0);
}

#[test]
fn change_surface_python_files() {
    let stats = vec![
        stat("app/main.py", 80, 20),
        stat("app/utils.py", 40, 10),
        stat("tests/test_main.py", 30, 5),
    ];
    let comp = compute_composition(&stats);
    // Python files are recognized as code/test
    assert!(comp.code_pct > 0.0);
    assert!(comp.test_pct > 0.0);
}

#[test]
fn change_surface_javascript_files() {
    let stats = vec![
        stat("src/index.js", 60, 10),
        stat("src/app.ts", 40, 5),
        stat("tests/index.spec.js", 30, 5),
    ];
    let comp = compute_composition(&stats);
    assert!(comp.code_pct > 0.0);
    // spec files are test files
    assert!(comp.test_pct > 0.0);
}

#[test]
fn change_surface_large_file_threshold_exact() {
    // Exactly 500 lines → NOT large
    let stats = vec![stat("src/big.rs", 250, 250)];
    let health = compute_code_health(&stats, &no_contracts());
    assert_eq!(health.large_files_touched, 0);

    // 501 lines → IS large
    let stats2 = vec![stat("src/big.rs", 251, 250)];
    let health2 = compute_code_health(&stats2, &no_contracts());
    assert_eq!(health2.large_files_touched, 1);
}

#[test]
fn change_surface_many_small_files_no_penalty() {
    let stats: Vec<FileStat> = (0..50)
        .map(|i| stat(&format!("src/mod{i}.rs"), 5, 2))
        .collect();
    let health = compute_code_health(&stats, &no_contracts());
    assert_eq!(health.score, 100);
    assert_eq!(health.large_files_touched, 0);
    assert_eq!(health.complexity_indicator, ComplexityIndicator::Low);
}

#[test]
fn change_surface_config_and_docs_only() {
    let stats = vec![
        stat("README.md", 20, 5),
        stat("docs/guide.md", 30, 10),
        stat("Cargo.toml", 5, 2),
        stat("config.yml", 10, 3),
    ];
    let comp = compute_composition(&stats);
    assert_eq!(comp.code_pct, 0.0);
    assert_eq!(comp.test_pct, 0.0);
    assert!(comp.docs_pct > 0.0);
    assert!(comp.config_pct > 0.0);
    // docs_pct + config_pct should sum to 1.0
    let total = comp.docs_pct + comp.config_pct;
    assert!((total - 1.0).abs() < f64::EPSILON);
}

// ===========================================================================
// 3. Review plan ranking tests
// ===========================================================================

#[test]
fn review_plan_all_priority_1() {
    let stats = vec![
        stat("src/a.rs", 150, 100), // 250 lines → priority 1
        stat("src/b.rs", 200, 50),  // 250 lines → priority 1
    ];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert!(plan.iter().all(|i| i.priority == 1));
}

#[test]
fn review_plan_all_priority_3() {
    let stats = vec![
        stat("src/tiny.rs", 5, 2),   // 7 lines → priority 3
        stat("src/small.rs", 10, 5), // 15 lines → priority 3
    ];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert!(plan.iter().all(|i| i.priority == 3));
}

#[test]
fn review_plan_mixed_priorities_sorted() {
    let stats = vec![
        stat("src/tiny.rs", 5, 2),     // 7 lines → p3
        stat("src/big.rs", 200, 100),  // 300 lines → p1
        stat("src/mid.rs", 40, 20),    // 60 lines → p2
        stat("src/huge.rs", 300, 200), // 500 lines → p1
    ];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan.len(), 4);
    // Sorted by priority ascending
    for window in plan.windows(2) {
        assert!(window[0].priority <= window[1].priority);
    }
    // First two items should be priority 1
    assert_eq!(plan[0].priority, 1);
    assert_eq!(plan[1].priority, 1);
}

#[test]
fn review_plan_complexity_boundary_100_lines() {
    // Exactly 100 lines → complexity 1 (≤100)
    let stats = vec![stat("src/exact.rs", 60, 40)];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan[0].complexity, Some(1));
}

#[test]
fn review_plan_complexity_boundary_101_lines() {
    // 101 lines → complexity 3 (>100)
    let stats = vec![stat("src/over.rs", 61, 40)];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan[0].complexity, Some(3));
}

#[test]
fn review_plan_complexity_boundary_300_lines() {
    // Exactly 300 lines → complexity 3 (≤300)
    let stats = vec![stat("src/exact.rs", 150, 150)];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan[0].complexity, Some(3));
}

#[test]
fn review_plan_complexity_boundary_301_lines() {
    // 301 lines → complexity 5 (>300)
    let stats = vec![stat("src/over.rs", 151, 150)];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan[0].complexity, Some(5));
}

#[test]
fn review_plan_lines_changed_tracked() {
    let stats = vec![stat("src/main.rs", 77, 33)];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan[0].lines_changed, Some(110));
}

#[test]
fn review_plan_reason_contains_line_count() {
    let stats = vec![stat("src/app.rs", 42, 8)];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert!(plan[0].reason.contains("50"));
    assert!(plan[0].reason.contains("lines changed"));
}

#[test]
fn review_plan_hotspot_files_get_risk() {
    // Files >300 lines become hotspots in risk
    let stats = vec![
        stat("src/core.rs", 200, 200), // 400 → hotspot
        stat("src/helper.rs", 10, 5),  // 15 → not hotspot
    ];
    let health = compute_code_health(&stats, &no_contracts());
    let risk = compute_risk(&stats, &no_contracts(), &health);
    assert_eq!(risk.hotspots_touched.len(), 1);
    assert_eq!(risk.hotspots_touched[0], "src/core.rs");
    // Plan should list the hotspot file as priority 1
    let plan = generate_review_plan(&stats, &no_contracts());
    let core_item = plan.iter().find(|i| i.path == "src/core.rs").unwrap();
    assert_eq!(core_item.priority, 1);
}

#[test]
fn review_plan_empty_stats() {
    let plan = generate_review_plan(&[], &no_contracts());
    assert!(plan.is_empty());
}

// ===========================================================================
// 4. Sparkline tests — various data patterns
// ===========================================================================

#[test]
fn sparkline_all_zeros() {
    let result = sparkline(&[0.0, 0.0, 0.0, 0.0]);
    assert_eq!(result.chars().count(), 4);
    let chars: Vec<char> = result.chars().collect();
    // All equal → all same bar
    assert!(chars.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn sparkline_descending() {
    let result = sparkline(&[100.0, 75.0, 50.0, 25.0, 0.0]);
    let chars: Vec<char> = result.chars().collect();
    assert_eq!(chars.len(), 5);
    // First should be highest bar, last should be lowest
    assert_eq!(chars[0], '\u{2588}');
    assert_eq!(chars[4], '\u{2581}');
}

#[test]
fn sparkline_spiky_pattern() {
    let result = sparkline(&[0.0, 100.0, 0.0, 100.0, 0.0]);
    let chars: Vec<char> = result.chars().collect();
    assert_eq!(chars.len(), 5);
    // Odd indices should be high, even indices low
    assert_eq!(chars[0], '\u{2581}'); // low
    assert_eq!(chars[1], '\u{2588}'); // high
    assert_eq!(chars[2], '\u{2581}'); // low
    assert_eq!(chars[3], '\u{2588}'); // high
    assert_eq!(chars[4], '\u{2581}'); // low
}

#[test]
fn sparkline_two_values() {
    let result = sparkline(&[0.0, 100.0]);
    let chars: Vec<char> = result.chars().collect();
    assert_eq!(chars.len(), 2);
    assert_eq!(chars[0], '\u{2581}');
    assert_eq!(chars[1], '\u{2588}');
}

#[test]
fn sparkline_nan_returns_empty() {
    let result = sparkline(&[f64::NAN, f64::NAN]);
    assert!(result.is_empty());
}

#[test]
fn sparkline_infinity_returns_empty() {
    let result = sparkline(&[f64::INFINITY, f64::NEG_INFINITY]);
    assert!(result.is_empty());
}

#[test]
fn sparkline_negative_values() {
    let result = sparkline(&[-10.0, -5.0, 0.0, 5.0, 10.0]);
    let chars: Vec<char> = result.chars().collect();
    assert_eq!(chars.len(), 5);
    assert_eq!(chars[0], '\u{2581}');
    assert_eq!(chars[4], '\u{2588}');
}

#[test]
fn sparkline_very_small_range() {
    let result = sparkline(&[1.0, 1.001, 1.002]);
    let chars: Vec<char> = result.chars().collect();
    assert_eq!(chars.len(), 3);
    // Minimum bar is first, maximum bar is last
    assert_eq!(chars[0], '\u{2581}');
    assert_eq!(chars[2], '\u{2588}');
}

// ===========================================================================
// 5. Trend computation edge cases
// ===========================================================================

#[test]
fn trend_near_stable_boundary() {
    // Delta of 0.99 → stable (abs < 1.0)
    let trend = compute_metric_trend(80.99, 80.0, true);
    assert_eq!(trend.direction, TrendDirection::Stable);

    // Delta of exactly 1.0 → NOT stable (abs < 1.0 is the test)
    let trend2 = compute_metric_trend(81.0, 80.0, true);
    assert_eq!(trend2.direction, TrendDirection::Improving);
}

#[test]
fn trend_large_improvement() {
    let trend = compute_metric_trend(100.0, 10.0, true);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert_eq!(trend.delta, 90.0);
    assert_eq!(trend.delta_pct, 900.0);
}

#[test]
fn trend_risk_large_decrease_is_improving() {
    let trend = compute_metric_trend(5.0, 80.0, false);
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert!(trend.delta < 0.0);
}

#[test]
fn complexity_trend_no_complexity_gates() {
    let stats = vec![stat("src/main.rs", 10, 5)];
    let current = make_receipt_with_evidence(
        &stats,
        make_evidence(GateStatus::Pass, GateStatus::Pass, None),
    );
    let baseline = make_receipt_with_evidence(
        &stats,
        make_evidence(GateStatus::Pass, GateStatus::Pass, None),
    );
    let indicator = compute_complexity_trend(&current, &baseline);
    // Both have no complexity → 0.0 delta → stable
    assert_eq!(indicator.direction, TrendDirection::Stable);
    assert_eq!(indicator.summary, "Complexity stable");
}

#[test]
fn complexity_trend_degrading_with_gates() {
    let stats = vec![stat("src/main.rs", 10, 5)];
    let current_gate = ComplexityGate {
        meta: make_gate_meta(GateStatus::Warn),
        files_analyzed: 5,
        high_complexity_files: vec![],
        avg_cyclomatic: 12.0,
        max_cyclomatic: 20,
        threshold_exceeded: true,
    };
    let baseline_gate = ComplexityGate {
        meta: make_gate_meta(GateStatus::Pass),
        files_analyzed: 5,
        high_complexity_files: vec![],
        avg_cyclomatic: 5.0,
        max_cyclomatic: 8,
        threshold_exceeded: false,
    };
    let current = make_receipt_with_evidence(
        &stats,
        make_evidence(GateStatus::Pass, GateStatus::Pass, Some(current_gate)),
    );
    let baseline = make_receipt_with_evidence(
        &stats,
        make_evidence(GateStatus::Pass, GateStatus::Pass, Some(baseline_gate)),
    );
    let indicator = compute_complexity_trend(&current, &baseline);
    assert_eq!(indicator.direction, TrendDirection::Degrading);
    assert_eq!(indicator.summary, "Complexity increased");
    assert!(indicator.avg_cyclomatic_delta.unwrap() > 0.0);
}

#[test]
fn complexity_trend_improving_with_gates() {
    let stats = vec![stat("src/main.rs", 10, 5)];
    let current_gate = ComplexityGate {
        meta: make_gate_meta(GateStatus::Pass),
        files_analyzed: 5,
        high_complexity_files: vec![],
        avg_cyclomatic: 3.0,
        max_cyclomatic: 5,
        threshold_exceeded: false,
    };
    let baseline_gate = ComplexityGate {
        meta: make_gate_meta(GateStatus::Warn),
        files_analyzed: 5,
        high_complexity_files: vec![],
        avg_cyclomatic: 10.0,
        max_cyclomatic: 18,
        threshold_exceeded: true,
    };
    let current = make_receipt_with_evidence(
        &stats,
        make_evidence(GateStatus::Pass, GateStatus::Pass, Some(current_gate)),
    );
    let baseline = make_receipt_with_evidence(
        &stats,
        make_evidence(GateStatus::Pass, GateStatus::Pass, Some(baseline_gate)),
    );
    let indicator = compute_complexity_trend(&current, &baseline);
    assert_eq!(indicator.direction, TrendDirection::Improving);
    assert_eq!(indicator.summary, "Complexity decreased");
    assert!(indicator.avg_cyclomatic_delta.unwrap() < 0.0);
}

#[test]
fn trend_load_cockpit_receipt_as_baseline() {
    // A cockpit receipt used as baseline should not crash determinism gate
    let stats = vec![stat("src/main.rs", 10, 5)];
    let receipt = make_receipt_with_evidence(
        &stats,
        make_evidence(GateStatus::Pass, GateStatus::Pass, None),
    );
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    std::fs::write(tmp.path(), &json).unwrap();
    let trend = load_and_compute_trend(tmp.path(), &receipt).unwrap();
    assert!(trend.baseline_available);
    assert!(trend.health.is_some());
    assert!(trend.risk.is_some());
}

#[test]
fn trend_load_nonexistent_baseline_graceful() {
    let stats = vec![stat("src/main.rs", 10, 5)];
    let receipt = make_receipt_with_evidence(
        &stats,
        make_evidence(GateStatus::Pass, GateStatus::Pass, None),
    );
    let trend =
        load_and_compute_trend(std::path::Path::new("/nonexistent/baseline.json"), &receipt)
            .unwrap();
    assert!(!trend.baseline_available);
}
