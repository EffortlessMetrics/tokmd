//! Extended identity & security tests for license radar scanning.
//!
//! Covers gaps not exercised by existing unit/bdd/edge/property suites:
//! - Additional license text variants (LICENSE.md, LICENSE-APACHE)
//! - SPDX compound expressions in metadata
//! - Metadata absence in pyproject.toml
//! - Near-miss text below min_hits threshold
//! - Whitespace-only license files
//! - package.json license array (deprecated format)
//! - Cargo.toml license-file in subdirectory

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokmd_analysis_license::build_license_report;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::LicenseSourceKind;

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ===========================================================================
// 1. LICENSE.md variant is recognized as license text
// ===========================================================================

#[test]
fn license_md_variant_is_recognized() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE.md"),
        "Permission is hereby granted, free of charge, to any person.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE.md")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "MIT"),
        "LICENSE.md should be scanned for license text: {:?}",
        report.findings
    );
}

// ===========================================================================
// 2. SPDX compound expression round-trips through metadata
// ===========================================================================

#[test]
fn spdx_compound_and_expression_round_trips() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT AND Apache-2.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT AND Apache-2.0");
    assert_eq!(report.effective.as_deref(), Some("MIT AND Apache-2.0"));
}

#[test]
fn spdx_with_exception_round_trips() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"Apache-2.0 WITH LLVM-exception\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings[0].spdx, "Apache-2.0 WITH LLVM-exception");
}

// ===========================================================================
// 3. pyproject.toml without license field yields no finding
// ===========================================================================

#[test]
fn pyproject_toml_without_license_field_yields_no_metadata() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"x\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.is_empty(),
        "pyproject.toml without license should produce no findings"
    );
}

// ===========================================================================
// 4. Near-miss license text below min_hits threshold
// ===========================================================================

#[test]
fn apache_text_with_only_one_phrase_below_threshold() {
    let dir = tempdir().unwrap();
    // Apache-2.0 requires min_hits=2, so a single phrase should not match
    fs::write(
        dir.path().join("LICENSE"),
        "This software is under the Apache License. That's all.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // Should not detect Apache-2.0 with only one of the required phrases
    assert!(
        !report.findings.iter().any(|f| f.spdx == "Apache-2.0"),
        "single Apache phrase should not meet min_hits threshold"
    );
}

#[test]
fn gpl_text_with_only_one_phrase_below_threshold() {
    let dir = tempdir().unwrap();
    // GPL-3.0 requires min_hits=2
    fs::write(
        dir.path().join("LICENSE"),
        "GNU General Public License. Some unrelated text.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        !report.findings.iter().any(|f| f.spdx == "GPL-3.0-or-later"),
        "single GPL phrase should not meet min_hits threshold"
    );
}

// ===========================================================================
// 5. Whitespace-only license file yields no findings
// ===========================================================================

#[test]
fn whitespace_only_license_file_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("LICENSE"), "   \n\n  \t  \n").unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // The text has no recognizable license phrases
    assert!(
        report.findings.is_empty(),
        "whitespace-only license file should produce no findings"
    );
}

// ===========================================================================
// 6. package.json deprecated array license format
// ===========================================================================

#[test]
fn package_json_array_license_yields_no_finding() {
    let dir = tempdir().unwrap();
    // Deprecated format: "licenses": [{"type": "MIT"}]
    // The code checks "license" (singular), so array under "licenses" is ignored
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "licenses": [{"type": "MIT"}]}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // "licenses" (plural) is not recognized
    assert!(report.findings.is_empty());
}

#[test]
fn package_json_license_as_number_yields_no_finding() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "license": 42}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.is_empty(),
        "numeric license value should not produce a finding"
    );
}

// ===========================================================================
// 7. Cargo.toml license-file in subdirectory resolves correctly
// ===========================================================================

