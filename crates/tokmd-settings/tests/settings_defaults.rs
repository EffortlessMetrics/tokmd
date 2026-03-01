//! Tests that all Default impls produce reasonable, documented values
//! and that edge-case inputs are handled correctly.

use tokmd_settings::*;

// =============================================================================
// Default impl verification
// =============================================================================

#[test]
fn scan_options_default_all_booleans_false() {
    let d = ScanOptions::default();
    assert!(d.excluded.is_empty());
    assert!(!d.hidden);
    assert!(!d.no_ignore);
    assert!(!d.no_ignore_parent);
    assert!(!d.no_ignore_dot);
    assert!(!d.no_ignore_vcs);
    assert!(!d.treat_doc_strings_as_comments);
}

#[test]
fn scan_settings_default_paths_empty() {
    let d = ScanSettings::default();
    assert!(d.paths.is_empty());
}

#[test]
fn scan_settings_current_dir_single_dot() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
    // Inner options should still be defaults
    assert!(!s.options.hidden);
}

#[test]
fn lang_settings_default_values() {
    let d = LangSettings::default();
    assert_eq!(d.top, 0, "top=0 means show all rows");
    assert!(!d.files);
    assert!(d.redact.is_none());
}

#[test]
fn module_settings_default_roots() {
    let d = ModuleSettings::default();
    assert_eq!(d.module_roots, vec!["crates", "packages"]);
    assert_eq!(d.module_depth, 2);
    assert_eq!(d.top, 0);
    assert!(d.redact.is_none());
}

#[test]
fn export_settings_default_format_jsonl() {
    let d = ExportSettings::default();
    assert!(matches!(d.format, ExportFormat::Jsonl));
    assert_eq!(d.min_code, 0);
    assert_eq!(d.max_rows, 0);
    assert!(matches!(d.redact, RedactMode::None));
    assert!(d.meta, "meta should be true by default");
    assert!(d.strip_prefix.is_none());
    assert_eq!(d.module_depth, 2);
}

#[test]
fn analyze_settings_default_preset_receipt() {
    let d = AnalyzeSettings::default();
    assert_eq!(d.preset, "receipt");
    assert_eq!(d.granularity, "module");
    assert!(d.window.is_none());
    assert!(d.git.is_none());
    assert!(d.max_files.is_none());
    assert!(d.max_bytes.is_none());
    assert!(d.max_file_bytes.is_none());
    assert!(d.max_commits.is_none());
    assert!(d.max_commit_files.is_none());
}

#[test]
fn cockpit_settings_default_refs() {
    let d = CockpitSettings::default();
    assert_eq!(d.base, "main");
    assert_eq!(d.head, "HEAD");
    assert_eq!(d.range_mode, "two-dot");
    assert!(d.baseline.is_none());
}

#[test]
fn diff_settings_default_empty_strings() {
    let d = DiffSettings::default();
    assert_eq!(d.from, "");
    assert_eq!(d.to, "");
}

#[test]
fn toml_config_default_all_none() {
    let d = TomlConfig::default();
    assert!(d.scan.paths.is_none());
    assert!(d.scan.exclude.is_none());
    assert!(d.scan.hidden.is_none());
    assert!(d.module.roots.is_none());
    assert!(d.module.depth.is_none());
    assert!(d.export.min_code.is_none());
    assert!(d.analyze.preset.is_none());
    assert!(d.context.budget.is_none());
    assert!(d.badge.metric.is_none());
    assert!(d.gate.policy.is_none());
    assert!(d.gate.rules.is_none());
    assert!(d.gate.ratchet.is_none());
    assert!(d.view.is_empty());
}

#[test]
fn view_profile_default_all_none() {
    let d = ViewProfile::default();
    assert!(d.format.is_none());
    assert!(d.top.is_none());
    assert!(d.files.is_none());
    assert!(d.module_roots.is_none());
    assert!(d.module_depth.is_none());
    assert!(d.min_code.is_none());
    assert!(d.max_rows.is_none());
    assert!(d.redact.is_none());
    assert!(d.meta.is_none());
    assert!(d.children.is_none());
    assert!(d.preset.is_none());
    assert!(d.window.is_none());
    assert!(d.budget.is_none());
    assert!(d.strategy.is_none());
    assert!(d.rank_by.is_none());
    assert!(d.output.is_none());
    assert!(d.compress.is_none());
    assert!(d.metric.is_none());
}

