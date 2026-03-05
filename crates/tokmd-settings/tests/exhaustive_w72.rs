//! Exhaustive serde and invariant tests for tokmd-settings (w72).

use tokmd_settings::*;

// =============================================================================
// ScanOptions: every field has a reasonable default
// =============================================================================

#[test]
fn scan_options_defaults_all_false_or_empty() {
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
fn scan_options_serde_roundtrip() {
    let opts = ScanOptions {
        excluded: vec!["target".into(), "*.bak".into()],
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
    assert!(back.no_ignore_vcs);
}

// =============================================================================
// ScanSettings
// =============================================================================

#[test]
fn scan_settings_default_paths_empty() {
    let s = ScanSettings::default();
    assert!(s.paths.is_empty());
}

#[test]
fn scan_settings_current_dir() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
}

#[test]
fn scan_settings_for_paths() {
    let s = ScanSettings::for_paths(vec!["src".into(), "lib".into()]);
    assert_eq!(s.paths.len(), 2);
}

// =============================================================================
// LangSettings
// =============================================================================

#[test]
fn lang_settings_default_values() {
    let s = LangSettings::default();
    assert_eq!(s.top, 0);
    assert!(!s.files);
    assert_eq!(s.children, ChildrenMode::Collapse);
    assert!(s.redact.is_none());
}

#[test]
fn lang_settings_serde_roundtrip() {
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
    assert_eq!(back.children, ChildrenMode::Separate);
}

// =============================================================================
// ChildrenMode enum complete coverage
// =============================================================================

#[test]
fn children_mode_collapse_and_separate() {
    let variants = [ChildrenMode::Collapse, ChildrenMode::Separate];
    assert_eq!(variants.len(), 2);
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: ChildrenMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

// =============================================================================
// ModuleSettings
// =============================================================================

#[test]
fn module_settings_default_values() {
    let s = ModuleSettings::default();
    assert_eq!(s.top, 0);
    assert_eq!(s.module_roots, vec!["crates", "packages"]);
    assert_eq!(s.module_depth, 2);
    assert_eq!(s.children, ChildIncludeMode::Separate);
    assert!(s.redact.is_none());
}

// =============================================================================
// ExportSettings
// =============================================================================

#[test]
fn export_settings_default_values() {
    let s = ExportSettings::default();
    assert_eq!(s.format, ExportFormat::Jsonl);
    assert_eq!(s.module_roots, vec!["crates", "packages"]);
    assert_eq!(s.module_depth, 2);
    assert_eq!(s.children, ChildIncludeMode::Separate);
    assert_eq!(s.min_code, 0);
    assert_eq!(s.max_rows, 0);
    assert_eq!(s.redact, RedactMode::None);
    assert!(s.meta);
    assert!(s.strip_prefix.is_none());
}

// =============================================================================
// AnalyzeSettings
// =============================================================================

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

// =============================================================================
// CockpitSettings
// =============================================================================

#[test]
fn cockpit_settings_default_values() {
    let s = CockpitSettings::default();
    assert_eq!(s.base, "main");
    assert_eq!(s.head, "HEAD");
    assert_eq!(s.range_mode, "two-dot");
    assert!(s.baseline.is_none());
}

// =============================================================================
// DiffSettings
// =============================================================================

#[test]
fn diff_settings_default_empty_strings() {
    let s = DiffSettings::default();
    assert!(s.from.is_empty());
    assert!(s.to.is_empty());
}

// =============================================================================
// TomlConfig: roundtrip and view profiles
// =============================================================================

#[test]
fn toml_config_default_all_sections() {
    let cfg = TomlConfig::default();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.export.format.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.policy.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn toml_config_parse_with_view_profile() {
    let toml_str = r#"
[scan]
hidden = true

[view.ci]
format = "json"
top = 20
"#;
    let config = TomlConfig::parse(toml_str).expect("parse config");
    assert_eq!(config.scan.hidden, Some(true));
    let ci = config.view.get("ci").expect("ci profile");
    assert_eq!(ci.format.as_deref(), Some("json"));
    assert_eq!(ci.top, Some(20));
}
