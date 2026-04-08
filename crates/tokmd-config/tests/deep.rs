//! Deep tests for tokmd-config TOML loading, serialization, and edge cases.
//!
//! Covers: defaults, parsing, validation, roundtrips, boundary values,
//! Unicode handling, TOML syntax variants, gate rule types, view profile
//! completeness, and config interaction patterns.

use std::io::Write;
use tempfile::NamedTempFile;
use tokmd_config::{
    AnalysisPreset, BadgeMetric, CliExportFormat, CliLangArgs, CockpitFormat, ColorMode,
    ContextOutput, ContextStrategy, DiffFormat, DiffRangeMode, GateFormat, GlobalArgs,
    HandoffPreset, ImportGranularity, InitProfile, NearDupScope, Profile, SensorFormat, Shell,
    TomlConfig, UserConfig, ValueMetric, ViewProfile,
};

// =========================================================================
// 1. Parse empty config yields defaults for every section
// =========================================================================

#[test]
fn empty_string_produces_default_toml_config() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("")?;
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.scan.exclude.is_none());
    assert!(cfg.scan.hidden.is_none());
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
    assert!(cfg.analyze.max_files.is_none());
    assert!(cfg.analyze.max_bytes.is_none());
    assert!(cfg.analyze.max_file_bytes.is_none());
    assert!(cfg.analyze.max_commits.is_none());
    assert!(cfg.analyze.max_commit_files.is_none());
    assert!(cfg.analyze.granularity.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.context.strategy.is_none());
    assert!(cfg.context.rank_by.is_none());
    assert!(cfg.context.output.is_none());
    assert!(cfg.context.compress.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.policy.is_none());
    assert!(cfg.gate.baseline.is_none());
    assert!(cfg.gate.preset.is_none());
    assert!(cfg.gate.fail_fast.is_none());
    assert!(cfg.gate.rules.is_none());
    assert!(cfg.gate.ratchet.is_none());
    assert!(cfg.gate.allow_missing_baseline.is_none());
    assert!(cfg.gate.allow_missing_current.is_none());
    assert!(cfg.view.is_empty());
    Ok(())
}

// =========================================================================
// 2. Minimal valid config: single field set
// =========================================================================

#[test]
fn minimal_config_single_scan_field() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[scan]\nhidden = true")?;
    assert_eq!(cfg.scan.hidden, Some(true));
    // Everything else is None/empty
    assert!(cfg.module.depth.is_none());
    assert!(cfg.view.is_empty());
    Ok(())
}

#[test]
fn minimal_config_single_view_profile() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[view.x]\nformat = \"md\"")?;
    assert_eq!(cfg.view.len(), 1);
    assert_eq!(cfg.view["x"].format.as_deref(), Some("md"));
    Ok(())
}

// =========================================================================
// 3. Config with ALL fields populated
// =========================================================================

