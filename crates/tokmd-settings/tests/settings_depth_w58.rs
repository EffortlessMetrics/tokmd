//! Depth tests for tokmd-settings (w58).
//!
//! Covers defaults, serde roundtrips, TOML parsing, edge-case values,
//! clone/debug, and profile merging.

use tokmd_settings::{
    AnalyzeSettings, ChildIncludeMode, ChildrenMode, CockpitSettings, ConfigMode, DiffSettings,
    ExportFormat, ExportSettings, LangSettings, ModuleSettings, RedactMode, ScanOptions,
    ScanSettings, TomlConfig, ViewProfile,
};

// ── ScanOptions defaults ────────────────────────────────────────────

#[test]
fn scan_options_all_defaults_are_falsy() {
    let opts = ScanOptions::default();
    assert!(opts.excluded.is_empty());
    assert!(!opts.hidden);
    assert!(!opts.no_ignore);
    assert!(!opts.no_ignore_parent);
    assert!(!opts.no_ignore_dot);
    assert!(!opts.no_ignore_vcs);
    assert!(!opts.treat_doc_strings_as_comments);
    assert!(matches!(opts.config, ConfigMode::Auto));
}

// ── ScanSettings constructors ───────────────────────────────────────

#[test]
fn scan_settings_current_dir_has_dot() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
    assert!(s.options.excluded.is_empty());
}

#[test]
fn scan_settings_for_paths_preserves_order() {
    let s = ScanSettings::for_paths(vec!["z".into(), "a".into(), "m".into()]);
    assert_eq!(s.paths, vec!["z", "a", "m"]);
}

#[test]
fn scan_settings_for_empty_paths() {
    let s = ScanSettings::for_paths(vec![]);
    assert!(s.paths.is_empty());
}

// ── LangSettings defaults ───────────────────────────────────────────

#[test]
fn lang_settings_default_values() {
    let s = LangSettings::default();
    assert_eq!(s.top, 0);
    assert!(!s.files);
    assert!(matches!(s.children, ChildrenMode::Collapse));
    assert!(s.redact.is_none());
}

// ── ModuleSettings defaults ─────────────────────────────────────────

#[test]
fn module_settings_default_values() {
    let s = ModuleSettings::default();
    assert_eq!(s.top, 0);
    assert_eq!(s.module_depth, 2);
    assert_eq!(s.module_roots, vec!["crates", "packages"]);
    assert!(matches!(s.children, ChildIncludeMode::Separate));
    assert!(s.redact.is_none());
}

// ── ExportSettings defaults ─────────────────────────────────────────

#[test]
fn export_settings_default_values() {
    let s = ExportSettings::default();
    assert!(matches!(s.format, ExportFormat::Jsonl));
    assert_eq!(s.min_code, 0);
    assert_eq!(s.max_rows, 0);
    assert!(matches!(s.redact, RedactMode::None));
    assert!(s.meta);
    assert!(s.strip_prefix.is_none());
}

// ── AnalyzeSettings defaults ────────────────────────────────────────

