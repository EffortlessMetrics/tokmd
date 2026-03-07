//! Deep property-based tests for tokmd-config.
//!
//! Covers: double-roundtrip determinism, variant completeness,
//! serialization format consistency, and rejection of invalid inputs.

use proptest::prelude::*;
use tokmd_config::{
    AnalysisFormat, AnalysisPreset, BadgeMetric, ChildIncludeMode, ChildrenMode, ConfigMode,
    ExportFormat, ImportGranularity, InitProfile, RedactMode, Shell, TableFormat,
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
    TableFormat,
    vec![TableFormat::Md, TableFormat::Tsv, TableFormat::Json]
);
double_roundtrip_test!(
    export_format_double_rt,
    ExportFormat,
    vec![ExportFormat::Csv, ExportFormat::Jsonl, ExportFormat::Json]
);
double_roundtrip_test!(
    config_mode_double_rt,
    ConfigMode,
    vec![ConfigMode::Auto, ConfigMode::None]
);
double_roundtrip_test!(
    children_mode_double_rt,
    ChildrenMode,
    vec![ChildrenMode::Collapse, ChildrenMode::Separate]
);
double_roundtrip_test!(
    child_include_mode_double_rt,
    ChildIncludeMode,
    vec![ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly]
);
double_roundtrip_test!(
    redact_mode_double_rt,
    RedactMode,
    vec![RedactMode::None, RedactMode::Paths, RedactMode::All]
);

double_roundtrip_test!(
    analysis_format_double_rt,
    AnalysisFormat,
    vec![
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
        AnalysisPreset::Estimate,
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
        for fmt in [ExportFormat::Csv, ExportFormat::Jsonl, ExportFormat::Json] {
            let json = serde_json::to_string(&fmt).unwrap();
            let s = json.trim_matches('"');
            prop_assert!(
                !s.chars().any(|c| c.is_uppercase()),
                "ExportFormat should be lowercase: {}", s
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
            let result: Result<ExportFormat, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "'{}' should not parse as ExportFormat", unknown);
        }
    }

    #[test]
    fn unknown_config_mode_rejected(unknown in "[a-z]{5,15}") {
        let known = ["auto", "none"];
        if !known.contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<ConfigMode, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "'{}' should not parse as ConfigMode", unknown);
        }
    }

    #[test]
    fn unknown_redact_mode_rejected(unknown in "[a-z]{5,15}") {
        let known = ["none", "paths", "all"];
        if !known.contains(&unknown.as_str()) {
            let json = format!("\"{}\"", unknown);
            let result: Result<RedactMode, _> = serde_json::from_str(&json);
            prop_assert!(result.is_err(), "'{}' should not parse as RedactMode", unknown);
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