#[test]
fn all_fields_populated_config() -> Result<(), Box<dyn std::error::Error>> {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target", "dist"]
hidden = true
config = "none"
no_ignore = true
no_ignore_parent = true
no_ignore_dot = true
no_ignore_vcs = true
doc_comments = true

[module]
roots = ["crates", "packages", "libs"]
depth = 3
children = "collapse"

[export]
min_code = 5
max_rows = 2000
redact = "all"
format = "csv"
children = "separate"

[analyze]
preset = "deep"
window = 256000
format = "json"
git = true
max_files = 5000
max_bytes = 50000000
max_file_bytes = 200000
max_commits = 1000
max_commit_files = 100
granularity = "file"

[context]
budget = "1m"
strategy = "spread"
rank_by = "hotspot"
output = "bundle"
compress = true

[badge]
metric = "doc"

[gate]
policy = "policy.toml"
baseline = "baseline.json"
preset = "risk"
fail_fast = true
allow_missing_baseline = false
allow_missing_current = true

[[gate.rules]]
name = "rule1"
pointer = "/a"
op = "gt"
value = 42

[[gate.ratchet]]
pointer = "/b"
max_increase_pct = 10.0
max_value = 500.0
level = "warn"
description = "keep it low"

[view.full]
format = "json"
top = 20
files = true
module_roots = ["crates"]
module_depth = 2
min_code = 1
max_rows = 100
redact = "paths"
meta = true
children = "collapse"
preset = "health"
window = 128000
budget = "64k"
strategy = "greedy"
rank_by = "code"
output = "list"
compress = false
metric = "lines"
"#;
    let cfg = TomlConfig::parse(toml_str)?;

    // Scan
    assert_eq!(cfg.scan.paths, Some(vec!["src".into(), "lib".into()]));
    assert_eq!(cfg.scan.exclude, Some(vec!["target".into(), "dist".into()]));
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.config, Some("none".into()));
    assert_eq!(cfg.scan.no_ignore, Some(true));
    assert_eq!(cfg.scan.no_ignore_parent, Some(true));
    assert_eq!(cfg.scan.no_ignore_dot, Some(true));
    assert_eq!(cfg.scan.no_ignore_vcs, Some(true));
    assert_eq!(cfg.scan.doc_comments, Some(true));

    // Module
    assert_eq!(cfg.module.roots.as_ref().ok_or("missing")?.len(), 3);
    assert_eq!(cfg.module.depth, Some(3));
    assert_eq!(cfg.module.children, Some("collapse".into()));

    // Export
    assert_eq!(cfg.export.min_code, Some(5));
    assert_eq!(cfg.export.max_rows, Some(2000));
    assert_eq!(cfg.export.redact, Some("all".into()));
    assert_eq!(cfg.export.format, Some("csv".into()));
    assert_eq!(cfg.export.children, Some("separate".into()));

    // Analyze
    assert_eq!(cfg.analyze.preset, Some("deep".into()));
    assert_eq!(cfg.analyze.window, Some(256000));
    assert_eq!(cfg.analyze.format, Some("json".into()));
    assert_eq!(cfg.analyze.git, Some(true));
    assert_eq!(cfg.analyze.max_files, Some(5000));
    assert_eq!(cfg.analyze.max_bytes, Some(50_000_000));
    assert_eq!(cfg.analyze.max_file_bytes, Some(200_000));
    assert_eq!(cfg.analyze.max_commits, Some(1000));
    assert_eq!(cfg.analyze.max_commit_files, Some(100));
    assert_eq!(cfg.analyze.granularity, Some("file".into()));

    // Context
    assert_eq!(cfg.context.budget, Some("1m".into()));
    assert_eq!(cfg.context.strategy, Some("spread".into()));
    assert_eq!(cfg.context.rank_by, Some("hotspot".into()));
    assert_eq!(cfg.context.output, Some("bundle".into()));
    assert_eq!(cfg.context.compress, Some(true));

    // Badge
    assert_eq!(cfg.badge.metric, Some("doc".into()));

    // Gate
    assert_eq!(cfg.gate.policy, Some("policy.toml".into()));
    assert_eq!(cfg.gate.baseline, Some("baseline.json".into()));
    assert_eq!(cfg.gate.preset, Some("risk".into()));
    assert_eq!(cfg.gate.fail_fast, Some(true));
    assert_eq!(cfg.gate.allow_missing_baseline, Some(false));
    assert_eq!(cfg.gate.allow_missing_current, Some(true));
    assert_eq!(cfg.gate.rules.as_ref().ok_or("missing")?.len(), 1);
    assert_eq!(cfg.gate.ratchet.as_ref().ok_or("missing")?.len(), 1);

    // View profile
    let vp = &cfg.view["full"];
    assert_eq!(vp.format, Some("json".into()));
    assert_eq!(vp.top, Some(20));
    assert_eq!(vp.files, Some(true));
    assert_eq!(vp.module_roots.as_ref().ok_or("missing")?, &["crates"]);
    assert_eq!(vp.module_depth, Some(2));
    assert_eq!(vp.min_code, Some(1));
    assert_eq!(vp.max_rows, Some(100));
    assert_eq!(vp.redact, Some("paths".into()));
    assert_eq!(vp.meta, Some(true));
    assert_eq!(vp.children, Some("collapse".into()));
    assert_eq!(vp.preset, Some("health".into()));
    assert_eq!(vp.window, Some(128000));
    assert_eq!(vp.budget, Some("64k".into()));
    assert_eq!(vp.strategy, Some("greedy".into()));
    assert_eq!(vp.rank_by, Some("code".into()));
    assert_eq!(vp.output, Some("list".into()));
    assert_eq!(vp.compress, Some(false));
    assert_eq!(vp.metric, Some("lines".into()));
    Ok(())
}

// =========================================================================
// 4. Missing file → error (not default config)
// =========================================================================

#[test]
fn missing_file_returns_io_error() -> Result<(), Box<dyn std::error::Error>> {
    let result = TomlConfig::from_file(std::path::Path::new("/nonexistent/tokmd.toml"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    Ok(())
}

// =========================================================================
// 5. Invalid TOML → error
// =========================================================================

#[test]
fn invalid_toml_missing_bracket() -> Result<(), Box<dyn std::error::Error>> {
    assert!(TomlConfig::parse("[scan\nhidden = true").is_err());
    Ok(())
}

#[test]
fn invalid_toml_duplicate_keys() -> Result<(), Box<dyn std::error::Error>> {
    // TOML spec: duplicate keys in the same table are errors
    let result = TomlConfig::parse("[scan]\nhidden = true\nhidden = false");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn invalid_toml_wrong_type_bool_for_usize() -> Result<(), Box<dyn std::error::Error>> {
    assert!(TomlConfig::parse("[module]\ndepth = true").is_err());
    Ok(())
}

#[test]
fn invalid_toml_wrong_type_string_for_bool() -> Result<(), Box<dyn std::error::Error>> {
    assert!(TomlConfig::parse("[scan]\nhidden = \"yes\"").is_err());
    Ok(())
}

#[test]
fn invalid_toml_negative_unsigned() -> Result<(), Box<dyn std::error::Error>> {
    assert!(TomlConfig::parse("[export]\nmin_code = -1").is_err());
    Ok(())
}

#[test]
fn invalid_toml_float_for_usize() -> Result<(), Box<dyn std::error::Error>> {
    assert!(TomlConfig::parse("[module]\ndepth = 2.5").is_err());
    Ok(())
}

#[test]
fn invalid_toml_array_for_string() -> Result<(), Box<dyn std::error::Error>> {
    assert!(TomlConfig::parse("[context]\nbudget = [\"128k\"]").is_err());
    Ok(())
}

#[test]
fn invalid_toml_string_for_array() -> Result<(), Box<dyn std::error::Error>> {
    assert!(TomlConfig::parse("[module]\nroots = \"crates\"").is_err());
    Ok(())
}

// =========================================================================
// 6. Unknown fields → silently ignored (serde default behavior)
// =========================================================================

#[test]
fn unknown_top_level_key_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("mystery_key = 42\n[scan]\nhidden = true")?;
    assert_eq!(cfg.scan.hidden, Some(true));
    Ok(())
}

#[test]
fn unknown_field_inside_section_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[scan]\nhidden = true\nfuture_flag = 99")?;
    assert_eq!(cfg.scan.hidden, Some(true));
    Ok(())
}

#[test]
fn unknown_section_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[unknown_section]\nfoo = \"bar\"\n[scan]\nhidden = false")?;
    assert_eq!(cfg.scan.hidden, Some(false));
    Ok(())
}

