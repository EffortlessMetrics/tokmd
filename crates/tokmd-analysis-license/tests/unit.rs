use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokmd_analysis_license::build_license_report;
use tokmd_analysis_types::LicenseSourceKind;
use tokmd_analysis_util::AnalysisLimits;

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ---------------------------------------------------------------------------
// 1. BSD-2-Clause text detection
// ---------------------------------------------------------------------------

#[test]
fn detects_bsd2_clause_from_text() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Redistribution and use in source and binary forms, with or without modification, \
         are permitted provided that the following conditions are met:\n\
         1. Redistributions of source code must retain the above copyright notice.\n\
         2. Redistributions in binary form must reproduce the above copyright notice.\n\
         THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS \"AS IS\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(!report.findings.is_empty());
    // Should match BSD-2-Clause or BSD-3-Clause (both share redistribution phrase)
    assert!(report.findings.iter().any(|f| f.spdx.starts_with("BSD")));
}

// ---------------------------------------------------------------------------
// 2. Cargo.toml without license field yields no metadata finding
// ---------------------------------------------------------------------------

#[test]
fn cargo_toml_without_license_field_yields_no_metadata() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.source_kind == LicenseSourceKind::Metadata),
        "should not find metadata license when field is absent"
    );
}

// ---------------------------------------------------------------------------
// 3. package.json without license field yields no findings
// ---------------------------------------------------------------------------

#[test]
fn package_json_without_license_field_yields_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "version": "1.0.0"}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ---------------------------------------------------------------------------
// 4. Multiple metadata files from different ecosystems
// ---------------------------------------------------------------------------

#[test]
fn multiple_metadata_files_produce_multiple_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "license": "ISC"}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 2);
    let spdx_ids: Vec<&str> = report.findings.iter().map(|f| f.spdx.as_str()).collect();
    assert!(spdx_ids.contains(&"MIT"));
    assert!(spdx_ids.contains(&"ISC"));
}

// ---------------------------------------------------------------------------
// 5. License text matching is case-insensitive
// ---------------------------------------------------------------------------

#[test]
fn license_text_matching_is_case_insensitive() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "PERMISSION IS HEREBY GRANTED, FREE OF CHARGE, TO ANY PERSON.\n\
         THE SOFTWARE IS PROVIDED \"AS IS\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "MIT"),
        "case-insensitive matching should detect MIT"
    );
}

// ---------------------------------------------------------------------------
// 6. MIT with more phrase hits yields higher confidence
// ---------------------------------------------------------------------------

#[test]
fn mit_with_both_phrases_has_higher_confidence_than_one() {
    let dir_one = tempdir().unwrap();
    fs::write(
        dir_one.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.",
    )
    .unwrap();
    let report_one = build_license_report(
        dir_one.path(),
        &[PathBuf::from("LICENSE")],
        &default_limits(),
    )
    .unwrap();

    let dir_both = tempdir().unwrap();
    fs::write(
        dir_both.path().join("LICENSE"),
        "Permission is hereby granted, free of charge, to any person.\n\
         The software is provided \"as is\", without warranty.",
    )
    .unwrap();
    let report_both = build_license_report(
        dir_both.path(),
        &[PathBuf::from("LICENSE")],
        &default_limits(),
    )
    .unwrap();

    let conf_one = report_one.findings.first().map(|f| f.confidence).unwrap();
    let conf_both = report_both.findings.first().map(|f| f.confidence).unwrap();
    assert!(
        conf_both > conf_one,
        "two phrase hits ({conf_both}) should beat one ({conf_one})"
    );
}

// ---------------------------------------------------------------------------
// 7. LICENSE.txt variant is recognized
// ---------------------------------------------------------------------------

#[test]
fn license_txt_variant_is_recognized() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE.txt"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE.txt")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "MIT"));
}

// ---------------------------------------------------------------------------
// 8. COPYING file with GPL text is detected
// ---------------------------------------------------------------------------

