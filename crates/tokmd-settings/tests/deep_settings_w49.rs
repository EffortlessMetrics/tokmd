//! Deep contract stability and enum coverage tests for `tokmd-settings`.
//!
//! Covers: ScanOptions construction, defaults for all settings types,
//! serde roundtrip, flatten behavior, TOML parsing, property tests,
//! and edge cases.

use proptest::prelude::*;
use serde_json::Value;
use tokmd_settings::{
    AnalyzeSettings, ChildIncludeMode, ChildrenMode, CockpitSettings, ConfigMode, DiffSettings,
    ExportFormat, ExportSettings, LangSettings, ModuleSettings, RedactMode, ScanOptions,
    ScanSettings, TomlConfig,
};

// =============================================================================
// 1. ScanOptions construction with all fields
// =============================================================================

#[test]
fn scan_options_all_fields_set() {
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
    assert_eq!(opts.excluded.len(), 2);
    assert!(opts.hidden);
    assert!(opts.no_ignore);
    assert!(opts.no_ignore_parent);
    assert!(opts.no_ignore_dot);
    assert!(opts.no_ignore_vcs);
    assert!(opts.treat_doc_strings_as_comments);
    assert_eq!(opts.config, ConfigMode::None);
}

#[test]
fn scan_options_defaults_are_all_false_or_empty() {
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

// =============================================================================
// 2. LangSettings defaults
// =============================================================================

#[test]
fn lang_settings_defaults() {
    let ls = LangSettings::default();
    assert_eq!(ls.top, 0);
    assert!(!ls.files);
    assert_eq!(ls.children, ChildrenMode::Collapse);
    assert!(ls.redact.is_none());
}

#[test]
fn lang_settings_custom_values_roundtrip() {
    let ls = LangSettings {
        top: 25,
        files: true,
        children: ChildrenMode::Separate,
        redact: Some(RedactMode::Paths),
    };
    let json = serde_json::to_string(&ls).unwrap();
    let back: LangSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 25);
    assert!(back.files);
    assert_eq!(back.children, ChildrenMode::Separate);
    assert_eq!(back.redact, Some(RedactMode::Paths));
}

// =============================================================================
// 3. ModuleSettings defaults
// =============================================================================

#[test]
fn module_settings_defaults() {
    let ms = ModuleSettings::default();
    assert_eq!(ms.top, 0);
    assert_eq!(ms.module_roots, vec!["crates", "packages"]);
    assert_eq!(ms.module_depth, 2);
    assert_eq!(ms.children, ChildIncludeMode::Separate);
    assert!(ms.redact.is_none());
}

#[test]
fn module_settings_custom_roundtrip() {
    let ms = ModuleSettings {
        top: 10,
        module_roots: vec!["src".into(), "lib".into()],
        module_depth: 3,
        children: ChildIncludeMode::ParentsOnly,
        redact: Some(RedactMode::All),
    };
    let json = serde_json::to_string(&ms).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 10);
    assert_eq!(back.module_roots, vec!["src", "lib"]);
    assert_eq!(back.module_depth, 3);
    assert_eq!(back.children, ChildIncludeMode::ParentsOnly);
    assert_eq!(back.redact, Some(RedactMode::All));
}

// =============================================================================
// 4. ExportSettings defaults
// =============================================================================

#[test]
fn export_settings_defaults() {
    let es = ExportSettings::default();
    assert_eq!(es.format, ExportFormat::Jsonl);
    assert_eq!(es.module_roots, vec!["crates", "packages"]);
    assert_eq!(es.module_depth, 2);
    assert_eq!(es.children, ChildIncludeMode::Separate);
    assert_eq!(es.min_code, 0);
    assert_eq!(es.max_rows, 0);
    assert_eq!(es.redact, RedactMode::None);
    assert!(es.meta);
    assert!(es.strip_prefix.is_none());
}

#[test]
fn export_settings_custom_roundtrip() {
    let es = ExportSettings {
        format: ExportFormat::Csv,
        module_roots: vec!["packages".into()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
        min_code: 50,
        max_rows: 500,
        redact: RedactMode::Paths,
        meta: false,
        strip_prefix: Some("/home/user/project".into()),
    };
    let json = serde_json::to_string(&es).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format, ExportFormat::Csv);
    assert_eq!(back.min_code, 50);
    assert_eq!(back.max_rows, 500);
    assert!(!back.meta);
    assert_eq!(back.strip_prefix, Some("/home/user/project".into()));
}

