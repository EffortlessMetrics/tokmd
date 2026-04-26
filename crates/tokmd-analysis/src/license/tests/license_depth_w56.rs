//! Wave-56 depth tests for license discovery.
//!
//! Covers license file detection, type classification, SPDX parsing,
//! edge cases for dual licenses, no licenses, and unusual license files.

use std::fs;
use std::path::PathBuf;

use crate::license::build_license_report;
use tempfile::tempdir;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::{LicenseReport, LicenseSourceKind};

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ── 1. GPL-3.0 text detection ───────────────────────────────────

#[test]
fn text_license_gpl3_detection() {
    let dir = tempdir().unwrap();
    let text = "GNU General Public License\n\
        Version 3, 29 June 2007\n\
        Everyone is permitted to copy and distribute verbatim copies of this \
        license document, but changing it is not allowed.\n\
        You may convey the Program under any later version.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(
        report.findings.iter().any(|f| f.spdx.contains("GPL")),
        "should detect GPL license"
    );
}

// ── 2. BSD-3-Clause text detection ──────────────────────────────

#[test]
fn text_license_bsd3_detection() {
    let dir = tempdir().unwrap();
    let text = "Redistribution and use in source and binary forms, with or without \
        modification, are permitted provided that the following conditions are met:\n\
        Neither the name of the copyright holder nor the names of its \
        contributors may be used to endorse or promote products derived from this software.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx.contains("BSD")));
}

// ── 3. Apache-2.0 text with all phrases ─────────────────────────

#[test]
fn text_license_apache_full_confidence() {
    let dir = tempdir().unwrap();
    let text = "Apache License\n\
        Version 2.0, January 2004\n\
        http://www.apache.org/licenses/\n\
        Unless required by applicable law or agreed to in writing, software \
        distributed under the License is distributed on an \"AS IS\" BASIS.\n\
        limitations under the License.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let apache = report
        .findings
        .iter()
        .find(|f| f.spdx == "Apache-2.0")
        .expect("should detect Apache-2.0");
    // All 4 phrases matched → confidence = 0.6 + 0.4 * (4/4) = 1.0
    assert!(
        apache.confidence > 0.9,
        "full match should yield high confidence: {}",
        apache.confidence
    );
}

// ── 4. MPL-2.0 text detection ───────────────────────────────────

#[test]
fn text_license_mpl2_detection() {
    let dir = tempdir().unwrap();
    let text = "Mozilla Public License\n\
        Version 2.0\n\
        http://mozilla.org/MPL/2.0/\n\
        If a copy of the MPL was not distributed with this file.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "MPL-2.0"));
}

// ── 5. Empty LICENSE file: no findings ──────────────────────────

#[test]
fn empty_license_file_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("LICENSE"), "").unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(
        report.findings.is_empty(),
        "empty LICENSE should yield no findings"
    );
    assert!(report.effective.is_none());
}

// ── 6. Non-license content: no match ────────────────────────────

#[test]
fn non_license_content_no_match() {
    let dir = tempdir().unwrap();
    let text = "This is a README file.\nIt describes the project.\nNothing about licenses here.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(
        report.findings.is_empty(),
        "non-license content should yield no findings"
    );
}

// ── 7. Dual license: metadata + text both detected ──────────────

#[test]
fn dual_license_metadata_and_text() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"MIT OR Apache-2.0\"\n",
    )
    .unwrap();
    let mit_text = "Permission is hereby granted, free of charge, to any person \
        obtaining a copy. The software is provided \"as is\".";
    fs::write(dir.path().join("LICENSE-MIT"), mit_text).unwrap();
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE-MIT")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(
        report.findings.len() >= 2,
        "should find both metadata and text licenses"
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.source_kind == LicenseSourceKind::Metadata)
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.source_kind == LicenseSourceKind::Text)
    );
}

// ── 8. Cargo.toml license-file reference ────────────────────────

#[test]
fn cargo_toml_license_file_reference() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense-file = \"CUSTOM-LICENSE\"\n",
    )
    .unwrap();
    let custom_text = "Permission is hereby granted, free of charge. \
        The software is provided \"as is\".";
    fs::write(dir.path().join("CUSTOM-LICENSE"), custom_text).unwrap();
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("CUSTOM-LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(
        report.findings.iter().any(|f| f.spdx == "MIT"),
        "should find MIT in custom license file"
    );
}

// ── 9. pyproject.toml with tool.poetry section ──────────────────

