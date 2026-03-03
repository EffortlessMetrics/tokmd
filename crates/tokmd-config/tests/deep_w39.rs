//! Wave-39 deep tests for tokmd-config: TOML parsing, defaults, merging,
//! invalid config, and section coverage.

use tokmd_config::*;

// ── TOML config parsing ──────────────────────────────────────────────────

#[test]
fn parse_minimal_toml() {
    let toml_str = "";
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.module.roots.is_none());
}

#[test]
fn parse_scan_section() {
    let toml_str = r#"
[scan]
paths = ["src", "tests"]
exclude = ["target", "vendor"]
hidden = true
no_ignore = false
doc_comments = true
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let scan = &cfg.scan;
    assert_eq!(scan.paths.as_ref().unwrap(), &["src", "tests"]);
    assert_eq!(scan.exclude.as_ref().unwrap(), &["target", "vendor"]);
    assert_eq!(scan.hidden, Some(true));
    assert_eq!(scan.no_ignore, Some(false));
    assert_eq!(scan.doc_comments, Some(true));
}

#[test]
fn parse_module_section() {
    let toml_str = r#"
[module]
roots = ["crates", "packages"]
depth = 3
children = "collapse"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(
        cfg.module.roots.as_ref().unwrap(),
        &["crates", "packages"]
    );
    assert_eq!(cfg.module.depth, Some(3));
    assert_eq!(cfg.module.children.as_deref(), Some("collapse"));
}

#[test]
fn parse_export_section() {
    let toml_str = r#"
[export]
min_code = 10
max_rows = 500
redact = "paths"
format = "csv"
children = "separate"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.export.min_code, Some(10));
    assert_eq!(cfg.export.max_rows, Some(500));
    assert_eq!(cfg.export.redact.as_deref(), Some("paths"));
    assert_eq!(cfg.export.format.as_deref(), Some("csv"));
    assert_eq!(cfg.export.children.as_deref(), Some("separate"));
}

#[test]
fn parse_analyze_section() {
    let toml_str = r#"
[analyze]
preset = "risk"
window = 128000
format = "json"
git = true
max_files = 5000
max_bytes = 10000000
max_file_bytes = 500000
max_commits = 200
max_commit_files = 50
granularity = "file"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.analyze.preset.as_deref(), Some("risk"));
    assert_eq!(cfg.analyze.window, Some(128000));
    assert_eq!(cfg.analyze.git, Some(true));
    assert_eq!(cfg.analyze.max_files, Some(5000));
    assert_eq!(cfg.analyze.max_bytes, Some(10_000_000));
    assert_eq!(cfg.analyze.max_file_bytes, Some(500_000));
    assert_eq!(cfg.analyze.max_commits, Some(200));
    assert_eq!(cfg.analyze.max_commit_files, Some(50));
    assert_eq!(cfg.analyze.granularity.as_deref(), Some("file"));
}

#[test]
fn parse_gate_section_with_rules() {
    let toml_str = r#"
[gate]
policy = "strict"
preset = "health"
fail_fast = true

[[gate.rules]]
name = "min-doc-ratio"
pointer = "/derived/doc_ratio"
op = "gte"
value = 0.1

[[gate.rules]]
name = "max-todo-density"
pointer = "/health/todo_density"
op = "lte"
value = 5.0
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.gate.policy.as_deref(), Some("strict"));
    assert_eq!(cfg.gate.preset.as_deref(), Some("health"));
    assert_eq!(cfg.gate.fail_fast, Some(true));
    let rules = cfg.gate.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "min-doc-ratio");
    assert_eq!(rules[0].op, "gte");
    assert_eq!(rules[1].name, "max-todo-density");
    assert_eq!(rules[1].pointer, "/health/todo_density");
}

#[test]
fn parse_gate_ratchet_rules() {
    let toml_str = r#"
[[gate.ratchet]]
pointer = "/derived/total_code"
max_increase_pct = 5.0
level = "warn"
description = "Code size should not increase by more than 5%"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let ratchet = cfg.gate.ratchet.as_ref().unwrap();
    assert_eq!(ratchet.len(), 1);
    assert_eq!(ratchet[0].pointer, "/derived/total_code");
    assert_eq!(ratchet[0].max_increase_pct, Some(5.0));
    assert_eq!(ratchet[0].level.as_deref(), Some("warn"));
}

#[test]
fn parse_context_section() {
    let toml_str = r#"
[context]
budget = "256k"
strategy = "spread"
rank_by = "hotspot"
output = "bundle"
compress = true
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.context.budget.as_deref(), Some("256k"));
    assert_eq!(cfg.context.strategy.as_deref(), Some("spread"));
    assert_eq!(cfg.context.rank_by.as_deref(), Some("hotspot"));
    assert_eq!(cfg.context.output.as_deref(), Some("bundle"));
    assert_eq!(cfg.context.compress, Some(true));
}

