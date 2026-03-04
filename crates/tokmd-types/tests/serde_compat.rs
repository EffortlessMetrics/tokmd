//! Serde backward-compatibility and fixture tests for `tokmd-types`.
//!
//! These tests ensure that:
//! * Known JSON fixtures from prior schema versions still deserialize.
//! * Optional field additions do not break old payloads.
//! * Serde aliases are respected.
//! * Unknown fields are silently ignored.
//! * Null / missing optional fields default correctly.
//! * JSON output is deterministic (same struct → same JSON every time).
//! * Roundtrips work for every public type.
//! * JSONL parsing works line-by-line.

use serde_json::{Value, json};
use tokmd_types::cockpit::*;
use tokmd_types::*;

// ═══════════════════════════════════════════════════════════════════════════
// 1. Known JSON fixture deserialization
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fixture_totals_v2() {
    let json = r#"{"code":100,"lines":200,"files":10,"bytes":5000,"tokens":250,"avg_lines":20}"#;
    let t: Totals = serde_json::from_str(json).unwrap();
    assert_eq!(t.code, 100);
    assert_eq!(t.lines, 200);
    assert_eq!(t.files, 10);
    assert_eq!(t.bytes, 5000);
    assert_eq!(t.tokens, 250);
    assert_eq!(t.avg_lines, 20);
}

#[test]
fn fixture_lang_row_v2() {
    let json = r#"{"lang":"Rust","code":500,"lines":750,"files":12,"bytes":20000,"tokens":3000,"avg_lines":62}"#;
    let r: LangRow = serde_json::from_str(json).unwrap();
    assert_eq!(r.lang, "Rust");
    assert_eq!(r.code, 500);
    assert_eq!(r.avg_lines, 62);
}

#[test]
fn fixture_file_row_v2() {
    let json = r#"{
        "path":"src/main.rs","module":"src","lang":"Rust","kind":"parent",
        "code":50,"comments":10,"blanks":5,"lines":65,"bytes":2000,"tokens":100
    }"#;
    let r: FileRow = serde_json::from_str(json).unwrap();
    assert_eq!(r.path, "src/main.rs");
    assert_eq!(r.kind, FileKind::Parent);
    assert_eq!(r.code, 50);
}

#[test]
fn fixture_diff_row_v2() {
    let json = r#"{
        "lang":"Go","old_code":100,"new_code":120,"delta_code":20,
        "old_lines":200,"new_lines":220,"delta_lines":20,
        "old_files":5,"new_files":6,"delta_files":1,
        "old_bytes":3000,"new_bytes":3600,"delta_bytes":600,
        "old_tokens":400,"new_tokens":480,"delta_tokens":80
    }"#;
    let r: DiffRow = serde_json::from_str(json).unwrap();
    assert_eq!(r.lang, "Go");
    assert_eq!(r.delta_code, 20);
    assert_eq!(r.delta_tokens, 80);
}

#[test]
fn fixture_diff_totals_v2() {
    let json = r#"{
        "old_code":1000,"new_code":1200,"delta_code":200,
        "old_lines":2000,"new_lines":2200,"delta_lines":200,
        "old_files":50,"new_files":55,"delta_files":5,
        "old_bytes":40000,"new_bytes":48000,"delta_bytes":8000,
        "old_tokens":5000,"new_tokens":6000,"delta_tokens":1000
    }"#;
    let t: DiffTotals = serde_json::from_str(json).unwrap();
    assert_eq!(t.delta_code, 200);
    assert_eq!(t.delta_tokens, 1000);
}

#[test]
fn fixture_module_row_v2() {
    let json = r#"{"module":"src/core","code":300,"lines":450,"files":8,"bytes":12000,"tokens":1500,"avg_lines":56}"#;
    let r: ModuleRow = serde_json::from_str(json).unwrap();
    assert_eq!(r.module, "src/core");
    assert_eq!(r.code, 300);
}

