//! Wave 43 deep tests for `tokmd-settings`.
//!
//! Covers: ScanOptions construction/defaults, feature flag combinations,
//! serde roundtrip for all settings types, BTreeMap ordering in serialized
//! output, TOML config parsing, ViewProfile merging, and edge cases.

use std::collections::BTreeMap;

use tokmd_settings::{
    AnalyzeSettings, ChildIncludeMode, ChildrenMode, CockpitSettings, ConfigMode, DiffSettings,
    ExportFormat, ExportSettings, LangSettings, ModuleSettings, RedactMode, ScanOptions,
    ScanSettings, TomlConfig, ViewProfile,
};

// =============================================================================
// 1. ScanOptions construction and defaults
// =============================================================================

#[test]
fn scan_options_default_all_booleans_false() {
    let opts = ScanOptions::default();
    assert!(!opts.hidden);
    assert!(!opts.no_ignore);
    assert!(!opts.no_ignore_parent);
    assert!(!opts.no_ignore_dot);
    assert!(!opts.no_ignore_vcs);
    assert!(!opts.treat_doc_strings_as_comments);
    assert!(opts.excluded.is_empty());
    assert!(matches!(opts.config, ConfigMode::Auto));
}

#[test]
fn scan_options_with_excludes_preserves_order() {
    let opts = ScanOptions {
        excluded: vec!["z_last".into(), "a_first".into(), "m_middle".into()],
        ..Default::default()
    };
    // Vec preserves insertion order, not sorted
    assert_eq!(opts.excluded[0], "z_last");
    assert_eq!(opts.excluded[1], "a_first");
    assert_eq!(opts.excluded[2], "m_middle");
}

#[test]
fn scan_options_all_ignore_flags_set() {
    let opts = ScanOptions {
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        ..Default::default()
    };
    assert!(opts.no_ignore);
    assert!(opts.no_ignore_parent);
    assert!(opts.no_ignore_dot);
    assert!(opts.no_ignore_vcs);
}

// =============================================================================
// 2. Feature flag combinations
// =============================================================================

#[test]
fn scan_options_hidden_and_no_ignore_independent() {
    let opts = ScanOptions {
        hidden: true,
        no_ignore: false,
        ..Default::default()
    };
    assert!(opts.hidden);
    assert!(!opts.no_ignore);
}

#[test]
fn config_mode_none_serializes_correctly() {
    let opts = ScanOptions {
        config: ConfigMode::None,
        ..Default::default()
    };
    let json = serde_json::to_string(&opts).unwrap();
    assert!(json.contains("\"none\"") || json.contains("\"None\""));
}

#[test]
fn analyze_settings_git_flag_combinations() {
    // git = None means unspecified
    let a1 = AnalyzeSettings::default();
    assert!(a1.git.is_none());

    // git = Some(true) means force-enable
    let a2 = AnalyzeSettings {
        git: Some(true),
        ..Default::default()
    };
    assert_eq!(a2.git, Some(true));

    // git = Some(false) means force-disable
    let a3 = AnalyzeSettings {
        git: Some(false),
        ..Default::default()
    };
    assert_eq!(a3.git, Some(false));
}

// =============================================================================
// 3. ScanSettings construction helpers
// =============================================================================

#[test]
fn scan_settings_current_dir_paths() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
    assert!(s.options.excluded.is_empty());
}

#[test]
fn scan_settings_for_paths_multiple() {
    let s = ScanSettings::for_paths(vec!["src".into(), "lib".into(), "tests".into()]);
    assert_eq!(s.paths.len(), 3);
    assert_eq!(s.paths[2], "tests");
    // Options should be default
    assert!(!s.options.hidden);
}

// =============================================================================
// 4. Serde roundtrip for all settings types
// =============================================================================

#[test]
fn serde_roundtrip_scan_options_all_fields() {
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
    let json = serde_json::to_string(&opts).unwrap();
    let back: ScanOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded.len(), 2);
    assert!(back.hidden);
    assert!(back.no_ignore);
    assert!(back.treat_doc_strings_as_comments);
}

