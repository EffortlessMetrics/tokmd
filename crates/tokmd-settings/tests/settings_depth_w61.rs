//! W61 depth tests for `tokmd-settings`.
//!
//! Coverage: serde roundtrips for every settings struct, default verification,
//! TOML parsing (edge cases, profiles, gate rules, ratchet rules),
//! enum exhaustiveness, flatten behavior, and proptest properties.

use proptest::prelude::*;
use serde_json::Value;
use tokmd_settings::{
    AnalyzeSettings, ChildIncludeMode, ChildrenMode, CockpitSettings, ConfigMode, DiffSettings,
    ExportFormat, ExportSettings, GateRule, LangSettings, ModuleSettings, RatchetRuleConfig,
    RedactMode, ScanOptions, ScanSettings, TomlConfig, ViewProfile,
};

// ══════════════════════════════════════════════════════════════════════
// 1. ScanOptions — default verification + all fields
// ══════════════════════════════════════════════════════════════════════

#[test]
fn scan_options_default_all_false_or_empty() {
    let opts = ScanOptions::default();
    assert!(opts.excluded.is_empty());
    assert_eq!(opts.config, ConfigMode::Auto);
    assert!(!opts.hidden);
    assert!(!opts.no_ignore);
    assert!(!opts.no_ignore_parent);
    assert!(!opts.no_ignore_dot);
    assert!(!opts.no_ignore_vcs);
    assert!(!opts.treat_doc_strings_as_comments);
}

#[test]
fn scan_options_all_true_roundtrip() {
    let opts = ScanOptions {
        excluded: vec!["target".into(), "dist".into()],
        config: ConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };
    let json = serde_json::to_string(&opts).unwrap();
    let back: ScanOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded.len(), 2);
    assert_eq!(back.config, ConfigMode::None);
    assert!(back.hidden);
    assert!(back.no_ignore);
    assert!(back.treat_doc_strings_as_comments);
}

// ══════════════════════════════════════════════════════════════════════
// 2. ScanSettings — constructors + flatten behavior
// ══════════════════════════════════════════════════════════════════════

#[test]
fn scan_settings_current_dir_path() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
    assert!(!s.options.hidden);
}

#[test]
fn scan_settings_for_paths_preserves_all() {
    let s = ScanSettings::for_paths(vec!["a".into(), "b".into(), "c".into()]);
    assert_eq!(s.paths.len(), 3);
}

#[test]
fn scan_settings_flatten_serializes_options_inline() {
    let s = ScanSettings {
        paths: vec![".".into()],
        options: ScanOptions {
            hidden: true,
            ..Default::default()
        },
    };
    let json = serde_json::to_string(&s).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    // `hidden` appears at top level due to #[serde(flatten)]
    assert_eq!(val["hidden"], true);
    assert!(val.get("options").is_none());
}

#[test]
fn scan_settings_default_paths_empty() {
    let s = ScanSettings::default();
    assert!(s.paths.is_empty());
}

// ══════════════════════════════════════════════════════════════════════
// 3. LangSettings defaults and roundtrip
// ══════════════════════════════════════════════════════════════════════

#[test]
fn lang_settings_defaults_collapse() {
    let ls = LangSettings::default();
    assert_eq!(ls.top, 0);
    assert!(!ls.files);
    assert_eq!(ls.children, ChildrenMode::Collapse);
    assert!(ls.redact.is_none());
}

#[test]
fn lang_settings_custom_roundtrip() {
    let ls = LangSettings {
        top: 5,
        files: true,
        children: ChildrenMode::Separate,
        redact: Some(RedactMode::All),
    };
    let json = serde_json::to_string(&ls).unwrap();
    let back: LangSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 5);
    assert!(back.files);
    assert_eq!(back.children, ChildrenMode::Separate);
    assert_eq!(back.redact, Some(RedactMode::All));
}

// ══════════════════════════════════════════════════════════════════════
// 4. ModuleSettings defaults
// ══════════════════════════════════════════════════════════════════════