#[test]
fn unknown_field_in_view_profile_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[view.x]\nformat = \"md\"\nalien = true")?;
    assert_eq!(cfg.view["x"].format.as_deref(), Some("md"));
    Ok(())
}

// =========================================================================
// 7. JSON serialization roundtrip for TomlConfig
// =========================================================================

#[test]
fn toml_config_json_roundtrip_empty() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("")?;
    let json = serde_json::to_string(&cfg)?;
    let back: TomlConfig = serde_json::from_str(&json)?;
    assert!(back.scan.hidden.is_none());
    assert!(back.view.is_empty());
    Ok(())
}

#[test]
fn toml_config_json_roundtrip_populated() -> Result<(), Box<dyn std::error::Error>> {
    let toml_str = r#"
[scan]
hidden = true
exclude = ["a", "b"]

[module]
depth = 5
roots = ["x"]

[export]
format = "csv"
min_code = 3

[analyze]
preset = "risk"
git = false
max_bytes = 999

[context]
budget = "512k"
compress = true

[badge]
metric = "bytes"

[gate]
fail_fast = true
allow_missing_baseline = true

[[gate.rules]]
name = "r1"
pointer = "/p"
op = "eq"
value = "hello"

[[gate.ratchet]]
pointer = "/q"
max_increase_pct = 2.5

[view.v1]
format = "tsv"
top = 7
"#;
    let cfg = TomlConfig::parse(toml_str)?;
    let json = serde_json::to_string_pretty(&cfg)?;
    let back: TomlConfig = serde_json::from_str(&json)?;

    assert_eq!(back.scan.hidden, Some(true));
    assert_eq!(back.scan.exclude, Some(vec!["a".into(), "b".into()]));
    assert_eq!(back.module.depth, Some(5));
    assert_eq!(back.export.format, Some("csv".into()));
    assert_eq!(back.analyze.preset, Some("risk".into()));
    assert_eq!(back.analyze.git, Some(false));
    assert_eq!(back.analyze.max_bytes, Some(999));
    assert_eq!(back.context.budget, Some("512k".into()));
    assert_eq!(back.context.compress, Some(true));
    assert_eq!(back.badge.metric, Some("bytes".into()));
    assert_eq!(back.gate.fail_fast, Some(true));
    assert_eq!(back.gate.allow_missing_baseline, Some(true));
    assert_eq!(back.gate.rules.as_ref().ok_or("missing")?.len(), 1);
    assert_eq!(back.gate.rules.as_ref().ok_or("missing")?[0].name, "r1");
    assert_eq!(back.gate.ratchet.as_ref().ok_or("missing")?.len(), 1);
    assert_eq!(back.view["v1"].top, Some(7));
    Ok(())
}

// =========================================================================
// 8. TOML → JSON → TOML: toml serialization roundtrip
// =========================================================================

#[test]
fn toml_serialize_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[scan]
hidden = true
exclude = ["target"]

[module]
depth = 2

[view.ci]
format = "tsv"
"#,
    )?;

    let toml_str = toml::to_string(&cfg)?;
    let back = TomlConfig::parse(&toml_str)?;
    assert_eq!(back.scan.hidden, Some(true));
    assert_eq!(back.scan.exclude, Some(vec!["target".into()]));
    assert_eq!(back.module.depth, Some(2));
    assert_eq!(back.view["ci"].format.as_deref(), Some("tsv"));
    Ok(())
}

// =========================================================================
// 9. Output format option strings
// =========================================================================

#[test]
fn export_format_variants_accepted() -> Result<(), Box<dyn std::error::Error>> {
    for fmt in ["jsonl", "csv", "json", "cyclonedx"] {
        let toml_str = format!("[export]\nformat = \"{}\"", fmt);
        let cfg = TomlConfig::parse(&toml_str)?;
        assert_eq!(cfg.export.format.as_deref(), Some(fmt));
    }
    Ok(())
}

#[test]
fn analyze_format_variants_accepted() -> Result<(), Box<dyn std::error::Error>> {
    for fmt in ["json", "md"] {
        let toml_str = format!("[analyze]\nformat = \"{}\"", fmt);
        let cfg = TomlConfig::parse(&toml_str)?;
        assert_eq!(cfg.analyze.format.as_deref(), Some(fmt));
    }
    Ok(())
}

#[test]
fn context_strategy_variants_accepted() -> Result<(), Box<dyn std::error::Error>> {
    for s in ["greedy", "spread"] {
        let toml_str = format!("[context]\nstrategy = \"{}\"", s);
        let cfg = TomlConfig::parse(&toml_str)?;
        assert_eq!(cfg.context.strategy.as_deref(), Some(s));
    }
    Ok(())
}

