//! Property-based tests for tokmd-config serialization.
//!
//! These tests verify that all config enums round-trip correctly through JSON.

use proptest::prelude::*;
use tokmd_config::{
    AnalysisPreset, BadgeMetric, CliAnalysisFormat, CliChildIncludeMode, CliChildrenMode,
    CliConfigMode, CliExportFormat, CliRedactMode, CliTableFormat, CockpitFormat, HandoffPreset,
    ImportGranularity, InitProfile, Shell,
};

/// Macro to generate round-trip tests for enums that implement Serialize + Deserialize + PartialEq
macro_rules! roundtrip_test {
    ($name:ident, $type:ty, $variants:expr) => {
        proptest! {
            #[test]
            fn $name(variant in prop::sample::select($variants)) {
                let json = serde_json::to_string(&variant).expect("serialize");
                let parsed: $type = serde_json::from_str(&json).expect("deserialize");
                prop_assert_eq!(variant, parsed, "Round-trip failed for {:?}", variant);
            }
        }
    };
}

/// Macro to ensure that enums serialize to kebab-case
macro_rules! kebab_case_test {
    ($name:ident, $variants:expr) => {
        proptest! {
            #[test]
            fn $name(_dummy in 0..1u8) {
                for variant in $variants {
                    let json = serde_json::to_string(&variant).expect("serialize");
                    let json_str = json.trim_matches('"');
                    prop_assert!(
                        !json_str.contains('_') && !json_str.chars().any(|c| c.is_uppercase()),
                        "Expected kebab-case, got: {}",
                        json_str
                    );
                }
            }
        }
    };
}

// All variants for each enum
const TABLE_FORMATS: [CliTableFormat; 3] = [
    CliTableFormat::Md,
    CliTableFormat::Tsv,
    CliTableFormat::Json,
];

const EXPORT_FORMATS: [CliExportFormat; 4] = [
    CliExportFormat::Csv,
    CliExportFormat::Jsonl,
    CliExportFormat::Json,
    CliExportFormat::Cyclonedx,
];

const CONFIG_MODES: [CliConfigMode; 2] = [CliConfigMode::Auto, CliConfigMode::None];

const CHILDREN_MODES: [CliChildrenMode; 2] = [CliChildrenMode::Collapse, CliChildrenMode::Separate];

const CHILD_INCLUDE_MODES: [CliChildIncludeMode; 2] = [
    CliChildIncludeMode::Separate,
    CliChildIncludeMode::ParentsOnly,
];

const REDACT_MODES: [CliRedactMode; 3] = [
    CliRedactMode::None,
    CliRedactMode::Paths,
    CliRedactMode::All,
];

const ANALYSIS_FORMATS: [CliAnalysisFormat; 10] = [
    CliAnalysisFormat::Md,
    CliAnalysisFormat::Json,
    CliAnalysisFormat::Jsonld,
    CliAnalysisFormat::Xml,
    CliAnalysisFormat::Svg,
    CliAnalysisFormat::Mermaid,
    CliAnalysisFormat::Obj,
    CliAnalysisFormat::Midi,
    CliAnalysisFormat::Tree,
    CliAnalysisFormat::Html,
];

const ANALYSIS_PRESETS: [AnalysisPreset; 12] = [
    AnalysisPreset::Receipt,
    AnalysisPreset::Estimate,
    AnalysisPreset::Health,
    AnalysisPreset::Risk,
    AnalysisPreset::Supply,
    AnalysisPreset::Architecture,
    AnalysisPreset::Topics,
    AnalysisPreset::Security,
    AnalysisPreset::Identity,
    AnalysisPreset::Git,
    AnalysisPreset::Deep,
    AnalysisPreset::Fun,
];

const IMPORT_GRANULARITIES: [ImportGranularity; 2] =
    [ImportGranularity::Module, ImportGranularity::File];

const BADGE_METRICS: [BadgeMetric; 6] = [
    BadgeMetric::Lines,
    BadgeMetric::Tokens,
    BadgeMetric::Bytes,
    BadgeMetric::Doc,
    BadgeMetric::Blank,
    BadgeMetric::Hotspot,
];

const INIT_PROFILES: [InitProfile; 7] = [
    InitProfile::Default,
    InitProfile::Rust,
    InitProfile::Node,
    InitProfile::Mono,
    InitProfile::Python,
    InitProfile::Go,
    InitProfile::Cpp,
];

const SHELLS: [Shell; 5] = [
    Shell::Bash,
    Shell::Elvish,
    Shell::Fish,
    Shell::Powershell,
    Shell::Zsh,
];

const COCKPIT_FORMATS: [CockpitFormat; 3] = [
    CockpitFormat::Json,
    CockpitFormat::Md,
    CockpitFormat::Sections,
];

