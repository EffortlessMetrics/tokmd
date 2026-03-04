//! W47 deep tests for context/handoff types in `tokmd-types`.
//!
//! Covers: ContextReceipt structure, HandoffManifest structure, ContextFileRow
//! fields and token counts, schema version constants, serde roundtrips, and
//! property-based invariants.

use proptest::prelude::*;
use tokmd_types::{
    ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION,
    CapabilityState, CapabilityStatus, ContextBundleManifest, ContextExcludedPath, ContextFileRow,
    ContextReceipt, FileClassification, HANDOFF_SCHEMA_VERSION, HandoffComplexity, HandoffDerived,
    HandoffExcludedPath, HandoffManifest, InclusionPolicy, PolicyExcludedFile, SmartExcludedFile,
    TokenEstimationMeta, ToolInfo,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tool() -> ToolInfo {
    ToolInfo {
        name: "tokmd".into(),
        version: "0.0.0-test".into(),
    }
}

fn make_row(path: &str, tokens: usize, effective: Option<usize>) -> ContextFileRow {
    ContextFileRow {
        path: path.into(),
        module: "src".into(),
        lang: "Rust".into(),
        tokens,
        code: tokens / 2,
        lines: tokens,
        bytes: tokens * 4,
        value: tokens,
        rank_reason: "code".into(),
        policy: InclusionPolicy::Full,
        effective_tokens: effective,
        policy_reason: None,
        classifications: vec![],
    }
}

fn minimal_context_receipt() -> ContextReceipt {
    ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_000_000,
        tool: tool(),
        mode: "greedy".into(),
        budget_tokens: 128_000,
        used_tokens: 5_000,
        utilization_pct: 3.9,
        strategy: "greedy".into(),
        rank_by: "code".into(),
        file_count: 1,
        files: vec![make_row("src/lib.rs", 500, None)],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    }
}

fn minimal_handoff_manifest() -> HandoffManifest {
    HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 1_000_000,
        tool: tool(),
        mode: "greedy".into(),
        inputs: vec![".".into()],
        output_dir: "out".into(),
        budget_tokens: 128_000,
        used_tokens: 5_000,
        utilization_pct: 3.9,
        strategy: "greedy".into(),
        rank_by: "code".into(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![make_row("src/lib.rs", 500, None)],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 10,
        bundled_files: 1,
        intelligence_preset: "receipt".into(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    }
}

// ===========================================================================
// 1. ContextReceipt structure validation
// ===========================================================================

#[test]
fn context_receipt_serializes_and_deserializes() {
    let receipt = minimal_context_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(back.budget_tokens, 128_000);
    assert_eq!(back.used_tokens, 5_000);
    assert_eq!(back.file_count, 1);
}

#[test]
fn context_receipt_from_minimal_json() {
    let json = r#"{
        "schema_version": 4,
        "generated_at_ms": 0,
        "tool": {"name":"tokmd","version":"1.0.0"},
        "mode": "greedy",
        "budget_tokens": 10000,
        "used_tokens": 5000,
        "utilization_pct": 50.0,
        "strategy": "greedy",
        "rank_by": "code",
        "file_count": 0,
        "files": []
    }"#;
    let receipt: ContextReceipt = serde_json::from_str(json).unwrap();
    assert_eq!(receipt.schema_version, CONTEXT_SCHEMA_VERSION);
    assert!(receipt.excluded_by_policy.is_empty());
    assert!(receipt.token_estimation.is_none());
    assert!(receipt.bundle_audit.is_none());
}

#[test]
fn context_receipt_optional_fields_absent_by_default() {
    let receipt = minimal_context_receipt();
    let json = serde_json::to_string(&receipt).unwrap();

    assert!(
        !json.contains("rank_by_effective"),
        "optional field should be omitted"
    );
    assert!(
        !json.contains("fallback_reason"),
        "optional field should be omitted"
    );
    assert!(
        !json.contains("token_estimation"),
        "optional field should be omitted"
    );
}

