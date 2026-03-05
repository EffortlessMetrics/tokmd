//! W64 contract tests for `tokmd-settings`.
//!
//! ~60 tests covering settings construction, defaults, serde round-trips,
//! TOML configuration, property-based validation, BDD-style scenarios,
//! edge cases, and boundary conditions.

use std::collections::BTreeMap;

use tokmd_settings::*;

// ═══════════════════════════════════════════════════════════════════════════════
// 1. ScanOptions construction and defaults
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scan_options_default_all_false() {
    let opts = ScanOptions::default();
    assert!(opts.excluded.is_empty());
    assert!(!opts.hidden);
    assert!(!opts.no_ignore);
    assert!(!opts.no_ignore_parent);
    assert!(!opts.no_ignore_dot);
    assert!(!opts.no_ignore_vcs);
    assert!(!opts.treat_doc_strings_as_comments);
    assert_eq!(opts.config, ConfigMode::Auto);
}

#[test]
fn scan_options_with_excludes() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string(), "node_modules".to_string()],
        ..Default::default()
    };
    assert_eq!(opts.excluded.len(), 2);
    assert_eq!(opts.excluded[0], "target");
    assert_eq!(opts.excluded[1], "node_modules");
}

#[test]
fn scan_options_all_flags_true() {
    let opts = ScanOptions {
        excluded: vec![],
        config: ConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };
    assert!(opts.hidden);
    assert!(opts.no_ignore);
    assert!(opts.no_ignore_parent);
    assert!(opts.no_ignore_dot);
    assert!(opts.no_ignore_vcs);
    assert!(opts.treat_doc_strings_as_comments);
    assert_eq!(opts.config, ConfigMode::None);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. ScanSettings construction
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scan_settings_default_has_empty_paths() {
    let s = ScanSettings::default();
    assert!(s.paths.is_empty());
}

#[test]
fn scan_settings_current_dir() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, [".".to_string()]);
    assert!(!s.options.hidden);
}

#[test]
fn scan_settings_for_paths() {
    let s = ScanSettings::for_paths(vec!["src".into(), "lib".into(), "tests".into()]);
    assert_eq!(s.paths.len(), 3);
    assert_eq!(s.paths[0], "src");
    assert_eq!(s.paths[2], "tests");
}