#[test]
fn context_rank_by_variants_accepted() -> Result<(), Box<dyn std::error::Error>> {
    for r in ["code", "tokens", "churn", "hotspot"] {
        let toml_str = format!("[context]\nrank_by = \"{}\"", r);
        let cfg = TomlConfig::parse(&toml_str)?;
        assert_eq!(cfg.context.rank_by.as_deref(), Some(r));
    }
    Ok(())
}

#[test]
fn context_output_variants_accepted() -> Result<(), Box<dyn std::error::Error>> {
    for o in ["list", "bundle", "json"] {
        let toml_str = format!("[context]\noutput = \"{}\"", o);
        let cfg = TomlConfig::parse(&toml_str)?;
        assert_eq!(cfg.context.output.as_deref(), Some(o));
    }
    Ok(())
}

#[test]
fn analyze_preset_variants_accepted() -> Result<(), Box<dyn std::error::Error>> {
    for p in [
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
        let toml_str = format!("[analyze]\npreset = \"{}\"", p);
        let cfg = TomlConfig::parse(&toml_str)?;
        assert_eq!(cfg.analyze.preset.as_deref(), Some(p));
    }
    Ok(())
}

#[test]
fn redact_mode_variants_accepted() -> Result<(), Box<dyn std::error::Error>> {
    for r in ["none", "paths", "all"] {
        let toml_str = format!("[export]\nredact = \"{}\"", r);
        let cfg = TomlConfig::parse(&toml_str)?;
        assert_eq!(cfg.export.redact.as_deref(), Some(r));
    }
    Ok(())
}

#[test]
fn children_mode_variants_accepted() -> Result<(), Box<dyn std::error::Error>> {
    for c in ["collapse", "separate"] {
        let toml_str = format!("[module]\nchildren = \"{}\"", c);
        let cfg = TomlConfig::parse(&toml_str)?;
        assert_eq!(cfg.module.children.as_deref(), Some(c));
    }
    Ok(())
}

#[test]
fn granularity_variants_accepted() -> Result<(), Box<dyn std::error::Error>> {
    for g in ["module", "file"] {
        let toml_str = format!("[analyze]\ngranularity = \"{}\"", g);
        let cfg = TomlConfig::parse(&toml_str)?;
        assert_eq!(cfg.analyze.granularity.as_deref(), Some(g));
    }
    Ok(())
}

// =========================================================================
// 10. Path config with forward slashes
// =========================================================================

#[test]
fn paths_with_forward_slashes() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[scan]
paths = ["src/main", "crates/tokmd-config/src"]
exclude = ["**/target/**", "node_modules/"]
"#,
    )?;
    assert_eq!(
        cfg.scan.paths,
        Some(vec!["src/main".into(), "crates/tokmd-config/src".into()])
    );
    assert!(cfg.scan.exclude.as_ref().ok_or("missing")?[0].contains('/'));
    Ok(())
}

#[test]
fn module_roots_with_slashes() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[module]\nroots = [\"src/crates\", \"lib/packages\"]")?;
    assert_eq!(
        cfg.module.roots,
        Some(vec!["src/crates".into(), "lib/packages".into()])
    );
    Ok(())
}

// =========================================================================
// 11. Gate rule value types (numbers, strings, bools, null, arrays)
// =========================================================================

#[test]
fn gate_rule_value_integer() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "int-rule"
pointer = "/x"
op = "gt"
value = 42
"#,
    )?;
    let rules = cfg.gate.rules.ok_or("missing rules")?;
    let v = rules[0].value.as_ref().ok_or("missing")?;
    assert_eq!(v.as_i64(), Some(42));
    Ok(())
}

#[test]
fn gate_rule_value_float() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "float-rule"
pointer = "/x"
op = "gt"
value = 1.23
"#,
    )?;
    let rules = cfg.gate.rules.ok_or("missing rules")?;
    let v = rules[0].value.as_ref().ok_or("missing")?;
    assert!((v.as_f64().ok_or("missing")? - 1.23).abs() < 1e-9);
    Ok(())
}

#[test]
fn gate_rule_value_string() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "str-rule"
pointer = "/lang"
op = "eq"
value = "Rust"
"#,
    )?;
    let rules = cfg.gate.rules.ok_or("missing rules")?;
    let v = rules[0].value.as_ref().ok_or("missing")?;
    assert_eq!(v.as_str(), Some("Rust"));
    Ok(())
}

#[test]
fn gate_rule_value_bool() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "bool-rule"
pointer = "/flag"
op = "eq"
value = true
"#,
    )?;
    let rules = cfg.gate.rules.ok_or("missing rules")?;
    let v = rules[0].value.as_ref().ok_or("missing")?;
    assert_eq!(v.as_bool(), Some(true));
    Ok(())
}

#[test]
fn gate_rule_values_array_of_strings() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "in-rule"
pointer = "/lang"
op = "in"
values = ["Rust", "Go", "Python"]
"#,
    )?;
    let rules = cfg.gate.rules.ok_or("missing rules")?;
    let vals = rules[0].values.as_ref().ok_or("missing")?;
    assert_eq!(vals.len(), 3);
    assert_eq!(vals[0].as_str(), Some("Rust"));
    assert_eq!(vals[2].as_str(), Some("Python"));
    Ok(())
}

#[test]
fn gate_rule_no_value_no_values() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "bare"
pointer = "/x"
op = "exists"
"#,
    )?;
    let rule = &cfg.gate.rules.ok_or("missing rules")?[0];
    assert!(rule.value.is_none());
    assert!(rule.values.is_none());
    Ok(())
}

