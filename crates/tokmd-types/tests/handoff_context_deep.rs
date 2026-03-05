//! Deep tests for handoff and context functionality.
//!
//! Covers HandoffManifest construction, preset variations, meta-budget
//! partitioning, token counting, file selection, tree rendering,
//! schema versioning, serialization roundtrips, deterministic output,
//! ContextReceipt construction, token budget enforcement, file packing
//! algorithms, overflow handling, selection policy, and edge cases.

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
        version: "0.0.0-deep-test".into(),
    }
}

fn ctx_row(path: &str, tokens: usize) -> ContextFileRow {
    ContextFileRow {
        path: path.into(),
        module: path.rsplit('/').nth(1).unwrap_or("root").into(),
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

fn ctx_row_with_policy(
    path: &str,
    tokens: usize,
    policy: InclusionPolicy,
    effective: usize,
) -> ContextFileRow {
    ContextFileRow {
        policy,
        effective_tokens: Some(effective),
        policy_reason: Some(format!("{:?} policy applied", policy)),
        ..ctx_row(path, tokens)
    }
}

fn empty_handoff(preset: &str, budget: usize, used: usize) -> HandoffManifest {
    HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool(),
        mode: "handoff".into(),
        inputs: vec![".".into()],
        output_dir: "out".into(),
        budget_tokens: budget,
        used_tokens: used,
        utilization_pct: if budget > 0 {
            used as f64 / budget as f64 * 100.0
        } else {
            0.0
        },
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
    }
}

fn empty_context(budget: usize, used: usize) -> ContextReceipt {
    ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool(),
        mode: "greedy".into(),
        budget_tokens: budget,
        used_tokens: used,
        utilization_pct: if budget > 0 {
            used as f64 / budget as f64 * 100.0
        } else {
            0.0
        },
        strategy: "greedy".into(),
        rank_by: "code".into(),
        file_count: 0,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    }
}

// =============================================================================
// 1. HandoffManifest — preset variations
// =============================================================================

#[test]
fn handoff_preset_receipt_is_minimal() {
    let m = empty_handoff("receipt", 128_000, 0);
    assert_eq!(m.intelligence_preset, "receipt");
    assert!(m.capabilities.is_empty());
}

#[test]
fn handoff_preset_health_stores_correctly() {
    let m = empty_handoff("health", 128_000, 50_000);
    assert_eq!(m.intelligence_preset, "health");
}

#[test]
fn handoff_preset_risk_stores_correctly() {
    let m = empty_handoff("risk", 128_000, 70_000);
    assert_eq!(m.intelligence_preset, "risk");
}

#[test]
fn handoff_preset_deep_stores_correctly() {
    let m = empty_handoff("deep", 200_000, 180_000);
    assert_eq!(m.intelligence_preset, "deep");
}

#[test]
fn handoff_preset_fun_stores_correctly() {
    let m = empty_handoff("fun", 64_000, 30_000);
    assert_eq!(m.intelligence_preset, "fun");
}

#[test]
fn handoff_all_known_presets_roundtrip() {
    for preset in &[
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
    ] {
        let m = empty_handoff(preset, 100_000, 50_000);
        let json = serde_json::to_string(&m).unwrap();
        let back: HandoffManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.intelligence_preset, *preset);
    }
}

// =============================================================================
// 2. HandoffManifest — meta-budget partitioning
// =============================================================================

#[test]
fn handoff_budget_fully_used_gives_100_pct() {
    let m = empty_handoff("receipt", 100_000, 100_000);
    assert!((m.utilization_pct - 100.0).abs() < f64::EPSILON);
}

#[test]
fn handoff_budget_half_used_gives_50_pct() {
    let m = empty_handoff("receipt", 100_000, 50_000);
    assert!((m.utilization_pct - 50.0).abs() < f64::EPSILON);
}

#[test]
fn handoff_budget_zero_used_gives_0_pct() {
    let m = empty_handoff("receipt", 100_000, 0);
    assert!((m.utilization_pct - 0.0).abs() < f64::EPSILON);
}

#[test]
fn handoff_budget_partitioning_files_vs_intelligence() {
    // Simulate: budget=200k, code=120k, intelligence=30k, overhead=50k
    let mut m = empty_handoff("health", 200_000, 150_000);
    m.included_files = vec![
        ctx_row("src/lib.rs", 60_000),
        ctx_row("src/main.rs", 60_000),
    ];
    let file_tokens: usize = m.included_files.iter().map(|f| f.tokens).sum();
    assert_eq!(file_tokens, 120_000);
    assert!(m.used_tokens > file_tokens, "used includes overhead");
}

// =============================================================================
// 3. HandoffManifest — token counting
// =============================================================================

#[test]
fn handoff_file_tokens_sum_le_used_tokens() {
    let mut m = empty_handoff("receipt", 100_000, 80_000);
    m.included_files = vec![
        ctx_row("src/a.rs", 20_000),
        ctx_row("src/b.rs", 30_000),
        ctx_row("src/c.rs", 10_000),
    ];
    let file_sum: usize = m.included_files.iter().map(|f| f.tokens).sum();
    assert!(file_sum <= m.used_tokens);
}

