use super::*;
use std::path::PathBuf;
use tokmd_settings::ExportSettings;
use tokmd_settings::ScanOptions;
use tokmd_types::ExportData;
use tokmd_types::RedactMode;

// Helper to create minimal ExportData
fn empty_export_data() -> ExportData {
    ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 3,
        children: tokmd_types::ChildIncludeMode::Separate,
    }
}

// Helper to create minimal ScanOptions
fn minimal_scan_opts() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: tokmd_types::ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

// Helper to create ExportSettings with specific redact/strip_prefix
fn export_settings(redact: RedactMode, strip_prefix: Option<String>) -> ExportSettings {
    ExportSettings {
        format: tokmd_settings::ExportFormat::Json,
        module_roots: vec![],
        module_depth: 3,
        children: tokmd_types::ChildIncludeMode::Separate,
        min_code: 1,
        max_rows: 1000,
        redact,
        meta: true,
        strip_prefix,
    }
}

// =============================================================================
// parse_analysis_preset — Kill 9/12 untested match arms
// =============================================================================

#[test]
#[cfg(feature = "analysis")]
fn parse_analysis_preset_all_twelve_variants() {
    #[cfg(feature = "analysis")]
    use tokmd_analysis::AnalysisPreset;

    let variants = [
        ("receipt", AnalysisPreset::Receipt),
        ("estimate", AnalysisPreset::Estimate),
        ("health", AnalysisPreset::Health),
        ("risk", AnalysisPreset::Risk),
        ("supply", AnalysisPreset::Supply),
        ("architecture", AnalysisPreset::Architecture),
        ("topics", AnalysisPreset::Topics),
        ("security", AnalysisPreset::Security),
        ("identity", AnalysisPreset::Identity),
        ("git", AnalysisPreset::Git),
        ("deep", AnalysisPreset::Deep),
        ("fun", AnalysisPreset::Fun),
    ];

    for (input, expected) in &variants {
        // Test exact lowercase
        let (preset, normalized) = parse_analysis_preset(input).unwrap();
        assert_eq!(preset, *expected, "Exact match failed for: {}", input);
        assert_eq!(normalized, *input, "Normalization failed for: {}", input);

        // Test uppercase (normalization)
        let upper = input.to_uppercase();
        let (preset, normalized) = parse_analysis_preset(&upper).unwrap();
        assert_eq!(preset, *expected, "Uppercase match failed for: {}", upper);
        assert_eq!(
            normalized, *input,
            "Uppercase normalization failed for: {}",
            upper
        );

        // Test mixed case with whitespace (normalization)
        let mixed = format!("  {}  ", input);
        let (preset, normalized) = parse_analysis_preset(&mixed).unwrap();
        assert_eq!(preset, *expected, "Mixed case match failed for: {}", mixed);
        assert_eq!(
            normalized, *input,
            "Mixed case normalization failed for: {}",
            mixed
        );
    }
}

#[test]
#[cfg(feature = "analysis")]
fn parse_analysis_preset_invalid_variants_fail() {
    let invalid = [
        "unknown",
        "invalid",
        "",
        "receipts",         // typo
        "healthh",          // typo
        "ARCH",             // partial match
        "receipt_estimate", // combined
    ];

    for input in &invalid {
        assert!(
            parse_analysis_preset(input).is_err(),
            "Should fail for invalid input: {}",
            input
        );
    }
}

// =============================================================================
// build_export_receipt — Kill && → || mutation on strip_prefix_redacted
// =============================================================================

#[test]
fn build_export_receipt_redact_paths_with_strip_prefix() {
    let settings = export_settings(RedactMode::Paths, Some("/project".to_string()));
    let data = empty_export_data();
    let paths = vec![PathBuf::from("/project/src/main.rs")];

    let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

    // strip_prefix_redacted = should_redact && strip_prefix.is_some()
    // = true && true = true
    assert!(
        receipt.args.strip_prefix_redacted,
        "strip_prefix_redacted should be true when redact=Paths and strip_prefix=Some"
    );
}

#[test]
fn build_export_receipt_redact_paths_without_strip_prefix() {
    let settings = export_settings(RedactMode::Paths, None);
    let data = empty_export_data();
    let paths = vec![PathBuf::from("/project/src/main.rs")];

    let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

    // strip_prefix_redacted = should_redact && strip_prefix.is_some()
    // = true && false = false
    // This kills the && → || mutation (|| would give true)
    assert!(
        !receipt.args.strip_prefix_redacted,
        "strip_prefix_redacted should be false when strip_prefix=None (kills &&→||)"
    );
}

#[test]
fn build_export_receipt_no_redact_with_strip_prefix() {
    let settings = export_settings(RedactMode::None, Some("/project".to_string()));
    let data = empty_export_data();
    let paths = vec![PathBuf::from("/project/src/main.rs")];

    let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

    // strip_prefix_redacted = should_redact && strip_prefix.is_some()
    // = false && true = false
    assert!(
        !receipt.args.strip_prefix_redacted,
        "strip_prefix_redacted should be false when redact=None"
    );
}