#[test]
fn scan_config_default_all_none() {
    let d = ScanConfig::default();
    assert!(d.paths.is_none());
    assert!(d.exclude.is_none());
    assert!(d.hidden.is_none());
    assert!(d.config.is_none());
    assert!(d.no_ignore.is_none());
    assert!(d.no_ignore_parent.is_none());
    assert!(d.no_ignore_dot.is_none());
    assert!(d.no_ignore_vcs.is_none());
    assert!(d.doc_comments.is_none());
}

#[test]
fn gate_config_default_all_none() {
    let d = GateConfig::default();
    assert!(d.policy.is_none());
    assert!(d.baseline.is_none());
    assert!(d.preset.is_none());
    assert!(d.fail_fast.is_none());
    assert!(d.rules.is_none());
    assert!(d.ratchet.is_none());
    assert!(d.allow_missing_baseline.is_none());
    assert!(d.allow_missing_current.is_none());
}

// =============================================================================
// Edge cases: empty strings
// =============================================================================

#[test]
fn scan_settings_for_paths_empty_string() {
    let s = ScanSettings::for_paths(vec!["".into()]);
    assert_eq!(s.paths, vec![""]);
}

#[test]
fn diff_settings_with_whitespace_refs() {
    let s = DiffSettings {
        from: "   ".into(),
        to: "\t\n".into(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "   ");
    assert_eq!(back.to, "\t\n");
}

// =============================================================================
// Edge cases: very long values
// =============================================================================

#[test]
fn export_settings_very_long_strip_prefix() {
    let long_prefix = "a/".repeat(1000);
    let s = ExportSettings {
        strip_prefix: Some(long_prefix.clone()),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.strip_prefix.unwrap(), long_prefix);
}

#[test]
fn module_settings_many_roots() {
    let roots: Vec<String> = (0..100).map(|i| format!("root_{i}")).collect();
    let s = ModuleSettings {
        module_roots: roots.clone(),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.module_roots.len(), 100);
    assert_eq!(back.module_roots, roots);
}

// =============================================================================
// Edge cases: special characters in paths
// =============================================================================

#[test]
fn scan_options_excluded_with_special_chars() {
    let patterns = vec![
        "path with spaces/file.rs".into(),
        "données/résumé.txt".into(),
        "中文/路径.rs".into(),
        "a\"b".into(),
        "new\nline".into(),
    ];
    let s = ScanOptions {
        excluded: patterns.clone(),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ScanOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded, patterns);
}

#[test]
fn cockpit_settings_special_ref_names() {
    let s = CockpitSettings {
        base: "refs/tags/v1.0.0-beta.1+build.123".into(),
        head: "feature/JIRA-1234/implement-widget".into(),
        range_mode: "two-dot".into(),
        baseline: Some("path/with spaces/baseline.json".into()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "refs/tags/v1.0.0-beta.1+build.123");
    assert_eq!(back.head, "feature/JIRA-1234/implement-widget");
    assert_eq!(
        back.baseline.as_deref(),
        Some("path/with spaces/baseline.json")
    );
}

// =============================================================================
// Edge cases: numeric boundaries
// =============================================================================

#[test]
fn analyze_settings_large_limits() {
    let s = AnalyzeSettings {
        max_bytes: Some(u64::MAX),
        max_file_bytes: Some(u64::MAX),
        max_files: Some(usize::MAX),
        max_commits: Some(usize::MAX),
        max_commit_files: Some(usize::MAX),
        window: Some(usize::MAX),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.max_bytes, Some(u64::MAX));
    assert_eq!(back.max_files, Some(usize::MAX));
    assert_eq!(back.window, Some(usize::MAX));
}

#[test]
fn export_settings_zero_boundaries() {
    let s = ExportSettings {
        min_code: 0,
        max_rows: 0,
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.min_code, 0);
    assert_eq!(back.max_rows, 0);
}
