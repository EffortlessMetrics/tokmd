//! Wave 47 deep tests for `tokmd-config`.
//!
//! Covers: TOML config parsing, profile resolution, settings merging,
//! config file discovery, determinism, property-based roundtrips,
//! edge cases, enum validation, and CliChildrenMode/CliChildIncludeMode parsing.

use std::collections::BTreeMap;
use std::io::Write;

use proptest::prelude::*;
use tempfile::{NamedTempFile, TempDir};
use tokmd_cli_args::{
    CliAnalysisFormat, CliChildIncludeMode, CliChildrenMode, CliConfigMode, CliExportFormat,
    CliRedactMode, CliTableFormat, Profile, ScanConfig, TomlConfig, UserConfig, ViewProfile,
};

// =========================================================================
// 1. TOML config parsing with all fields populated
// =========================================================================

#[test]
fn full_config_all_sections_populated() {
    let toml_str = r#"
[scan]
paths = ["src", "lib", "tests"]
exclude = ["target", "node_modules", "dist"]
hidden = true
config = "none"
no_ignore = true
no_ignore_parent = true
no_ignore_dot = true
no_ignore_vcs = true
doc_comments = true

[module]
roots = ["crates", "packages", "libs"]
depth = 4
children = "collapse"

[export]
min_code = 10
max_rows = 5000
redact = "all"
format = "csv"
children = "separate"

[analyze]
preset = "deep"
window = 256000
format = "json"
git = true
max_files = 10000
max_bytes = 100000000
max_file_bytes = 500000
max_commits = 2000
max_commit_files = 200
granularity = "file"

[context]
budget = "2m"
strategy = "spread"
rank_by = "hotspot"
output = "bundle"
compress = true

[badge]
metric = "tokens"

[gate]
policy = "policy.toml"
baseline = "baseline.json"
preset = "risk"
fail_fast = true
allow_missing_baseline = false
allow_missing_current = true

[[gate.rules]]
name = "max-lines"
pointer = "/summary/total_code"
op = "lt"
value = 100000

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
max_value = 25.0
level = "error"
description = "cyclomatic complexity ceiling"

[view.llm_safe]
format = "json"
top = 15
redact = "all"
compress = true

[view.ci]
format = "tsv"
min_code = 1
meta = false
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();

    // Scan
    assert_eq!(
        cfg.scan.paths,
        Some(vec!["src".into(), "lib".into(), "tests".into()])
    );
    assert_eq!(cfg.scan.exclude.as_ref().unwrap().len(), 3);
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.config, Some("none".into()));
    assert_eq!(cfg.scan.no_ignore, Some(true));
    assert_eq!(cfg.scan.no_ignore_parent, Some(true));
    assert_eq!(cfg.scan.no_ignore_dot, Some(true));
    assert_eq!(cfg.scan.no_ignore_vcs, Some(true));
    assert_eq!(cfg.scan.doc_comments, Some(true));

    // Module
    assert_eq!(cfg.module.roots.as_ref().unwrap().len(), 3);
    assert_eq!(cfg.module.depth, Some(4));
    assert_eq!(cfg.module.children, Some("collapse".into()));

    // Export
    assert_eq!(cfg.export.min_code, Some(10));
    assert_eq!(cfg.export.max_rows, Some(5000));
    assert_eq!(cfg.export.redact, Some("all".into()));
    assert_eq!(cfg.export.format, Some("csv".into()));
    assert_eq!(cfg.export.children, Some("separate".into()));

    // Analyze
    assert_eq!(cfg.analyze.preset, Some("deep".into()));
    assert_eq!(cfg.analyze.window, Some(256000));
    assert_eq!(cfg.analyze.format, Some("json".into()));
    assert_eq!(cfg.analyze.git, Some(true));
    assert_eq!(cfg.analyze.max_files, Some(10000));
    assert_eq!(cfg.analyze.max_bytes, Some(100_000_000));
    assert_eq!(cfg.analyze.max_file_bytes, Some(500_000));
    assert_eq!(cfg.analyze.max_commits, Some(2000));
    assert_eq!(cfg.analyze.max_commit_files, Some(200));
    assert_eq!(cfg.analyze.granularity, Some("file".into()));

    // Context
    assert_eq!(cfg.context.budget, Some("2m".into()));
    assert_eq!(cfg.context.strategy, Some("spread".into()));
    assert_eq!(cfg.context.rank_by, Some("hotspot".into()));
    assert_eq!(cfg.context.output, Some("bundle".into()));
    assert_eq!(cfg.context.compress, Some(true));

    // Badge
    assert_eq!(cfg.badge.metric, Some("tokens".into()));

    // Gate
    assert_eq!(cfg.gate.policy, Some("policy.toml".into()));
    assert_eq!(cfg.gate.baseline, Some("baseline.json".into()));
    assert_eq!(cfg.gate.preset, Some("risk".into()));
    assert_eq!(cfg.gate.fail_fast, Some(true));
    assert_eq!(cfg.gate.allow_missing_baseline, Some(false));
    assert_eq!(cfg.gate.allow_missing_current, Some(true));
    assert_eq!(cfg.gate.rules.as_ref().unwrap().len(), 1);
    assert_eq!(cfg.gate.ratchet.as_ref().unwrap().len(), 1);

    // Ratchet rule details
    let ratchet = &cfg.gate.ratchet.as_ref().unwrap()[0];
    assert_eq!(ratchet.pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchet.max_increase_pct, Some(5.0));
    assert_eq!(ratchet.max_value, Some(25.0));
    assert_eq!(ratchet.level, Some("error".into()));
    assert_eq!(
        ratchet.description,
        Some("cyclomatic complexity ceiling".into())
    );

    // View profiles
    assert_eq!(cfg.view.len(), 2);
    let llm = &cfg.view["llm_safe"];
    assert_eq!(llm.format, Some("json".into()));
    assert_eq!(llm.top, Some(15));
    assert_eq!(llm.redact, Some("all".into()));
    assert_eq!(llm.compress, Some(true));

    let ci = &cfg.view["ci"];
    assert_eq!(ci.format, Some("tsv".into()));
    assert_eq!(ci.min_code, Some(1));
    assert_eq!(ci.meta, Some(false));
}

