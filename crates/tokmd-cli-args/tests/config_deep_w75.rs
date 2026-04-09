//! Deep tests for tokmd-config (w75).
//!
//! Covers: TOML config loading, profile system, UserConfig/Profile,
//! default values, serialization roundtrips, enum coverage, and
//! GlobalArgs → ScanOptions conversion edge cases.

use std::io::Write;

use tempfile::NamedTempFile;
use tokmd_cli_args::*;
use tokmd_settings::ScanOptions;

// ── 1. TOML config: full section parse ──────────────────────────────

#[test]
fn toml_full_config_all_sections() {
    let toml_str = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target", "*.bak"]
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
min_code = 10
max_rows = 500
redact = "paths"
format = "csv"
children = "separate"

[analyze]
preset = "deep"
window = 128000
format = "json"
git = true
max_files = 5000
max_bytes = 1048576
max_file_bytes = 65536
max_commits = 200
max_commit_files = 50
granularity = "file"

[context]
budget = "256k"
strategy = "spread"
rank_by = "tokens"
output = "bundle"
compress = true

[badge]
metric = "tokens"

[gate]
policy = "policy.toml"
baseline = "baseline.json"
preset = "health"
fail_fast = true
allow_missing_baseline = true
allow_missing_current = false
"#;
    let config = TomlConfig::parse(toml_str).expect("parse full config");

    // scan
    assert_eq!(
        config.scan.paths,
        Some(vec!["src".to_string(), "lib".to_string()])
    );
    assert_eq!(
        config.scan.exclude,
        Some(vec!["target".to_string(), "*.bak".to_string()])
    );
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.config, Some("none".to_string()));
    assert_eq!(config.scan.no_ignore, Some(true));
    assert_eq!(config.scan.no_ignore_parent, Some(true));
    assert_eq!(config.scan.no_ignore_dot, Some(true));
    assert_eq!(config.scan.no_ignore_vcs, Some(true));
    assert_eq!(config.scan.doc_comments, Some(true));

    // module
    assert_eq!(
        config.module.roots,
        Some(vec![
            "crates".to_string(),
            "packages".to_string(),
            "libs".to_string()
        ])
    );
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(config.module.children, Some("collapse".to_string()));

    // export
    assert_eq!(config.export.min_code, Some(10));
    assert_eq!(config.export.max_rows, Some(500));
    assert_eq!(config.export.redact, Some("paths".to_string()));
    assert_eq!(config.export.format, Some("csv".to_string()));
    assert_eq!(config.export.children, Some("separate".to_string()));

    // analyze
    assert_eq!(config.analyze.preset, Some("deep".to_string()));
    assert_eq!(config.analyze.window, Some(128000));
    assert_eq!(config.analyze.format, Some("json".to_string()));
    assert_eq!(config.analyze.git, Some(true));
    assert_eq!(config.analyze.max_files, Some(5000));
    assert_eq!(config.analyze.max_bytes, Some(1_048_576));
    assert_eq!(config.analyze.max_file_bytes, Some(65536));
    assert_eq!(config.analyze.max_commits, Some(200));
    assert_eq!(config.analyze.max_commit_files, Some(50));
    assert_eq!(config.analyze.granularity, Some("file".to_string()));

    // context
    assert_eq!(config.context.budget, Some("256k".to_string()));
    assert_eq!(config.context.strategy, Some("spread".to_string()));
    assert_eq!(config.context.rank_by, Some("tokens".to_string()));
    assert_eq!(config.context.output, Some("bundle".to_string()));
    assert_eq!(config.context.compress, Some(true));

    // badge
    assert_eq!(config.badge.metric, Some("tokens".to_string()));

    // gate
    assert_eq!(config.gate.policy, Some("policy.toml".to_string()));
    assert_eq!(config.gate.baseline, Some("baseline.json".to_string()));
    assert_eq!(config.gate.preset, Some("health".to_string()));
    assert_eq!(config.gate.fail_fast, Some(true));
    assert_eq!(config.gate.allow_missing_baseline, Some(true));
    assert_eq!(config.gate.allow_missing_current, Some(false));
}

// ── 2. Empty TOML yields all-None defaults ──────────────────────────

#[test]
fn toml_empty_string_gives_defaults() {
    let config = TomlConfig::parse("").expect("parse empty");
    assert!(config.scan.paths.is_none());
    assert!(config.scan.hidden.is_none());
    assert!(config.module.roots.is_none());
    assert!(config.module.depth.is_none());
    assert!(config.export.min_code.is_none());
    assert!(config.analyze.preset.is_none());
    assert!(config.context.budget.is_none());
    assert!(config.badge.metric.is_none());
    assert!(config.gate.policy.is_none());
    assert!(config.view.is_empty());
}

