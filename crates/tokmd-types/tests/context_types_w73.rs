//! Deep tests for context and handoff types in tokmd-types (W73).

use tokmd_types::*;

// ---------------------------------------------------------------------------
// Schema version constants
// ---------------------------------------------------------------------------

#[test]
fn context_schema_version_is_4() {
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
}

#[test]
fn context_bundle_schema_version_is_2() {
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
}

#[test]
fn handoff_schema_version_is_5() {
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
}

// ---------------------------------------------------------------------------
// InclusionPolicy defaults and serde
// ---------------------------------------------------------------------------

#[test]
fn inclusion_policy_default_is_full() {
    assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
}

#[test]
fn inclusion_policy_serde_roundtrip() {
    for policy in [
        InclusionPolicy::Full,
        InclusionPolicy::HeadTail,
        InclusionPolicy::Summary,
        InclusionPolicy::Skip,
    ] {
        let json = serde_json::to_string(&policy).unwrap();
        let back: InclusionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, policy);
    }
}

#[test]
fn inclusion_policy_snake_case_serialization() {
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

// ---------------------------------------------------------------------------
// FileClassification serde
// ---------------------------------------------------------------------------

#[test]
fn file_classification_serde_roundtrip() {
    for class in [
        FileClassification::Generated,
        FileClassification::Fixture,
        FileClassification::Vendored,
        FileClassification::Lockfile,
        FileClassification::Minified,
        FileClassification::DataBlob,
        FileClassification::Sourcemap,
    ] {
        let json = serde_json::to_string(&class).unwrap();
        let back: FileClassification = serde_json::from_str(&json).unwrap();
        assert_eq!(back, class);
    }
}

#[test]
fn file_classification_snake_case_names() {
    assert_eq!(
        serde_json::to_string(&FileClassification::DataBlob).unwrap(),
        "\"data_blob\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::Sourcemap).unwrap(),
        "\"sourcemap\""
    );
}

// ---------------------------------------------------------------------------
// ContextFileRow construction and serialization
// ---------------------------------------------------------------------------

fn sample_context_file_row() -> ContextFileRow {
    ContextFileRow {
        path: "src/main.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        tokens: 500,
        code: 120,
        lines: 150,
        bytes: 3200,
        value: 120,
        rank_reason: String::new(),
        policy: InclusionPolicy::Full,
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    }
}

#[test]
fn context_file_row_full_policy_serialization() {
    let row = sample_context_file_row();
    let json = serde_json::to_value(&row).unwrap();

    assert_eq!(json["path"], "src/main.rs");
    assert_eq!(json["tokens"], 500);
    assert_eq!(json["code"], 120);
    // Default policy (Full) is skipped by skip_serializing_if
    assert!(json.get("policy").is_none());
    // Empty rank_reason is skipped
    assert!(json.get("rank_reason").is_none());
    // None effective_tokens is skipped
    assert!(json.get("effective_tokens").is_none());
    // Empty classifications is skipped
    assert!(json.get("classifications").is_none());
}

#[test]
fn context_file_row_with_head_tail_policy() {
    let mut row = sample_context_file_row();
    row.policy = InclusionPolicy::HeadTail;
    row.effective_tokens = Some(200);
    row.policy_reason = Some("file exceeds cap".to_string());

    let json = serde_json::to_value(&row).unwrap();
    assert_eq!(json["policy"], "head_tail");
    assert_eq!(json["effective_tokens"], 200);
    assert_eq!(json["policy_reason"], "file exceeds cap");
}

#[test]
fn context_file_row_with_classifications() {
    let mut row = sample_context_file_row();
    row.classifications = vec![FileClassification::Generated, FileClassification::DataBlob];

    let json = serde_json::to_value(&row).unwrap();
    let classes = json["classifications"].as_array().unwrap();
    assert_eq!(classes.len(), 2);
    assert_eq!(classes[0], "generated");
    assert_eq!(classes[1], "data_blob");
}

#[test]
fn context_file_row_effective_tokens_relationship() {
    let mut row = sample_context_file_row();
    row.tokens = 5000;
    row.effective_tokens = Some(1000);

    // effective_tokens < tokens when HeadTail
    assert!(row.effective_tokens.unwrap() < row.tokens);
    // When Full, effective_tokens is None (same as tokens)
    row.effective_tokens = None;
    assert!(row.effective_tokens.is_none());
}

// ---------------------------------------------------------------------------
// ContextReceipt
// ---------------------------------------------------------------------------

#[test]
fn context_receipt_json_roundtrip() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo {
            name: "tokmd".to_string(),
            version: "0.1.0".to_string(),
        },
        mode: "context".to_string(),
        budget_tokens: 128_000,
        used_tokens: 50_000,
        utilization_pct: 39.06,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 10,
        files: vec![sample_context_file_row()],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    let json_str = serde_json::to_string(&receipt).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed["schema_version"], CONTEXT_SCHEMA_VERSION);
    assert_eq!(parsed["mode"], "context");
    assert_eq!(parsed["budget_tokens"], 128_000);
    assert_eq!(parsed["used_tokens"], 50_000);
    assert_eq!(parsed["file_count"], 10);
    assert!(parsed["files"].is_array());
    assert_eq!(parsed["files"].as_array().unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// HandoffManifest
// ---------------------------------------------------------------------------

#[test]
fn handoff_manifest_required_fields_present() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo {
            name: "tokmd".to_string(),
            version: "0.1.0".to_string(),
        },
        mode: "handoff".to_string(),
        inputs: vec![".".to_string()],
        output_dir: ".handoff".to_string(),
        budget_tokens: 128_000,
        used_tokens: 40_000,
        utilization_pct: 31.25,
        strategy: "greedy".to_string(),
        rank_by: "hotspot".to_string(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 50,
        bundled_files: 30,
        intelligence_preset: "risk".to_string(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    };

    let json = serde_json::to_value(&manifest).unwrap();
    assert_eq!(json["schema_version"], HANDOFF_SCHEMA_VERSION);
    assert_eq!(json["mode"], "handoff");
    assert_eq!(json["output_dir"], ".handoff");
    assert_eq!(json["intelligence_preset"], "risk");
    assert_eq!(json["total_files"], 50);
    assert_eq!(json["bundled_files"], 30);
}

#[test]
fn handoff_manifest_optional_fields_skipped_when_empty() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo {
            name: "tokmd".to_string(),
            version: "0.1.0".to_string(),
        },
        mode: "handoff".to_string(),
        inputs: vec![],
        output_dir: ".handoff".to_string(),
        budget_tokens: 0,
        used_tokens: 0,
        utilization_pct: 0.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 0,
        bundled_files: 0,
        intelligence_preset: "minimal".to_string(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    };

    let json = serde_json::to_value(&manifest).unwrap();
    assert!(json.get("rank_by_effective").is_none());
    assert!(json.get("fallback_reason").is_none());
    assert!(json.get("excluded_by_policy").is_none());
    assert!(json.get("token_estimation").is_none());
    assert!(json.get("code_audit").is_none());
}

// ---------------------------------------------------------------------------
// TokenEstimationMeta
// ---------------------------------------------------------------------------

#[test]
fn token_estimation_invariant_min_le_est_le_max() {
    let est = TokenEstimationMeta::from_bytes(4000, 4.0);
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
    assert_eq!(est.source_bytes, 4000);
    assert_eq!(est.tokens_est, 1000);
}

// ---------------------------------------------------------------------------
// TokenAudit
// ---------------------------------------------------------------------------

#[test]
fn token_audit_overhead_calculation() {
    let audit = TokenAudit::from_output(5000, 4500);
    assert_eq!(audit.output_bytes, 5000);
    assert_eq!(audit.overhead_bytes, 500);
    let expected_pct = 500.0 / 5000.0;
    assert!((audit.overhead_pct - expected_pct).abs() < 1e-10);
}
