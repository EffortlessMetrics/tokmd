//! Deep tests for tokmd-settings – wave 39.

use tokmd_settings::{
    AnalyzeSettings, ChildIncludeMode, ChildrenMode, CockpitSettings, ConfigMode, DiffSettings,
    ExportFormat, ExportSettings, LangSettings, ModuleSettings, RedactMode, ScanOptions,
    ScanSettings, TomlConfig,
};

// ---------------------------------------------------------------------------
// ScanOptions construction and defaults
// ---------------------------------------------------------------------------

#[test]
fn scan_options_default_values() {
    let o = ScanOptions::default();
    assert!(o.excluded.is_empty());
    assert!(!o.hidden);
    assert!(!o.no_ignore);
    assert!(!o.no_ignore_parent);
    assert!(!o.no_ignore_dot);
    assert!(!o.no_ignore_vcs);
    assert!(!o.treat_doc_strings_as_comments);
    // ConfigMode default
    let json = serde_json::to_value(o.config).unwrap();
    assert_eq!(json, serde_json::to_value(ConfigMode::default()).unwrap());
}

#[test]
fn scan_settings_current_dir() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
}

#[test]
fn scan_settings_for_paths() {
    let s = ScanSettings::for_paths(vec!["src".into(), "lib".into(), "tests".into()]);
    assert_eq!(s.paths.len(), 3);
    assert_eq!(s.paths[2], "tests");
}

// ---------------------------------------------------------------------------
// All settings types – defaults
// ---------------------------------------------------------------------------

#[test]
fn lang_settings_defaults() {
    let s = LangSettings::default();
    assert_eq!(s.top, 0);
    assert!(!s.files);
    assert!(matches!(s.children, ChildrenMode::Collapse));
    assert!(s.redact.is_none());
}

#[test]
fn module_settings_defaults() {
    let s = ModuleSettings::default();
    assert_eq!(s.top, 0);
    assert_eq!(s.module_roots, vec!["crates", "packages"]);
    assert_eq!(s.module_depth, 2);
    assert!(matches!(s.children, ChildIncludeMode::Separate));
    assert!(s.redact.is_none());
}

#[test]
fn export_settings_defaults() {
    let s = ExportSettings::default();
    assert!(matches!(s.format, ExportFormat::Jsonl));
    assert_eq!(s.min_code, 0);
    assert_eq!(s.max_rows, 0);
    assert!(matches!(s.redact, RedactMode::None));
    assert!(s.meta);
    assert!(s.strip_prefix.is_none());
    assert_eq!(s.module_depth, 2);
}

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
fn cockpit_settings_defaults() {
    let s = CockpitSettings::default();
    assert_eq!(s.base, "main");
    assert_eq!(s.head, "HEAD");
    assert_eq!(s.range_mode, "two-dot");
    assert!(s.baseline.is_none());
}

#[test]
fn diff_settings_defaults() {
    let s = DiffSettings::default();
    assert!(s.from.is_empty());
    assert!(s.to.is_empty());
}

// ---------------------------------------------------------------------------
// Serde roundtrips
// ---------------------------------------------------------------------------

#[test]
fn serde_roundtrip_scan_options_full() {
    let o = ScanOptions {
        excluded: vec!["target".into(), "*.bak".into()],
        config: ConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };
    let json = serde_json::to_string(&o).unwrap();
    let back: ScanOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded.len(), 2);
    assert!(back.hidden);
    assert!(back.no_ignore);
    assert!(back.no_ignore_parent);
    assert!(back.no_ignore_dot);
    assert!(back.no_ignore_vcs);
    assert!(back.treat_doc_strings_as_comments);
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
}

#[test]
fn serde_roundtrip_module_settings() {
    let s = ModuleSettings {
        top: 3,
        module_roots: vec!["src".into()],
        module_depth: 4,
        children: ChildIncludeMode::Separate,
        redact: Some(RedactMode::All),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 3);
    assert_eq!(back.module_roots, vec!["src"]);
    assert_eq!(back.module_depth, 4);
}

#[test]
fn serde_roundtrip_export_settings() {
    let s = ExportSettings {
        format: ExportFormat::Csv,
        min_code: 10,
        max_rows: 100,
        redact: RedactMode::Paths,
        meta: false,
        strip_prefix: Some("vendor/".into()),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.min_code, 10);
    assert_eq!(back.max_rows, 100);
    assert!(!back.meta);
    assert_eq!(back.strip_prefix.as_deref(), Some("vendor/"));
}

