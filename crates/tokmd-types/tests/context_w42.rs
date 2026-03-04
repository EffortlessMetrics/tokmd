//! Wave 42 deep tests for context-related types in tokmd-types.
//!
//! Covers ContextFileRow construction, token counts, schema versions,
//! context bundle types, handoff manifest, and serde roundtrips.

use tokmd_types::*;

// ── Schema version constants ──────────────────────────────────────────

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

// ── ContextFileRow construction and field semantics ───────────────────

fn make_context_file_row() -> ContextFileRow {
    ContextFileRow {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        tokens: 500,
        code: 200,
        lines: 250,
        bytes: 2000,
        value: 500,
        rank_reason: String::new(),
        policy: InclusionPolicy::Full,
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    }
}

#[test]
fn context_file_row_full_policy_no_effective_tokens() {
    let row = make_context_file_row();
    assert_eq!(row.policy, InclusionPolicy::Full);
    assert!(row.effective_tokens.is_none());
}

#[test]
fn context_file_row_head_tail_has_effective_tokens() {
    let mut row = make_context_file_row();
    row.policy = InclusionPolicy::HeadTail;
    row.effective_tokens = Some(300);
    row.policy_reason = Some("file exceeds cap".to_string());
    assert_eq!(row.effective_tokens, Some(300));
    assert!(row.effective_tokens.unwrap() < row.tokens);
}

#[test]
fn context_file_row_skip_policy() {
    let mut row = make_context_file_row();
    row.policy = InclusionPolicy::Skip;
    row.effective_tokens = Some(0);
    row.classifications = vec![FileClassification::Generated];
    assert_eq!(row.effective_tokens, Some(0));
}

// ── Serde roundtrip for ContextFileRow ────────────────────────────────

#[test]
fn context_file_row_serde_roundtrip_full() {
    let row = make_context_file_row();
    let json = serde_json::to_string(&row).unwrap();
    let back: ContextFileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, row.path);
    assert_eq!(back.tokens, row.tokens);
    assert_eq!(back.policy, InclusionPolicy::Full);
    assert!(back.effective_tokens.is_none());
}

#[test]
fn context_file_row_serde_skips_empty_rank_reason() {
    let row = make_context_file_row();
    let json = serde_json::to_string(&row).unwrap();
    assert!(
        !json.contains("rank_reason"),
        "empty rank_reason should be skipped: {json}"
    );
}

#[test]
fn context_file_row_serde_skips_default_policy() {
    let row = make_context_file_row();
    let json = serde_json::to_string(&row).unwrap();
    assert!(
        !json.contains("\"policy\""),
        "default Full policy should be skipped: {json}"
    );
}

#[test]
fn context_file_row_serde_includes_non_default_policy() {
    let mut row = make_context_file_row();
    row.policy = InclusionPolicy::HeadTail;
    row.effective_tokens = Some(250);
    let json = serde_json::to_string(&row).unwrap();
    assert!(
        json.contains("\"head_tail\""),
        "non-default policy should be present: {json}"
    );
    assert!(
        json.contains("\"effective_tokens\""),
        "effective_tokens should be present: {json}"
    );
}

#[test]
fn context_file_row_serde_with_classifications() {
    let mut row = make_context_file_row();
    row.classifications = vec![FileClassification::Lockfile, FileClassification::DataBlob];
    let json = serde_json::to_string(&row).unwrap();
    let back: ContextFileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.classifications.len(), 2);
    assert!(back.classifications.contains(&FileClassification::Lockfile));
    assert!(back.classifications.contains(&FileClassification::DataBlob));
}

// ── FileClassification serde ──────────────────────────────────────────

#[test]
fn file_classification_serde_roundtrip_all_variants() {
    let variants = [
        FileClassification::Generated,
        FileClassification::Fixture,
        FileClassification::Vendored,
        FileClassification::Lockfile,
        FileClassification::Minified,
        FileClassification::DataBlob,
        FileClassification::Sourcemap,
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let back: FileClassification = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant, "roundtrip failed for {:?}", variant);
    }
}

#[test]
fn file_classification_uses_snake_case() {
    assert_eq!(
        serde_json::to_string(&FileClassification::DataBlob).unwrap(),
        "\"data_blob\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::Generated).unwrap(),
        "\"generated\""
    );
}

// ── InclusionPolicy serde ─────────────────────────────────────────────

