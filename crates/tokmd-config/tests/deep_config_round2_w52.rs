//! Deep round-2 tests for tokmd-config: TOML loading, profile resolution,
//! and settings validation.

use std::io::Write;
use tempfile::NamedTempFile;
use tokmd_config::{GlobalArgs, Profile, TomlConfig, UserConfig};
use tokmd_settings::{
    AnalyzeSettings, ExportSettings, LangSettings, ModuleSettings, ScanOptions, ScanSettings,
};

// =========================================================================
// 1. TOML File Loading (8+ tests)
// =========================================================================

#[test]
fn parse_valid_toml_with_all_fields_populated() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target", "**/*.min.js"]
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
max_rows = 500
redact = "paths"
format = "csv"
children = "separate"

[analyze]
preset = "deep"
window = 128000
format = "json"
git = true
max_files = 10000
max_bytes = 50000000
max_file_bytes = 1000000
max_commits = 500
max_commit_files = 200
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

[view.ci]
format = "json"
top = 10
files = true
redact = "paths"
"#;
    let cfg = TomlConfig::parse(toml_str).expect("valid full TOML");

    // scan
    assert_eq!(cfg.scan.paths, Some(vec!["src".into(), "lib".into()]));
    assert_eq!(
        cfg.scan.exclude,
        Some(vec!["target".into(), "**/*.min.js".into()])
    );
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.config, Some("none".into()));
    assert_eq!(cfg.scan.no_ignore, Some(false));
    assert_eq!(cfg.scan.no_ignore_parent, Some(true));
    assert_eq!(cfg.scan.no_ignore_dot, Some(false));
    assert_eq!(cfg.scan.no_ignore_vcs, Some(true));
    assert_eq!(cfg.scan.doc_comments, Some(true));

    // module
    assert_eq!(
        cfg.module.roots,
        Some(vec!["crates".into(), "packages".into(), "libs".into()])
    );
    assert_eq!(cfg.module.depth, Some(3));
    assert_eq!(cfg.module.children, Some("collapse".into()));

    // export
    assert_eq!(cfg.export.min_code, Some(5));
    assert_eq!(cfg.export.max_rows, Some(500));
    assert_eq!(cfg.export.redact, Some("paths".into()));
    assert_eq!(cfg.export.format, Some("csv".into()));
    assert_eq!(cfg.export.children, Some("separate".into()));

    // analyze
    assert_eq!(cfg.analyze.preset, Some("deep".into()));
    assert_eq!(cfg.analyze.window, Some(128000));
    assert_eq!(cfg.analyze.format, Some("json".into()));
    assert_eq!(cfg.analyze.git, Some(true));
    assert_eq!(cfg.analyze.max_files, Some(10000));
    assert_eq!(cfg.analyze.max_bytes, Some(50_000_000));
    assert_eq!(cfg.analyze.max_file_bytes, Some(1_000_000));
    assert_eq!(cfg.analyze.max_commits, Some(500));
    assert_eq!(cfg.analyze.max_commit_files, Some(200));
    assert_eq!(cfg.analyze.granularity, Some("file".into()));

    // context
    assert_eq!(cfg.context.budget, Some("256k".into()));
    assert_eq!(cfg.context.strategy, Some("spread".into()));
    assert_eq!(cfg.context.rank_by, Some("hotspot".into()));
    assert_eq!(cfg.context.output, Some("bundle".into()));
    assert_eq!(cfg.context.compress, Some(true));

    // badge
    assert_eq!(cfg.badge.metric, Some("tokens".into()));

    // gate
    assert_eq!(cfg.gate.policy, Some("policy.toml".into()));
    assert_eq!(cfg.gate.baseline, Some("baseline.json".into()));
    assert_eq!(cfg.gate.preset, Some("health".into()));
    assert_eq!(cfg.gate.fail_fast, Some(true));

    // view profile
    let ci = cfg.view.get("ci").expect("ci profile should exist");
    assert_eq!(ci.format, Some("json".into()));
    assert_eq!(ci.top, Some(10));
    assert_eq!(ci.files, Some(true));
    assert_eq!(ci.redact, Some("paths".into()));
}

