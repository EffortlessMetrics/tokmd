//! Deep contract tests for `tokmd-settings`.
//!
//! Covers: default-value invariants, serde roundtrip edge cases, forward
//! compatibility, flatten behaviour, TOML parsing, gate rules, ratchet
//! rules, view profiles, enum variant serialization, error rejection,
//! and clone independence.

use tokmd_settings::{
    AnalyzeSettings, ChildIncludeMode, ChildrenMode, CockpitSettings, ConfigMode, DiffSettings,
    ExportFormat, ExportSettings, GateConfig, GateRule, LangSettings, ModuleSettings,
    RatchetRuleConfig, RedactMode, ScanOptions, ScanSettings, TomlConfig, ViewProfile,
};

// =============================================================================
// 1. ScanOptions – every default is false/empty
// =============================================================================

#[test]
fn scan_options_all_defaults() {
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

// =============================================================================
// 2. LangSettings defaults
// =============================================================================

#[test]
fn lang_settings_defaults() {
    let ls = LangSettings::default();
    assert_eq!(ls.top, 0);
    assert!(!ls.files);
    assert!(matches!(ls.children, ChildrenMode::Collapse));
    assert!(ls.redact.is_none());
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
    assert!(matches!(ms.children, ChildIncludeMode::Separate));
    assert!(ms.redact.is_none());
}

// =============================================================================
// 4. ExportSettings defaults
// =============================================================================

#[test]
fn export_settings_defaults() {
    let es = ExportSettings::default();
    assert!(matches!(es.format, ExportFormat::Jsonl));
    assert_eq!(es.module_roots, vec!["crates", "packages"]);
    assert_eq!(es.module_depth, 2);
    assert!(matches!(es.children, ChildIncludeMode::Separate));
    assert_eq!(es.min_code, 0);
    assert_eq!(es.max_rows, 0);
    assert!(matches!(es.redact, RedactMode::None));
    assert!(es.meta);
    assert!(es.strip_prefix.is_none());
}

// =============================================================================
// 5. AnalyzeSettings defaults
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

// =============================================================================
// 7. DiffSettings default is empty strings
// =============================================================================

#[test]
fn diff_settings_defaults() {
    let d = DiffSettings::default();
    assert_eq!(d.from, "");
    assert_eq!(d.to, "");
}

// =============================================================================
// 8. ScanSettings::current_dir preserves default options
// =============================================================================

#[test]
fn scan_settings_current_dir_has_default_options() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
    assert!(!s.options.hidden);
    assert!(s.options.excluded.is_empty());
}

// =============================================================================
// 9. ScanSettings JSON flatten behaviour
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
    let value = serde_json::to_value(s).unwrap();

    // Flattened: options fields appear at top level, not nested under "options"
    assert!(value.get("options").is_none());
    assert_eq!(value["hidden"], true);
    assert_eq!(value["no_ignore"], true);
    assert_eq!(value["paths"][0], "src");
}

// =============================================================================
// 10. ScanSettings flatten roundtrip
// =============================================================================

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
    assert_eq!(back.paths, s.paths);
    assert_eq!(back.options.excluded, s.options.excluded);
    assert!(back.options.hidden);
    assert!(back.options.no_ignore_parent);
    assert!(back.options.no_ignore_dot);
    assert!(back.options.treat_doc_strings_as_comments);
}

// =============================================================================
// 11. ConfigMode enum roundtrip all variants
// =============================================================================

#[test]
fn config_mode_all_variants_roundtrip() {
    for mode in [ConfigMode::Auto, ConfigMode::None] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: ConfigMode = serde_json::from_str(&json).unwrap();
        assert_eq!(
            serde_json::to_string(&back).unwrap(),
            json,
            "ConfigMode roundtrip failed"
        );
    }
}

// =============================================================================
// 12. ChildrenMode enum roundtrip all variants
// =============================================================================

