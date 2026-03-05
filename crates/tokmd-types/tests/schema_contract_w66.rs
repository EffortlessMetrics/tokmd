//! Schema contract tests for `tokmd-types` receipt families.
//!
//! These tests verify that receipt JSON schemas are correct, stable,
//! and backwards-compatible. They guard against accidental breaking
//! changes to the serialized contract consumed by downstream tools.

use serde_json::Value;
use tokmd_types::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_scan_args() -> ScanArgs {
    ScanArgs {
        paths: vec![".".into()],
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

fn make_totals() -> Totals {
    Totals {
        code: 100,
        lines: 150,
        files: 5,
        bytes: 4000,
        tokens: 1000,
        avg_lines: 30,
    }
}

fn make_lang_receipt() -> LangReceipt {
    LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "lang".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: LangArgsMeta {
            format: "json".into(),
            top: 10,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![LangRow {
                lang: "Rust".into(),
                code: 100,
                lines: 150,
                files: 5,
                bytes: 4000,
                tokens: 1000,
                avg_lines: 30,
            }],
            total: make_totals(),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 10,
        },
    }
}

fn make_module_receipt() -> ModuleReceipt {
    ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "module".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: ModuleArgsMeta {
            format: "json".into(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            top: 10,
        },
        report: ModuleReport {
            rows: vec![ModuleRow {
                module: "src".into(),
                code: 100,
                lines: 150,
                files: 5,
                bytes: 4000,
                tokens: 1000,
                avg_lines: 30,
            }],
            total: make_totals(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            top: 10,
        },
    }
}

fn make_export_receipt() -> ExportReceipt {
    ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "export".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: ExportArgsMeta {
            format: ExportFormat::Json,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 1000,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        },
        data: ExportData {
            rows: vec![FileRow {
                path: "src/main.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 30,
                lines: 150,
                bytes: 4000,
                tokens: 1000,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        },
    }
}

fn make_diff_receipt() -> DiffReceipt {
    DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "diff".into(),
        from_source: "old.json".into(),
        to_source: "new.json".into(),
        diff_rows: vec![],
        totals: DiffTotals::default(),
    }
}

fn make_context_receipt() -> ContextReceipt {
    ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "context".into(),
        budget_tokens: 128_000,
        used_tokens: 50_000,
        utilization_pct: 39.06,
        strategy: "greedy".into(),
        rank_by: "tokens".into(),
        file_count: 10,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    }
}

fn make_handoff_manifest() -> HandoffManifest {
    HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "handoff".into(),
        inputs: vec![".".into()],
        output_dir: "output".into(),
        budget_tokens: 128_000,
        used_tokens: 50_000,
        utilization_pct: 39.06,
        strategy: "greedy".into(),
        rank_by: "tokens".into(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 10,
        bundled_files: 5,
        intelligence_preset: "none".into(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    }
}

// ===========================================================================
// 1. Schema version constants are positive integers
// ===========================================================================

#[test]
fn schema_version_is_positive() {
    assert!(SCHEMA_VERSION > 0);
}

#[test]
fn context_schema_version_is_positive() {
    assert!(CONTEXT_SCHEMA_VERSION > 0);
}

#[test]
fn context_bundle_schema_version_is_positive() {
    assert!(CONTEXT_BUNDLE_SCHEMA_VERSION > 0);
}

#[test]
fn cockpit_schema_version_is_positive() {
    assert!(cockpit::COCKPIT_SCHEMA_VERSION > 0);
}

#[test]
fn handoff_schema_version_is_positive() {
    assert!(HANDOFF_SCHEMA_VERSION > 0);
}

// ===========================================================================
// 2. JSON roundtrip for every receipt type
// ===========================================================================

#[test]
fn lang_receipt_json_roundtrip() {
    let receipt = make_lang_receipt();
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "lang");
}

#[test]
fn module_receipt_json_roundtrip() {
    let receipt = make_module_receipt();
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let back: ModuleReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "module");
}

#[test]
fn export_receipt_json_roundtrip() {
    let receipt = make_export_receipt();
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let back: ExportReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "export");
}

#[test]
fn diff_receipt_json_roundtrip() {
    let receipt = make_diff_receipt();
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let back: DiffReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "diff");
}

#[test]
fn context_receipt_json_roundtrip() {
    let receipt = make_context_receipt();
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(back.mode, "context");
}

#[test]
fn handoff_manifest_json_roundtrip() {
    let manifest = make_handoff_manifest();
    let json = serde_json::to_string_pretty(&manifest).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, HANDOFF_SCHEMA_VERSION);
    assert_eq!(back.mode, "handoff");
}

#[test]
fn run_receipt_json_roundtrip() {
    let receipt = RunReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        lang_file: "lang.json".into(),
        module_file: "module.json".into(),
        export_file: "export.json".into(),
    };
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let back: RunReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
}

// ===========================================================================
// 3. Receipt envelope metadata includes schema_version in JSON output
// ===========================================================================

