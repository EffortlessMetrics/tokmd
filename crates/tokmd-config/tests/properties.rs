//! Property-based tests for tokmd-config serialization.
//!
//! These tests verify that all config enums round-trip correctly through JSON.

use proptest::prelude::*;
use tokmd_config::{
    AnalysisFormat, AnalysisPreset, BadgeMetric, ChildIncludeMode, ChildrenMode, ConfigMode,
    ExportFormat, ImportGranularity, InitProfile, RedactMode, Shell, TableFormat,
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
const TABLE_FORMATS: [TableFormat; 3] = [TableFormat::Md, TableFormat::Tsv, TableFormat::Json];

const EXPORT_FORMATS: [ExportFormat; 3] =
    [ExportFormat::Csv, ExportFormat::Jsonl, ExportFormat::Json];

const CONFIG_MODES: [ConfigMode; 2] = [ConfigMode::Auto, ConfigMode::None];

const CHILDREN_MODES: [ChildrenMode; 2] = [ChildrenMode::Collapse, ChildrenMode::Separate];

const CHILD_INCLUDE_MODES: [ChildIncludeMode; 2] =
    [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly];

const REDACT_MODES: [RedactMode; 3] = [RedactMode::None, RedactMode::Paths, RedactMode::All];

const ANALYSIS_FORMATS: [AnalysisFormat; 10] = [
    AnalysisFormat::Md,
    AnalysisFormat::Json,
    AnalysisFormat::Jsonld,
    AnalysisFormat::Xml,
    AnalysisFormat::Svg,
    AnalysisFormat::Mermaid,
    AnalysisFormat::Obj,
    AnalysisFormat::Midi,
    AnalysisFormat::Tree,
    AnalysisFormat::Html,
];

const ANALYSIS_PRESETS: [AnalysisPreset; 11] = [
    AnalysisPreset::Receipt,
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
roundtrip_test!(table_format_roundtrip, TableFormat, TABLE_FORMATS.to_vec());
roundtrip_test!(
    export_format_roundtrip,
    ExportFormat,
    EXPORT_FORMATS.to_vec()
);
roundtrip_test!(config_mode_roundtrip, ConfigMode, CONFIG_MODES.to_vec());
roundtrip_test!(
    children_mode_roundtrip,
    ChildrenMode,
    CHILDREN_MODES.to_vec()
);
roundtrip_test!(
    child_include_mode_roundtrip,
    ChildIncludeMode,
    CHILD_INCLUDE_MODES.to_vec()
);
roundtrip_test!(redact_mode_roundtrip, RedactMode, REDACT_MODES.to_vec());
roundtrip_test!(
    analysis_format_roundtrip,
    AnalysisFormat,
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
        // All TableFormat variants should serialize to kebab-case
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
            let result: Result<TableFormat, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "Unknown format '{}' should fail to parse", unknown);
        }
    }

    #[test]
    fn unknown_analysis_preset_fails(unknown in "[a-z]{5,15}") {
        let known = [
            "receipt", "health", "risk", "supply", "architecture",
            "topics", "security", "identity", "git", "deep", "fun"
        ];
        if !known.contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<AnalysisPreset, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "Unknown preset '{}' should fail to parse", unknown);
        }
    }
}