#[test]
fn gate_rule_negate_default_false() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "r"
pointer = "/x"
op = "eq"
value = 1
"#,
    )?;
    assert!(!cfg.gate.rules.ok_or("missing rules")?[0].negate);
    Ok(())
}

// =========================================================================
// 12. Preset selection in analyze config
// =========================================================================

#[test]
fn analyze_preset_none_when_omitted() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[analyze]\nwindow = 100")?;
    assert!(cfg.analyze.preset.is_none());
    Ok(())
}

#[test]
fn analyze_preset_set_when_provided() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[analyze]\npreset = \"security\"")?;
    assert_eq!(cfg.analyze.preset, Some("security".into()));
    Ok(())
}

// =========================================================================
// 13. View profile field completeness
// =========================================================================

#[test]
fn view_profile_default_all_none() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

// =========================================================================
// 14. Multiple view profiles are independent
// =========================================================================

#[test]
fn three_profiles_independent() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[view.a]
format = "json"
top = 1

[view.b]
format = "md"
top = 99

[view.c]
compress = true
"#,
    )?;
    assert_eq!(cfg.view.len(), 3);
    assert_eq!(cfg.view["a"].top, Some(1));
    assert_eq!(cfg.view["b"].top, Some(99));
    assert!(cfg.view["c"].top.is_none());
    assert_eq!(cfg.view["c"].compress, Some(true));
    Ok(())
}

// =========================================================================
// 15. BTreeMap ordering of view profiles
// =========================================================================

#[test]
fn view_profiles_sorted_by_name() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[view.zeta]
format = "md"
[view.alpha]
format = "json"
[view.mid]
format = "tsv"
"#,
    )?;
    let keys: Vec<&String> = cfg.view.keys().collect();
    assert_eq!(keys, vec!["alpha", "mid", "zeta"]);
    Ok(())
}

// =========================================================================
// 16. Boundary values
// =========================================================================

#[test]
fn zero_values_accepted() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[export]
min_code = 0
max_rows = 0

[analyze]
window = 0
max_files = 0
"#,
    )?;
    assert_eq!(cfg.export.min_code, Some(0));
    assert_eq!(cfg.export.max_rows, Some(0));
    assert_eq!(cfg.analyze.window, Some(0));
    assert_eq!(cfg.analyze.max_files, Some(0));
    Ok(())
}

#[test]
fn large_values_accepted() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[analyze]
max_bytes = 9999999999999
window = 1000000
"#,
    )?;
    assert_eq!(cfg.analyze.max_bytes, Some(9_999_999_999_999));
    assert_eq!(cfg.analyze.window, Some(1_000_000));
    Ok(())
}

#[test]
fn empty_arrays_accepted() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[scan]\nexclude = []\npaths = []")?;
    assert_eq!(cfg.scan.exclude, Some(vec![]));
    assert_eq!(cfg.scan.paths, Some(vec![]));
    Ok(())
}

#[test]
fn empty_string_values_accepted() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[context]\nbudget = \"\"")?;
    assert_eq!(cfg.context.budget, Some(String::new()));
    Ok(())
}

// =========================================================================
// 17. TOML comments are ignored
// =========================================================================

#[test]
fn toml_comments_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
# This is a comment
[scan]
hidden = true  # inline comment
# another comment
exclude = ["target"]
"#,
    )?;
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.scan.exclude, Some(vec!["target".into()]));
    Ok(())
}

// =========================================================================
// 18. Unicode in string values
// =========================================================================

#[test]
fn unicode_in_paths() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[scan]
paths = ["src/日本語", "lib/données"]
exclude = ["目标/**"]
"#,
    )?;
    assert_eq!(cfg.scan.paths.as_ref().ok_or("missing")?[0], "src/日本語");
    assert_eq!(cfg.scan.paths.as_ref().ok_or("missing")?[1], "lib/données");
    assert_eq!(cfg.scan.exclude.as_ref().ok_or("missing")?[0], "目标/**");
    Ok(())
}

#[test]
fn unicode_in_view_profile_name() -> Result<(), Box<dyn std::error::Error>> {
    // TOML requires quoting keys with non-ASCII chars
    let cfg = TomlConfig::parse("[view.\"über\"]\nformat = \"md\"")?;
    assert!(cfg.view.contains_key("über"));
    Ok(())
}

// =========================================================================
// 19. from_file with valid tempfile
// =========================================================================

#[test]
fn from_file_valid_content() -> Result<(), Box<dyn std::error::Error>> {
    let content = b"[scan]\nhidden = true\n[module]\ndepth = 4";
    let mut f = NamedTempFile::new()?;
    f.write_all(content)?;

    let cfg = TomlConfig::from_file(f.path())?;
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.module.depth, Some(4));
    Ok(())
}

#[test]
fn from_file_invalid_content_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut f = NamedTempFile::new()?;
    f.write_all(b"[broken\nno = good")?;

    let result = TomlConfig::from_file(f.path());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
    Ok(())
}

#[test]
fn from_file_empty_file_yields_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let f = NamedTempFile::new()?;
    let cfg = TomlConfig::from_file(f.path())?;
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.view.is_empty());
    Ok(())
}

// =========================================================================
// 20. Ratchet rule edge cases
// =========================================================================

#[test]
fn ratchet_rule_minimal_only_pointer() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[[gate.ratchet]]
pointer = "/metric"
"#,
    )?;
    let r = &cfg.gate.ratchet.ok_or("missing ratchet")?[0];
    assert_eq!(r.pointer, "/metric");
    assert!(r.max_increase_pct.is_none());
    assert!(r.max_value.is_none());
    assert!(r.level.is_none());
    assert!(r.description.is_none());
    Ok(())
}