// =========================================================================
// 2. Missing optional fields fallback to defaults (None)
// =========================================================================

#[test]
fn missing_optional_fields_are_none() {
    let cfg = TomlConfig::parse("[scan]\nhidden = true").unwrap();
    assert_eq!(cfg.scan.hidden, Some(true));
    // Everything else should be None/empty
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.scan.exclude.is_none());
    assert!(cfg.scan.config.is_none());
    assert!(cfg.scan.no_ignore.is_none());
    assert!(cfg.scan.no_ignore_parent.is_none());
    assert!(cfg.scan.no_ignore_dot.is_none());
    assert!(cfg.scan.no_ignore_vcs.is_none());
    assert!(cfg.scan.doc_comments.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.module.depth.is_none());
    assert!(cfg.module.children.is_none());
    assert!(cfg.export.min_code.is_none());
    assert!(cfg.export.max_rows.is_none());
    assert!(cfg.export.redact.is_none());
    assert!(cfg.export.format.is_none());
    assert!(cfg.export.children.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.analyze.window.is_none());
    assert!(cfg.analyze.format.is_none());
    assert!(cfg.analyze.git.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.context.strategy.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.fail_fast.is_none());
    assert!(cfg.gate.rules.is_none());
    assert!(cfg.gate.ratchet.is_none());
    assert!(cfg.view.is_empty());
}

// =========================================================================
// 3. Profile selection and merging
// =========================================================================

#[test]
fn user_config_profile_selection_by_name() {
    let mut profiles = BTreeMap::new();
    profiles.insert(
        "llm".to_string(),
        Profile {
            format: Some("json".into()),
            redact: Some(CliRedactMode::All),
            top: Some(10),
            ..Default::default()
        },
    );
    profiles.insert(
        "ci".to_string(),
        Profile {
            format: Some("tsv".into()),
            min_code: Some(1),
            ..Default::default()
        },
    );
    let uc = UserConfig {
        profiles,
        repos: BTreeMap::new(),
    };

    let llm = uc.profiles.get("llm").unwrap();
    assert_eq!(llm.format, Some("json".into()));
    assert_eq!(llm.redact, Some(CliRedactMode::All));
    assert_eq!(llm.top, Some(10));

    let ci = uc.profiles.get("ci").unwrap();
    assert_eq!(ci.format, Some("tsv".into()));
    assert_eq!(ci.min_code, Some(1));
    // Unset fields remain None
    assert!(ci.redact.is_none());
    assert!(ci.top.is_none());
}

#[test]
fn user_config_repo_to_profile_mapping() {
    let mut profiles = BTreeMap::new();
    profiles.insert(
        "strict".to_string(),
        Profile {
            format: Some("json".into()),
            ..Default::default()
        },
    );
    let mut repos = BTreeMap::new();
    repos.insert("owner/repo".to_string(), "strict".to_string());

    let uc = UserConfig { profiles, repos };
    let profile_name = uc.repos.get("owner/repo").unwrap();
    assert_eq!(profile_name, "strict");
    let profile = uc.profiles.get(profile_name).unwrap();
    assert_eq!(profile.format, Some("json".into()));
}