#[test]
fn context_receipt_with_excluded_by_policy() {
    let mut receipt = minimal_context_receipt();
    receipt.excluded_by_policy = vec![PolicyExcludedFile {
        path: "vendor/big.js".into(),
        original_tokens: 50_000,
        policy: InclusionPolicy::Skip,
        reason: "vendored file exceeds cap".into(),
        classifications: vec![FileClassification::Vendored],
    }];

    let json = serde_json::to_string(&receipt).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded_by_policy.len(), 1);
    assert_eq!(back.excluded_by_policy[0].original_tokens, 50_000);
}

#[test]
fn context_receipt_with_token_estimation() {
    let mut receipt = minimal_context_receipt();
    receipt.token_estimation = Some(TokenEstimationMeta::from_bytes(4000, 4.0));

    let json = serde_json::to_string(&receipt).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    let est = back.token_estimation.unwrap();
    assert_eq!(est.tokens_est, 1000);
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
}

// ===========================================================================
// 2. HandoffManifest structure validation
// ===========================================================================

#[test]
fn handoff_manifest_serializes_and_deserializes() {
    let manifest = minimal_handoff_manifest();
    let json = serde_json::to_string(&manifest).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, HANDOFF_SCHEMA_VERSION);
    assert_eq!(back.intelligence_preset, "receipt");
    assert_eq!(back.total_files, 10);
    assert_eq!(back.bundled_files, 1);
}

#[test]
fn handoff_manifest_with_capabilities() {
    let mut manifest = minimal_handoff_manifest();
    manifest.capabilities = vec![
        CapabilityStatus {
            name: "git".into(),
            status: CapabilityState::Available,
            reason: None,
        },
        CapabilityStatus {
            name: "content".into(),
            status: CapabilityState::Skipped,
            reason: Some("disabled by flag".into()),
        },
    ];

    let json = serde_json::to_string(&manifest).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.capabilities.len(), 2);
    assert_eq!(back.capabilities[0].status, CapabilityState::Available);
    assert_eq!(back.capabilities[1].status, CapabilityState::Skipped);
}

#[test]
fn handoff_manifest_with_artifacts() {
    let mut manifest = minimal_handoff_manifest();
    manifest.artifacts = vec![ArtifactEntry {
        name: "receipt.json".into(),
        path: "out/receipt.json".into(),
        description: "Main receipt".into(),
        bytes: 1024,
        hash: Some(ArtifactHash {
            algo: "blake3".into(),
            hash: "abc123".into(),
        }),
    }];

    let json = serde_json::to_string(&manifest).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.artifacts.len(), 1);
    assert_eq!(back.artifacts[0].bytes, 1024);
    assert!(back.artifacts[0].hash.is_some());
}

#[test]
fn handoff_manifest_with_excluded_paths_and_patterns() {
    let mut manifest = minimal_handoff_manifest();
    manifest.excluded_paths = vec![HandoffExcludedPath {
        path: "target/".into(),
        reason: "build output".into(),
    }];
    manifest.excluded_patterns = vec!["*.log".into()];
    manifest.smart_excluded_files = vec![SmartExcludedFile {
        path: "Cargo.lock".into(),
        reason: "lockfile".into(),
        tokens: 5000,
    }];

    let json = serde_json::to_string(&manifest).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded_paths.len(), 1);
    assert_eq!(back.excluded_patterns, vec!["*.log"]);
    assert_eq!(back.smart_excluded_files.len(), 1);
}

// ===========================================================================
// 3. ContextFileRow fields and token counts
// ===========================================================================

#[test]
fn context_file_row_all_fields() {
    let row = make_row("src/main.rs", 1000, None);
    assert_eq!(row.path, "src/main.rs");
    assert_eq!(row.tokens, 1000);
    assert_eq!(row.code, 500);
    assert_eq!(row.lines, 1000);
    assert_eq!(row.bytes, 4000);
    assert_eq!(row.value, 1000);
    assert_eq!(row.policy, InclusionPolicy::Full);
    assert!(row.effective_tokens.is_none());
    assert!(row.classifications.is_empty());
}