#[test]
fn ratchet_rule_all_fields() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[[gate.ratchet]]
pointer = "/complexity/max"
max_increase_pct = 10.5
max_value = 200.0
level = "error"
description = "Complexity ceiling"
"#,
    )?;
    let r = &cfg.gate.ratchet.ok_or("missing ratchet")?[0];
    assert_eq!(r.pointer, "/complexity/max");
    assert!((r.max_increase_pct.ok_or("missing")? - 10.5).abs() < 1e-9);
    assert!((r.max_value.ok_or("missing")? - 200.0).abs() < 1e-9);
    assert_eq!(r.level, Some("error".into()));
    assert_eq!(r.description, Some("Complexity ceiling".into()));
    Ok(())
}

// =========================================================================
// 21. Multiple gate rules
// =========================================================================

#[test]
fn many_gate_rules_preserved_in_order() -> Result<(), Box<dyn std::error::Error>> {
    let mut toml_str = String::from("[gate]\n");
    for i in 0..10 {
        toml_str.push_str(&format!(
            "\n[[gate.rules]]\nname = \"rule-{}\"\npointer = \"/p{}\"\nop = \"eq\"\nvalue = {}\n",
            i, i, i
        ));
    }
    let cfg = TomlConfig::parse(&toml_str)?;
    let rules = cfg.gate.rules.ok_or("missing rules")?;
    assert_eq!(rules.len(), 10);
    for (i, rule) in rules.iter().enumerate() {
        assert_eq!(rule.name, format!("rule-{}", i));
    }
    Ok(())
}

// =========================================================================
// 22. UserConfig serde
// =========================================================================

#[test]
fn user_config_default_empty() -> Result<(), Box<dyn std::error::Error>> {
    let uc = UserConfig::default();
    assert!(uc.profiles.is_empty());
    assert!(uc.repos.is_empty());
    Ok(())
}

#[test]
fn user_config_json_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let mut uc = UserConfig::default();
    uc.profiles.insert(
        "ci".into(),
        Profile {
            format: Some("tsv".into()),
            top: Some(5),
            files: Some(true),
            module_roots: Some(vec!["crates".into()]),
            module_depth: Some(2),
            min_code: Some(1),
            max_rows: Some(50),
            redact: Some(tokmd_config::CliRedactMode::Paths),
            meta: Some(false),
            children: Some("collapse".into()),
        },
    );
    uc.repos.insert("org/repo".into(), "ci".into());

    let json = serde_json::to_string(&uc)?;
    let back: UserConfig = serde_json::from_str(&json)?;
    assert_eq!(back.profiles.len(), 1);
    assert_eq!(back.repos.len(), 1);
    let p = &back.profiles["ci"];
    assert_eq!(p.format, Some("tsv".into()));
    assert_eq!(p.top, Some(5));
    assert_eq!(p.files, Some(true));
    assert_eq!(p.module_roots.as_ref().ok_or("missing")?, &["crates"]);
    assert_eq!(p.module_depth, Some(2));
    assert_eq!(p.min_code, Some(1));
    assert_eq!(p.max_rows, Some(50));
    assert_eq!(p.redact, Some(tokmd_config::CliRedactMode::Paths));
    assert_eq!(p.meta, Some(false));
    assert_eq!(p.children, Some("collapse".into()));
    assert_eq!(back.repos["org/repo"], "ci");
    Ok(())
}

// =========================================================================
// 23. GlobalArgs default values
// =========================================================================

#[test]
fn global_args_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let g = GlobalArgs::default();
    assert!(g.excluded.is_empty());
    assert_eq!(g.config, tokmd_config::CliConfigMode::Auto);
    assert!(!g.hidden);
    assert!(!g.no_ignore);
    assert!(!g.no_ignore_parent);
    assert!(!g.no_ignore_dot);
    assert!(!g.no_ignore_vcs);
    assert!(!g.treat_doc_strings_as_comments);
    assert_eq!(g.verbose, 0);
    assert!(!g.no_progress);
    Ok(())
}

// =========================================================================
// 24. GlobalArgs → ScanOptions conversion preserves all fields
// =========================================================================

#[test]
fn global_args_to_scan_options_all_fields() -> Result<(), Box<dyn std::error::Error>> {
    let g = GlobalArgs {
        excluded: vec!["a".into(), "b".into()],
        config: tokmd_config::CliConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
        verbose: 3,
        no_progress: true,
    };
    let opts: tokmd_settings::ScanOptions = (&g).into();
    assert_eq!(opts.excluded, vec!["a", "b"]);
    assert_eq!(opts.config, tokmd_types::ConfigMode::None);
    assert!(opts.hidden);
    assert!(opts.no_ignore);
    assert!(opts.no_ignore_parent);
    assert!(opts.no_ignore_dot);
    assert!(opts.no_ignore_vcs);
    assert!(opts.treat_doc_strings_as_comments);
    Ok(())
}

// =========================================================================
// 25. CliLangArgs default
// =========================================================================

#[test]
fn cli_lang_args_default_all_none() -> Result<(), Box<dyn std::error::Error>> {
    let a = CliLangArgs::default();
    assert!(a.paths.is_none());
    assert!(a.format.is_none());
    assert!(a.top.is_none());
    assert!(!a.files);
    assert!(a.children.is_none());
    Ok(())
}

