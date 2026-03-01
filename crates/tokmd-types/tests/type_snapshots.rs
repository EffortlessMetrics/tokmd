//! Snapshot tests for full receipt and report type serialization.
//!
//! Supplements `snapshots.rs` (which covers individual rows/enums) by
//! testing complete schema-versioned receipts and composite report types.
//! Uses `insta::assert_json_snapshot!` with volatile fields redacted.

use tokmd_types::{
    ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION,
    CapabilityState, CapabilityStatus, ChildIncludeMode, ChildrenMode, ConfigMode,
    ContextBundleManifest, ContextExcludedPath, ContextFileRow, ContextLogRecord, ContextReceipt,
    DiffReceipt, DiffRow, DiffTotals, ExportArgsMeta, ExportData, ExportFormat, ExportReceipt,
    FileKind, FileRow, HANDOFF_SCHEMA_VERSION, HandoffComplexity, HandoffDerived,
    HandoffExcludedPath, HandoffHotspot, HandoffIntelligence, HandoffManifest, InclusionPolicy,
    LangArgsMeta, LangReceipt, LangReport, LangRow, ModuleArgsMeta, ModuleReceipt, ModuleReport,
    ModuleRow, PolicyExcludedFile, RedactMode, RunReceipt, SCHEMA_VERSION, ScanArgs, ScanStatus,
    SmartExcludedFile, TokenAudit, TokenEstimationMeta, ToolInfo, Totals,
    cockpit::{
        ChangeSurface, CodeHealth, CommitMatch, ComplexityIndicator, Composition, Contracts,
        Evidence, EvidenceSource, GateMeta, GateStatus, HealthWarning, MutationGate, Risk,
        RiskLevel, ScopeCoverage, TrendComparison, TrendDirection, TrendIndicator, TrendMetric,
        WarningType,
    },
};

// =============================================================================
// Helper: deterministic ToolInfo (avoids baking real version into snapshots)
// =============================================================================

fn tool() -> ToolInfo {
    ToolInfo {
        name: "tokmd".to_string(),
        version: "0.0.0-test".to_string(),
    }
}