#[test]
fn parse_minimal_toml_single_section() {
    let toml_str = "[scan]\nhidden = true\n";
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.scan.hidden, Some(true));
    // Everything else stays default/None
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.scan.exclude.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.export.format.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn parse_toml_with_custom_profile_definitions() {
    let toml_str = r#"
[view.llm_safe]
format = "json"
redact = "all"
top = 20
max_rows = 100

[view.ci]
format = "json"
preset = "receipt"
compress = true

[view.human]
format = "md"
files = true
top = 15
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.view.len(), 3);

    let llm = cfg.view.get("llm_safe").unwrap();
    assert_eq!(llm.format, Some("json".into()));
    assert_eq!(llm.redact, Some("all".into()));
    assert_eq!(llm.top, Some(20));
    assert_eq!(llm.max_rows, Some(100));

    let ci = cfg.view.get("ci").unwrap();
    assert_eq!(ci.preset, Some("receipt".into()));
    assert_eq!(ci.compress, Some(true));

    let human = cfg.view.get("human").unwrap();
    assert_eq!(human.format, Some("md".into()));
    assert_eq!(human.files, Some(true));
    assert_eq!(human.top, Some(15));
}

#[test]
fn defaults_applied_for_missing_optional_fields() {
    let toml_str = r#"
[analyze]
preset = "health"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();

    // Explicitly set field
    assert_eq!(cfg.analyze.preset, Some("health".into()));

    // All other analyze fields should remain None
    assert!(cfg.analyze.window.is_none());
    assert!(cfg.analyze.format.is_none());
    assert!(cfg.analyze.git.is_none());
    assert!(cfg.analyze.max_files.is_none());
    assert!(cfg.analyze.max_bytes.is_none());
    assert!(cfg.analyze.max_file_bytes.is_none());
    assert!(cfg.analyze.max_commits.is_none());
    assert!(cfg.analyze.max_commit_files.is_none());
    assert!(cfg.analyze.granularity.is_none());

    // Other sections should also be defaults
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.module.depth.is_none());
}

#[test]
fn exclude_patterns_parsed_correctly() {
    let toml_str = r#"
[scan]
exclude = [
    "target",
    "node_modules",
    "**/*.min.js",
    "dist/**",
    ".git",
    "*.pyc",
]
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let excludes = cfg.scan.exclude.expect("excludes present");
    assert_eq!(excludes.len(), 6);
    assert_eq!(excludes[0], "target");
    assert_eq!(excludes[1], "node_modules");
    assert_eq!(excludes[2], "**/*.min.js");
    assert_eq!(excludes[3], "dist/**");
    assert_eq!(excludes[4], ".git");
    assert_eq!(excludes[5], "*.pyc");
}

#[test]
fn custom_analysis_presets_parsed() {
    let toml_str = r#"
[analyze]
preset = "security"
window = 64000
git = false
max_files = 5000
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.analyze.preset, Some("security".into()));
    assert_eq!(cfg.analyze.window, Some(64000));
    assert_eq!(cfg.analyze.git, Some(false));
    assert_eq!(cfg.analyze.max_files, Some(5000));
}

#[test]
fn empty_toml_uses_all_defaults() {
    let cfg = TomlConfig::parse("").unwrap();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.scan.exclude.is_none());
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.module.depth.is_none());
    assert!(cfg.export.format.is_none());
    assert!(cfg.export.min_code.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.policy.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn malformed_toml_errors_gracefully() {
    // Missing closing bracket
    let result = TomlConfig::parse("[scan\nhidden = true");
    assert!(result.is_err());

    // Invalid value type (string where bool expected)
    let result = TomlConfig::parse("[scan]\nhidden = \"yes\"");
    assert!(result.is_err());

    // Duplicate section
    let result = TomlConfig::parse("[scan]\nhidden = true\n[scan]\nhidden = false");
    assert!(result.is_err());

    // Completely broken syntax
    let result = TomlConfig::parse("}{invalid{{}}}");
    assert!(result.is_err());
}

#[test]
fn toml_from_file_loads_correctly() {
    let toml_content = r#"
[scan]
hidden = true
exclude = ["target"]

[module]
depth = 4
roots = ["crates"]
"#;
    let mut temp = NamedTempFile::new().expect("create temp file");
    temp.write_all(toml_content.as_bytes()).expect("write");

    let cfg = TomlConfig::from_file(temp.path()).expect("load from file");
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.exclude, Some(vec!["target".into()]));
    assert_eq!(cfg.module.depth, Some(4));
    assert_eq!(cfg.module.roots, Some(vec!["crates".into()]));
}