#[test]
fn handoff_token_estimation_invariants() {
    let est = TokenEstimationMeta::from_bytes(400_000, TokenEstimationMeta::DEFAULT_BPT_EST);
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
    assert_eq!(est.source_bytes, 400_000);
    assert_eq!(est.tokens_est, 100_000); // 400_000 / 4.0
}

#[test]
fn handoff_token_estimation_with_custom_bounds() {
    let est = TokenEstimationMeta::from_bytes_with_bounds(1000, 4.0, 2.0, 8.0);
    assert_eq!(est.tokens_est, 250); // 1000/4.0
    assert_eq!(est.tokens_max, 500); // 1000/2.0
    assert_eq!(est.tokens_min, 125); // 1000/8.0
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
}

#[test]
fn handoff_token_audit_overhead_computation() {
    let audit = TokenAudit::from_output(10_000, 8_000);
    assert_eq!(audit.output_bytes, 10_000);
    assert_eq!(audit.overhead_bytes, 2_000);
    assert!((audit.overhead_pct - 0.2).abs() < f64::EPSILON);
}

#[test]
fn handoff_token_audit_zero_overhead() {
    let audit = TokenAudit::from_output(5_000, 5_000);
    assert_eq!(audit.overhead_bytes, 0);
    assert!((audit.overhead_pct - 0.0).abs() < f64::EPSILON);
}

#[test]
fn handoff_token_audit_zero_output() {
    let audit = TokenAudit::from_output(0, 0);
    assert_eq!(audit.overhead_bytes, 0);
    assert!((audit.overhead_pct - 0.0).abs() < f64::EPSILON);
}

// =============================================================================
// 4. HandoffManifest — file selection
// =============================================================================

#[test]
fn handoff_bundled_files_le_total_files() {
    let mut m = empty_handoff("receipt", 100_000, 80_000);
    m.total_files = 200;
    m.bundled_files = 150;
    assert!(m.bundled_files <= m.total_files);
}

#[test]
fn handoff_smart_excluded_files_accounted() {
    let mut m = empty_handoff("receipt", 100_000, 50_000);
    m.smart_excluded_files = vec![
        SmartExcludedFile {
            path: "Cargo.lock".into(),
            reason: "lockfile".into(),
            tokens: 30_000,
        },
        SmartExcludedFile {
            path: "dist/app.min.js".into(),
            reason: "minified".into(),
            tokens: 100_000,
        },
    ];
    let excluded_tokens: usize = m.smart_excluded_files.iter().map(|f| f.tokens).sum();
    assert_eq!(excluded_tokens, 130_000);
    assert_eq!(m.smart_excluded_files.len(), 2);
}

#[test]
fn handoff_excluded_paths_roundtrip() {
    let mut m = empty_handoff("receipt", 100_000, 50_000);
    m.excluded_paths = vec![
        HandoffExcludedPath {
            path: "target/".into(),
            reason: "build artifacts".into(),
        },
        HandoffExcludedPath {
            path: "node_modules/".into(),
            reason: "vendored deps".into(),
        },
    ];
    let json = serde_json::to_string(&m).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded_paths.len(), 2);
    assert_eq!(back.excluded_paths[0].path, "target/");
    assert_eq!(back.excluded_paths[1].reason, "vendored deps");
}

// =============================================================================
// 5. HandoffManifest — tree rendering (via HandoffIntelligence)
// =============================================================================

#[test]
fn handoff_intelligence_tree_depth_bounds() {
    let intel = HandoffIntelligence {
        tree: Some("├── src/\n│   ├── a/\n│   │   └── b/\n│   │       └── c.rs".into()),
        tree_depth: Some(4),
        hotspots: None,
        complexity: None,
        derived: None,
        warnings: vec![],
    };
    assert!(intel.tree_depth.unwrap() > 0);
    assert!(intel.tree.as_ref().unwrap().contains("src/"));
}

#[test]
fn handoff_intelligence_hotspots_sorted_by_score() {
    let hotspots = vec![
        HandoffHotspot {
            path: "a.rs".into(),
            commits: 10,
            lines: 100,
            score: 1_000,
        },
        HandoffHotspot {
            path: "b.rs".into(),
            commits: 50,
            lines: 500,
            score: 25_000,
        },
        HandoffHotspot {
            path: "c.rs".into(),
            commits: 30,
            lines: 200,
            score: 6_000,
        },
    ];
    let mut sorted = hotspots.clone();
    sorted.sort_by_key(|x| std::cmp::Reverse(x.score));
    assert_eq!(sorted[0].path, "b.rs");
    assert_eq!(sorted[1].path, "c.rs");
    assert_eq!(sorted[2].path, "a.rs");
}

// =============================================================================
// 6. HandoffManifest — schema version
// =============================================================================

#[test]
fn handoff_schema_version_is_five() {
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
}

#[test]
fn handoff_manifest_carries_correct_schema_version() {
    let m = empty_handoff("receipt", 100_000, 50_000);
    let json: Value = serde_json::to_value(m).unwrap();
    assert_eq!(json["schema_version"], 5);
}