// =============================================================================
// 5. AnalyzeSettings defaults and preset
// =============================================================================

#[test]
fn analyze_settings_defaults() {
    let a = AnalyzeSettings::default();
    assert_eq!(a.preset, "receipt");
    assert_eq!(a.granularity, "module");
    assert!(a.window.is_none());
    assert!(a.git.is_none());
    assert!(a.max_files.is_none());
    assert!(a.max_bytes.is_none());
    assert!(a.max_file_bytes.is_none());
    assert!(a.max_commits.is_none());
    assert!(a.max_commit_files.is_none());
}

#[test]
fn analyze_settings_preset_override() {
    let a = AnalyzeSettings {
        preset: "deep".to_string(),
        window: Some(128000),
        git: Some(true),
        max_files: Some(1000),
        max_bytes: Some(50_000_000),
        max_file_bytes: Some(1_000_000),
        max_commits: Some(500),
        max_commit_files: Some(100),
        granularity: "file".to_string(),
    };
    let json = serde_json::to_string(&a).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "deep");
    assert_eq!(back.window, Some(128000));
    assert_eq!(back.git, Some(true));
    assert_eq!(back.max_files, Some(1000));
    assert_eq!(back.granularity, "file");
}

// =============================================================================
// 6. CockpitSettings defaults
// =============================================================================

#[test]
fn cockpit_settings_defaults() {
    let c = CockpitSettings::default();
    assert_eq!(c.base, "main");
    assert_eq!(c.head, "HEAD");
    assert_eq!(c.range_mode, "two-dot");
    assert!(c.baseline.is_none());
}

#[test]
fn cockpit_settings_custom_roundtrip() {
    let c = CockpitSettings {
        base: "develop".to_string(),
        head: "feature/xyz".to_string(),
        range_mode: "three-dot".to_string(),
        baseline: Some("baseline.json".to_string()),
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "develop");
    assert_eq!(back.head, "feature/xyz");
    assert_eq!(back.range_mode, "three-dot");
    assert_eq!(back.baseline.as_deref(), Some("baseline.json"));
}

// =============================================================================
// 7. DiffSettings defaults
// =============================================================================

#[test]
fn diff_settings_defaults() {
    let d = DiffSettings::default();
    assert_eq!(d.from, "");
    assert_eq!(d.to, "");
}

#[test]
fn diff_settings_roundtrip() {
    let d = DiffSettings {
        from: "v1.0.0".to_string(),
        to: "v2.0.0".to_string(),
    };
    let json = serde_json::to_string(&d).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "v1.0.0");
    assert_eq!(back.to, "v2.0.0");
}

// =============================================================================
// 8. ScanSettings flatten behavior
// =============================================================================

#[test]
fn scan_settings_flatten_in_json() {
    let s = ScanSettings {
        paths: vec!["src".into()],
        options: ScanOptions {
            hidden: true,
            no_ignore: true,
            ..Default::default()
        },
    };
    let v: Value = serde_json::to_value(&s).unwrap();
    // Options fields should be at top level due to #[serde(flatten)]
    assert!(v.get("options").is_none(), "options should be flattened");
    assert_eq!(v["hidden"], true);
    assert_eq!(v["no_ignore"], true);
    assert_eq!(v["paths"][0], "src");
}

#[test]
fn scan_settings_flatten_roundtrip() {
    let s = ScanSettings {
        paths: vec!["a".into(), "b".into()],
        options: ScanOptions {
            excluded: vec!["*.log".into()],
            config: ConfigMode::None,
            hidden: true,
            no_ignore: false,
            no_ignore_parent: true,
            no_ignore_dot: true,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: true,
        },
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths, vec!["a", "b"]);
    assert_eq!(back.options.excluded, vec!["*.log"]);
    assert!(back.options.hidden);
    assert!(back.options.no_ignore_parent);
    assert!(back.options.no_ignore_dot);
    assert!(back.options.treat_doc_strings_as_comments);
}

