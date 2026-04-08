//! Deep property-based tests for tokmd-config.
//!
//! Covers: double-roundtrip determinism, variant completeness,
//! serialization format consistency, and rejection of invalid inputs.

use proptest::prelude::*;
use tokmd_config::{
    AnalysisPreset, BadgeMetric, CliAnalysisFormat, CliChildIncludeMode, CliChildrenMode,
    CliConfigMode, CliExportFormat, CliRedactMode, CliTableFormat, ImportGranularity, InitProfile,
    Shell,
};

// =========================================================================
// Double serde roundtrip: serialize -> deserialize -> serialize = same JSON
// =========================================================================

macro_rules! double_roundtrip_test {
    ($name:ident, $type:ty, $variants:expr) => {
        proptest! {
            #[test]
            fn $name(variant in prop::sample::select($variants)) {
                let json1 = serde_json::to_string(&variant).expect("serialize");
                let parsed: $type = serde_json::from_str(&json1).expect("deserialize");
                let json2 = serde_json::to_string(&parsed).expect("re-serialize");
                prop_assert_eq!(&json1, &json2,
                    "Double roundtrip not stable for {:?}", variant);
            }
        }
    };
}

double_roundtrip_test!(
    table_format_double_rt,
    CliTableFormat,
    vec![
        CliTableFormat::Md,
        CliTableFormat::Tsv,
        CliTableFormat::Json
    ]
);
double_roundtrip_test!(
    export_format_double_rt,
    CliExportFormat,
    vec![
        CliExportFormat::Csv,
        CliExportFormat::Jsonl,
        CliExportFormat::Json
    ]
);
double_roundtrip_test!(
    config_mode_double_rt,
    CliConfigMode,
    vec![CliConfigMode::Auto, CliConfigMode::None]
);
double_roundtrip_test!(
    children_mode_double_rt,
    CliChildrenMode,
    vec![CliChildrenMode::Collapse, CliChildrenMode::Separate]
);
double_roundtrip_test!(
    child_include_mode_double_rt,
    CliChildIncludeMode,
    vec![
        CliChildIncludeMode::Separate,
        CliChildIncludeMode::ParentsOnly
    ]
);
double_roundtrip_test!(
    redact_mode_double_rt,
    CliRedactMode,
    vec![
        CliRedactMode::None,
        CliRedactMode::Paths,
        CliRedactMode::All
    ]
);

double_roundtrip_test!(
    analysis_format_double_rt,
    CliAnalysisFormat,
    vec![
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
    ]
);

double_roundtrip_test!(
    analysis_preset_double_rt,
    AnalysisPreset,
    vec![
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
    ]
);

double_roundtrip_test!(
    import_granularity_double_rt,
    ImportGranularity,
    vec![ImportGranularity::Module, ImportGranularity::File]
);
double_roundtrip_test!(
    badge_metric_double_rt,
    BadgeMetric,
    vec![
        BadgeMetric::Lines,
        BadgeMetric::Tokens,
        BadgeMetric::Bytes,
        BadgeMetric::Doc,
        BadgeMetric::Blank,
        BadgeMetric::Hotspot,
    ]
);
double_roundtrip_test!(
    init_profile_double_rt,
    InitProfile,
    vec![
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ]
);
double_roundtrip_test!(
    shell_double_rt,
    Shell,
    vec![
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::Powershell,
        Shell::Zsh
    ]
);

// =========================================================================
// All enum variants serialize to lowercase/kebab-case (no uppercase)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn all_export_formats_lowercase(_dummy in 0..1u8) {
        for fmt in [CliExportFormat::Csv, CliExportFormat::Jsonl, CliExportFormat::Json] {
            let json = serde_json::to_string(&fmt).unwrap();
            let s = json.trim_matches('"');
            prop_assert!(
                !s.chars().any(|c| c.is_uppercase()),
                "CliExportFormat should be lowercase: {}", s
            );
        }
    }

    #[test]
    fn all_badge_metrics_lowercase(_dummy in 0..1u8) {
        for m in [
            BadgeMetric::Lines, BadgeMetric::Tokens, BadgeMetric::Bytes,
            BadgeMetric::Doc, BadgeMetric::Blank, BadgeMetric::Hotspot,
        ] {
            let json = serde_json::to_string(&m).unwrap();
            let s = json.trim_matches('"');
            prop_assert!(
                !s.chars().any(|c| c.is_uppercase()),
                "BadgeMetric should be lowercase: {}", s
            );
        }
    }

    #[test]
    fn all_shells_lowercase(_dummy in 0..1u8) {
        for sh in [Shell::Bash, Shell::Elvish, Shell::Fish, Shell::Powershell, Shell::Zsh] {
            let json = serde_json::to_string(&sh).unwrap();
            let s = json.trim_matches('"');
            prop_assert!(
                !s.chars().any(|c| c.is_uppercase()),
                "Shell should be lowercase: {}", s
            );
        }
    }

    #[test]
    fn all_init_profiles_lowercase(_dummy in 0..1u8) {
        for p in [
            InitProfile::Default, InitProfile::Rust, InitProfile::Node,
            InitProfile::Mono, InitProfile::Python, InitProfile::Go, InitProfile::Cpp,
        ] {
            let json = serde_json::to_string(&p).unwrap();
            let s = json.trim_matches('"');
            prop_assert!(
                !s.chars().any(|c| c.is_uppercase()),
                "InitProfile should be lowercase: {}", s
            );
        }
    }
}

// =========================================================================
// Unknown variant rejection: random strings fail to parse
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn unknown_export_format_rejected(unknown in "[a-z]{6,15}") {
        let known = ["csv", "jsonl", "json"];
        if !known.contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<CliExportFormat, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "'{}' should not parse as CliExportFormat", unknown);
        }
    }

    #[test]
    fn unknown_config_mode_rejected(unknown in "[a-z]{5,15}") {
        let known = ["auto", "none"];
        if !known.contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<CliConfigMode, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "'{}' should not parse as CliConfigMode", unknown);
        }
    }

    #[test]
    fn unknown_redact_mode_rejected(unknown in "[a-z]{5,15}") {
        let known = ["none", "paths", "all"];
        if !known.contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<CliRedactMode, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "'{}' should not parse as CliRedactMode", unknown);
        }
    }

    #[test]
    fn unknown_shell_rejected(unknown in "[a-z]{5,15}") {
        let known = ["bash", "elvish", "fish", "powershell", "zsh"];
        if !known.contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<Shell, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "'{}' should not parse as Shell", unknown);
        }
    }
}
