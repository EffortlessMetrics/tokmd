//! W53: Extended property-based tests for `tokmd-envelope`.
//!
//! Covers: full SensorReport roundtrip stability, schema version non-zero,
//! finding ID uniqueness, builder chain invariants, and boundary conditions.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};

// ── Strategies ──────────────────────────────────────────────────────────

fn arb_verdict() -> impl Strategy<Value = Verdict> {
    prop_oneof![
        Just(Verdict::Pass),
        Just(Verdict::Fail),
        Just(Verdict::Warn),
        Just(Verdict::Skip),
        Just(Verdict::Pending),
    ]
}

fn arb_severity() -> impl Strategy<Value = FindingSeverity> {
    prop_oneof![
        Just(FindingSeverity::Error),
        Just(FindingSeverity::Warn),
        Just(FindingSeverity::Info),
    ]
}

fn arb_tool_meta() -> impl Strategy<Value = ToolMeta> {
    (
        "[a-z_]{1,20}",
        "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
        "[a-z_]{1,15}",
    )
        .prop_map(|(name, version, mode)| ToolMeta::new(&name, &version, &mode))
}

fn arb_finding() -> impl Strategy<Value = Finding> {
    (
        "[a-z_]{1,20}",
        "[a-z_]{1,20}",
        arb_severity(),
        "[A-Za-z0-9 ]{1,40}",
        "[A-Za-z0-9 ]{1,100}",
    )
        .prop_map(|(check_id, code, severity, title, message)| {
            Finding::new(check_id, code, severity, title, message)
        })
}

fn arb_artifact() -> impl Strategy<Value = Artifact> {
    ("[a-z_]{1,15}", "[a-z/._]{1,40}").prop_map(|(atype, path)| Artifact::new(atype, path))
}

fn arb_gate_item() -> impl Strategy<Value = GateItem> {
    ("[a-z_]{1,20}", arb_verdict()).prop_map(|(id, status)| GateItem::new(id, status))
}

fn arb_capability_status() -> impl Strategy<Value = CapabilityStatus> {
    (
        prop_oneof![
            Just(CapabilityState::Available),
            Just(CapabilityState::Unavailable),
            Just(CapabilityState::Skipped),
        ],
        proptest::option::of("[A-Za-z0-9 ]{1,50}"),
    )
        .prop_map(|(state, reason)| {
            let mut cs = CapabilityStatus::new(state);
            cs.reason = reason;
            cs
        })
}

fn full_report(
    meta: ToolMeta,
    verdict: Verdict,
    summary: String,
    findings: Vec<Finding>,
    artifacts: Vec<Artifact>,
    caps: BTreeMap<String, CapabilityStatus>,
) -> SensorReport {
    let mut report = SensorReport::new(meta, "2025-01-01T00:00:00Z".into(), verdict, summary);
    for f in findings {
        report.add_finding(f);
    }
    report.with_artifacts(artifacts).with_capabilities(caps)
}

