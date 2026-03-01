//! Serialization / deserialization roundtrip tests for every settings type.

use serde_json;
use tokmd_settings::*;

// =============================================================================
// Helpers
// =============================================================================

/// JSON roundtrip: serialize then deserialize and return the recovered value.
fn json_roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(val: &T) -> T {
    let json = serde_json::to_string(val).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

/// TOML roundtrip via TomlConfig::parse.
fn toml_roundtrip(toml_str: &str) -> TomlConfig {
    TomlConfig::parse(toml_str).expect("parse TOML")
}

// =============================================================================
// ScanOptions
// =============================================================================

#[test]
fn scan_options_roundtrip_all_fields_set() {
    let opts = ScanOptions {
        excluded: vec!["target".into(), "node_modules".into()],
        config: ConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };
    let back = json_roundtrip(&opts);
    assert_eq!(back.excluded, opts.excluded);
    assert!(back.hidden);
    assert!(back.no_ignore);
    assert!(back.no_ignore_parent);
    assert!(back.no_ignore_dot);
    assert!(back.no_ignore_vcs);
    assert!(back.treat_doc_strings_as_comments);
}

#[test]
fn scan_options_roundtrip_defaults() {
    let back = json_roundtrip(&ScanOptions::default());
    assert!(back.excluded.is_empty());
    assert!(!back.hidden);
}

#[test]
fn scan_options_empty_strings_in_excluded() {
    let opts = ScanOptions {
        excluded: vec!["".into(), "   ".into()],
        ..Default::default()
    };
    let back = json_roundtrip(&opts);
    assert_eq!(back.excluded, vec!["", "   "]);
}

#[test]
fn scan_options_special_chars_in_excluded() {
    let opts = ScanOptions {
        excluded: vec![
            "path/with spaces".into(),
            "日本語/パス".into(),
            "foo*bar?baz".into(),
            r"back\slash".into(),
        ],
        ..Default::default()
    };
    let back = json_roundtrip(&opts);
    assert_eq!(back.excluded, opts.excluded);
}

// =============================================================================
// ScanSettings
// =============================================================================

#[test]
fn scan_settings_roundtrip_with_flatten() {
    let s = ScanSettings {
        paths: vec!["src".into(), "lib".into()],
        options: ScanOptions {
            hidden: true,
            excluded: vec!["*.bak".into()],
            ..Default::default()
        },
    };
    let back = json_roundtrip(&s);
    assert_eq!(back.paths, vec!["src", "lib"]);
    assert!(back.options.hidden);
    assert_eq!(back.options.excluded, vec!["*.bak"]);
}

#[test]
fn scan_settings_empty_paths() {
    let s = ScanSettings {
        paths: vec![],
        options: ScanOptions::default(),
    };
    let back = json_roundtrip(&s);
    assert!(back.paths.is_empty());
}

#[test]
fn scan_settings_very_long_path() {
    let long = "a/".repeat(500) + "file.rs";
    let s = ScanSettings::for_paths(vec![long.clone()]);
    let back = json_roundtrip(&s);
    assert_eq!(back.paths[0], long);
}

#[test]
fn scan_settings_unicode_paths() {
    let s = ScanSettings::for_paths(vec![
        "données/résumé.rs".into(),
        "中文路径/文件.rs".into(),
        "путь/файл.rs".into(),
    ]);
    let back = json_roundtrip(&s);
    assert_eq!(back.paths.len(), 3);
    assert_eq!(back.paths[0], "données/résumé.rs");
}

// =============================================================================
// LangSettings
// =============================================================================

#[test]
fn lang_settings_roundtrip_custom() {
    let s = LangSettings {
        top: 42,
        files: true,
        children: ChildrenMode::Separate,
        redact: Some(RedactMode::Paths),
    };
    let back = json_roundtrip(&s);
    assert_eq!(back.top, 42);
    assert!(back.files);
}

#[test]
fn lang_settings_roundtrip_default() {
    let s = LangSettings::default();
    let back = json_roundtrip(&s);
    assert_eq!(back.top, 0);
    assert!(!back.files);
    assert!(back.redact.is_none());
}

// =============================================================================
// ModuleSettings
// =============================================================================

#[test]
fn module_settings_roundtrip_custom() {
    let s = ModuleSettings {
        top: 5,
        module_roots: vec!["packages".into(), "libs".into()],
        module_depth: 4,
        children: ChildIncludeMode::ParentsOnly,
        redact: Some(RedactMode::All),
    };
    let back = json_roundtrip(&s);
    assert_eq!(back.top, 5);
    assert_eq!(back.module_roots, vec!["packages", "libs"]);
    assert_eq!(back.module_depth, 4);
}

#[test]
fn module_settings_roundtrip_default() {
    let s = ModuleSettings::default();
    let back = json_roundtrip(&s);
    assert_eq!(back.module_roots, vec!["crates", "packages"]);
    assert_eq!(back.module_depth, 2);
}

// =============================================================================
// ExportSettings
// =============================================================================

#[test]
fn export_settings_roundtrip_custom() {
    let s = ExportSettings {
        format: ExportFormat::Csv,
        module_roots: vec!["src".into()],
        module_depth: 3,
        children: ChildIncludeMode::ParentsOnly,
        min_code: 10,
        max_rows: 100,
        redact: RedactMode::Paths,
        meta: false,
        strip_prefix: Some("prefix/".into()),
    };
    let back = json_roundtrip(&s);
    assert_eq!(back.min_code, 10);
    assert_eq!(back.max_rows, 100);
    assert!(!back.meta);
    assert_eq!(back.strip_prefix, Some("prefix/".into()));
}

#[test]
fn export_settings_roundtrip_default() {
    let s = ExportSettings::default();
    let back = json_roundtrip(&s);
    assert_eq!(back.min_code, 0);
    assert_eq!(back.max_rows, 0);
    assert!(back.meta);
    assert!(back.strip_prefix.is_none());
}

// =============================================================================
// AnalyzeSettings
// =============================================================================

#[test]
fn analyze_settings_roundtrip_custom() {
    let s = AnalyzeSettings {
        preset: "deep".into(),
        window: Some(128_000),
        git: Some(true),
        max_files: Some(5000),
        max_bytes: Some(50_000_000),
        max_file_bytes: Some(1_000_000),
        max_commits: Some(1000),
        max_commit_files: Some(500),
        granularity: "file".into(),
    };
    let back = json_roundtrip(&s);
    assert_eq!(back.preset, "deep");
    assert_eq!(back.window, Some(128_000));
    assert_eq!(back.git, Some(true));
    assert_eq!(back.max_files, Some(5000));
    assert_eq!(back.max_bytes, Some(50_000_000));
    assert_eq!(back.max_file_bytes, Some(1_000_000));
    assert_eq!(back.max_commits, Some(1000));
    assert_eq!(back.max_commit_files, Some(500));
    assert_eq!(back.granularity, "file");
}

#[test]
fn analyze_settings_roundtrip_default() {
    let s = AnalyzeSettings::default();
    let back = json_roundtrip(&s);
    assert_eq!(back.preset, "receipt");
    assert_eq!(back.granularity, "module");
    assert!(back.window.is_none());
    assert!(back.git.is_none());
}

// =============================================================================
// CockpitSettings
// =============================================================================

#[test]
fn cockpit_settings_roundtrip_custom() {
    let s = CockpitSettings {
        base: "v2.0.0".into(),
        head: "feat/new".into(),
        range_mode: "three-dot".into(),
        baseline: Some("path/to/baseline.json".into()),
    };
    let back = json_roundtrip(&s);
    assert_eq!(back.base, "v2.0.0");
    assert_eq!(back.head, "feat/new");
    assert_eq!(back.range_mode, "three-dot");
    assert_eq!(back.baseline, Some("path/to/baseline.json".into()));
}

#[test]
fn cockpit_settings_roundtrip_default() {
    let s = CockpitSettings::default();
    let back = json_roundtrip(&s);
    assert_eq!(back.base, "main");
    assert_eq!(back.head, "HEAD");
    assert_eq!(back.range_mode, "two-dot");
    assert!(back.baseline.is_none());
}

// =============================================================================
// DiffSettings
// =============================================================================

#[test]
fn diff_settings_roundtrip() {
    let s = DiffSettings {
        from: "v1.0.0".into(),
        to: "v2.0.0".into(),
    };
    let back = json_roundtrip(&s);
    assert_eq!(back.from, "v1.0.0");
    assert_eq!(back.to, "v2.0.0");
}

#[test]
fn diff_settings_empty_refs() {
    let s = DiffSettings {
        from: "".into(),
        to: "".into(),
    };
    let back = json_roundtrip(&s);
    assert_eq!(back.from, "");
    assert_eq!(back.to, "");
}

// =============================================================================
// TomlConfig (TOML-specific roundtrips)
// =============================================================================

#[test]
fn toml_empty_string_parses_to_defaults() {
    let cfg = toml_roundtrip("");
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn toml_full_config_roundtrip() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target", "vendor"]
hidden = true
config = "none"
no_ignore = true

[module]
roots = ["crates", "packages"]
depth = 3
children = "separate"

[export]
min_code = 5
max_rows = 200
redact = "paths"
format = "csv"

[analyze]
preset = "deep"
window = 128000
git = true
max_files = 5000
granularity = "file"

[context]
budget = "100k"
strategy = "spread"

[badge]
metric = "code"

[gate]
policy = "gate.json"
fail_fast = true

[view.llm]
format = "json"
top = 10
budget = "50k"

[view.ci]
format = "tsv"
preset = "health"
"#;
    let cfg = toml_roundtrip(toml_str);
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.no_ignore, Some(true));
    assert_eq!(
        cfg.scan.exclude,
        Some(vec!["target".into(), "vendor".into()])
    );
    assert_eq!(cfg.module.depth, Some(3));
    assert_eq!(cfg.export.min_code, Some(5));
    assert_eq!(cfg.analyze.preset, Some("deep".into()));
    assert_eq!(cfg.analyze.window, Some(128_000));
    assert_eq!(cfg.context.budget, Some("100k".into()));
    assert_eq!(cfg.badge.metric, Some("code".into()));
    assert_eq!(cfg.gate.policy, Some("gate.json".into()));
    assert_eq!(cfg.gate.fail_fast, Some(true));

    let llm = cfg.view.get("llm").expect("llm profile");
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(10));

    let ci = cfg.view.get("ci").expect("ci profile");
    assert_eq!(ci.format.as_deref(), Some("tsv"));
    assert_eq!(ci.preset.as_deref(), Some("health"));
}