#[test]
fn module_settings_defaults() {
    let ms = ModuleSettings::default();
    assert_eq!(ms.top, 0);
    assert_eq!(ms.module_roots, vec!["crates", "packages"]);
    assert_eq!(ms.module_depth, 2);
    assert_eq!(ms.children, ChildIncludeMode::Separate);
    assert!(ms.redact.is_none());
}

#[test]
fn module_settings_custom_roundtrip() {
    let ms = ModuleSettings {
        top: 10,
        module_roots: vec!["src".into()],
        module_depth: 3,
        children: ChildIncludeMode::ParentsOnly,
        redact: Some(RedactMode::Paths),
    };
    let json = serde_json::to_string(&ms).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.module_depth, 3);
    assert_eq!(back.children, ChildIncludeMode::ParentsOnly);
}

// ══════════════════════════════════════════════════════════════════════
// 5. ExportSettings defaults
// ══════════════════════════════════════════════════════════════════════

#[test]
fn export_settings_defaults() {
    let es = ExportSettings::default();
    assert_eq!(es.format, ExportFormat::Jsonl);
    assert_eq!(es.module_roots, vec!["crates", "packages"]);
    assert_eq!(es.module_depth, 2);
    assert_eq!(es.children, ChildIncludeMode::Separate);
    assert_eq!(es.min_code, 0);
    assert_eq!(es.max_rows, 0);
    assert_eq!(es.redact, RedactMode::None);
    assert!(es.meta);
    assert!(es.strip_prefix.is_none());
}

#[test]
fn export_settings_all_formats_roundtrip() {
    for fmt in [
        ExportFormat::Csv,
        ExportFormat::Jsonl,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ] {
        let es = ExportSettings {
            format: fmt,
            ..Default::default()
        };
        let json = serde_json::to_string(&es).unwrap();
        let back: ExportSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.format, fmt);
    }
}

#[test]
fn export_settings_strip_prefix_roundtrip() {
    let es = ExportSettings {
        strip_prefix: Some("/home/user/project".into()),
        ..Default::default()
    };
    let json = serde_json::to_string(&es).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.strip_prefix.as_deref(), Some("/home/user/project"));
}

// ══════════════════════════════════════════════════════════════════════
// 6. AnalyzeSettings defaults
// ══════════════════════════════════════════════════════════════════════

#[test]
fn analyze_settings_defaults() {
    let a = AnalyzeSettings::default();
    assert_eq!(a.preset, "receipt");
    assert_eq!(a.granularity, "module");
    assert!(a.window.is_none());
    assert!(a.git.is_none());
    assert!(a.max_files.is_none());
    assert!(a.max_bytes.is_none());
    assert!(a.max_file_bytes.is_none());
    assert!(a.max_commits.is_none());
    assert!(a.max_commit_files.is_none());
}

#[test]
fn analyze_settings_full_roundtrip() {
    let a = AnalyzeSettings {
        preset: "deep".into(),
        window: Some(128_000),
        git: Some(true),
        max_files: Some(1000),
        max_bytes: Some(50_000_000),
        max_file_bytes: Some(500_000),
        max_commits: Some(500),
        max_commit_files: Some(200),
        granularity: "file".into(),
        ..Default::default()
    };
    let json = serde_json::to_string(&a).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "deep");
    assert_eq!(back.window, Some(128_000));
    assert_eq!(back.max_bytes, Some(50_000_000));
}

// ══════════════════════════════════════════════════════════════════════
// 7. CockpitSettings defaults
// ══════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_settings_defaults() {
    let c = CockpitSettings::default();
    assert_eq!(c.base, "main");
    assert_eq!(c.head, "HEAD");
    assert_eq!(c.range_mode, "two-dot");
    assert!(c.baseline.is_none());
}

#[test]
fn cockpit_settings_custom_roundtrip() {
    let c = CockpitSettings {
        base: "v1.0.0".into(),
        head: "feature/x".into(),
        range_mode: "three-dot".into(),
        baseline: Some("baseline.json".into()),
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "v1.0.0");
    assert_eq!(back.baseline, Some("baseline.json".into()));
}

// ══════════════════════════════════════════════════════════════════════
// 8. DiffSettings
// ══════════════════════════════════════════════════════════════════════