// ── Tests ───────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(120))]

    // 1. Full report serialization roundtrip (all optional fields populated)
    #[test]
    fn full_report_serde_roundtrip(
        meta in arb_tool_meta(),
        verdict in arb_verdict(),
        summary in "[A-Za-z0-9 ]{1,80}",
        findings in proptest::collection::vec(arb_finding(), 0..6),
        artifacts in proptest::collection::vec(arb_artifact(), 0..4),
        caps in proptest::collection::btree_map("[a-z]{1,10}", arb_capability_status(), 0..4),
    ) {
        let report = full_report(meta, verdict, summary.clone(), findings.clone(), artifacts.clone(), caps.clone());
        let json = serde_json::to_string_pretty(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(SENSOR_REPORT_SCHEMA, back.schema.as_str());
        prop_assert_eq!(verdict, back.verdict);
        prop_assert_eq!(&summary, &back.summary);
        prop_assert_eq!(findings.len(), back.findings.len());
        prop_assert_eq!(artifacts.len(), back.artifacts.unwrap_or_default().len());
        prop_assert_eq!(caps.len(), back.capabilities.unwrap_or_default().len());
    }

    // 2. Schema string is always the expected constant
    #[test]
    fn schema_always_sensor_report_v1(
        meta in arb_tool_meta(),
        verdict in arb_verdict(),
    ) {
        let report = SensorReport::new(meta, "t".into(), verdict, "s".into());
        prop_assert_eq!(report.schema.as_str(), SENSOR_REPORT_SCHEMA);
        prop_assert!(!report.schema.is_empty());
    }

    // 3. Finding IDs with distinct (check_id, code) are unique
    #[test]
    fn finding_ids_unique_for_distinct_pairs(
        tool in "[a-z]{1,10}",
        pairs in proptest::collection::vec(
            ("[a-z]{1,8}", "[a-z]{1,8}"),
            2..6,
        ),
    ) {
        let ids: Vec<String> = pairs
            .iter()
            .map(|(cid, code)| tokmd_envelope::findings::finding_id(&tool, cid, code))
            .collect();
        // Deduplicate: unique pairs should produce unique IDs
        let unique_pairs: std::collections::HashSet<(&str, &str)> =
            pairs.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let unique_ids: std::collections::HashSet<&str> =
            ids.iter().map(|s| s.as_str()).collect();
        prop_assert_eq!(unique_pairs.len(), unique_ids.len());
    }

    // 4. Serialization is deterministic (two calls → identical output)
    #[test]
    fn serialization_deterministic(
        meta in arb_tool_meta(),
        verdict in arb_verdict(),
        summary in "[A-Za-z0-9 ]{1,40}",
        findings in proptest::collection::vec(arb_finding(), 0..4),
    ) {
        let mut report = SensorReport::new(meta, "t".into(), verdict, summary);
        for f in findings {
            report.add_finding(f);
        }
        let a = serde_json::to_string(&report).unwrap();
        let b = serde_json::to_string(&report).unwrap();
        prop_assert_eq!(a, b);
    }

    // 5. Empty findings vec survives roundtrip as empty
    #[test]
    fn empty_findings_roundtrip(meta in arb_tool_meta(), verdict in arb_verdict()) {
        let report = SensorReport::new(meta, "t".into(), verdict, "s".into());
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert!(back.findings.is_empty());
    }

    // 6. Fingerprint with_fingerprint builder sets the field
    #[test]
    fn with_fingerprint_builder_sets_field(
        check_id in "[a-z]{1,10}",
        code in "[a-z]{1,10}",
        path in "[a-z/]{1,30}",
    ) {
        let f = Finding::new(&check_id, &code, FindingSeverity::Warn, "T", "M")
            .with_location(FindingLocation::path(&path))
            .with_fingerprint("tokmd");
        prop_assert!(f.fingerprint.is_some());
        let fp = f.fingerprint.unwrap();
        prop_assert_eq!(fp.len(), 32);
        prop_assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // 7. GateResults with many items roundtrips
    #[test]
    fn gate_results_many_items(
        status in arb_verdict(),
        items in proptest::collection::vec(arb_gate_item(), 0..10),
    ) {
        let gates = GateResults::new(status, items.clone());
        let json = serde_json::to_string(&gates).unwrap();
        let back: GateResults = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(status, back.status);
        prop_assert_eq!(items.len(), back.items.len());
    }

    // 8. Capability ordering is deterministic (BTreeMap)
    #[test]
    fn capability_ordering_deterministic(
        caps in proptest::collection::btree_map("[a-z]{1,8}", arb_capability_status(), 2..6),
    ) {
        let report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "test"),
            "t".into(),
            Verdict::Pass,
            "s".into(),
        )
        .with_capabilities(caps.clone());

        let json1 = serde_json::to_string(&report).unwrap();
        let json2 = serde_json::to_string(&report).unwrap();
        prop_assert_eq!(&json1, &json2);

        let back: SensorReport = serde_json::from_str(&json1).unwrap();
        let back_keys: Vec<_> = back.capabilities.unwrap().keys().cloned().collect();
        let orig_keys: Vec<_> = caps.keys().cloned().collect();
        prop_assert_eq!(orig_keys, back_keys);
    }

    // 9. add_capability accumulates
    #[test]
    fn add_capability_accumulates(
        names in proptest::collection::vec("[a-z]{1,8}", 1..8),
    ) {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "test"),
            "t".into(),
            Verdict::Pass,
            "s".into(),
        );
        for name in &names {
            report.add_capability(name.clone(), CapabilityStatus::available());
        }
        let unique: std::collections::HashSet<&str> =
            names.iter().map(|s| s.as_str()).collect();
        let caps = report.capabilities.unwrap();
        prop_assert_eq!(unique.len(), caps.len());
    }

    // 10. Artifact with_id and with_mime survive roundtrip
    #[test]
    fn artifact_optional_fields_roundtrip(
        atype in "[a-z_]{1,10}",
        path in "[a-z/._]{1,30}",
        id in "[a-z_]{1,10}",
        mime in "[a-z/]{3,20}",
    ) {
        let a = Artifact::new(&atype, &path).with_id(&id).with_mime(&mime);
        let json = serde_json::to_string(&a).unwrap();
        let back: Artifact = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.id.as_deref(), Some(id.as_str()));
        prop_assert_eq!(back.mime.as_deref(), Some(mime.as_str()));
    }

    // 11. with_data preserves opaque JSON payload
    #[test]
    fn with_data_preserves_payload(
        key in "[a-z]{1,10}",
        val in 0i64..100_000,
    ) {
        let data = serde_json::json!({ key.clone(): val });
        let report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "test"),
            "t".into(),
            Verdict::Pass,
            "s".into(),
        )
        .with_data(data.clone());
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.data.unwrap(), data);
    }
}