// =============================================================================
// 7. HandoffManifest — serialization roundtrip
// =============================================================================

#[test]
fn handoff_full_manifest_roundtrip_preserves_all_fields() {
    let mut m = empty_handoff("deep", 200_000, 180_000);
    m.inputs = vec![".".into(), "lib/".into()];
    m.capabilities = vec![
        CapabilityStatus {
            name: "git".into(),
            status: CapabilityState::Available,
            reason: None,
        },
        CapabilityStatus {
            name: "content".into(),
            status: CapabilityState::Skipped,
            reason: Some("--no-content".into()),
        },
        CapabilityStatus {
            name: "walk".into(),
            status: CapabilityState::Unavailable,
            reason: Some("feature not compiled".into()),
        },
    ];
    m.artifacts = vec![ArtifactEntry {
        name: "code.txt".into(),
        path: "out/code.txt".into(),
        description: "Source bundle".into(),
        bytes: 720_000,
        hash: Some(ArtifactHash {
            algo: "blake3".into(),
            hash: "deadbeef01234567".into(),
        }),
    }];
    m.included_files = vec![
        ctx_row("src/lib.rs", 90_000),
        ctx_row("src/main.rs", 60_000),
    ];
    m.excluded_paths = vec![HandoffExcludedPath {
        path: "target/".into(),
        reason: "build output".into(),
    }];
    m.excluded_patterns = vec!["*.lock".into()];
    m.smart_excluded_files = vec![SmartExcludedFile {
        path: "Cargo.lock".into(),
        reason: "lockfile".into(),
        tokens: 25_000,
    }];
    m.total_files = 100;
    m.bundled_files = 80;
    m.rank_by_effective = Some("churn".into());
    m.fallback_reason = Some("no git".into());
    m.excluded_by_policy = vec![PolicyExcludedFile {
        path: "gen/proto.rs".into(),
        original_tokens: 50_000,
        policy: InclusionPolicy::Skip,
        reason: "generated code".into(),
        classifications: vec![FileClassification::Generated],
    }];
    m.token_estimation = Some(TokenEstimationMeta::from_bytes(
        720_000,
        TokenEstimationMeta::DEFAULT_BPT_EST,
    ));
    m.code_audit = Some(TokenAudit::from_output(740_000, 720_000));

    let json = serde_json::to_string(&m).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();

    assert_eq!(back.schema_version, m.schema_version);
    assert_eq!(back.inputs, m.inputs);
    assert_eq!(back.capabilities.len(), 3);
    assert_eq!(back.capabilities[0].status, CapabilityState::Available);
    assert_eq!(back.capabilities[1].status, CapabilityState::Skipped);
    assert_eq!(back.capabilities[2].status, CapabilityState::Unavailable);
    assert_eq!(back.artifacts.len(), 1);
    assert!(back.artifacts[0].hash.is_some());
    assert_eq!(back.included_files.len(), 2);
    assert_eq!(back.excluded_paths.len(), 1);
    assert_eq!(back.excluded_patterns, vec!["*.lock"]);
    assert_eq!(back.smart_excluded_files.len(), 1);
    assert_eq!(back.total_files, 100);
    assert_eq!(back.bundled_files, 80);
    assert_eq!(back.rank_by_effective.as_deref(), Some("churn"));
    assert_eq!(back.fallback_reason.as_deref(), Some("no git"));
    assert_eq!(back.excluded_by_policy.len(), 1);
    assert!(back.token_estimation.is_some());
    assert!(back.code_audit.is_some());
}

// =============================================================================
// 8. HandoffManifest — deterministic output
// =============================================================================

#[test]
fn handoff_serialization_deterministic_across_runs() {
    let m = empty_handoff("health", 100_000, 85_000);
    let a = serde_json::to_string_pretty(&m).unwrap();
    let b = serde_json::to_string_pretty(&m).unwrap();
    assert_eq!(a, b, "serialization must be deterministic");
}

#[test]
fn handoff_json_field_order_is_stable() {
    let m = empty_handoff("receipt", 100_000, 50_000);
    let json = serde_json::to_string(&m).unwrap();
    let idx_schema = json.find("\"schema_version\"").unwrap();
    let idx_mode = json.find("\"mode\"").unwrap();
    let idx_budget = json.find("\"budget_tokens\"").unwrap();
    assert!(idx_schema < idx_mode);
    assert!(idx_mode < idx_budget);
}

// =============================================================================
// 9. HandoffManifest — capability states
// =============================================================================

#[test]
fn capability_state_all_variants_serialize() {
    for state in &[
        CapabilityState::Available,
        CapabilityState::Skipped,
        CapabilityState::Unavailable,
    ] {
        let json = serde_json::to_string(state).unwrap();
        let back: CapabilityState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, *state);
    }
}

#[test]
fn capability_state_rename_all_snake_case() {
    let json = serde_json::to_string(&CapabilityState::Available).unwrap();
    assert_eq!(json, "\"available\"");
    let json = serde_json::to_string(&CapabilityState::Unavailable).unwrap();
    assert_eq!(json, "\"unavailable\"");
}

