//! Contract hardening tests for tokmd-envelope SensorReport.

use serde_json::Value;
use tokmd_envelope::{
    Artifact, Finding, FindingLocation, FindingSeverity, SENSOR_REPORT_SCHEMA, SensorReport,
    ToolMeta, Verdict,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn sample_tool_meta() -> ToolMeta {
    ToolMeta::tokmd("1.0.0", "cockpit")
}

fn sample_report() -> SensorReport {
    SensorReport::new(
        sample_tool_meta(),
        "2024-01-15T12:00:00Z".into(),
        Verdict::Pass,
        "All checks passed".into(),
    )
}

fn sample_finding() -> Finding {
    Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "Hot file detected",
        "src/lib.rs has high churn",
    )
    .with_location(FindingLocation::path_line("src/lib.rs", 42))
    .with_fingerprint("tokmd")
}

// ── SensorReport serialization/deserialization ───────────────────────────

#[test]
fn sensor_report_json_roundtrip() {
    let report = sample_report();
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(back.verdict, Verdict::Pass);
    assert_eq!(back.summary, "All checks passed");
}

#[test]
fn sensor_report_with_findings_roundtrip() {
    let mut report = sample_report();
    report.add_finding(sample_finding());
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.findings.len(), 1);
    assert_eq!(back.findings[0].check_id, "risk");
    assert_eq!(back.findings[0].code, "hotspot");
}

#[test]
fn sensor_report_with_artifacts_roundtrip() {
    let report = sample_report().with_artifacts(vec![Artifact::new("receipt", "output.json")]);
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let artifacts = back.artifacts.unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].artifact_type, "receipt");
}

#[test]
fn sensor_report_with_data_roundtrip() {
    let data = serde_json::json!({"key": "value", "count": 42});
    let report = sample_report().with_data(data.clone());
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.data.unwrap(), data);
}

// ── Envelope fields ──────────────────────────────────────────────────────

#[test]
fn envelope_has_schema_field() {
    let report = sample_report();
    let json = serde_json::to_string(&report).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema"].as_str().unwrap(), "sensor.report.v1");
}

#[test]
fn envelope_has_timestamp() {
    let report = sample_report();
    let json = serde_json::to_string(&report).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v["generated_at"].is_string());
    assert!(!v["generated_at"].as_str().unwrap().is_empty());
}

#[test]
fn envelope_has_tool_fields() {
    let report = sample_report();
    let json = serde_json::to_string(&report).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["tool"]["name"].as_str().unwrap(), "tokmd");
    assert_eq!(v["tool"]["version"].as_str().unwrap(), "1.0.0");
    assert_eq!(v["tool"]["mode"].as_str().unwrap(), "cockpit");
}

#[test]
fn envelope_has_verdict() {
    let report = sample_report();
    let json = serde_json::to_string(&report).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["verdict"].as_str().unwrap(), "pass");
}

// ── Verdict serde stability ──────────────────────────────────────────────

#[test]
fn verdict_serde_values() {
    assert_eq!(serde_json::to_string(&Verdict::Pass).unwrap(), "\"pass\"");
    assert_eq!(serde_json::to_string(&Verdict::Fail).unwrap(), "\"fail\"");
    assert_eq!(serde_json::to_string(&Verdict::Warn).unwrap(), "\"warn\"");
    assert_eq!(serde_json::to_string(&Verdict::Skip).unwrap(), "\"skip\"");
    assert_eq!(
        serde_json::to_string(&Verdict::Pending).unwrap(),
        "\"pending\""
    );
}

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

// ── Finding ID stability ─────────────────────────────────────────────────

#[test]
fn finding_ids_are_stable() {
    use tokmd_envelope::findings;

    assert_eq!(findings::risk::CHECK_ID, "risk");
    assert_eq!(findings::risk::HOTSPOT, "hotspot");
    assert_eq!(findings::risk::COUPLING, "coupling");
    assert_eq!(findings::risk::BUS_FACTOR, "bus_factor");

    assert_eq!(findings::contract::CHECK_ID, "contract");
    assert_eq!(findings::contract::SCHEMA_CHANGED, "schema_changed");
    assert_eq!(findings::contract::API_CHANGED, "api_changed");

    assert_eq!(findings::supply::CHECK_ID, "supply");
    assert_eq!(findings::supply::LOCKFILE_CHANGED, "lockfile_changed");

    assert_eq!(findings::gate::CHECK_ID, "gate");
    assert_eq!(findings::gate::MUTATION_FAILED, "mutation_failed");
}

// ── Fingerprint stability ────────────────────────────────────────────────

#[test]
fn fingerprint_is_deterministic() {
    let f = sample_finding();
    let fp1 = f.compute_fingerprint("tokmd");
    let fp2 = f.compute_fingerprint("tokmd");
    assert_eq!(fp1, fp2);
    assert_eq!(fp1.len(), 32, "fingerprint should be 32 hex chars");
}

#[test]
fn fingerprint_changes_with_tool_name() {
    let f = sample_finding();
    let fp1 = f.compute_fingerprint("tokmd");
    let fp2 = f.compute_fingerprint("other-tool");
    assert_ne!(fp1, fp2);
}

// ── Empty receipts are valid envelopes ───────────────────────────────────

#[test]
fn empty_report_is_valid() {
    let report = SensorReport::new(
        ToolMeta::new("test", "0.0.1", "check"),
        "2024-01-01T00:00:00Z".into(),
        Verdict::Skip,
        "No checks ran".into(),
    );
    let json = serde_json::to_string(&report).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema"].as_str().unwrap(), SENSOR_REPORT_SCHEMA);
    assert!(v["findings"].as_array().unwrap().is_empty());
    // Optional fields should not appear when None
    assert!(v.get("artifacts").is_none() || v["artifacts"].is_null());
    assert!(v.get("data").is_none() || v["data"].is_null());
}

// ── Schema string constant ───────────────────────────────────────────────

#[test]
fn sensor_report_schema_constant() {
    assert_eq!(SENSOR_REPORT_SCHEMA, "sensor.report.v1");
}

// ── Property tests ───────────────────────────────────────────────────────

mod properties {
    use proptest::prelude::*;
    use tokmd_envelope::{SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict};

    fn arb_verdict() -> impl Strategy<Value = Verdict> {
        prop_oneof![
            Just(Verdict::Pass),
            Just(Verdict::Fail),
            Just(Verdict::Warn),
            Just(Verdict::Skip),
            Just(Verdict::Pending),
        ]
    }

    fn arb_sensor_report() -> impl Strategy<Value = SensorReport> {
        (
            "[a-z]{3,8}",
            "[0-9]+\\.[0-9]+\\.[0-9]+",
            "[a-z]{3,8}",
            arb_verdict(),
            ".{1,50}",
        )
            .prop_map(|(name, version, mode, verdict, summary)| {
                SensorReport::new(
                    ToolMeta::new(&name, &version, &mode),
                    "2024-01-01T00:00:00Z".into(),
                    verdict,
                    summary,
                )
            })
    }

    proptest! {
        #[test]
        fn envelope_json_roundtrip(report in arb_sensor_report()) {
            let json = serde_json::to_string(&report).unwrap();
            let back: SensorReport = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(&back.schema, SENSOR_REPORT_SCHEMA);
            prop_assert_eq!(back.verdict, report.verdict);
            prop_assert_eq!(&back.tool.name, &report.tool.name);
        }
    }
}