#[test]
fn analyze_settings_default_values() {
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

// ── CockpitSettings defaults ────────────────────────────────────────

#[test]
fn cockpit_settings_default_values() {
    let s = CockpitSettings::default();
    assert_eq!(s.base, "main");
    assert_eq!(s.head, "HEAD");
    assert_eq!(s.range_mode, "two-dot");
    assert!(s.baseline.is_none());
}

// ── DiffSettings defaults ───────────────────────────────────────────

#[test]
fn diff_settings_default_has_empty_refs() {
    let s = DiffSettings::default();
    assert!(s.from.is_empty());
    assert!(s.to.is_empty());
}

// ── Serde roundtrips (JSON) ─────────────────────────────────────────

#[test]
fn roundtrip_module_settings_json() {
    let s = ModuleSettings {
        top: 5,
        module_roots: vec!["src".into(), "lib".into()],
        module_depth: 4,
        children: ChildIncludeMode::ParentsOnly,
        redact: Some(RedactMode::Paths),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 5);
    assert_eq!(back.module_depth, 4);
    assert_eq!(back.module_roots, vec!["src", "lib"]);
}

#[test]
fn roundtrip_export_settings_all_fields() {
    let s = ExportSettings {
        format: ExportFormat::Csv,
        module_roots: vec!["packages".into()],
        module_depth: 3,
        children: ChildIncludeMode::ParentsOnly,
        min_code: 10,
        max_rows: 500,
        redact: RedactMode::Paths,
        meta: false,
        strip_prefix: Some("/repo/".into()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.min_code, 10);
    assert_eq!(back.max_rows, 500);
    assert!(!back.meta);
    assert_eq!(back.strip_prefix, Some("/repo/".into()));
}

#[test]
fn roundtrip_diff_settings_json() {
    let s = DiffSettings {
        from: "v1.0.0".into(),
        to: "v2.0.0".into(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "v1.0.0");
    assert_eq!(back.to, "v2.0.0");
}

// ── Clone / Debug ───────────────────────────────────────────────────

#[test]
fn clone_scan_options_is_independent() {
    let mut orig = ScanOptions::default();
    let cloned = orig.clone();
    orig.hidden = true;
    assert!(!cloned.hidden);
}

#[test]
fn debug_format_contains_field_names() {
    let s = LangSettings::default();
    let dbg = format!("{s:?}");
    assert!(dbg.contains("top"));
    assert!(dbg.contains("files"));
    assert!(dbg.contains("children"));
}

// ── TOML parsing ────────────────────────────────────────────────────

#[test]
fn toml_empty_string_produces_defaults() {
    let config = TomlConfig::parse("").unwrap();
    assert!(config.scan.hidden.is_none());
    assert!(config.module.depth.is_none());
    assert!(config.view.is_empty());
}

#[test]
fn toml_full_scan_section() {
    let toml_str = r#"
[scan]
hidden = true
no_ignore = true
no_ignore_parent = false
exclude = ["target", "node_modules"]
config = "none"
doc_comments = true
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.no_ignore, Some(true));
    assert_eq!(config.scan.no_ignore_parent, Some(false));
    assert_eq!(
        config.scan.exclude,
        Some(vec!["target".to_string(), "node_modules".to_string()])
    );
    assert_eq!(config.scan.doc_comments, Some(true));
}

#[test]
fn toml_multiple_view_profiles() {
    let toml_str = r#"
[view.llm]
format = "json"
top = 20
budget = "128k"

[view.ci]
format = "jsonl"
top = 0
compress = true
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.view.len(), 2);
    let llm = config.view.get("llm").unwrap();
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(20));
    assert_eq!(llm.budget.as_deref(), Some("128k"));
    let ci = config.view.get("ci").unwrap();
    assert_eq!(ci.compress, Some(true));
}

#[test]
fn toml_analyze_section() {
    let toml_str = r#"
[analyze]
preset = "deep"
window = 128000
git = true
max_files = 5000
max_bytes = 10000000
granularity = "file"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.analyze.preset.as_deref(), Some("deep"));
    assert_eq!(config.analyze.window, Some(128000));
    assert_eq!(config.analyze.git, Some(true));
    assert_eq!(config.analyze.max_files, Some(5000));
    assert_eq!(config.analyze.granularity.as_deref(), Some("file"));
}

#[test]
fn toml_gate_with_inline_rules() {
    let toml_str = r#"
[gate]
fail_fast = true

[[gate.rules]]
name = "min-coverage"
pointer = "/coverage"
op = ">="
value = 80.0
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.gate.fail_fast, Some(true));
    let rules = config.gate.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "min-coverage");
    assert_eq!(rules[0].pointer, "/coverage");
    assert_eq!(rules[0].op, ">=");
}

// ── Edge cases ──────────────────────────────────────────────────────

#[test]
fn view_profile_all_none_by_default() {
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
    assert!(vp.preset.is_none());
    assert!(vp.window.is_none());
    assert!(vp.budget.is_none());
    assert!(vp.strategy.is_none());
    assert!(vp.rank_by.is_none());
    assert!(vp.output.is_none());
    assert!(vp.compress.is_none());
    assert!(vp.metric.is_none());
    assert!(vp.children.is_none());
}

#[test]
fn toml_roundtrip_via_toml_crate() {
    let config = TomlConfig::parse(
        r#"
[scan]
hidden = true

[module]
depth = 3

[analyze]
preset = "health"
"#,
    )
    .unwrap();
    let serialized = toml::to_string(&config).unwrap();
    let back: TomlConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(back.scan.hidden, Some(true));
    assert_eq!(back.module.depth, Some(3));
    assert_eq!(back.analyze.preset.as_deref(), Some("health"));
}
