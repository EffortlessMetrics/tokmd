//! Wave-66 deep tests for tokmd-config.
//!
//! Coverage targets:
//! - TomlConfig parsing from TOML strings (all sections)
//! - Default values for all config structs
//! - Edge cases: empty string, unknown keys, partial configs
//! - ViewProfile (named view profiles)
//! - GateConfig with inline rules and ratchet rules
//! - Serialization roundtrip (deterministic)
//! - UserConfig and Profile types
//! - ScanConfig → GlobalArgs conversion

use std::collections::BTreeMap;
use tokmd_config::*;

// =========================================================================
// 1. TomlConfig defaults
// =========================================================================

#[test]
fn toml_config_default_all_none() {
    let cfg = TomlConfig::default();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.scan.exclude.is_none());
    assert!(cfg.scan.hidden.is_none());
    assert!(cfg.module.roots.is_none());
    assert!(cfg.module.depth.is_none());
    assert!(cfg.export.min_code.is_none());
    assert!(cfg.export.max_rows.is_none());
    assert!(cfg.analyze.preset.is_none());
    assert!(cfg.context.budget.is_none());
    assert!(cfg.badge.metric.is_none());
    assert!(cfg.gate.policy.is_none());
    assert!(cfg.view.is_empty());
}

// =========================================================================
// 2. Parse empty / minimal TOML
// =========================================================================

#[test]
fn parse_empty_string() {
    let cfg = TomlConfig::parse("").unwrap();
    assert!(cfg.scan.paths.is_none());
    assert!(cfg.view.is_empty());
}

#[test]
fn parse_only_scan_section() {
    let toml = r#"
[scan]
paths = ["src"]
hidden = true
"#;
    let cfg = TomlConfig::parse(toml).unwrap();
    assert_eq!(cfg.scan.paths.as_ref().unwrap(), &["src"]);
    assert_eq!(cfg.scan.hidden, Some(true));
    assert!(cfg.module.roots.is_none());
}

// =========================================================================
// 3. All sections
// =========================================================================

#[test]
fn parse_scan_section_full() {
    let toml = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target", "*.bak"]
hidden = false
config = "none"
no_ignore = true
no_ignore_parent = true
no_ignore_dot = false
no_ignore_vcs = true
doc_comments = true
"#;
    let cfg = TomlConfig::parse(toml).unwrap();
    assert_eq!(cfg.scan.exclude.as_ref().unwrap().len(), 2);
    assert_eq!(cfg.scan.no_ignore, Some(true));
    assert_eq!(cfg.scan.no_ignore_vcs, Some(true));
    assert_eq!(cfg.scan.doc_comments, Some(true));
}

#[test]
fn parse_module_section() {
    let toml = r#"
[module]
roots = ["crates", "packages"]
depth = 3
children = "collapse"
"#;
    let cfg = TomlConfig::parse(toml).unwrap();
    assert_eq!(cfg.module.roots.as_ref().unwrap(), &["crates", "packages"]);
    assert_eq!(cfg.module.depth, Some(3));
    assert_eq!(cfg.module.children.as_deref(), Some("collapse"));
}

#[test]
fn parse_export_section() {
    let toml = r#"
[export]
min_code = 10
max_rows = 500
redact = "paths"
format = "csv"
children = "separate"
"#;
    let cfg = TomlConfig::parse(toml).unwrap();
    assert_eq!(cfg.export.min_code, Some(10));
    assert_eq!(cfg.export.max_rows, Some(500));
    assert_eq!(cfg.export.redact.as_deref(), Some("paths"));
}

#[test]
fn parse_analyze_section() {
    let toml = r#"
[analyze]
preset = "deep"
window = 200000
format = "json"
git = true
max_files = 10000
max_bytes = 50000000
max_file_bytes = 1000000
max_commits = 500
max_commit_files = 100
granularity = "file"
"#;
    let cfg = TomlConfig::parse(toml).unwrap();
    assert_eq!(cfg.analyze.preset.as_deref(), Some("deep"));
    assert_eq!(cfg.analyze.window, Some(200000));
    assert_eq!(cfg.analyze.git, Some(true));
    assert_eq!(cfg.analyze.max_files, Some(10000));
    assert_eq!(cfg.analyze.granularity.as_deref(), Some("file"));
}

#[test]
fn parse_context_section() {
    let toml = r#"
[context]
budget = "256k"
strategy = "spread"
rank_by = "churn"
output = "bundle"
compress = true
"#;
    let cfg = TomlConfig::parse(toml).unwrap();
    assert_eq!(cfg.context.budget.as_deref(), Some("256k"));
    assert_eq!(cfg.context.strategy.as_deref(), Some("spread"));
    assert_eq!(cfg.context.compress, Some(true));
}

#[test]
fn parse_badge_section() {
    let toml = r#"
[badge]
metric = "tokens"
"#;
    let cfg = TomlConfig::parse(toml).unwrap();
    assert_eq!(cfg.badge.metric.as_deref(), Some("tokens"));
}