#[test]
fn cargo_toml_license_file_in_subdirectory() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("licenses");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense-file = \"licenses/CUSTOM\"\n",
    )
    .unwrap();
    fs::write(
        sub.join("CUSTOM"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("licenses").join("CUSTOM"),
    ];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "MIT"
            && f.source_kind == LicenseSourceKind::Text
            && f.source_path == "licenses/CUSTOM"),
        "license-file in subdirectory should be scanned: {:?}",
        report.findings
    );
}

// ===========================================================================
// 8. Multiple LICENSE text files produce distinct source_path values
// ===========================================================================

#[test]
fn multiple_license_files_have_distinct_source_paths() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE-MIT"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE-APACHE"),
        "Apache License\nVersion 2.0\n\
         http://www.apache.org/licenses/\n\
         limitations under the License.",
    )
    .unwrap();

    let files = vec![
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("LICENSE-APACHE"),
    ];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    let paths: Vec<&str> = report
        .findings
        .iter()
        .map(|f| f.source_path.as_str())
        .collect();
    // All source_path values should be unique
    let mut unique = paths.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        paths.len(),
        unique.len(),
        "each finding should have a distinct source_path"
    );
}

// ===========================================================================
// 9. Cargo.toml license field in non-[package] section is ignored
// ===========================================================================

#[test]
fn cargo_toml_license_in_dependencies_section_ignored() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\nlicense = \"MIT\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.is_empty(),
        "license field outside [package] section should be ignored"
    );
}

// ===========================================================================
// 10. MPL-2.0 with only two of three phrases detected
// ===========================================================================

#[test]
fn mpl2_with_two_phrases_still_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Mozilla Public License\nVersion 2.0\n\
         This Source Code Form is subject to the terms of the Mozilla Public License.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "MPL-2.0"),
        "MPL-2.0 should be detected with 2 phrase hits"
    );
}

// ===========================================================================
// 11. Findings from both metadata and text for same license are both present
// ===========================================================================

#[test]
fn metadata_and_text_same_license_produces_two_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    let metadata_count = report
        .findings
        .iter()
        .filter(|f| f.source_kind == LicenseSourceKind::Metadata)
        .count();
    let text_count = report
        .findings
        .iter()
        .filter(|f| f.source_kind == LicenseSourceKind::Text)
        .count();

    assert!(
        metadata_count >= 1,
        "should have at least one metadata finding"
    );
    assert!(text_count >= 1, "should have at least one text finding");
    assert!(
        report.findings.len() >= 2,
        "should have both metadata and text findings"
    );
}

// ===========================================================================
// 12. Empty file list with non-empty root yields empty report
// ===========================================================================

#[test]
fn empty_file_list_with_populated_root_yields_empty_report() {
    let dir = tempdir().unwrap();
    // Create files in the directory but pass empty file list
    fs::write(dir.path().join("LICENSE"), "MIT License").unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nlicense = \"MIT\"\n",
    )
    .unwrap();

    let report = build_license_report(dir.path(), &[], &default_limits()).unwrap();
    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ===========================================================================
// 13. pyproject.toml with tool.poetry license
// ===========================================================================

#[test]
fn pyproject_toml_poetry_license_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.poetry]\nname = \"x\"\nlicense = \"BSD-3-Clause\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "BSD-3-Clause");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
}

// ===========================================================================
// 14. package.json with object-style license
// ===========================================================================

#[test]
fn package_json_object_license_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "license": {"type": "ISC"}}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "ISC");
}

// ===========================================================================
// 15. Effective license is highest confidence finding
// ===========================================================================

#[test]
fn effective_license_is_highest_confidence() {
    let dir = tempdir().unwrap();
    // Metadata confidence is 0.95; MIT text with both phrases yields 1.0
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"Apache-2.0\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // MIT text matching both phrases gives confidence 1.0, beating metadata 0.95
    assert_eq!(report.effective.as_deref(), Some("MIT"));
    // First finding should be highest confidence
    assert!(report.findings[0].confidence >= report.findings.last().unwrap().confidence);
}