#[test]
fn pyproject_toml_poetry_section() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.poetry]\nname = \"demo\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "MIT"));
}

// ── 10. package.json with no license field ──────────────────────

#[test]
fn package_json_no_license_field() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "demo", "version": "1.0.0"}"#,
    )
    .unwrap();
    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(
        report.findings.is_empty(),
        "package.json without license should yield no findings"
    );
}

// ── 11. Confidence in range [0.6, 1.0] for text matches ────────

#[test]
fn text_confidence_bounded() {
    let dir = tempdir().unwrap();
    // Minimal MIT match: just one phrase
    let text = "Permission is hereby granted, free of charge, to all.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    for f in &report.findings {
        if f.source_kind == LicenseSourceKind::Text {
            assert!(
                f.confidence >= 0.6 && f.confidence <= 1.0,
                "text confidence should be in [0.6, 1.0], got {}",
                f.confidence
            );
        }
    }
}

// ── 12. Metadata confidence is always 0.95 ──────────────────────

#[test]
fn metadata_confidence_fixed() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"LGPL-2.1\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let metadata = report
        .findings
        .iter()
        .find(|f| f.source_kind == LicenseSourceKind::Metadata)
        .unwrap();
    assert!(
        (metadata.confidence - 0.95).abs() < f32::EPSILON,
        "metadata confidence should be exactly 0.95"
    );
}

// ── 13. Effective license is highest confidence ─────────────────

#[test]
fn effective_is_highest_confidence_finding() {
    let dir = tempdir().unwrap();
    // Metadata (0.95) should win over text
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"Apache-2.0\"\n",
    )
    .unwrap();
    // Only one MIT phrase → text confidence = 0.6 + 0.4*(1/2) = 0.8 < metadata 0.95
    let mit_text = "Permission is hereby granted, free of charge.";
    fs::write(dir.path().join("LICENSE"), mit_text).unwrap();
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    // Metadata has 0.95 > text 0.8, so metadata effective wins
    assert_eq!(report.effective.as_deref(), Some("Apache-2.0"));
}

// ── 14. Findings sorted: confidence desc, spdx asc, path asc ───

#[test]
fn findings_sort_order() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let apache_text = "Apache License\nVersion 2.0\n\
        http://www.apache.org/licenses/\nlimitations under the License.";
    fs::write(dir.path().join("LICENSE"), apache_text).unwrap();
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    for pair in report.findings.windows(2) {
        assert!(
            pair[0].confidence >= pair[1].confidence,
            "confidence should be descending"
        );
        if (pair[0].confidence - pair[1].confidence).abs() < f32::EPSILON {
            assert!(
                pair[0].spdx <= pair[1].spdx,
                "tied confidence should sort by spdx asc"
            );
        }
    }
}

// ── 15. Source paths use forward slashes ─────────────────────────

#[test]
fn source_paths_forward_slashes() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    for f in &report.findings {
        assert!(
            !f.source_path.contains('\\'),
            "paths must use forward slashes"
        );
    }
}

// ── 16. Serde roundtrip: LicenseSourceKind variants ─────────────

#[test]
fn license_source_kind_serde_variants() {
    let kinds = [
        (LicenseSourceKind::Metadata, "\"metadata\""),
        (LicenseSourceKind::Text, "\"text\""),
    ];
    for (kind, expected) in &kinds {
        let json = serde_json::to_string(kind).unwrap();
        assert_eq!(&json, *expected);
        let rt: LicenseSourceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(&rt, kind);
    }
}

// ── 17. Serde roundtrip: full report ────────────────────────────

#[test]
fn serde_roundtrip_full_report() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let mit_text = "Permission is hereby granted, free of charge. \
        The software is provided \"as is\".";
    fs::write(dir.path().join("LICENSE"), mit_text).unwrap();
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    let rt: LicenseReport = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.findings.len(), report.findings.len());
    assert_eq!(rt.effective, report.effective);
}

// ── 18. Deterministic across calls ──────────────────────────────

#[test]
fn deterministic_across_calls() {
    let dir = tempdir().unwrap();
    let mit_text = "Permission is hereby granted, free of charge. \
        The software is provided \"as is\".";
    fs::write(dir.path().join("LICENSE"), mit_text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let r1 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let r2 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2, "license report must be deterministic");
}

// ── 19. SPDX expression preserved as-is ─────────────────────────

#[test]
fn spdx_expression_preserved() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"MIT OR Apache-2.0\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(
        report.findings[0].spdx, "MIT OR Apache-2.0",
        "SPDX expression should be preserved verbatim"
    );
}