#[test]
fn fixture_tool_info_v2() {
    let json = r#"{"name":"tokmd","version":"0.9.0"}"#;
    let ti: ToolInfo = serde_json::from_str(json).unwrap();
    assert_eq!(ti.name, "tokmd");
    assert_eq!(ti.version, "0.9.0");
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Optional field addition doesn't break old JSON
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn context_receipt_missing_optional_fields() {
    // Simulate a v3 payload that lacks v4 optional fields
    let json = r#"{
        "schema_version":3,"generated_at_ms":1000,"tool":{"name":"tokmd","version":"0.8.0"},
        "mode":"context","budget_tokens":128000,"used_tokens":50000,"utilization_pct":0.39,
        "strategy":"greedy","rank_by":"tokens","file_count":5,"files":[]
    }"#;
    let r: ContextReceipt = serde_json::from_str(json).unwrap();
    assert!(r.rank_by_effective.is_none());
    assert!(r.fallback_reason.is_none());
    assert!(r.excluded_by_policy.is_empty());
    assert!(r.token_estimation.is_none());
    assert!(r.bundle_audit.is_none());
}

#[test]
fn handoff_manifest_missing_optional_fields() {
    let json = r#"{
        "schema_version":4,"generated_at_ms":2000,"tool":{"name":"tokmd","version":"0.8.0"},
        "mode":"handoff","inputs":["src"],"output_dir":"out","budget_tokens":100000,
        "used_tokens":80000,"utilization_pct":0.8,"strategy":"greedy","rank_by":"tokens",
        "capabilities":[],"artifacts":[],"included_files":[],"excluded_paths":[],
        "excluded_patterns":[],"smart_excluded_files":[],"total_files":100,
        "bundled_files":80,"intelligence_preset":"receipt"
    }"#;
    let m: HandoffManifest = serde_json::from_str(json).unwrap();
    assert!(m.rank_by_effective.is_none());
    assert!(m.fallback_reason.is_none());
    assert!(m.excluded_by_policy.is_empty());
    assert!(m.token_estimation.is_none());
    assert!(m.code_audit.is_none());
}

#[test]
fn context_bundle_manifest_missing_optional_fields() {
    let json = r#"{
        "schema_version":1,"generated_at_ms":3000,"tool":{"name":"tokmd","version":"0.8.0"},
        "mode":"context-bundle","budget_tokens":100000,"used_tokens":60000,
        "utilization_pct":0.6,"strategy":"greedy","rank_by":"tokens",
        "file_count":10,"bundle_bytes":50000,"artifacts":[],"included_files":[],
        "excluded_paths":[],"excluded_patterns":[]
    }"#;
    let m: ContextBundleManifest = serde_json::from_str(json).unwrap();
    assert!(m.rank_by_effective.is_none());
    assert!(m.fallback_reason.is_none());
    assert!(m.excluded_by_policy.is_empty());
    assert!(m.token_estimation.is_none());
    assert!(m.bundle_audit.is_none());
}

#[test]
fn scan_args_missing_excluded_redacted() {
    let json = r#"{
        "paths":["."],"excluded":[],"config":"auto","hidden":false,
        "no_ignore":false,"no_ignore_parent":false,"no_ignore_dot":false,
        "no_ignore_vcs":false,"treat_doc_strings_as_comments":false
    }"#;
    let s: ScanArgs = serde_json::from_str(json).unwrap();
    assert!(!s.excluded_redacted);
}

#[test]
fn export_args_meta_missing_strip_prefix_redacted() {
    let json = r#"{
        "format":"csv","module_roots":[],"module_depth":1,"children":"separate",
        "min_code":0,"max_rows":10000,"redact":"none","strip_prefix":null
    }"#;
    let e: ExportArgsMeta = serde_json::from_str(json).unwrap();
    assert!(!e.strip_prefix_redacted);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Serde aliases (field renames with backward compat)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn token_estimation_alias_tokens_high_maps_to_tokens_min() {
    let json = r#"{
        "bytes_per_token_est":4.0,"bytes_per_token_low":3.0,"bytes_per_token_high":5.0,
        "tokens_high":800,"tokens_est":1000,"tokens_low":1334,"source_bytes":4000
    }"#;
    let est: TokenEstimationMeta = serde_json::from_str(json).unwrap();
    assert_eq!(est.tokens_min, 800);
    assert_eq!(est.tokens_max, 1334);
}

#[test]
fn token_estimation_canonical_names_work() {
    let json = r#"{
        "bytes_per_token_est":4.0,"bytes_per_token_low":3.0,"bytes_per_token_high":5.0,
        "tokens_min":800,"tokens_est":1000,"tokens_max":1334,"source_bytes":4000
    }"#;
    let est: TokenEstimationMeta = serde_json::from_str(json).unwrap();
    assert_eq!(est.tokens_min, 800);
    assert_eq!(est.tokens_max, 1334);
}