// =============================================================================
// 10. ContextReceipt — construction
// =============================================================================

#[test]
fn context_receipt_minimal_construction() {
    let r = empty_context(128_000, 0);
    assert_eq!(r.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(r.file_count, 0);
    assert!(r.files.is_empty());
}

#[test]
fn context_receipt_with_files_construction() {
    let mut r = empty_context(128_000, 80_000);
    r.files = vec![
        ctx_row("src/lib.rs", 40_000),
        ctx_row("src/main.rs", 30_000),
        ctx_row("src/util.rs", 10_000),
    ];
    r.file_count = 3;
    assert_eq!(r.files.len(), 3);
    assert_eq!(r.file_count, 3);
}

// =============================================================================
// 11. ContextReceipt — token budget enforcement
// =============================================================================

#[test]
fn context_budget_not_exceeded_normal_case() {
    let r = empty_context(128_000, 100_000);
    assert!(r.used_tokens <= r.budget_tokens);
    assert!(r.utilization_pct <= 100.0);
}

#[test]
fn context_budget_overflow_recorded_faithfully() {
    let r = empty_context(10_000, 10_500);
    assert!(r.used_tokens > r.budget_tokens);
    assert!(r.utilization_pct > 100.0);
}

#[test]
fn context_budget_exact_match() {
    let r = empty_context(50_000, 50_000);
    assert!((r.utilization_pct - 100.0).abs() < f64::EPSILON);
}

#[test]
fn context_budget_utilization_precision() {
    let r = empty_context(300_000, 100_000);
    let expected = 100_000.0 / 300_000.0 * 100.0;
    assert!(
        (r.utilization_pct - expected).abs() < 0.01,
        "expected ~{expected}, got {}",
        r.utilization_pct
    );
}

// =============================================================================
// 12. ContextReceipt — file packing simulation
// =============================================================================

#[test]
fn context_greedy_packing_fills_budget() {
    // Simulate greedy: add files until budget would be exceeded.
    let budget = 50_000_usize;
    let candidates = vec![
        ctx_row("src/a.rs", 20_000),
        ctx_row("src/b.rs", 15_000),
        ctx_row("src/c.rs", 10_000),
        ctx_row("src/d.rs", 8_000),
    ];

    let mut packed = Vec::new();
    let mut used = 0_usize;
    for f in &candidates {
        if used + f.tokens <= budget {
            packed.push(f.clone());
            used += f.tokens;
        }
    }

    assert_eq!(packed.len(), 3); // a+b+c = 45k, d would push to 53k
    assert!(used <= budget);
    assert_eq!(used, 45_000);
}

#[test]
fn context_greedy_packing_single_oversized_file() {
    let budget = 10_000_usize;
    let candidates = vec![ctx_row("huge.rs", 50_000)];
    let mut packed = Vec::new();
    let mut used = 0_usize;
    for f in &candidates {
        if used + f.tokens <= budget {
            packed.push(f.clone());
            used += f.tokens;
        }
    }
    assert!(packed.is_empty());
    assert_eq!(used, 0);
}

// =============================================================================
// 13. ContextReceipt — overflow handling
// =============================================================================

#[test]
fn context_overflow_one_file_scenario() {
    // When only one file exists and it exceeds budget, it may still be included.
    let mut r = empty_context(5_000, 8_000);
    r.files = vec![ctx_row("only_file.rs", 8_000)];
    r.file_count = 1;
    assert!(r.used_tokens > r.budget_tokens);
    assert!(r.utilization_pct > 100.0);
}

#[test]
fn context_overflow_is_serializable() {
    let r = empty_context(1_000, 2_000);
    let json = serde_json::to_string(&r).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.used_tokens, 2_000);
    assert!(back.utilization_pct > 100.0);
}

// =============================================================================
// 14. ContextReceipt — selection policy
// =============================================================================

#[test]
fn context_file_row_full_policy_no_effective_tokens() {
    let row = ctx_row("src/lib.rs", 1_000);
    assert_eq!(row.policy, InclusionPolicy::Full);
    assert!(row.effective_tokens.is_none());
}

#[test]
fn context_file_row_head_tail_reduces_tokens() {
    let row = ctx_row_with_policy("src/big.rs", 50_000, InclusionPolicy::HeadTail, 5_000);
    assert_eq!(row.policy, InclusionPolicy::HeadTail);
    assert!(row.effective_tokens.unwrap() < row.tokens);
}

#[test]
fn context_file_row_skip_zeroes_tokens() {
    let row = ctx_row_with_policy("vendor/lib.js", 100_000, InclusionPolicy::Skip, 0);
    assert_eq!(row.policy, InclusionPolicy::Skip);
    assert_eq!(row.effective_tokens, Some(0));
}

#[test]
fn context_file_row_summary_policy_exists() {
    let row = ctx_row_with_policy("gen/huge.rs", 80_000, InclusionPolicy::Summary, 500);
    assert_eq!(row.policy, InclusionPolicy::Summary);
}

