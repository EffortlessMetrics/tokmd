//! Property-based tests for tokmd-config serialization.
//!
//! These tests verify that all config enums round-trip correctly through JSON.

use proptest::prelude::*;
use tokmd_config::{
    CliAnalysisFormat, AnalysisPreset, BadgeMetric, CliChildIncludeMode, CliChildrenMode, CockpitFormat,
    CliConfigMode, CliExportFormat, HandoffPreset, ImportGranularity, InitProfile, CliRedactMode, Shell,
    CliTableFormat,
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

// All variants for each enum
const TABLE_FORMATS: [CliTableFormat; 3] = [CliTableFormat::Md, CliTableFormat::Tsv, CliTableFormat::Json];

const EXPORT_FORMATS: [CliExportFormat; 3] =
    [CliExportFormat::Csv, CliExportFormat::Jsonl, CliExportFormat::Json];

const CONFIG_MODES: [CliConfigMode; 2] = [CliConfigMode::Auto, CliConfigMode::None];

const CHILDREN_MODES: [CliChildrenMode; 2] = [CliChildrenMode::Collapse, CliChildrenMode::Separate];

const CHILD_INCLUDE_MODES: [CliChildIncludeMode; 2] =
    [CliChildIncludeMode::Separate, CliChildIncludeMode::ParentsOnly];

const REDACT_MODES: [CliRedactMode; 3] = [CliRedactMode::None, CliRedactMode::Paths, CliRedactMode::All];

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

// Generate round-trip tests
roundtrip_test!(table_format_roundtrip, CliTableFormat, TABLE_FORMATS.to_vec());
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

// Additional property tests for serialization format consistency

proptest! {
    #[test]
    fn table_format_kebab_case(_dummy in 0..1u8) {
        // All CliTableFormat variants should serialize to kebab-case
        for variant in TABLE_FORMATS {
            let json = serde_json::to_string(&variant).expect("serialize");
            let json_str = json.trim_matches('"');
            prop_assert!(
                !json_str.contains('_') && !json_str.chars().any(|c| c.is_uppercase()),
                "Expected kebab-case, got: {}",
                json_str
            );
        }
    }

    #[test]
    fn analysis_preset_kebab_case(_dummy in 0..1u8) {
        for variant in ANALYSIS_PRESETS {
            let json = serde_json::to_string(&variant).expect("serialize");
            let json_str = json.trim_matches('"');
            prop_assert!(
                !json_str.contains('_') && !json_str.chars().any(|c| c.is_uppercase()),
                "Expected kebab-case, got: {}",
                json_str
            );
        }
    }

    #[test]
    fn redact_mode_kebab_case(_dummy in 0..1u8) {
        for variant in REDACT_MODES {
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

// Test that unknown variants fail gracefully
proptest! {
    #[test]
    fn unknown_table_format_fails(unknown in "[a-z]{5,10}") {
        // Ensure random strings don't parse as valid formats (unless they happen to match)
        if !["md", "tsv", "json"].contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<CliTableFormat, _> = serde_json::from_str(&json);
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

    // NEW property tests

    #[test]
    fn cockpit_format_serde_rt(
        variant in prop::sample::select(vec![
            CockpitFormat::Json, CockpitFormat::Md, CockpitFormat::Sections,
        ])
    ) {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CockpitFormat = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(variant, back);
    }

    #[test]
    fn handoff_preset_serde_rt(
        variant in prop::sample::select(vec![
            HandoffPreset::Minimal, HandoffPreset::Standard,
            HandoffPreset::Risk, HandoffPreset::Deep,
        ])
    ) {
        let json = serde_json::to_string(&variant).unwrap();
        let back: HandoffPreset = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(variant, back);
    }

    #[test]
    fn config_mode_serde_roundtrip(
        variant in prop::sample::select(vec![CliConfigMode::Auto, CliConfigMode::None])
    ) {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CliConfigMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(variant, back);
    }

    #[test]
    fn cockpit_format_display(
        variant in prop::sample::select(vec![
            CockpitFormat::Json, CockpitFormat::Md, CockpitFormat::Sections,
        ])
    ) {
        let s = format!("{:?}", variant);
        prop_assert!(!s.is_empty());
    }

    #[test]
    fn handoff_preset_debug(
        variant in prop::sample::select(vec![
            HandoffPreset::Minimal, HandoffPreset::Standard,
            HandoffPreset::Risk, HandoffPreset::Deep,
        ])
    ) {
        let s = format!("{:?}", variant);
        prop_assert!(!s.is_empty());
    }

}