#[test]
fn children_mode_all_variants_roundtrip() {
    for mode in [ChildrenMode::Collapse, ChildrenMode::Separate] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: ChildrenMode = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 13. ChildIncludeMode enum roundtrip
// =============================================================================

#[test]
fn child_include_mode_all_variants_roundtrip() {
    for mode in [ChildIncludeMode::ParentsOnly, ChildIncludeMode::Separate] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: ChildIncludeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 14. ExportFormat enum roundtrip all variants
// =============================================================================

#[test]
fn export_format_all_variants_roundtrip() {
    for fmt in [
        ExportFormat::Jsonl,
        ExportFormat::Csv,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ] {
        let json = serde_json::to_string(&fmt).unwrap();
        let back: ExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 15. RedactMode enum roundtrip all variants
// =============================================================================

#[test]
fn redact_mode_all_variants_roundtrip() {
    for mode in [RedactMode::None, RedactMode::Paths, RedactMode::All] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: RedactMode = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 16. Forward compat: extra JSON fields ignored on deserialize
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
        "future_field": "hello",
        "another_future": 42
    }"#;
    let opts: ScanOptions = serde_json::from_str(json).unwrap();
    assert!(!opts.hidden);
}

// =============================================================================
// 17. Deserialize from empty JSON object gets defaults
// =============================================================================

#[test]
fn empty_json_object_gives_defaults_scan_options() {
    let opts: ScanOptions = serde_json::from_str("{}").unwrap();
    assert!(!opts.hidden);
    assert!(opts.excluded.is_empty());
}

#[test]
fn empty_json_object_gives_defaults_analyze_settings() {
    let a: AnalyzeSettings = serde_json::from_str("{}").unwrap();
    assert_eq!(a.preset, "receipt");
    assert_eq!(a.granularity, "module");
}

// =============================================================================
// 18. TOML empty string parses to default TomlConfig
// =============================================================================

#[test]
fn toml_empty_string_parses() {
    let cfg = TomlConfig::parse("").unwrap();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.view.is_empty());
    assert!(cfg.gate.rules.is_none());
}

// =============================================================================
// 19. TOML with all sections empty
// =============================================================================

#[test]
fn toml_all_empty_sections() {
    let toml_str = r#"
[scan]
[module]
[export]
[analyze]
[context]
[badge]
[gate]
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.export.min_code.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.policy.is_none());
}

// =============================================================================
// 20. TOML with view profiles
// =============================================================================

#[test]
fn toml_view_profiles() {
    let toml_str = r#"
[view.llm]
format = "json"
top = 10
budget = "128k"

[view.ci]
format = "tsv"
top = 0
preset = "risk"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.view.len(), 2);

    let llm = &cfg.view["llm"];
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(10));
    assert_eq!(llm.budget.as_deref(), Some("128k"));

    let ci = &cfg.view["ci"];
    assert_eq!(ci.format.as_deref(), Some("tsv"));
    assert_eq!(ci.preset.as_deref(), Some("risk"));
}

// =============================================================================
// 21. TOML gate rules inline
// =============================================================================

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

[[gate.rules]]
name = "no-todo"
pointer = "/health/todo_density"
op = "<="
value = 0.05
negate = false
level = "warn"
message = "Too many TODOs"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.gate.fail_fast, Some(true));
    let rules = cfg.gate.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "min-code");
    assert_eq!(rules[0].op, ">=");
    assert_eq!(rules[1].level.as_deref(), Some("warn"));
    assert_eq!(rules[1].message.as_deref(), Some("Too many TODOs"));
}

// =============================================================================
// 22. TOML ratchet rules
// =============================================================================

#[test]
fn toml_ratchet_rules() {
    let toml_str = r#"
[gate]

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
max_value = 25.0
level = "error"
description = "Cyclomatic complexity ceiling"

[[gate.ratchet]]
pointer = "/health/todo_density"
max_increase_pct = 5.0
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let ratchet = cfg.gate.ratchet.as_ref().unwrap();
    assert_eq!(ratchet.len(), 2);
    assert_eq!(ratchet[0].pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchet[0].max_increase_pct, Some(10.0));
    assert_eq!(ratchet[0].max_value, Some(25.0));
    assert_eq!(
        ratchet[0].description.as_deref(),
        Some("Cyclomatic complexity ceiling")
    );
    assert!(ratchet[1].level.is_none());
    assert!(ratchet[1].max_value.is_none());
}

// =============================================================================
// 23. GateRule serde with JSON value types
// =============================================================================