#[test]
fn view_profile_merging_partial_overrides() {
    let toml_str = r#"
[scan]
hidden = false

[view.base]
format = "md"
top = 20
min_code = 5
compress = false

[view.override_top]
format = "json"
top = 5
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();

    let base = &cfg.view["base"];
    assert_eq!(base.format, Some("md".into()));
    assert_eq!(base.top, Some(20));
    assert_eq!(base.min_code, Some(5));
    assert_eq!(base.compress, Some(false));

    let over = &cfg.view["override_top"];
    assert_eq!(over.format, Some("json".into()));
    assert_eq!(over.top, Some(5));
    // Fields not in override_top remain None (not inherited from base)
    assert!(over.min_code.is_none());
    assert!(over.compress.is_none());
}

// =========================================================================
// 4. Invalid TOML handling (error messages)
// =========================================================================

#[test]
fn invalid_toml_missing_bracket_gives_error() {
    let result = TomlConfig::parse("[scan\nhidden = true");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(!err_msg.is_empty());
}

#[test]
fn invalid_toml_wrong_type_gives_descriptive_error() {
    let result = TomlConfig::parse("[module]\ndepth = \"not_a_number\"");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    // Error should mention type mismatch
    assert!(
        err_msg.contains("invalid type") || err_msg.contains("expected"),
        "Error message should be descriptive: {err_msg}"
    );
}

#[test]
fn invalid_toml_duplicate_key_is_error() {
    let result = TomlConfig::parse("[scan]\nhidden = true\nhidden = false");
    assert!(result.is_err());
}

#[test]
fn invalid_toml_negative_for_usize() {
    let result = TomlConfig::parse("[export]\nmin_code = -5");
    assert!(result.is_err());
}

#[test]
fn invalid_toml_bool_for_string_field() {
    let result = TomlConfig::parse("[context]\nbudget = true");
    assert!(result.is_err());
}

// =========================================================================
// 5. Config file discovery (from_file with temp directories)
// =========================================================================

#[test]
fn from_file_loads_valid_toml() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "[scan]\nhidden = true\n[module]\ndepth = 3").unwrap();
    let cfg = TomlConfig::from_file(f.path()).unwrap();
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.module.depth, Some(3));
}

#[test]
fn from_file_missing_file_gives_not_found() {
    let result = TomlConfig::from_file(std::path::Path::new("/nonexistent/tokmd.toml"));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn from_file_invalid_content_gives_invalid_data() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "[broken\nhidden = true").unwrap();
    let result = TomlConfig::from_file(f.path());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn from_file_in_nested_directory() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("tokmd.toml");
    std::fs::write(
        &file_path,
        "[scan]\nhidden = true\n[badge]\nmetric = \"lines\"\n",
    )
    .unwrap();
    let cfg = TomlConfig::from_file(&file_path).unwrap();
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.badge.metric, Some("lines".into()));
}

// =========================================================================
// 6. Determinism: parsing same TOML always gives same config
// =========================================================================

#[test]
fn deterministic_parse_same_toml() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
hidden = true
exclude = ["target"]

[module]
roots = ["crates", "packages"]
depth = 2

[export]
format = "jsonl"
min_code = 5

[view.alpha]
format = "json"
top = 10

[view.beta]
format = "tsv"
compress = true
"#;
    let cfg1 = TomlConfig::parse(toml_str).unwrap();
    let cfg2 = TomlConfig::parse(toml_str).unwrap();

    let json1 = serde_json::to_string(&cfg1).unwrap();
    let json2 = serde_json::to_string(&cfg2).unwrap();
    assert_eq!(
        json1, json2,
        "Parsing same TOML must produce identical JSON"
    );
}

#[test]
fn deterministic_parse_hundred_iterations() {
    let toml_str = "[scan]\nhidden = true\n[module]\ndepth = 3\n[view.x]\nformat = \"md\"";
    let reference = serde_json::to_string(&TomlConfig::parse(toml_str).unwrap()).unwrap();
    for _ in 0..100 {
        let json = serde_json::to_string(&TomlConfig::parse(toml_str).unwrap()).unwrap();
        assert_eq!(json, reference);
    }
}

#[test]
fn view_profile_keys_always_sorted() {
    let toml_str = r#"
[view.zebra]
format = "md"
[view.alpha]
format = "json"
[view.middle]
format = "tsv"
"#;
    for _ in 0..50 {
        let cfg = TomlConfig::parse(toml_str).unwrap();
        let keys: Vec<&String> = cfg.view.keys().collect();
        assert_eq!(keys, vec!["alpha", "middle", "zebra"]);
    }
}