#[test]
fn toml_gate_with_inline_rules() {
    let toml_str = r#"
[gate]
policy = "gate.json"

[[gate.rules]]
name = "min-code"
pointer = "/summary/code"
op = ">="
value = 100

[[gate.rules]]
name = "lang-check"
pointer = "/summary/language"
op = "in"
values = ["Rust", "Go"]
negate = false
level = "error"
message = "Unexpected language"

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
max_value = 20.0
level = "warn"
description = "Cyclomatic complexity ceiling"
"#;
    let cfg = toml_roundtrip(toml_str);
    let rules = cfg.gate.rules.as_ref().expect("rules");
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "min-code");
    assert_eq!(rules[0].pointer, "/summary/code");
    assert_eq!(rules[0].op, ">=");
    assert_eq!(rules[1].values.as_ref().unwrap().len(), 2);
    assert!(!rules[1].negate);

    let ratchet = cfg.gate.ratchet.as_ref().expect("ratchet");
    assert_eq!(ratchet.len(), 1);
    assert_eq!(ratchet[0].pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchet[0].max_increase_pct, Some(10.0));
    assert_eq!(ratchet[0].max_value, Some(20.0));
    assert_eq!(ratchet[0].level.as_deref(), Some("warn"));
}

#[test]
fn toml_view_profile_all_fields() {
    let toml_str = r#"
[view.full]
format = "json"
top = 20
files = true
module_roots = ["crates"]
module_depth = 3
min_code = 1
max_rows = 500
redact = "all"
meta = true
children = "collapse"
preset = "deep"
window = 64000
budget = "200k"
strategy = "greedy"
rank_by = "churn"
output = "bundle"
compress = true
metric = "tokens"
"#;
    let cfg = toml_roundtrip(toml_str);
    let p = cfg.view.get("full").expect("full profile");
    assert_eq!(p.format.as_deref(), Some("json"));
    assert_eq!(p.top, Some(20));
    assert_eq!(p.files, Some(true));
    assert_eq!(p.module_depth, Some(3));
    assert_eq!(p.min_code, Some(1));
    assert_eq!(p.max_rows, Some(500));
    assert_eq!(p.compress, Some(true));
    assert_eq!(p.metric.as_deref(), Some("tokens"));
}

