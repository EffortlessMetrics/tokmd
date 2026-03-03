//! Expanded contract tests for tokmd-envelope.
//!
//! Covers edge-case fingerprints, Unicode fields, all-optional-sections
//! populated simultaneously, and Finding with every optional field None.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};

// =============================================================================
// Helpers
// =============================================================================

fn sample_tool() -> ToolMeta {
    ToolMeta {
        name: "tokmd".to_string(),
        version: "0.1.0-test".to_string(),
        mode: "cockpit".to_string(),
    }
}

// =============================================================================
// Scenario: SensorReport with ALL optional sections populated simultaneously
// =============================================================================

#[test]
fn given_report_with_all_optional_sections_when_roundtripped_then_preserved() {
    let mut caps = BTreeMap::new();
    caps.insert("mutation".to_string(), CapabilityStatus::available());
    caps.insert(
        "coverage".to_string(),
        CapabilityStatus::unavailable("no CI artifact"),
    );
    caps.insert(
        "lint".to_string(),
        CapabilityStatus::skipped("no relevant files"),
    );

    let report = SensorReport {
        schema: SENSOR_REPORT_SCHEMA.to_string(),
        tool: sample_tool(),
        generated_at: "2024-06-15T12:30:45Z".to_string(),
        verdict: Verdict::Warn,
        summary: "3 warnings found".to_string(),
        findings: vec![Finding {
            check_id: "risk".to_string(),
            code: "hotspot".to_string(),
            severity: FindingSeverity::Warn,
            title: "Hotspot detected".to_string(),
            message: "src/core.rs has 42 commits".to_string(),
            location: Some(FindingLocation::path("src/core.rs")),
            evidence: Some(serde_json::json!({"commits": 42})),
            docs_url: Some("https://example.com/docs/hotspot".to_string()),
            fingerprint: None,
        }],
        artifacts: Some(vec![
            Artifact {
                id: Some("comment".to_string()),
                artifact_type: "comment".to_string(),
                path: "out/comment.md".to_string(),
                mime: Some("text/markdown".to_string()),
            },
            Artifact {
                id: None,
                artifact_type: "badge".to_string(),
                path: "out/badge.svg".to_string(),
                mime: Some("image/svg+xml".to_string()),
            },
        ]),
        capabilities: Some(caps),
        data: Some(serde_json::json!({"custom_key": [1, 2, 3]})),
    };

    let json = serde_json::to_string_pretty(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();

    assert_eq!(back.verdict, Verdict::Warn);
    assert_eq!(back.findings.len(), 1);
    assert_eq!(back.findings[0].check_id, "risk");
    assert_eq!(
        back.findings[0].evidence,
        Some(serde_json::json!({"commits": 42}))
    );

    let arts = back.artifacts.as_ref().unwrap();
    assert_eq!(arts.len(), 2);
    assert_eq!(arts[0].artifact_type, "comment");
    assert_eq!(arts[1].id, None);

    let caps = back.capabilities.as_ref().unwrap();
    assert_eq!(caps.len(), 3);
    assert_eq!(caps["mutation"].status, CapabilityState::Available);
    assert_eq!(caps["coverage"].status, CapabilityState::Unavailable);
    assert_eq!(caps["lint"].status, CapabilityState::Skipped);

    assert!(back.data.is_some());
}

// =============================================================================
// Scenario: Finding with all optional fields None
// =============================================================================

#[test]
fn given_finding_with_all_optional_none_when_roundtripped_then_preserved() {
    let finding = Finding {
        check_id: "contract".to_string(),
        code: "schema".to_string(),
        severity: FindingSeverity::Info,
        title: "Schema check".to_string(),
        message: "All schemas valid".to_string(),
        location: None,
        evidence: None,
        docs_url: None,
        fingerprint: None,
    };

    let json = serde_json::to_string(&finding).unwrap();

    // None fields should be omitted (skip_serializing_if)
    assert!(!json.contains("\"location\""));
    assert!(!json.contains("\"evidence\""));
    assert!(!json.contains("\"docs_url\""));
    assert!(!json.contains("\"fingerprint\""));

    let back: Finding = serde_json::from_str(&json).unwrap();
    assert!(back.location.is_none());
    assert!(back.evidence.is_none());
    assert!(back.docs_url.is_none());
    assert!(back.fingerprint.is_none());
}

// =============================================================================
// Scenario: Fingerprint edge cases (empty strings, Unicode)
// =============================================================================

#[test]
fn given_finding_with_empty_location_path_when_fingerprinted_then_deterministic() {
    let f1 = Finding {
        check_id: "check".to_string(),
        code: "code".to_string(),
        severity: FindingSeverity::Info,
        title: "title".to_string(),
        message: "msg".to_string(),
        location: Some(FindingLocation::path("")),
        evidence: None,
        docs_url: None,
        fingerprint: None,
    }
    .with_fingerprint("test-tool");

    let f2 = Finding {
        check_id: "check".to_string(),
        code: "code".to_string(),
        severity: FindingSeverity::Info,
        title: "title".to_string(),
        message: "different message".to_string(),
        location: Some(FindingLocation::path("")),
        evidence: None,
        docs_url: None,
        fingerprint: None,
    }
    .with_fingerprint("test-tool");

    // Same inputs => same fingerprint (message not included in hash)
    assert_eq!(f1.fingerprint, f2.fingerprint);
    assert!(f1.fingerprint.is_some());
}

#[test]
fn given_finding_with_unicode_path_when_fingerprinted_then_stable() {
    let f1 = Finding {
        check_id: "risk".to_string(),
        code: "hotspot".to_string(),
        severity: FindingSeverity::Warn,
        title: "ホットスポット".to_string(),
        message: "日本語のメッセージ".to_string(),
        location: Some(FindingLocation::path("src/日本語.rs")),
        evidence: None,
        docs_url: None,
        fingerprint: None,
    }
    .with_fingerprint("tokmd");

    // Same construction => same fingerprint
    let f2 = Finding {
        check_id: "risk".to_string(),
        code: "hotspot".to_string(),
        severity: FindingSeverity::Error,
        title: "Different title".to_string(),
        message: "Different msg".to_string(),
        location: Some(FindingLocation::path("src/日本語.rs")),
        evidence: None,
        docs_url: None,
        fingerprint: None,
    }
    .with_fingerprint("tokmd");

    assert_eq!(f1.fingerprint, f2.fingerprint);
}

#[test]
fn given_finding_without_location_when_fingerprinted_then_uses_empty_path() {
    let f1 = Finding {
        check_id: "gate".to_string(),
        code: "threshold".to_string(),
        severity: FindingSeverity::Error,
        title: "Gate failed".to_string(),
        message: "Below threshold".to_string(),
        location: None,
        evidence: None,
        docs_url: None,
        fingerprint: None,
    }
    .with_fingerprint("tokmd");

    assert!(f1.fingerprint.is_some());

    // Round-trip
    let json = serde_json::to_string(&f1).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();
    assert_eq!(back.fingerprint, f1.fingerprint);
}

// =============================================================================
// Scenario: GateItem with all optional fields populated
// =============================================================================

#[test]
fn given_gate_item_with_all_fields_when_roundtripped_then_preserved() {
    let item = GateItem {
        id: "mutation".to_string(),
        status: Verdict::Fail,
        threshold: Some(80.0),
        actual: Some(72.5),
        reason: Some("Mutation score 72.5% < 80%".to_string()),
        source: Some("ci_artifact".to_string()),
        artifact_path: Some("out/mutants.json".to_string()),
    };

    let json = serde_json::to_string(&item).unwrap();
    let back: GateItem = serde_json::from_str(&json).unwrap();

    assert_eq!(back.id, "mutation");
    assert_eq!(back.status, Verdict::Fail);
    assert_eq!(back.threshold, Some(80.0));
    assert_eq!(back.actual, Some(72.5));
    assert_eq!(back.reason.as_deref(), Some("Mutation score 72.5% < 80%"));
    assert_eq!(back.source.as_deref(), Some("ci_artifact"));
    assert_eq!(back.artifact_path.as_deref(), Some("out/mutants.json"));
}

#[test]
fn given_gate_item_with_all_optional_none_when_serialized_then_omitted() {
    let item = GateItem {
        id: "coverage".to_string(),
        status: Verdict::Pending,
        threshold: None,
        actual: None,
        reason: None,
        source: None,
        artifact_path: None,
    };

    let json = serde_json::to_string(&item).unwrap();

    assert!(!json.contains("\"threshold\""));
    assert!(!json.contains("\"actual\""));
    assert!(!json.contains("\"reason\""));
    assert!(!json.contains("\"source\""));
    assert!(!json.contains("\"artifact_path\""));
}

// =============================================================================
// Scenario: GateResults roundtrip
// =============================================================================

#[test]
fn given_gate_results_with_mixed_verdicts_when_roundtripped_then_preserved() {
    let gates = GateResults {
        status: Verdict::Fail,
        items: vec![
            GateItem {
                id: "mutation".to_string(),
                status: Verdict::Fail,
                threshold: Some(80.0),
                actual: Some(72.5),
                reason: Some("Below threshold".to_string()),
                source: None,
                artifact_path: None,
            },
            GateItem {
                id: "coverage".to_string(),
                status: Verdict::Pass,
                threshold: Some(60.0),
                actual: Some(85.0),
                reason: None,
                source: None,
                artifact_path: None,
            },
            GateItem {
                id: "lint".to_string(),
                status: Verdict::Skip,
                threshold: None,
                actual: None,
                reason: Some("No lint config".to_string()),
                source: None,
                artifact_path: None,
            },
        ],
    };

    let json = serde_json::to_string(&gates).unwrap();
    let back: GateResults = serde_json::from_str(&json).unwrap();

    assert_eq!(back.status, Verdict::Fail);
    assert_eq!(back.items.len(), 3);
    assert_eq!(back.items[0].status, Verdict::Fail);
    assert_eq!(back.items[1].status, Verdict::Pass);
    assert_eq!(back.items[2].status, Verdict::Skip);
}

// =============================================================================
// Scenario: Verdict Display matches serde
// =============================================================================

#[test]
fn given_all_verdicts_when_display_then_lowercase_matches_serde() {
    for (verdict, expected) in [
        (Verdict::Pass, "pass"),
        (Verdict::Fail, "fail"),
        (Verdict::Warn, "warn"),
        (Verdict::Skip, "skip"),
        (Verdict::Pending, "pending"),
    ] {
        let json = serde_json::to_value(verdict).unwrap();
        assert_eq!(json.as_str().unwrap(), expected);
        assert_eq!(format!("{verdict}"), expected);
    }
}

// =============================================================================
// Scenario: Schema constant is stable
// =============================================================================

#[test]
fn given_schema_constant_then_value_is_sensor_report_v1() {
    assert_eq!(SENSOR_REPORT_SCHEMA, "sensor.report.v1");
}

// =============================================================================
// Property: SensorReport roundtrip with arbitrary verdicts and severities
// =============================================================================

proptest! {
    #[test]
    fn prop_report_roundtrip_with_varying_finding_count(
        n_findings in 0usize..20,
        verdict_idx in 0usize..5,
    ) {
        let verdicts = [Verdict::Pass, Verdict::Fail, Verdict::Warn, Verdict::Skip, Verdict::Pending];
        let severities = [FindingSeverity::Error, FindingSeverity::Warn, FindingSeverity::Info];

        let findings: Vec<Finding> = (0..n_findings)
            .map(|i| Finding {
                check_id: format!("check-{i}"),
                code: format!("code-{i}"),
                severity: severities[i % 3],
                title: format!("Finding #{i}"),
                message: format!("Message for finding {i}"),
                location: if i % 2 == 0 {
                    Some(FindingLocation::path(format!("src/file{i}.rs")))
                } else {
                    None
                },
                evidence: None,
                docs_url: None,
                fingerprint: None,
            })
            .collect();

        let report = SensorReport {
            schema: SENSOR_REPORT_SCHEMA.to_string(),
            tool: sample_tool(),
            generated_at: "2024-01-01T00:00:00Z".to_string(),
            verdict: verdicts[verdict_idx],
            summary: format!("{n_findings} findings"),
            findings,
            artifacts: None,
            capabilities: None,
            data: None,
        };

        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(back.findings.len(), n_findings);
        prop_assert_eq!(back.verdict, verdicts[verdict_idx]);
    }
}