// =========================================================================
// 7. Property test: ScanOptions roundtrips through serde
// =========================================================================

proptest! {
    #[test]
    fn scan_options_json_roundtrip(
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        no_ignore_parent in any::<bool>(),
        no_ignore_dot in any::<bool>(),
        no_ignore_vcs in any::<bool>(),
        treat_doc in any::<bool>(),
    ) {
        let opts = tokmd_settings::ScanOptions {
            excluded: vec!["target".into()],
            config: tokmd_types::ConfigMode::Auto,
            hidden,
            no_ignore,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            treat_doc_strings_as_comments: treat_doc,
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: tokmd_settings::ScanOptions = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.hidden, hidden);
        prop_assert_eq!(back.no_ignore, no_ignore);
        prop_assert_eq!(back.no_ignore_parent, no_ignore_parent);
        prop_assert_eq!(back.no_ignore_dot, no_ignore_dot);
        prop_assert_eq!(back.no_ignore_vcs, no_ignore_vcs);
        prop_assert_eq!(back.treat_doc_strings_as_comments, treat_doc);
    }
}

// =========================================================================
// 8. Property test: profile names validated / stored correctly
// =========================================================================

proptest! {
    #[test]
    fn profile_name_preserved_in_btreemap(name in "[a-z_][a-z0-9_]{0,20}") {
        let toml_str = format!("[view.{name}]\nformat = \"md\"");
        let cfg = TomlConfig::parse(&toml_str).unwrap();
        prop_assert!(cfg.view.contains_key(&name));
        prop_assert_eq!(cfg.view[&name].format.as_deref(), Some("md"));
    }
}

proptest! {
    #[test]
    fn user_config_profile_name_roundtrip(name in "[a-z][a-z0-9_]{0,15}") {
        let mut profiles = BTreeMap::new();
        profiles.insert(name.clone(), Profile {
            format: Some("json".into()),
            ..Default::default()
        });
        let uc = UserConfig { profiles, repos: BTreeMap::new() };
        let json = serde_json::to_string(&uc).unwrap();
        let back: UserConfig = serde_json::from_str(&json).unwrap();
        prop_assert!(back.profiles.contains_key(&name));
    }
}

// =========================================================================
// 9. Edge cases: empty config, config with only [profile.x]
// =========================================================================

#[test]
fn empty_config_string_produces_defaults() {
    let cfg = TomlConfig::parse("").unwrap();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.module.depth.is_none());
    assert!(cfg.export.format.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.fail_fast.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn config_with_only_view_section() {
    let cfg = TomlConfig::parse("[view.minimal]\nformat = \"md\"").unwrap();
    // All non-view sections default
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.module.depth.is_none());
    assert!(cfg.export.format.is_none());
    // View section populated
    assert_eq!(cfg.view.len(), 1);
    assert_eq!(cfg.view["minimal"].format.as_deref(), Some("md"));
}

#[test]
fn config_with_empty_sections_all_none() {
    let cfg =
        TomlConfig::parse("[scan]\n[module]\n[export]\n[analyze]\n[context]\n[badge]\n[gate]")
            .unwrap();
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.module.depth.is_none());
    assert!(cfg.export.format.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.fail_fast.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn config_only_gate_rules_section() {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "solo"
pointer = "/x"
op = "gt"
value = 1
"#,
    )
    .unwrap();
    assert_eq!(cfg.gate.rules.as_ref().unwrap().len(), 1);
    assert_eq!(cfg.gate.rules.as_ref().unwrap()[0].name, "solo");
    // Other gate fields remain None
    assert!(cfg.gate.policy.is_none());
    assert!(cfg.gate.baseline.is_none());
    assert!(cfg.gate.fail_fast.is_none());
}

#[test]
fn empty_file_on_disk_parses_ok() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "").unwrap();
    let cfg = TomlConfig::from_file(f.path()).unwrap();
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.view.is_empty());
}

// =========================================================================
// 10. CliExportFormat and CliTableFormat enum validation
// =========================================================================