#[test]
fn diff_settings_default_empty() {
    let d = DiffSettings::default();
    assert!(d.from.is_empty());
    assert!(d.to.is_empty());
}

#[test]
fn diff_settings_roundtrip() {
    let d = DiffSettings {
        from: "v1.0".into(),
        to: "v2.0".into(),
    };
    let json = serde_json::to_string(&d).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "v1.0");
    assert_eq!(back.to, "v2.0");
}

// ══════════════════════════════════════════════════════════════════════
// 9. Enum serde exhaustiveness
// ══════════════════════════════════════════════════════════════════════

#[test]
fn config_mode_all_variants() {
    for v in [ConfigMode::Auto, ConfigMode::None] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ConfigMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn children_mode_all_variants() {
    for v in [ChildrenMode::Collapse, ChildrenMode::Separate] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ChildrenMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn child_include_mode_all_variants() {
    for v in [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ChildIncludeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn redact_mode_all_variants() {
    for v in [RedactMode::None, RedactMode::Paths, RedactMode::All] {
        let json = serde_json::to_string(&v).unwrap();
        let back: RedactMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn export_format_all_variants() {
    for v in [
        ExportFormat::Csv,
        ExportFormat::Jsonl,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

// ══════════════════════════════════════════════════════════════════════
// 10. TOML parsing — empty, minimal, full
// ══════════════════════════════════════════════════════════════════════

#[test]
fn toml_parse_empty_string() {
    let config = TomlConfig::parse("").unwrap();
    assert!(config.scan.hidden.is_none());
    assert!(config.module.depth.is_none());
    assert!(config.view.is_empty());
}

#[test]
fn toml_parse_scan_section() {
    let toml_str = r#"
[scan]
hidden = true
no_ignore = false
paths = ["src", "lib"]
exclude = ["target", "*.bak"]
config = "none"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.no_ignore, Some(false));
    assert_eq!(config.scan.paths, Some(vec!["src".into(), "lib".into()]));
    assert_eq!(
        config.scan.exclude,
        Some(vec!["target".into(), "*.bak".into()])
    );
}

#[test]
fn toml_parse_module_section() {
    let toml_str = r#"
[module]
roots = ["packages"]
depth = 3
children = "collapse"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.module.roots, Some(vec!["packages".into()]));
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(config.module.children.as_deref(), Some("collapse"));
}

#[test]
fn toml_parse_export_section() {
    let toml_str = r#"
[export]
format = "csv"
min_code = 10
max_rows = 500
redact = "paths"
children = "separate"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.export.format.as_deref(), Some("csv"));
    assert_eq!(config.export.min_code, Some(10));
    assert_eq!(config.export.max_rows, Some(500));
}

#[test]
fn toml_parse_analyze_section() {
    let toml_str = r#"
[analyze]
preset = "risk"
window = 128000
git = true
max_files = 5000
max_bytes = 100000000
max_commits = 1000
granularity = "file"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.analyze.preset.as_deref(), Some("risk"));
    assert_eq!(config.analyze.window, Some(128000));
    assert_eq!(config.analyze.git, Some(true));
    assert_eq!(config.analyze.granularity.as_deref(), Some("file"));
}

#[test]
fn toml_parse_context_section() {
    let toml_str = r#"
[context]
budget = "128k"
strategy = "spread"
rank_by = "churn"
output = "bundle"
compress = true
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.context.budget.as_deref(), Some("128k"));
    assert_eq!(config.context.strategy.as_deref(), Some("spread"));
    assert_eq!(config.context.compress, Some(true));
}

#[test]
fn toml_parse_badge_section() {
    let toml_str = r#"
[badge]
metric = "complexity"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.badge.metric.as_deref(), Some("complexity"));
}

// ══════════════════════════════════════════════════════════════════════
// 11. TOML gate rules (inline policy)
// ══════════════════════════════════════════════════════════════════════

#[test]
fn toml_parse_gate_with_inline_rules() {
    let toml_str = r#"
[gate]
policy = "policy.json"
fail_fast = true

[[gate.rules]]
name = "max_complexity"
pointer = "/complexity/max_cyclomatic"
op = "<="
value = 25

[[gate.rules]]
name = "no_high_entropy"
pointer = "/entropy/suspects"
op = "len<="
value = 0
negate = false
level = "error"
message = "No high-entropy files allowed"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.gate.policy.as_deref(), Some("policy.json"));
    assert_eq!(config.gate.fail_fast, Some(true));

    let rules = config.gate.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "max_complexity");
    assert_eq!(rules[0].pointer, "/complexity/max_cyclomatic");
    assert_eq!(rules[0].op, "<=");
    assert_eq!(rules[1].level.as_deref(), Some("error"));
}

