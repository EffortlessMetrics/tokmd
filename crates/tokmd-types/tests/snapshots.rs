//! Snapshot tests for tokmd-types JSON serialization.
//!
//! Uses `insta` to capture and verify the exact JSON shape of all major types.
//! Run `cargo insta review` to update snapshots after intentional changes.

use tokmd_types::{
    ArtifactEntry, ArtifactHash, CapabilityState, CapabilityStatus,
    CommitIntentKind, ConfigMode, ContextFileRow, DiffRow, DiffTotals, ExportFormat,
    FileClassification, FileKind, FileRow, InclusionPolicy, LangRow, ModuleRow, PolicyExcludedFile,
    RedactMode, SmartExcludedFile, TableFormat, TokenAudit, TokenEstimationMeta, ToolInfo, Totals,
    cockpit::{GateStatus, MutationSurvivor, ReviewItem, RiskLevel},
};

// =============================================================================
// Core row types
// =============================================================================

#[test]
fn snapshot_lang_row() {
    let row = LangRow {
        lang: "Rust".to_string(),
        code: 5000,
        lines: 7000,
        files: 42,
        bytes: 200_000,
        tokens: 50_000,
        avg_lines: 166,
    };
    insta::assert_json_snapshot!("lang_row", row);
}

#[test]
fn snapshot_module_row() {
    let row = ModuleRow {
        module: "crates/tokmd-types".to_string(),
        code: 1200,
        lines: 1800,
        files: 8,
        bytes: 48_000,
        tokens: 12_000,
        avg_lines: 225,
    };
    insta::assert_json_snapshot!("module_row", row);
}

#[test]
fn snapshot_file_row_parent() {
    let row = FileRow {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 300,
        comments: 50,
        blanks: 40,
        lines: 390,
        bytes: 12_000,
        tokens: 3_000,
    };
    insta::assert_json_snapshot!("file_row_parent", row);
}

#[test]
fn snapshot_file_row_child() {
    let row = FileRow {
        path: "template.html".to_string(),
        module: ".".to_string(),
        lang: "HTML".to_string(),
        kind: FileKind::Child,
        code: 20,
        comments: 2,
        blanks: 5,
        lines: 27,
        bytes: 800,
        tokens: 200,
    };
    insta::assert_json_snapshot!("file_row_child", row);
}

#[test]
fn snapshot_totals() {
    let totals = Totals {
        code: 10_000,
        lines: 15_000,
        files: 100,
        bytes: 500_000,
        tokens: 125_000,
        avg_lines: 150,
    };
    insta::assert_json_snapshot!("totals", totals);
}

// =============================================================================
// Diff types
// =============================================================================

#[test]
fn snapshot_diff_row() {
    let row = DiffRow {
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
    };
    insta::assert_json_snapshot!("diff_row", row);
}

#[test]
fn snapshot_diff_totals_default() {
    let totals = DiffTotals::default();
    insta::assert_json_snapshot!("diff_totals_default", totals);
}

// =============================================================================
// Token estimation & audit
// =============================================================================

#[test]
fn snapshot_token_estimation_meta() {
    let meta = TokenEstimationMeta::from_bytes(10_000, TokenEstimationMeta::DEFAULT_BPT_EST);
    insta::assert_json_snapshot!("token_estimation_meta", meta);
}

#[test]
fn snapshot_token_audit() {
    let audit = TokenAudit::from_output(10_000, 8_000);
    insta::assert_json_snapshot!("token_audit", audit);
}

// =============================================================================
// Context file row
// =============================================================================

#[test]
fn snapshot_context_file_row_full() {
    let row = ContextFileRow {
        path: "src/main.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        tokens: 500,
        code: 200,
        lines: 300,
        bytes: 2_000,
        value: 200,
        rank_reason: "code".to_string(),
        policy: InclusionPolicy::Full,
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    };
    insta::assert_json_snapshot!("context_file_row_full", row);
}

#[test]
fn snapshot_context_file_row_head_tail() {
    let row = ContextFileRow {
        path: "gen/proto.rs".to_string(),
        module: "gen".to_string(),
        lang: "Rust".to_string(),
        tokens: 50_000,
        code: 10_000,
        lines: 15_000,
        bytes: 200_000,
        value: 10,
        rank_reason: "code".to_string(),
        policy: InclusionPolicy::HeadTail,
        effective_tokens: Some(2_000),
        policy_reason: Some("exceeds per-file cap".to_string()),
        classifications: vec![FileClassification::Generated],
    };
    insta::assert_json_snapshot!("context_file_row_head_tail", row);
}

// =============================================================================
// Enums
// =============================================================================

#[test]
fn snapshot_all_file_classifications() {
    let all = vec![
        FileClassification::Generated,
        FileClassification::Fixture,
        FileClassification::Vendored,
        FileClassification::Lockfile,
        FileClassification::Minified,
        FileClassification::DataBlob,
        FileClassification::Sourcemap,
    ];
    insta::assert_json_snapshot!("file_classifications", all);
}