#[test]
fn scan_settings_options_are_accessible() {
    let s = ScanSettings {
        paths: vec![".".into()],
        options: ScanOptions {
            hidden: true,
            no_ignore_vcs: true,
            ..Default::default()
        },
    };
    assert!(s.options.hidden);
    assert!(s.options.no_ignore_vcs);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. LangSettings defaults
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn lang_settings_defaults() {
    let s = LangSettings::default();
    assert_eq!(s.top, 0);
    assert!(!s.files);
    assert_eq!(s.children, ChildrenMode::Collapse);
    assert!(s.redact.is_none());
}

#[test]
fn lang_settings_custom() {
    let s = LangSettings {
        top: 10,
        files: true,
        children: ChildrenMode::Separate,
        redact: Some(RedactMode::Paths),
    };
    assert_eq!(s.top, 10);
    assert!(s.files);
    assert_eq!(s.children, ChildrenMode::Separate);
    assert_eq!(s.redact.unwrap(), RedactMode::Paths);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. ModuleSettings defaults
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn module_settings_defaults() {
    let s = ModuleSettings::default();
    assert_eq!(s.top, 0);
    assert_eq!(s.module_roots, ["crates", "packages"]);
    assert_eq!(s.module_depth, 2);
    assert_eq!(s.children, ChildIncludeMode::Separate);
    assert!(s.redact.is_none());
}

#[test]
fn module_settings_custom_roots() {
    let s = ModuleSettings {
        module_roots: vec!["src".to_string(), "internal".to_string()],
        module_depth: 3,
        ..Default::default()
    };
    assert_eq!(s.module_roots.len(), 2);
    assert_eq!(s.module_depth, 3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. ExportSettings defaults
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn export_settings_defaults() {
    let s = ExportSettings::default();
    assert_eq!(s.format, ExportFormat::Jsonl);
    assert_eq!(s.module_roots, ["crates", "packages"]);
    assert_eq!(s.module_depth, 2);
    assert_eq!(s.children, ChildIncludeMode::Separate);
    assert_eq!(s.min_code, 0);
    assert_eq!(s.max_rows, 0);
    assert_eq!(s.redact, RedactMode::None);
    assert!(s.meta);
    assert!(s.strip_prefix.is_none());
}

#[test]
fn export_settings_with_strip_prefix() {
    let s = ExportSettings {
        strip_prefix: Some("/home/user/project".to_string()),
        ..Default::default()
    };
    assert_eq!(s.strip_prefix.as_deref(), Some("/home/user/project"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. AnalyzeSettings defaults
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn analyze_settings_defaults() {
    let s = AnalyzeSettings::default();
    assert_eq!(s.preset, "receipt");
    assert_eq!(s.granularity, "module");
    assert!(s.window.is_none());
    assert!(s.git.is_none());
    assert!(s.max_files.is_none());
    assert!(s.max_bytes.is_none());
    assert!(s.max_file_bytes.is_none());
    assert!(s.max_commits.is_none());
    assert!(s.max_commit_files.is_none());
}

#[test]
fn analyze_settings_with_limits() {
    let s = AnalyzeSettings {
        preset: "deep".to_string(),
        max_files: Some(1000),
        max_bytes: Some(50_000_000),
        max_file_bytes: Some(1_000_000),
        max_commits: Some(500),
        max_commit_files: Some(100),
        git: Some(true),
        window: Some(128_000),
        granularity: "file".to_string(),
    };
    assert_eq!(s.preset, "deep");
    assert_eq!(s.max_files, Some(1000));
    assert_eq!(s.window, Some(128_000));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. CockpitSettings defaults
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_settings_defaults() {
    let s = CockpitSettings::default();
    assert_eq!(s.base, "main");
    assert_eq!(s.head, "HEAD");
    assert_eq!(s.range_mode, "two-dot");
    assert!(s.baseline.is_none());
}

#[test]
fn cockpit_settings_three_dot() {
    let s = CockpitSettings {
        base: "origin/main".to_string(),
        head: "feature-branch".to_string(),
        range_mode: "three-dot".to_string(),
        baseline: Some("baseline.json".to_string()),
    };
    assert_eq!(s.range_mode, "three-dot");
    assert!(s.baseline.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. DiffSettings
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn diff_settings_default() {
    let s = DiffSettings::default();
    assert!(s.from.is_empty());
    assert!(s.to.is_empty());
}

#[test]
fn diff_settings_with_refs() {
    let s = DiffSettings {
        from: "v1.0.0".to_string(),
        to: "v2.0.0".to_string(),
    };
    assert_eq!(s.from, "v1.0.0");
    assert_eq!(s.to, "v2.0.0");
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. Serde round-trips
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn serde_roundtrip_scan_options() {
    let opts = ScanOptions {
        excluded: vec!["target".into(), "*.bak".into()],
        config: ConfigMode::None,
        hidden: true,
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };
    let json = serde_json::to_string(&opts).unwrap();
    let back: ScanOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded, opts.excluded);
    assert!(back.hidden);
    assert!(back.no_ignore_parent);
    assert!(back.no_ignore_vcs);
    assert!(back.treat_doc_strings_as_comments);
    assert_eq!(back.config, ConfigMode::None);
}

#[test]
fn serde_roundtrip_scan_settings() {
    let s = ScanSettings {
        paths: vec!["src".into(), "lib".into()],
        options: ScanOptions {
            excluded: vec!["vendor".into()],
            hidden: true,
            ..Default::default()
        },
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths, s.paths);
    assert_eq!(back.options.excluded, s.options.excluded);
    assert!(back.options.hidden);
}

#[test]
fn serde_roundtrip_lang_settings() {
    let s = LangSettings {
        top: 5,
        files: true,
        children: ChildrenMode::Separate,
        redact: Some(RedactMode::All),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: LangSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 5);
    assert!(back.files);
    assert_eq!(back.children, ChildrenMode::Separate);
    assert_eq!(back.redact, Some(RedactMode::All));
}

#[test]
fn serde_roundtrip_module_settings() {
    let s = ModuleSettings {
        top: 20,
        module_roots: vec!["packages".to_string()],
        module_depth: 4,
        children: ChildIncludeMode::ParentsOnly,
        redact: Some(RedactMode::Paths),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 20);
    assert_eq!(back.module_depth, 4);
    assert_eq!(back.children, ChildIncludeMode::ParentsOnly);
}

#[test]
fn serde_roundtrip_export_settings() {
    let s = ExportSettings {
        format: ExportFormat::Csv,
        min_code: 10,
        max_rows: 500,
        redact: RedactMode::All,
        meta: false,
        strip_prefix: Some("/usr/src".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format, ExportFormat::Csv);
    assert_eq!(back.min_code, 10);
    assert_eq!(back.max_rows, 500);
    assert!(!back.meta);
    assert_eq!(back.strip_prefix.as_deref(), Some("/usr/src"));
}

#[test]
fn serde_roundtrip_analyze_settings() {
    let s = AnalyzeSettings {
        preset: "health".to_string(),
        window: Some(200_000),
        git: Some(false),
        max_files: Some(5000),
        max_bytes: Some(100_000_000),
        max_file_bytes: Some(2_000_000),
        max_commits: Some(1000),
        max_commit_files: Some(200),
        granularity: "file".to_string(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "health");
    assert_eq!(back.window, Some(200_000));
    assert_eq!(back.git, Some(false));
}

#[test]
fn serde_roundtrip_cockpit_settings() {
    let s = CockpitSettings {
        base: "v1.0".to_string(),
        head: "feature".to_string(),
        range_mode: "three-dot".to_string(),
        baseline: Some("bl.json".to_string()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "v1.0");
    assert_eq!(back.baseline, Some("bl.json".to_string()));
}

#[test]
fn serde_roundtrip_diff_settings() {
    let s = DiffSettings {
        from: "abc123".to_string(),
        to: "def456".to_string(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "abc123");
    assert_eq!(back.to, "def456");
}

// ═══════════════════════════════════════════════════════════════════════════════
// 10. TOML configuration
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn toml_config_default() {
    let config = TomlConfig::default();
    assert!(config.scan.paths.is_none());
    assert!(config.scan.hidden.is_none());
    assert!(config.module.roots.is_none());
    assert!(config.export.format.is_none());
    assert!(config.analyze.preset.is_none());
    assert!(config.view.is_empty());
}

#[test]
fn toml_config_parse_minimal() {
    let toml_str = "";
    let config = TomlConfig::parse(toml_str).unwrap();
    assert!(config.scan.paths.is_none());
}

#[test]
fn toml_config_parse_scan_section() {
    let toml_str = r#"
[scan]
hidden = true
exclude = ["target", "node_modules"]
config = "none"
no_ignore = true
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(
        config.scan.exclude,
        Some(vec!["target".to_string(), "node_modules".to_string()])
    );
    assert_eq!(config.scan.config, Some("none".to_string()));
    assert_eq!(config.scan.no_ignore, Some(true));
}

#[test]
fn toml_config_parse_module_section() {
    let toml_str = r#"
[module]
roots = ["src", "tests", "internal"]
depth = 3
children = "collapse"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(
        config.module.roots,
        Some(vec![
            "src".to_string(),
            "tests".to_string(),
            "internal".to_string()
        ])
    );
    assert_eq!(config.module.children, Some("collapse".to_string()));
}

#[test]
fn toml_config_parse_export_section() {
    let toml_str = r#"
[export]
min_code = 10
max_rows = 1000
redact = "paths"
format = "csv"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.export.min_code, Some(10));
    assert_eq!(config.export.max_rows, Some(1000));
    assert_eq!(config.export.redact, Some("paths".to_string()));
    assert_eq!(config.export.format, Some("csv".to_string()));
}

#[test]
fn toml_config_parse_analyze_section() {
    let toml_str = r#"
[analyze]
preset = "deep"
window = 128000
git = true
max_files = 5000
granularity = "file"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.analyze.preset, Some("deep".to_string()));
    assert_eq!(config.analyze.window, Some(128000));
    assert_eq!(config.analyze.git, Some(true));
    assert_eq!(config.analyze.max_files, Some(5000));
    assert_eq!(config.analyze.granularity, Some("file".to_string()));
}

#[test]
fn toml_config_parse_view_profiles() {
    let toml_str = r#"
[view.llm]
format = "json"
top = 15
budget = "128k"
strategy = "greedy"

[view.ci]
format = "md"
top = 5
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let llm = config.view.get("llm").unwrap();
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(15));
    assert_eq!(llm.budget.as_deref(), Some("128k"));
    assert_eq!(llm.strategy.as_deref(), Some("greedy"));

    let ci = config.view.get("ci").unwrap();
    assert_eq!(ci.format.as_deref(), Some("md"));
    assert_eq!(ci.top, Some(5));
}

#[test]
fn toml_config_parse_gate_section() {
    let toml_str = r#"
[gate]
policy = "policy.json"
fail_fast = true
allow_missing_baseline = false
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.gate.policy, Some("policy.json".to_string()));
    assert_eq!(config.gate.fail_fast, Some(true));
    assert_eq!(config.gate.allow_missing_baseline, Some(false));
}

#[test]
fn toml_config_parse_context_section() {
    let toml_str = r#"
[context]
budget = "64k"
strategy = "spread"
rank_by = "churn"
compress = true
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.context.budget, Some("64k".to_string()));
    assert_eq!(config.context.strategy, Some("spread".to_string()));
    assert_eq!(config.context.rank_by, Some("churn".to_string()));
    assert_eq!(config.context.compress, Some(true));
}

#[test]
fn toml_config_parse_gate_rules() {
    let toml_str = r#"
[gate]
[[gate.rules]]
name = "no-high-complexity"
pointer = "/complexity/max_cyclomatic"
op = "le"
value = 20

[[gate.rules]]
name = "min-test-ratio"
pointer = "/composition/test_ratio"
op = "ge"
value = 0.3
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let rules = config.gate.rules.unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "no-high-complexity");
    assert_eq!(rules[0].pointer, "/complexity/max_cyclomatic");
    assert_eq!(rules[0].op, "le");
    assert_eq!(rules[1].name, "min-test-ratio");
}

#[test]
fn toml_config_parse_ratchet_rules() {
    let toml_str = r#"
[gate]
[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
max_value = 15.0
level = "error"
description = "complexity must not regress"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let ratchets = config.gate.ratchet.unwrap();
    assert_eq!(ratchets.len(), 1);
    assert_eq!(ratchets[0].pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchets[0].max_increase_pct, Some(5.0));
    assert_eq!(ratchets[0].max_value, Some(15.0));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 11. TOML from file
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn toml_from_file_roundtrip() {
    let toml_content = r#"
[scan]
hidden = true
paths = ["src", "lib"]

[module]
depth = 4
"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tokmd.toml");
    std::fs::write(&path, toml_content).unwrap();

    let config = TomlConfig::from_file(&path).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.module.depth, Some(4));
}

#[test]
fn toml_from_file_missing_returns_error() {
    let result = TomlConfig::from_file(std::path::Path::new("/nonexistent/path/tokmd.toml"));
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 12. Property: all defaults produce valid settings
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn property_all_defaults_are_serializable() {
    // Every default settings struct must serialize to valid JSON
    let _j1 = serde_json::to_string(&ScanOptions::default()).unwrap();
    let _j2 = serde_json::to_string(&ScanSettings::default()).unwrap();
    let _j3 = serde_json::to_string(&LangSettings::default()).unwrap();
    let _j4 = serde_json::to_string(&ModuleSettings::default()).unwrap();
    let _j5 = serde_json::to_string(&ExportSettings::default()).unwrap();
    let _j6 = serde_json::to_string(&AnalyzeSettings::default()).unwrap();
    let _j7 = serde_json::to_string(&CockpitSettings::default()).unwrap();
    let _j8 = serde_json::to_string(&DiffSettings::default()).unwrap();
}

#[test]
fn property_all_defaults_roundtrip_through_json() {
    let json = serde_json::to_string(&LangSettings::default()).unwrap();
    let back: LangSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 0);

    let json = serde_json::to_string(&ModuleSettings::default()).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.module_depth, 2);

    let json = serde_json::to_string(&ExportSettings::default()).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format, ExportFormat::Jsonl);
}

#[test]
fn property_toml_config_default_roundtrips_through_toml() {
    let config = TomlConfig::default();
    let toml_str = toml::to_string(&config).unwrap();
    let back: TomlConfig = toml::from_str(&toml_str).unwrap();
    assert!(back.scan.paths.is_none());
    assert!(back.view.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 13. BDD: Given user input / When creating settings / Then defaults filled
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bdd_given_minimal_lang_when_default_then_collapse() {
    // Given: a user provides only top=5
    let s = LangSettings {
        top: 5,
        ..Default::default()
    };
    // Then: children defaults to Collapse
    assert_eq!(s.children, ChildrenMode::Collapse);
    // And: redact defaults to None
    assert!(s.redact.is_none());
}

#[test]
fn bdd_given_scan_paths_when_for_paths_then_defaults_filled() {
    // Given: user provides ["src"]
    let s = ScanSettings::for_paths(vec!["src".into()]);
    // Then: options are all defaults
    assert!(!s.options.hidden);
    assert!(!s.options.no_ignore);
    assert_eq!(s.options.config, ConfigMode::Auto);
}

#[test]
fn bdd_given_empty_toml_when_parsed_then_all_none() {
    // Given: an empty TOML string
    let config = TomlConfig::parse("").unwrap();
    // Then: all sections have None/empty values
    assert!(config.scan.paths.is_none());
    assert!(config.scan.hidden.is_none());
    assert!(config.module.roots.is_none());
    assert!(config.export.min_code.is_none());
    assert!(config.analyze.preset.is_none());
    assert!(config.context.budget.is_none());
    assert!(config.badge.metric.is_none());
    assert!(config.gate.policy.is_none());
    assert!(config.view.is_empty());
}

#[test]
fn bdd_given_full_toml_when_parsed_then_all_present() {
    let toml_str = r#"
[scan]
hidden = true
paths = ["."]

[module]
depth = 3
roots = ["crates"]

[export]
min_code = 5
max_rows = 100
format = "json"

[analyze]
preset = "risk"
window = 200000
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(config.export.min_code, Some(5));
    assert_eq!(config.analyze.preset, Some("risk".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 14. Edge: empty paths, special characters in paths
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn edge_empty_paths_vec() {
    let s = ScanSettings::for_paths(vec![]);
    assert!(s.paths.is_empty());
}

#[test]
fn edge_path_with_spaces() {
    let s = ScanSettings::for_paths(vec!["my project/src".into()]);
    assert_eq!(s.paths[0], "my project/src");
    let json = serde_json::to_string(&s).unwrap();
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths[0], "my project/src");
}

#[test]
fn edge_path_with_unicode() {
    let s = ScanSettings::for_paths(vec!["日本語/ソース".into()]);
    let json = serde_json::to_string(&s).unwrap();
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths[0], "日本語/ソース");
}

#[test]
fn edge_path_with_special_chars() {
    let s = ScanSettings::for_paths(vec!["path/with spaces & (parens)/[brackets]".into()]);
    let json = serde_json::to_string(&s).unwrap();
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths[0], "path/with spaces & (parens)/[brackets]");
}

#[test]
fn edge_empty_exclude_pattern() {
    let opts = ScanOptions {
        excluded: vec!["".to_string()],
        ..Default::default()
    };
    assert_eq!(opts.excluded.len(), 1);
    assert_eq!(opts.excluded[0], "");
}

#[test]
fn edge_export_settings_strip_prefix_empty() {
    let s = ExportSettings {
        strip_prefix: Some(String::new()),
        ..Default::default()
    };
    assert_eq!(s.strip_prefix.as_deref(), Some(""));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 15. Boundary: max depth, zero depth
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn boundary_module_depth_zero() {
    let s = ModuleSettings {
        module_depth: 0,
        ..Default::default()
    };
    assert_eq!(s.module_depth, 0);
    let json = serde_json::to_string(&s).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.module_depth, 0);
}

#[test]
fn boundary_module_depth_large() {
    let s = ModuleSettings {
        module_depth: usize::MAX,
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.module_depth, usize::MAX);
}

#[test]
fn boundary_export_min_code_max() {
    let s = ExportSettings {
        min_code: usize::MAX,
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.min_code, usize::MAX);
}

#[test]
fn boundary_export_max_rows_zero() {
    let s = ExportSettings {
        max_rows: 0,
        ..Default::default()
    };
    assert_eq!(s.max_rows, 0);
}

#[test]
fn boundary_analyze_window_zero() {
    let s = AnalyzeSettings {
        window: Some(0),
        ..Default::default()
    };
    assert_eq!(s.window, Some(0));
}

#[test]
fn boundary_lang_top_max() {
    let s = LangSettings {
        top: usize::MAX,
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: LangSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, usize::MAX);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 16. Deterministic JSON output
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn json_deterministic_scan_settings() {
    let s = ScanSettings::current_dir();
    let j1 = serde_json::to_string(&s).unwrap();
    let j2 = serde_json::to_string(&s).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn json_deterministic_toml_config() {
    let config = TomlConfig::default();
    let j1 = serde_json::to_string(&config).unwrap();
    let j2 = serde_json::to_string(&config).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn btreemap_view_profiles_sorted() {
    let mut views = BTreeMap::new();
    views.insert(
        "z_profile".to_string(),
        ViewProfile {
            format: Some("json".to_string()),
            ..Default::default()
        },
    );
    views.insert(
        "a_profile".to_string(),
        ViewProfile {
            format: Some("md".to_string()),
            ..Default::default()
        },
    );
    let config = TomlConfig {
        view: views,
        ..Default::default()
    };
    let json = serde_json::to_string(&config).unwrap();
    let a_pos = json.find("\"a_profile\"").unwrap();
    let z_pos = json.find("\"z_profile\"").unwrap();
    assert!(a_pos < z_pos, "BTreeMap must produce sorted keys");
}

// ═══════════════════════════════════════════════════════════════════════════════
// 17. ViewProfile defaults
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn view_profile_default_all_none() {
    let p = ViewProfile::default();
    assert!(p.format.is_none());
    assert!(p.top.is_none());
    assert!(p.files.is_none());
    assert!(p.module_roots.is_none());
    assert!(p.module_depth.is_none());
    assert!(p.min_code.is_none());
    assert!(p.max_rows.is_none());
    assert!(p.redact.is_none());
    assert!(p.meta.is_none());
    assert!(p.children.is_none());
    assert!(p.preset.is_none());
    assert!(p.window.is_none());
    assert!(p.budget.is_none());
    assert!(p.strategy.is_none());
    assert!(p.rank_by.is_none());
    assert!(p.output.is_none());
    assert!(p.compress.is_none());
    assert!(p.metric.is_none());
}

#[test]
fn view_profile_serde_roundtrip() {
    let p = ViewProfile {
        format: Some("json".to_string()),
        top: Some(10),
        files: Some(true),
        module_roots: Some(vec!["src".to_string()]),
        module_depth: Some(3),
        min_code: Some(5),
        max_rows: Some(100),
        redact: Some("all".to_string()),
        meta: Some(false),
        children: Some("collapse".to_string()),
        preset: Some("deep".to_string()),
        window: Some(128_000),
        budget: Some("64k".to_string()),
        strategy: Some("greedy".to_string()),
        rank_by: Some("churn".to_string()),
        output: Some("bundle".to_string()),
        compress: Some(true),
        metric: Some("code".to_string()),
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: ViewProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format, p.format);
    assert_eq!(back.top, p.top);
    assert_eq!(back.budget, p.budget);
    assert_eq!(back.metric, p.metric);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 18. GateRule and RatchetRuleConfig
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn gate_rule_serde_roundtrip() {
    let rule = GateRule {
        name: "max-complexity".to_string(),
        pointer: "/complexity/max_cyclomatic".to_string(),
        op: "le".to_string(),
        value: Some(serde_json::json!(25)),
        values: None,
        negate: false,
        level: Some("error".to_string()),
        message: Some("Complexity too high".to_string()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "max-complexity");
    assert_eq!(back.op, "le");
    assert!(!back.negate);
}

#[test]
fn gate_rule_with_negate() {
    let rule = GateRule {
        name: "not-zero".to_string(),
        pointer: "/total_files".to_string(),
        op: "eq".to_string(),
        value: Some(serde_json::json!(0)),
        values: None,
        negate: true,
        level: None,
        message: None,
    };
    assert!(rule.negate);
    let json = serde_json::to_string(&rule).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert!(back.negate);
}

#[test]
fn ratchet_rule_config_serde_roundtrip() {
    let rule = RatchetRuleConfig {
        pointer: "/health/score".to_string(),
        max_increase_pct: Some(10.0),
        max_value: Some(100.0),
        level: Some("warn".to_string()),
        description: Some("Health must not degrade".to_string()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: RatchetRuleConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.pointer, "/health/score");
    assert_eq!(back.max_increase_pct, Some(10.0));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 19. Re-exported enum types from tokmd-types
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn reexported_children_mode() {
    assert_eq!(ChildrenMode::Collapse, ChildrenMode::Collapse);
    assert_ne!(ChildrenMode::Collapse, ChildrenMode::Separate);
}

#[test]
fn reexported_child_include_mode() {
    assert_eq!(ChildIncludeMode::Separate, ChildIncludeMode::Separate);
    assert_ne!(ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly);
}

#[test]
fn reexported_config_mode_default() {
    assert_eq!(ConfigMode::default(), ConfigMode::Auto);
}

#[test]
fn reexported_export_format() {
    let _f = ExportFormat::Jsonl;
    let _c = ExportFormat::Csv;
    let _j = ExportFormat::Json;
    let _d = ExportFormat::Cyclonedx;
}

#[test]
fn reexported_redact_mode() {
    assert_ne!(RedactMode::None, RedactMode::Paths);
    assert_ne!(RedactMode::Paths, RedactMode::All);
}
