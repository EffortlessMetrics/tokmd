//! W70: Comprehensive serde roundtrip tests for tokmd-config types.
//!
//! Validates TOML and JSON roundtrip for configuration types,
//! profile serialization, BTreeMap ordering, and deterministic output.

use std::collections::BTreeMap;

use proptest::prelude::*;
use serde_json;
use tokmd_config::{
    AnalysisPreset, BadgeMetric, CockpitFormat, ColorMode, ContextOutput, ContextStrategy,
    DiffFormat, DiffRangeMode, GateFormat, HandoffPreset, ImportGranularity, InitProfile,
    NearDupScope, Profile, SensorFormat, UserConfig, ValueMetric,
};
use tokmd_settings::{
    AnalyzeConfig, BadgeConfig, ContextConfig, ExportConfig, GateConfig, GateRule, ModuleConfig,
    RatchetRuleConfig, ScanConfig, TomlConfig, ViewProfile,
};

// ─── 1. UserConfig JSON roundtrip ───────────────────────────────────────────

#[test]
fn user_config_empty_roundtrip() {
    let config = UserConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let back: UserConfig = serde_json::from_str(&json).unwrap();
    assert!(back.profiles.is_empty());
    assert!(back.repos.is_empty());
}

// ─── 2. UserConfig with profiles roundtrip ──────────────────────────────────

#[test]
fn user_config_with_profiles_roundtrip() {
    let mut config = UserConfig::default();
    config.profiles.insert(
        "llm_safe".to_string(),
        Profile {
            format: Some("json".to_string()),
            top: Some(10),
            files: Some(true),
            module_roots: Some(vec!["crates".to_string(), "packages".to_string()]),
            module_depth: Some(2),
            min_code: Some(5),
            max_rows: Some(100),
            redact: Some(tokmd_config::RedactMode::Paths),
            meta: Some(true),
            children: Some("collapse".to_string()),
        },
    );
    config
        .repos
        .insert("owner/repo".to_string(), "llm_safe".to_string());

    let json = serde_json::to_string(&config).unwrap();
    let back: UserConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.profiles.len(), 1);
    let p = &back.profiles["llm_safe"];
    assert_eq!(p.format.as_deref(), Some("json"));
    assert_eq!(p.top, Some(10));
    assert_eq!(p.module_depth, Some(2));
    assert_eq!(back.repos["owner/repo"], "llm_safe");
}

// ─── 3. Profile with all None fields roundtrip ──────────────────────────────

#[test]
fn profile_default_roundtrip() {
    let profile = Profile::default();
    let json = serde_json::to_string(&profile).unwrap();
    let back: Profile = serde_json::from_str(&json).unwrap();
    assert!(back.format.is_none());
    assert!(back.top.is_none());
    assert!(back.files.is_none());
    assert!(back.module_roots.is_none());
    assert!(back.redact.is_none());
}

// ─── 4. TomlConfig roundtrip (TOML) ────────────────────────────────────────

#[test]
fn toml_config_toml_roundtrip() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target", "node_modules"]
hidden = true

[module]
roots = ["crates", "packages"]
depth = 3

[export]
min_code = 10
max_rows = 500
format = "csv"

[analyze]
preset = "risk"
window = 128000

[context]
budget = "256k"
strategy = "spread"

[badge]
metric = "tokens"

[gate]
fail_fast = true
"#;
    let config: TomlConfig = toml::from_str(toml_str).unwrap();
    let serialized = toml::to_string(&config).unwrap();
    let back: TomlConfig = toml::from_str(&serialized).unwrap();

    assert_eq!(
        back.scan.paths.as_deref(),
        Some(&["src".to_string(), "lib".to_string()][..])
    );
    assert_eq!(back.module.depth, Some(3));
    assert_eq!(back.export.min_code, Some(10));
    assert_eq!(back.analyze.preset.as_deref(), Some("risk"));
    assert_eq!(back.context.budget.as_deref(), Some("256k"));
    assert_eq!(back.badge.metric.as_deref(), Some("tokens"));
    assert_eq!(back.gate.fail_fast, Some(true));
}

// ─── 5. TomlConfig JSON roundtrip ───────────────────────────────────────────

#[test]
fn toml_config_json_roundtrip() {
    let config = TomlConfig {
        scan: ScanConfig {
            paths: Some(vec![".".to_string()]),
            exclude: Some(vec!["vendor".to_string()]),
            hidden: Some(false),
            ..Default::default()
        },
        module: ModuleConfig {
            roots: Some(vec!["crates".to_string()]),
            depth: Some(2),
            children: Some("collapse".to_string()),
        },
        export: ExportConfig {
            min_code: Some(1),
            max_rows: Some(200),
            format: Some("jsonl".to_string()),
            ..Default::default()
        },
        analyze: AnalyzeConfig {
            preset: Some("health".to_string()),
            window: Some(64000),
            ..Default::default()
        },
        context: ContextConfig {
            budget: Some("128k".to_string()),
            strategy: Some("greedy".to_string()),
            ..Default::default()
        },
        badge: BadgeConfig {
            metric: Some("lines".to_string()),
        },
        gate: GateConfig::default(),
        view: BTreeMap::new(),
    };

    let json = serde_json::to_string(&config).unwrap();
    let back: TomlConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scan.paths.as_deref(), Some(&[".".to_string()][..]));
    assert_eq!(
        back.module.roots.as_deref(),
        Some(&["crates".to_string()][..])
    );
    assert_eq!(back.analyze.preset.as_deref(), Some("health"));
}

