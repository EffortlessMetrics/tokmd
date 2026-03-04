//! Property-based tests for tokmd-envelope.
//!
//! Covers: SensorReport serde roundtrip, finding_id non-empty,
//! fingerprint format, verdict exhaustiveness, and capability roundtrip.

use proptest::prelude::*;
use tokmd_envelope::{
    Artifact, CapabilityStatus, Finding, FindingLocation, FindingSeverity, SENSOR_REPORT_SCHEMA,
    SensorReport, ToolMeta, Verdict, findings,
};

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

// =========================================================================
// SensorReport roundtrips through serde
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn sensor_report_serde_roundtrip(
        tool_name in "[a-z]{3,12}",
        version in "[0-9]{1,2}\\.[0-9]{1,2}\\.[0-9]{1,2}",
        mode in "[a-z]{3,10}",
        verdict in arb_verdict(),
        summary in "[a-zA-Z0-9 ]{5,50}",
    ) {
        let report = SensorReport::new(
            ToolMeta::new(&tool_name, &version, &mode),
            "2024-06-15T12:00:00Z".to_string(),
            verdict,
            summary.clone(),
        );

        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(&back.schema, SENSOR_REPORT_SCHEMA);
        prop_assert_eq!(&back.tool.name, &tool_name);
        prop_assert_eq!(&back.tool.version, &version);
        prop_assert_eq!(&back.tool.mode, &mode);
        prop_assert_eq!(back.verdict, verdict);
        prop_assert_eq!(&back.summary, &summary);
    }
}

// =========================================================================
// SensorReport with findings roundtrips
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn report_with_findings_roundtrip(
        check_id in "[a-z]{3,10}",
        code in "[a-z_]{3,15}",
        severity in arb_severity(),
        title in "[a-zA-Z0-9 ]{5,30}",
        message in "[a-zA-Z0-9 ]{10,60}",
        path in "[a-z/]{3,20}\\.rs",
    ) {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "test"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            "Test report".to_string(),
        );
        report.add_finding(
            Finding::new(&check_id, &code, severity, &title, &message)
                .with_location(FindingLocation::path(&path)),
        );

        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(back.findings.len(), 1);
        prop_assert_eq!(&back.findings[0].check_id, &check_id);
        prop_assert_eq!(&back.findings[0].code, &code);
        prop_assert_eq!(&back.findings[0].title, &title);
        let loc = back.findings[0].location.as_ref().unwrap();
        prop_assert_eq!(&loc.path, &path);
    }
}

// =========================================================================
// finding_id is always non-empty
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn finding_id_always_non_empty(
        tool in "[a-z]{2,10}",
        check_id in "[a-z]{2,10}",
        code in "[a-z_]{2,15}",
    ) {
        let fid = findings::finding_id(&tool, &check_id, &code);
        prop_assert!(!fid.is_empty(), "finding_id must not be empty");
        prop_assert!(fid.contains('.'), "finding_id should contain dots: {}", fid);
        let parts: Vec<&str> = fid.split('.').collect();
        prop_assert_eq!(parts.len(), 3, "finding_id should have 3 parts: {}", fid);
        prop_assert_eq!(parts[0], tool.as_str());
        prop_assert_eq!(parts[1], check_id.as_str());
        prop_assert_eq!(parts[2], code.as_str());
    }
}

// =========================================================================
// Fingerprint: always 32 hex characters
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn fingerprint_is_32_hex_chars(
        check_id in "[a-z]{3,10}",
        code in "[a-z_]{3,15}",
        path in "[a-z/]{3,20}\\.rs",
    ) {
        let finding = Finding::new(&check_id, &code, FindingSeverity::Info, "title", "message")
            .with_location(FindingLocation::path(&path));
        let fp = finding.compute_fingerprint("tokmd");

        prop_assert_eq!(fp.len(), 32,
            "Fingerprint should be 32 chars, got {} ('{}')", fp.len(), fp);
        prop_assert!(fp.chars().all(|c| c.is_ascii_hexdigit()),
            "Fingerprint should be all hex: '{}'", fp);
    }

    #[test]
    fn fingerprint_deterministic(
        check_id in "[a-z]{3,10}",
        code in "[a-z_]{3,15}",
    ) {
        let finding = Finding::new(&check_id, &code, FindingSeverity::Warn, "t", "m");
        let fp1 = finding.compute_fingerprint("tokmd");
        let fp2 = finding.compute_fingerprint("tokmd");
        prop_assert_eq!(fp1, fp2, "Fingerprint must be deterministic");
    }
}

// =========================================================================
// Verdict: Display and serde agree
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn verdict_display_matches_serde(
        verdict in arb_verdict(),
    ) {
        let display = verdict.to_string();
        let json = serde_json::to_value(verdict).unwrap();
        prop_assert_eq!(json.as_str().unwrap(), display.as_str(),
            "Display '{}' != serde '{}'", display, json);
    }
}

// =========================================================================
// Capability roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn capability_status_roundtrip(
        reason in "[a-zA-Z0-9 ]{5,30}",
        state_idx in 0usize..3,
    ) {
        let status = match state_idx {
            0 => CapabilityStatus::available().with_reason(&reason),
            1 => CapabilityStatus::unavailable(&reason),
            _ => CapabilityStatus::skipped(&reason),
        };

        let json = serde_json::to_string(&status).unwrap();
        let back: CapabilityStatus = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.reason.as_deref(), Some(reason.as_str()));
    }
}

// =========================================================================
// Artifact serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn artifact_roundtrip(
        artifact_type in "[a-z]{3,10}",
        path in "[a-z/]{3,25}",
    ) {
        let artifact = Artifact::new(&artifact_type, &path);
        let json = serde_json::to_string(&artifact).unwrap();
        let back: Artifact = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.artifact_type, &artifact_type);
        prop_assert_eq!(&back.path, &path);
    }
}