#[test]
fn toml_from_file_nonexistent_path_errors() {
    let result = TomlConfig::from_file(std::path::Path::new("/no/such/file/tokmd.toml"));
    assert!(result.is_err());
}

// =========================================================================
// 2. Profile Resolution (6+ tests)
// =========================================================================

#[test]
fn default_profile_selection_all_none() {
    let cfg = UserConfig::default();
    assert!(cfg.profiles.is_empty());
    // With no profiles, looking up any name yields None
    assert!(!cfg.profiles.contains_key("default"));
    assert!(!cfg.profiles.contains_key("ci"));
}

#[test]
fn named_profile_selection_overrides_defaults() {
    let mut cfg = UserConfig::default();
    cfg.profiles.insert(
        "llm".into(),
        Profile {
            format: Some("json".into()),
            top: Some(10),
            redact: Some(tokmd_config::RedactMode::Paths),
            ..Profile::default()
        },
    );

    let llm = cfg.profiles.get("llm").expect("llm profile exists");
    assert_eq!(llm.format, Some("json".into()));
    assert_eq!(llm.top, Some(10));
    assert!(llm.redact.is_some());
    // Fields not set remain None
    assert!(llm.files.is_none());
    assert!(llm.module_roots.is_none());
    assert!(llm.children.is_none());
}

#[test]
fn profile_inheritance_view_profiles_from_toml() {
    // ViewProfile in TomlConfig simulates inheritance: section defaults + profile overrides
    let toml_str = r#"
[export]
format = "jsonl"
min_code = 10

[view.ci]
format = "csv"
min_code = 0

[view.llm]
format = "json"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();

    // Section-level defaults
    assert_eq!(cfg.export.format, Some("jsonl".into()));
    assert_eq!(cfg.export.min_code, Some(10));

    // ci profile overrides both
    let ci = cfg.view.get("ci").unwrap();
    assert_eq!(ci.format, Some("csv".into()));
    assert_eq!(ci.min_code, Some(0));

    // llm profile overrides format only; min_code stays None at profile level
    let llm = cfg.view.get("llm").unwrap();
    assert_eq!(llm.format, Some("json".into()));
    assert!(llm.min_code.is_none());
}

#[test]
fn profile_with_format_overrides() {
    let toml_str = r#"
[view.json_out]
format = "json"
top = 50

[view.csv_out]
format = "csv"
max_rows = 1000
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();

    let json_prof = cfg.view.get("json_out").unwrap();
    assert_eq!(json_prof.format, Some("json".into()));
    assert_eq!(json_prof.top, Some(50));
    assert!(json_prof.max_rows.is_none());

    let csv_prof = cfg.view.get("csv_out").unwrap();
    assert_eq!(csv_prof.format, Some("csv".into()));
    assert_eq!(csv_prof.max_rows, Some(1000));
    assert!(csv_prof.top.is_none());
}

#[test]
fn profile_settings_override_global_settings() {
    let toml_str = r#"
[analyze]
preset = "receipt"
window = 128000
git = false

[view.risk_view]
preset = "risk"
window = 256000
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();

    // Global analyze settings
    assert_eq!(cfg.analyze.preset, Some("receipt".into()));
    assert_eq!(cfg.analyze.window, Some(128000));
    assert_eq!(cfg.analyze.git, Some(false));

    // Profile-level overrides
    let risk = cfg.view.get("risk_view").unwrap();
    assert_eq!(risk.preset, Some("risk".into()));
    assert_eq!(risk.window, Some(256000));
}