#[test]
fn gate_rule_serde_with_various_value_types() {
    // value can be number, string, bool
    let rule_num = GateRule {
        name: "r1".into(),
        pointer: "/a".into(),
        op: ">=".into(),
        value: Some(serde_json::json!(42)),
        values: None,
        negate: false,
        level: None,
        message: None,
    };
    let json = serde_json::to_string(&rule_num).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert_eq!(back.value.unwrap(), 42);

    let rule_str = GateRule {
        name: "r2".into(),
        pointer: "/b".into(),
        op: "==".into(),
        value: Some(serde_json::json!("hello")),
        values: None,
        negate: true,
        level: Some("warn".into()),
        message: Some("custom msg".into()),
    };
    let json = serde_json::to_string(&rule_str).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert!(back.negate);
    assert_eq!(back.value.unwrap(), "hello");
}

// =============================================================================
// 24. GateRule with values list (for "in" operator)
// =============================================================================

#[test]
fn gate_rule_with_values_list() {
    let rule = GateRule {
        name: "allowed-langs".into(),
        pointer: "/lang".into(),
        op: "in".into(),
        value: None,
        values: Some(vec![
            serde_json::json!("Rust"),
            serde_json::json!("Python"),
            serde_json::json!("Go"),
        ]),
        negate: false,
        level: None,
        message: None,
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: GateRule = serde_json::from_str(&json).unwrap();
    assert!(back.value.is_none());
    let vals = back.values.unwrap();
    assert_eq!(vals.len(), 3);
    assert_eq!(vals[0], "Rust");
}

// =============================================================================
// 25. Clone mutation independence for all settings types
// =============================================================================

#[test]
fn clone_independence_scan_settings() {
    let orig = ScanSettings {
        paths: vec!["src".into()],
        options: ScanOptions {
            hidden: true,
            ..Default::default()
        },
    };
    let mut clone = orig.clone();
    clone.paths.push("lib".into());
    clone.options.hidden = false;
    assert_eq!(orig.paths.len(), 1);
    assert!(orig.options.hidden);
}

#[test]
fn clone_independence_export_settings() {
    let orig = ExportSettings::default();
    let mut clone = orig.clone();
    clone.min_code = 999;
    clone.module_roots.push("new_root".into());
    assert_eq!(orig.min_code, 0);
    assert_eq!(orig.module_roots.len(), 2);
}

// =============================================================================
// 26. ViewProfile with all fields None
// =============================================================================

#[test]
fn view_profile_all_none() {
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

// =============================================================================
// 27. ViewProfile with all fields populated
// =============================================================================

#[test]
fn view_profile_all_populated_roundtrip() {
    let vp = ViewProfile {
        format: Some("json".into()),
        top: Some(10),
        files: Some(true),
        module_roots: Some(vec!["crates".into()]),
        module_depth: Some(3),
        min_code: Some(5),
        max_rows: Some(100),
        redact: Some("paths".into()),
        meta: Some(false),
        children: Some("collapse".into()),
        preset: Some("deep".into()),
        window: Some(128_000),
        budget: Some("128k".into()),
        strategy: Some("greedy".into()),
        rank_by: Some("churn".into()),
        output: Some("bundle".into()),
        compress: Some(true),
        metric: Some("code".into()),
    };
    let json = serde_json::to_string(&vp).unwrap();
    let back: ViewProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format.as_deref(), Some("json"));
    assert_eq!(back.top, Some(10));
    assert_eq!(back.window, Some(128_000));
    assert_eq!(back.compress, Some(true));
}

// =============================================================================
// 28. TomlConfig JSON roundtrip stability
// =============================================================================

#[test]
fn toml_config_json_roundtrip_stability() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target"]
hidden = true

[module]
roots = ["crates"]
depth = 3

[export]
min_code = 10
format = "csv"

[analyze]
preset = "deep"
window = 128000

[context]
budget = "64k"
strategy = "spread"

[badge]
metric = "tokens"

[gate]
fail_fast = true
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let json = serde_json::to_string(&cfg).unwrap();
    let back: TomlConfig = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json, json2);
}

// =============================================================================
// 29. DiffSettings with special characters
// =============================================================================

#[test]
fn diff_settings_special_chars() {
    let d = DiffSettings {
        from: "refs/tags/v1.0.0-beta.1".into(),
        to: "refs/heads/feature/日本語".into(),
    };
    let json = serde_json::to_string(&d).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "refs/tags/v1.0.0-beta.1");
    assert!(back.to.contains("日本語"));
}

