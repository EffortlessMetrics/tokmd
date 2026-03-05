//! Deep tests for tokmd-settings (w75).
//!
//! Covers: every settings struct, default values, serde roundtrips,
//! TOML config structs, edge cases, boundary conditions, and
//! re-exported type coverage.

use std::io::Write;

use tempfile::NamedTempFile;
use tokmd_settings::*;

// ── 1. ScanOptions: defaults ────────────────────────────────────────

#[test]
fn scan_options_all_defaults_false_or_empty() {
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

// ── 2. ScanOptions: all flags true roundtrip ────────────────────────

#[test]
fn scan_options_all_flags_true_roundtrip() {
    let opts = ScanOptions {
        excluded: vec!["target".into(), "node_modules".into(), "*.bak".into()],
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
    assert_eq!(back.excluded.len(), 3);
    assert_eq!(back.config, ConfigMode::None);
    assert!(back.hidden);
    assert!(back.no_ignore);
    assert!(back.no_ignore_parent);
    assert!(back.no_ignore_dot);
    assert!(back.no_ignore_vcs);
    assert!(back.treat_doc_strings_as_comments);
}

// ── 3. ScanSettings: current_dir constructor ────────────────────────

#[test]
fn scan_settings_current_dir_has_dot_path() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
    assert!(s.options.excluded.is_empty());
    assert!(!s.options.hidden);
}

// ── 4. ScanSettings: for_paths constructor ──────────────────────────

#[test]
fn scan_settings_for_paths_multiple() {
    let s = ScanSettings::for_paths(vec!["src".into(), "lib".into(), "tests".into()]);
    assert_eq!(s.paths.len(), 3);
    assert_eq!(s.paths[2], "tests");
    // options should be default
    assert!(!s.options.hidden);
}

// ── 5. ScanSettings: empty paths ────────────────────────────────────

#[test]
fn scan_settings_for_empty_paths() {
    let s = ScanSettings::for_paths(vec![]);
    assert!(s.paths.is_empty());
}

// ── 6. ScanSettings: serde flatten merges options ───────────────────

#[test]
fn scan_settings_serde_flatten_roundtrip() {
    let s = ScanSettings {
        paths: vec!["src".into()],
        options: ScanOptions {
            excluded: vec!["vendor".into()],
            hidden: true,
            no_ignore_vcs: true,
            ..Default::default()
        },
    };
    let json = serde_json::to_string(&s).unwrap();
    // Flattened fields should appear at top level
    assert!(json.contains("\"hidden\":true"));
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths, vec!["src"]);
    assert!(back.options.hidden);
    assert!(back.options.no_ignore_vcs);
}

// ── 7. LangSettings: default values ────────────────────────────────

#[test]
fn lang_settings_defaults() {
    let s = LangSettings::default();
    assert_eq!(s.top, 0);
    assert!(!s.files);
    assert_eq!(s.children, ChildrenMode::Collapse);
    assert!(s.redact.is_none());
}

// ── 8. LangSettings: all fields populated roundtrip ─────────────────

#[test]
fn lang_settings_full_roundtrip() {
    let s = LangSettings {
        top: 15,
        files: true,
        children: ChildrenMode::Separate,
        redact: Some(RedactMode::All),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: LangSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 15);
    assert!(back.files);
    assert_eq!(back.children, ChildrenMode::Separate);
    assert_eq!(back.redact, Some(RedactMode::All));
}

// ── 9. ModuleSettings: default values ───────────────────────────────

#[test]
fn module_settings_defaults() {
    let s = ModuleSettings::default();
    assert_eq!(s.top, 0);
    assert_eq!(s.module_roots, vec!["crates", "packages"]);
    assert_eq!(s.module_depth, 2);
    assert_eq!(s.children, ChildIncludeMode::Separate);
    assert!(s.redact.is_none());
}

// ── 10. ModuleSettings: custom values roundtrip ─────────────────────