#[test]
fn unknown_profile_name_returns_none() {
    let toml_str = r#"
[view.ci]
format = "json"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert!(!cfg.view.contains_key("nonexistent"));
    assert!(!cfg.view.contains_key(""));
    assert!(!cfg.view.contains_key("CI")); // case-sensitive
}

#[test]
fn user_config_repo_to_profile_mapping() {
    let mut cfg = UserConfig::default();
    cfg.repos.insert("org/repo-a".into(), "ci".into());
    cfg.repos.insert("org/repo-b".into(), "llm".into());
    cfg.profiles.insert(
        "ci".into(),
        Profile {
            format: Some("json".into()),
            ..Profile::default()
        },
    );
    cfg.profiles.insert(
        "llm".into(),
        Profile {
            format: Some("json".into()),
            redact: Some(tokmd_config::RedactMode::Paths),
            ..Profile::default()
        },
    );

    // Look up profile name for a repo, then resolve it
    let profile_name = cfg.repos.get("org/repo-a").expect("mapped");
    assert_eq!(profile_name, "ci");
    let profile = cfg.profiles.get(profile_name).expect("profile exists");
    assert_eq!(profile.format, Some("json".into()));

    let profile_name_b = cfg.repos.get("org/repo-b").unwrap();
    let profile_b = cfg.profiles.get(profile_name_b).unwrap();
    assert!(profile_b.redact.is_some());
}

// =========================================================================
// 3. Settings Validation (6+ tests)
// =========================================================================

#[test]
fn scan_options_builder_combinations() {
    // Default
    let opts = ScanOptions::default();
    assert!(opts.excluded.is_empty());
    assert!(!opts.hidden);
    assert!(!opts.no_ignore);
    assert!(!opts.no_ignore_parent);
    assert!(!opts.no_ignore_dot);
    assert!(!opts.no_ignore_vcs);
    assert!(!opts.treat_doc_strings_as_comments);

    // Fully populated
    let opts = ScanOptions {
        excluded: vec!["target".into(), "node_modules".into()],
        config: tokmd_settings::ConfigMode::None,
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
    assert!(opts.treat_doc_strings_as_comments);

    // Partial: only excludes
    let opts = ScanOptions {
        excluded: vec!["*.bak".into()],
        ..ScanOptions::default()
    };
    assert_eq!(opts.excluded.len(), 1);
    assert!(!opts.hidden);
}

#[test]
fn analyze_settings_preset_validation() {
    let default = AnalyzeSettings::default();
    assert_eq!(default.preset, "receipt");
    assert_eq!(default.granularity, "module");
    assert!(default.window.is_none());
    assert!(default.git.is_none());

    // Custom preset
    let custom = AnalyzeSettings {
        preset: "deep".into(),
        window: Some(256000),
        git: Some(true),
        max_files: Some(50000),
        max_bytes: Some(100_000_000),
        max_file_bytes: Some(5_000_000),
        max_commits: Some(1000),
        max_commit_files: Some(500),
        granularity: "file".into(),
    };
    assert_eq!(custom.preset, "deep");
    assert_eq!(custom.granularity, "file");
    assert_eq!(custom.max_files, Some(50000));
}

#[test]
fn export_settings_format_validation() {
    let default = ExportSettings::default();
    assert_eq!(default.format, tokmd_settings::ExportFormat::Jsonl);
    assert_eq!(default.min_code, 0);
    assert_eq!(default.max_rows, 0);
    assert!(default.meta);
    assert_eq!(default.redact, tokmd_settings::RedactMode::None);
    assert!(default.strip_prefix.is_none());

    // Module defaults
    assert_eq!(
        default.module_roots,
        vec!["crates".to_string(), "packages".to_string()]
    );
    assert_eq!(default.module_depth, 2);
}

#[test]
fn path_normalization_in_exclude_patterns() {
    // Verify that backslash and forward-slash patterns both parse
    let toml_str = r#"
[scan]
exclude = ["src/generated", "tests\\fixtures", "**/*.snap"]
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let excludes = cfg.scan.exclude.unwrap();
    assert_eq!(excludes.len(), 3);
    assert_eq!(excludes[0], "src/generated");
    assert_eq!(excludes[1], "tests\\fixtures");
    assert_eq!(excludes[2], "**/*.snap");
}

#[test]
fn numerical_setting_bounds() {
    // top=0 means "show all"
    let settings = LangSettings {
        top: 0,
        ..LangSettings::default()
    };
    assert_eq!(settings.top, 0);

    // Large top value
    let settings = LangSettings {
        top: usize::MAX,
        ..LangSettings::default()
    };
    assert_eq!(settings.top, usize::MAX);

    // Module depth bounds
    let m = ModuleSettings {
        module_depth: 1,
        ..ModuleSettings::default()
    };
    assert_eq!(m.module_depth, 1);

    let m = ModuleSettings {
        module_depth: 100,
        ..ModuleSettings::default()
    };
    assert_eq!(m.module_depth, 100);

    // Export max_rows = 0 means unlimited
    let e = ExportSettings {
        max_rows: 0,
        ..ExportSettings::default()
    };
    assert_eq!(e.max_rows, 0);

    // Context budget as string
    let toml_str = "[context]\nbudget = \"1g\"";
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.context.budget, Some("1g".into()));
}

