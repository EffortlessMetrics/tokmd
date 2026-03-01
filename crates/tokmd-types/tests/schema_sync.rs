//! Schema version synchronization tests.
//!
//! These tests verify that schema version constants are valid and that
//! receipt types serialize with the expected schema_version fields.

use tokmd_types::{
    CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, ChildIncludeMode, ChildrenMode,
    ConfigMode, ContextReceipt, DiffReceipt, DiffTotals, ExportArgsMeta, ExportData, ExportFormat,
    ExportReceipt, HANDOFF_SCHEMA_VERSION, HandoffManifest, LangArgsMeta, LangReceipt, LangReport,
    ModuleArgsMeta, ModuleReceipt, ModuleReport, RedactMode, SCHEMA_VERSION, ScanArgs, ScanStatus,
    ToolInfo, Totals, cockpit::COCKPIT_SCHEMA_VERSION,
};

// =============================================================================
// Schema version constants are valid
// =============================================================================

#[test]
fn schema_version_constants_are_positive() {
    assert!(SCHEMA_VERSION > 0, "SCHEMA_VERSION must be positive");
    assert!(
        COCKPIT_SCHEMA_VERSION > 0,
        "COCKPIT_SCHEMA_VERSION must be positive"
    );
    assert!(
        HANDOFF_SCHEMA_VERSION > 0,
        "HANDOFF_SCHEMA_VERSION must be positive"
    );
    assert!(
        CONTEXT_SCHEMA_VERSION > 0,
        "CONTEXT_SCHEMA_VERSION must be positive"
    );
    assert!(
        CONTEXT_BUNDLE_SCHEMA_VERSION > 0,
        "CONTEXT_BUNDLE_SCHEMA_VERSION must be positive"
    );
}

#[test]
fn schema_version_constants_match_expected_values() {
    assert_eq!(SCHEMA_VERSION, 2, "Core receipt schema version");
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3, "Cockpit schema version");
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5, "Handoff schema version");
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4, "Context schema version");
    assert_eq!(
        CONTEXT_BUNDLE_SCHEMA_VERSION, 2,
        "Context bundle schema version"
    );
}

// =============================================================================
// Helper constructors
// =============================================================================

fn tool_info() -> ToolInfo {
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

fn totals() -> Totals {
    Totals {
        code: 100,
        lines: 200,
        files: 5,
        bytes: 5000,
        tokens: 1250,
        avg_lines: 40,
    }
}

// =============================================================================
// LangReceipt serialization includes schema_version
// =============================================================================

#[test]
fn lang_receipt_json_includes_schema_version() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![],
            total: totals(),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    };

    let json = serde_json::to_value(&receipt).expect("LangReceipt must serialize");
    assert_eq!(
        json["schema_version"], SCHEMA_VERSION,
        "LangReceipt JSON must include schema_version"
    );
    assert!(json["tool"].is_object(), "LangReceipt must include tool");
    assert!(json["mode"].is_string(), "LangReceipt must include mode");
    assert!(
        json["generated_at_ms"].is_number(),
        "LangReceipt must include generated_at_ms"
    );
    assert!(
        json["status"].is_string(),
        "LangReceipt must include status"
    );
}

// =============================================================================
// ModuleReceipt serialization includes schema_version
// =============================================================================

#[test]
fn module_receipt_json_includes_schema_version() {
    let receipt = ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(),
        args: ModuleArgsMeta {
            format: "json".to_string(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
        report: ModuleReport {
            rows: vec![],
            total: totals(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
    };

    let json = serde_json::to_value(&receipt).expect("ModuleReceipt must serialize");
    assert_eq!(json["schema_version"], SCHEMA_VERSION);
    assert!(json["tool"].is_object());
    assert!(json["mode"].is_string());
}

// =============================================================================
// ExportReceipt serialization includes schema_version
// =============================================================================

#[test]
fn export_receipt_json_includes_schema_version() {
    let receipt = ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(),
        args: ExportArgsMeta {
            format: ExportFormat::Json,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        },
        data: ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        },
    };

    let json = serde_json::to_value(&receipt).expect("ExportReceipt must serialize");
    assert_eq!(json["schema_version"], SCHEMA_VERSION);
    assert!(json["tool"].is_object());
}

// =============================================================================
// DiffReceipt serialization includes schema_version
// =============================================================================

#[test]
fn diff_receipt_json_includes_schema_version() {
    let receipt = DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "diff".to_string(),
        from_source: "a.json".to_string(),
        to_source: "b.json".to_string(),
        diff_rows: vec![],
        totals: DiffTotals::default(),
    };

    let json = serde_json::to_value(&receipt).expect("DiffReceipt must serialize");
    assert_eq!(json["schema_version"], SCHEMA_VERSION);
    assert!(json["from_source"].is_string());
    assert!(json["to_source"].is_string());
}

// =============================================================================
// ContextReceipt serialization includes CONTEXT_SCHEMA_VERSION
// =============================================================================

#[test]
fn context_receipt_json_includes_schema_version() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "context".to_string(),
        budget_tokens: 128_000,
        used_tokens: 50_000,
        utilization_pct: 39.06,
        strategy: "greedy".to_string(),
        rank_by: "tokens".to_string(),
        file_count: 0,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    let json = serde_json::to_value(&receipt).expect("ContextReceipt must serialize");
    assert_eq!(json["schema_version"], CONTEXT_SCHEMA_VERSION);
    assert!(json["budget_tokens"].is_number());
    assert!(json["used_tokens"].is_number());
}

// =============================================================================
// HandoffManifest serialization includes HANDOFF_SCHEMA_VERSION
// =============================================================================

#[test]
fn handoff_manifest_json_includes_schema_version() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool_info(),
        mode: "handoff".to_string(),
        inputs: vec![".".to_string()],
        output_dir: "out".to_string(),
        budget_tokens: 128_000,
        used_tokens: 50_000,
        utilization_pct: 39.06,
        strategy: "greedy".to_string(),
        rank_by: "tokens".to_string(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 0,
        bundled_files: 0,
        intelligence_preset: "none".to_string(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    };

    let json = serde_json::to_value(&manifest).expect("HandoffManifest must serialize");
    assert_eq!(json["schema_version"], HANDOFF_SCHEMA_VERSION);
    assert!(json["output_dir"].is_string());
    assert!(json["artifacts"].is_array());
}

// =============================================================================
// Receipt envelope required fields
// =============================================================================

#[test]
fn schema_receipt_envelopes_have_required_fields() {
    // All core receipts must include: schema_version, generated_at_ms, tool, mode
    let lang = serde_json::to_value(&LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 12345,
        tool: tool_info(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![],
            total: totals(),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    })
    .unwrap();

    let required_fields = [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "status",
    ];
    for field in &required_fields {
        assert!(
            !lang[field].is_null(),
            "LangReceipt missing required field: {field}"
        );
    }
}