#[test]
fn parse_badge_section() {
    let toml_str = r#"
[badge]
metric = "tokens"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.badge.metric.as_deref(), Some("tokens"));
}

#[test]
fn parse_view_profiles() {
    let toml_str = r#"
[view.llm_safe]
format = "json"
top = 20
redact = "all"
compress = true

[view.ci]
format = "json"
preset = "receipt"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.view.len(), 2);
    let llm = &cfg.view["llm_safe"];
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(20));
    assert_eq!(llm.redact.as_deref(), Some("all"));
    assert_eq!(llm.compress, Some(true));
    let ci = &cfg.view["ci"];
    assert_eq!(ci.preset.as_deref(), Some("receipt"));
}

// ── Config defaults ──────────────────────────────────────────────────────

#[test]
fn toml_config_default_all_sections_empty() {
    let cfg = TomlConfig::default();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.scan.exclude.is_none());
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.module.depth.is_none());
    assert!(cfg.export.min_code.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.rules.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn scan_config_default_all_none() {
    let s = ScanConfig::default();
    assert!(s.paths.is_none());
    assert!(s.exclude.is_none());
    assert!(s.hidden.is_none());
    assert!(s.config.is_none());
    assert!(s.no_ignore.is_none());
    assert!(s.no_ignore_parent.is_none());
    assert!(s.no_ignore_dot.is_none());
    assert!(s.no_ignore_vcs.is_none());
    assert!(s.doc_comments.is_none());
}

// ── Invalid config handling ──────────────────────────────────────────────

#[test]
fn invalid_toml_syntax_is_err() {
    let bad = "[scan\npaths = [";
    assert!(TomlConfig::parse(bad).is_err());
}

#[test]
fn unknown_section_is_silently_ignored() {
    // TomlConfig uses #[serde(default)] without deny_unknown_fields,
    // so unknown top-level sections are silently ignored.
    let toml_str = r#"
[totally_unknown_section]
key = "value"
"#;
    let result = TomlConfig::parse(toml_str);
    assert!(
        result.is_ok(),
        "unknown section should be silently ignored with serde(default)"
    );
}

#[test]
fn wrong_type_for_field_is_err() {
    let toml_str = r#"
[scan]
hidden = "not_a_bool"
"#;
    assert!(TomlConfig::parse(toml_str).is_err());
}

#[test]
fn negative_usize_field_is_err() {
    let toml_str = r#"
[module]
depth = -1
"#;
    assert!(TomlConfig::parse(toml_str).is_err());
}

// ── Full config roundtrip ────────────────────────────────────────────────

#[test]
fn full_config_toml_roundtrip() {
    let toml_str = r#"
[scan]
paths = ["src"]
exclude = ["target"]
hidden = false

[module]
roots = ["crates"]
depth = 2

[export]
format = "jsonl"

[analyze]
preset = "deep"
window = 64000

[context]
budget = "128k"

[badge]
metric = "lines"

[gate]
fail_fast = false
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    // Verify key fields survived the parse
    assert_eq!(cfg.scan.paths.as_ref().unwrap(), &["src"]);
    assert_eq!(cfg.module.depth, Some(2));
    assert_eq!(cfg.export.format.as_deref(), Some("jsonl"));
    assert_eq!(cfg.analyze.preset.as_deref(), Some("deep"));
    assert_eq!(cfg.context.budget.as_deref(), Some("128k"));
    assert_eq!(cfg.badge.metric.as_deref(), Some("lines"));
    assert_eq!(cfg.gate.fail_fast, Some(false));
}

// ── Enum exhaustiveness ──────────────────────────────────────────────────

#[test]
fn all_analysis_presets_serialize_lowercase() {
    let presets = [
        (AnalysisPreset::Receipt, "receipt"),
        (AnalysisPreset::Health, "health"),
        (AnalysisPreset::Risk, "risk"),
        (AnalysisPreset::Supply, "supply"),
        (AnalysisPreset::Architecture, "architecture"),
        (AnalysisPreset::Topics, "topics"),
        (AnalysisPreset::Security, "security"),
        (AnalysisPreset::Identity, "identity"),
        (AnalysisPreset::Git, "git"),
        (AnalysisPreset::Deep, "deep"),
        (AnalysisPreset::Fun, "fun"),
    ];
    for (variant, expected) in &presets {
        let json = serde_json::to_string(variant).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

// ── GateRule negate default ──────────────────────────────────────────────

#[test]
fn gate_rule_negate_defaults_false() {
    let toml_str = r#"
[[gate.rules]]
name = "test-rule"
pointer = "/foo"
op = "eq"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let rules = cfg.gate.rules.unwrap();
    assert!(!rules[0].negate, "negate should default to false");
}

// ── Multiple view profiles ──────────────────────────────────────────────

#[test]
fn view_profile_fields_all_optional() {
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