#[test]
fn serde_roundtrip_full_toml_config() {
    let toml_str = r#"
[scan]
paths = ["src"]
exclude = ["target"]
hidden = true
no_ignore_vcs = true
doc_comments = false

[module]
roots = ["crates"]
depth = 2
children = "separate"

[export]
format = "csv"
min_code = 10
max_rows = 200
redact = "paths"

[analyze]
preset = "risk"
window = 64000
git = true

[context]
budget = "128k"
strategy = "greedy"

[badge]
metric = "lines"

[gate]
policy = "gate.toml"
fail_fast = true

[view.test_profile]
format = "json"
top = 5
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();

    // Serialize to JSON and back
    let json = serde_json::to_string(&cfg).expect("serialize to JSON");
    let roundtrip: TomlConfig = serde_json::from_str(&json).expect("deserialize from JSON");

    // Verify key fields survived the roundtrip
    assert_eq!(roundtrip.scan.paths, Some(vec!["src".into()]));
    assert_eq!(roundtrip.scan.exclude, Some(vec!["target".into()]));
    assert_eq!(roundtrip.scan.hidden, Some(true));
    assert_eq!(roundtrip.scan.no_ignore_vcs, Some(true));
    assert_eq!(roundtrip.scan.doc_comments, Some(false));
    assert_eq!(roundtrip.module.roots, Some(vec!["crates".into()]));
    assert_eq!(roundtrip.module.depth, Some(2));
    assert_eq!(roundtrip.module.children, Some("separate".into()));
    assert_eq!(roundtrip.export.format, Some("csv".into()));
    assert_eq!(roundtrip.export.min_code, Some(10));
    assert_eq!(roundtrip.export.max_rows, Some(200));
    assert_eq!(roundtrip.export.redact, Some("paths".into()));
    assert_eq!(roundtrip.analyze.preset, Some("risk".into()));
    assert_eq!(roundtrip.analyze.window, Some(64000));
    assert_eq!(roundtrip.analyze.git, Some(true));
    assert_eq!(roundtrip.context.budget, Some("128k".into()));
    assert_eq!(roundtrip.context.strategy, Some("greedy".into()));
    assert_eq!(roundtrip.badge.metric, Some("lines".into()));
    assert_eq!(roundtrip.gate.policy, Some("gate.toml".into()));
    assert_eq!(roundtrip.gate.fail_fast, Some(true));

    let prof = roundtrip.view.get("test_profile").unwrap();
    assert_eq!(prof.format, Some("json".into()));
    assert_eq!(prof.top, Some(5));
}