#[test]
fn serde_roundtrip_analyze_settings_with_limits() {
    let s = AnalyzeSettings {
        preset: "deep".into(),
        window: Some(128_000),
        git: Some(true),
        max_files: Some(5000),
        max_bytes: Some(10_000_000),
        max_file_bytes: Some(500_000),
        max_commits: Some(1000),
        max_commit_files: Some(50),
        granularity: "file".into(),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "deep");
    assert_eq!(back.window, Some(128_000));
    assert_eq!(back.git, Some(true));
    assert_eq!(back.max_files, Some(5000));
    assert_eq!(back.granularity, "file");
}

#[test]
fn serde_roundtrip_cockpit_with_baseline() {
    let s = CockpitSettings {
        base: "v1.0".into(),
        head: "feature-branch".into(),
        range_mode: "three-dot".into(),
        baseline: Some("baseline.json".into()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "v1.0");
    assert_eq!(back.baseline.as_deref(), Some("baseline.json"));
}

#[test]
fn serde_roundtrip_diff_settings() {
    let s = DiffSettings {
        from: "v1.0".into(),
        to: "v2.0".into(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "v1.0");
    assert_eq!(back.to, "v2.0");
}

// ---------------------------------------------------------------------------
// ScanSettings flatten
// ---------------------------------------------------------------------------

#[test]
fn scan_settings_flatten_serde() {
    let s = ScanSettings {
        paths: vec!["src".into()],
        options: ScanOptions {
            hidden: true,
            excluded: vec!["node_modules".into()],
            ..Default::default()
        },
    };
    let json = serde_json::to_string(&s).unwrap();
    // Flattened: options fields appear at top level alongside paths
    assert!(json.contains("\"hidden\":true"));
    assert!(json.contains("\"paths\""));
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert!(back.options.hidden);
    assert_eq!(back.options.excluded, vec!["node_modules"]);
}

// ---------------------------------------------------------------------------
// TomlConfig parsing
// ---------------------------------------------------------------------------

#[test]
fn toml_config_empty_string_parses() {
    let c = TomlConfig::parse("").unwrap();
    assert!(c.scan.hidden.is_none());
    assert!(c.module.roots.is_none());
    assert!(c.view.is_empty());
}

#[test]
fn toml_config_full_parse() {
    let toml = r#"
[scan]
hidden = true
exclude = ["target", "node_modules"]
no_ignore = false

[module]
roots = ["crates"]
depth = 3

[export]
min_code = 5
format = "csv"

[analyze]
preset = "deep"
window = 128000
git = true

[context]
budget = "100k"
strategy = "greedy"

[gate]
fail_fast = true

[[gate.rules]]
name = "no-large-files"
pointer = "/export/max_code"
op = "le"
value = 1000

[view.ci]
format = "json"
top = 20
"#;
    let c = TomlConfig::parse(toml).unwrap();
    assert_eq!(c.scan.hidden, Some(true));
    assert_eq!(
        c.scan.exclude,
        Some(vec!["target".into(), "node_modules".into()])
    );
    assert_eq!(c.module.depth, Some(3));
    assert_eq!(c.export.min_code, Some(5));
    assert_eq!(c.analyze.preset.as_deref(), Some("deep"));
    assert_eq!(c.context.budget.as_deref(), Some("100k"));
    assert_eq!(c.gate.fail_fast, Some(true));
    assert_eq!(c.gate.rules.as_ref().unwrap().len(), 1);
    let ci = c.view.get("ci").unwrap();
    assert_eq!(ci.format.as_deref(), Some("json"));
    assert_eq!(ci.top, Some(20));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn export_settings_zero_limits() {
    let s = ExportSettings {
        min_code: 0,
        max_rows: 0,
        ..Default::default()
    };
    assert_eq!(s.min_code, 0);
    assert_eq!(s.max_rows, 0);
}

#[test]
fn analyze_settings_all_none_limits() {
    let s = AnalyzeSettings::default();
    assert!(s.window.is_none());
    assert!(s.git.is_none());
    assert!(s.max_files.is_none());
    assert!(s.max_bytes.is_none());
    assert!(s.max_file_bytes.is_none());
    assert!(s.max_commits.is_none());
    assert!(s.max_commit_files.is_none());
}

#[test]
fn toml_config_gate_ratchet_rules() {
    let toml = r#"
[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
max_value = 20.0
level = "warn"
description = "Keep complexity stable"
"#;
    let c = TomlConfig::parse(toml).unwrap();
    let ratchet = c.gate.ratchet.as_ref().unwrap();
    assert_eq!(ratchet.len(), 1);
    assert_eq!(ratchet[0].pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchet[0].max_increase_pct, Some(10.0));
    assert_eq!(ratchet[0].max_value, Some(20.0));
    assert_eq!(ratchet[0].level.as_deref(), Some("warn"));
}