// ─── 6. GateConfig with rules roundtrip ─────────────────────────────────────

#[test]
fn gate_config_with_rules_roundtrip() {
    let config = GateConfig {
        policy: Some("policy.toml".to_string()),
        baseline: Some("baseline.json".to_string()),
        preset: Some("risk".to_string()),
        fail_fast: Some(true),
        rules: Some(vec![GateRule {
            name: "max-complexity".to_string(),
            pointer: "/complexity/avg_cyclomatic".to_string(),
            op: "<=".to_string(),
            value: Some(serde_json::json!(15.0)),
            values: None,
            negate: false,
            level: Some("error".to_string()),
            message: Some("Complexity too high".to_string()),
        }]),
        ratchet: Some(vec![RatchetRuleConfig {
            pointer: "/complexity/avg_cyclomatic".to_string(),
            max_increase_pct: Some(10.0),
            max_value: Some(20.0),
            level: Some("warn".to_string()),
            description: Some("Cyclomatic complexity limit".to_string()),
        }]),
        allow_missing_baseline: Some(true),
        allow_missing_current: Some(false),
    };

    let json = serde_json::to_string(&config).unwrap();
    let back: GateConfig = serde_json::from_str(&json).unwrap();
    let rules = back.rules.unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "max-complexity");
    assert_eq!(rules[0].op, "<=");
    let ratchet = back.ratchet.unwrap();
    assert_eq!(ratchet[0].max_increase_pct, Some(10.0));
}

// ─── 7. ViewProfile roundtrip ───────────────────────────────────────────────

#[test]
fn view_profile_roundtrip() {
    let profile = ViewProfile {
        format: Some("json".to_string()),
        top: Some(20),
        files: Some(true),
        module_roots: Some(vec!["packages".to_string()]),
        module_depth: Some(3),
        min_code: Some(10),
        max_rows: Some(50),
        redact: Some("paths".to_string()),
        meta: Some(true),
        children: Some("separate".to_string()),
        preset: Some("deep".to_string()),
        window: Some(200000),
        budget: Some("1m".to_string()),
        strategy: Some("spread".to_string()),
        rank_by: Some("hotspot".to_string()),
        output: Some("json".to_string()),
        compress: Some(true),
        metric: Some("tokens".to_string()),
    };

    let json = serde_json::to_string(&profile).unwrap();
    let back: ViewProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format.as_deref(), Some("json"));
    assert_eq!(back.top, Some(20));
    assert_eq!(back.budget.as_deref(), Some("1m"));
    assert_eq!(back.strategy.as_deref(), Some("spread"));
}

// ─── 8. BTreeMap ordering in serialized output ──────────────────────────────

#[test]
fn btreemap_ordering_in_views() {
    let mut views = BTreeMap::new();
    views.insert("ci".to_string(), ViewProfile::default());
    views.insert("alpha".to_string(), ViewProfile::default());
    views.insert("zebra".to_string(), ViewProfile::default());

    let config = TomlConfig {
        view: views,
        ..Default::default()
    };

    let json = serde_json::to_string(&config).unwrap();
    let alpha_pos = json.find("\"alpha\"").unwrap();
    let ci_pos = json.find("\"ci\"").unwrap();
    let zebra_pos = json.find("\"zebra\"").unwrap();
    assert!(
        alpha_pos < ci_pos && ci_pos < zebra_pos,
        "BTreeMap keys must be in alphabetical order"
    );
}

#[test]
fn btreemap_ordering_in_user_config_profiles() {
    let mut config = UserConfig::default();
    config
        .profiles
        .insert("zebra".to_string(), Profile::default());
    config
        .profiles
        .insert("alpha".to_string(), Profile::default());
    config
        .profiles
        .insert("middle".to_string(), Profile::default());

    let json = serde_json::to_string(&config).unwrap();
    let a = json.find("\"alpha\"").unwrap();
    let m = json.find("\"middle\"").unwrap();
    let z = json.find("\"zebra\"").unwrap();
    assert!(
        a < m && m < z,
        "Profile keys must be alphabetically ordered"
    );
}

// ─── 9. AnalysisPreset all variants roundtrip ───────────────────────────────