// =============================================================================
// 30. AnalyzeSettings with all optional fields set
// =============================================================================

#[test]
fn analyze_settings_all_optional_fields() {
    let a = AnalyzeSettings {
        preset: "deep".into(),
        window: Some(128_000),
        git: Some(true),
        max_files: Some(5000),
        max_bytes: Some(50_000_000),
        max_file_bytes: Some(1_000_000),
        max_commits: Some(1000),
        max_commit_files: Some(100),
        granularity: "file".into(),
    };
    let json = serde_json::to_string(&a).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.max_bytes, Some(50_000_000));
    assert_eq!(back.max_file_bytes, Some(1_000_000));
    assert_eq!(back.granularity, "file");
}

// =============================================================================
// 31. TOML parse error on wrong type
// =============================================================================

#[test]
fn toml_parse_error_wrong_type() {
    // hidden should be bool, not string
    let toml_str = r#"
[scan]
hidden = "yes"
"#;
    assert!(TomlConfig::parse(toml_str).is_err());
}

// =============================================================================
// 32. TomlConfig from_file with non-existent path
// =============================================================================

#[test]
fn toml_from_file_nonexistent() {
    let result = TomlConfig::from_file(std::path::Path::new("/nonexistent/path/tokmd.toml"));
    assert!(result.is_err());
}

// =============================================================================
// 33. TOML with scan exclude patterns
// =============================================================================

#[test]
fn toml_scan_exclude_patterns() {
    let toml_str = r#"
[scan]
exclude = ["target", "node_modules", "*.min.js", "vendor/**"]
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let excl = cfg.scan.exclude.as_ref().unwrap();
    assert_eq!(excl.len(), 4);
    assert!(excl.contains(&"*.min.js".to_string()));
}

// =============================================================================
// 34. TOML view profile BTreeMap key ordering
// =============================================================================

#[test]
fn toml_view_profiles_deterministic_ordering() {
    let toml_str = r#"
[view.zebra]
format = "json"

[view.alpha]
format = "tsv"

[view.middle]
format = "csv"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let keys: Vec<&String> = cfg.view.keys().collect();
    assert_eq!(keys, &["alpha", "middle", "zebra"]);
}

// =============================================================================
// 35. ModuleSettings roundtrip with custom module_roots
// =============================================================================

#[test]
fn module_settings_custom_roots_roundtrip() {
    let ms = ModuleSettings {
        top: 5,
        module_roots: vec!["apps".into(), "libs".into(), "shared".into()],
        module_depth: 4,
        children: ChildIncludeMode::ParentsOnly,
        redact: Some(RedactMode::Paths),
    };
    let json = serde_json::to_string(&ms).unwrap();
    let back: ModuleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.module_roots.len(), 3);
    assert_eq!(back.module_depth, 4);
    assert!(matches!(back.children, ChildIncludeMode::ParentsOnly));
    assert!(matches!(back.redact, Some(RedactMode::Paths)));
}

// =============================================================================
// 36. ExportSettings with strip_prefix
// =============================================================================

#[test]
fn export_settings_strip_prefix_roundtrip() {
    let es = ExportSettings {
        strip_prefix: Some("/home/user/project".into()),
        ..Default::default()
    };
    let json = serde_json::to_string(&es).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.strip_prefix.as_deref(), Some("/home/user/project"));
}

// =============================================================================
// 37. CockpitSettings roundtrip with baseline
// =============================================================================

#[test]
fn cockpit_settings_with_baseline() {
    let c = CockpitSettings {
        base: "release/1.0".into(),
        head: "feature/new".into(),
        range_mode: "three-dot".into(),
        baseline: Some("baseline.json".into()),
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "release/1.0");
    assert_eq!(back.range_mode, "three-dot");
    assert_eq!(back.baseline.as_deref(), Some("baseline.json"));
}

// =============================================================================
// 38. RatchetRuleConfig with minimal fields
// =============================================================================

#[test]
fn ratchet_rule_minimal_fields() {
    let r = RatchetRuleConfig {
        pointer: "/metrics/lines".into(),
        max_increase_pct: None,
        max_value: None,
        level: None,
        description: None,
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: RatchetRuleConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.pointer, "/metrics/lines");
    assert!(back.max_increase_pct.is_none());
}