fn scan_args() -> ScanArgs {
    ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec![],
        excluded_redacted: false,
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

fn totals(code: usize, lines: usize, files: usize) -> Totals {
    Totals {
        code,
        lines,
        files,
        bytes: code * 50,
        tokens: code * 5 / 2,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

// =============================================================================
// ScanArgs
// =============================================================================

#[test]
fn snapshot_scan_args_default() {
    insta::assert_json_snapshot!("scan_args_default", scan_args());
}

#[test]
fn snapshot_scan_args_with_excludes() {
    let args = ScanArgs {
        paths: vec!["src".to_string(), "tests".to_string()],
        excluded: vec!["target".to_string(), "*.lock".to_string()],
        excluded_redacted: true,
        config: ConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: true,
    };
    insta::assert_json_snapshot!("scan_args_with_excludes", args);
}

// =============================================================================
// LangReport — empty, single, multi-row, with children
// =============================================================================

#[test]
fn snapshot_lang_report_empty() {
    let report = LangReport {
        rows: vec![],
        total: totals(0, 0, 0),
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    insta::assert_json_snapshot!("lang_report_empty", report);
}

#[test]
fn snapshot_lang_report_single_row() {
    let report = LangReport {
        rows: vec![LangRow {
            lang: "Rust".to_string(),
            code: 5000,
            lines: 7000,
            files: 42,
            bytes: 250_000,
            tokens: 62_500,
            avg_lines: 166,
        }],
        total: totals(5000, 7000, 42),
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    insta::assert_json_snapshot!("lang_report_single_row", report);
}

#[test]
fn snapshot_lang_report_multi_row_separate() {
    let report = LangReport {
        rows: vec![
            LangRow {
                lang: "Rust".to_string(),
                code: 3000,
                lines: 4000,
                files: 30,
                bytes: 150_000,
                tokens: 37_500,
                avg_lines: 133,
            },
            LangRow {
                lang: "HTML (embedded)".to_string(),
                code: 200,
                lines: 250,
                files: 5,
                bytes: 10_000,
                tokens: 2_500,
                avg_lines: 50,
            },
        ],
        total: totals(3200, 4250, 35),
        with_files: false,
        children: ChildrenMode::Separate,
        top: 10,
    };
    insta::assert_json_snapshot!("lang_report_multi_separate", report);
}

// =============================================================================
// ModuleReport — empty, single, multi-row
// =============================================================================

#[test]
fn snapshot_module_report_empty() {
    let report = ModuleReport {
        rows: vec![],
        total: totals(0, 0, 0),
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    };
    insta::assert_json_snapshot!("module_report_empty", report);
}

#[test]
fn snapshot_module_report_single_row() {
    let report = ModuleReport {
        rows: vec![ModuleRow {
            module: "src".to_string(),
            code: 1000,
            lines: 1300,
            files: 10,
            bytes: 50_000,
            tokens: 12_500,
            avg_lines: 130,
        }],
        total: totals(1000, 1300, 10),
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
        top: 0,
    };
    insta::assert_json_snapshot!("module_report_single_row", report);
}

// =============================================================================
// ExportData — with children
// =============================================================================

#[test]
fn snapshot_export_data_with_children() {
    let data = ExportData {
        rows: vec![
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 300,
                comments: 50,
                blanks: 30,
                lines: 380,
                bytes: 15_000,
                tokens: 3_750,
            },
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Markdown (Rust)".to_string(),
                kind: FileKind::Child,
                code: 20,
                comments: 0,
                blanks: 5,
                lines: 25,
                bytes: 800,
                tokens: 200,
            },
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    insta::assert_json_snapshot!("export_data_with_children", data);
}

// =============================================================================
// LangReceipt — full schema-versioned receipt
// =============================================================================

#[test]
fn snapshot_lang_receipt() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(),
        args: LangArgsMeta {
            format: "md".to_string(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 1000,
                lines: 1200,
                files: 10,
                bytes: 50_000,
                tokens: 12_500,
                avg_lines: 120,
            }],
            total: totals(1000, 1200, 10),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    };
    insta::assert_json_snapshot!("lang_receipt", receipt);
}

#[test]
fn snapshot_lang_receipt_with_warnings() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "lang".to_string(),
        status: ScanStatus::Partial,
        warnings: vec!["some files could not be read".to_string()],
        scan: scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 5,
            with_files: true,
            children: ChildrenMode::Separate,
        },
        report: LangReport {
            rows: vec![],
            total: totals(0, 0, 0),
            with_files: true,
            children: ChildrenMode::Separate,
            top: 5,
        },
    };
    insta::assert_json_snapshot!("lang_receipt_with_warnings", receipt);
}

// =============================================================================
// ModuleReceipt
// =============================================================================

#[test]
fn snapshot_module_receipt() {
    let receipt = ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(),
        args: ModuleArgsMeta {
            format: "md".to_string(),
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
        report: ModuleReport {
            rows: vec![
                ModuleRow {
                    module: "crates/alpha".to_string(),
                    code: 800,
                    lines: 1000,
                    files: 8,
                    bytes: 40_000,
                    tokens: 10_000,
                    avg_lines: 125,
                },
                ModuleRow {
                    module: "crates/beta".to_string(),
                    code: 200,
                    lines: 250,
                    files: 3,
                    bytes: 10_000,
                    tokens: 2_500,
                    avg_lines: 83,
                },
            ],
            total: totals(1000, 1250, 11),
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
    };
    insta::assert_json_snapshot!("module_receipt", receipt);
}

// =============================================================================
// ExportReceipt
// =============================================================================

#[test]
fn snapshot_export_receipt() {
    let receipt = ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(),
        args: ExportArgsMeta {
            format: ExportFormat::Jsonl,
            module_roots: vec!["src".to_string()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        },
        data: ExportData {
            rows: vec![FileRow {
                path: "src/main.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 10,
                blanks: 5,
                lines: 65,
                bytes: 2_500,
                tokens: 625,
            }],
            module_roots: vec!["src".to_string()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        },
    };
    insta::assert_json_snapshot!("export_receipt", receipt);
}

// =============================================================================
// DiffReceipt
// =============================================================================

#[test]
fn snapshot_diff_receipt() {
    let receipt = DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "diff".to_string(),
        from_source: "v1.0.0".to_string(),
        to_source: "v2.0.0".to_string(),
        diff_rows: vec![DiffRow {
            lang: "Rust".to_string(),
            old_code: 1000,
            new_code: 1200,
            delta_code: 200,
            old_lines: 1500,
            new_lines: 1800,
            delta_lines: 300,
            old_files: 10,
            new_files: 12,
            delta_files: 2,
            old_bytes: 40_000,
            new_bytes: 48_000,
            delta_bytes: 8_000,
            old_tokens: 10_000,
            new_tokens: 12_000,
            delta_tokens: 2_000,
        }],
        totals: DiffTotals {
            old_code: 1000,
            new_code: 1200,
            delta_code: 200,
            old_lines: 1500,
            new_lines: 1800,
            delta_lines: 300,
            old_files: 10,
            new_files: 12,
            delta_files: 2,
            old_bytes: 40_000,
            new_bytes: 48_000,
            delta_bytes: 8_000,
            old_tokens: 10_000,
            new_tokens: 12_000,
            delta_tokens: 2_000,
        },
    };
    insta::assert_json_snapshot!("diff_receipt", receipt);
}

#[test]
fn snapshot_diff_receipt_empty() {
    let receipt = DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "diff".to_string(),
        from_source: "a".to_string(),
        to_source: "b".to_string(),
        diff_rows: vec![],
        totals: DiffTotals::default(),
    };
    insta::assert_json_snapshot!("diff_receipt_empty", receipt);
}

