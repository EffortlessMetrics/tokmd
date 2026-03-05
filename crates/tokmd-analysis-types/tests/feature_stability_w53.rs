//! Feature-stability tests for WASM readiness seams.
//!
//! These tests verify that tokmd-analysis-types works correctly WITHOUT
//! optional features. They must NOT use `#[cfg(feature = ...)]` guards.

use tokmd_analysis_types::*;
use tokmd_types::{ScanStatus, ToolInfo};

// ── Schema constants ──────────────────────────────────────────────────

#[test]
fn analysis_schema_version_is_accessible() {
    const { assert!(ANALYSIS_SCHEMA_VERSION >= 8) };
}

#[test]
fn baseline_version_is_accessible() {
    const { assert!(BASELINE_VERSION >= 1) };
}

// ── Core type construction ────────────────────────────────────────────

fn minimal_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".into()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: "separate".into(),
    }
}

fn minimal_args() -> AnalysisArgsMeta {
    AnalysisArgsMeta {
        preset: "receipt".into(),
        format: "json".into(),
        window_tokens: None,
        git: None,
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".into(),
    }
}

#[test]
fn analysis_receipt_construction() {
    let receipt = AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo::default(),
        mode: "analyze".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: minimal_source(),
        args: minimal_args(),
        archetype: None,
        topics: None,
        entropy: None,
        predictive_churn: None,
        corporate_fingerprint: None,
        license: None,
        derived: None,
        assets: None,
        deps: None,
        git: None,
        imports: None,
        dup: None,
        complexity: None,
        api_surface: None,
        fun: None,
    };
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert_eq!(receipt.mode, "analyze");
}

#[test]
fn analysis_receipt_serde_roundtrip() {
    let receipt = AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 12345,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "0.1.0".into(),
        },
        mode: "analyze".into(),
        status: ScanStatus::Complete,
        warnings: vec!["test warning".into()],
        source: minimal_source(),
        args: minimal_args(),
        archetype: None,
        topics: None,
        entropy: None,
        predictive_churn: None,
        corporate_fingerprint: None,
        license: None,
        derived: None,
        assets: None,
        deps: None,
        git: None,
        imports: None,
        dup: None,
        complexity: None,
        api_surface: None,
        fun: None,
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let restored: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert_eq!(restored.warnings, vec!["test warning"]);
}

// ── Enrichment result types ───────────────────────────────────────────

#[test]
fn entropy_finding_construction() {
    let f = EntropyFinding {
        path: "secrets.bin".into(),
        module: "(root)".into(),
        entropy_bits_per_byte: 7.5,
        sample_bytes: 1024,
        class: EntropyClass::High,
    };
    assert_eq!(f.class, EntropyClass::High);
}

#[test]
fn entropy_class_serde_roundtrip() {
    for (variant, expected) in [
        (EntropyClass::Low, "\"low\""),
        (EntropyClass::Normal, "\"normal\""),
        (EntropyClass::Suspicious, "\"suspicious\""),
        (EntropyClass::High, "\"high\""),
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
        let restored: EntropyClass = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, variant);
    }
}

#[test]
fn archetype_construction() {
    let a = Archetype {
        kind: "web-app".into(),
        evidence: vec!["package.json".into()],
    };
    assert_eq!(a.kind, "web-app");
}

#[test]
fn fun_report_construction() {
    let f = FunReport {
        eco_label: Some(EcoLabel {
            score: 85.0,
            label: "A".into(),
            bytes: 50000,
            notes: String::new(),
        }),
    };
    assert!((f.eco_label.as_ref().unwrap().score - 85.0).abs() < f64::EPSILON);
}

#[test]
fn complexity_risk_serde_roundtrip() {
    for (variant, expected) in [
        (ComplexityRisk::Low, "\"low\""),
        (ComplexityRisk::Moderate, "\"moderate\""),
        (ComplexityRisk::High, "\"high\""),
        (ComplexityRisk::Critical, "\"critical\""),
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
    }
}

#[test]
fn import_edge_construction() {
    let edge = ImportEdge {
        from: "mod_a".into(),
        to: "mod_b".into(),
        count: 3,
    };
    let json = serde_json::to_string(&edge).unwrap();
    let restored: ImportEdge = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.from, "mod_a");
    assert_eq!(restored.count, 3);
}