#[test]
fn analysis_preset_all_variants_roundtrip() {
    for variant in [
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
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: AnalysisPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ─── 10. DiffFormat / ColorMode / CockpitFormat variants ────────────────────

#[test]
fn diff_format_roundtrip() {
    for variant in [DiffFormat::Md, DiffFormat::Json] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: DiffFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn color_mode_roundtrip() {
    for variant in [ColorMode::Auto, ColorMode::Always, ColorMode::Never] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ColorMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn cockpit_format_roundtrip() {
    for variant in [
        CockpitFormat::Json,
        CockpitFormat::Md,
        CockpitFormat::Sections,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CockpitFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ─── 11. Context/handoff enums roundtrip ────────────────────────────────────

#[test]
fn context_enums_roundtrip() {
    for v in [ContextStrategy::Greedy, ContextStrategy::Spread] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ContextStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    for v in [
        ValueMetric::Code,
        ValueMetric::Tokens,
        ValueMetric::Churn,
        ValueMetric::Hotspot,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ValueMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    for v in [
        ContextOutput::List,
        ContextOutput::Bundle,
        ContextOutput::Json,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ContextOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    for v in [
        HandoffPreset::Minimal,
        HandoffPreset::Standard,
        HandoffPreset::Risk,
        HandoffPreset::Deep,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: HandoffPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

// ─── 12. Remaining enum variants ────────────────────────────────────────────

#[test]
fn misc_enums_roundtrip() {
    for v in [ImportGranularity::Module, ImportGranularity::File] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ImportGranularity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    for v in [
        BadgeMetric::Lines,
        BadgeMetric::Tokens,
        BadgeMetric::Bytes,
        BadgeMetric::Doc,
        BadgeMetric::Blank,
        BadgeMetric::Hotspot,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: BadgeMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    for v in [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: InitProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    for v in [GateFormat::Text, GateFormat::Json] {
        let json = serde_json::to_string(&v).unwrap();
        let back: GateFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    for v in [
        NearDupScope::Module,
        NearDupScope::Lang,
        NearDupScope::Global,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: NearDupScope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    for v in [DiffRangeMode::TwoDot, DiffRangeMode::ThreeDot] {
        let json = serde_json::to_string(&v).unwrap();
        let back: DiffRangeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    for v in [SensorFormat::Json, SensorFormat::Md] {
        let json = serde_json::to_string(&v).unwrap();
        let back: SensorFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

// ─── 13. Deterministic TOML output ──────────────────────────────────────────

#[test]
fn deterministic_toml_output() {
    let mut views = BTreeMap::new();
    views.insert(
        "ci".to_string(),
        ViewProfile {
            format: Some("json".to_string()),
            top: Some(5),
            ..Default::default()
        },
    );
    let config = TomlConfig {
        scan: ScanConfig {
            paths: Some(vec![".".to_string()]),
            ..Default::default()
        },
        view: views,
        ..Default::default()
    };

    let toml1 = toml::to_string(&config).unwrap();
    let toml2 = toml::to_string(&config).unwrap();
    assert_eq!(toml1, toml2, "TOML output must be deterministic");
}

// ─── 14. Deterministic JSON output ──────────────────────────────────────────

#[test]
fn deterministic_json_output() {
    let mut config = UserConfig::default();
    config.profiles.insert("a".to_string(), Profile::default());
    config.profiles.insert("b".to_string(), Profile::default());

    let json1 = serde_json::to_string(&config).unwrap();
    let json2 = serde_json::to_string(&config).unwrap();
    assert_eq!(json1, json2, "JSON output must be deterministic");
}

// ─── 15. Property: UserConfig with arbitrary profiles ───────────────────────

proptest! {
    #[test]
    fn prop_user_config_roundtrip(
        profile_name in "[a-z]{1,10}",
        repo_name in "[a-z]{1,10}/[a-z]{1,10}",
        top in proptest::option::of(1usize..100),
        format in proptest::option::of("(json|md|tsv|csv|jsonl)"),
    ) {
        let mut config = UserConfig::default();
        config.profiles.insert(profile_name.clone(), Profile {
            format,
            top,
            ..Default::default()
        });
        config.repos.insert(repo_name, profile_name.clone());

        let json = serde_json::to_string(&config).unwrap();
        let back: UserConfig = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.profiles.len(), 1);
        prop_assert!(back.profiles.contains_key(&profile_name));
    }

    #[test]
    fn prop_toml_config_json_roundtrip(
        min_code in proptest::option::of(0usize..1000),
        max_rows in proptest::option::of(0usize..5000),
        depth in proptest::option::of(1usize..5),
    ) {
        let config = TomlConfig {
            export: ExportConfig {
                min_code,
                max_rows,
                ..Default::default()
            },
            module: ModuleConfig {
                depth,
                ..Default::default()
            },
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let back: TomlConfig = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.export.min_code, min_code);
        prop_assert_eq!(back.export.max_rows, max_rows);
        prop_assert_eq!(back.module.depth, depth);
    }
}
