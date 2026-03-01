//! Property-based tests for the `tokmd-envelope` crate using `proptest`.
//!
//! These tests verify that serialization round-trips preserve data
//! and that builder patterns produce consistent results for arbitrary inputs.

use proptest::prelude::*;
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

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

fn arb_capability_state() -> impl Strategy<Value = CapabilityState> {
    prop_oneof![
        Just(CapabilityState::Available),
        Just(CapabilityState::Unavailable),
        Just(CapabilityState::Skipped),
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

fn arb_finding_location() -> impl Strategy<Value = FindingLocation> {
    ("[a-z/._]{1,50}", any::<Option<u32>>(), any::<Option<u32>>())
        .prop_map(|(path, line, column)| FindingLocation { path, line, column })
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
        arb_capability_state(),
        proptest::option::of("[A-Za-z0-9 ]{1,50}"),
    )
        .prop_map(|(state, reason)| {
            let mut cs = CapabilityStatus::new(state);
            cs.reason = reason;
            cs
        })
}

// ---------------------------------------------------------------------------
// Round-trip properties
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_verdict_serde_roundtrip(v in arb_verdict()) {
        let json = serde_json::to_string(&v).unwrap();
        let back: Verdict = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(v, back);
    }

    #[test]
    fn prop_severity_serde_roundtrip(s in arb_severity()) {
        let json = serde_json::to_string(&s).unwrap();
        let back: FindingSeverity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, back);
    }

    #[test]
    fn prop_capability_state_serde_roundtrip(s in arb_capability_state()) {
        let json = serde_json::to_string(&s).unwrap();
        let back: CapabilityState = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, back);
    }

    #[test]
    fn prop_tool_meta_serde_roundtrip(meta in arb_tool_meta()) {
        let json = serde_json::to_string(&meta).unwrap();
        let back: ToolMeta = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(meta.name, back.name);
        prop_assert_eq!(meta.version, back.version);
        prop_assert_eq!(meta.mode, back.mode);
    }

    #[test]
    fn prop_finding_location_serde_roundtrip(loc in arb_finding_location()) {
        let json = serde_json::to_string(&loc).unwrap();
        let back: FindingLocation = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(loc.path, back.path);
        prop_assert_eq!(loc.line, back.line);
        prop_assert_eq!(loc.column, back.column);
    }

    #[test]
    fn prop_finding_serde_roundtrip(f in arb_finding()) {
        let json = serde_json::to_string(&f).unwrap();
        let back: Finding = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(f.check_id, back.check_id);
        prop_assert_eq!(f.code, back.code);
        prop_assert_eq!(f.severity, back.severity);
        prop_assert_eq!(f.title, back.title);
        prop_assert_eq!(f.message, back.message);
    }

    #[test]
    fn prop_artifact_serde_roundtrip(a in arb_artifact()) {
        let json = serde_json::to_string(&a).unwrap();
        let back: Artifact = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(a.artifact_type, back.artifact_type);
        prop_assert_eq!(a.path, back.path);
    }

    #[test]
    fn prop_gate_item_serde_roundtrip(g in arb_gate_item()) {
        let json = serde_json::to_string(&g).unwrap();
        let back: GateItem = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(g.id, back.id);
        prop_assert_eq!(g.status, back.status);
    }

    #[test]
    fn prop_capability_status_serde_roundtrip(cs in arb_capability_status()) {
        let json = serde_json::to_string(&cs).unwrap();
        let back: CapabilityStatus = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(cs.status, back.status);
        prop_assert_eq!(cs.reason, back.reason);
    }

    #[test]
    fn prop_sensor_report_serde_roundtrip(
        meta in arb_tool_meta(),
        verdict in arb_verdict(),
        summary in "[A-Za-z0-9 ]{1,80}",
        findings in proptest::collection::vec(arb_finding(), 0..5),
    ) {
        let mut report = SensorReport::new(
            meta,
            "2025-01-01T00:00:00Z".into(),
            verdict,
            summary.clone(),
        );
        for f in &findings {
            report.add_finding(f.clone());
        }

        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(SENSOR_REPORT_SCHEMA, back.schema.as_str());
        prop_assert_eq!(verdict, back.verdict);
        prop_assert_eq!(&summary, &back.summary);
        prop_assert_eq!(findings.len(), back.findings.len());
    }

    #[test]
    fn prop_fingerprint_deterministic(
        tool in "[a-z]{1,10}",
        check_id in "[a-z]{1,10}",
        code in "[a-z]{1,10}",
        path in "[a-z/]{1,30}",
    ) {
        let f = Finding::new(&check_id, &code, FindingSeverity::Info, "T", "M")
            .with_location(FindingLocation::path(&path));
        let fp1 = f.compute_fingerprint(&tool);
        let fp2 = f.compute_fingerprint(&tool);
        prop_assert_eq!(&fp1, &fp2);
        prop_assert_eq!(fp1.len(), 32);
        // All hex chars
        prop_assert!(fp1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn prop_fingerprint_length_always_32(
        tool in ".*",
        check_id in ".*",
        code in ".*",
    ) {
        let f = Finding::new(&check_id, &code, FindingSeverity::Info, "T", "M");
        let fp = f.compute_fingerprint(&tool);
        prop_assert_eq!(fp.len(), 32);
    }

    #[test]
    fn prop_verdict_display_matches_serde(v in arb_verdict()) {
        let display = v.to_string();
        let json_val = serde_json::to_value(v).unwrap();
        prop_assert_eq!(display, json_val.as_str().unwrap().to_string());
    }

    #[test]
    fn prop_severity_display_matches_serde(s in arb_severity()) {
        let display = s.to_string();
        let json_val = serde_json::to_value(s).unwrap();
        prop_assert_eq!(display, json_val.as_str().unwrap().to_string());
    }

    #[test]
    fn prop_gate_results_roundtrip(
        status in arb_verdict(),
        items in proptest::collection::vec(arb_gate_item(), 0..5),
    ) {
        let gates = GateResults::new(status, items.clone());
        let json = serde_json::to_string(&gates).unwrap();
        let back: GateResults = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(status, back.status);
        prop_assert_eq!(items.len(), back.items.len());
    }

    #[test]
    fn prop_report_with_capabilities_roundtrip(
        meta in arb_tool_meta(),
        caps in proptest::collection::btree_map("[a-z]{1,10}", arb_capability_status(), 0..5),
    ) {
        let report = SensorReport::new(
            meta,
            "2025-01-01T00:00:00Z".into(),
            Verdict::Pass,
            "test".into(),
        )
        .with_capabilities(caps.clone());

        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();

        let back_caps = back.capabilities.unwrap_or_default();
        prop_assert_eq!(caps.len(), back_caps.len());
        for (key, val) in &caps {
            let back_val = back_caps.get(key).unwrap();
            prop_assert_eq!(val.status, back_val.status);
            prop_assert_eq!(&val.reason, &back_val.reason);
        }
    }

    // ---------------------------------------------------------------------------
    // Invariant: schema field is always SENSOR_REPORT_SCHEMA
    // ---------------------------------------------------------------------------

    #[test]
    fn prop_sensor_report_schema_always_set(
        meta in arb_tool_meta(),
        verdict in arb_verdict(),
        summary in "[A-Za-z0-9 ]{1,80}",
    ) {
        let report = SensorReport::new(
            meta,
            "2025-01-01T00:00:00Z".into(),
            verdict,
            summary,
        );
        prop_assert_eq!(report.schema.as_str(), SENSOR_REPORT_SCHEMA);

        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.schema.as_str(), SENSOR_REPORT_SCHEMA);
    }

    // ---------------------------------------------------------------------------
    // Invariant: findings count preserved through roundtrip
    // ---------------------------------------------------------------------------

    #[test]
    fn prop_findings_count_preserved(
        meta in arb_tool_meta(),
        findings in proptest::collection::vec(arb_finding(), 0..10),
    ) {
        let mut report = SensorReport::new(
            meta,
            "2025-01-01T00:00:00Z".into(),
            Verdict::Warn,
            "test".into(),
        );
        for f in &findings {
            report.add_finding(f.clone());
        }
        prop_assert_eq!(report.findings.len(), findings.len());

        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.findings.len(), findings.len());
    }

    // ---------------------------------------------------------------------------
    // Invariant: serialization is deterministic
    // ---------------------------------------------------------------------------

    #[test]
    fn prop_sensor_report_serialization_deterministic(
        meta in arb_tool_meta(),
        verdict in arb_verdict(),
        summary in "[A-Za-z0-9 ]{1,80}",
    ) {
        let report = SensorReport::new(
            meta,
            "2025-01-01T00:00:00Z".into(),
            verdict,
            summary,
        );
        let json1 = serde_json::to_string(&report).unwrap();
        let json2 = serde_json::to_string(&report).unwrap();
        prop_assert_eq!(json1, json2);
    }

    // ---------------------------------------------------------------------------
    // GateItem with optional fields roundtrip
    // ---------------------------------------------------------------------------

    #[test]
    fn prop_gate_item_with_threshold_roundtrip(
        id in "[a-z_]{1,20}",
        status in arb_verdict(),
        threshold in 0.0f64..100.0,
        actual in 0.0f64..100.0,
        reason in "[A-Za-z0-9 ]{1,40}",
    ) {
        let gate = GateItem::new(id, status)
            .with_threshold(threshold, actual)
            .with_reason(reason);
        let json = serde_json::to_string(&gate).unwrap();
        let back: GateItem = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&gate.id, &back.id);
        prop_assert_eq!(gate.status, back.status);
        prop_assert!(back.threshold.is_some());
        prop_assert!(back.actual.is_some());
        prop_assert!(back.reason.is_some());
    }

    // ---------------------------------------------------------------------------
    // Artifact with all optional fields roundtrip
    // ---------------------------------------------------------------------------

    #[test]
    fn prop_artifact_with_all_fields_roundtrip(
        atype in "[a-z_]{1,15}",
        path in "[a-z/._]{1,40}",
        id in "[a-z_]{1,15}",
        mime in "[a-z/]{3,20}",
    ) {
        let artifact = Artifact::new(atype, path)
            .with_id(id.clone())
            .with_mime(mime.clone());
        let json = serde_json::to_string(&artifact).unwrap();
        let back: Artifact = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&artifact.artifact_type, &back.artifact_type);
        prop_assert_eq!(&artifact.path, &back.path);
        prop_assert_eq!(back.id.as_deref(), Some(id.as_str()));
        prop_assert_eq!(back.mime.as_deref(), Some(mime.as_str()));
    }

    // ---------------------------------------------------------------------------
    // finding_id determinism and format
    // ---------------------------------------------------------------------------

    #[test]
    fn prop_finding_id_deterministic(
        tool in "[a-z]{1,10}",
        check_id in "[a-z]{1,10}",
        code in "[a-z]{1,10}",
    ) {
        let id1 = tokmd_envelope::findings::finding_id(&tool, &check_id, &code);
        let id2 = tokmd_envelope::findings::finding_id(&tool, &check_id, &code);
        prop_assert_eq!(&id1, &id2);
    }

    #[test]
    fn prop_finding_id_has_two_dots(
        tool in "[a-z]{1,10}",
        check_id in "[a-z]{1,10}",
        code in "[a-z]{1,10}",
    ) {
        let id = tokmd_envelope::findings::finding_id(&tool, &check_id, &code);
        let dot_count = id.chars().filter(|c| *c == '.').count();
        prop_assert_eq!(dot_count, 2, "finding_id should have exactly 2 dots: {}", id);
    }

    // ---------------------------------------------------------------------------
    // Fingerprint: different (check_id, code) pairs produce different fingerprints
    // ---------------------------------------------------------------------------

    #[test]
    fn prop_fingerprint_differs_for_different_codes(
        tool in "[a-z]{1,10}",
        check_id in "[a-z]{1,10}",
        code_a in "[a-z]{1,10}",
        code_b in "[a-z]{1,10}",
    ) {
        prop_assume!(code_a != code_b);
        let fa = Finding::new(&check_id, &code_a, FindingSeverity::Info, "T", "M");
        let fb = Finding::new(&check_id, &code_b, FindingSeverity::Info, "T", "M");
        prop_assert_ne!(fa.compute_fingerprint(&tool), fb.compute_fingerprint(&tool));
    }

    // ---------------------------------------------------------------------------
    // Finding with location roundtrip
    // ---------------------------------------------------------------------------

    #[test]
    fn prop_finding_with_location_roundtrip(
        f in arb_finding(),
        loc in arb_finding_location(),
    ) {
        let finding = f.with_location(loc);
        let json = serde_json::to_string(&finding).unwrap();
        let back: Finding = serde_json::from_str(&json).unwrap();
        prop_assert!(back.location.is_some());
        let back_loc = back.location.unwrap();
        prop_assert_eq!(&finding.location.as_ref().unwrap().path, &back_loc.path);
        prop_assert_eq!(finding.location.as_ref().unwrap().line, back_loc.line);
        prop_assert_eq!(finding.location.as_ref().unwrap().column, back_loc.column);
    }
}