// =============================================================================
// RunReceipt
// =============================================================================

#[test]
fn snapshot_run_receipt() {
    let receipt = RunReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        lang_file: "lang.json".to_string(),
        module_file: "module.json".to_string(),
        export_file: "export.jsonl".to_string(),
    };
    insta::assert_json_snapshot!("run_receipt", receipt);
}

// =============================================================================
// ContextReceipt — minimal and full
// =============================================================================

#[test]
fn snapshot_context_receipt_minimal() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "context".to_string(),
        budget_tokens: 100_000,
        used_tokens: 5_000,
        utilization_pct: 5.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 1,
        files: vec![ContextFileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            tokens: 5_000,
            code: 2_000,
            lines: 3_000,
            bytes: 20_000,
            value: 2_000,
            rank_reason: "code".to_string(),
            policy: InclusionPolicy::Full,
            effective_tokens: None,
            policy_reason: None,
            classifications: vec![],
        }],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };
    insta::assert_json_snapshot!("context_receipt_minimal", receipt);
}

#[test]
fn snapshot_context_receipt_full() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "context".to_string(),
        budget_tokens: 128_000,
        used_tokens: 120_000,
        utilization_pct: 93.75,
        strategy: "greedy".to_string(),
        rank_by: "churn".to_string(),
        file_count: 2,
        files: vec![
            ContextFileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                tokens: 80_000,
                code: 32_000,
                lines: 48_000,
                bytes: 320_000,
                value: 100,
                rank_reason: "churn".to_string(),
                policy: InclusionPolicy::Full,
                effective_tokens: None,
                policy_reason: None,
                classifications: vec![],
            },
            ContextFileRow {
                path: "gen/proto.rs".to_string(),
                module: "gen".to_string(),
                lang: "Rust".to_string(),
                tokens: 200_000,
                code: 80_000,
                lines: 100_000,
                bytes: 800_000,
                value: 5,
                rank_reason: "churn".to_string(),
                policy: InclusionPolicy::HeadTail,
                effective_tokens: Some(40_000),
                policy_reason: Some("exceeds per-file cap".to_string()),
                classifications: vec![tokmd_types::FileClassification::Generated],
            },
        ],
        rank_by_effective: Some("code".to_string()),
        fallback_reason: Some("churn data unavailable".to_string()),
        excluded_by_policy: vec![PolicyExcludedFile {
            path: "vendor/lib.js".to_string(),
            original_tokens: 500_000,
            policy: InclusionPolicy::Skip,
            reason: "vendored file".to_string(),
            classifications: vec![tokmd_types::FileClassification::Vendored],
        }],
        token_estimation: Some(TokenEstimationMeta::from_bytes(1_120_000, 4.0)),
        bundle_audit: Some(TokenAudit::from_output(500_000, 480_000)),
    };
    insta::assert_json_snapshot!("context_receipt_full", receipt);
}

// =============================================================================
// ContextLogRecord
// =============================================================================