#[test]
fn token_audit_alias_tokens_high_maps_to_tokens_min() {
    let json = r#"{
        "output_bytes":5000,"tokens_high":1000,"tokens_est":1250,
        "tokens_low":1667,"overhead_bytes":500,"overhead_pct":0.1
    }"#;
    let audit: TokenAudit = serde_json::from_str(json).unwrap();
    assert_eq!(audit.tokens_min, 1000);
    assert_eq!(audit.tokens_max, 1667);
}

#[test]
fn token_audit_canonical_names_work() {
    let json = r#"{
        "output_bytes":5000,"tokens_min":1000,"tokens_est":1250,
        "tokens_max":1667,"overhead_bytes":500,"overhead_pct":0.1
    }"#;
    let audit: TokenAudit = serde_json::from_str(json).unwrap();
    assert_eq!(audit.tokens_min, 1000);
    assert_eq!(audit.tokens_max, 1667);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Unknown fields in JSON are ignored
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn totals_ignores_unknown_fields() {
    let json = r#"{"code":1,"lines":2,"files":3,"bytes":4,"tokens":5,"avg_lines":6,"future_field":"hello","another":42}"#;
    let t: Totals = serde_json::from_str(json).unwrap();
    assert_eq!(t.code, 1);
}

#[test]
fn lang_row_ignores_unknown_fields() {
    let json = r#"{"lang":"Rust","code":1,"lines":2,"files":3,"bytes":4,"tokens":5,"avg_lines":6,"new_metric":99}"#;
    let r: LangRow = serde_json::from_str(json).unwrap();
    assert_eq!(r.lang, "Rust");
}

#[test]
fn file_row_ignores_unknown_fields() {
    let json = r#"{
        "path":"a.rs","module":"m","lang":"Rust","kind":"parent",
        "code":1,"comments":2,"blanks":3,"lines":6,"bytes":100,"tokens":10,
        "complexity":42
    }"#;
    let r: FileRow = serde_json::from_str(json).unwrap();
    assert_eq!(r.code, 1);
}

#[test]
fn diff_row_ignores_unknown_fields() {
    let json = r#"{
        "lang":"C","old_code":1,"new_code":2,"delta_code":1,
        "old_lines":10,"new_lines":12,"delta_lines":2,
        "old_files":1,"new_files":1,"delta_files":0,
        "old_bytes":100,"new_bytes":120,"delta_bytes":20,
        "old_tokens":50,"new_tokens":60,"delta_tokens":10,
        "future_delta_something":999
    }"#;
    let r: DiffRow = serde_json::from_str(json).unwrap();
    assert_eq!(r.lang, "C");
}

#[test]
fn module_row_ignores_unknown_fields() {
    let json = r#"{"module":"lib","code":10,"lines":20,"files":1,"bytes":100,"tokens":50,"avg_lines":20,"depth":3}"#;
    let r: ModuleRow = serde_json::from_str(json).unwrap();
    assert_eq!(r.module, "lib");
}

#[test]
fn tool_info_ignores_unknown_fields() {
    let json = r#"{"name":"tokmd","version":"1.0.0","build_hash":"abc123"}"#;
    let ti: ToolInfo = serde_json::from_str(json).unwrap();
    assert_eq!(ti.name, "tokmd");
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Null values for optional fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn context_receipt_null_optionals() {
    let json = r#"{
        "schema_version":4,"generated_at_ms":1000,"tool":{"name":"tokmd","version":"0.9.0"},
        "mode":"context","budget_tokens":128000,"used_tokens":50000,"utilization_pct":0.39,
        "strategy":"greedy","rank_by":"tokens","file_count":0,"files":[],
        "rank_by_effective":null,"fallback_reason":null,
        "token_estimation":null,"bundle_audit":null
    }"#;
    let r: ContextReceipt = serde_json::from_str(json).unwrap();
    assert!(r.rank_by_effective.is_none());
    assert!(r.fallback_reason.is_none());
    assert!(r.token_estimation.is_none());
    assert!(r.bundle_audit.is_none());
}