#[test]
fn copying_file_with_gpl_text_is_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("COPYING"),
        "GNU General Public License\n\
         Version 3, 29 June 2007\n\
         Everyone is permitted to copy and distribute verbatim copies of this \
         license document, but changing it is not allowed.\n\
         You may redistribute it under the terms of the GNU General Public License \
         as published by the Free Software Foundation, either version 3 of the License, \
         or (at your option) any later version.",
    )
    .unwrap();

    let files = vec![PathBuf::from("COPYING")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "GPL-3.0-or-later"));
}

// ---------------------------------------------------------------------------
// 9. Determinism: identical inputs produce identical outputs
// ---------------------------------------------------------------------------

#[test]
fn deterministic_output_for_same_input() {
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

    let r1 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let r2 = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(r1.findings.len(), r2.findings.len());
    for (a, b) in r1.findings.iter().zip(r2.findings.iter()) {
        assert_eq!(a.spdx, b.spdx);
        assert_eq!(a.confidence, b.confidence);
        assert_eq!(a.source_path, b.source_path);
        assert_eq!(a.source_kind, b.source_kind);
    }
    assert_eq!(r1.effective, r2.effective);
}

// ---------------------------------------------------------------------------
// 10. Custom max_file_bytes limit truncates content
// ---------------------------------------------------------------------------

#[test]
fn small_max_file_bytes_may_miss_license_phrases() {
    let dir = tempdir().unwrap();
    // Write a LICENSE where the key phrase appears after many bytes of padding
    let mut content = "x".repeat(100);
    content.push_str("\nPermission is hereby granted, free of charge.");
    fs::write(dir.path().join("LICENSE"), &content).unwrap();

    let mut limits = AnalysisLimits::default();
    // Set limit so small it won't read the license phrase
    limits.max_file_bytes = Some(10);

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &limits).unwrap();

    // With only 10 bytes read, the license phrase is truncated
    assert!(
        report.findings.is_empty(),
        "should not detect license from truncated content"
    );
}

// ---------------------------------------------------------------------------
// 11. Cargo.toml with single-quoted license value
// ---------------------------------------------------------------------------

#[test]
fn cargo_toml_single_quoted_license_value() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = 'x'\nlicense = 'Apache-2.0'\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "Apache-2.0");
}

// ---------------------------------------------------------------------------
// 12. package.json with object license missing "type" yields no finding
// ---------------------------------------------------------------------------

#[test]
fn package_json_license_object_missing_type_yields_no_finding() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "license": {"url": "https://example.com"}}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
}

// ---------------------------------------------------------------------------
// 13. All metadata findings have confidence 0.95
// ---------------------------------------------------------------------------

#[test]
fn all_metadata_findings_have_fixed_confidence() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "license": "ISC"}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"x\"\nlicense = \"BSD-3-Clause\"\n",
    )
    .unwrap();

    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
        PathBuf::from("pyproject.toml"),
    ];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    for f in &report.findings {
        assert!(
            f.source_kind == LicenseSourceKind::Metadata,
            "expected metadata finding"
        );
        let diff = (f.confidence - 0.95_f32).abs();
        assert!(
            diff < f32::EPSILON,
            "metadata confidence should be 0.95, got {}",
            f.confidence
        );
    }
}

// ---------------------------------------------------------------------------
// 14. Text confidence is always in (0.6, 1.0]
// ---------------------------------------------------------------------------

#[test]
fn text_finding_confidence_within_expected_range() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Apache License\nVersion 2.0, January 2004\n\
         http://www.apache.org/licenses/\n\
         limitations under the License.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    for f in &report.findings {
        assert!(
            f.confidence > 0.6 && f.confidence <= 1.0,
            "text confidence {} should be in (0.6, 1.0]",
            f.confidence
        );
    }
}

// ---------------------------------------------------------------------------
// 15. Nested subdirectory metadata paths are normalized
// ---------------------------------------------------------------------------

#[test]
fn nested_metadata_source_path_is_forward_slash_normalized() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("packages").join("foo");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        sub.join("package.json"),
        r#"{"name": "foo", "license": "MIT"}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("packages").join("foo").join("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].source_path, "packages/foo/package.json");
    assert!(!report.findings[0].source_path.contains('\\'));
}