#[test]
fn snapshot_context_log_record() {
    let record = ContextLogRecord {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        budget_tokens: 100_000,
        used_tokens: 50_000,
        utilization_pct: 50.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 25,
        total_bytes: 200_000,
        output_destination: "stdout".to_string(),
    };
    insta::assert_json_snapshot!("context_log_record", record);
}

// =============================================================================
// HandoffManifest (minimal)
// =============================================================================

#[test]
fn snapshot_handoff_manifest_minimal() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "handoff".to_string(),
        inputs: vec![".".to_string()],
        output_dir: "handoff-output".to_string(),
        budget_tokens: 128_000,
        used_tokens: 10_000,
        utilization_pct: 7.8,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        capabilities: vec![CapabilityStatus {
            name: "git".to_string(),
            status: CapabilityState::Available,
            reason: None,
        }],
        artifacts: vec![ArtifactEntry {
            name: "code.txt".to_string(),
            path: "handoff-output/code.txt".to_string(),
            description: "Source code bundle".to_string(),
            bytes: 10_000,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: "abc123".to_string(),
            }),
        }],
        included_files: vec![],
        excluded_paths: vec![HandoffExcludedPath {
            path: "target".to_string(),
            reason: "build output".to_string(),
        }],
        excluded_patterns: vec!["*.lock".to_string()],
        smart_excluded_files: vec![SmartExcludedFile {
            path: "Cargo.lock".to_string(),
            reason: "lockfile".to_string(),
            tokens: 50_000,
        }],
        total_files: 100,
        bundled_files: 80,
        intelligence_preset: "health".to_string(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    };
    insta::assert_json_snapshot!("handoff_manifest_minimal", manifest);
}

// =============================================================================
// ContextBundleManifest
// =============================================================================

#[test]
fn snapshot_context_bundle_manifest() {
    let manifest = ContextBundleManifest {
        schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "context".to_string(),
        budget_tokens: 64_000,
        used_tokens: 32_000,
        utilization_pct: 50.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 10,
        bundle_bytes: 128_000,
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![ContextExcludedPath {
            path: "node_modules".to_string(),
            reason: "default exclude".to_string(),
        }],
        excluded_patterns: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };
    insta::assert_json_snapshot!("context_bundle_manifest", manifest);
}

// =============================================================================
// HandoffIntelligence
// =============================================================================

#[test]
fn snapshot_handoff_intelligence_empty() {
    let intel = HandoffIntelligence {
        tree: None,
        tree_depth: None,
        hotspots: None,
        complexity: None,
        derived: None,
        warnings: vec![],
    };
    insta::assert_json_snapshot!("handoff_intelligence_empty", intel);
}

#[test]
fn snapshot_handoff_intelligence_full() {
    let intel = HandoffIntelligence {
        tree: Some("src/\n  lib.rs\n  main.rs".to_string()),
        tree_depth: Some(3),
        hotspots: Some(vec![HandoffHotspot {
            path: "src/lib.rs".to_string(),
            commits: 42,
            lines: 500,
            score: 21_000,
        }]),
        complexity: Some(HandoffComplexity {
            total_functions: 100,
            avg_function_length: 25.5,
            max_function_length: 200,
            avg_cyclomatic: 3.2,
            max_cyclomatic: 15,
            high_risk_files: 2,
        }),
        derived: Some(HandoffDerived {
            total_files: 50,
            total_code: 10_000,
            total_lines: 15_000,
            total_tokens: 25_000,
            lang_count: 3,
            dominant_lang: "Rust".to_string(),
            dominant_pct: 85.0,
        }),
        warnings: vec!["git history truncated".to_string()],
    };
    insta::assert_json_snapshot!("handoff_intelligence_full", intel);
}

// =============================================================================
// Cockpit — CockpitReceipt (full composite)
// =============================================================================

#[test]
fn snapshot_cockpit_change_surface() {
    let cs = ChangeSurface {
        commits: 5,
        files_changed: 12,
        insertions: 300,
        deletions: 100,
        net_lines: 200,
        churn_velocity: 80.0,
        change_concentration: 0.65,
    };
    insta::assert_json_snapshot!("cockpit_change_surface", cs);
}

#[test]
fn snapshot_cockpit_composition() {
    let comp = Composition {
        code_pct: 70.0,
        test_pct: 20.0,
        docs_pct: 5.0,
        config_pct: 5.0,
        test_ratio: 0.29,
    };
    insta::assert_json_snapshot!("cockpit_composition", comp);
}