#[test]
fn inclusion_policy_all_variants_serde_roundtrip() {
    for variant in &[
        InclusionPolicy::Full,
        InclusionPolicy::HeadTail,
        InclusionPolicy::Summary,
        InclusionPolicy::Skip,
    ] {
        let json = serde_json::to_string(variant).unwrap();
        let back: InclusionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, *variant);
    }
}

#[test]
fn inclusion_policy_snake_case_serialization() {
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::HeadTail).unwrap(),
        "\"head_tail\""
    );
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::Full).unwrap(),
        "\"full\""
    );
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::Skip).unwrap(),
        "\"skip\""
    );
}

// =============================================================================
// 15. ContextReceipt — schema version
// =============================================================================

#[test]
fn context_schema_version_is_four() {
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
}

#[test]
fn context_bundle_schema_version_is_two() {
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
}

// =============================================================================
// 16. ContextReceipt — serialization roundtrip
// =============================================================================

#[test]
fn context_receipt_full_roundtrip() {
    let mut r = empty_context(128_000, 100_000);
    r.files = vec![
        ctx_row("src/lib.rs", 50_000),
        ctx_row_with_policy("src/big.rs", 80_000, InclusionPolicy::HeadTail, 8_000),
    ];
    r.file_count = 2;
    r.rank_by_effective = Some("tokens".into());
    r.fallback_reason = Some("churn unavailable".into());
    r.excluded_by_policy = vec![PolicyExcludedFile {
        path: "gen/proto.rs".into(),
        original_tokens: 50_000,
        policy: InclusionPolicy::Skip,
        reason: "generated".into(),
        classifications: vec![FileClassification::Generated],
    }];
    r.token_estimation = Some(TokenEstimationMeta::from_bytes(
        400_000,
        TokenEstimationMeta::DEFAULT_BPT_EST,
    ));
    r.bundle_audit = Some(TokenAudit::from_output(420_000, 400_000));

    let json = serde_json::to_string(&r).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();

    assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(back.budget_tokens, 128_000);
    assert_eq!(back.used_tokens, 100_000);
    assert_eq!(back.files.len(), 2);
    assert_eq!(back.rank_by_effective.as_deref(), Some("tokens"));
    assert_eq!(back.excluded_by_policy.len(), 1);
    assert!(back.token_estimation.is_some());
    assert!(back.bundle_audit.is_some());
}

// =============================================================================
// 17. ContextBundleManifest — construction and roundtrip
// =============================================================================

#[test]
fn context_bundle_manifest_construction() {
    let manifest = ContextBundleManifest {
        schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool(),
        mode: "bundle".into(),
        budget_tokens: 128_000,
        used_tokens: 100_000,
        utilization_pct: 78.125,
        strategy: "greedy".into(),
        rank_by: "code".into(),
        file_count: 10,
        bundle_bytes: 400_000,
        artifacts: vec![ArtifactEntry {
            name: "bundle.txt".into(),
            path: "out/bundle.txt".into(),
            description: "Concatenated source".into(),
            bytes: 400_000,
            hash: Some(ArtifactHash {
                algo: "blake3".into(),
                hash: "0123456789abcdef".into(),
            }),
        }],
        included_files: vec![ctx_row("src/lib.rs", 50_000)],
        excluded_paths: vec![ContextExcludedPath {
            path: "target/".into(),
            reason: "build output".into(),
        }],
        excluded_patterns: vec!["*.lock".into(), "*.min.js".into()],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    let json = serde_json::to_string(&manifest).unwrap();
    let back: ContextBundleManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, CONTEXT_BUNDLE_SCHEMA_VERSION);
    assert_eq!(back.bundle_bytes, 400_000);
    assert_eq!(back.artifacts.len(), 1);
    assert!(back.artifacts[0].hash.is_some());
}

// =============================================================================
// 18. ContextLogRecord — roundtrip
// =============================================================================

#[test]
fn context_log_record_roundtrip() {
    let record = ContextLogRecord {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool(),
        budget_tokens: 128_000,
        used_tokens: 110_000,
        utilization_pct: 85.9375,
        strategy: "greedy".into(),
        rank_by: "churn".into(),
        file_count: 42,
        total_bytes: 440_000,
        output_destination: "/tmp/context.json".into(),
    };

    let json = serde_json::to_string(&record).unwrap();
    let back: ContextLogRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(back.file_count, 42);
    assert_eq!(back.rank_by, "churn");
    assert_eq!(back.output_destination, "/tmp/context.json");
}

// =============================================================================
// 19. FileClassification — serde and ordering
// =============================================================================