#[test]
fn context_file_row_null_optionals() {
    let json = r#"{
        "path":"a.rs","module":"m","lang":"Rust","tokens":100,"code":80,"lines":120,
        "bytes":500,"value":100,"effective_tokens":null,"policy_reason":null
    }"#;
    let r: ContextFileRow = serde_json::from_str(json).unwrap();
    assert!(r.effective_tokens.is_none());
    assert!(r.policy_reason.is_none());
}

#[test]
fn export_args_meta_null_strip_prefix() {
    let json = r#"{
        "format":"jsonl","module_roots":[],"module_depth":1,"children":"parents-only",
        "min_code":0,"max_rows":5000,"redact":"none","strip_prefix":null
    }"#;
    let e: ExportArgsMeta = serde_json::from_str(json).unwrap();
    assert!(e.strip_prefix.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Missing optional fields default correctly
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn context_file_row_defaults() {
    let json = r#"{
        "path":"b.py","module":"src","lang":"Python","tokens":200,"code":150,"lines":300,
        "bytes":1000,"value":200
    }"#;
    let r: ContextFileRow = serde_json::from_str(json).unwrap();
    assert_eq!(r.rank_reason, "");
    assert_eq!(r.policy, InclusionPolicy::Full);
    assert!(r.effective_tokens.is_none());
    assert!(r.policy_reason.is_none());
    assert!(r.classifications.is_empty());
}

#[test]
fn cockpit_receipt_missing_trend() {
    let json = make_cockpit_json(false);
    let r: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert!(r.trend.is_none());
}