#[test]
fn toml_parse_gate_with_ratchet_rules() {
    let toml_str = r#"
[gate]
baseline = "baseline.json"

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
level = "error"
description = "Complexity must not increase by more than 5%"

[[gate.ratchet]]
pointer = "/complexity/max_cyclomatic"
max_value = 30.0
level = "warn"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.gate.baseline.as_deref(), Some("baseline.json"));

    let ratchets = config.gate.ratchet.as_ref().unwrap();
    assert_eq!(ratchets.len(), 2);
    assert_eq!(ratchets[0].max_increase_pct, Some(5.0));
    assert_eq!(ratchets[1].max_value, Some(30.0));
}

// ══════════════════════════════════════════════════════════════════════
// 12. TOML view profiles
// ══════════════════════════════════════════════════════════════════════

#[test]
fn toml_parse_multiple_view_profiles() {
    let toml_str = r#"
[view.llm]
format = "json"
top = 20
budget = "128k"
compress = true

[view.ci]
format = "tsv"
preset = "health"
redact = "paths"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.view.len(), 2);

    let llm = config.view.get("llm").unwrap();
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(20));
    assert_eq!(llm.budget.as_deref(), Some("128k"));
    assert_eq!(llm.compress, Some(true));

    let ci = config.view.get("ci").unwrap();
    assert_eq!(ci.preset.as_deref(), Some("health"));
    assert_eq!(ci.redact.as_deref(), Some("paths"));
}

#[test]
fn view_profile_default_all_none() {
    let vp = ViewProfile::default();
    assert!(vp.format.is_none());
    assert!(vp.top.is_none());
    assert!(vp.files.is_none());
    assert!(vp.module_roots.is_none());
    assert!(vp.module_depth.is_none());
    assert!(vp.min_code.is_none());
    assert!(vp.max_rows.is_none());
    assert!(vp.redact.is_none());
    assert!(vp.meta.is_none());
    assert!(vp.children.is_none());
    assert!(vp.preset.is_none());
    assert!(vp.window.is_none());
    assert!(vp.budget.is_none());
    assert!(vp.strategy.is_none());
    assert!(vp.rank_by.is_none());
    assert!(vp.output.is_none());
    assert!(vp.compress.is_none());
    assert!(vp.metric.is_none());
}

// ══════════════════════════════════════════════════════════════════════
// 13. GateRule serde roundtrip
// ══════════════════════════════════════════════════════════════════════