#[test]
fn file_classification_all_variants_roundtrip() {
    for variant in &[
        FileClassification::Generated,
        FileClassification::Fixture,
        FileClassification::Vendored,
        FileClassification::Lockfile,
        FileClassification::Minified,
        FileClassification::DataBlob,
        FileClassification::Sourcemap,
    ] {
        let json = serde_json::to_string(variant).unwrap();
        let back: FileClassification = serde_json::from_str(&json).unwrap();
        assert_eq!(back, *variant);
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

#[test]
fn file_classification_is_sorted_and_deduped() {
    let mut classes = vec![
        FileClassification::Vendored,
        FileClassification::Minified,
        FileClassification::Vendored,
        FileClassification::Generated,
    ];
    classes.sort();
    classes.dedup();
    assert_eq!(classes.len(), 3);
    // Verify ordering
    for i in 0..classes.len() - 1 {
        assert!(classes[i] <= classes[i + 1]);
    }
}

// =============================================================================
// 20. PolicyExcludedFile — construction and serde
// =============================================================================

#[test]
fn policy_excluded_file_roundtrip() {
    let pef = PolicyExcludedFile {
        path: "vendor/heavy.js".into(),
        original_tokens: 200_000,
        policy: InclusionPolicy::Skip,
        reason: "vendored file exceeds cap".into(),
        classifications: vec![FileClassification::Vendored, FileClassification::Minified],
    };
    let json = serde_json::to_string(&pef).unwrap();
    let back: PolicyExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "vendor/heavy.js");
    assert_eq!(back.original_tokens, 200_000);
    assert_eq!(back.classifications.len(), 2);
}

// =============================================================================
// 21. SmartExcludedFile — construction
// =============================================================================

#[test]
fn smart_excluded_file_construction() {
    let sef = SmartExcludedFile {
        path: "yarn.lock".into(),
        reason: "lockfile".into(),
        tokens: 85_000,
    };
    assert_eq!(sef.path, "yarn.lock");
    assert_eq!(sef.tokens, 85_000);
}

// =============================================================================
// 22. HandoffComplexity — invariants
// =============================================================================

#[test]
fn handoff_complexity_avg_le_max_invariant() {
    let c = HandoffComplexity {
        total_functions: 200,
        avg_function_length: 20.0,
        max_function_length: 300,
        avg_cyclomatic: 4.5,
        max_cyclomatic: 50,
        high_risk_files: 5,
    };
    assert!(c.avg_function_length <= c.max_function_length as f64);
    assert!(c.avg_cyclomatic <= c.max_cyclomatic as f64);
    assert!(c.high_risk_files <= c.total_functions);
}

// =============================================================================
// 23. HandoffDerived — invariants
// =============================================================================

#[test]
fn handoff_derived_dominant_pct_bounded() {
    let d = HandoffDerived {
        total_files: 50,
        total_code: 10_000,
        total_lines: 15_000,
        total_tokens: 40_000,
        lang_count: 3,
        dominant_lang: "Rust".into(),
        dominant_pct: 75.0,
    };
    assert!((0.0..=100.0).contains(&d.dominant_pct));
    assert!(d.lang_count >= 1);
    assert!(d.total_code <= d.total_lines);
}

// =============================================================================
// 24. Cross-type — context receipt with estimation and audit
// =============================================================================

#[test]
fn context_receipt_with_estimation_and_audit_roundtrip() {
    let mut r = empty_context(128_000, 100_000);
    r.token_estimation = Some(TokenEstimationMeta::from_bytes(400_000, 4.0));
    r.bundle_audit = Some(TokenAudit::from_output(420_000, 400_000));

    let json: Value = serde_json::to_value(r).unwrap();
    assert!(json["token_estimation"].is_object());
    assert!(json["bundle_audit"].is_object());

    let est = &json["token_estimation"];
    assert!(est["tokens_min"].as_u64().unwrap() <= est["tokens_est"].as_u64().unwrap());
    assert!(est["tokens_est"].as_u64().unwrap() <= est["tokens_max"].as_u64().unwrap());
}

// =============================================================================
// 25. Cross-type — handoff optional fields omission
// =============================================================================

#[test]
fn handoff_optional_fields_omitted_when_none_or_empty() {
    let m = empty_handoff("receipt", 100_000, 50_000);
    let json = serde_json::to_string(&m).unwrap();

    assert!(!json.contains("\"rank_by_effective\""));
    assert!(!json.contains("\"fallback_reason\""));
    assert!(!json.contains("\"excluded_by_policy\""));
    assert!(!json.contains("\"token_estimation\""));
    assert!(!json.contains("\"code_audit\""));
}

// =============================================================================
// 26. Cross-type — context optional fields omission
// =============================================================================

#[test]
fn context_optional_fields_omitted_when_none_or_empty() {
    let r = empty_context(100_000, 50_000);
    let json = serde_json::to_string(&r).unwrap();

    assert!(!json.contains("\"rank_by_effective\""));
    assert!(!json.contains("\"fallback_reason\""));
    assert!(!json.contains("\"excluded_by_policy\""));
    assert!(!json.contains("\"token_estimation\""));
    assert!(!json.contains("\"bundle_audit\""));
}

// =============================================================================
// 27. ContextFileRow — classifications in serialization
// =============================================================================

#[test]
fn context_file_row_classifications_omitted_when_empty() {
    let row = ctx_row("src/lib.rs", 1_000);
    let json = serde_json::to_string(&row).unwrap();
    assert!(!json.contains("\"classifications\""));
}

#[test]
fn context_file_row_classifications_present_when_non_empty() {
    let mut row = ctx_row("vendor/lib.js", 50_000);
    row.classifications = vec![FileClassification::Vendored];
    let json = serde_json::to_string(&row).unwrap();
    assert!(json.contains("\"classifications\""));
    assert!(json.contains("\"vendored\""));
}

// =============================================================================
// 28. ContextFileRow — policy omitted when Full (default)
// =============================================================================

#[test]
fn context_file_row_full_policy_omitted_in_json() {
    let row = ctx_row("src/lib.rs", 1_000);
    let json = serde_json::to_string(&row).unwrap();
    assert!(!json.contains("\"policy\""));
}

#[test]
fn context_file_row_non_default_policy_present_in_json() {
    let row = ctx_row_with_policy("src/big.rs", 50_000, InclusionPolicy::HeadTail, 5_000);
    let json = serde_json::to_string(&row).unwrap();
    assert!(json.contains("\"policy\""));
    assert!(json.contains("\"head_tail\""));
}

// =============================================================================
// 29. ArtifactEntry — hash omission
// =============================================================================

#[test]
fn artifact_entry_hash_omitted_when_none() {
    let entry = ArtifactEntry {
        name: "map.jsonl".into(),
        path: "out/map.jsonl".into(),
        description: "File map".into(),
        bytes: 1_000,
        hash: None,
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(!json.contains("\"hash\""));
}

#[test]
fn artifact_entry_hash_present_when_some() {
    let entry = ArtifactEntry {
        name: "code.txt".into(),
        path: "out/code.txt".into(),
        description: "Source code".into(),
        bytes: 50_000,
        hash: Some(ArtifactHash {
            algo: "blake3".into(),
            hash: "abc123".into(),
        }),
    };
    let json: Value = serde_json::to_value(entry).unwrap();
    assert_eq!(json["hash"]["algo"], "blake3");
}

// =============================================================================
// 30. TokenEstimationMeta — from_bytes edge cases
// =============================================================================

#[test]
fn token_estimation_zero_bytes() {
    let est = TokenEstimationMeta::from_bytes(0, 4.0);
    assert_eq!(est.tokens_est, 0);
    assert_eq!(est.tokens_min, 0);
    assert_eq!(est.tokens_max, 0);
    assert_eq!(est.source_bytes, 0);
}

#[test]
fn token_estimation_one_byte() {
    let est = TokenEstimationMeta::from_bytes(1, 4.0);
    assert_eq!(est.tokens_est, 1); // ceil(1/4.0) = 1
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
}

#[test]
fn token_estimation_large_bytes() {
    let est = TokenEstimationMeta::from_bytes(10_000_000, 4.0);
    assert_eq!(est.tokens_est, 2_500_000);
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
}

// =============================================================================
// 31. TokenAudit — edge cases
// =============================================================================

#[test]
fn token_audit_content_exceeds_output_saturates() {
    // When content_bytes > output_bytes, overhead saturates to 0.
    let audit = TokenAudit::from_output(100, 200);
    assert_eq!(audit.overhead_bytes, 0);
    assert!((audit.overhead_pct - 0.0).abs() < f64::EPSILON);
}

// =============================================================================
// 32. HandoffManifest — JSON has no null required fields
// =============================================================================

#[test]
fn handoff_json_required_fields_never_null() {
    let m = empty_handoff("receipt", 100_000, 50_000);
    let json: Value = serde_json::to_value(m).unwrap();
    let obj = json.as_object().unwrap();
    for (key, value) in obj {
        assert!(!value.is_null(), "field '{key}' must not be null");
    }
}

// =============================================================================
// 33. ContextReceipt — JSON has no null required fields
// =============================================================================

#[test]
fn context_json_required_fields_never_null() {
    let r = empty_context(100_000, 50_000);
    let json: Value = serde_json::to_value(r).unwrap();
    let obj = json.as_object().unwrap();
    for (key, value) in obj {
        assert!(!value.is_null(), "field '{key}' must not be null");
    }
}

// =============================================================================
// 34. HandoffManifest — multiple inputs
// =============================================================================

#[test]
fn handoff_multiple_inputs() {
    let mut m = empty_handoff("receipt", 100_000, 50_000);
    m.inputs = vec!["src/".into(), "lib/".into(), "bin/".into()];
    let json = serde_json::to_string(&m).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.inputs.len(), 3);
}

// =============================================================================
// 35. HandoffIntelligence — roundtrip with all fields
// =============================================================================

#[test]
fn handoff_intelligence_full_roundtrip() {
    let intel = HandoffIntelligence {
        tree: Some("├── src/\n│   └── lib.rs".into()),
        tree_depth: Some(2),
        hotspots: Some(vec![HandoffHotspot {
            path: "hot.rs".into(),
            commits: 100,
            lines: 2000,
            score: 200_000,
        }]),
        complexity: Some(HandoffComplexity {
            total_functions: 500,
            avg_function_length: 18.5,
            max_function_length: 200,
            avg_cyclomatic: 6.0,
            max_cyclomatic: 45,
            high_risk_files: 12,
        }),
        derived: Some(HandoffDerived {
            total_files: 200,
            total_code: 80_000,
            total_lines: 120_000,
            total_tokens: 320_000,
            lang_count: 5,
            dominant_lang: "TypeScript".into(),
            dominant_pct: 55.0,
        }),
        warnings: vec!["high complexity".into(), "stale files detected".into()],
    };

    let json = serde_json::to_string(&intel).unwrap();
    let back: HandoffIntelligence = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tree_depth, Some(2));
    assert_eq!(back.hotspots.as_ref().unwrap().len(), 1);
    assert_eq!(back.complexity.as_ref().unwrap().total_functions, 500);
    assert_eq!(back.derived.as_ref().unwrap().lang_count, 5);
    assert_eq!(back.warnings.len(), 2);
}