// =============================================================================
// 9. ScanSettings constructors
// =============================================================================

#[test]
fn scan_settings_current_dir() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
    assert!(!s.options.hidden);
    assert!(s.options.excluded.is_empty());
}

#[test]
fn scan_settings_for_paths() {
    let s = ScanSettings::for_paths(vec!["src".into(), "tests".into()]);
    assert_eq!(s.paths, vec!["src", "tests"]);
    assert!(!s.options.hidden);
}

// =============================================================================
// 10. Forward compat: extra JSON fields ignored
// =============================================================================

#[test]
fn forward_compat_extra_fields_scan_options() {
    let json = r#"{
        "excluded": [],
        "config": "auto",
        "hidden": false,
        "no_ignore": false,
        "no_ignore_parent": false,
        "no_ignore_dot": false,
        "no_ignore_vcs": false,
        "treat_doc_strings_as_comments": false,
        "future_field": "hello"
    }"#;
    let opts: ScanOptions = serde_json::from_str(json).unwrap();
    assert!(!opts.hidden);
}

#[test]
fn empty_json_object_gives_defaults() {
    let opts: ScanOptions = serde_json::from_str("{}").unwrap();
    assert!(!opts.hidden);
    assert!(opts.excluded.is_empty());

    let a: AnalyzeSettings = serde_json::from_str("{}").unwrap();
    assert_eq!(a.preset, "receipt");
}

// =============================================================================
// 11. TOML parsing
// =============================================================================

#[test]
fn toml_empty_string_parses() {
    let cfg = TomlConfig::parse("").unwrap();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn toml_scan_section() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
hidden = true
no_ignore = true
exclude = ["target", "*.bak"]
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(
        cfg.scan.paths.as_deref(),
        Some(&["src".to_string(), "lib".to_string()][..])
    );
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.no_ignore, Some(true));
    assert_eq!(cfg.scan.exclude.as_ref().unwrap().len(), 2);
}

#[test]
fn toml_analyze_section() {
    let toml_str = r#"
[analyze]
preset = "deep"
window = 128000
git = true
max_files = 1000
granularity = "file"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.analyze.preset.as_deref(), Some("deep"));
    assert_eq!(cfg.analyze.window, Some(128000));
    assert_eq!(cfg.analyze.git, Some(true));
    assert_eq!(cfg.analyze.max_files, Some(1000));
    assert_eq!(cfg.analyze.granularity.as_deref(), Some("file"));
}

#[test]
fn toml_view_profiles() {
    let toml_str = r#"
[view.llm]
format = "json"
top = 10
budget = "128k"

[view.ci]
format = "tsv"
preset = "risk"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.view.len(), 2);
    let llm = &cfg.view["llm"];
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(10));
    assert_eq!(llm.budget.as_deref(), Some("128k"));
}

#[test]
fn toml_gate_rules_inline() {
    let toml_str = r#"
[gate]
fail_fast = true

[[gate.rules]]
name = "min-code"
pointer = "/summary/code"
op = ">="
value = 100
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.gate.fail_fast, Some(true));
    let rules = cfg.gate.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "min-code");
    assert_eq!(rules[0].op, ">=");
}

// =============================================================================
// 12. Enum variant serde coverage
// =============================================================================

#[test]
fn config_mode_all_variants_roundtrip() {
    for mode in [ConfigMode::Auto, ConfigMode::None] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: ConfigMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, mode);
    }
}

#[test]
fn children_mode_all_variants_roundtrip() {
    for mode in [ChildrenMode::Collapse, ChildrenMode::Separate] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: ChildrenMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, mode);
    }
}

#[test]
fn child_include_mode_all_variants_roundtrip() {
    for mode in [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: ChildIncludeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, mode);
    }
}

#[test]
fn export_format_all_variants_roundtrip() {
    for fmt in [
        ExportFormat::Csv,
        ExportFormat::Jsonl,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ] {
        let json = serde_json::to_string(&fmt).unwrap();
        let back: ExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, fmt);
    }
}