// ── 20. BSD-2-Clause text detection ─────────────────────────────

#[test]
fn text_license_bsd2_detection() {
    let dir = tempdir().unwrap();
    let text = "Redistribution and use in source and binary forms, with or without \
        modification, are permitted.\n\
        This software is provided by the copyright holders and contributors \"as is\" \
        and any express or implied warranties.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(
        report.findings.iter().any(|f| f.spdx.contains("BSD")),
        "should detect BSD license variant"
    );
}

// ── 21. Multiple metadata files ─────────────────────────────────

#[test]
fn multiple_metadata_files() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "demo", "license": "ISC"}"#,
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(
        report.findings.len() >= 2,
        "should detect licenses from multiple metadata files"
    );
    assert!(report.findings.iter().any(|f| f.spdx == "MIT"));
    assert!(report.findings.iter().any(|f| f.spdx == "ISC"));
}

// ── 22. AGPL-3.0 text detection ────────────────────────────────

#[test]
fn text_license_agpl3_detection() {
    let dir = tempdir().unwrap();
    let text = "GNU Affero General Public License\n\
        Version 3, 19 November 2007\n\
        Everyone is permitted to copy. You may convey under any later version.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(
        report.findings.iter().any(|f| f.spdx.contains("AGPL")),
        "should detect AGPL license"
    );
}

// ── 23. No license files at all: empty report ───────────────────

#[test]
fn no_license_files_empty_report() {
    let dir = tempdir().unwrap();
    // Create a non-license file
    fs::write(dir.path().join("README.md"), "# Hello").unwrap();
    let files = vec![PathBuf::from("README.md")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ── 24. Confidence increases with more phrase matches ───────────

#[test]
fn confidence_increases_with_phrases() {
    let dir1 = tempdir().unwrap();
    // Apache with 2 phrases
    let text_min = "Apache License\nVersion 2.0";
    fs::write(dir1.path().join("LICENSE"), text_min).unwrap();
    let files1 = vec![PathBuf::from("LICENSE")];
    let r1 = build_license_report(dir1.path(), &files1, &default_limits()).unwrap();

    let dir2 = tempdir().unwrap();
    // Apache with all 4 phrases
    let text_full = "Apache License\nVersion 2.0\n\
        http://www.apache.org/licenses/\nlimitations under the License.";
    fs::write(dir2.path().join("LICENSE"), text_full).unwrap();
    let files2 = vec![PathBuf::from("LICENSE")];
    let r2 = build_license_report(dir2.path(), &files2, &default_limits()).unwrap();

    let c1 = r1
        .findings
        .iter()
        .find(|f| f.spdx == "Apache-2.0")
        .map(|f| f.confidence)
        .unwrap_or(0.0);
    let c2 = r2
        .findings
        .iter()
        .find(|f| f.spdx == "Apache-2.0")
        .map(|f| f.confidence)
        .unwrap_or(0.0);
    assert!(
        c2 >= c1,
        "more phrase matches should yield higher confidence: {} >= {}",
        c2,
        c1
    );
}

// ── 25. Cargo.toml with single quotes ───────────────────────────

#[test]
fn cargo_toml_single_quotes() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = 'demo'\nlicense = 'MIT'\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(
        report.findings.iter().any(|f| f.spdx == "MIT"),
        "should handle single-quoted values"
    );
}

// ── 26. Unrecognized license text: no match ─────────────────────

#[test]
fn unrecognized_license_text_no_match() {
    let dir = tempdir().unwrap();
    let text = "This software is released under the Beerware License. \
        As long as you retain this notice you can do whatever you want with this stuff.";
    fs::write(dir.path().join("LICENSE"), text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    // Beerware is not in our pattern list
    assert!(
        report.findings.is_empty(),
        "unrecognized license should not produce findings"
    );
}

// ── Proptest ────────────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn effective_always_from_findings(
            spdx in "[A-Z]{2,5}(-[0-9]\\.[0-9])?",
        ) {
            let dir = tempdir().unwrap();
            let content = format!("[package]\nname = \"test\"\nlicense = \"{spdx}\"\n");
            fs::write(dir.path().join("Cargo.toml"), &content).unwrap();
            let files = vec![PathBuf::from("Cargo.toml")];
            let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
            if let Some(eff) = &report.effective {
                prop_assert!(
                    report.findings.iter().any(|f| &f.spdx == eff),
                    "effective must come from findings"
                );
            }
        }
    }
}