// ── 3. TOML from_file with tempfile ─────────────────────────────────

#[test]
fn toml_from_file_loads_correctly() {
    let content = r#"
[scan]
hidden = true
exclude = ["vendor"]

[analyze]
preset = "risk"
max_commits = 100
"#;
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();

    let config = TomlConfig::from_file(f.path()).expect("from_file");
    assert_eq!(config.scan.hidden, Some(true));
    assert_eq!(config.scan.exclude, Some(vec!["vendor".to_string()]));
    assert_eq!(config.analyze.preset, Some("risk".to_string()));
    assert_eq!(config.analyze.max_commits, Some(100));
}

// ── 4. TOML from_file: non-existent path → io::Error ────────────────

#[test]
fn toml_from_file_nonexistent_returns_error() {
    let result = TomlConfig::from_file(std::path::Path::new("/nonexistent/tokmd.toml"));
    assert!(result.is_err());
}

// ── 5. TOML parse: malformed TOML → error ───────────────────────────

#[test]
fn toml_parse_malformed_returns_error() {
    let result = TomlConfig::parse("[scan\nhidden = ");
    assert!(result.is_err());
}

// ── 6. View profiles: multiple named profiles ───────────────────────

#[test]
fn toml_multiple_view_profiles() {
    let toml_str = r#"
[view.ci]
format = "json"
top = 5
preset = "receipt"

[view.llm]
format = "json"
redact = "all"
budget = "64k"
strategy = "greedy"
compress = true
meta = false

[view.dev]
files = true
module_depth = 4
module_roots = ["src", "crates"]
children = "collapse"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.view.len(), 3);

    let ci = &config.view["ci"];
    assert_eq!(ci.format.as_deref(), Some("json"));
    assert_eq!(ci.top, Some(5));
    assert_eq!(ci.preset.as_deref(), Some("receipt"));

    let llm = &config.view["llm"];
    assert_eq!(llm.redact.as_deref(), Some("all"));
    assert_eq!(llm.budget.as_deref(), Some("64k"));
    assert_eq!(llm.compress, Some(true));
    assert_eq!(llm.meta, Some(false));

    let dev = &config.view["dev"];
    assert_eq!(dev.files, Some(true));
    assert_eq!(dev.module_depth, Some(4));
    assert_eq!(
        dev.module_roots,
        Some(vec!["src".to_string(), "crates".to_string()])
    );
    assert_eq!(dev.children.as_deref(), Some("collapse"));
}

// ── 7. Gate rules inline in TOML ────────────────────────────────────

#[test]
fn toml_gate_inline_rules() {
    let toml_str = r#"
[[gate.rules]]
name = "min-coverage"
pointer = "/derived/comment_density"
op = ">"
value = 0.1

[[gate.rules]]
name = "lang-check"
pointer = "/languages"
op = "in"
values = ["Rust", "Python"]
negate = true
level = "warn"
message = "Unexpected language detected"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let rules = config.gate.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 2);

    assert_eq!(rules[0].name, "min-coverage");
    assert_eq!(rules[0].pointer, "/derived/comment_density");
    assert_eq!(rules[0].op, ">");
    assert_eq!(rules[0].value, Some(serde_json::json!(0.1)));
    assert!(!rules[0].negate);
    assert!(rules[0].level.is_none());

    assert_eq!(rules[1].name, "lang-check");
    assert_eq!(rules[1].op, "in");
    assert!(rules[1].negate);
    assert_eq!(rules[1].level.as_deref(), Some("warn"));
    assert_eq!(
        rules[1].message.as_deref(),
        Some("Unexpected language detected")
    );
}

// ── 8. Ratchet rules inline in TOML ─────────────────────────────────