#[test]
fn inclusion_policy_serde_roundtrip_all_variants() {
    let variants = [
        InclusionPolicy::Full,
        InclusionPolicy::HeadTail,
        InclusionPolicy::Summary,
        InclusionPolicy::Skip,
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let back: InclusionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant, "roundtrip failed for {:?}", variant);
    }
}

#[test]
fn inclusion_policy_default_is_full() {
    assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
}

// ── TokenEstimationMeta ───────────────────────────────────────────────

#[test]
fn token_estimation_from_bytes_invariant() {
    let est = TokenEstimationMeta::from_bytes(4000, 4.0);
    assert!(
        est.tokens_min <= est.tokens_est,
        "tokens_min ({}) > tokens_est ({})",
        est.tokens_min,
        est.tokens_est
    );
    assert!(
        est.tokens_est <= est.tokens_max,
        "tokens_est ({}) > tokens_max ({})",
        est.tokens_est,
        est.tokens_max
    );
    assert_eq!(est.source_bytes, 4000);
    assert_eq!(est.tokens_est, 1000);
}

#[test]
fn token_estimation_serde_roundtrip() {
    let est = TokenEstimationMeta::from_bytes(8000, 4.0);
    let json = serde_json::to_string(&est).unwrap();
    let back: TokenEstimationMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.source_bytes, 8000);
    assert_eq!(back.tokens_est, est.tokens_est);
    assert_eq!(back.tokens_min, est.tokens_min);
    assert_eq!(back.tokens_max, est.tokens_max);
}

#[test]
fn token_estimation_zero_bytes() {
    let est = TokenEstimationMeta::from_bytes(0, 4.0);
    assert_eq!(est.tokens_est, 0);
    assert_eq!(est.tokens_min, 0);
    assert_eq!(est.tokens_max, 0);
}

// ── TokenAudit ────────────────────────────────────────────────────────

#[test]
fn token_audit_overhead_calculation() {
    let audit = TokenAudit::from_output(5000, 4500);
    assert_eq!(audit.output_bytes, 5000);
    assert_eq!(audit.overhead_bytes, 500);
    let expected_pct = 500.0 / 5000.0;
    assert!((audit.overhead_pct - expected_pct).abs() < 1e-10);
}

#[test]
fn token_audit_zero_output_bytes() {
    let audit = TokenAudit::from_output(0, 0);
    assert_eq!(audit.overhead_bytes, 0);
    assert_eq!(audit.overhead_pct, 0.0);
}

#[test]
fn token_audit_serde_roundtrip() {
    let audit = TokenAudit::from_output(10_000, 9_000);
    let json = serde_json::to_string(&audit).unwrap();
    let back: TokenAudit = serde_json::from_str(&json).unwrap();
    assert_eq!(back.output_bytes, 10_000);
    assert_eq!(back.overhead_bytes, 1_000);
}

// ── PolicyExcludedFile serde ──────────────────────────────────────────

#[test]
fn policy_excluded_file_serde_roundtrip() {
    let pef = PolicyExcludedFile {
        path: "vendor/big.js".to_string(),
        original_tokens: 50_000,
        policy: InclusionPolicy::Skip,
        reason: "vendored file exceeds cap".to_string(),
        classifications: vec![FileClassification::Vendored],
    };
    let json = serde_json::to_string(&pef).unwrap();
    let back: PolicyExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "vendor/big.js");
    assert_eq!(back.original_tokens, 50_000);
    assert_eq!(back.policy, InclusionPolicy::Skip);
    assert_eq!(back.classifications, vec![FileClassification::Vendored]);
}

// ── SmartExcludedFile serde ───────────────────────────────────────────

#[test]
fn smart_excluded_file_serde_roundtrip() {
    let sef = SmartExcludedFile {
        path: "Cargo.lock".to_string(),
        reason: "lockfile".to_string(),
        tokens: 10_000,
    };
    let json = serde_json::to_string(&sef).unwrap();
    let back: SmartExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "Cargo.lock");
    assert_eq!(back.reason, "lockfile");
    assert_eq!(back.tokens, 10_000);
}

// ── ContextExcludedPath serde ─────────────────────────────────────────

#[test]
fn context_excluded_path_serde_roundtrip() {
    let cep = ContextExcludedPath {
        path: "target/debug".to_string(),
        reason: "build output".to_string(),
    };
    let json = serde_json::to_string(&cep).unwrap();
    let back: ContextExcludedPath = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "target/debug");
    assert_eq!(back.reason, "build output");
}