#[test]
fn snapshot_cockpit_code_health_with_warnings() {
    let health = CodeHealth {
        score: 65,
        grade: "C".to_string(),
        large_files_touched: 2,
        avg_file_size: 350,
        complexity_indicator: ComplexityIndicator::High,
        warnings: vec![HealthWarning {
            path: "src/monster.rs".to_string(),
            warning_type: WarningType::LargeFile,
            message: "file exceeds 500 lines".to_string(),
        }],
    };
    insta::assert_json_snapshot!("cockpit_code_health_with_warnings", health);
}

#[test]
fn snapshot_cockpit_risk() {
    let risk = Risk {
        hotspots_touched: vec!["src/core.rs".to_string()],
        bus_factor_warnings: vec!["only 1 contributor to src/core.rs".to_string()],
        level: RiskLevel::High,
        score: 75,
    };
    insta::assert_json_snapshot!("cockpit_risk", risk);
}

#[test]
fn snapshot_cockpit_contracts() {
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: true,
        breaking_indicators: 2,
    };
    insta::assert_json_snapshot!("cockpit_contracts", contracts);
}

#[test]
fn snapshot_cockpit_trend_comparison() {
    let trend = TrendComparison {
        baseline_available: true,
        baseline_path: Some("baseline.json".to_string()),
        baseline_generated_at_ms: Some(0),
        health: Some(TrendMetric {
            current: 85.0,
            previous: 80.0,
            delta: 5.0,
            delta_pct: 6.25,
            direction: TrendDirection::Improving,
        }),
        risk: Some(TrendMetric {
            current: 30.0,
            previous: 25.0,
            delta: 5.0,
            delta_pct: 20.0,
            direction: TrendDirection::Degrading,
        }),
        complexity: Some(TrendIndicator {
            direction: TrendDirection::Stable,
            summary: "complexity unchanged".to_string(),
            files_increased: 1,
            files_decreased: 1,
            avg_cyclomatic_delta: Some(0.0),
            avg_cognitive_delta: None,
        }),
    };
    insta::assert_json_snapshot!("cockpit_trend_comparison", trend);
}

#[test]
fn snapshot_cockpit_evidence_minimal() {
    let evidence = Evidence {
        overall_status: GateStatus::Skipped,
        mutation: MutationGate {
            meta: GateMeta {
                status: GateStatus::Skipped,
                source: EvidenceSource::RanLocal,
                commit_match: CommitMatch::Unknown,
                scope: ScopeCoverage {
                    relevant: vec![],
                    tested: vec![],
                    ratio: 0.0,
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
    };
    insta::assert_json_snapshot!("cockpit_evidence_minimal", evidence);
}

// =============================================================================
// Enum completeness — warning types, complexity indicators, trend directions
// =============================================================================

#[test]
fn snapshot_all_warning_types() {
    let all = vec![
        WarningType::LargeFile,
        WarningType::HighChurn,
        WarningType::LowTestCoverage,
        WarningType::ComplexChange,
        WarningType::BusFactor,
    ];
    insta::assert_json_snapshot!("warning_types", all);
}

#[test]
fn snapshot_all_complexity_indicators() {
    let all = vec![
        ComplexityIndicator::Low,
        ComplexityIndicator::Medium,
        ComplexityIndicator::High,
        ComplexityIndicator::Critical,
    ];
    insta::assert_json_snapshot!("complexity_indicators", all);
}

#[test]
fn snapshot_all_trend_directions() {
    let all = vec![
        TrendDirection::Improving,
        TrendDirection::Stable,
        TrendDirection::Degrading,
    ];
    insta::assert_json_snapshot!("trend_directions", all);
}

#[test]
fn snapshot_all_evidence_sources() {
    let all = vec![
        EvidenceSource::CiArtifact,
        EvidenceSource::Cached,
        EvidenceSource::RanLocal,
    ];
    insta::assert_json_snapshot!("evidence_sources", all);
}

#[test]
fn snapshot_all_commit_matches() {
    let all = vec![
        CommitMatch::Exact,
        CommitMatch::Partial,
        CommitMatch::Stale,
        CommitMatch::Unknown,
    ];
    insta::assert_json_snapshot!("commit_matches", all);
}