#[test]
fn toml_gate_inline_ratchet_rules() {
    let toml_str = r#"
[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0
max_value = 25.0
level = "error"
description = "Cyclomatic complexity must not regress"

[[gate.ratchet]]
pointer = "/derived/total_code"
level = "warn"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let ratchets = config.gate.ratchet.as_ref().unwrap();
    assert_eq!(ratchets.len(), 2);

    assert_eq!(ratchets[0].pointer, "/complexity/avg_cyclomatic");
    assert_eq!(ratchets[0].max_increase_pct, Some(5.0));
    assert_eq!(ratchets[0].max_value, Some(25.0));
    assert_eq!(ratchets[0].level.as_deref(), Some("error"));
    assert!(ratchets[0].description.is_some());

    assert_eq!(ratchets[1].pointer, "/derived/total_code");
    assert!(ratchets[1].max_increase_pct.is_none());
    assert!(ratchets[1].max_value.is_none());
    assert_eq!(ratchets[1].level.as_deref(), Some("warn"));
}

// ── 9. UserConfig serde roundtrip with multiple profiles ────────────

#[test]
fn user_config_multiple_profiles_roundtrip() {
    let mut config = UserConfig::default();
    config.profiles.insert(
        "ci".into(),
        Profile {
            format: Some("json".into()),
            top: Some(5),
            files: Some(true),
            module_roots: Some(vec!["src".into()]),
            module_depth: Some(3),
            min_code: Some(10),
            max_rows: Some(100),
            redact: Some(CliRedactMode::Paths),
            meta: Some(true),
            children: Some("collapse".into()),
        },
    );
    config.profiles.insert(
        "dev".into(),
        Profile {
            format: Some("md".into()),
            ..Profile::default()
        },
    );
    config.repos.insert("org/repo-a".into(), "ci".into());
    config.repos.insert("org/repo-b".into(), "dev".into());

    let json = serde_json::to_string_pretty(&config).unwrap();
    let back: UserConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(back.profiles.len(), 2);
    assert_eq!(back.repos.len(), 2);
    let ci = &back.profiles["ci"];
    assert_eq!(ci.format.as_deref(), Some("json"));
    assert_eq!(ci.top, Some(5));
    assert_eq!(ci.files, Some(true));
    assert_eq!(ci.module_depth, Some(3));
    assert_eq!(ci.min_code, Some(10));
    assert_eq!(ci.max_rows, Some(100));
    assert_eq!(ci.redact, Some(CliRedactMode::Paths));
    assert_eq!(ci.meta, Some(true));
    assert_eq!(ci.children.as_deref(), Some("collapse"));
    assert_eq!(back.repos["org/repo-a"], "ci");
}

// ── 10. Profile default: all fields None ────────────────────────────

#[test]
fn profile_all_fields_none_by_default() {
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

// ── 11. GlobalArgs default values ───────────────────────────────────

#[test]
fn global_args_defaults_complete() {
    let g = GlobalArgs::default();
    assert!(g.excluded.is_empty());
    assert_eq!(g.config, CliConfigMode::Auto);
    assert!(!g.hidden);
    assert!(!g.no_ignore);
    assert!(!g.no_ignore_parent);
    assert!(!g.no_ignore_dot);
    assert!(!g.no_ignore_vcs);
    assert!(!g.treat_doc_strings_as_comments);
    assert_eq!(g.verbose, 0);
    assert!(!g.no_progress);
}

// ── 12. GlobalArgs → ScanOptions: all flags true ────────────────────

#[test]
fn global_args_to_scan_options_all_true() {
    let g = GlobalArgs {
        excluded: vec!["target".into(), "dist".into()],
        config: CliConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
        verbose: 3,
        no_progress: true,
    };
    let opts: ScanOptions = (&g).into();
    assert_eq!(opts.excluded, vec!["target", "dist"]);
    assert_eq!(opts.config, tokmd_types::ConfigMode::None);
    assert!(opts.hidden);
    assert!(opts.no_ignore);
    assert!(opts.no_ignore_parent);
    assert!(opts.no_ignore_dot);
    assert!(opts.no_ignore_vcs);
    assert!(opts.treat_doc_strings_as_comments);
}

// ── 13. GlobalArgs → ScanOptions: owned conversion ──────────────────

#[test]
fn global_args_owned_conversion_preserves_values() {
    let g = GlobalArgs {
        excluded: vec!["*.log".into()],
        config: CliConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: false,
        verbose: 1,
        no_progress: false,
    };
    let opts: ScanOptions = g.into();
    assert_eq!(opts.excluded, vec!["*.log"]);
    assert!(!opts.hidden);
    assert!(opts.no_ignore_parent);
    assert!(opts.no_ignore_vcs);
}

// ── 14. Enum serde: all AnalysisPreset variants ─────────────────────

#[test]
fn analysis_preset_all_variants_kebab_case() {
    let expected = vec![
        (AnalysisPreset::Receipt, "\"receipt\""),
        (AnalysisPreset::Health, "\"health\""),
        (AnalysisPreset::Risk, "\"risk\""),
        (AnalysisPreset::Supply, "\"supply\""),
        (AnalysisPreset::Architecture, "\"architecture\""),
        (AnalysisPreset::Topics, "\"topics\""),
        (AnalysisPreset::Security, "\"security\""),
        (AnalysisPreset::Identity, "\"identity\""),
        (AnalysisPreset::Git, "\"git\""),
        (AnalysisPreset::Deep, "\"deep\""),
        (AnalysisPreset::Fun, "\"fun\""),
    ];
    for (variant, expected_str) in expected {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected_str, "variant {:?}", variant);
        let back: AnalysisPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ── 15. Enum defaults ───────────────────────────────────────────────

#[test]
fn enum_defaults_all_correct() {
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
}

// ── 16. Enum serde: HandoffPreset variants ──────────────────────────

#[test]
fn handoff_preset_serde_roundtrip() {
    for variant in [
        HandoffPreset::Minimal,
        HandoffPreset::Standard,
        HandoffPreset::Risk,
        HandoffPreset::Deep,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: HandoffPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ── 17. TOML config serialization roundtrip ─────────────────────────

#[test]
fn toml_config_json_roundtrip() {
    let toml_str = r#"
[scan]
hidden = true

[module]
depth = 4
roots = ["packages"]

[context]
budget = "1m"

[view.ci]
format = "json"
top = 3
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let json = serde_json::to_string(&config).unwrap();
    let back: TomlConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(back.scan.hidden, Some(true));
    assert_eq!(back.module.depth, Some(4));
    assert_eq!(back.context.budget, Some("1m".to_string()));
    assert_eq!(back.view.len(), 1);
    assert_eq!(back.view["ci"].top, Some(3));
}

// ── 18. ViewProfile: all fields populated ───────────────────────────

#[test]
fn view_profile_all_fields() {
    let toml_str = r#"
[view.full]
format = "json"
top = 20
files = true
module_roots = ["crates"]
module_depth = 3
min_code = 5
max_rows = 200
redact = "paths"
meta = true
children = "separate"
preset = "deep"
window = 128000
budget = "512k"
strategy = "spread"
rank_by = "hotspot"
output = "json"
compress = false
metric = "lines"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    let p = &config.view["full"];
    assert_eq!(p.format.as_deref(), Some("json"));
    assert_eq!(p.top, Some(20));
    assert_eq!(p.files, Some(true));
    assert_eq!(p.module_roots, Some(vec!["crates".to_string()]));
    assert_eq!(p.module_depth, Some(3));
    assert_eq!(p.min_code, Some(5));
    assert_eq!(p.max_rows, Some(200));
    assert_eq!(p.redact.as_deref(), Some("paths"));
    assert_eq!(p.meta, Some(true));
    assert_eq!(p.children.as_deref(), Some("separate"));
    assert_eq!(p.preset.as_deref(), Some("deep"));
    assert_eq!(p.window, Some(128000));
    assert_eq!(p.budget.as_deref(), Some("512k"));
    assert_eq!(p.strategy.as_deref(), Some("spread"));
    assert_eq!(p.rank_by.as_deref(), Some("hotspot"));
    assert_eq!(p.output.as_deref(), Some("json"));
    assert_eq!(p.compress, Some(false));
    assert_eq!(p.metric.as_deref(), Some("lines"));
}

// ── 19. TOML from_file: invalid TOML content → error ────────────────

#[test]
fn toml_from_file_invalid_content_returns_error() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"not valid [[ toml !!!").unwrap();
    let result = TomlConfig::from_file(f.path());
    assert!(result.is_err());
}

// ── 20. UserConfig: empty profiles serialize to empty maps ──────────

#[test]
fn user_config_empty_roundtrip() {
    let config = UserConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let back: UserConfig = serde_json::from_str(&json).unwrap();
    assert!(back.profiles.is_empty());
    assert!(back.repos.is_empty());
}

// ── 21. TOML: partial sections leave others as default ──────────────

#[test]
fn toml_partial_sections_others_default() {
    let toml_str = r#"
[analyze]
preset = "supply"
"#;
    let config = TomlConfig::parse(toml_str).unwrap();
    assert_eq!(config.analyze.preset, Some("supply".to_string()));
    // All other sections should have None/default values
    assert!(config.scan.hidden.is_none());
    assert!(config.module.depth.is_none());
    assert!(config.export.format.is_none());
    assert!(config.context.budget.is_none());
    assert!(config.badge.metric.is_none());
    assert!(config.gate.policy.is_none());
    assert!(config.view.is_empty());
}

// ── 22. CliLangArgs default ─────────────────────────────────────────

#[test]
fn cli_lang_args_default_values() {
    let a = CliLangArgs::default();
    assert!(a.paths.is_none());
    assert!(a.format.is_none());
    assert!(a.top.is_none());
    assert!(!a.files);
    assert!(a.children.is_none());
}
