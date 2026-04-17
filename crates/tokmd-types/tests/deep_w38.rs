//! Deep tests for tokmd-types: receipts, serde, token estimation, schema constants.

use serde_json::{Value, json};
use tokmd_types::cockpit::*;
use tokmd_types::*;

// =============================================================================
// Schema version constants
// =============================================================================

#[test]
fn schema_version_is_2() {
    assert_eq!(SCHEMA_VERSION, 2);
}

#[test]
fn handoff_schema_version_is_5() {
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
}

#[test]
fn context_bundle_schema_version_is_2() {
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
}

#[test]
fn context_schema_version_is_4() {
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
}

#[test]
fn cockpit_schema_version_is_3() {
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3);
}

// =============================================================================
// LangReceipt serde roundtrip
// =============================================================================

fn make_scan_args() -> ScanArgs {
    ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec!["target".to_string()],
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

#[test]
fn lang_receipt_roundtrip() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 12345,
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec!["warn1".to_string()],
        scan: make_scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 10,
            with_files: true,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![LangRow {
                lang: "Rust".to_string(),
                code: 1000,
                lines: 1500,
                files: 10,
                bytes: 50000,
                tokens: 12500,
                avg_lines: 150,
            }],
            total: Totals {
                code: 1000,
                lines: 1500,
                files: 10,
                bytes: 50000,
                tokens: 12500,
                avg_lines: 150,
            },
            with_files: true,
            children: ChildrenMode::Collapse,
            top: 10,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "lang");
    assert_eq!(back.report.rows.len(), 1);
    assert_eq!(back.report.rows[0].lang, "Rust");
    assert_eq!(back.report.total.code, 1000);
}

#[test]
fn lang_receipt_json_has_flattened_report() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1,
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![],
            total: Totals {
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    };

    let v: Value = serde_json::to_value(receipt).unwrap();
    // `rows` should be at top level due to #[serde(flatten)]
    assert!(
        v.get("rows").is_some(),
        "rows should be flattened to top level"
    );
    assert!(v.get("total").is_some(), "total should be flattened");
}

// =============================================================================
// ModuleReceipt serde roundtrip
// =============================================================================

