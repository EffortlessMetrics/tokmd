//! Deep tests for context receipt and handoff manifest types.
//!
//! Covers: JSON-first deserialization from raw strings, serde skip-if behaviour,
//! schema forward-compat (unknown fields), InclusionPolicy/FileClassification
//! ordering, token estimation edge cases, audit divisor overrides,
//! context bundle with policy exclusions, and intelligence preset strings.

use serde_json::Value;
use tokmd_types::{
    ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION,
    CapabilityState, CapabilityStatus, ContextBundleManifest, ContextExcludedPath, ContextFileRow,
    ContextLogRecord, ContextReceipt, FileClassification, HANDOFF_SCHEMA_VERSION,
    HandoffComplexity, HandoffDerived, HandoffExcludedPath, HandoffHotspot, HandoffIntelligence,
    HandoffManifest, InclusionPolicy, PolicyExcludedFile, SmartExcludedFile, TokenAudit,
    TokenEstimationMeta, ToolInfo,
};

// ── helpers ────────────────────────────────────────────────────────────────

fn tool() -> ToolInfo {
    ToolInfo {
        name: "tokmd".into(),
        version: "0.0.0-test".into(),
    }
}

fn row(path: &str, tokens: usize) -> ContextFileRow {
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
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    }
}

// ===========================================================================
// 1. JSON-first deserialization from raw strings
// ===========================================================================

#[test]
fn context_receipt_deserializes_from_minimal_json() {
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
    assert_eq!(receipt.budget_tokens, 10_000);
    assert!(receipt.excluded_by_policy.is_empty());
    assert!(receipt.token_estimation.is_none());
}

#[test]
fn handoff_manifest_deserializes_from_minimal_json() {
    let json = r#"{
        "schema_version": 5,
        "generated_at_ms": 0,
        "tool": {"name":"tokmd","version":"1.0.0"},
        "mode": "greedy",
        "inputs": ["."],
        "output_dir": "out",
        "budget_tokens": 100000,
        "used_tokens": 50000,
        "utilization_pct": 50.0,
        "strategy": "greedy",
        "rank_by": "code",
        "capabilities": [],
        "artifacts": [],
        "included_files": [],
        "excluded_paths": [],
        "excluded_patterns": [],
        "smart_excluded_files": [],
        "total_files": 10,
        "bundled_files": 8,
        "intelligence_preset": "receipt"
    }"#;
    let manifest: HandoffManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.schema_version, HANDOFF_SCHEMA_VERSION);
    assert_eq!(manifest.intelligence_preset, "receipt");
    assert!(manifest.rank_by_effective.is_none());
}

// ===========================================================================
// 2. Serde skip-if behaviour for optional/default fields
// ===========================================================================

#[test]
fn context_file_row_full_policy_omitted_in_json() {
    let r = row("src/main.rs", 500);
    let json = serde_json::to_string(&r).unwrap();
    // "policy" should be skipped when Full (is_default_policy)
    assert!(
        !json.contains("\"policy\""),
        "Full policy should be omitted: {json}"
    );
}

#[test]
fn context_file_row_headtail_policy_present_in_json() {
    let mut r = row("src/big.rs", 20_000);
    r.policy = InclusionPolicy::HeadTail;
    r.effective_tokens = Some(4_000);
    r.policy_reason = Some("exceeds cap".into());
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains("\"policy\":\"head_tail\""));
    assert!(json.contains("\"effective_tokens\":4000"));
    assert!(json.contains("\"policy_reason\""));
}

#[test]
fn context_file_row_empty_classifications_omitted() {
    let r = row("src/main.rs", 500);
    let json = serde_json::to_string(&r).unwrap();
    assert!(
        !json.contains("\"classifications\""),
        "empty vec should be omitted"
    );
}

#[test]
fn context_file_row_nonempty_classifications_present() {
    let mut r = row("vendor/dep.js", 500);
    r.classifications = vec![FileClassification::Vendored];
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains("\"classifications\""));
    assert!(json.contains("\"vendored\""));
}

// ===========================================================================
// 3. Forward-compat: unknown fields ignored during deserialization
// ===========================================================================

#[test]
fn context_receipt_ignores_unknown_fields() {
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
        "files": [],
        "future_field_v99": "should be ignored"
    }"#;
    // serde by default denies unknown fields only if #[serde(deny_unknown_fields)] is set.
    // tokmd-types uses derive(Deserialize) without deny_unknown_fields.
    let result: Result<ContextReceipt, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "unknown fields should be silently ignored");
}

// ===========================================================================
// 4. InclusionPolicy and FileClassification ordering
// ===========================================================================