#[test]
fn serde_roundtrip_scan_settings_flatten_works() {
    let s = ScanSettings {
        paths: vec!["src".into()],
        options: ScanOptions {
            hidden: true,
            excluded: vec!["*.bak".into()],
            ..Default::default()
        },
    };
    let json = serde_json::to_string(&s).unwrap();
    // Flatten means hidden appears at top level in JSON, not nested
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["hidden"], true);
    assert!(v["options"].is_null()); // flattened, no "options" key

    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert!(back.options.hidden);
    assert_eq!(back.options.excluded, vec!["*.bak"]);
}

#[test]
fn serde_roundtrip_lang_settings() {
    let s = LangSettings {
        top: 5,
        files: true,
        children: ChildrenMode::Separate,
        redact: Some(RedactMode::Paths),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: LangSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 5);
    assert!(back.files);
    assert!(matches!(back.children, ChildrenMode::Separate));
    assert_eq!(back.redact, Some(RedactMode::Paths));
}

#[test]
fn serde_roundtrip_module_settings() {
    let s = ModuleSettings {
        top: 3,
        module_roots: vec!["src".into(), "packages".into()],
        module_depth: 4,
        children: ChildIncludeMode::ParentsOnly,
        redact: Some(RedactMode::All),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 3);
    assert_eq!(back.module_depth, 4);
    assert_eq!(back.module_roots, vec!["src", "packages"]);
    assert!(matches!(back.children, ChildIncludeMode::ParentsOnly));
    assert_eq!(back.redact, Some(RedactMode::All));
}