#[test]
fn module_settings_custom_roundtrip() {
    let s = ModuleSettings {
        top: 10,
        module_roots: vec!["libs".into(), "plugins".into()],
        module_depth: 5,
        children: ChildIncludeMode::ParentsOnly,
        redact: Some(RedactMode::Paths),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 10);
    assert_eq!(back.module_roots, vec!["libs", "plugins"]);
    assert_eq!(back.module_depth, 5);
    assert_eq!(back.children, ChildIncludeMode::ParentsOnly);
    assert_eq!(back.redact, Some(RedactMode::Paths));
}

// ── 11. ModuleSettings: depth boundary values ───────────────────────

#[test]
fn module_settings_depth_boundaries() {
    // depth = 0
    let s0 = ModuleSettings {
        module_depth: 0,
        ..Default::default()
    };
    let json = serde_json::to_string(&s0).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.module_depth, 0);

    // depth = large value
    let s_big = ModuleSettings {
        module_depth: usize::MAX,
        ..Default::default()
    };
    let json = serde_json::to_string(&s_big).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.module_depth, usize::MAX);
}

// ── 12. ExportSettings: default values ──────────────────────────────

#[test]
fn export_settings_defaults() {
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

// ── 13. ExportSettings: all formats roundtrip ───────────────────────

#[test]
fn export_settings_all_formats() {
    for fmt in [
        ExportFormat::Jsonl,
        ExportFormat::Csv,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ] {
        let s = ExportSettings {
            format: fmt,
            ..Default::default()
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ExportSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.format, fmt);
    }
}

// ── 14. ExportSettings: full configuration roundtrip ────────────────

#[test]
fn export_settings_full_roundtrip() {
    let s = ExportSettings {
        format: ExportFormat::Csv,
        module_roots: vec!["src".into()],
        module_depth: 4,
        children: ChildIncludeMode::ParentsOnly,
        min_code: 50,
        max_rows: 1000,
        redact: RedactMode::All,
        meta: false,
        strip_prefix: Some("/home/user/project".into()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format, ExportFormat::Csv);
    assert_eq!(back.module_depth, 4);
    assert_eq!(back.min_code, 50);
    assert_eq!(back.max_rows, 1000);
    assert_eq!(back.redact, RedactMode::All);
    assert!(!back.meta);
    assert_eq!(back.strip_prefix.as_deref(), Some("/home/user/project"));
}

// ── 15. AnalyzeSettings: default values ─────────────────────────────

#[test]
fn analyze_settings_defaults() {
    let s = AnalyzeSettings::default();
    assert_eq!(s.preset, "receipt");
    assert!(s.window.is_none());
    assert!(s.git.is_none());
    assert!(s.max_files.is_none());
    assert!(s.max_bytes.is_none());
    assert!(s.max_file_bytes.is_none());
    assert!(s.max_commits.is_none());
    assert!(s.max_commit_files.is_none());
    assert_eq!(s.granularity, "module");
}

// ── 16. AnalyzeSettings: all limits populated ───────────────────────

#[test]
fn analyze_settings_full_limits_roundtrip() {
    let s = AnalyzeSettings {
        preset: "deep".into(),
        window: Some(256_000),
        git: Some(true),
        max_files: Some(10_000),
        max_bytes: Some(100_000_000),
        max_file_bytes: Some(1_000_000),
        max_commits: Some(500),
        max_commit_files: Some(200),
        granularity: "file".into(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "deep");
    assert_eq!(back.window, Some(256_000));
    assert_eq!(back.git, Some(true));
    assert_eq!(back.max_files, Some(10_000));
    assert_eq!(back.max_bytes, Some(100_000_000));
    assert_eq!(back.max_file_bytes, Some(1_000_000));
    assert_eq!(back.max_commits, Some(500));
    assert_eq!(back.max_commit_files, Some(200));
    assert_eq!(back.granularity, "file");
}

// ── 17. CockpitSettings: default values ─────────────────────────────

#[test]
fn cockpit_settings_defaults() {
    let s = CockpitSettings::default();
    assert_eq!(s.base, "main");
    assert_eq!(s.head, "HEAD");
    assert_eq!(s.range_mode, "two-dot");
    assert!(s.baseline.is_none());
}

// ── 18. CockpitSettings: custom values roundtrip ────────────────────

#[test]
fn cockpit_settings_custom_roundtrip() {
    let s = CockpitSettings {
        base: "v2.0.0".into(),
        head: "feature-branch".into(),
        range_mode: "three-dot".into(),
        baseline: Some("path/to/baseline.json".into()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "v2.0.0");
    assert_eq!(back.head, "feature-branch");
    assert_eq!(back.range_mode, "three-dot");
    assert_eq!(
        back.baseline.as_deref(),
        Some("path/to/baseline.json")
    );
}

// ── 19. DiffSettings: default is empty strings ──────────────────────

#[test]
fn diff_settings_default_empty() {
    let s = DiffSettings::default();
    assert!(s.from.is_empty());
    assert!(s.to.is_empty());
}

// ── 20. DiffSettings: roundtrip ─────────────────────────────────────

#[test]
fn diff_settings_roundtrip() {
    let s = DiffSettings {
        from: "v1.0.0".into(),
        to: "v2.0.0".into(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "v1.0.0");
    assert_eq!(back.to, "v2.0.0");
}

// ── 21. TomlConfig: parse full TOML with all sections ───────────────

#[test]
fn toml_config_full_parse() {
    let toml_str = r#"
[scan]
paths = ["src"]
exclude = ["dist"]
hidden = false
config = "auto"
no_ignore = false
no_ignore_parent = false
no_ignore_dot = false
no_ignore_vcs = false
doc_comments = false

[module]
roots = ["packages"]
depth = 1
children = "separate"

[export]
min_code = 5
max_rows = 100
redact = "all"
format = "json"
children = "collapse"

[analyze]
preset = "health"
window = 64000
format = "md"
git = false
max_files = 1000
max_bytes = 50000000
max_file_bytes = 500000
max_commits = 300
max_commit_files = 80
granularity = "module"

[context]
budget = "64k"
strategy = "greedy"
rank_by = "code"
output = "list"
compress = false

[badge]
metric = "doc"

[gate]
policy = "gate.toml"
baseline = "old.json"
preset = "receipt"
fail_fast = false
allow_missing_baseline = false
allow_missing_current = true
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.scan.paths, Some(vec!["src".to_string()]));
    assert_eq!(config.scan.hidden, Some(false));
    assert_eq!(config.module.roots, Some(vec!["packages".to_string()]));
    assert_eq!(config.module.depth, Some(1));
    assert_eq!(config.export.min_code, Some(5));
    assert_eq!(config.export.format, Some("json".to_string()));
    assert_eq!(config.analyze.preset, Some("health".to_string()));
    assert_eq!(config.analyze.git, Some(false));
    assert_eq!(config.analyze.max_bytes, Some(50_000_000));
    assert_eq!(config.context.budget, Some("64k".to_string()));
    assert_eq!(config.badge.metric, Some("doc".to_string()));
    assert_eq!(config.gate.fail_fast, Some(false));
    assert_eq!(config.gate.allow_missing_current, Some(true));
}

// ── 22. TomlConfig: from_file ───────────────────────────────────────

#[test]
fn toml_config_from_file_roundtrip() {
    let content = r#"
[context]
budget = "1m"
strategy = "spread"
compress = true
"#;
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    let config = TomlConfig::from_file(f.path()).unwrap();
    assert_eq!(config.context.budget, Some("1m".to_string()));
    assert_eq!(config.context.strategy, Some("spread".to_string()));
    assert_eq!(config.context.compress, Some(true));
}

// ── 23. TomlConfig: view profiles with BTreeMap ordering ────────────

#[test]
fn toml_view_profiles_btreemap_ordering() {
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
    // BTreeMap gives sorted order
    assert_eq!(keys, vec!["alpha", "middle", "zebra"]);
}

// ── 24. GateRule: serde roundtrip ───────────────────────────────────

#[test]
fn gate_rule_full_serde_roundtrip() {
    let rule = GateRule {
        name: "density-check".into(),
        pointer: "/derived/comment_density".into(),
        op: ">=".into(),
        value: Some(serde_json::json!(0.15)),
        values: None,
        negate: false,
        level: Some("error".into()),
        message: Some("Comment density too low".into()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "density-check");
    assert_eq!(back.pointer, "/derived/comment_density");
    assert_eq!(back.op, ">=");
    assert!(!back.negate);
    assert_eq!(back.level.as_deref(), Some("error"));
    assert_eq!(back.message.as_deref(), Some("Comment density too low"));
}

// ── 25. RatchetRuleConfig: serde roundtrip ──────────────────────────

#[test]
fn ratchet_rule_config_serde_roundtrip() {
    let rule = RatchetRuleConfig {
        pointer: "/complexity/max_cyclomatic".into(),
        max_increase_pct: Some(10.0),
        max_value: Some(50.0),
        level: Some("warn".into()),
        description: Some("Max cyclomatic complexity guard".into()),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: RatchetRuleConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.pointer, "/complexity/max_cyclomatic");
    assert_eq!(back.max_increase_pct, Some(10.0));
    assert_eq!(back.max_value, Some(50.0));
    assert_eq!(back.level.as_deref(), Some("warn"));
    assert!(back.description.is_some());
}

// ── 26. Re-exported types: ConfigMode variants ──────────────────────

#[test]
fn config_mode_variants() {
    let auto = ConfigMode::Auto;
    let none = ConfigMode::None;
    assert_ne!(auto, none);
    // Default is Auto
    assert_eq!(ConfigMode::default(), ConfigMode::Auto);
}

// ── 27. Re-exported types: RedactMode variants ──────────────────────

#[test]
fn redact_mode_all_variants() {
    let variants = [RedactMode::None, RedactMode::Paths, RedactMode::All];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: RedactMode = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

// ── 28. Re-exported types: ExportFormat variants ────────────────────

#[test]
fn export_format_all_variants() {
    let variants = [
        ExportFormat::Jsonl,
        ExportFormat::Csv,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: ExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, v);
    }
}

// ── 29. TomlConfig: empty parse gives all None ──────────────────────

#[test]
fn toml_config_empty_defaults() {
    let config = TomlConfig::parse("").unwrap();
    assert!(config.scan.paths.is_none());
    assert!(config.scan.exclude.is_none());
    assert!(config.scan.hidden.is_none());
    assert!(config.module.roots.is_none());
    assert!(config.module.depth.is_none());
    assert!(config.export.min_code.is_none());
    assert!(config.export.max_rows.is_none());
    assert!(config.analyze.preset.is_none());
    assert!(config.analyze.window.is_none());
    assert!(config.context.budget.is_none());
    assert!(config.badge.metric.is_none());
    assert!(config.gate.policy.is_none());
    assert!(config.gate.rules.is_none());
    assert!(config.gate.ratchet.is_none());
    assert!(config.view.is_empty());
}

// ── 30. TomlConfig: malformed TOML error ────────────────────────────

#[test]
fn toml_config_malformed_error() {
    assert!(TomlConfig::parse("{{{not toml}}}").is_err());
}

// ── 31. AnalyzeSettings: git disabled explicitly ────────────────────

#[test]
fn analyze_settings_git_disabled() {
    let s = AnalyzeSettings {
        git: Some(false),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.git, Some(false));
}

// ── 32. ExportSettings: max_rows zero means unlimited ───────────────

#[test]
fn export_settings_max_rows_zero_is_unlimited() {
    let s = ExportSettings {
        max_rows: 0,
        ..Default::default()
    };
    assert_eq!(s.max_rows, 0);
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.max_rows, 0);
}
