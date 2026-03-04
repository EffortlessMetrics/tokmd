//! Wave-49 deep tests for license report building.
//!
//! Covers metadata detection, text matching, confidence scoring,
//! sorting, serde roundtrips, and property-based tests.

use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use tokmd_analysis_license::build_license_report;
use tokmd_analysis_types::{LicenseFinding, LicenseReport, LicenseSourceKind};
use tokmd_analysis_util::AnalysisLimits;

// ── 1. Empty files list returns empty report ────────────────────

#[test]
fn empty_files_returns_empty_report() {
    let dir = tempdir().unwrap();
    let report = build_license_report(dir.path(), &[], &AnalysisLimits::default()).unwrap();
    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ── 2. Cargo.toml metadata detection ────────────────────────────

#[test]
fn cargo_toml_metadata_detection() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"Apache-2.0\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "Apache-2.0");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
    assert!((report.findings[0].confidence - 0.95).abs() < f32::EPSILON);
    assert_eq!(report.effective.as_deref(), Some("Apache-2.0"));
}

// ── 3. package.json string license ──────────────────────────────

#[test]
fn package_json_string_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "demo", "license": "ISC"}"#,
    )
    .unwrap();
    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "ISC");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
}

// ── 4. package.json object license with type field ──────────────

#[test]
fn package_json_object_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "demo", "license": {"type": "MIT", "url": "https://example.com"}}"#,
    )
    .unwrap();
    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.findings[0].spdx, "MIT");
}

// ── 5. pyproject.toml detection ─────────────────────────────────

#[test]
fn pyproject_toml_detection() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"demo\"\nlicense = \"BSD-3-Clause\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "BSD-3-Clause");
}

// ── 6. Text license MIT detection ───────────────────────────────

#[test]
fn text_license_mit_detection() {
    let dir = tempdir().unwrap();
    let text = "MIT License\n\n\
        Permission is hereby granted, free of charge, to any person obtaining a copy \
        of this software and associated documentation files. \
        THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "MIT"));
    let finding = report.findings.iter().find(|f| f.spdx == "MIT").unwrap();
    assert_eq!(finding.source_kind, LicenseSourceKind::Text);
    assert!(finding.confidence >= 0.6);
    assert!(finding.confidence <= 1.0);
}

// ── 7. Text license Apache-2.0 detection ────────────────────────

#[test]
fn text_license_apache_detection() {
    let dir = tempdir().unwrap();
    let text = "Apache License\nVersion 2.0, January 2004\n\
        http://www.apache.org/licenses/\n\
        limitations under the License.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "Apache-2.0"));
}

// ── 8. Findings sorted by confidence desc, spdx asc ────────────

#[test]
fn findings_sorted_confidence_desc_spdx_asc() {
    let dir = tempdir().unwrap();
    // Metadata finding (confidence 0.95) + text finding (confidence < 0.95)
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let apache_text = "Apache License\nVersion 2.0\n\
        http://www.apache.org/licenses/\nlimitations under the License.";
    fs::write(dir.path().join("LICENSE"), apache_text).unwrap();
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
    assert!(report.findings.len() >= 2);
    // Metadata (0.95) should come before text
    for pair in report.findings.windows(2) {
        assert!(
            pair[0].confidence >= pair[1].confidence,
            "findings should be sorted by confidence desc: {} >= {}",
            pair[0].confidence,
            pair[1].confidence
        );
    }
}

// ── 9. Effective is highest confidence finding ──────────────────

#[test]
fn effective_is_highest_confidence() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"GPL-3.0\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
    assert_eq!(
        report.effective.as_deref(),
        Some("GPL-3.0"),
        "effective should be the first finding's spdx"
    );
}

// ── 10. Serde roundtrip preserves all fields ────────────────────

#[test]
fn serde_roundtrip_preserves_all_fields() {
    let dir = tempdir().unwrap();
    let mit_text = "Permission is hereby granted, free of charge, \
        to any person. The software is provided \"as is\".";
    fs::write(dir.path().join("LICENSE"), mit_text).unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    let deser: LicenseReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.findings.len(), report.findings.len());
    assert_eq!(deser.effective, report.effective);
    for (orig, rt) in report.findings.iter().zip(deser.findings.iter()) {
        assert_eq!(orig.spdx, rt.spdx);
        assert!((orig.confidence - rt.confidence).abs() < f32::EPSILON);
        assert_eq!(orig.source_path, rt.source_path);
    }
}

// ── 11. Source path uses forward slashes ─────────────────────────

#[test]
fn source_path_forward_slashes() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
    for f in &report.findings {
        assert!(
            !f.source_path.contains('\\'),
            "paths should use forward slashes"
        );
    }
}

// ── Proptest ────────────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn confidence_always_in_range(
            spdx in "[A-Z]{2,6}-[0-9]\\.[0-9]",
            conf in 0.0f32..=1.0f32,
        ) {
            let finding = LicenseFinding {
                spdx,
                confidence: conf,
                source_path: "LICENSE".to_string(),
                source_kind: LicenseSourceKind::Text,
            };
            let json = serde_json::to_string(&finding).unwrap();
            let rt: LicenseFinding = serde_json::from_str(&json).unwrap();
            prop_assert!(rt.confidence >= 0.0 && rt.confidence <= 1.0);
            prop_assert_eq!(rt.source_kind, LicenseSourceKind::Text);
        }
    }
}
