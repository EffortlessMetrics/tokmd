//! Additional coverage tests for tokmd-settings.
//!
//! Focuses on edge cases, advanced TOML configs, gate/ratchet rules,
//! deterministic serialization, and error handling.

use tokmd_settings::*;

// =============================================================================
// GateRule and RatchetRuleConfig serde
// =============================================================================

#[test]
fn gate_rule_roundtrip_with_all_fields() {
    let rule = GateRule {
        name: "max-complexity".to_string(),
        pointer: "/complexity/max_cyclomatic".to_string(),
        op: "<=".to_string(),
        value: Some(serde_json::json!(25)),
        values: None,
        negate: false,
        level: Some("error".to_string()),
        message: Some("Cyclomatic complexity must not exceed 25".to_string()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "max-complexity");
    assert_eq!(back.pointer, "/complexity/max_cyclomatic");
    assert_eq!(back.op, "<=");
    assert_eq!(back.value, Some(serde_json::json!(25)));
    assert!(!back.negate);
    assert_eq!(back.level.as_deref(), Some("error"));
    assert_eq!(
        back.message.as_deref(),
        Some("Cyclomatic complexity must not exceed 25")
    );
}

#[test]
fn gate_rule_with_values_list_roundtrip() {
    let rule = GateRule {
        name: "lang-allowlist".to_string(),
        pointer: "/dominant_lang".to_string(),
        op: "in".to_string(),
        value: None,
        values: Some(vec![serde_json::json!("Rust"), serde_json::json!("Python")]),
        negate: true,
        level: Some("warn".to_string()),
        message: None,
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert!(back.negate);
    assert_eq!(back.values.as_ref().unwrap().len(), 2);
    assert!(back.message.is_none());
}

#[test]
fn ratchet_rule_config_all_fields_roundtrip() {
    let rule = RatchetRuleConfig {
        pointer: "/complexity/avg_cyclomatic".to_string(),
        max_increase_pct: Some(10.0),
        max_value: Some(50.0),
        level: Some("error".to_string()),
        description: Some("Average cyclomatic must not exceed 50".to_string()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: RatchetRuleConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.pointer, "/complexity/avg_cyclomatic");
    assert!((back.max_increase_pct.unwrap() - 10.0).abs() < f64::EPSILON);
    assert!((back.max_value.unwrap() - 50.0).abs() < f64::EPSILON);
    assert_eq!(
        back.description.as_deref(),
        Some("Average cyclomatic must not exceed 50")
    );
}

#[test]
fn ratchet_rule_config_minimal_roundtrip() {
    let rule = RatchetRuleConfig {
        pointer: "/derived/totals/code".to_string(),
        max_increase_pct: None,
        max_value: None,
        level: None,
        description: None,
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: RatchetRuleConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.pointer, "/derived/totals/code");
    assert!(back.max_increase_pct.is_none());
    assert!(back.max_value.is_none());
    assert!(back.level.is_none());
}

// =============================================================================
// GateConfig with inline rules
// =============================================================================

#[test]
fn gate_config_with_rules_and_ratchet_toml_roundtrip() {
    let toml_str = r#"
[gate]
policy = "strict.json"
baseline = "baseline.json"
preset = "health"
fail_fast = true
allow_missing_baseline = false
allow_missing_current = true

[[gate.rules]]
name = "max-loc"
pointer = "/derived/totals/code"
op = "<="
value = 100000

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
max_value = 30.0
level = "error"
description = "Keep average cyclomatic complexity under control"
"#;
    let config: TomlConfig = toml::from_str(toml_str).expect("parse gate config");
    assert_eq!(config.gate.policy.as_deref(), Some("strict.json"));
    assert_eq!(config.gate.baseline.as_deref(), Some("baseline.json"));
    assert_eq!(config.gate.fail_fast, Some(true));
    assert_eq!(config.gate.allow_missing_current, Some(true));

    let rules = config.gate.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "max-loc");
    assert_eq!(rules[0].op, "<=");

    let ratchet = config.gate.ratchet.as_ref().unwrap();
    assert_eq!(ratchet.len(), 1);
    assert_eq!(ratchet[0].pointer, "/complexity/avg_cyclomatic");
}

// =============================================================================
// ViewProfile with all fields populated
// =============================================================================

#[test]
fn view_profile_all_fields_toml_roundtrip() {
    let toml_str = r#"
[view.ci]
format = "json"
top = 5
files = true
module_roots = ["src", "lib"]
module_depth = 3
min_code = 10
max_rows = 100
redact = "paths"
meta = false
children = "collapse"
preset = "health"
window = 128000
budget = "64k"
strategy = "greedy"
rank_by = "churn"
output = "bundle"
compress = true
metric = "code"
"#;
    let config: TomlConfig = toml::from_str(toml_str).expect("parse");
    let ci = config.view.get("ci").expect("ci profile");
    assert_eq!(ci.format.as_deref(), Some("json"));
    assert_eq!(ci.top, Some(5));
    assert_eq!(ci.files, Some(true));
    assert_eq!(ci.module_roots.as_ref().unwrap().len(), 2);
    assert_eq!(ci.module_depth, Some(3));
    assert_eq!(ci.min_code, Some(10));
    assert_eq!(ci.max_rows, Some(100));
    assert_eq!(ci.redact.as_deref(), Some("paths"));
    assert_eq!(ci.meta, Some(false));
    assert_eq!(ci.children.as_deref(), Some("collapse"));
    assert_eq!(ci.preset.as_deref(), Some("health"));
    assert_eq!(ci.window, Some(128000));
    assert_eq!(ci.budget.as_deref(), Some("64k"));
    assert_eq!(ci.strategy.as_deref(), Some("greedy"));
    assert_eq!(ci.rank_by.as_deref(), Some("churn"));
    assert_eq!(ci.output.as_deref(), Some("bundle"));
    assert_eq!(ci.compress, Some(true));
    assert_eq!(ci.metric.as_deref(), Some("code"));
}

// =============================================================================
// TomlConfig error handling
// =============================================================================

#[test]
fn toml_parse_invalid_toml_returns_error() {
    let bad = "this is [[[not valid toml";
    let result = TomlConfig::parse(bad);
    assert!(result.is_err());
}

#[test]
fn toml_from_file_nonexistent_returns_io_error() {
    let result = TomlConfig::from_file(std::path::Path::new("/no/such/file.toml"));
    assert!(result.is_err());
}

// =============================================================================
// Deterministic JSON output (sorted keys via derive order)
// =============================================================================

#[test]
fn analyze_settings_json_key_order_is_deterministic() {
    let s = AnalyzeSettings {
        preset: "deep".to_string(),
        window: Some(128000),
        git: Some(true),
        max_files: Some(500),
        max_bytes: Some(10_000_000),
        max_file_bytes: Some(100_000),
        max_commits: Some(1000),
        max_commit_files: Some(50),
        granularity: "file".to_string(),
    };
    let json1 = serde_json::to_string(&s).unwrap();
    let json2 = serde_json::to_string(&s).unwrap();
    assert_eq!(json1, json2, "Serialization must be deterministic");

    // Verify deserialize-reserialize stability
    let back: AnalyzeSettings = serde_json::from_str(&json1).unwrap();
    let json3 = serde_json::to_string(&back).unwrap();
    assert_eq!(json1, json3, "Double roundtrip must be stable");
}

// =============================================================================
// ExportSettings with all fields populated
// =============================================================================

#[test]
fn export_settings_fully_populated_roundtrip() {
    let s = ExportSettings {
        format: ExportFormat::Csv,
        module_roots: vec!["src".into(), "tests".into()],
        module_depth: 3,
        children: ChildIncludeMode::ParentsOnly,
        min_code: 10,
        max_rows: 500,
        redact: RedactMode::All,
        meta: false,
        strip_prefix: Some("project/".into()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert!(matches!(back.format, ExportFormat::Csv));
    assert_eq!(back.module_roots, vec!["src", "tests"]);
    assert_eq!(back.module_depth, 3);
    assert!(matches!(back.children, ChildIncludeMode::ParentsOnly));
    assert_eq!(back.min_code, 10);
    assert_eq!(back.max_rows, 500);
    assert!(matches!(back.redact, RedactMode::All));
    assert!(!back.meta);
    assert_eq!(back.strip_prefix.as_deref(), Some("project/"));
}

// =============================================================================
// TomlConfig full round-trip with all sections
// =============================================================================

#[test]
fn toml_config_full_sections_roundtrip() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target", "*.bak"]
hidden = true
config = "none"
no_ignore = false
no_ignore_parent = true
no_ignore_dot = false
no_ignore_vcs = true
doc_comments = true

[module]
roots = ["crates"]
depth = 3
children = "collapse"

[export]
min_code = 5
max_rows = 1000
redact = "paths"
format = "csv"
children = "separate"

[analyze]
preset = "deep"
window = 200000
format = "json"
git = true
max_files = 1000
max_bytes = 50000000
max_file_bytes = 500000
max_commits = 5000
max_commit_files = 100
granularity = "file"

[context]
budget = "128k"
strategy = "spread"
rank_by = "hotspot"
output = "bundle"
compress = true

[badge]
metric = "code"
"#;
    let config: TomlConfig = toml::from_str(toml_str).expect("parse full config");

    // Verify scan section
    assert_eq!(
        config.scan.paths,
        Some(vec!["src".to_string(), "lib".to_string()])
    );
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.doc_comments, Some(true));
    assert_eq!(config.scan.no_ignore_vcs, Some(true));

    // Verify module section
    assert_eq!(config.module.depth, Some(3));

    // Verify export section
    assert_eq!(config.export.min_code, Some(5));
    assert_eq!(config.export.format.as_deref(), Some("csv"));

    // Verify analyze section
    assert_eq!(config.analyze.preset.as_deref(), Some("deep"));
    assert_eq!(config.analyze.window, Some(200000));
    assert_eq!(config.analyze.git, Some(true));
    assert_eq!(config.analyze.max_files, Some(1000));

    // Verify context section
    assert_eq!(config.context.budget.as_deref(), Some("128k"));
    assert_eq!(config.context.strategy.as_deref(), Some("spread"));
    assert_eq!(config.context.compress, Some(true));

    // Verify badge section
    assert_eq!(config.badge.metric.as_deref(), Some("code"));
}

// =============================================================================
// Deserialize from empty TOML for config types with #[serde(default)]
// =============================================================================

#[test]
fn scan_config_deserialize_from_empty_toml() {
    let cfg: ScanConfig = toml::from_str("").expect("empty TOML");
    assert!(cfg.paths.is_none());
    assert!(cfg.exclude.is_none());
    assert!(cfg.hidden.is_none());
}

#[test]
fn module_config_deserialize_from_empty_toml() {
    let cfg: ModuleConfig = toml::from_str("").expect("empty TOML");
    assert!(cfg.roots.is_none());
    assert!(cfg.depth.is_none());
    assert!(cfg.children.is_none());
}

#[test]
fn context_config_deserialize_from_empty_toml() {
    let cfg: ContextConfig = toml::from_str("").expect("empty TOML");
    assert!(cfg.budget.is_none());
    assert!(cfg.strategy.is_none());
    assert!(cfg.compress.is_none());
}

// =============================================================================
// CockpitSettings with custom values
// =============================================================================

#[test]
fn cockpit_settings_custom_range_mode_roundtrip() {
    let s = CockpitSettings {
        base: "v2.0.0".into(),
        head: "feature/new-api".into(),
        range_mode: "three-dot".into(),
        baseline: Some("baselines/v2.json".into()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "v2.0.0");
    assert_eq!(back.head, "feature/new-api");
    assert_eq!(back.range_mode, "three-dot");
    assert_eq!(back.baseline.as_deref(), Some("baselines/v2.json"));
}
