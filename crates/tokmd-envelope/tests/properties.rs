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

    // -----------------------------------------------------------------------
    // Full SensorReport round-trip (all optional sections populated)
    // -----------------------------------------------------------------------

    #[test]
    fn prop_sensor_report_full_roundtrip(
        meta in arb_tool_meta(),
        verdict in arb_verdict(),
        summary in "[A-Za-z0-9 ]{1,80}",
        findings in proptest::collection::vec(arb_finding(), 0..5),
        artifacts in proptest::collection::vec(arb_artifact(), 0..4),
        caps in proptest::collection::btree_map("[a-z]{1,10}", arb_capability_status(), 0..4),
        data_key in "[a-z]{1,10}",
        data_val in 0u32..1000u32,
    ) {
        let report = SensorReport {
            schema: SENSOR_REPORT_SCHEMA.to_string(),
            tool: meta.clone(),
            generated_at: "2025-01-01T00:00:00Z".into(),
            verdict,
            summary: summary.clone(),
            findings: findings.clone(),
            artifacts: Some(artifacts.clone()),
            capabilities: Some(caps.clone()),
            data: Some(serde_json::json!({ data_key.clone(): data_val })),
        };

        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(SENSOR_REPORT_SCHEMA, back.schema.as_str());
        prop_assert_eq!(meta.name, back.tool.name);
        prop_assert_eq!(meta.version, back.tool.version);
        prop_assert_eq!(meta.mode, back.tool.mode);
        prop_assert_eq!(verdict, back.verdict);
        prop_assert_eq!(&summary, &back.summary);
        prop_assert_eq!(findings.len(), back.findings.len());
        for (orig, rt) in findings.iter().zip(back.findings.iter()) {
            prop_assert_eq!(&orig.check_id, &rt.check_id);
            prop_assert_eq!(&orig.code, &rt.code);
            prop_assert_eq!(orig.severity, rt.severity);
            prop_assert_eq!(&orig.title, &rt.title);
            prop_assert_eq!(&orig.message, &rt.message);
        }
        let back_arts = back.artifacts.unwrap();
        prop_assert_eq!(artifacts.len(), back_arts.len());
        for (orig, rt) in artifacts.iter().zip(back_arts.iter()) {
            prop_assert_eq!(&orig.artifact_type, &rt.artifact_type);
            prop_assert_eq!(&orig.path, &rt.path);
        }
        let back_caps = back.capabilities.unwrap();
        prop_assert_eq!(caps.len(), back_caps.len());
        let back_data = back.data.unwrap();
        prop_assert_eq!(back_data[&data_key].as_u64().unwrap(), data_val as u64);
    }

    // -----------------------------------------------------------------------
    // ToolMeta (SensorMeta) preserves all fields through JSON
    // -----------------------------------------------------------------------

    #[test]
    fn prop_tool_meta_json_preserves_all_fields(
        name in "[a-z_-]{1,30}",
        version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}(-[a-z0-9]+)?",
        mode in "[a-z_]{1,20}",
    ) {
        let meta = ToolMeta::new(&name, &version, &mode);
        let value: serde_json::Value = serde_json::to_value(&meta).unwrap();
        let obj = value.as_object().unwrap();

        // Exactly 3 keys in JSON
        prop_assert_eq!(obj.len(), 3);
        prop_assert_eq!(obj["name"].as_str().unwrap(), name.as_str());
        prop_assert_eq!(obj["version"].as_str().unwrap(), version.as_str());
        prop_assert_eq!(obj["mode"].as_str().unwrap(), mode.as_str());

        // Round-trip via JSON string
        let json = serde_json::to_string(&meta).unwrap();
        let back: ToolMeta = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&name, &back.name);
        prop_assert_eq!(&version, &back.version);
        prop_assert_eq!(&mode, &back.mode);
    }

    // -----------------------------------------------------------------------
    // Envelope integrity: data sections maintain structure through serde
    // -----------------------------------------------------------------------

    #[test]
    fn prop_envelope_data_integrity(
        meta in arb_tool_meta(),
        verdict in arb_verdict(),
        gate_status in arb_verdict(),
        gate_items in proptest::collection::vec(arb_gate_item(), 0..4),
        extra_int in 0i64..10000i64,
        extra_str in "[a-z]{1,30}",
    ) {
        let gates = GateResults::new(gate_status, gate_items.clone());
        let data = serde_json::json!({
            "gates": serde_json::to_value(&gates).unwrap(),
            "metrics": { "value": extra_int },
            "label": extra_str.clone(),
        });

        let report = SensorReport::new(
            meta,
            "2025-01-01T00:00:00Z".into(),
            verdict,
            "data integrity test".into(),
        )
        .with_data(data);

        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        let back_data = back.data.unwrap();

        // Gates sub-structure round-trips
        let back_gates: GateResults =
            serde_json::from_value(back_data["gates"].clone()).unwrap();
        prop_assert_eq!(gate_status, back_gates.status);
        prop_assert_eq!(gate_items.len(), back_gates.items.len());
        for (orig, rt) in gate_items.iter().zip(back_gates.items.iter()) {
            prop_assert_eq!(&orig.id, &rt.id);
            prop_assert_eq!(orig.status, rt.status);
        }

        // Scalar values survive
        prop_assert_eq!(back_data["metrics"]["value"].as_i64().unwrap(), extra_int);
        prop_assert_eq!(back_data["label"].as_str().unwrap(), extra_str.as_str());
    }

    // -----------------------------------------------------------------------
    // Default SensorReport produces valid JSON
    // -----------------------------------------------------------------------

    #[test]
    fn prop_default_report_is_valid_json(
        summary in "[A-Za-z0-9 ]{0,50}",
        mode in "[a-z]{1,10}",
    ) {
        let report = SensorReport::new(
            ToolMeta::tokmd(env!("CARGO_PKG_VERSION"), &mode),
            String::new(),
            Verdict::default(),
            summary.clone(),
        );

        // Serializes without error
        let json = serde_json::to_string(&report).unwrap();
        // Parses as valid JSON Value
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        prop_assert!(value.is_object());
        let obj = value.as_object().unwrap();
        // Required keys present
        prop_assert!(obj.contains_key("schema"));
        prop_assert!(obj.contains_key("tool"));
        prop_assert!(obj.contains_key("generated_at"));
        prop_assert!(obj.contains_key("verdict"));
        prop_assert!(obj.contains_key("summary"));
        prop_assert!(obj.contains_key("findings"));
        // Default verdict is "pass"
        prop_assert_eq!(obj["verdict"].as_str().unwrap(), "pass");
        // Findings is an empty array
        prop_assert!(obj["findings"].as_array().unwrap().is_empty());
        // Optional keys absent
        prop_assert!(!obj.contains_key("artifacts"));
        prop_assert!(!obj.contains_key("capabilities"));
        prop_assert!(!obj.contains_key("data"));
        // Deserializes back
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&summary, &back.summary);
        prop_assert_eq!(Verdict::Pass, back.verdict);
    }

    // -----------------------------------------------------------------------
    // Vec<SensorReport> round-trips correctly (multi-sensor aggregation)
    // -----------------------------------------------------------------------

    #[test]
    fn prop_multiple_sensor_reports_roundtrip(
        metas in proptest::collection::vec(arb_tool_meta(), 1..6),
        verdicts in proptest::collection::vec(arb_verdict(), 1..6),
        summaries in proptest::collection::vec("[A-Za-z0-9 ]{1,40}", 1..6),
    ) {
        let count = metas.len().min(verdicts.len()).min(summaries.len());
        let reports: Vec<SensorReport> = (0..count)
            .map(|i| {
                SensorReport::new(
                    metas[i].clone(),
                    format!("2025-01-01T{:02}:00:00Z", i % 24),
                    verdicts[i],
                    summaries[i].clone(),
                )
            })
            .collect();

        let json = serde_json::to_string(&reports).unwrap();
        let back: Vec<SensorReport> = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(reports.len(), back.len());
        for (orig, rt) in reports.iter().zip(back.iter()) {
            prop_assert_eq!(&orig.schema, &rt.schema);
            prop_assert_eq!(&orig.tool.name, &rt.tool.name);
            prop_assert_eq!(&orig.tool.version, &rt.tool.version);
            prop_assert_eq!(&orig.tool.mode, &rt.tool.mode);
            prop_assert_eq!(orig.verdict, rt.verdict);
            prop_assert_eq!(&orig.summary, &rt.summary);
            prop_assert_eq!(&orig.generated_at, &rt.generated_at);
        }
    }

    // -----------------------------------------------------------------------
    // Vec<SensorReport> with data payloads round-trips
    // -----------------------------------------------------------------------

    #[test]
    fn prop_multiple_sensor_reports_with_data_roundtrip(
        metas in proptest::collection::vec(arb_tool_meta(), 1..4),
        verdicts in proptest::collection::vec(arb_verdict(), 1..4),
        data_vals in proptest::collection::vec(0u32..1000u32, 1..4),
    ) {
        let count = metas.len().min(verdicts.len()).min(data_vals.len());
        let reports: Vec<SensorReport> = (0..count)
            .map(|i| {
                SensorReport::new(
                    metas[i].clone(),
                    "2025-01-01T00:00:00Z".into(),
                    verdicts[i],
                    format!("report {}", i),
                )
                .with_data(serde_json::json!({ "score": data_vals[i] }))
            })
            .collect();

        let json = serde_json::to_string(&reports).unwrap();
        let back: Vec<SensorReport> = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(reports.len(), back.len());
        for (i, rt) in back.iter().enumerate() {
            let rt_data = rt.data.as_ref().unwrap();
            prop_assert_eq!(rt_data["score"].as_u64().unwrap(), data_vals[i] as u64);
        }
    }
}