// =============================================================================
// GateRule and RatchetRuleConfig standalone serde
// =============================================================================

#[test]
fn gate_rule_roundtrip() {
    let rule = GateRule {
        name: "test-rule".into(),
        pointer: "/summary/code".into(),
        op: ">=".into(),
        value: Some(serde_json::json!(42)),
        values: None,
        negate: true,
        level: Some("warn".into()),
        message: Some("Code too low".into()),
    };
    let back = json_roundtrip(&rule);
    assert_eq!(back.name, "test-rule");
    assert_eq!(back.pointer, "/summary/code");
    assert!(back.negate);
    assert_eq!(back.value, Some(serde_json::json!(42)));
    assert_eq!(back.message.as_deref(), Some("Code too low"));
}

#[test]
fn ratchet_rule_config_roundtrip() {
    let rule = RatchetRuleConfig {
        pointer: "/metrics/churn".into(),
        max_increase_pct: Some(5.5),
        max_value: Some(100.0),
        level: Some("error".into()),
        description: Some("Churn ceiling".into()),
    };
    let back = json_roundtrip(&rule);
    assert_eq!(back.pointer, "/metrics/churn");
    assert_eq!(back.max_increase_pct, Some(5.5));
    assert_eq!(back.max_value, Some(100.0));
    assert_eq!(back.description.as_deref(), Some("Churn ceiling"));
}

// =============================================================================
// Deserialize from partial / missing fields
// =============================================================================

#[test]
fn scan_options_from_empty_json() {
    let back: ScanOptions = serde_json::from_str("{}").expect("deserialize empty");
    assert!(back.excluded.is_empty());
    assert!(!back.hidden);
}

#[test]
fn lang_settings_from_minimal_json() {
    // Only required-ish fields; serde defaults fill the rest.
    let json = r#"{"top": 5}"#;
    let back: LangSettings = serde_json::from_str(json).expect("deserialize");
    assert_eq!(back.top, 5);
    assert!(!back.files);
}

#[test]
fn analyze_settings_from_partial_json() {
    let json = r#"{"preset": "health"}"#;
    let back: AnalyzeSettings = serde_json::from_str(json).expect("deserialize");
    assert_eq!(back.preset, "health");
    assert_eq!(back.granularity, "module"); // default
    assert!(back.window.is_none());
}