// =============================================================================
// 36. Schema version independence
// =============================================================================

#[test]
fn schema_versions_are_all_independent() {
    // Each receipt family has its own schema version constant.
    let versions = [
        HANDOFF_SCHEMA_VERSION,
        CONTEXT_SCHEMA_VERSION,
        CONTEXT_BUNDLE_SCHEMA_VERSION,
    ];
    // At least two must differ (they currently all differ).
    let unique: std::collections::HashSet<u32> = versions.iter().copied().collect();
    assert!(unique.len() >= 2, "schema versions should be independent");
}

// =============================================================================
// 37. ContextReceipt — rank_by fallback
// =============================================================================

#[test]
fn context_receipt_fallback_fields_present_when_set() {
    let mut r = empty_context(128_000, 100_000);
    r.rank_by = "churn".into();
    r.rank_by_effective = Some("code".into());
    r.fallback_reason = Some("git not available".into());

    let json: Value = serde_json::to_value(r).unwrap();
    assert_eq!(json["rank_by"], "churn");
    assert_eq!(json["rank_by_effective"], "code");
    assert_eq!(json["fallback_reason"], "git not available");
}

// =============================================================================
// 38. ContextReceipt — deterministic serialization
// =============================================================================

#[test]
fn context_receipt_deterministic_serialization() {
    let r = empty_context(128_000, 80_000);
    let a = serde_json::to_string_pretty(&r).unwrap();
    let b = serde_json::to_string_pretty(&r).unwrap();
    assert_eq!(a, b);
}