#[test]
fn export_format_serde_roundtrip_all_variants() {
    for variant in [
        CliExportFormat::Csv,
        CliExportFormat::Jsonl,
        CliExportFormat::Json,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CliExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn table_format_serde_roundtrip_all_variants() {
    for variant in [
        CliTableFormat::Md,
        CliTableFormat::Tsv,
        CliTableFormat::Json,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CliTableFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn export_format_strings_in_toml_config() {
    for fmt in ["jsonl", "csv", "json", "cyclonedx"] {
        let toml_str = format!("[export]\nformat = \"{fmt}\"");
        let cfg = TomlConfig::parse(&toml_str).unwrap();
        assert_eq!(cfg.export.format.as_deref(), Some(fmt));
    }
}

#[test]
fn analysis_format_serde_roundtrip() {
    for variant in [CliAnalysisFormat::Md, CliAnalysisFormat::Json] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CliAnalysisFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// =========================================================================
// 11. CliChildrenMode / CliChildIncludeMode config parsing
// =========================================================================

#[test]
fn children_mode_collapse_and_separate() {
    for (val, expected) in [
        ("collapse", CliChildrenMode::Collapse),
        ("separate", CliChildrenMode::Separate),
    ] {
        let json = serde_json::to_string(&expected).unwrap();
        let back: CliChildrenMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, expected);

        // Also verify TOML config accepts these strings
        let toml_str = format!("[module]\nchildren = \"{val}\"");
        let cfg = TomlConfig::parse(&toml_str).unwrap();
        assert_eq!(cfg.module.children, Some(val.into()));
    }
}

#[test]
fn child_include_mode_variants() {
    for variant in [
        CliChildIncludeMode::Separate,
        CliChildIncludeMode::ParentsOnly,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CliChildIncludeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn children_mode_in_export_section() {
    let cfg = TomlConfig::parse("[export]\nchildren = \"collapse\"").unwrap();
    assert_eq!(cfg.export.children, Some("collapse".into()));

    let cfg2 = TomlConfig::parse("[export]\nchildren = \"separate\"").unwrap();
    assert_eq!(cfg2.export.children, Some("separate".into()));
}

#[test]
fn children_mode_in_view_profile() {
    let cfg = TomlConfig::parse("[view.test]\nchildren = \"collapse\"").unwrap();
    assert_eq!(cfg.view["test"].children, Some("collapse".into()));
}

// =========================================================================
// 12. Additional coverage: unknown fields, JSON roundtrips
// =========================================================================

#[test]
fn unknown_top_level_keys_silently_ignored() {
    let cfg = TomlConfig::parse("future_field = 42\n[scan]\nhidden = true").unwrap();
    assert_eq!(cfg.scan.hidden, Some(true));
}

#[test]
fn unknown_fields_inside_sections_ignored() {
    let cfg = TomlConfig::parse("[scan]\nhidden = true\nalien_field = \"x\"").unwrap();
    assert_eq!(cfg.scan.hidden, Some(true));
}

#[test]
fn toml_config_json_roundtrip_with_all_sections() {
    let toml_str = r#"
[scan]
hidden = true
exclude = ["a"]

[module]
depth = 3

[export]
format = "csv"
min_code = 2

[analyze]
preset = "health"
git = false

[context]
budget = "64k"

[badge]
metric = "lines"

[gate]
fail_fast = true

[[gate.rules]]
name = "r1"
pointer = "/p"
op = "eq"
value = "hello"

[view.v1]
format = "tsv"
top = 5
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let json = serde_json::to_string_pretty(&cfg).unwrap();
    let back: TomlConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(back.scan.hidden, Some(true));
    assert_eq!(back.module.depth, Some(3));
    assert_eq!(back.export.format, Some("csv".into()));
    assert_eq!(back.analyze.preset, Some("health".into()));
    assert_eq!(back.context.budget, Some("64k".into()));
    assert_eq!(back.badge.metric, Some("lines".into()));
    assert_eq!(back.gate.fail_fast, Some(true));
    assert_eq!(back.gate.rules.as_ref().unwrap()[0].name, "r1");
    assert_eq!(back.view["v1"].top, Some(5));
}

#[test]
fn toml_serialize_then_parse_roundtrip() {
    let cfg = TomlConfig::parse(
        "[scan]\nhidden = true\nexclude = [\"target\"]\n[module]\ndepth = 2\n[view.ci]\nformat = \"tsv\"",
    )
    .unwrap();
    let toml_str = toml::to_string(&cfg).unwrap();
    let back = TomlConfig::parse(&toml_str).unwrap();
    assert_eq!(back.scan.hidden, Some(true));
    assert_eq!(back.scan.exclude, Some(vec!["target".into()]));
    assert_eq!(back.module.depth, Some(2));
    assert_eq!(back.view["ci"].format.as_deref(), Some("tsv"));
}

// =========================================================================
// 13. Gate rule with negate and message fields
// =========================================================================

#[test]
fn gate_rule_negate_and_message() {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "not-java"
pointer = "/lang"
op = "eq"
value = "Java"
negate = true
level = "warn"
message = "Java not allowed"
"#,
    )
    .unwrap();
    let rule = &cfg.gate.rules.unwrap()[0];
    assert!(rule.negate);
    assert_eq!(rule.level, Some("warn".into()));
    assert_eq!(rule.message, Some("Java not allowed".into()));
}

#[test]
fn gate_rule_values_array_for_in_operator() {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "lang-check"
pointer = "/lang"
op = "in"
values = ["Rust", "Go", "Python"]
"#,
    )
    .unwrap();
    let rule = &cfg.gate.rules.unwrap()[0];
    let vals = rule.values.as_ref().unwrap();
    assert_eq!(vals.len(), 3);
    assert_eq!(vals[0].as_str(), Some("Rust"));
}

// =========================================================================
// 14. Multiple ratchet rules
// =========================================================================

#[test]
fn multiple_ratchet_rules() {
    let cfg = TomlConfig::parse(
        r#"
[[gate.ratchet]]
pointer = "/complexity/avg"
max_increase_pct = 5.0

[[gate.ratchet]]
pointer = "/lines/total"
max_value = 100000.0
level = "warn"
"#,
    )
    .unwrap();
    let ratchets = cfg.gate.ratchet.unwrap();
    assert_eq!(ratchets.len(), 2);
    assert_eq!(ratchets[0].pointer, "/complexity/avg");
    assert_eq!(ratchets[0].max_increase_pct, Some(5.0));
    assert!(ratchets[0].max_value.is_none());
    assert_eq!(ratchets[1].pointer, "/lines/total");
    assert_eq!(ratchets[1].max_value, Some(100000.0));
    assert_eq!(ratchets[1].level, Some("warn".into()));
}

// =========================================================================
// 15. Boundary values
// =========================================================================

#[test]
fn zero_values_everywhere() {
    let cfg = TomlConfig::parse(
        r#"
[export]
min_code = 0
max_rows = 0

[analyze]
window = 0
max_files = 0
max_bytes = 0
max_commits = 0

[view.zero]
top = 0
min_code = 0
"#,
    )
    .unwrap();
    assert_eq!(cfg.export.min_code, Some(0));
    assert_eq!(cfg.export.max_rows, Some(0));
    assert_eq!(cfg.analyze.window, Some(0));
    assert_eq!(cfg.analyze.max_files, Some(0));
    assert_eq!(cfg.analyze.max_bytes, Some(0));
    assert_eq!(cfg.analyze.max_commits, Some(0));
    assert_eq!(cfg.view["zero"].top, Some(0));
    assert_eq!(cfg.view["zero"].min_code, Some(0));
}

#[test]
fn very_large_u64_values() {
    let cfg =
        TomlConfig::parse("[analyze]\nmax_bytes = 9999999999999\nmax_file_bytes = 8888888888")
            .unwrap();
    assert_eq!(cfg.analyze.max_bytes, Some(9_999_999_999_999));
    assert_eq!(cfg.analyze.max_file_bytes, Some(8_888_888_888));
}

#[test]
fn empty_arrays_and_strings() {
    let cfg = TomlConfig::parse(
        "[scan]\nexclude = []\npaths = []\n[context]\nbudget = \"\"\n[module]\nroots = []",
    )
    .unwrap();
    assert_eq!(cfg.scan.exclude, Some(vec![]));
    assert_eq!(cfg.scan.paths, Some(vec![]));
    assert_eq!(cfg.context.budget, Some(String::new()));
    assert_eq!(cfg.module.roots, Some(vec![]));
}

// =========================================================================
// 16. View profile all fields set
// =========================================================================

#[test]
fn view_profile_every_field_populated() {
    let cfg = TomlConfig::parse(
        r#"
[view.complete]
format = "json"
top = 25
files = true
module_roots = ["crates", "packages"]
module_depth = 3
min_code = 2
max_rows = 500
redact = "paths"
meta = true
children = "collapse"
preset = "risk"
window = 200000
budget = "256k"
strategy = "spread"
rank_by = "tokens"
output = "bundle"
compress = true
metric = "doc"
"#,
    )
    .unwrap();
    let vp = &cfg.view["complete"];
    assert_eq!(vp.format, Some("json".into()));
    assert_eq!(vp.top, Some(25));
    assert_eq!(vp.files, Some(true));
    assert_eq!(
        vp.module_roots,
        Some(vec!["crates".into(), "packages".into()])
    );
    assert_eq!(vp.module_depth, Some(3));
    assert_eq!(vp.min_code, Some(2));
    assert_eq!(vp.max_rows, Some(500));
    assert_eq!(vp.redact, Some("paths".into()));
    assert_eq!(vp.meta, Some(true));
    assert_eq!(vp.children, Some("collapse".into()));
    assert_eq!(vp.preset, Some("risk".into()));
    assert_eq!(vp.window, Some(200000));
    assert_eq!(vp.budget, Some("256k".into()));
    assert_eq!(vp.strategy, Some("spread".into()));
    assert_eq!(vp.rank_by, Some("tokens".into()));
    assert_eq!(vp.output, Some("bundle".into()));
    assert_eq!(vp.compress, Some(true));
    assert_eq!(vp.metric, Some("doc".into()));
}

// =========================================================================
// 17. TOML comments preserved through parsing (ignored correctly)
// =========================================================================

#[test]
fn inline_and_full_line_comments() {
    let cfg = TomlConfig::parse(
        r#"
# Full-line comment
[scan]
hidden = true  # inline comment
# between keys
exclude = ["target"]
"#,
    )
    .unwrap();
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.exclude, Some(vec!["target".into()]));
}

// =========================================================================
// 18. Unicode paths and profile names
// =========================================================================

#[test]
fn unicode_in_scan_paths_and_excludes() {
    let cfg = TomlConfig::parse(
        r#"
[scan]
paths = ["src/日本語", "lib/données"]
exclude = ["目标/**"]
"#,
    )
    .unwrap();
    assert_eq!(cfg.scan.paths.as_ref().unwrap()[0], "src/日本語");
    assert_eq!(cfg.scan.exclude.as_ref().unwrap()[0], "目标/**");
}

#[test]
fn quoted_unicode_view_profile_name() {
    let cfg = TomlConfig::parse("[view.\"über\"]\nformat = \"md\"").unwrap();
    assert!(cfg.view.contains_key("über"));
    assert_eq!(cfg.view["über"].format.as_deref(), Some("md"));
}

// =========================================================================
// 19. Property test: ViewProfile JSON roundtrip
// =========================================================================

proptest! {
    #[test]
    fn view_profile_json_roundtrip(
        top in proptest::option::of(0usize..1000),
        min_code in proptest::option::of(0usize..500),
        compress in proptest::option::of(any::<bool>()),
    ) {
        let vp = ViewProfile {
            format: Some("json".into()),
            top,
            min_code,
            compress,
            ..Default::default()
        };
        let json = serde_json::to_string(&vp).unwrap();
        let back: ViewProfile = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.top, top);
        prop_assert_eq!(back.min_code, min_code);
        prop_assert_eq!(back.compress, compress);
        prop_assert_eq!(back.format, Some("json".into()));
    }
}

// =========================================================================
// 20. ScanConfig default
// =========================================================================

#[test]
fn scan_config_default_all_none() {
    let sc = ScanConfig::default();
    assert!(sc.paths.is_none());
    assert!(sc.exclude.is_none());
    assert!(sc.hidden.is_none());
    assert!(sc.config.is_none());
    assert!(sc.no_ignore.is_none());
    assert!(sc.no_ignore_parent.is_none());
    assert!(sc.no_ignore_dot.is_none());
    assert!(sc.no_ignore_vcs.is_none());
    assert!(sc.doc_comments.is_none());
}

// =========================================================================
// 21. CliRedactMode variants in config
// =========================================================================

#[test]
fn redact_mode_variants_in_export_and_view() {
    for mode in ["none", "paths", "all"] {
        let toml_str = format!("[export]\nredact = \"{mode}\"");
        let cfg = TomlConfig::parse(&toml_str).unwrap();
        assert_eq!(cfg.export.redact.as_deref(), Some(mode));

        let toml_str2 = format!("[view.r]\nredact = \"{mode}\"");
        let cfg2 = TomlConfig::parse(&toml_str2).unwrap();
        assert_eq!(cfg2.view["r"].redact.as_deref(), Some(mode));
    }
}

#[test]
fn redact_mode_enum_serde() {
    for variant in [
        CliRedactMode::None,
        CliRedactMode::Paths,
        CliRedactMode::All,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CliRedactMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// =========================================================================
// 22. Analyze preset and granularity string variants
// =========================================================================

#[test]
fn all_analyze_presets_accepted_in_toml() {
    for preset in [
        "receipt",
        "health",
        "risk",
        "supply",
        "architecture",
        "topics",
        "security",
        "identity",
        "git",
        "deep",
        "fun",
    ] {
        let toml_str = format!("[analyze]\npreset = \"{preset}\"");
        let cfg = TomlConfig::parse(&toml_str).unwrap();
        assert_eq!(cfg.analyze.preset.as_deref(), Some(preset));
    }
}

#[test]
fn granularity_module_and_file() {
    for g in ["module", "file"] {
        let toml_str = format!("[analyze]\ngranularity = \"{g}\"");
        let cfg = TomlConfig::parse(&toml_str).unwrap();
        assert_eq!(cfg.analyze.granularity.as_deref(), Some(g));
    }
}

// =========================================================================
// 23. Context strategy and output variants
// =========================================================================

#[test]
fn context_strategy_variants() {
    for s in ["greedy", "spread"] {
        let cfg = TomlConfig::parse(&format!("[context]\nstrategy = \"{s}\"")).unwrap();
        assert_eq!(cfg.context.strategy.as_deref(), Some(s));
    }
}

#[test]
fn context_output_variants() {
    for o in ["list", "bundle", "json"] {
        let cfg = TomlConfig::parse(&format!("[context]\noutput = \"{o}\"")).unwrap();
        assert_eq!(cfg.context.output.as_deref(), Some(o));
    }
}

#[test]
fn context_rank_by_variants() {
    for r in ["code", "tokens", "churn", "hotspot"] {
        let cfg = TomlConfig::parse(&format!("[context]\nrank_by = \"{r}\"")).unwrap();
        assert_eq!(cfg.context.rank_by.as_deref(), Some(r));
    }
}

// =========================================================================
// 24. Multiple view profiles are independent (no cross-contamination)
// =========================================================================

#[test]
fn five_profiles_no_cross_contamination() {
    let cfg = TomlConfig::parse(
        r#"
[view.a]
format = "json"
top = 1

[view.b]
format = "md"
compress = true

[view.c]
min_code = 10

[view.d]
redact = "all"
strategy = "spread"

[view.e]
budget = "1m"
"#,
    )
    .unwrap();
    assert_eq!(cfg.view.len(), 5);
    // a has format + top, nothing else
    assert_eq!(cfg.view["a"].format, Some("json".into()));
    assert_eq!(cfg.view["a"].top, Some(1));
    assert!(cfg.view["a"].compress.is_none());
    assert!(cfg.view["a"].min_code.is_none());
    // b has format + compress, nothing else
    assert_eq!(cfg.view["b"].compress, Some(true));
    assert!(cfg.view["b"].top.is_none());
    // c has only min_code
    assert_eq!(cfg.view["c"].min_code, Some(10));
    assert!(cfg.view["c"].format.is_none());
    // d has redact + strategy
    assert_eq!(cfg.view["d"].redact, Some("all".into()));
    assert_eq!(cfg.view["d"].strategy, Some("spread".into()));
    assert!(cfg.view["d"].budget.is_none());
    // e has only budget
    assert_eq!(cfg.view["e"].budget, Some("1m".into()));
    assert!(cfg.view["e"].redact.is_none());
}

// =========================================================================
// 25. Property test: TomlConfig JSON roundtrip
// =========================================================================

proptest! {
    #[test]
    fn toml_config_json_roundtrip_prop(
        hidden in proptest::option::of(any::<bool>()),
        depth in proptest::option::of(1usize..100),
        min_code in proptest::option::of(0usize..10000),
        fail_fast in proptest::option::of(any::<bool>()),
    ) {
        let mut cfg = TomlConfig::default();
        cfg.scan.hidden = hidden;
        cfg.module.depth = depth;
        cfg.export.min_code = min_code;
        cfg.gate.fail_fast = fail_fast;

        let json = serde_json::to_string(&cfg).unwrap();
        let back: TomlConfig = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.scan.hidden, hidden);
        prop_assert_eq!(back.module.depth, depth);
        prop_assert_eq!(back.export.min_code, min_code);
        prop_assert_eq!(back.gate.fail_fast, fail_fast);
    }
}

// =========================================================================
// 26. Profile with children set
// =========================================================================

#[test]
fn profile_children_field_accepted() {
    let p = Profile {
        children: Some("collapse".into()),
        ..Default::default()
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: Profile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.children, Some("collapse".into()));
}

// =========================================================================
// 27. CliConfigMode enum
// =========================================================================

#[test]
fn config_mode_variants_serde() {
    for variant in [CliConfigMode::Auto, CliConfigMode::None] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CliConfigMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn config_mode_in_scan_section() {
    let cfg = TomlConfig::parse("[scan]\nconfig = \"none\"").unwrap();
    assert_eq!(cfg.scan.config, Some("none".into()));

    let cfg2 = TomlConfig::parse("[scan]\nconfig = \"auto\"").unwrap();
    assert_eq!(cfg2.scan.config, Some("auto".into()));
}