#[test]
fn gate_rule_serde_roundtrip() {
    let rule = GateRule {
        name: "test_rule".into(),
        pointer: "/derived/totals/code".into(),
        op: ">=".into(),
        value: Some(serde_json::json!(100)),
        values: None,
        negate: false,
        level: Some("error".into()),
        message: Some("Must have at least 100 lines of code".into()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "test_rule");
    assert_eq!(back.pointer, "/derived/totals/code");
    assert!(!back.negate);
}

#[test]
fn gate_rule_with_values_list() {
    let rule = GateRule {
        name: "allowed_licenses".into(),
        pointer: "/license/effective".into(),
        op: "in".into(),
        value: None,
        values: Some(vec![
            serde_json::json!("MIT"),
            serde_json::json!("Apache-2.0"),
        ]),
        negate: false,
        level: None,
        message: None,
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert_eq!(back.values.as_ref().unwrap().len(), 2);
}

#[test]
fn gate_rule_negated() {
    let rule = GateRule {
        name: "no_critical".into(),
        pointer: "/complexity/high_risk_files".into(),
        op: ">".into(),
        value: Some(serde_json::json!(0)),
        values: None,
        negate: true,
        level: Some("error".into()),
        message: None,
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert!(back.negate);
}

// ══════════════════════════════════════════════════════════════════════
// 14. RatchetRuleConfig serde roundtrip
// ══════════════════════════════════════════════════════════════════════

#[test]
fn ratchet_rule_config_full_roundtrip() {
    let rule = RatchetRuleConfig {
        pointer: "/complexity/avg_cyclomatic".into(),
        max_increase_pct: Some(5.0),
        max_value: Some(20.0),
        level: Some("error".into()),
        description: Some("Complexity guard".into()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: RatchetRuleConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.pointer, "/complexity/avg_cyclomatic");
    assert_eq!(back.max_increase_pct, Some(5.0));
    assert_eq!(back.max_value, Some(20.0));
}

#[test]
fn ratchet_rule_config_minimal() {
    let rule = RatchetRuleConfig {
        pointer: "/derived/totals/code".into(),
        max_increase_pct: None,
        max_value: None,
        level: None,
        description: None,
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: RatchetRuleConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.pointer, "/derived/totals/code");
    assert!(back.level.is_none());
}

// ══════════════════════════════════════════════════════════════════════
// 15. TOML config defaults (all sections)
// ══════════════════════════════════════════════════════════════════════

#[test]
fn toml_config_default_all_sections_none_or_empty() {
    let config = TomlConfig::default();
    assert!(config.scan.hidden.is_none());
    assert!(config.scan.paths.is_none());
    assert!(config.module.roots.is_none());
    assert!(config.module.depth.is_none());
    assert!(config.export.format.is_none());
    assert!(config.analyze.preset.is_none());
    assert!(config.context.budget.is_none());
    assert!(config.badge.metric.is_none());
    assert!(config.gate.policy.is_none());
    assert!(config.gate.rules.is_none());
    assert!(config.gate.ratchet.is_none());
    assert!(config.view.is_empty());
}

// ══════════════════════════════════════════════════════════════════════
// 16. Gate config allow_missing flags
// ══════════════════════════════════════════════════════════════════════

#[test]
fn gate_config_allow_missing_flags() {
    let toml_str = r#"
[gate]
allow_missing_baseline = true
allow_missing_current = false
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.gate.allow_missing_baseline, Some(true));
    assert_eq!(config.gate.allow_missing_current, Some(false));
}

// ══════════════════════════════════════════════════════════════════════
// 17. TOML full kitchen-sink config
// ══════════════════════════════════════════════════════════════════════

#[test]
fn toml_full_config_roundtrip() {
    let toml_str = r#"
[scan]
hidden = true
paths = ["."]
exclude = ["target"]

[module]
roots = ["crates"]
depth = 2

[export]
format = "jsonl"
min_code = 5

[analyze]
preset = "health"
window = 64000

[context]
budget = "64k"
strategy = "greedy"

[badge]
metric = "lines"

[gate]
policy = "policy.json"
fail_fast = false

[view.default]
format = "markdown"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.module.roots, Some(vec!["crates".into()]));
    assert_eq!(config.export.min_code, Some(5));
    assert_eq!(config.analyze.preset.as_deref(), Some("health"));
    assert_eq!(config.context.budget.as_deref(), Some("64k"));
    assert_eq!(config.badge.metric.as_deref(), Some("lines"));
    assert_eq!(config.gate.fail_fast, Some(false));
    assert_eq!(
        config.view.get("default").unwrap().format.as_deref(),
        Some("markdown")
    );
}

// ══════════════════════════════════════════════════════════════════════
// 18. JSON deserialize with missing optional fields uses defaults
// ══════════════════════════════════════════════════════════════════════

#[test]
fn lang_settings_from_minimal_json() {
    let json = r#"{}"#;
    let ls: LangSettings = serde_json::from_str(json).unwrap();
    assert_eq!(ls.top, 0);
    assert_eq!(ls.children, ChildrenMode::Collapse);
}

#[test]
fn module_settings_from_minimal_json() {
    let json = r#"{}"#;
    let ms: ModuleSettings = serde_json::from_str(json).unwrap();
    assert_eq!(ms.module_roots, vec!["crates", "packages"]);
    assert_eq!(ms.module_depth, 2);
    assert_eq!(ms.children, ChildIncludeMode::Separate);
}

#[test]
fn export_settings_from_minimal_json() {
    let json = r#"{}"#;
    let es: ExportSettings = serde_json::from_str(json).unwrap();
    assert_eq!(es.format, ExportFormat::Jsonl);
    assert!(es.meta);
    assert_eq!(es.redact, RedactMode::None);
}

#[test]
fn analyze_settings_from_minimal_json() {
    let json = r#"{}"#;
    let a: AnalyzeSettings = serde_json::from_str(json).unwrap();
    assert_eq!(a.preset, "receipt");
    assert_eq!(a.granularity, "module");
}

// ══════════════════════════════════════════════════════════════════════
// 19. Proptest — property-based verification
// ══════════════════════════════════════════════════════════════════════

fn arb_config_mode() -> impl Strategy<Value = ConfigMode> {
    prop_oneof![Just(ConfigMode::Auto), Just(ConfigMode::None),]
}

fn arb_children_mode() -> impl Strategy<Value = ChildrenMode> {
    prop_oneof![Just(ChildrenMode::Collapse), Just(ChildrenMode::Separate),]
}

fn arb_child_include_mode() -> impl Strategy<Value = ChildIncludeMode> {
    prop_oneof![
        Just(ChildIncludeMode::Separate),
        Just(ChildIncludeMode::ParentsOnly),
    ]
}

fn arb_redact_mode() -> impl Strategy<Value = RedactMode> {
    prop_oneof![
        Just(RedactMode::None),
        Just(RedactMode::Paths),
        Just(RedactMode::All),
    ]
}

fn arb_export_format() -> impl Strategy<Value = ExportFormat> {
    prop_oneof![
        Just(ExportFormat::Csv),
        Just(ExportFormat::Jsonl),
        Just(ExportFormat::Json),
        Just(ExportFormat::Cyclonedx),
    ]
}

proptest! {
    #[test]
    fn prop_config_mode_roundtrip(v in arb_config_mode()) {
        let json = serde_json::to_string(&v).unwrap();
        let back: ConfigMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, v);
    }

    #[test]
    fn prop_children_mode_roundtrip(v in arb_children_mode()) {
        let json = serde_json::to_string(&v).unwrap();
        let back: ChildrenMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, v);
    }

    #[test]
    fn prop_child_include_mode_roundtrip(v in arb_child_include_mode()) {
        let json = serde_json::to_string(&v).unwrap();
        let back: ChildIncludeMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, v);
    }

    #[test]
    fn prop_redact_mode_roundtrip(v in arb_redact_mode()) {
        let json = serde_json::to_string(&v).unwrap();
        let back: RedactMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, v);
    }

    #[test]
    fn prop_export_format_roundtrip(v in arb_export_format()) {
        let json = serde_json::to_string(&v).unwrap();
        let back: ExportFormat = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, v);
    }

    #[test]
    fn prop_scan_options_hidden_preserved(hidden in proptest::bool::ANY) {
        let opts = ScanOptions {
            hidden,
            ..Default::default()
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: ScanOptions = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.hidden, hidden);
    }

    #[test]
    fn prop_lang_settings_top_preserved(top in 0usize..10000) {
        let ls = LangSettings {
            top,
            ..Default::default()
        };
        let json = serde_json::to_string(&ls).unwrap();
        let back: LangSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.top, top);
    }

    #[test]
    fn prop_analyze_window_preserved(window in proptest::option::of(1usize..1_000_000)) {
        let a = AnalyzeSettings {
            window,
            ..Default::default()
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.window, window);
    }

    #[test]
    fn prop_export_min_code_preserved(min_code in 0usize..10000) {
        let es = ExportSettings {
            min_code,
            ..Default::default()
        };
        let json = serde_json::to_string(&es).unwrap();
        let back: ExportSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.min_code, min_code);
    }
}