// =========================================================================
// 26. Enum default values
// =========================================================================

#[test]
fn enum_defaults() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(DiffFormat::default(), DiffFormat::Md);
    assert_eq!(ColorMode::default(), ColorMode::Auto);
    assert_eq!(ContextStrategy::default(), ContextStrategy::Greedy);
    assert_eq!(ValueMetric::default(), ValueMetric::Code);
    assert_eq!(ContextOutput::default(), ContextOutput::List);
    assert_eq!(GateFormat::default(), GateFormat::Text);
    assert_eq!(CockpitFormat::default(), CockpitFormat::Json);
    assert_eq!(HandoffPreset::default(), HandoffPreset::Risk);
    assert_eq!(SensorFormat::default(), SensorFormat::Json);
    assert_eq!(NearDupScope::default(), NearDupScope::Module);
    assert_eq!(DiffRangeMode::default(), DiffRangeMode::TwoDot);
    Ok(())
}

// =========================================================================
// 27. Enum serde kebab-case naming
// =========================================================================

#[test]
fn analysis_preset_kebab_names() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        serde_json::to_string(&AnalysisPreset::Receipt)?,
        "\"receipt\""
    );
    assert_eq!(
        serde_json::to_string(&AnalysisPreset::Health)?,
        "\"health\""
    );
    assert_eq!(serde_json::to_string(&AnalysisPreset::Risk)?, "\"risk\"");
    assert_eq!(
        serde_json::to_string(&AnalysisPreset::Supply)?,
        "\"supply\""
    );
    assert_eq!(
        serde_json::to_string(&AnalysisPreset::Architecture)?,
        "\"architecture\""
    );
    assert_eq!(
        serde_json::to_string(&AnalysisPreset::Topics)?,
        "\"topics\""
    );
    assert_eq!(
        serde_json::to_string(&AnalysisPreset::Security)?,
        "\"security\""
    );
    assert_eq!(
        serde_json::to_string(&AnalysisPreset::Identity)?,
        "\"identity\""
    );
    assert_eq!(serde_json::to_string(&AnalysisPreset::Git)?, "\"git\"");
    assert_eq!(serde_json::to_string(&AnalysisPreset::Deep)?, "\"deep\"");
    assert_eq!(serde_json::to_string(&AnalysisPreset::Fun)?, "\"fun\"");
    Ok(())
}

#[test]
fn handoff_preset_kebab_names() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Minimal)?,
        "\"minimal\""
    );
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Standard)?,
        "\"standard\""
    );
    assert_eq!(serde_json::to_string(&HandoffPreset::Risk)?, "\"risk\"");
    assert_eq!(serde_json::to_string(&HandoffPreset::Deep)?, "\"deep\"");
    Ok(())
}

#[test]
fn import_granularity_kebab_names() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        serde_json::to_string(&ImportGranularity::Module)?,
        "\"module\""
    );
    assert_eq!(serde_json::to_string(&ImportGranularity::File)?, "\"file\"");
    Ok(())
}

#[test]
fn badge_metric_kebab_names() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(serde_json::to_string(&BadgeMetric::Lines)?, "\"lines\"");
    assert_eq!(serde_json::to_string(&BadgeMetric::Tokens)?, "\"tokens\"");
    assert_eq!(serde_json::to_string(&BadgeMetric::Bytes)?, "\"bytes\"");
    assert_eq!(serde_json::to_string(&BadgeMetric::Doc)?, "\"doc\"");
    assert_eq!(serde_json::to_string(&BadgeMetric::Blank)?, "\"blank\"");
    assert_eq!(serde_json::to_string(&BadgeMetric::Hotspot)?, "\"hotspot\"");
    Ok(())
}

#[test]
fn init_profile_kebab_names() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(serde_json::to_string(&InitProfile::Default)?, "\"default\"");
    assert_eq!(serde_json::to_string(&InitProfile::Rust)?, "\"rust\"");
    assert_eq!(serde_json::to_string(&InitProfile::Node)?, "\"node\"");
    assert_eq!(serde_json::to_string(&InitProfile::Mono)?, "\"mono\"");
    assert_eq!(serde_json::to_string(&InitProfile::Python)?, "\"python\"");
    assert_eq!(serde_json::to_string(&InitProfile::Go)?, "\"go\"");
    assert_eq!(serde_json::to_string(&InitProfile::Cpp)?, "\"cpp\"");
    Ok(())
}

#[test]
fn shell_kebab_names() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(serde_json::to_string(&Shell::Bash)?, "\"bash\"");
    assert_eq!(serde_json::to_string(&Shell::Fish)?, "\"fish\"");
    assert_eq!(serde_json::to_string(&Shell::Zsh)?, "\"zsh\"");
    assert_eq!(serde_json::to_string(&Shell::Powershell)?, "\"powershell\"");
    assert_eq!(serde_json::to_string(&Shell::Elvish)?, "\"elvish\"");
    Ok(())
}

// =========================================================================
// 28. Enum JSON deserialization roundtrip
// =========================================================================

#[test]
fn all_analysis_presets_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    for v in [
        AnalysisPreset::Receipt,
        AnalysisPreset::Health,
        AnalysisPreset::Risk,
        AnalysisPreset::Supply,
        AnalysisPreset::Architecture,
        AnalysisPreset::Topics,
        AnalysisPreset::Security,
        AnalysisPreset::Identity,
        AnalysisPreset::Git,
        AnalysisPreset::Deep,
        AnalysisPreset::Fun,
    ] {
        let json = serde_json::to_string(&v)?;
        let back: AnalysisPreset = serde_json::from_str(&json)?;
        assert_eq!(v, back);
    }
    Ok(())
}

