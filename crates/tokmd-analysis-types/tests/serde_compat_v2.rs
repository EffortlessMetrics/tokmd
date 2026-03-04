//! Backward-compatibility tests for serde rename conventions in
//! `tokmd-analysis-types` and `tokmd-envelope`.
//!
//! This file verifies that all `#[serde(rename_all = "...")]` and
//! `#[serde(rename = "...")]` attributes produce the expected JSON
//! wire names and survive round-trips, guarding against accidental
//! regressions when refactoring field or variant names.

use serde_json::{Value, json};
use tokmd_analysis_types::*;
use tokmd_envelope::{
    Artifact, CapabilityState as EnvCapState, FindingSeverity, SensorReport, ToolMeta, Verdict,
};

// ═══════════════════════════════════════════════════════════════════════════
// tokmd-analysis-types enums — rename_all = "snake_case"
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn entropy_class_all_variants_snake_case() {
    let variants = [
        (EntropyClass::Low, "low"),
        (EntropyClass::Normal, "normal"),
        (EntropyClass::Suspicious, "suspicious"),
        (EntropyClass::High, "high"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "EntropyClass::{variant:?}"
        );
        let back: EntropyClass = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn trend_class_all_variants_snake_case() {
    let variants = [
        (TrendClass::Rising, "rising"),
        (TrendClass::Flat, "flat"),
        (TrendClass::Falling, "falling"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, format!("\"{}\"", expected), "TrendClass::{variant:?}");
        let back: TrendClass = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn license_source_kind_all_variants_snake_case() {
    let variants = [
        (LicenseSourceKind::Metadata, "metadata"),
        (LicenseSourceKind::Text, "text"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "LicenseSourceKind::{variant:?}"
        );
        let back: LicenseSourceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn complexity_risk_all_variants_snake_case() {
    let variants = [
        (ComplexityRisk::Low, "low"),
        (ComplexityRisk::Moderate, "moderate"),
        (ComplexityRisk::High, "high"),
        (ComplexityRisk::Critical, "critical"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "ComplexityRisk::{variant:?}"
        );
        let back: ComplexityRisk = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn technical_debt_level_all_variants_snake_case() {
    let variants = [
        (TechnicalDebtLevel::Low, "low"),
        (TechnicalDebtLevel::Moderate, "moderate"),
        (TechnicalDebtLevel::High, "high"),
        (TechnicalDebtLevel::Critical, "critical"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "TechnicalDebtLevel::{variant:?}"
        );
        let back: TechnicalDebtLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NearDupScope — rename_all = "kebab-case"
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn near_dup_scope_all_variants_kebab_case() {
    let variants = [
        (NearDupScope::Module, "module"),
        (NearDupScope::Lang, "lang"),
        (NearDupScope::Global, "global"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "NearDupScope::{variant:?}"
        );
        let back: NearDupScope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn near_dup_scope_default_is_module() {
    let scope = NearDupScope::default();
    assert_eq!(scope, NearDupScope::Module);
    assert_eq!(serde_json::to_string(&scope).unwrap(), "\"module\"");
}

// ═══════════════════════════════════════════════════════════════════════════
// Struct round-trip tests (analysis types)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn entropy_finding_roundtrip() {
    let finding = EntropyFinding {
        path: "secrets.env".into(),
        module: "root".into(),
        entropy_bits_per_byte: 7.5,
        sample_bytes: 1024,
        class: EntropyClass::High,
    };
    let json = serde_json::to_value(finding).unwrap();
    assert_eq!(json["class"], "high");
    let back: EntropyFinding = serde_json::from_value(json).unwrap();
    assert_eq!(back.class, EntropyClass::High);
    assert_eq!(back.path, "secrets.env");
}

#[test]
fn churn_trend_roundtrip_with_classification() {
    let trend = ChurnTrend {
        slope: 1.5,
        r2: 0.85,
        recent_change: 42,
        classification: TrendClass::Rising,
    };
    let json = serde_json::to_value(trend).unwrap();
    assert_eq!(json["classification"], "rising");
    let back: ChurnTrend = serde_json::from_value(json).unwrap();
    assert_eq!(back.classification, TrendClass::Rising);
}

#[test]
fn file_complexity_roundtrip_with_risk_level() {
    let fc = FileComplexity {
        path: "src/parser.rs".into(),
        module: "src".into(),
        function_count: 15,
        max_function_length: 120,
        cyclomatic_complexity: 25,
        cognitive_complexity: Some(30),
        max_nesting: Some(5),
        risk_level: ComplexityRisk::High,
        functions: None,
    };
    let json = serde_json::to_value(fc).unwrap();
    assert_eq!(json["risk_level"], "high");
    let back: FileComplexity = serde_json::from_value(json).unwrap();
    assert_eq!(back.risk_level, ComplexityRisk::High);
    assert_eq!(back.cognitive_complexity, Some(30));
}

#[test]
fn technical_debt_ratio_roundtrip() {
    let td = TechnicalDebtRatio {
        ratio: 12.5,
        complexity_points: 250,
        code_kloc: 20.0,
        level: TechnicalDebtLevel::Moderate,
    };
    let json = serde_json::to_value(td).unwrap();
    assert_eq!(json["level"], "moderate");
    let back: TechnicalDebtRatio = serde_json::from_value(json).unwrap();
    assert_eq!(back.level, TechnicalDebtLevel::Moderate);
}

#[test]
fn near_dup_params_roundtrip_with_scope() {
    let params = NearDupParams {
        scope: NearDupScope::Global,
        threshold: 0.8,
        max_files: 1000,
        max_pairs: Some(5000),
        max_file_bytes: Some(100_000),
        selection_method: Some("all".into()),
        algorithm: Some(NearDupAlgorithm {
            k_gram_size: 5,
            window_size: 4,
            max_postings: 50,
        }),
        exclude_patterns: vec!["*.lock".into()],
    };
    let json = serde_json::to_value(params).unwrap();
    assert_eq!(json["scope"], "global");
    let back: NearDupParams = serde_json::from_value(json).unwrap();
    assert_eq!(back.scope, NearDupScope::Global);
    assert_eq!(back.max_pairs, Some(5000));
}

#[test]
fn code_age_distribution_uses_trend_class() {
    let dist = CodeAgeDistributionReport {
        buckets: vec![CodeAgeBucket {
            label: "0-30 days".into(),
            min_days: 0,
            max_days: Some(30),
            files: 10,
            pct: 50.0,
        }],
        recent_refreshes: 5,
        prior_refreshes: 3,
        refresh_trend: TrendClass::Falling,
    };
    let json = serde_json::to_value(dist).unwrap();
    assert_eq!(json["refresh_trend"], "falling");
    let back: CodeAgeDistributionReport = serde_json::from_value(json).unwrap();
    assert_eq!(back.refresh_trend, TrendClass::Falling);
}

// ═══════════════════════════════════════════════════════════════════════════
// tokmd-envelope enums — rename_all = "lowercase"
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn verdict_all_variants_lowercase() {
    let variants = [
        (Verdict::Pass, "pass"),
        (Verdict::Fail, "fail"),
        (Verdict::Warn, "warn"),
        (Verdict::Skip, "skip"),
        (Verdict::Pending, "pending"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, format!("\"{}\"", expected), "Verdict::{variant:?}");
        let back: Verdict = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn verdict_default_is_pass() {
    let v = Verdict::default();
    assert_eq!(v, Verdict::Pass);
    assert_eq!(serde_json::to_string(&v).unwrap(), "\"pass\"");
}

#[test]
fn finding_severity_all_variants_lowercase() {
    let variants = [
        (FindingSeverity::Error, "error"),
        (FindingSeverity::Warn, "warn"),
        (FindingSeverity::Info, "info"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "FindingSeverity::{variant:?}"
        );
        let back: FindingSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn envelope_capability_state_all_variants_lowercase() {
    let variants = [
        (EnvCapState::Available, "available"),
        (EnvCapState::Unavailable, "unavailable"),
        (EnvCapState::Skipped, "skipped"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "CapabilityState::{variant:?}"
        );
        let back: EnvCapState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Artifact — #[serde(rename = "type")] for artifact_type field
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn artifact_type_field_serializes_as_type() {
    let artifact = Artifact {
        id: Some("analysis".into()),
        artifact_type: "receipt".into(),
        path: "output/receipt.json".into(),
        mime: Some("application/json".into()),
    };
    let json = serde_json::to_value(artifact).unwrap();
    // Field must be serialized as "type", not "artifact_type"
    assert!(json.get("type").is_some(), "must serialize as 'type'");
    assert!(
        json.get("artifact_type").is_none(),
        "must NOT serialize as 'artifact_type'"
    );
    assert_eq!(json["type"], "receipt");
}

#[test]
fn artifact_type_field_deserializes_from_type_key() {
    let json = json!({
        "id": "badge",
        "type": "badge",
        "path": "output/badge.svg",
        "mime": "image/svg+xml"
    });
    let artifact: Artifact = serde_json::from_value(json).unwrap();
    assert_eq!(artifact.artifact_type, "badge");
    assert_eq!(artifact.path, "output/badge.svg");
}

#[test]
fn artifact_roundtrip_preserves_type_rename() {
    let original = Artifact {
        id: None,
        artifact_type: "comment".into(),
        path: "pr-comment.md".into(),
        mime: None,
    };
    let json = serde_json::to_value(original).unwrap();
    let back: Artifact = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(back.artifact_type, "comment");
    // Verify the JSON key is "type"
    assert_eq!(json["type"], "comment");
}

// ═══════════════════════════════════════════════════════════════════════════
// SensorReport round-trip with verdict + findings
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn sensor_report_roundtrip_with_verdict_and_severity() {
    let json = json!({
        "schema": "sensor.report.v1",
        "tool": {
            "name": "tokmd",
            "version": "1.5.0",
            "mode": "cockpit"
        },
        "generated_at": "2025-01-01T00:00:00Z",
        "verdict": "warn",
        "summary": "2 warnings found",
        "findings": [{
            "check_id": "risk",
            "code": "hotspot",
            "severity": "warn",
            "title": "Hotspot detected",
            "message": "src/parser.rs has high churn"
        }],
        "artifacts": [{
            "type": "receipt",
            "path": "output/receipt.json"
        }]
    });
    let report: SensorReport = serde_json::from_value(json).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
    let arts = report.artifacts.as_ref().unwrap();
    assert_eq!(arts[0].artifact_type, "receipt");

    // Re-serialize and verify keys
    let reserialized: Value = serde_json::to_value(report).unwrap();
    assert_eq!(reserialized["verdict"], "warn");
    assert_eq!(reserialized["findings"][0]["severity"], "warn");
    // Artifact uses "type" not "artifact_type"
    assert!(reserialized["artifacts"][0].get("type").is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// Re-export alias compatibility (analysis-types re-exports envelope types)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn envelope_re_export_aliases_are_same_type() {
    // Verify that the re-exported type aliases resolve correctly
    let report = Envelope {
        schema: ENVELOPE_SCHEMA.to_string(),
        tool: EnvelopeTool {
            name: "tokmd".into(),
            version: "1.0.0".into(),
            mode: "analyze".into(),
        },
        generated_at: "2025-01-01T00:00:00Z".into(),
        verdict: Verdict::Pass,
        summary: "ok".into(),
        findings: vec![],
        artifacts: None,
        capabilities: None,
        data: None,
    };
    let json = serde_json::to_value(report).unwrap();
    assert_eq!(json["verdict"], "pass");
    assert_eq!(json["schema"], "sensor.report.v1");

    // Deserialize as canonical name
    let back: SensorReport = serde_json::from_value(json).unwrap();
    assert_eq!(back.verdict, Verdict::Pass);
}