const HANDOFF_PRESETS: [HandoffPreset; 4] = [
    HandoffPreset::Minimal,
    HandoffPreset::Standard,
    HandoffPreset::Risk,
    HandoffPreset::Deep,
];

// Generate round-trip tests
roundtrip_test!(
    table_format_roundtrip,
    CliTableFormat,
    TABLE_FORMATS.to_vec()
);
roundtrip_test!(
    export_format_roundtrip,
    CliExportFormat,
    EXPORT_FORMATS.to_vec()
);
roundtrip_test!(config_mode_roundtrip, CliConfigMode, CONFIG_MODES.to_vec());
roundtrip_test!(
    children_mode_roundtrip,
    CliChildrenMode,
    CHILDREN_MODES.to_vec()
);
roundtrip_test!(
    child_include_mode_roundtrip,
    CliChildIncludeMode,
    CHILD_INCLUDE_MODES.to_vec()
);
roundtrip_test!(redact_mode_roundtrip, CliRedactMode, REDACT_MODES.to_vec());
roundtrip_test!(
    analysis_format_roundtrip,
    CliAnalysisFormat,
    ANALYSIS_FORMATS.to_vec()
);
roundtrip_test!(
    analysis_preset_roundtrip,
    AnalysisPreset,
    ANALYSIS_PRESETS.to_vec()
);
roundtrip_test!(
    import_granularity_roundtrip,
    ImportGranularity,
    IMPORT_GRANULARITIES.to_vec()
);
roundtrip_test!(badge_metric_roundtrip, BadgeMetric, BADGE_METRICS.to_vec());
roundtrip_test!(init_profile_roundtrip, InitProfile, INIT_PROFILES.to_vec());
roundtrip_test!(shell_roundtrip, Shell, SHELLS.to_vec());
roundtrip_test!(
    cockpit_format_serde_rt,
    CockpitFormat,
    COCKPIT_FORMATS.to_vec()
);
roundtrip_test!(
    handoff_preset_serde_rt,
    HandoffPreset,
    HANDOFF_PRESETS.to_vec()
);

// Generate kebab-case tests
kebab_case_test!(table_format_kebab_case, TABLE_FORMATS);
kebab_case_test!(export_format_kebab_case, EXPORT_FORMATS);
kebab_case_test!(config_mode_kebab_case, CONFIG_MODES);
kebab_case_test!(children_mode_kebab_case, CHILDREN_MODES);
kebab_case_test!(child_include_mode_kebab_case, CHILD_INCLUDE_MODES);
kebab_case_test!(redact_mode_kebab_case, REDACT_MODES);
kebab_case_test!(analysis_format_kebab_case, ANALYSIS_FORMATS);
kebab_case_test!(analysis_preset_kebab_case, ANALYSIS_PRESETS);
kebab_case_test!(import_granularity_kebab_case, IMPORT_GRANULARITIES);
kebab_case_test!(badge_metric_kebab_case, BADGE_METRICS);
kebab_case_test!(init_profile_kebab_case, INIT_PROFILES);
kebab_case_test!(shell_kebab_case, SHELLS);
kebab_case_test!(cockpit_format_kebab_case, COCKPIT_FORMATS);
kebab_case_test!(handoff_preset_kebab_case, HANDOFF_PRESETS);

// Test that unknown variants fail gracefully
proptest! {
    #[test]
    fn unknown_table_format_fails(unknown in "[a-z]{5,10}") {
        if !["md", "tsv", "json"].contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<CliTableFormat, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "Unknown format '{}' should fail to parse", unknown);
        }
    }

    #[test]
    fn unknown_export_format_fails(unknown in "[a-z]{5,10}") {
        if !["csv", "jsonl", "json", "cyclonedx"].contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<CliExportFormat, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "Unknown format '{}' should fail to parse", unknown);
        }
    }

    #[test]
    fn unknown_analysis_preset_fails(unknown in "[a-z]{5,15}") {
        let known = [
            "receipt", "estimate", "health", "risk", "supply", "architecture",
            "topics", "security", "identity", "git", "deep", "fun"
        ];
        if !known.contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<AnalysisPreset, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "Unknown preset '{}' should fail to parse", unknown);
        }
    }
}

// Ensure non-string variants display correctly
proptest! {
    #[test]
    fn cockpit_format_display(
        variant in prop::sample::select(COCKPIT_FORMATS.to_vec())
    ) {
        let s = format!("{:?}", variant);
        prop_assert!(!s.is_empty());
    }

    #[test]
    fn handoff_preset_debug(
        variant in prop::sample::select(HANDOFF_PRESETS.to_vec())
    ) {
        let s = format!("{:?}", variant);
        prop_assert!(!s.is_empty());
    }
}