#[test]
fn context_file_row_with_effective_tokens() {
    let row = make_row("src/big.rs", 20_000, Some(5_000));
    assert_eq!(row.tokens, 20_000);
    assert_eq!(row.effective_tokens, Some(5_000));
}

#[test]
fn context_file_row_with_classifications() {
    let mut row = make_row("vendor/lib.js", 500, None);
    row.classifications = vec![FileClassification::Vendored];
    assert_eq!(row.classifications.len(), 1);
}

#[test]
fn context_file_row_with_head_tail_policy() {
    let mut row = make_row("src/big.rs", 20_000, Some(5_000));
    row.policy = InclusionPolicy::HeadTail;
    row.policy_reason = Some("file exceeds cap".into());

    let json = serde_json::to_string(&row).unwrap();
    let back: ContextFileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.policy, InclusionPolicy::HeadTail);
    assert_eq!(back.effective_tokens, Some(5_000));
    assert!(back.policy_reason.is_some());
}

#[test]
fn context_file_row_full_policy_omitted_in_json() {
    let row = make_row("src/lib.rs", 500, None);
    let json = serde_json::to_string(&row).unwrap();
    // Full policy is default, should be skipped in serialization
    assert!(
        !json.contains("\"policy\""),
        "Full policy should be omitted: {json}"
    );
}

#[test]
fn context_file_row_skip_policy_present_in_json() {
    let mut row = make_row("src/lib.rs", 500, None);
    row.policy = InclusionPolicy::Skip;
    let json = serde_json::to_string(&row).unwrap();
    assert!(
        json.contains("\"skip\""),
        "Skip policy should be present: {json}"
    );
}

// ===========================================================================
// 4. Schema version constants are correct types
// ===========================================================================

#[test]
fn context_schema_version_is_u32() {
    let _: u32 = CONTEXT_SCHEMA_VERSION;
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
}

#[test]
fn handoff_schema_version_is_u32() {
    let _: u32 = HANDOFF_SCHEMA_VERSION;
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
}

#[test]
fn context_bundle_schema_version_is_u32() {
    let _: u32 = CONTEXT_BUNDLE_SCHEMA_VERSION;
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
}

// ===========================================================================
// 5. Schema version round-trips through JSON
// ===========================================================================

#[test]
fn context_schema_version_roundtrip() {
    let receipt = minimal_context_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], CONTEXT_SCHEMA_VERSION);
}

#[test]
fn handoff_schema_version_roundtrip() {
    let manifest = minimal_handoff_manifest();
    let json = serde_json::to_string(&manifest).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], HANDOFF_SCHEMA_VERSION);
}

#[test]
fn context_bundle_schema_version_roundtrip() {
    let bundle = ContextBundleManifest {
        schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "greedy".into(),
        budget_tokens: 10_000,
        used_tokens: 5_000,
        utilization_pct: 50.0,
        strategy: "greedy".into(),
        rank_by: "code".into(),
        file_count: 0,
        bundle_bytes: 0,
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    let json = serde_json::to_string(&bundle).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], CONTEXT_BUNDLE_SCHEMA_VERSION);
}

// ===========================================================================
// 6. InclusionPolicy and FileClassification serde
// ===========================================================================

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

// ===========================================================================
// 7. TokenEstimationMeta invariants
// ===========================================================================

#[test]
fn token_estimation_invariant() {
    let est = TokenEstimationMeta::from_bytes(4000, 4.0);
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
    assert_eq!(est.source_bytes, 4000);
}

#[test]
fn token_estimation_zero_bytes() {
    let est = TokenEstimationMeta::from_bytes(0, 4.0);
    assert_eq!(est.tokens_min, 0);
    assert_eq!(est.tokens_est, 0);
    assert_eq!(est.tokens_max, 0);
}