#[test]
fn all_export_formats_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    for v in [
        CliExportFormat::Jsonl,
        CliExportFormat::Csv,
        CliExportFormat::Json,
        CliExportFormat::Cyclonedx,
    ] {
        let json = serde_json::to_string(&v)?;
        let back: CliExportFormat = serde_json::from_str(&json)?;
        assert_eq!(v, back);
    }
    Ok(())
}

// =========================================================================
// 29. TOML dotted keys
// =========================================================================

#[test]
fn dotted_key_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("scan.hidden = true\nmodule.depth = 5")?;
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.module.depth, Some(5));
    Ok(())
}

// =========================================================================
// 30. Multiline arrays in TOML
// =========================================================================

#[test]
fn multiline_array_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[scan]
exclude = [
    "target",
    "node_modules",
    "dist",
]
"#,
    )?;
    assert_eq!(
        cfg.scan.exclude,
        Some(vec!["target".into(), "node_modules".into(), "dist".into()])
    );
    Ok(())
}

// =========================================================================
// 31. Config with only gate section
// =========================================================================

#[test]
fn gate_only_config() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[gate]
policy = "strict.toml"
fail_fast = false
allow_missing_baseline = true
allow_missing_current = false
"#,
    )?;
    assert_eq!(cfg.gate.policy, Some("strict.toml".into()));
    assert_eq!(cfg.gate.fail_fast, Some(false));
    assert_eq!(cfg.gate.allow_missing_baseline, Some(true));
    assert_eq!(cfg.gate.allow_missing_current, Some(false));
    // Other sections default
    assert!(cfg.scan.hidden.is_none());
    Ok(())
}

// =========================================================================
// 32. Whitespace-only TOML
// =========================================================================

#[test]
fn whitespace_only_toml_yields_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("   \n  \n\t\n")?;
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.view.is_empty());
    Ok(())
}

// =========================================================================
// 33. Boolean false values are explicitly stored (not None)
// =========================================================================

#[test]
fn explicit_false_stored_as_some_false() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[scan]
hidden = false
no_ignore = false

[gate]
fail_fast = false
allow_missing_baseline = false

[context]
compress = false
"#,
    )?;
    assert_eq!(cfg.scan.hidden, Some(false));
    assert_eq!(cfg.scan.no_ignore, Some(false));
    assert_eq!(cfg.gate.fail_fast, Some(false));
    assert_eq!(cfg.gate.allow_missing_baseline, Some(false));
    assert_eq!(cfg.context.compress, Some(false));
    Ok(())
}

// =========================================================================
// 34. Config section with "config" field (for tokei loading strategy)
// =========================================================================

#[test]
fn scan_config_field_accepts_strings() -> Result<(), Box<dyn std::error::Error>> {
    for val in ["auto", "none"] {
        let toml_str = format!("[scan]\nconfig = \"{}\"", val);
        let cfg = TomlConfig::parse(&toml_str)?;
        assert_eq!(cfg.scan.config, Some(val.to_string()));
    }
    Ok(())
}

// =========================================================================
// 35. Gate rules with mixed value types
// =========================================================================

#[test]
fn gate_rules_mixed_value_types_in_same_config() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[[gate.rules]]
name = "num"
pointer = "/a"
op = "gt"
value = 100

[[gate.rules]]
name = "str"
pointer = "/b"
op = "eq"
value = "hello"

[[gate.rules]]
name = "bool"
pointer = "/c"
op = "eq"
value = false
"#,
    )?;
    let rules = cfg.gate.rules.ok_or("missing rules")?;
    assert_eq!(rules.len(), 3);
    assert!(rules[0].value.as_ref().ok_or("missing")?.is_i64());
    assert!(rules[1].value.as_ref().ok_or("missing")?.is_string());
    assert!(rules[2].value.as_ref().ok_or("missing")?.is_boolean());
    Ok(())
}

// =========================================================================
// 36. Interaction: scan + analyze + gate in same config
// =========================================================================

#[test]
fn multiple_sections_coexist() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse(
        r#"
[scan]
hidden = true
exclude = ["vendor"]

[analyze]
preset = "health"
git = false

[gate]
fail_fast = true

[[gate.rules]]
name = "check"
pointer = "/x"
op = "lt"
value = 50
"#,
    )?;
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.analyze.preset, Some("health".into()));
    assert_eq!(cfg.analyze.git, Some(false));
    assert_eq!(cfg.gate.fail_fast, Some(true));
    assert_eq!(cfg.gate.rules.ok_or("missing rules")?.len(), 1);
    Ok(())
}

// =========================================================================
// 37. Analyze git field: true, false, and omitted
// =========================================================================

#[test]
fn analyze_git_true() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[analyze]\ngit = true")?;
    assert_eq!(cfg.analyze.git, Some(true));
    Ok(())
}

#[test]
fn analyze_git_false() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[analyze]\ngit = false")?;
    assert_eq!(cfg.analyze.git, Some(false));
    Ok(())
}

#[test]
fn analyze_git_omitted() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = TomlConfig::parse("[analyze]\npreset = \"receipt\"")?;
    assert!(cfg.analyze.git.is_none());
    Ok(())
}