#[test]
fn inclusion_policy_ordering_full_lt_skip() {
    assert!(InclusionPolicy::Full < InclusionPolicy::HeadTail);
    assert!(InclusionPolicy::HeadTail < InclusionPolicy::Summary);
    assert!(InclusionPolicy::Summary < InclusionPolicy::Skip);
}

#[test]
fn file_classification_serde_roundtrip_all_variants() {
    let all = [
        FileClassification::Generated,
        FileClassification::Fixture,
        FileClassification::Vendored,
        FileClassification::Lockfile,
        FileClassification::Minified,
        FileClassification::DataBlob,
        FileClassification::Sourcemap,
    ];
    for c in all {
        let json = serde_json::to_string(&c).unwrap();
        let back: FileClassification = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }
}

#[test]
fn inclusion_policy_serde_roundtrip_all_variants() {
    let all = [
        InclusionPolicy::Full,
        InclusionPolicy::HeadTail,
        InclusionPolicy::Summary,
        InclusionPolicy::Skip,
    ];
    for p in all {
        let json = serde_json::to_string(&p).unwrap();
        let back: InclusionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
    }
}

// ===========================================================================
// 5. TokenEstimationMeta edge cases
// ===========================================================================

#[test]
fn token_estimation_invariant_min_le_est_le_max() {
    for bytes in [0, 1, 100, 12_345, 1_000_000] {
        let est = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        assert!(
            est.tokens_min <= est.tokens_est,
            "min={} > est={} for bytes={bytes}",
            est.tokens_min,
            est.tokens_est
        );
        assert!(
            est.tokens_est <= est.tokens_max,
            "est={} > max={} for bytes={bytes}",
            est.tokens_est,
            est.tokens_max
        );
    }
}

#[test]
fn token_estimation_custom_bounds_respected() {
    let est = TokenEstimationMeta::from_bytes_with_bounds(1200, 4.0, 2.0, 6.0);
    // min = ceil(1200/6) = 200, est = ceil(1200/4) = 300, max = ceil(1200/2) = 600
    assert_eq!(est.tokens_min, 200);
    assert_eq!(est.tokens_est, 300);
    assert_eq!(est.tokens_max, 600);
    assert_eq!(est.source_bytes, 1200);
}

#[test]
fn token_estimation_serde_roundtrip() {
    let est = TokenEstimationMeta::from_bytes(8000, 4.0);
    let json = serde_json::to_string(&est).unwrap();
    let back: TokenEstimationMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tokens_est, est.tokens_est);
    assert_eq!(back.source_bytes, 8000);
}

// ===========================================================================
// 6. TokenAudit edge cases
// ===========================================================================

#[test]
fn token_audit_zero_output_bytes() {
    let audit = TokenAudit::from_output(0, 0);
    assert_eq!(audit.output_bytes, 0);
    assert_eq!(audit.overhead_bytes, 0);
    assert!((audit.overhead_pct - 0.0).abs() < 1e-10);
}

#[test]
fn token_audit_content_exceeds_output_saturates() {
    let audit = TokenAudit::from_output(100, 200);
    // overhead = saturating_sub → 0
    assert_eq!(audit.overhead_bytes, 0);
}

#[test]
fn token_audit_custom_divisors() {
    let audit = TokenAudit::from_output_with_divisors(1000, 800, 4.0, 3.0, 5.0);
    assert_eq!(audit.output_bytes, 1000);
    assert_eq!(audit.overhead_bytes, 200);
    // tokens_est = ceil(1000/4) = 250
    assert_eq!(audit.tokens_est, 250);
    // tokens_min = ceil(1000/5) = 200
    assert_eq!(audit.tokens_min, 200);
    // tokens_max = ceil(1000/3) = 334
    assert_eq!(audit.tokens_max, 334);
}

#[test]
fn token_audit_serde_roundtrip() {
    let audit = TokenAudit::from_output(5000, 4500);
    let json = serde_json::to_string(&audit).unwrap();
    let back: TokenAudit = serde_json::from_str(&json).unwrap();
    assert_eq!(back.output_bytes, 5000);
    assert_eq!(back.overhead_bytes, 500);
}

// ===========================================================================
// 7. ContextBundleManifest with policy exclusions
// ===========================================================================

#[test]
fn context_bundle_manifest_with_excluded_by_policy_roundtrips() {
    let manifest = ContextBundleManifest {
        schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "greedy".into(),
        budget_tokens: 50_000,
        used_tokens: 30_000,
        utilization_pct: 60.0,
        strategy: "greedy".into(),
        rank_by: "code".into(),
        file_count: 1,
        bundle_bytes: 120_000,
        artifacts: vec![],
        included_files: vec![row("src/main.rs", 30_000)],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![PolicyExcludedFile {
            path: "api/types.pb.go".into(),
            original_tokens: 40_000,
            policy: InclusionPolicy::Skip,
            reason: "generated file exceeds cap".into(),
            classifications: vec![FileClassification::Generated],
        }],
        token_estimation: None,
        bundle_audit: None,
    };

    let json = serde_json::to_string(&manifest).unwrap();
    let back: ContextBundleManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded_by_policy.len(), 1);
    assert_eq!(back.excluded_by_policy[0].path, "api/types.pb.go");
    assert_eq!(back.excluded_by_policy[0].policy, InclusionPolicy::Skip);
}