#[test]
fn trend_comparison_defaults() {
    let t = TrendComparison::default();
    assert!(!t.baseline_available);
    assert!(t.baseline_path.is_none());
    assert!(t.baseline_generated_at_ms.is_none());
    assert!(t.health.is_none());
    assert!(t.risk.is_none());
    assert!(t.complexity.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Schema version constants are positive integers
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn schema_version_is_positive() {
    const {
        assert!(SCHEMA_VERSION > 0);
    }
}

#[test]
fn handoff_schema_version_is_positive() {
    const {
        assert!(HANDOFF_SCHEMA_VERSION > 0);
    }
}

#[test]
fn context_schema_version_is_positive() {
    const {
        assert!(CONTEXT_SCHEMA_VERSION > 0);
    }
}

#[test]
fn context_bundle_schema_version_is_positive() {
    const {
        assert!(CONTEXT_BUNDLE_SCHEMA_VERSION > 0);
    }
}

#[test]
fn cockpit_schema_version_is_positive() {
    const {
        assert!(COCKPIT_SCHEMA_VERSION > 0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. JSON output is deterministic
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn totals_json_deterministic() {
    let t = Totals {
        code: 100,
        lines: 200,
        files: 10,
        bytes: 5000,
        tokens: 250,
        avg_lines: 20,
    };
    let a = serde_json::to_string(&t).unwrap();
    let b = serde_json::to_string(&t).unwrap();
    assert_eq!(a, b);
}

#[test]
fn lang_row_json_deterministic() {
    let r = LangRow {
        lang: "Rust".into(),
        code: 100,
        lines: 150,
        files: 5,
        bytes: 3000,
        tokens: 200,
        avg_lines: 30,
    };
    let a = serde_json::to_string(&r).unwrap();
    let b = serde_json::to_string(&r).unwrap();
    assert_eq!(a, b);
}

#[test]
fn diff_totals_json_deterministic() {
    let t = DiffTotals {
        old_code: 100,
        new_code: 120,
        delta_code: 20,
        ..DiffTotals::default()
    };
    let a = serde_json::to_string(&t).unwrap();
    let b = serde_json::to_string(&t).unwrap();
    assert_eq!(a, b);
}

#[test]
fn token_estimation_json_deterministic() {
    let est = TokenEstimationMeta::from_bytes(4000, 4.0);
    let a = serde_json::to_string(&est).unwrap();
    let b = serde_json::to_string(&est).unwrap();
    assert_eq!(a, b);
}

#[test]
fn file_row_json_deterministic() {
    let r = FileRow {
        path: "src/main.rs".into(),
        module: "src".into(),
        lang: "Rust".into(),
        kind: FileKind::Parent,
        code: 50,
        comments: 10,
        blanks: 5,
        lines: 65,
        bytes: 2000,
        tokens: 100,
    };
    let a = serde_json::to_string(&r).unwrap();
    let b = serde_json::to_string(&r).unwrap();
    assert_eq!(a, b);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. JSON roundtrip for every public type
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn roundtrip_totals() {
    let orig = Totals {
        code: 42,
        lines: 100,
        files: 3,
        bytes: 1500,
        tokens: 200,
        avg_lines: 33,
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: Totals = serde_json::from_str(&json).unwrap();
    assert_eq!(orig, back);
}

#[test]
fn roundtrip_module_row() {
    let orig = ModuleRow {
        module: "crates/core".into(),
        code: 999,
        lines: 1500,
        files: 20,
        bytes: 40000,
        tokens: 5000,
        avg_lines: 75,
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: ModuleRow = serde_json::from_str(&json).unwrap();
    assert_eq!(orig, back);
}

#[test]
fn roundtrip_token_estimation_meta() {
    let orig = TokenEstimationMeta::from_bytes(8000, 4.0);
    let json = serde_json::to_string(&orig).unwrap();
    let back: TokenEstimationMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tokens_est, orig.tokens_est);
    assert_eq!(back.tokens_min, orig.tokens_min);
    assert_eq!(back.tokens_max, orig.tokens_max);
    assert_eq!(back.source_bytes, orig.source_bytes);
}

#[test]
fn roundtrip_token_audit() {
    let orig = TokenAudit::from_output(10000, 9000);
    let json = serde_json::to_string(&orig).unwrap();
    let back: TokenAudit = serde_json::from_str(&json).unwrap();
    assert_eq!(back.output_bytes, orig.output_bytes);
    assert_eq!(back.overhead_bytes, orig.overhead_bytes);
    assert_eq!(back.tokens_est, orig.tokens_est);
}

#[test]
fn roundtrip_policy_excluded_file() {
    let orig = PolicyExcludedFile {
        path: "vendor/lib.js".into(),
        original_tokens: 5000,
        policy: InclusionPolicy::Skip,
        reason: "vendored code".into(),
        classifications: vec![FileClassification::Vendored],
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: PolicyExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "vendor/lib.js");
    assert_eq!(back.policy, InclusionPolicy::Skip);
}

#[test]
fn roundtrip_smart_excluded_file() {
    let orig = SmartExcludedFile {
        path: "package-lock.json".into(),
        reason: "lockfile".into(),
        tokens: 100000,
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: SmartExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, orig.path);
    assert_eq!(back.tokens, orig.tokens);
}

#[test]
fn roundtrip_artifact_entry() {
    let orig = ArtifactEntry {
        name: "bundle.txt".into(),
        path: "out/bundle.txt".into(),
        description: "Code bundle".into(),
        bytes: 50000,
        hash: Some(ArtifactHash {
            algo: "blake3".into(),
            hash: "abc123".into(),
        }),
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: ArtifactEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "bundle.txt");
    assert!(back.hash.is_some());
}

#[test]
fn roundtrip_artifact_entry_no_hash() {
    let orig = ArtifactEntry {
        name: "receipt.json".into(),
        path: "out/receipt.json".into(),
        description: "Receipt".into(),
        bytes: 1234,
        hash: None,
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: ArtifactEntry = serde_json::from_str(&json).unwrap();
    assert!(back.hash.is_none());
}

#[test]
fn roundtrip_capability_status() {
    let orig = CapabilityStatus {
        name: "git".into(),
        status: CapabilityState::Available,
        reason: Some("in a git repo".into()),
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: CapabilityStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "git");
    assert_eq!(back.status, CapabilityState::Available);
}

#[test]
fn roundtrip_context_excluded_path() {
    let orig = ContextExcludedPath {
        path: "node_modules".into(),
        reason: "excluded by pattern".into(),
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: ContextExcludedPath = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, orig.path);
}

#[test]
fn roundtrip_handoff_excluded_path() {
    let orig = HandoffExcludedPath {
        path: "target".into(),
        reason: "build output".into(),
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: HandoffExcludedPath = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, orig.path);
}

#[test]
fn roundtrip_handoff_hotspot() {
    let orig = HandoffHotspot {
        path: "src/lib.rs".into(),
        commits: 50,
        lines: 300,
        score: 15000,
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: HandoffHotspot = serde_json::from_str(&json).unwrap();
    assert_eq!(back.commits, 50);
    assert_eq!(back.score, 15000);
}

#[test]
fn roundtrip_handoff_complexity() {
    let orig = HandoffComplexity {
        total_functions: 100,
        avg_function_length: 25.5,
        max_function_length: 200,
        avg_cyclomatic: 3.2,
        max_cyclomatic: 15,
        high_risk_files: 2,
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: HandoffComplexity = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_functions, 100);
}

#[test]
fn roundtrip_handoff_derived() {
    let orig = HandoffDerived {
        total_files: 200,
        total_code: 50000,
        total_lines: 80000,
        total_tokens: 25000,
        lang_count: 5,
        dominant_lang: "Rust".into(),
        dominant_pct: 72.5,
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: HandoffDerived = serde_json::from_str(&json).unwrap();
    assert_eq!(back.dominant_lang, "Rust");
}

#[test]
fn roundtrip_context_log_record() {
    let orig = ContextLogRecord {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1234567890,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "1.0.0".into(),
        },
        budget_tokens: 128000,
        used_tokens: 50000,
        utilization_pct: 0.39,
        strategy: "greedy".into(),
        rank_by: "tokens".into(),
        file_count: 10,
        total_bytes: 200000,
        output_destination: "stdout".into(),
    };
    let json = serde_json::to_string(&orig).unwrap();
    let back: ContextLogRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(back.file_count, 10);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. JSONL format (one record per line)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn jsonl_lang_rows_parse_line_by_line() {
    let rows = [
        LangRow {
            lang: "Rust".into(),
            code: 100,
            lines: 150,
            files: 5,
            bytes: 3000,
            tokens: 200,
            avg_lines: 30,
        },
        LangRow {
            lang: "Python".into(),
            code: 200,
            lines: 300,
            files: 8,
            bytes: 6000,
            tokens: 400,
            avg_lines: 37,
        },
        LangRow {
            lang: "Go".into(),
            code: 50,
            lines: 70,
            files: 2,
            bytes: 1500,
            tokens: 100,
            avg_lines: 35,
        },
    ];

    // Serialize as JSONL
    let jsonl: String = rows
        .iter()
        .map(|r| serde_json::to_string(r).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    // Parse back line by line
    let parsed: Vec<LangRow> = jsonl
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0].lang, "Rust");
    assert_eq!(parsed[1].lang, "Python");
    assert_eq!(parsed[2].lang, "Go");
}

#[test]
fn jsonl_file_rows_parse_line_by_line() {
    let rows = [
        FileRow {
            path: "src/main.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 50,
            comments: 10,
            blanks: 5,
            lines: 65,
            bytes: 2000,
            tokens: 100,
        },
        FileRow {
            path: "src/lib.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 200,
            comments: 30,
            blanks: 20,
            lines: 250,
            bytes: 8000,
            tokens: 500,
        },
    ];

    let jsonl: String = rows
        .iter()
        .map(|r| serde_json::to_string(r).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    let parsed: Vec<FileRow> = jsonl
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();

    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0], rows[0]);
    assert_eq!(parsed[1], rows[1]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Cross-format consistency (JSON value matches field value)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn totals_json_field_values_match() {
    let t = Totals {
        code: 42,
        lines: 100,
        files: 3,
        bytes: 1500,
        tokens: 200,
        avg_lines: 33,
    };
    let v: Value = serde_json::to_value(t).unwrap();
    assert_eq!(v["code"], json!(42));
    assert_eq!(v["lines"], json!(100));
    assert_eq!(v["files"], json!(3));
    assert_eq!(v["bytes"], json!(1500));
    assert_eq!(v["tokens"], json!(200));
    assert_eq!(v["avg_lines"], json!(33));
}

#[test]
fn diff_row_json_field_values_match() {
    let r = DiffRow {
        lang: "Rust".into(),
        old_code: 100,
        new_code: 120,
        delta_code: 20,
        old_lines: 200,
        new_lines: 220,
        delta_lines: 20,
        old_files: 10,
        new_files: 11,
        delta_files: 1,
        old_bytes: 5000,
        new_bytes: 6000,
        delta_bytes: 1000,
        old_tokens: 250,
        new_tokens: 300,
        delta_tokens: 50,
    };
    let v: Value = serde_json::to_value(r).unwrap();
    assert_eq!(v["lang"], json!("Rust"));
    assert_eq!(v["delta_code"], json!(20));
    assert_eq!(v["delta_tokens"], json!(50));
}

#[test]
fn enum_json_values_are_kebab_case() {
    assert_eq!(
        serde_json::to_value(ChildIncludeMode::ParentsOnly).unwrap(),
        json!("parents-only")
    );
    assert_eq!(
        serde_json::to_value(ChildrenMode::Collapse).unwrap(),
        json!("collapse")
    );
    assert_eq!(
        serde_json::to_value(ExportFormat::Cyclonedx).unwrap(),
        json!("cyclonedx")
    );
}

#[test]
fn enum_json_values_are_snake_case_where_applicable() {
    assert_eq!(
        serde_json::to_value(FileKind::Parent).unwrap(),
        json!("parent")
    );
    assert_eq!(
        serde_json::to_value(FileKind::Child).unwrap(),
        json!("child")
    );
    assert_eq!(
        serde_json::to_value(FileClassification::DataBlob).unwrap(),
        json!("data_blob")
    );
    assert_eq!(
        serde_json::to_value(InclusionPolicy::HeadTail).unwrap(),
        json!("head_tail")
    );
}

#[test]
fn cockpit_enum_json_values() {
    assert_eq!(
        serde_json::to_value(GateStatus::Pass).unwrap(),
        json!("pass")
    );
    assert_eq!(
        serde_json::to_value(GateStatus::Warn).unwrap(),
        json!("warn")
    );
    assert_eq!(
        serde_json::to_value(GateStatus::Fail).unwrap(),
        json!("fail")
    );
    assert_eq!(
        serde_json::to_value(RiskLevel::Critical).unwrap(),
        json!("critical")
    );
    assert_eq!(
        serde_json::to_value(ComplexityIndicator::Medium).unwrap(),
        json!("medium")
    );
    assert_eq!(
        serde_json::to_value(TrendDirection::Improving).unwrap(),
        json!("improving")
    );
}

#[test]
fn commit_intent_kind_json_values_are_snake_case() {
    assert_eq!(
        serde_json::to_value(CommitIntentKind::Feat).unwrap(),
        json!("feat")
    );
    assert_eq!(
        serde_json::to_value(CommitIntentKind::Fix).unwrap(),
        json!("fix")
    );
    assert_eq!(
        serde_json::to_value(CommitIntentKind::Refactor).unwrap(),
        json!("refactor")
    );
}

#[test]
fn scan_status_json_values_are_snake_case() {
    assert_eq!(
        serde_json::to_value(ScanStatus::Complete).unwrap(),
        json!("complete")
    );
    assert_eq!(
        serde_json::to_value(ScanStatus::Partial).unwrap(),
        json!("partial")
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

fn make_cockpit_json(include_trend: bool) -> String {
    let trend = if include_trend {
        r#","trend":{"baseline_available":false}"#
    } else {
        ""
    };
    format!(
        r#"{{
        "schema_version":3,"mode":"cockpit","generated_at_ms":1000,
        "base_ref":"main","head_ref":"HEAD",
        "change_surface":{{"commits":1,"files_changed":2,"insertions":10,"deletions":5,"net_lines":5,"churn_velocity":15.0,"change_concentration":0.8}},
        "composition":{{"code_pct":70.0,"test_pct":20.0,"docs_pct":5.0,"config_pct":5.0,"test_ratio":0.28}},
        "code_health":{{"score":85,"grade":"B","large_files_touched":0,"avg_file_size":100,"complexity_indicator":"low","warnings":[]}},
        "risk":{{"hotspots_touched":[],"bus_factor_warnings":[],"level":"low","score":10}},
        "contracts":{{"api_changed":false,"cli_changed":false,"schema_changed":false,"breaking_indicators":0}},
        "evidence":{{"overall_status":"pass","mutation":{{"status":"skipped","source":"ran_local","commit_match":"unknown","scope":{{"relevant":[],"tested":[],"ratio":0.0}},"survivors":[],"killed":0,"timeout":0,"unviable":0}}}},
        "review_plan":[]
        {trend}
    }}"#
    )
}