#[test]
fn redact_mode_all_variants_roundtrip() {
    for mode in [RedactMode::None, RedactMode::Paths, RedactMode::All] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: RedactMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, mode);
    }
}

// =============================================================================
// 13. Edge cases: empty paths, zero limits
// =============================================================================

#[test]
fn scan_settings_empty_paths() {
    let s = ScanSettings::for_paths(vec![]);
    assert!(s.paths.is_empty());
    let json = serde_json::to_string(&s).unwrap();
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert!(back.paths.is_empty());
}

#[test]
fn export_settings_zero_limits() {
    let es = ExportSettings {
        min_code: 0,
        max_rows: 0,
        ..Default::default()
    };
    let json = serde_json::to_string(&es).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.min_code, 0);
    assert_eq!(back.max_rows, 0);
}

#[test]
fn analyze_settings_zero_window() {
    let a = AnalyzeSettings {
        window: Some(0),
        ..Default::default()
    };
    let json = serde_json::to_string(&a).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.window, Some(0));
}

#[test]
fn scan_options_empty_excluded_pattern() {
    let opts = ScanOptions {
        excluded: vec!["".into()],
        ..Default::default()
    };
    let json = serde_json::to_string(&opts).unwrap();
    let back: ScanOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded, vec![""]);
}

// =============================================================================
// 14. Clone independence
// =============================================================================

#[test]
fn scan_settings_clone_independent() {
    let original = ScanSettings {
        paths: vec!["src".into()],
        options: ScanOptions {
            hidden: true,
            ..Default::default()
        },
    };
    let mut cloned = original.clone();
    cloned.paths.push("lib".into());
    cloned.options.hidden = false;
    // Original should be unchanged
    assert_eq!(original.paths, vec!["src"]);
    assert!(original.options.hidden);
}

// =============================================================================
// Property tests
// =============================================================================

proptest! {
    #[test]
    fn proptest_scan_options_roundtrip(
        hidden in proptest::bool::ANY,
        no_ignore in proptest::bool::ANY,
        no_ignore_parent in proptest::bool::ANY,
        no_ignore_dot in proptest::bool::ANY,
        no_ignore_vcs in proptest::bool::ANY,
        treat_doc in proptest::bool::ANY,
    ) {
        let opts = ScanOptions {
            excluded: vec!["target".into()],
            config: ConfigMode::Auto,
            hidden,
            no_ignore,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            treat_doc_strings_as_comments: treat_doc,
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: ScanOptions = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.hidden, hidden);
        prop_assert_eq!(back.no_ignore, no_ignore);
        prop_assert_eq!(back.no_ignore_parent, no_ignore_parent);
        prop_assert_eq!(back.no_ignore_dot, no_ignore_dot);
        prop_assert_eq!(back.no_ignore_vcs, no_ignore_vcs);
        prop_assert_eq!(back.treat_doc_strings_as_comments, treat_doc);
    }

    #[test]
    fn proptest_scan_settings_with_paths(
        path_count in 0..10usize,
    ) {
        let paths: Vec<String> = (0..path_count).map(|i| format!("path_{i}")).collect();
        let s = ScanSettings::for_paths(paths.clone());
        let json = serde_json::to_string(&s).unwrap();
        let back: ScanSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.paths.len(), path_count);
        for (a, b) in back.paths.iter().zip(paths.iter()) {
            prop_assert_eq!(a, b);
        }
    }

    #[test]
    fn proptest_lang_settings_roundtrip(
        top in 0..1000usize,
        files in proptest::bool::ANY,
    ) {
        let ls = LangSettings {
            top,
            files,
            children: ChildrenMode::Collapse,
            redact: None,
        };
        let json = serde_json::to_string(&ls).unwrap();
        let back: LangSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.top, top);
        prop_assert_eq!(back.files, files);
    }

    #[test]
    fn proptest_export_settings_roundtrip(
        min_code in 0..10_000usize,
        max_rows in 0..100_000usize,
    ) {
        let es = ExportSettings {
            min_code,
            max_rows,
            ..Default::default()
        };
        let json = serde_json::to_string(&es).unwrap();
        let back: ExportSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.min_code, min_code);
        prop_assert_eq!(back.max_rows, max_rows);
    }
}