// ===========================================================================
// 8. HandoffManifest intelligence presets
// ===========================================================================

#[test]
fn handoff_manifest_intelligence_preset_values() {
    let presets = [
        "receipt",
        "health",
        "risk",
        "supply",
        "architecture",
        "topics",
        "security",
        "identity",
        "git",
        "deep",
        "fun",
    ];
    for preset in presets {
        let manifest = HandoffManifest {
            schema_version: HANDOFF_SCHEMA_VERSION,
            generated_at_ms: 0,
            tool: tool(),
            mode: "greedy".into(),
            inputs: vec![".".into()],
            output_dir: "out".into(),
            budget_tokens: 100_000,
            used_tokens: 50_000,
            utilization_pct: 50.0,
            strategy: "greedy".into(),
            rank_by: "code".into(),
            capabilities: vec![],
            artifacts: vec![],
            included_files: vec![],
            excluded_paths: vec![],
            excluded_patterns: vec![],
            smart_excluded_files: vec![],
            total_files: 0,
            bundled_files: 0,
            intelligence_preset: preset.into(),
            rank_by_effective: None,
            fallback_reason: None,
            excluded_by_policy: vec![],
            token_estimation: None,
            code_audit: None,
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let back: HandoffManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.intelligence_preset, preset);
    }
}

// ===========================================================================
// 9. CapabilityState serde
// ===========================================================================

#[test]
fn capability_state_serde_all_variants() {
    let all = [
        CapabilityState::Available,
        CapabilityState::Skipped,
        CapabilityState::Unavailable,
    ];
    for state in all {
        let json = serde_json::to_string(&state).unwrap();
        let back: CapabilityState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state);
    }
}

#[test]
fn capability_status_with_reason_roundtrips() {
    let status = CapabilityStatus {
        name: "git".into(),
        status: CapabilityState::Unavailable,
        reason: Some("not a git repository".into()),
    };
    let json = serde_json::to_string(&status).unwrap();
    let back: CapabilityStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back.reason.as_deref(), Some("not a git repository"));
}

// ===========================================================================
// 10. ArtifactEntry with and without hash
// ===========================================================================

#[test]
fn artifact_entry_without_hash_omits_field() {
    let entry = ArtifactEntry {
        name: "receipt.json".into(),
        path: "out/receipt.json".into(),
        description: "JSON receipt".into(),
        bytes: 1_200,
        hash: None,
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(!json.contains("\"hash\""), "None hash should be omitted");
}

#[test]
fn artifact_entry_with_hash_roundtrips() {
    let entry = ArtifactEntry {
        name: "bundle.txt".into(),
        path: "out/bundle.txt".into(),
        description: "Code bundle".into(),
        bytes: 500_000,
        hash: Some(ArtifactHash {
            algo: "blake3".into(),
            hash: "deadbeef".into(),
        }),
    };
    let json = serde_json::to_string(&entry).unwrap();
    let back: ArtifactEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(back.hash.unwrap().algo, "blake3");
}

// ===========================================================================
// 11. HandoffIntelligence serde roundtrip
// ===========================================================================

#[test]
fn handoff_intelligence_full_serde_roundtrip() {
    let intel = HandoffIntelligence {
        tree: Some("├── src/\n│   └── main.rs".into()),
        tree_depth: Some(3),
        hotspots: Some(vec![HandoffHotspot {
            path: "src/main.rs".into(),
            commits: 50,
            lines: 200,
            score: 10_000,
        }]),
        complexity: Some(HandoffComplexity {
            total_functions: 80,
            avg_function_length: 12.5,
            max_function_length: 60,
            avg_cyclomatic: 2.8,
            max_cyclomatic: 18,
            high_risk_files: 3,
        }),
        derived: Some(HandoffDerived {
            total_files: 30,
            total_code: 8_000,
            total_lines: 12_000,
            total_tokens: 20_000,
            lang_count: 2,
            dominant_lang: "Rust".into(),
            dominant_pct: 90.0,
        }),
        warnings: vec!["high complexity".into()],
    };
    let json = serde_json::to_string(&intel).unwrap();
    let back: HandoffIntelligence = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tree_depth, Some(3));
    assert_eq!(back.hotspots.unwrap().len(), 1);
    assert_eq!(back.complexity.unwrap().high_risk_files, 3);
    assert_eq!(back.derived.unwrap().dominant_lang, "Rust");
    assert_eq!(back.warnings.len(), 1);
}