#[test]
fn serde_roundtrip_export_settings() {
    let s = ExportSettings {
        format: ExportFormat::Csv,
        min_code: 10,
        max_rows: 100,
        redact: RedactMode::Paths,
        meta: false,
        strip_prefix: Some("/home/user/project".into()),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert!(matches!(back.format, ExportFormat::Csv));
    assert_eq!(back.min_code, 10);
    assert_eq!(back.max_rows, 100);
    assert!(!back.meta);
    assert_eq!(back.strip_prefix, Some("/home/user/project".into()));
}

#[test]
fn serde_roundtrip_analyze_settings_with_limits() {
    let s = AnalyzeSettings {
        preset: "deep".into(),
        window: Some(128_000),
        git: Some(true),
        max_files: Some(5000),
        max_bytes: Some(50_000_000),
        max_file_bytes: Some(1_000_000),
        max_commits: Some(500),
        max_commit_files: Some(100),
        granularity: "file".into(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "deep");
    assert_eq!(back.window, Some(128_000));
    assert_eq!(back.git, Some(true));
    assert_eq!(back.max_files, Some(5000));
    assert_eq!(back.max_bytes, Some(50_000_000));
    assert_eq!(back.max_commit_files, Some(100));
    assert_eq!(back.granularity, "file");
}

#[test]
fn serde_roundtrip_cockpit_settings() {
    let s = CockpitSettings {
        base: "v2.0".into(),
        head: "feature-branch".into(),
        range_mode: "three-dot".into(),
        baseline: Some("baseline.json".into()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "v2.0");
    assert_eq!(back.head, "feature-branch");
    assert_eq!(back.range_mode, "three-dot");
    assert_eq!(back.baseline.as_deref(), Some("baseline.json"));
}

#[test]
fn serde_roundtrip_diff_settings() {
    let s = DiffSettings {
        from: "v1.0.0".into(),
        to: "v2.0.0".into(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "v1.0.0");
    assert_eq!(back.to, "v2.0.0");
}

// =============================================================================
// 5. BTreeMap ordering in serialized output
// =============================================================================

#[test]
fn toml_config_view_profiles_btreemap_ordering() {
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
    // BTreeMap should sort alphabetically
    assert_eq!(keys, vec!["alpha", "middle", "zebra"]);
}

#[test]
fn view_profile_serialization_preserves_btreemap_order() {
    let mut profiles = BTreeMap::new();
    profiles.insert(
        "z_profile".to_string(),
        ViewProfile {
            format: Some("json".into()),
            ..Default::default()
        },
    );
    profiles.insert(
        "a_profile".to_string(),
        ViewProfile {
            format: Some("md".into()),
            ..Default::default()
        },
    );

    let json = serde_json::to_string(&profiles).unwrap();
    // "a_profile" should appear before "z_profile" in JSON
    let a_pos = json.find("a_profile").unwrap();
    let z_pos = json.find("z_profile").unwrap();
    assert!(a_pos < z_pos, "BTreeMap should produce sorted JSON keys");
}

// =============================================================================
// 6. TOML parsing – full config
// =============================================================================

#[test]
fn toml_parse_full_config() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target", "node_modules"]
hidden = true
config = "none"
no_ignore = false
no_ignore_parent = true
no_ignore_dot = false
no_ignore_vcs = true
doc_comments = true

[module]
roots = ["crates", "packages", "libs"]
depth = 3
children = "collapse"

[export]
min_code = 5
max_rows = 1000
redact = "paths"
format = "csv"
children = "separate"

[analyze]
preset = "risk"
window = 128000
format = "json"
git = true
max_files = 5000
max_bytes = 50000000
max_file_bytes = 1000000
max_commits = 500
max_commit_files = 100
granularity = "file"

[context]
budget = "256k"
strategy = "spread"
rank_by = "hotspot"
output = "bundle"
compress = true

[badge]
metric = "tokens"

[gate]
policy = "policy.toml"
baseline = "baseline.json"
preset = "health"
fail_fast = true
"#;
    let config = TomlConfig::parse(toml_str).unwrap();

    // Scan
    assert_eq!(
        config.scan.paths,
        Some(vec!["src".to_string(), "lib".to_string()])
    );
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.no_ignore_vcs, Some(true));
    assert_eq!(config.scan.doc_comments, Some(true));

    // Module
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(config.module.children, Some("collapse".to_string()));

    // Export
    assert_eq!(config.export.min_code, Some(5));
    assert_eq!(config.export.max_rows, Some(1000));
    assert_eq!(config.export.format, Some("csv".to_string()));

    // Analyze
    assert_eq!(config.analyze.preset, Some("risk".to_string()));
    assert_eq!(config.analyze.window, Some(128000));
    assert_eq!(config.analyze.git, Some(true));
    assert_eq!(config.analyze.max_files, Some(5000));
    assert_eq!(config.analyze.granularity, Some("file".to_string()));

    // Context
    assert_eq!(config.context.budget, Some("256k".to_string()));
    assert_eq!(config.context.strategy, Some("spread".to_string()));
    assert_eq!(config.context.compress, Some(true));

    // Badge
    assert_eq!(config.badge.metric, Some("tokens".to_string()));

    // Gate
    assert_eq!(config.gate.policy, Some("policy.toml".to_string()));
    assert_eq!(config.gate.fail_fast, Some(true));
}

#[test]
fn toml_parse_gate_rules_inline() {
    let toml_str = r#"
[[gate.rules]]
name = "max_complexity"
pointer = "/complexity/avg_cyclomatic"
op = "<="
value = 10.0

[[gate.rules]]
name = "min_coverage"
pointer = "/coverage/line_pct"
op = ">="
value = 80.0
negate = false
level = "error"
message = "Coverage too low"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let rules = config.gate.rules.unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "max_complexity");
    assert_eq!(rules[0].op, "<=");
    assert_eq!(rules[1].level, Some("error".to_string()));
    assert_eq!(rules[1].message, Some("Coverage too low".to_string()));
}

#[test]
fn toml_parse_ratchet_rules() {
    let toml_str = r#"
[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
max_value = 15.0
level = "warn"
description = "Cyclomatic complexity guard"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let ratchet = config.gate.ratchet.unwrap();
    assert_eq!(ratchet.len(), 1);
    assert_eq!(ratchet[0].pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchet[0].max_increase_pct, Some(5.0));
    assert_eq!(ratchet[0].max_value, Some(15.0));
    assert_eq!(ratchet[0].level, Some("warn".to_string()));
}

// =============================================================================
// 7. Default invariants
// =============================================================================

#[test]
fn module_settings_default_roots_are_crates_packages() {
    let ms = ModuleSettings::default();
    assert_eq!(ms.module_roots, vec!["crates", "packages"]);
    assert_eq!(ms.module_depth, 2);
}

#[test]
fn export_settings_default_format_is_jsonl() {
    let es = ExportSettings::default();
    assert!(matches!(es.format, ExportFormat::Jsonl));
    assert!(es.meta);
    assert_eq!(es.min_code, 0);
    assert_eq!(es.max_rows, 0);
    assert!(matches!(es.redact, RedactMode::None));
    assert!(es.strip_prefix.is_none());
}

#[test]
fn analyze_settings_default_preset_is_receipt() {
    let a = AnalyzeSettings::default();
    assert_eq!(a.preset, "receipt");
    assert_eq!(a.granularity, "module");
    assert!(a.window.is_none());
    assert!(a.git.is_none());
    assert!(a.max_files.is_none());
}

#[test]
fn cockpit_settings_defaults() {
    let c = CockpitSettings::default();
    assert_eq!(c.base, "main");
    assert_eq!(c.head, "HEAD");
    assert_eq!(c.range_mode, "two-dot");
    assert!(c.baseline.is_none());
}

// =============================================================================
// 8. Clone independence
// =============================================================================

#[test]
fn scan_options_clone_is_independent() {
    let mut original = ScanOptions {
        excluded: vec!["target".into()],
        hidden: true,
        ..Default::default()
    };
    let cloned = original.clone();
    original.excluded.push("vendor".into());
    original.hidden = false;

    assert_eq!(cloned.excluded, vec!["target"]);
    assert!(cloned.hidden);
}

// =============================================================================
// 9. JSON deserialization from partial input
// =============================================================================

#[test]
fn lang_settings_from_partial_json() {
    let json = r#"{"top": 3}"#;
    let s: LangSettings = serde_json::from_str(json).unwrap();
    assert_eq!(s.top, 3);
    assert!(!s.files);
    assert!(matches!(s.children, ChildrenMode::Collapse));
    assert!(s.redact.is_none());
}

#[test]
fn module_settings_from_partial_json() {
    let json = r#"{"module_depth": 5}"#;
    let s: ModuleSettings = serde_json::from_str(json).unwrap();
    assert_eq!(s.module_depth, 5);
    assert_eq!(s.top, 0);
    // module_roots uses default when missing
    assert_eq!(s.module_roots, vec!["crates", "packages"]);
}

#[test]
fn export_settings_from_partial_json() {
    let json = r#"{"min_code": 42, "meta": false}"#;
    let s: ExportSettings = serde_json::from_str(json).unwrap();
    assert_eq!(s.min_code, 42);
    assert!(!s.meta);
    // format should use default
    assert!(matches!(s.format, ExportFormat::Jsonl));
}

// =============================================================================
// 10. TOML config from file
// =============================================================================

#[test]
fn toml_config_from_file_reads_correctly() {
    let content = r#"
[scan]
hidden = true
exclude = ["vendor"]

[module]
depth = 4

[view.ci]
format = "json"
top = 5
"#;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut tmp, content.as_bytes()).unwrap();

    let config = TomlConfig::from_file(tmp.path()).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.module.depth, Some(4));
    let ci = config.view.get("ci").unwrap();
    assert_eq!(ci.format.as_deref(), Some("json"));
    assert_eq!(ci.top, Some(5));
}

#[test]
fn toml_config_from_nonexistent_file_errors() {
    let result = TomlConfig::from_file(std::path::Path::new("/nonexistent/tokmd.toml"));
    assert!(result.is_err());
}

#[test]
fn toml_config_invalid_toml_errors() {
    let result = TomlConfig::parse("this is [[[not valid toml");
    assert!(result.is_err());
}