// =============================================================================
// 39. HandoffManifest — excluded_patterns
// =============================================================================

#[test]
fn handoff_excluded_patterns_preserved() {
    let mut m = empty_handoff("receipt", 100_000, 50_000);
    m.excluded_patterns = vec!["*.lock".into(), "*.min.js".into(), "*.js.map".into()];
    let json = serde_json::to_string(&m).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded_patterns.len(), 3);
    assert!(back.excluded_patterns.contains(&"*.lock".to_string()));
}

// =============================================================================
// 40. HandoffManifest — large file count stress
// =============================================================================

#[test]
fn handoff_large_file_count_roundtrip() {
    let mut m = empty_handoff("deep", 1_000_000, 900_000);
    m.included_files = (0..100)
        .map(|i| ctx_row(&format!("src/file_{i}.rs"), 1_000))
        .collect();
    m.total_files = 500;
    m.bundled_files = 100;

    let json = serde_json::to_string(&m).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.included_files.len(), 100);
    assert_eq!(back.total_files, 500);
    assert_eq!(back.bundled_files, 100);
}

// =============================================================================
// 41. ContextReceipt — large file list
// =============================================================================

#[test]
fn context_receipt_large_file_list_roundtrip() {
    let mut r = empty_context(500_000, 400_000);
    r.files = (0..200)
        .map(|i| ctx_row(&format!("src/mod_{i}.rs"), 2_000))
        .collect();
    r.file_count = 200;

    let json = serde_json::to_string(&r).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 200);
    assert_eq!(back.file_count, 200);
}

// =============================================================================
// 42. TokenEstimationMeta — serde alias compatibility
// =============================================================================

#[test]
fn token_estimation_alias_tokens_high_deserializes_as_tokens_min() {
    let json = r#"{
        "bytes_per_token_est": 4.0,
        "bytes_per_token_low": 3.0,
        "bytes_per_token_high": 5.0,
        "tokens_high": 200,
        "tokens_est": 250,
        "tokens_low": 334,
        "source_bytes": 1000
    }"#;
    let est: TokenEstimationMeta = serde_json::from_str(json).unwrap();
    assert_eq!(est.tokens_min, 200);
    assert_eq!(est.tokens_max, 334);
}

// =============================================================================
// 43. TokenAudit — serde alias compatibility
// =============================================================================

#[test]
fn token_audit_alias_tokens_high_deserializes_as_tokens_min() {
    let json = r#"{
        "output_bytes": 5000,
        "tokens_high": 1000,
        "tokens_est": 1250,
        "tokens_low": 1667,
        "overhead_bytes": 500,
        "overhead_pct": 0.1
    }"#;
    let audit: TokenAudit = serde_json::from_str(json).unwrap();
    assert_eq!(audit.tokens_min, 1000);
    assert_eq!(audit.tokens_max, 1667);
}