#[test]
fn gate_rules_inline_toml_parsing() {
    let toml_str = r#"
[gate]
fail_fast = true

[[gate.rules]]
name = "max_code_lines"
pointer = "/summary/total_code"
op = "<="
value = 100000

[[gate.rules]]
name = "min_test_ratio"
pointer = "/derived/test_ratio"
op = ">="
value = 0.1

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
max_value = 15.0
level = "error"
description = "Cyclomatic complexity must not increase by more than 5%"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(cfg.gate.fail_fast, Some(true));

    let rules = cfg.gate.rules.expect("rules present");
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].name, "max_code_lines");
    assert_eq!(rules[0].pointer, "/summary/total_code");
    assert_eq!(rules[0].op, "<=");
    assert_eq!(rules[1].name, "min_test_ratio");
    assert_eq!(rules[1].op, ">=");

    let ratchet = cfg.gate.ratchet.expect("ratchet rules present");
    assert_eq!(ratchet.len(), 1);
    assert_eq!(ratchet[0].pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchet[0].max_increase_pct, Some(5.0));
    assert_eq!(ratchet[0].max_value, Some(15.0));
    assert_eq!(ratchet[0].level, Some("error".into()));
    assert!(ratchet[0].description.is_some());
}

#[test]
fn view_profile_all_fields_populated() {
    let toml_str = r#"
[view.full]
format = "json"
top = 25
files = true
module_roots = ["crates", "packages"]
module_depth = 3
min_code = 5
max_rows = 500
redact = "all"
meta = true
children = "collapse"
preset = "deep"
window = 200000
budget = "256k"
strategy = "spread"
rank_by = "hotspot"
output = "bundle"
compress = true
metric = "tokens"
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let full = cfg.view.get("full").unwrap();

    assert_eq!(full.format, Some("json".into()));
    assert_eq!(full.top, Some(25));
    assert_eq!(full.files, Some(true));
    assert_eq!(
        full.module_roots,
        Some(vec!["crates".into(), "packages".into()])
    );
    assert_eq!(full.module_depth, Some(3));
    assert_eq!(full.min_code, Some(5));
    assert_eq!(full.max_rows, Some(500));
    assert_eq!(full.redact, Some("all".into()));
    assert_eq!(full.meta, Some(true));
    assert_eq!(full.children, Some("collapse".into()));
    assert_eq!(full.preset, Some("deep".into()));
    assert_eq!(full.window, Some(200000));
    assert_eq!(full.budget, Some("256k".into()));
    assert_eq!(full.strategy, Some("spread".into()));
    assert_eq!(full.rank_by, Some("hotspot".into()));
    assert_eq!(full.output, Some("bundle".into()));
    assert_eq!(full.compress, Some(true));
    assert_eq!(full.metric, Some("tokens".into()));
}

#[test]
fn global_args_to_scan_options_conversion() {
    let g = GlobalArgs {
        excluded: vec!["target".into(), "dist".into()],
        config: tokmd_config::ConfigMode::None,
        hidden: true,
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
        verbose: 2,
        no_progress: true,
    };
    let opts: ScanOptions = (&g).into();

    assert_eq!(opts.excluded, vec!["target", "dist"]);
    assert!(opts.hidden);
    assert!(!opts.no_ignore);
    assert!(opts.no_ignore_parent);
    assert!(opts.no_ignore_vcs);
    assert!(opts.treat_doc_strings_as_comments);
    // verbose and no_progress are UI-only, not in ScanOptions
}

#[test]
fn scan_settings_serde_roundtrip_with_options() {
    let settings = ScanSettings {
        paths: vec!["src".into(), "tests".into()],
        options: ScanOptions {
            excluded: vec!["target".into(), "**/*.bak".into()],
            hidden: true,
            no_ignore_vcs: true,
            ..ScanOptions::default()
        },
    };

    let json = serde_json::to_string(&settings).unwrap();
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths, vec!["src", "tests"]);
    assert_eq!(back.options.excluded, vec!["target", "**/*.bak"]);
    assert!(back.options.hidden);
    assert!(back.options.no_ignore_vcs);
    assert!(!back.options.no_ignore);
}