#[test]
fn lang_receipt_envelope_has_schema_version() {
    let receipt = make_lang_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["schema_version"], SCHEMA_VERSION);
}

#[test]
fn export_receipt_envelope_has_schema_version() {
    let receipt = make_export_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["schema_version"], SCHEMA_VERSION);
}

#[test]
fn context_receipt_envelope_has_schema_version() {
    let receipt = make_context_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["schema_version"], CONTEXT_SCHEMA_VERSION);
}

#[test]
fn handoff_manifest_envelope_has_schema_version() {
    let manifest = make_handoff_manifest();
    let json = serde_json::to_string(&manifest).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["schema_version"], HANDOFF_SCHEMA_VERSION);
}

// ===========================================================================
// 4. Serde field names match expected snake_case contract
// ===========================================================================

#[test]
fn lang_receipt_field_names_are_snake_case() {
    let receipt = make_lang_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    let obj = val.as_object().unwrap();
    for key in obj.keys() {
        assert!(
            !key.contains('-') || key == "treat-doc-strings-as-comments",
            "Unexpected non-snake_case key in lang receipt: {key}"
        );
        assert!(
            *key == key.to_lowercase() || key.contains('_'),
            "Key should be lowercase or snake_case: {key}"
        );
    }
}

#[test]
fn diff_receipt_field_names_stable() {
    let receipt = make_diff_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    let obj = val.as_object().unwrap();
    let expected_keys = [
        "schema_version",
        "generated_at_ms",
        "tool",
        "mode",
        "from_source",
        "to_source",
        "diff_rows",
        "totals",
    ];
    for key in &expected_keys {
        assert!(obj.contains_key(*key), "Missing expected key: {key}");
    }
}

// ===========================================================================
// 5. Backward compat: extra fields don't break deserialization
// ===========================================================================

#[test]
fn lang_receipt_ignores_extra_fields() {
    let receipt = make_lang_receipt();
    let mut json: Value = serde_json::to_value(&receipt).unwrap();
    json["new_future_field"] = Value::String("hello".into());
    json["another_field"] = Value::Number(42.into());
    let back: LangReceipt = serde_json::from_value(json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
}

#[test]
fn export_receipt_ignores_extra_fields() {
    let receipt = make_export_receipt();
    let mut json: Value = serde_json::to_value(&receipt).unwrap();
    json["future_v3_field"] = Value::Bool(true);
    let back: ExportReceipt = serde_json::from_value(json).unwrap();
    assert_eq!(back.mode, "export");
}

#[test]
fn diff_receipt_ignores_extra_fields() {
    let receipt = make_diff_receipt();
    let mut json: Value = serde_json::to_value(&receipt).unwrap();
    json["extra"] = Value::Null;
    let back: DiffReceipt = serde_json::from_value(json).unwrap();
    assert_eq!(back.mode, "diff");
}

#[test]
fn context_receipt_ignores_extra_fields() {
    let receipt = make_context_receipt();
    let mut json: Value = serde_json::to_value(&receipt).unwrap();
    json["v5_addition"] = Value::String("compat".into());
    let back: ContextReceipt = serde_json::from_value(json).unwrap();
    assert_eq!(back.mode, "context");
}

// ===========================================================================
// 6. Enum variants serialize to expected case
// ===========================================================================

#[test]
fn scan_status_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&ScanStatus::Complete).unwrap(),
        "\"complete\""
    );
    assert_eq!(
        serde_json::to_string(&ScanStatus::Partial).unwrap(),
        "\"partial\""
    );
}

#[test]
fn children_mode_serializes_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ChildrenMode::Collapse).unwrap(),
        "\"collapse\""
    );
    assert_eq!(
        serde_json::to_string(&ChildrenMode::Separate).unwrap(),
        "\"separate\""
    );
}

#[test]
fn file_classification_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&FileClassification::Generated).unwrap(),
        "\"generated\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::DataBlob).unwrap(),
        "\"data_blob\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::Sourcemap).unwrap(),
        "\"sourcemap\""
    );
}

#[test]
fn inclusion_policy_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::Full).unwrap(),
        "\"full\""
    );
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::HeadTail).unwrap(),
        "\"head_tail\""
    );
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::Skip).unwrap(),
        "\"skip\""
    );
}

#[test]
fn commit_intent_kind_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&CommitIntentKind::Feat).unwrap(),
        "\"feat\""
    );
    assert_eq!(
        serde_json::to_string(&CommitIntentKind::Fix).unwrap(),
        "\"fix\""
    );
    assert_eq!(
        serde_json::to_string(&CommitIntentKind::Refactor).unwrap(),
        "\"refactor\""
    );
}

#[test]
fn export_format_serializes_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ExportFormat::Csv).unwrap(),
        "\"csv\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Jsonl).unwrap(),
        "\"jsonl\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Cyclonedx).unwrap(),
        "\"cyclonedx\""
    );
}