#[test]
fn module_receipt_roundtrip() {
    let receipt = ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 99,
        tool: ToolInfo::current(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: ModuleArgsMeta {
            format: "json".to_string(),
            module_roots: vec!["src".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 5,
        },
        report: ModuleReport {
            rows: vec![ModuleRow {
                module: "src".to_string(),
                code: 500,
                lines: 700,
                files: 3,
                bytes: 25000,
                tokens: 6250,
                avg_lines: 233,
            }],
            total: Totals {
                code: 500,
                lines: 700,
                files: 3,
                bytes: 25000,
                tokens: 6250,
                avg_lines: 233,
            },
            module_roots: vec!["src".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 5,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: ModuleReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.mode, "module");
    assert_eq!(back.report.rows[0].module, "src");
}

// =============================================================================
// ExportReceipt serde roundtrip
// =============================================================================

#[test]
fn export_receipt_roundtrip() {
    let receipt = ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 42,
        tool: ToolInfo::current(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: ExportArgsMeta {
            format: ExportFormat::Json,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::ParentsOnly,
            min_code: 0,
            max_rows: 10000,
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
                code: 100,
                comments: 20,
                blanks: 10,
                lines: 130,
                bytes: 5000,
                tokens: 1250,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::ParentsOnly,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: ExportReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.mode, "export");
    assert_eq!(back.data.rows.len(), 1);
    assert_eq!(back.data.rows[0].path, "src/main.rs");
    assert_eq!(back.data.rows[0].kind, FileKind::Parent);
}

// =============================================================================
// DiffReceipt serde roundtrip
// =============================================================================

#[test]
fn diff_receipt_roundtrip() {
    let receipt = DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 7,
        tool: ToolInfo::current(),
        mode: "diff".to_string(),
        from_source: "v1.0".to_string(),
        to_source: "v2.0".to_string(),
        diff_rows: vec![DiffRow {
            lang: "Rust".to_string(),
            old_code: 100,
            new_code: 200,
            delta_code: 100,
            old_lines: 150,
            new_lines: 300,
            delta_lines: 150,
            old_files: 5,
            new_files: 8,
            delta_files: 3,
            old_bytes: 5000,
            new_bytes: 10000,
            delta_bytes: 5000,
            old_tokens: 1250,
            new_tokens: 2500,
            delta_tokens: 1250,
        }],
        totals: DiffTotals {
            old_code: 100,
            new_code: 200,
            delta_code: 100,
            old_lines: 150,
            new_lines: 300,
            delta_lines: 150,
            old_files: 5,
            new_files: 8,
            delta_files: 3,
            old_bytes: 5000,
            new_bytes: 10000,
            delta_bytes: 5000,
            old_tokens: 1250,
            new_tokens: 2500,
            delta_tokens: 1250,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: DiffReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.mode, "diff");
    assert_eq!(back.from_source, "v1.0");
    assert_eq!(back.to_source, "v2.0");
    assert_eq!(back.diff_rows[0].delta_code, 100);
    assert_eq!(back.totals.delta_tokens, 1250);
}

// =============================================================================
// ScanArgs construction and metadata
// =============================================================================

#[test]
fn scan_args_excluded_redacted_skips_when_false() {
    let args = ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec!["secret_dir".to_string()],
        excluded_redacted: false,
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    };

    let json = serde_json::to_string(&args).unwrap();
    // excluded_redacted should be skipped when false
    assert!(
        !json.contains("excluded_redacted"),
        "excluded_redacted should be skipped via skip_serializing_if"
    );
}

#[test]
fn scan_args_excluded_redacted_present_when_true() {
    let args = ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec![],
        excluded_redacted: true,
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    };

    let json = serde_json::to_string(&args).unwrap();
    assert!(json.contains("excluded_redacted"));
}

// =============================================================================
// TokenEstimationMeta
// =============================================================================

#[test]
fn token_estimation_from_bytes_default_divisors() {
    let est = TokenEstimationMeta::from_bytes(8000, TokenEstimationMeta::DEFAULT_BPT_EST);
    assert_eq!(est.source_bytes, 8000);
    assert_eq!(est.tokens_est, 2000); // 8000 / 4.0
    assert_eq!(est.tokens_min, 1600); // 8000 / 5.0
    assert_eq!(est.tokens_max, 2667); // ceil(8000 / 3.0)
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
}

#[test]
fn token_estimation_custom_bounds() {
    let est = TokenEstimationMeta::from_bytes_with_bounds(1200, 4.0, 2.0, 6.0);
    assert_eq!(est.tokens_est, 300); // 1200 / 4.0
    assert_eq!(est.tokens_min, 200); // 1200 / 6.0
    assert_eq!(est.tokens_max, 600); // 1200 / 2.0
}

#[test]
fn token_estimation_zero_bytes_all_zero() {
    let est = TokenEstimationMeta::from_bytes(0, 4.0);
    assert_eq!(est.tokens_min, 0);
    assert_eq!(est.tokens_est, 0);
    assert_eq!(est.tokens_max, 0);
    assert_eq!(est.source_bytes, 0);
}

#[test]
fn token_estimation_one_byte() {
    let est = TokenEstimationMeta::from_bytes(1, 4.0);
    // ceil(1/5.0) = 1, ceil(1/4.0) = 1, ceil(1/3.0) = 1
    assert_eq!(est.tokens_min, 1);
    assert_eq!(est.tokens_est, 1);
    assert_eq!(est.tokens_max, 1);
}

#[test]
fn token_estimation_serde_with_aliases() {
    // Verify that tokens_low alias maps to tokens_max
    let json = json!({
        "bytes_per_token_est": 4.0,
        "bytes_per_token_low": 3.0,
        "bytes_per_token_high": 5.0,
        "tokens_high": 100,
        "tokens_est": 200,
        "tokens_low": 300,
        "source_bytes": 800
    });
    let est: TokenEstimationMeta = serde_json::from_value(json).unwrap();
    assert_eq!(est.tokens_min, 100);
    assert_eq!(est.tokens_est, 200);
    assert_eq!(est.tokens_max, 300);
}

// =============================================================================
// TokenAudit
// =============================================================================

#[test]
fn token_audit_basic() {
    let audit = TokenAudit::from_output(2000, 1500);
    assert_eq!(audit.output_bytes, 2000);
    assert_eq!(audit.overhead_bytes, 500);
    assert!((audit.overhead_pct - 0.25).abs() < 1e-10);
}

#[test]
fn token_audit_zero_output_zero_pct() {
    let audit = TokenAudit::from_output(0, 0);
    assert_eq!(audit.overhead_pct, 0.0);
    assert_eq!(audit.tokens_est, 0);
}

#[test]
fn token_audit_content_exceeds_output_saturates() {
    let audit = TokenAudit::from_output(100, 500);
    assert_eq!(audit.overhead_bytes, 0);
    assert_eq!(audit.overhead_pct, 0.0);
}

#[test]
fn token_audit_custom_divisors() {
    let audit = TokenAudit::from_output_with_divisors(1000, 800, 4.0, 2.0, 8.0);
    assert_eq!(audit.tokens_est, 250); // 1000 / 4.0
    assert_eq!(audit.tokens_min, 125); // 1000 / 8.0
    assert_eq!(audit.tokens_max, 500); // 1000 / 2.0
    assert_eq!(audit.overhead_bytes, 200);
}

// =============================================================================
// ToolInfo
// =============================================================================

#[test]
fn tool_info_current_has_name_and_version() {
    let ti = ToolInfo::current();
    assert_eq!(ti.name, "tokmd");
    assert!(!ti.version.is_empty());
}

#[test]
fn tool_info_default_is_empty() {
    let ti = ToolInfo::default();
    assert!(ti.name.is_empty());
    assert!(ti.version.is_empty());
}

// =============================================================================
// Display/Debug for enums
// =============================================================================

#[test]
fn risk_level_display_all_variants() {
    assert_eq!(format!("{}", RiskLevel::Low), "low");
    assert_eq!(format!("{}", RiskLevel::Medium), "medium");
    assert_eq!(format!("{}", RiskLevel::High), "high");
    assert_eq!(format!("{}", RiskLevel::Critical), "critical");
}

#[test]
fn gate_status_serde_all_variants() {
    let variants = [
        (GateStatus::Pass, "\"pass\""),
        (GateStatus::Warn, "\"warn\""),
        (GateStatus::Fail, "\"fail\""),
        (GateStatus::Skipped, "\"skipped\""),
        (GateStatus::Pending, "\"pending\""),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
        let back: GateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn trend_direction_serde_all_variants() {
    let variants = [
        (TrendDirection::Improving, "\"improving\""),
        (TrendDirection::Stable, "\"stable\""),
        (TrendDirection::Degrading, "\"degrading\""),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
        let back: TrendDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn complexity_indicator_serde_all_variants() {
    let variants = [
        (ComplexityIndicator::Low, "\"low\""),
        (ComplexityIndicator::Medium, "\"medium\""),
        (ComplexityIndicator::High, "\"high\""),
        (ComplexityIndicator::Critical, "\"critical\""),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
    }
}

#[test]
fn warning_type_serde_all_variants() {
    let variants = [
        WarningType::LargeFile,
        WarningType::HighChurn,
        WarningType::LowTestCoverage,
        WarningType::ComplexChange,
        WarningType::BusFactor,
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let back: WarningType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn evidence_source_serde_all_variants() {
    let variants = [
        (EvidenceSource::CiArtifact, "\"ci_artifact\""),
        (EvidenceSource::Cached, "\"cached\""),
        (EvidenceSource::RanLocal, "\"ran_local\""),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
    }
}

#[test]
fn commit_match_serde_all_variants() {
    let variants = [
        (CommitMatch::Exact, "\"exact\""),
        (CommitMatch::Partial, "\"partial\""),
        (CommitMatch::Stale, "\"stale\""),
        (CommitMatch::Unknown, "\"unknown\""),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
    }
}

// =============================================================================
// Default implementations
// =============================================================================

#[test]
fn config_mode_default_auto() {
    assert_eq!(ConfigMode::default(), ConfigMode::Auto);
}

#[test]
fn inclusion_policy_default_full() {
    assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
}

#[test]
fn diff_totals_default_zeroed() {
    let dt = DiffTotals::default();
    assert_eq!(dt.old_code, 0);
    assert_eq!(dt.new_code, 0);
    assert_eq!(dt.delta_code, 0);
    assert_eq!(dt.old_tokens, 0);
    assert_eq!(dt.new_tokens, 0);
    assert_eq!(dt.delta_tokens, 0);
    assert_eq!(dt.old_files, 0);
    assert_eq!(dt.new_files, 0);
    assert_eq!(dt.delta_files, 0);
}

#[test]
fn trend_comparison_default() {
    let tc = TrendComparison::default();
    assert!(!tc.baseline_available);
    assert!(tc.baseline_path.is_none());
    assert!(tc.health.is_none());
    assert!(tc.risk.is_none());
    assert!(tc.complexity.is_none());
}

// =============================================================================
// ArtifactEntry serde
// =============================================================================

#[test]
fn artifact_entry_serde_roundtrip() {
    let entry = ArtifactEntry {
        name: "receipt.json".to_string(),
        path: "output/receipt.json".to_string(),
        description: "JSON receipt".to_string(),
        bytes: 4096,
        hash: Some(ArtifactHash {
            algo: "blake3".to_string(),
            hash: "abc123".to_string(),
        }),
    };
    let json = serde_json::to_string(&entry).unwrap();
    let back: ArtifactEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "receipt.json");
    assert_eq!(back.hash.unwrap().algo, "blake3");
}

#[test]
fn artifact_entry_hash_optional_skip() {
    let entry = ArtifactEntry {
        name: "f.txt".to_string(),
        path: "f.txt".to_string(),
        description: "test".to_string(),
        bytes: 1,
        hash: None,
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(!json.contains("hash"), "hash should be skipped when None");
}

// =============================================================================
// CockpitReceipt full serde
// =============================================================================

#[test]
fn cockpit_receipt_full_roundtrip_with_trend() {
    let receipt = CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 999,
        base_ref: "main".to_string(),
        head_ref: "feat-branch".to_string(),
        change_surface: ChangeSurface {
            commits: 3,
            files_changed: 5,
            insertions: 100,
            deletions: 50,
            net_lines: 50,
            churn_velocity: 50.0,
            change_concentration: 0.6,
        },
        composition: Composition {
            code_pct: 0.8,
            test_pct: 0.1,
            docs_pct: 0.05,
            config_pct: 0.05,
            test_ratio: 0.125,
        },
        code_health: CodeHealth {
            score: 95,
            grade: "A".to_string(),
            large_files_touched: 0,
            avg_file_size: 50,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Low,
            score: 5,
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
        },
        review_plan: vec![ReviewItem {
            path: "src/lib.rs".to_string(),
            reason: "20 lines changed".to_string(),
            priority: 2,
            complexity: Some(1),
            lines_changed: Some(20),
        }],
        trend: Some(TrendComparison {
            baseline_available: true,
            baseline_path: Some("baseline.json".to_string()),
            baseline_generated_at_ms: Some(500),
            health: Some(TrendMetric {
                current: 95.0,
                previous: 90.0,
                delta: 5.0,
                delta_pct: 5.56,
                direction: TrendDirection::Improving,
            }),
            risk: None,
            complexity: None,
        }),
    };

    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(back.head_ref, "feat-branch");
    assert!(back.trend.is_some());
    let trend = back.trend.unwrap();
    assert!(trend.baseline_available);
    assert_eq!(trend.health.unwrap().direction, TrendDirection::Improving);
}

// =============================================================================
// FileClassification ordering
// =============================================================================

#[test]
fn file_classification_ord_stable() {
    let mut classes = [
        FileClassification::Vendored,
        FileClassification::Generated,
        FileClassification::Lockfile,
    ];
    classes.sort();
    // Derived Ord follows declaration order: Generated < Vendored < Lockfile
    assert_eq!(classes[0], FileClassification::Generated);
}

// =============================================================================
// CapabilityState serde
// =============================================================================

#[test]
fn capability_state_snake_case() {
    assert_eq!(
        serde_json::to_string(&CapabilityState::Available).unwrap(),
        "\"available\""
    );
    assert_eq!(
        serde_json::to_string(&CapabilityState::Skipped).unwrap(),
        "\"skipped\""
    );
    assert_eq!(
        serde_json::to_string(&CapabilityState::Unavailable).unwrap(),
        "\"unavailable\""
    );
}

// =============================================================================
// CommitIntentKind serde
// =============================================================================

#[test]
fn commit_intent_kind_all_variants_roundtrip() {
    let variants = [
        CommitIntentKind::Feat,
        CommitIntentKind::Fix,
        CommitIntentKind::Refactor,
        CommitIntentKind::Docs,
        CommitIntentKind::Test,
        CommitIntentKind::Chore,
        CommitIntentKind::Ci,
        CommitIntentKind::Build,
        CommitIntentKind::Perf,
        CommitIntentKind::Style,
        CommitIntentKind::Revert,
        CommitIntentKind::Other,
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CommitIntentKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}