#[test]
fn snapshot_all_inclusion_policies() {
    let all = vec![
        InclusionPolicy::Full,
        InclusionPolicy::HeadTail,
        InclusionPolicy::Summary,
        InclusionPolicy::Skip,
    ];
    insta::assert_json_snapshot!("inclusion_policies", all);
}

#[test]
fn snapshot_all_commit_intent_kinds() {
    let all = vec![
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
    insta::assert_json_snapshot!("commit_intent_kinds", all);
}

#[test]
fn snapshot_all_table_formats() {
    let all = vec![TableFormat::Md, TableFormat::Tsv, TableFormat::Json];
    insta::assert_json_snapshot!("table_formats", all);
}

#[test]
fn snapshot_all_export_formats() {
    let all = vec![
        ExportFormat::Csv,
        ExportFormat::Jsonl,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ];
    insta::assert_json_snapshot!("export_formats", all);
}

#[test]
fn snapshot_all_config_modes() {
    let all = vec![ConfigMode::Auto, ConfigMode::None];
    insta::assert_json_snapshot!("config_modes", all);
}

#[test]
fn snapshot_all_redact_modes() {
    let all = vec![RedactMode::None, RedactMode::Paths, RedactMode::All];
    insta::assert_json_snapshot!("redact_modes", all);
}

#[test]
fn snapshot_all_capability_states() {
    let all = vec![
        CapabilityState::Available,
        CapabilityState::Skipped,
        CapabilityState::Unavailable,
    ];
    insta::assert_json_snapshot!("capability_states", all);
}

// =============================================================================
// Composite types
// =============================================================================

#[test]
fn snapshot_tool_info_default() {
    let info = ToolInfo::default();
    insta::assert_json_snapshot!("tool_info_default", info);
}

#[test]
fn snapshot_policy_excluded_file() {
    let f = PolicyExcludedFile {
        path: "vendor/jquery.min.js".to_string(),
        original_tokens: 80_000,
        policy: InclusionPolicy::Skip,
        reason: "vendored minified file".to_string(),
        classifications: vec![FileClassification::Vendored, FileClassification::Minified],
    };
    insta::assert_json_snapshot!("policy_excluded_file", f);
}

#[test]
fn snapshot_smart_excluded_file() {
    let f = SmartExcludedFile {
        path: "Cargo.lock".to_string(),
        reason: "lockfile".to_string(),
        tokens: 120_000,
    };
    insta::assert_json_snapshot!("smart_excluded_file", f);
}

#[test]
fn snapshot_artifact_entry_with_hash() {
    let entry = ArtifactEntry {
        name: "code.txt".to_string(),
        path: "bundle/code.txt".to_string(),
        description: "Source code bundle".to_string(),
        bytes: 50_000,
        hash: Some(ArtifactHash {
            algo: "blake3".to_string(),
            hash: "a1b2c3d4e5f6".to_string(),
        }),
    };
    insta::assert_json_snapshot!("artifact_entry_with_hash", entry);
}

#[test]
fn snapshot_capability_status() {
    let status = CapabilityStatus {
        name: "git".to_string(),
        status: CapabilityState::Available,
        reason: None,
    };
    insta::assert_json_snapshot!("capability_status_available", status);
}

#[test]
fn snapshot_capability_status_unavailable() {
    let status = CapabilityStatus {
        name: "content".to_string(),
        status: CapabilityState::Unavailable,
        reason: Some("feature not enabled".to_string()),
    };
    insta::assert_json_snapshot!("capability_status_unavailable", status);
}

// =============================================================================
// Cockpit types
// =============================================================================

#[test]
fn snapshot_gate_status_variants() {
    let all = vec![
        GateStatus::Pass,
        GateStatus::Warn,
        GateStatus::Fail,
        GateStatus::Skipped,
        GateStatus::Pending,
    ];
    insta::assert_json_snapshot!("gate_status_variants", all);
}

#[test]
fn snapshot_risk_level_variants() {
    let all = vec![
        RiskLevel::Low,
        RiskLevel::Medium,
        RiskLevel::High,
        RiskLevel::Critical,
    ];
    insta::assert_json_snapshot!("risk_level_variants", all);
}

#[test]
fn snapshot_review_item() {
    let item = ReviewItem {
        path: "src/critical.rs".to_string(),
        reason: "hotspot + high complexity".to_string(),
        priority: 1,
        complexity: Some(4),
        lines_changed: Some(150),
    };
    insta::assert_json_snapshot!("review_item", item);
}

#[test]
fn snapshot_mutation_survivor() {
    let survivor = MutationSurvivor {
        file: "src/calc.rs".to_string(),
        line: 42,
        mutation: "replaced + with -".to_string(),
    };
    insta::assert_json_snapshot!("mutation_survivor", survivor);
}