// ===========================================================================
// 12. PolicyExcludedFile serde
// ===========================================================================

#[test]
fn policy_excluded_file_with_multiple_classifications_roundtrips() {
    let excluded = PolicyExcludedFile {
        path: "vendor/lib/react.min.js".into(),
        original_tokens: 80_000,
        policy: InclusionPolicy::Skip,
        reason: "vendored+minified+data_blob file exceeds cap".into(),
        classifications: vec![
            FileClassification::Vendored,
            FileClassification::Minified,
            FileClassification::DataBlob,
        ],
    };
    let json = serde_json::to_string(&excluded).unwrap();
    let back: PolicyExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.classifications.len(), 3);
    assert_eq!(back.original_tokens, 80_000);
}

// ===========================================================================
// 13. SmartExcludedFile and HandoffExcludedPath serde
// ===========================================================================

#[test]
fn smart_excluded_file_roundtrips() {
    let se = SmartExcludedFile {
        path: "Cargo.lock".into(),
        reason: "lockfile".into(),
        tokens: 25_000,
    };
    let json = serde_json::to_string(&se).unwrap();
    let back: SmartExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "Cargo.lock");
    assert_eq!(back.tokens, 25_000);
}

#[test]
fn handoff_excluded_path_roundtrips() {
    let ep = HandoffExcludedPath {
        path: "target/".into(),
        reason: "build artifacts".into(),
    };
    let json = serde_json::to_string(&ep).unwrap();
    let back: HandoffExcludedPath = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "target/");
}

#[test]
fn context_excluded_path_roundtrips() {
    let ep = ContextExcludedPath {
        path: ".git/".into(),
        reason: "VCS directory".into(),
    };
    let json = serde_json::to_string(&ep).unwrap();
    let back: ContextExcludedPath = serde_json::from_str(&json).unwrap();
    assert_eq!(back.reason, "VCS directory");
}

// ===========================================================================
// 14. ContextReceipt with fallback and estimation
// ===========================================================================

#[test]
fn context_receipt_with_fallback_reason_roundtrips() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "greedy".into(),
        budget_tokens: 128_000,
        used_tokens: 64_000,
        utilization_pct: 50.0,
        strategy: "greedy".into(),
        rank_by: "churn".into(),
        file_count: 0,
        files: vec![],
        rank_by_effective: Some("tokens".into()),
        fallback_reason: Some("no git history available".into()),
        excluded_by_policy: vec![],
        token_estimation: Some(TokenEstimationMeta::from_bytes(256_000, 4.0)),
        bundle_audit: Some(TokenAudit::from_output(260_000, 250_000)),
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["rank_by_effective"], "tokens");
    assert_eq!(v["fallback_reason"], "no git history available");
    assert!(v["token_estimation"]["source_bytes"].as_u64().unwrap() == 256_000);
    assert!(v["bundle_audit"]["output_bytes"].as_u64().unwrap() == 260_000);
}

// ===========================================================================
// 15. ContextLogRecord serde
// ===========================================================================

#[test]
fn context_log_record_minimal_roundtrip() {
    let record = ContextLogRecord {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool(),
        budget_tokens: 50_000,
        used_tokens: 45_000,
        utilization_pct: 90.0,
        strategy: "greedy".into(),
        rank_by: "code".into(),
        file_count: 10,
        total_bytes: 200_000,
        output_destination: "file:///tmp/ctx.txt".into(),
    };
    let json = serde_json::to_string(&record).unwrap();
    let back: ContextLogRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(back.output_destination, "file:///tmp/ctx.txt");
    assert_eq!(back.utilization_pct, 90.0);
}

// ===========================================================================
// 16. HandoffManifest with estimation but no audit
// ===========================================================================

#[test]
fn handoff_manifest_estimation_without_audit_serializes_cleanly() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tool(),
        mode: "greedy".into(),
        inputs: vec![".".into()],
        output_dir: "out".into(),
        budget_tokens: 200_000,
        used_tokens: 100_000,
        utilization_pct: 50.0,
        strategy: "greedy".into(),
        rank_by: "code".into(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 0,
        bundled_files: 0,
        intelligence_preset: "health".into(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: Some(TokenEstimationMeta::from_bytes(400_000, 4.0)),
        code_audit: None,
    };
    let json = serde_json::to_string(&manifest).unwrap();
    assert!(json.contains("\"token_estimation\""));
    assert!(
        !json.contains("\"code_audit\""),
        "None audit should be omitted"
    );
}