// ===========================================================================
// 8. Related types: ContextExcludedPath, HandoffDerived, HandoffComplexity
// ===========================================================================

#[test]
fn context_excluded_path_roundtrip() {
    let excl = ContextExcludedPath {
        path: "target/".into(),
        reason: "build output".into(),
    };
    let json = serde_json::to_string(&excl).unwrap();
    let back: ContextExcludedPath = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "target/");
    assert_eq!(back.reason, "build output");
}

#[test]
fn handoff_derived_roundtrip() {
    let derived = HandoffDerived {
        total_files: 100,
        total_code: 50_000,
        total_lines: 80_000,
        total_tokens: 200_000,
        lang_count: 5,
        dominant_lang: "Rust".into(),
        dominant_pct: 65.5,
    };
    let json = serde_json::to_string(&derived).unwrap();
    let back: HandoffDerived = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_files, 100);
    assert_eq!(back.dominant_lang, "Rust");
}

#[test]
fn handoff_complexity_roundtrip() {
    let complexity = HandoffComplexity {
        total_functions: 50,
        avg_function_length: 25.0,
        max_function_length: 200,
        avg_cyclomatic: 3.5,
        max_cyclomatic: 15,
        high_risk_files: 2,
    };
    let json = serde_json::to_string(&complexity).unwrap();
    let back: HandoffComplexity = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_functions, 50);
    assert_eq!(back.high_risk_files, 2);
}

// ===========================================================================
// 9. Property tests
// ===========================================================================

proptest! {
    #[test]
    fn prop_effective_tokens_le_tokens(
        tokens in 1usize..200_000,
        effective_pct in 0.0f64..1.0,
    ) {
        let effective = (tokens as f64 * effective_pct) as usize;
        let row = make_row("src/file.rs", tokens, Some(effective));
        let eff = row.effective_tokens.unwrap_or(row.tokens);
        prop_assert!(eff <= row.tokens, "effective {} > tokens {}", eff, row.tokens);
    }

    #[test]
    fn prop_context_file_row_serde_roundtrip(
        tokens in 1usize..100_000,
        code in 0usize..50_000,
        lines in 1usize..10_000,
    ) {
        let row = ContextFileRow {
            path: "src/test.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            tokens,
            code,
            lines,
            bytes: tokens * 4,
            value: tokens,
            rank_reason: "code".into(),
            policy: InclusionPolicy::Full,
            effective_tokens: None,
            policy_reason: None,
            classifications: vec![],
        };

        let json = serde_json::to_string(&row).unwrap();
        let back: ContextFileRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.tokens, row.tokens);
        prop_assert_eq!(back.code, row.code);
        prop_assert_eq!(back.lines, row.lines);
    }

    #[test]
    fn prop_token_estimation_invariant(
        bytes in 0usize..1_000_000,
    ) {
        let est = TokenEstimationMeta::from_bytes(bytes, 4.0);
        prop_assert!(est.tokens_min <= est.tokens_est);
        prop_assert!(est.tokens_est <= est.tokens_max);
        prop_assert_eq!(est.source_bytes, bytes);
    }

    #[test]
    fn prop_inclusion_policy_roundtrip(
        idx in 0usize..4,
    ) {
        let policies = [
            InclusionPolicy::Full,
            InclusionPolicy::HeadTail,
            InclusionPolicy::Summary,
            InclusionPolicy::Skip,
        ];
        let policy = policies[idx];
        let json = serde_json::to_string(&policy).unwrap();
        let back: InclusionPolicy = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, policy);
    }

    #[test]
    fn prop_file_classification_roundtrip(
        idx in 0usize..7,
    ) {
        let classes = [
            FileClassification::Generated,
            FileClassification::Fixture,
            FileClassification::Vendored,
            FileClassification::Lockfile,
            FileClassification::Minified,
            FileClassification::DataBlob,
            FileClassification::Sourcemap,
        ];
        let class = classes[idx];
        let json = serde_json::to_string(&class).unwrap();
        let back: FileClassification = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, class);
    }
}