#[test]
fn parse_gate_section_with_inline_rules() {
    let toml = r#"
[gate]
policy = "gate.toml"
baseline = ".tokmd/baseline.json"
preset = "risk"
fail_fast = true
allow_missing_baseline = true
allow_missing_current = false

[[gate.rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"

[[gate.ratchet]]
pointer = "/complexity/avg"
max_increase_pct = 5.0
max_value = 50.0
level = "warn"
description = "complexity ceiling"
"#;
    let cfg = TomlConfig::parse(toml).unwrap();
    assert_eq!(cfg.gate.policy.as_deref(), Some("gate.toml"));
    assert!(cfg.gate.fail_fast.unwrap());
    let rules = cfg.gate.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "max_tokens");
    assert_eq!(rules[0].op, "lte");
    let ratchet = cfg.gate.ratchet.as_ref().unwrap();
    assert_eq!(ratchet.len(), 1);
    assert_eq!(ratchet[0].max_increase_pct, Some(5.0));
    assert_eq!(
        ratchet[0].description.as_deref(),
        Some("complexity ceiling")
    );
}

// =========================================================================
// 4. View profiles
// =========================================================================

#[test]
fn parse_view_profiles() {
    let toml = r#"
[view.llm]
format = "json"
top = 20
redact = "all"

[view.ci]
format = "md"
files = true
"#;
    let cfg = TomlConfig::parse(toml).unwrap();
    assert_eq!(cfg.view.len(), 2);
    let llm = &cfg.view["llm"];
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(20));
    assert_eq!(llm.redact.as_deref(), Some("all"));
    let ci = &cfg.view["ci"];
    assert_eq!(ci.format.as_deref(), Some("md"));
    assert_eq!(ci.files, Some(true));
}

#[test]
fn view_profile_default_all_none() {
    let vp = ViewProfile::default();
    assert!(vp.format.is_none());
    assert!(vp.top.is_none());
    assert!(vp.files.is_none());
    assert!(vp.budget.is_none());
    assert!(vp.preset.is_none());
}

// =========================================================================
// 5. UserConfig and Profile
// =========================================================================

#[test]
fn user_config_default_empty() {
    let c = UserConfig::default();
    assert!(c.profiles.is_empty());
    assert!(c.repos.is_empty());
}

#[test]
fn profile_default_all_none() {
    let p = Profile::default();
    assert!(p.format.is_none());
    assert!(p.top.is_none());
    assert!(p.files.is_none());
    assert!(p.module_roots.is_none());
    assert!(p.module_depth.is_none());
    assert!(p.min_code.is_none());
    assert!(p.max_rows.is_none());
    assert!(p.redact.is_none());
    assert!(p.meta.is_none());
    assert!(p.children.is_none());
}

// =========================================================================
// 6. GlobalArgs default
// =========================================================================

#[test]
fn global_args_default_values() {
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
}

// =========================================================================
// 7. Serialization roundtrip
// =========================================================================

#[test]
fn toml_config_serde_roundtrip() {
    let toml_str = r#"
[scan]
paths = ["src"]
hidden = true

[module]
roots = ["crates"]
depth = 2

[export]
min_code = 5
format = "jsonl"

[analyze]
preset = "receipt"
window = 128000
"#;
    let cfg = TomlConfig::parse(toml_str).unwrap();
    let json = serde_json::to_string(&cfg).unwrap();
    let back: TomlConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scan.paths, cfg.scan.paths);
    assert_eq!(back.module.roots, cfg.module.roots);
    assert_eq!(back.export.min_code, cfg.export.min_code);
    assert_eq!(back.analyze.preset, cfg.analyze.preset);
}

#[test]
fn profile_serde_roundtrip() {
    let p = Profile {
        format: Some("json".into()),
        top: Some(10),
        files: Some(true),
        module_roots: Some(vec!["crates".into()]),
        module_depth: Some(2),
        min_code: Some(5),
        max_rows: Some(100),
        redact: None,
        meta: Some(true),
        children: Some("collapse".into()),
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: Profile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format, p.format);
    assert_eq!(back.top, p.top);
    assert_eq!(back.children, p.children);
}

#[test]
fn user_config_serde_roundtrip_with_profiles() {
    let mut profiles = BTreeMap::new();
    profiles.insert(
        "ci".into(),
        Profile {
            format: Some("json".into()),
            ..Default::default()
        },
    );
    let mut repos = BTreeMap::new();
    repos.insert("org/repo".into(), "ci".into());
    let uc = UserConfig { profiles, repos };
    let json = serde_json::to_string(&uc).unwrap();
    let back: UserConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.profiles.len(), 1);
    assert_eq!(back.repos.len(), 1);
    assert_eq!(back.profiles["ci"].format.as_deref(), Some("json"));
}

// =========================================================================
// 8. from_file roundtrip
// =========================================================================

#[test]
fn toml_config_from_file() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
hidden = true

[module]
roots = ["crates"]
"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tokmd.toml");
    std::fs::write(&path, toml_str).unwrap();

    let cfg = TomlConfig::from_file(&path).unwrap();
    assert_eq!(cfg.scan.paths.as_ref().unwrap(), &["src", "lib"]);
    assert_eq!(cfg.scan.hidden, Some(true));
    assert_eq!(cfg.module.roots.as_ref().unwrap(), &["crates"]);
}

// =========================================================================
// 9. Property tests: determinism
// =========================================================================

#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn toml_parse_deterministic(hidden in proptest::bool::ANY, depth in 1usize..10) {
            let toml = format!(
                "[scan]\nhidden = {hidden}\n\n[module]\ndepth = {depth}\n"
            );
            let c1 = TomlConfig::parse(&toml).unwrap();
            let c2 = TomlConfig::parse(&toml).unwrap();
            prop_assert_eq!(c1.scan.hidden, c2.scan.hidden);
            prop_assert_eq!(c1.module.depth, c2.module.depth);
        }

        #[test]
        fn profile_serde_roundtrip_prop(top in 0usize..1000) {
            let p = Profile {
                top: Some(top),
                ..Default::default()
            };
            let json = serde_json::to_string(&p).unwrap();
            let back: Profile = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(back.top, p.top);
        }
    }
}
