//! W68 integration tests for tokmd-config: TOML parsing, profile loading,
//! default generation, serde roundtrips, and deterministic serialization.

use tokmd_config::{
    AnalysisPreset, BadgeMetric, CockpitFormat, ColorMode, ContextOutput,
    ContextStrategy, DiffFormat, DiffRangeMode, GateFormat, GlobalArgs, HandoffPreset, InitProfile,
    NearDupScope, Profile, SensorFormat, TomlConfig, UserConfig, ValueMetric,
};

// ═══════════════════════════════════════════════════════════════════════════
// 1. TomlConfig parsing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_minimal_toml() {
    let toml_str = "";
    let config = TomlConfig::parse(toml_str).unwrap();
    assert!(config.scan.paths.is_none());
    assert!(config.module.roots.is_none());
}

#[test]
fn parse_full_toml_config() {
    let toml_str = r#"
[scan]
paths = ["src"]
exclude = ["target", "vendor"]
hidden = true

[module]
roots = ["crates"]
depth = 2

[export]
min_code = 10
max_rows = 500
format = "jsonl"

[analyze]
preset = "health"
window = 128000

[context]
budget = "128k"
strategy = "greedy"

[badge]
metric = "lines"

[gate]
fail_fast = true
policy = "gate.toml"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.scan.paths.as_ref().unwrap(), &["src"]);
    assert_eq!(
        config.scan.exclude.as_ref().unwrap(),
        &["target", "vendor"]
    );
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.module.roots.as_ref().unwrap(), &["crates"]);
    assert_eq!(config.module.depth, Some(2));
    assert_eq!(config.export.min_code, Some(10));
    assert_eq!(config.export.max_rows, Some(500));
    assert_eq!(config.analyze.preset.as_deref(), Some("health"));
    assert_eq!(config.analyze.window, Some(128000));
    assert_eq!(config.context.budget.as_deref(), Some("128k"));
    assert_eq!(config.badge.metric.as_deref(), Some("lines"));
    assert_eq!(config.gate.fail_fast, Some(true));
    assert_eq!(config.gate.policy.as_deref(), Some("gate.toml"));
}

#[test]
fn parse_gate_config_with_inline_rules() {
    let toml_str = r#"
[gate]
fail_fast = false

[[gate.rules]]
name = "max_tokens"
pointer = "/tokens"
op = "lte"
value = 500000

[[gate.rules]]
name = "has_license"
pointer = "/license"
op = "exists"
level = "warn"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let rules = config.gate.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "max_tokens");
    assert_eq!(rules[0].op, "lte");
    assert_eq!(rules[1].name, "has_license");
    assert_eq!(rules[1].level.as_deref(), Some("warn"));
}

#[test]
fn parse_gate_config_with_ratchet_rules() {
    let toml_str = r#"
[gate]
baseline = ".tokmd/baseline.json"
allow_missing_baseline = true

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
level = "error"

[[gate.ratchet]]
pointer = "/complexity/max_cyclomatic"
max_value = 50.0
level = "warn"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(
        config.gate.baseline.as_deref(),
        Some(".tokmd/baseline.json")
    );
    assert_eq!(config.gate.allow_missing_baseline, Some(true));
    let ratchet = config.gate.ratchet.as_ref().unwrap();
    assert_eq!(ratchet.len(), 2);
    assert_eq!(ratchet[0].pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchet[0].max_increase_pct, Some(5.0));
    assert_eq!(ratchet[1].max_value, Some(50.0));
}

#[test]
fn parse_view_profiles() {
    let toml_str = r#"
[view.llm_safe]
format = "json"
top = 10
redact = "all"

[view.ci]
format = "json"
preset = "risk"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.view.len(), 2);
    let llm = &config.view["llm_safe"];
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(10));
    assert_eq!(llm.redact.as_deref(), Some("all"));
    let ci = &config.view["ci"];
    assert_eq!(ci.preset.as_deref(), Some("risk"));
}

#[test]
fn toml_config_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tokmd.toml");
    std::fs::write(
        &path,
        r#"
[scan]
hidden = true

[module]
depth = 3
"#,
    )
    .unwrap();
    let config = TomlConfig::from_file(&path).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.module.depth, Some(3));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Profile / UserConfig
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn user_config_default_is_empty() {
    let c = UserConfig::default();
    assert!(c.profiles.is_empty());
    assert!(c.repos.is_empty());
}

#[test]
fn profile_default_all_none() {
    let p = Profile::default();
    assert!(p.format.is_none());
    assert!(p.top.is_none());
    assert!(p.files.is_none());
    assert!(p.module_roots.is_none());
    assert!(p.redact.is_none());
}