// =============================================================================
// 39. GateConfig with allow_missing flags
// =============================================================================

#[test]
fn gate_config_allow_missing_flags() {
    let gc = GateConfig {
        policy: None,
        baseline: None,
        preset: None,
        fail_fast: None,
        rules: None,
        ratchet: None,
        allow_missing_baseline: Some(true),
        allow_missing_current: Some(false),
    };
    let json = serde_json::to_string(&gc).unwrap();
    let back: GateConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.allow_missing_baseline, Some(true));
    assert_eq!(back.allow_missing_current, Some(false));
}

// =============================================================================
// 40. Debug trait is implemented for all types
// =============================================================================

#[test]
fn debug_trait_all_types() {
    let _ = format!("{:?}", ScanOptions::default());
    let _ = format!("{:?}", ScanSettings::default());
    let _ = format!("{:?}", LangSettings::default());
    let _ = format!("{:?}", ModuleSettings::default());
    let _ = format!("{:?}", ExportSettings::default());
    let _ = format!("{:?}", AnalyzeSettings::default());
    let _ = format!("{:?}", CockpitSettings::default());
    let _ = format!("{:?}", DiffSettings::default());
    let _ = format!("{:?}", TomlConfig::default());
    let _ = format!("{:?}", ViewProfile::default());
    let _ = format!(
        "{:?}",
        GateRule {
            name: "x".into(),
            pointer: "/a".into(),
            op: "==".into(),
            value: None,
            values: None,
            negate: false,
            level: None,
            message: None,
        }
    );
    let _ = format!(
        "{:?}",
        RatchetRuleConfig {
            pointer: "/a".into(),
            max_increase_pct: None,
            max_value: None,
            level: None,
            description: None,
        }
    );
}

// =============================================================================
// 41. TOML full config roundtrip via JSON
// =============================================================================

#[test]
fn toml_full_config_json_roundtrip() {
    let toml_str = r#"
[scan]
paths = ["src"]
exclude = ["target"]
hidden = false
config = "auto"
no_ignore = false
no_ignore_parent = false
no_ignore_dot = false
no_ignore_vcs = false
doc_comments = true

[module]
roots = ["crates", "packages"]
depth = 2
children = "separate"

[export]
min_code = 5
max_rows = 1000
redact = "none"
format = "jsonl"
children = "collapse"

[analyze]
preset = "risk"
window = 64000
git = true
max_files = 10000
max_bytes = 100000000
max_file_bytes = 5000000
max_commits = 500
max_commit_files = 50
granularity = "file"

[context]
budget = "128k"
strategy = "greedy"
rank_by = "hotspot"
output = "bundle"
compress = true

[badge]
metric = "code"

[gate]
policy = "policy.json"
baseline = "baseline.json"
preset = "health"
fail_fast = false
allow_missing_baseline = true
allow_missing_current = false

[view.ci]
format = "tsv"
top = 20
preset = "risk"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    // Roundtrip through JSON
    let json = serde_json::to_string_pretty(&cfg).unwrap();
    let back: TomlConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scan.config.as_deref(), cfg.scan.config.as_deref());
    assert_eq!(back.analyze.preset.as_deref(), Some("risk"));
    assert_eq!(back.view.len(), 1);
}

// =============================================================================
// 42. ScanSettings::for_paths with empty vec
// =============================================================================

#[test]
fn scan_settings_for_paths_empty() {
    let s = ScanSettings::for_paths(vec![]);
    assert!(s.paths.is_empty());
}

// =============================================================================
// 43. TOML context config roundtrip
// =============================================================================

#[test]
fn toml_context_config() {
    let toml_str = r#"
[context]
budget = "256k"
strategy = "spread"
rank_by = "tokens"
output = "json"
compress = false
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.context.budget.as_deref(), Some("256k"));
    assert_eq!(cfg.context.strategy.as_deref(), Some("spread"));
    assert_eq!(cfg.context.rank_by.as_deref(), Some("tokens"));
    assert_eq!(cfg.context.output.as_deref(), Some("json"));
    assert_eq!(cfg.context.compress, Some(false));
}