#[test]
fn build_export_receipt_redact_all_with_strip_prefix() {
    let settings = export_settings(RedactMode::All, Some("/project".to_string()));
    let data = empty_export_data();
    let paths = vec![PathBuf::from("/project/src/main.rs")];

    let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

    // strip_prefix_redacted = should_redact && strip_prefix.is_some()
    // = true && true = true (All also triggers should_redact)
    assert!(
        receipt.args.strip_prefix_redacted,
        "strip_prefix_redacted should be true when redact=All and strip_prefix=Some"
    );
}

#[test]
fn build_export_receipt_redact_all_without_strip_prefix() {
    let settings = export_settings(RedactMode::All, None);
    let data = empty_export_data();
    let paths = vec![PathBuf::from("/project/src/main.rs")];

    let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

    // strip_prefix_redacted = should_redact && strip_prefix.is_some()
    // = true && false = false
    // This kills the && → || mutation
    assert!(
        !receipt.args.strip_prefix_redacted,
        "strip_prefix_redacted should be false when strip_prefix=None (kills &&→||)"
    );
}

#[test]
fn build_export_receipt_strip_prefix_redaction_logic() {
    // Test the ternary logic: strip_prefix redaction in ExportArgsMeta
    // Kills mutations that change the if/else logic on strip_prefix

    // Case 1: redact=Paths → strip_prefix should be redacted
    let settings = export_settings(RedactMode::Paths, Some("/project".to_string()));
    let data = empty_export_data();
    let paths = vec![PathBuf::from("/project/src/main.rs")];
    let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

    // When redacted, strip_prefix should be transformed (not the original)
    assert!(receipt.args.strip_prefix.is_some());
    assert_ne!(
        receipt.args.strip_prefix,
        Some("/project".to_string()),
        "strip_prefix should be redacted/transformed when redact=Paths"
    );

    // Case 2: redact=None → strip_prefix should pass through unchanged
    let settings = export_settings(RedactMode::None, Some("/project".to_string()));
    let data = empty_export_data();
    let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

    assert_eq!(
        receipt.args.strip_prefix,
        Some("/project".to_string()),
        "strip_prefix should pass through unchanged when redact=None"
    );

    // Case 3: redact=All → strip_prefix should be redacted
    let settings = export_settings(RedactMode::All, Some("/project".to_string()));
    let data = empty_export_data();
    let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

    assert!(receipt.args.strip_prefix.is_some());
    assert_ne!(
        receipt.args.strip_prefix,
        Some("/project".to_string()),
        "strip_prefix should be redacted when redact=All"
    );
}

#[test]
#[cfg(feature = "analysis")]
fn parse_analysis_preset_normalization_edge_cases() {
    // Kills mutations that remove .trim() or .to_ascii_lowercase()

    // Test trim removal
    let (preset, _) = parse_analysis_preset("  receipt  ").unwrap();
    assert_eq!(
        preset,
        tokmd_analysis::AnalysisPreset::Receipt,
        "Leading/trailing whitespace should be trimmed"
    );

    let (preset, _) = parse_analysis_preset("\tHEALTH\n").unwrap();
    assert_eq!(
        preset,
        tokmd_analysis::AnalysisPreset::Health,
        "Tabs and newlines should be trimmed, case normalized"
    );

    // Test to_ascii_lowercase removal
    let (preset, _) = parse_analysis_preset("ReCeIpT").unwrap();
    assert_eq!(
        preset,
        tokmd_analysis::AnalysisPreset::Receipt,
        "Mixed case should be normalized to lowercase"
    );

    let (preset, _) = parse_analysis_preset("ESTIMATE").unwrap();
    assert_eq!(
        preset,
        tokmd_analysis::AnalysisPreset::Estimate,
        "Uppercase should be normalized"
    );

    // Test combined trim + lowercase
    let (preset, normalized) = parse_analysis_preset("  DeEp  ").unwrap();
    assert_eq!(preset, tokmd_analysis::AnalysisPreset::Deep);
    assert_eq!(normalized, "deep", "Should be trimmed and lowercased");
}

// =============================================================================
// cockpit_workflow — Kill boolean logic mutations (requires git + cockpit feature)
// =============================================================================

#[cfg(feature = "cockpit")]
#[test]
fn cockpit_workflow_range_mode_parsing() {
    assert!(matches!(
        parse_cockpit_range_mode("three-dot").expect("three-dot should parse"),
        tokmd_git::GitRangeMode::ThreeDot
    ));
    assert!(matches!(
        parse_cockpit_range_mode("3dot").expect("3dot should parse"),
        tokmd_git::GitRangeMode::ThreeDot
    ));
    assert!(matches!(
        parse_cockpit_range_mode("two-dot").expect("two-dot should parse"),
        tokmd_git::GitRangeMode::TwoDot
    ));
    assert!(matches!(
        parse_cockpit_range_mode("2dot").expect("2dot should parse"),
        tokmd_git::GitRangeMode::TwoDot
    ));
    assert!(matches!(
        parse_cockpit_range_mode("  THREE-DOT  ").expect("trimmed/case-insensitive parse"),
        tokmd_git::GitRangeMode::ThreeDot
    ));
}

#[cfg(feature = "cockpit")]
#[test]
fn cockpit_workflow_range_mode_invalid_rejected() {
    let err = parse_cockpit_range_mode("invalid").expect_err("invalid mode should fail");
    let msg = err.to_string();
    assert!(
        msg.contains("range_mode"),
        "Error should reference range_mode field; got: {msg}"
    );
}