#[test]
fn user_config_serde_roundtrip() {
    let mut c = UserConfig::default();
    c.profiles.insert(
        "llm_safe".into(),
        Profile {
            format: Some("json".into()),
            top: Some(10),
            ..Profile::default()
        },
    );
    c.repos.insert("owner/repo".into(), "llm_safe".into());

    let json = serde_json::to_string(&c).unwrap();
    let back: UserConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.profiles.len(), 1);
    assert_eq!(back.repos.len(), 1);
    assert_eq!(back.profiles["llm_safe"].top, Some(10));
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Default values
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn global_args_default() {
    let g = GlobalArgs::default();
    assert!(g.excluded.is_empty());
    assert!(!g.hidden);
    assert_eq!(g.verbose, 0);
}

#[test]
fn defaults_for_enums() {
    assert_eq!(DiffFormat::default(), DiffFormat::Md);
    assert_eq!(ColorMode::default(), ColorMode::Auto);
    assert_eq!(ContextStrategy::default(), ContextStrategy::Greedy);
    assert_eq!(ValueMetric::default(), ValueMetric::Code);
    assert_eq!(ContextOutput::default(), ContextOutput::List);
    assert_eq!(GateFormat::default(), GateFormat::Text);
    assert_eq!(CockpitFormat::default(), CockpitFormat::Json);
    assert_eq!(HandoffPreset::default(), HandoffPreset::Risk);
    assert_eq!(SensorFormat::default(), SensorFormat::Json);
    assert_eq!(NearDupScope::default(), NearDupScope::Module);
    assert_eq!(DiffRangeMode::default(), DiffRangeMode::TwoDot);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Serde roundtrips (kebab-case)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn analysis_preset_serde_roundtrip() {
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

#[test]
fn diff_format_serde_roundtrip() {
    for variant in [DiffFormat::Md, DiffFormat::Json] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: DiffFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn context_strategy_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ContextStrategy::Greedy).unwrap(),
        "\"greedy\""
    );
    assert_eq!(
        serde_json::to_string(&ContextStrategy::Spread).unwrap(),
        "\"spread\""
    );
}

#[test]
fn badge_metric_serde_roundtrip() {
    for variant in [
        BadgeMetric::Lines,
        BadgeMetric::Tokens,
        BadgeMetric::Bytes,
        BadgeMetric::Doc,
        BadgeMetric::Blank,
        BadgeMetric::Hotspot,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: BadgeMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn init_profile_serde_roundtrip() {
    for variant in [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: InitProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. GlobalArgs → ScanOptions conversion
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn global_args_to_scan_options() {
    let g = GlobalArgs {
        excluded: vec!["target".into()],
        hidden: true,
        no_ignore: true,
        treat_doc_strings_as_comments: true,
        ..GlobalArgs::default()
    };
    let opts: tokmd_settings::ScanOptions = (&g).into();
    assert_eq!(opts.excluded, vec!["target"]);
    assert!(opts.hidden);
    assert!(opts.no_ignore);
    assert!(opts.treat_doc_strings_as_comments);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Deterministic serialization
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn toml_config_serialization_deterministic() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target"]

[module]
roots = ["crates"]
depth = 2

[analyze]
preset = "health"
"#;
    let c1 = TomlConfig::parse(toml_str).unwrap();
    let c2 = TomlConfig::parse(toml_str).unwrap();
    let j1 = serde_json::to_string(&c1).unwrap();
    let j2 = serde_json::to_string(&c2).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn user_config_serialization_deterministic() {
    let mut c = UserConfig::default();
    c.profiles.insert(
        "alpha".into(),
        Profile {
            format: Some("json".into()),
            ..Profile::default()
        },
    );
    c.profiles.insert(
        "beta".into(),
        Profile {
            top: Some(5),
            ..Profile::default()
        },
    );
    let j1 = serde_json::to_string(&c).unwrap();
    let j2 = serde_json::to_string(&c).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn view_profile_btreemap_maintains_order() {
    let toml_str = r#"
[view.zebra]
format = "json"
[view.alpha]
format = "md"
[view.middle]
format = "tsv"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let keys: Vec<&String> = config.view.keys().collect();
    // BTreeMap maintains sorted order
    assert_eq!(keys, vec!["alpha", "middle", "zebra"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Proptest: config parsing determinism
// ═══════════════════════════════════════════════════════════════════════════

mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn toml_parse_deterministic(depth in 1usize..10) {
            let toml_str = format!("[module]\ndepth = {depth}");
            let c1 = TomlConfig::parse(&toml_str).unwrap();
            let c2 = TomlConfig::parse(&toml_str).unwrap();
            prop_assert_eq!(c1.module.depth, c2.module.depth);
            prop_assert_eq!(c1.module.depth, Some(depth));
        }

        #[test]
        fn user_config_json_roundtrip(top in 0usize..100) {
            let mut c = UserConfig::default();
            c.profiles.insert("test".into(), Profile {
                top: Some(top),
                ..Profile::default()
            });
            let json = serde_json::to_string(&c).unwrap();
            let back: UserConfig = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(back.profiles["test"].top, Some(top));
        }
    }
}
