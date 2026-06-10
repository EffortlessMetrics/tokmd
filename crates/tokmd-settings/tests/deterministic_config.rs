use tokmd_settings::TomlConfig;

#[test]
fn toml_config_parses_kitchen_sink() {
    let toml_str = r#"
[scan]
paths = ["src", "tests", "docs"]
exclude = ["target", "*.bak"]
hidden = true
no_ignore = false
no_ignore_parent = true
no_ignore_dot = false
no_ignore_vcs = true
doc_comments = true
config = "auto"

[module]
roots = ["crates"]
depth = 2
children = "collapse"

[export]
min_code = 10
max_rows = 100
redact = "paths"
format = "jsonl"
children = "separate"

[analyze]
preset = "receipt"
window = 20
format = "table"
git = true
max_files = 1000
max_bytes = 10485760
max_file_bytes = 1048576
max_commits = 50
max_commit_files = 100
granularity = "module"
effort_model = "standard"
effort_layer = "complexity"
effort_base_ref = "main"
effort_head_ref = "feature"
effort_monte_carlo = true
effort_mc_iterations = 1000
effort_mc_seed = 42

[context]
budget = "8k"
strategy = "spread"
rank_by = "code"
output = "bundle"
compress = true

[badge]
metric = "code"

[gate]
policy = "policy.toml"
baseline = "baseline.json"
preset = "default"
fail_fast = true
allow_missing_baseline = false
allow_missing_current = false

[[gate.rules]]
name = "Max Complexity"
pointer = "/complexity"
op = "lt"
value = 100
level = "error"
message = "Complexity exceeded threshold"

[[gate.ratchet]]
pointer = "/totals/code"
max_increase_pct = 5.0
max_value = 50000.0
level = "warn"
description = "Code size growth limit"

[view.ci]
format = "json"
top = 50
files = true
meta = true
preset = "ci"

[view.llm]
budget = "128k"
strategy = "greedy"
output = "json"
compress = false
"#;

    let config = TomlConfig::parse(toml_str).unwrap();

    // Scan
    assert_eq!(
        config.scan.paths,
        Some(vec!["src".into(), "tests".into(), "docs".into()])
    );
    assert_eq!(config.scan.hidden, Some(true));

    // Module
    assert_eq!(config.module.depth, Some(2));

    // Export
    assert_eq!(config.export.min_code, Some(10));
    assert_eq!(config.export.redact.as_deref(), Some("paths"));

    // Analyze
    assert_eq!(config.analyze.effort_mc_seed, Some(42));
    assert_eq!(config.analyze.git, Some(true));

    // Gate
    assert_eq!(config.gate.rules.as_ref().unwrap().len(), 1);
    assert_eq!(config.gate.ratchet.as_ref().unwrap().len(), 1);

    // View
    let ci_view = config.view.get("ci").unwrap();
    assert_eq!(ci_view.top, Some(50));
    assert_eq!(ci_view.meta, Some(true));
}

#[test]
fn toml_config_rejects_invalid_types() {
    let toml_str = r#"
[scan]
paths = "src" # Should be array
"#;
    assert!(TomlConfig::parse(toml_str).is_err());
}
