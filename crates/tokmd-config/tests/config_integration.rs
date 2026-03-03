//! Integration and determinism tests for tokmd-config.
//!
//! Focus areas:
//! - TomlConfig determinism (parse same input → identical output)
//! - Config with combined sections stress test
//! - Profile name edge cases (Unicode, special chars, very long names)
//! - from_file with concurrent-safe tempdir patterns
//! - TomlConfig merge semantics (TOML sections vs view profiles)
//! - Handoff/sensor/baseline CLI arg interaction with config

use std::io::Write;
use tempfile::NamedTempFile;
use tokmd_config::TomlConfig;

// =========================================================================
// Scenario: TomlConfig determinism — same TOML always parses identically
// =========================================================================

mod toml_determinism {
    use super::*;

    #[test]
    fn given_same_toml_when_parsed_ten_times_then_json_identical() {
        let toml_str = r#"
[scan]
hidden = true
no_ignore = false
exclude = ["target", "node_modules"]

[module]
roots = ["crates", "packages"]
depth = 3

[export]
format = "csv"
min_code = 5
max_rows = 1000

[analyze]
preset = "risk"
window = 200000

[context]
budget = "256k"
strategy = "spread"
compress = true

[badge]
metric = "tokens"

[gate]
fail_fast = true

[[gate.rules]]
name = "max-lines"
pointer = "/summary/total_code"
op = "lt"
value = 100000

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 5.0

[view.ci]
format = "tsv"
top = 50
"#;
        let first_json = {
            let config = TomlConfig::parse(toml_str).unwrap();
            serde_json::to_string(&config).unwrap()
        };

        for i in 1..10 {
            let config = TomlConfig::parse(toml_str).unwrap();
            let json = serde_json::to_string(&config).unwrap();
            assert_eq!(first_json, json, "parse iteration {} diverged", i);
        }
    }

    #[test]
    fn given_toml_from_file_when_loaded_twice_then_identical() {
        let toml_content = r#"
[scan]
hidden = true

[module]
depth = 4
roots = ["src", "lib"]

[view.test]
format = "json"
top = 10
"#;
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(toml_content.as_bytes()).unwrap();

        let config1 = TomlConfig::from_file(tmp.path()).unwrap();
        let config2 = TomlConfig::from_file(tmp.path()).unwrap();

        let json1 = serde_json::to_string(&config1).unwrap();
        let json2 = serde_json::to_string(&config2).unwrap();
        assert_eq!(json1, json2);
    }
}

// =========================================================================
// Scenario: Profile names with special characters
// =========================================================================

mod profile_name_edge_cases {
    use super::*;

    #[test]
    fn given_profile_name_with_hyphens_then_parsed_correctly() {
        let toml_str = r#"
[view.my-ci-profile]
format = "tsv"
top = 25
"#;
        let config = TomlConfig::parse(toml_str).unwrap();
        let profile = config.view.get("my-ci-profile").unwrap();
        assert_eq!(profile.format, Some("tsv".to_string()));
        assert_eq!(profile.top, Some(25));
    }

    #[test]
    fn given_profile_name_with_numbers_then_parsed_correctly() {
        let toml_str = r#"
[view.v2]
format = "json"

[view.profile123]
format = "md"
"#;
        let config = TomlConfig::parse(toml_str).unwrap();
        assert!(config.view.contains_key("v2"));
        assert!(config.view.contains_key("profile123"));
    }

    #[test]
    fn given_profile_name_with_dots_quoted_then_parsed_correctly() {
        let toml_str = r#"
[view."org.team.config"]
format = "json"
"#;
        let config = TomlConfig::parse(toml_str).unwrap();
        assert!(config.view.contains_key("org.team.config"));
    }

    #[test]
    fn given_profile_name_with_unicode_then_parsed_correctly() {
        let toml_str = "[view.\"日本語\"]\nformat = \"json\"\n";
        let config = TomlConfig::parse(toml_str).unwrap();
        assert!(config.view.contains_key("日本語"));
        assert_eq!(
            config.view.get("日本語").unwrap().format,
            Some("json".to_string())
        );
    }

    #[test]
    fn given_many_profiles_then_btreemap_maintains_sorted_order() {
        let toml_str = r#"
[view.z_last]
format = "md"

[view.a_first]
format = "json"

[view.m_middle]
format = "tsv"

[view.b_second]
format = "csv"
"#;
        let config = TomlConfig::parse(toml_str).unwrap();
        let keys: Vec<&String> = config.view.keys().collect();
        assert_eq!(keys, vec!["a_first", "b_second", "m_middle", "z_last"]);
    }
}

// =========================================================================
// Scenario: Gate config with boundary values
// =========================================================================

mod gate_boundary_values {
    use super::*;

    #[test]
    fn given_ratchet_with_zero_pct_then_parsed() {
        let toml_str = r#"
[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 0.0
"#;
        let config = TomlConfig::parse(toml_str).unwrap();
        let ratchet = config.gate.ratchet.unwrap();
        assert_eq!(ratchet[0].max_increase_pct, Some(0.0));
    }

    #[test]
    fn given_ratchet_with_100_pct_then_parsed() {
        let toml_str = r#"
[[gate.ratchet]]
pointer = "/summary/total_code"
max_increase_pct = 100.0
"#;
        let config = TomlConfig::parse(toml_str).unwrap();
        let ratchet = config.gate.ratchet.unwrap();
        assert_eq!(ratchet[0].max_increase_pct, Some(100.0));
    }

    #[test]
    fn given_rule_with_value_zero_then_parsed() {
        let toml_str = r#"
[[gate.rules]]
name = "no-code"
pointer = "/summary/total_code"
op = "eq"
value = 0
"#;
        let config = TomlConfig::parse(toml_str).unwrap();
        let rules = config.gate.rules.unwrap();
        assert_eq!(rules[0].name, "no-code");
    }

    #[test]
    fn given_many_rules_then_all_parsed_in_order() {
        let mut toml_str = String::from("[gate]\nfail_fast = true\n\n");
        for i in 0..10 {
            toml_str.push_str(&format!(
                "[[gate.rules]]\nname = \"rule-{i}\"\npointer = \"/metric/{i}\"\nop = \"lt\"\nvalue = {}\n\n",
                i * 100
            ));
        }
        let config = TomlConfig::parse(&toml_str).unwrap();
        let rules = config.gate.rules.unwrap();
        assert_eq!(rules.len(), 10);
        for (i, rule) in rules.iter().enumerate() {
            assert_eq!(rule.name, format!("rule-{i}"));
        }
    }
}

// =========================================================================
// Scenario: from_file error conditions
// =========================================================================

mod from_file_errors {
    use super::*;
    use std::path::Path;

    #[test]
    fn given_nonexistent_path_then_returns_error() {
        let result = TomlConfig::from_file(Path::new("/absolutely/nonexistent/path/tokmd.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn given_empty_file_then_returns_default_config() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"").unwrap();
        let config = TomlConfig::from_file(tmp.path()).unwrap();
        assert_eq!(config.scan.hidden, None);
        assert!(config.view.is_empty());
        assert!(config.gate.rules.is_none());
    }

    #[test]
    fn given_file_with_only_whitespace_then_returns_default_config() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"   \n\n  \t  \n").unwrap();
        let config = TomlConfig::from_file(tmp.path()).unwrap();
        assert_eq!(config.scan.hidden, None);
    }

    #[test]
    fn given_file_with_only_comments_then_returns_default_config() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"# This is a comment\n# Another comment\n")
            .unwrap();
        let config = TomlConfig::from_file(tmp.path()).unwrap();
        assert_eq!(config.scan.hidden, None);
    }

    #[test]
    fn given_file_with_invalid_toml_then_returns_error() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"[broken\nhidden = true").unwrap();
        let result = TomlConfig::from_file(tmp.path());
        assert!(result.is_err());
    }
}

// =========================================================================
// Scenario: Section interactions — all sections coexist
// =========================================================================

mod section_interactions {
    use super::*;

    #[test]
    fn given_all_sections_present_then_each_parsed_independently() {
        let toml_str = r#"
[scan]
hidden = true
no_ignore = true
no_ignore_parent = true
no_ignore_dot = true
no_ignore_vcs = true
doc_comments = true
paths = ["src"]
exclude = ["target"]

[module]
roots = ["crates"]
depth = 2
children = "collapse"

[export]
format = "jsonl"
min_code = 1
max_rows = 10000
redact = "paths"
children = "separate"

[analyze]
preset = "deep"
window = 256000
format = "json"
git = true
max_files = 5000
max_bytes = 50000000
max_file_bytes = 500000
max_commits = 1000
max_commit_files = 200
granularity = "module"

[context]
budget = "1m"
strategy = "greedy"
rank_by = "hotspot"
output = "bundle"
compress = true

[badge]
metric = "doc"

[gate]
fail_fast = false
allow_missing_baseline = true
allow_missing_current = false
policy = "my-policy.toml"
baseline = "baseline.json"
preset = "risk"

[[gate.rules]]
name = "rule-1"
pointer = "/summary/total_code"
op = "lt"
value = 100000

[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 3.0
level = "error"
description = "Keep complexity low"

[view.llm]
format = "json"
redact = "all"
top = 25
compress = true
preset = "deep"
window = 256000

[view.ci]
format = "tsv"
min_code = 1
max_rows = 50
"#;
        let config = TomlConfig::parse(toml_str).unwrap();

        // Spot-check every section
        assert_eq!(config.scan.hidden, Some(true));
        assert_eq!(config.scan.doc_comments, Some(true));
        assert_eq!(config.scan.exclude, Some(vec!["target".to_string()]));

        assert_eq!(config.module.depth, Some(2));
        assert_eq!(config.module.children, Some("collapse".to_string()));

        assert_eq!(config.export.format, Some("jsonl".to_string()));
        assert_eq!(config.export.children, Some("separate".to_string()));

        assert_eq!(config.analyze.preset, Some("deep".to_string()));
        assert_eq!(config.analyze.window, Some(256000));
        assert_eq!(config.analyze.granularity, Some("module".to_string()));
        assert_eq!(config.analyze.max_file_bytes, Some(500000));

        assert_eq!(config.context.budget, Some("1m".to_string()));
        assert_eq!(config.context.rank_by, Some("hotspot".to_string()));

        assert_eq!(config.badge.metric, Some("doc".to_string()));

        assert_eq!(config.gate.fail_fast, Some(false));
        assert_eq!(config.gate.allow_missing_baseline, Some(true));
        assert_eq!(config.gate.allow_missing_current, Some(false));
        assert_eq!(config.gate.policy, Some("my-policy.toml".to_string()));
        assert_eq!(config.gate.baseline, Some("baseline.json".to_string()));
        assert_eq!(config.gate.preset, Some("risk".to_string()));
        assert_eq!(config.gate.rules.as_ref().map(|r| r.len()), Some(1));
        assert_eq!(config.gate.ratchet.as_ref().map(|r| r.len()), Some(1));

        assert_eq!(config.view.len(), 2);
        assert_eq!(config.view.get("llm").unwrap().compress, Some(true));
        assert_eq!(config.view.get("ci").unwrap().max_rows, Some(50));
    }

    #[test]
    fn given_all_sections_when_roundtripped_through_json_then_values_preserved() {
        let toml_str = r#"
[scan]
hidden = true
paths = ["src", "tests"]

[module]
depth = 3
roots = ["crates"]

[export]
min_code = 5

[analyze]
preset = "health"
window = 128000

[context]
budget = "64k"

[badge]
metric = "lines"

[gate]
fail_fast = true

[view.ci]
format = "tsv"
top = 20
"#;
        let config1 = TomlConfig::parse(toml_str).unwrap();
        let json = serde_json::to_string(&config1).unwrap();
        let config2: TomlConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config1.scan.hidden, config2.scan.hidden);
        assert_eq!(config1.scan.paths, config2.scan.paths);
        assert_eq!(config1.module.depth, config2.module.depth);
        assert_eq!(config1.module.roots, config2.module.roots);
        assert_eq!(config1.export.min_code, config2.export.min_code);
        assert_eq!(config1.analyze.preset, config2.analyze.preset);
        assert_eq!(config1.analyze.window, config2.analyze.window);
        assert_eq!(config1.context.budget, config2.context.budget);
        assert_eq!(config1.badge.metric, config2.badge.metric);
        assert_eq!(config1.gate.fail_fast, config2.gate.fail_fast);
        assert_eq!(
            config1.view.get("ci").unwrap().format,
            config2.view.get("ci").unwrap().format
        );
        assert_eq!(
            config1.view.get("ci").unwrap().top,
            config2.view.get("ci").unwrap().top
        );
    }
}

// =========================================================================
// Scenario: CLI subcommand defaults with TOML config interaction
// =========================================================================

mod cli_toml_interaction {
    use super::*;
    use clap::Parser;
    use tokmd_config::Cli;

    #[test]
    fn given_toml_with_view_profile_when_cli_uses_profile_flag_then_name_captured() {
        // This tests that the --profile flag correctly captures a profile name
        // that can be looked up in TomlConfig
        let toml_str = r#"
[view.my_profile]
format = "json"
top = 10
redact = "all"
"#;
        let config = TomlConfig::parse(toml_str).unwrap();
        let cli = Cli::try_parse_from(["tokmd", "--profile", "my_profile"]).unwrap();

        let profile_name = cli.profile.as_deref().unwrap();
        let profile = config.view.get(profile_name).unwrap();
        assert_eq!(profile.format, Some("json".to_string()));
        assert_eq!(profile.top, Some(10));
        assert_eq!(profile.redact, Some("all".to_string()));
    }

    #[test]
    fn given_toml_with_multiple_profiles_when_cli_selects_one_then_correct_profile_used() {
        let toml_str = r#"
[view.quick]
format = "md"
top = 5

[view.deep]
format = "json"
top = 100
preset = "deep"
"#;
        let config = TomlConfig::parse(toml_str).unwrap();

        let cli_quick = Cli::try_parse_from(["tokmd", "--profile", "quick"]).unwrap();
        let cli_deep = Cli::try_parse_from(["tokmd", "--profile", "deep"]).unwrap();

        let quick = config
            .view
            .get(cli_quick.profile.as_deref().unwrap())
            .unwrap();
        let deep = config
            .view
            .get(cli_deep.profile.as_deref().unwrap())
            .unwrap();

        assert_eq!(quick.format, Some("md".to_string()));
        assert_eq!(quick.top, Some(5));
        assert_eq!(deep.format, Some("json".to_string()));
        assert_eq!(deep.top, Some(100));
    }

    #[test]
    fn given_toml_config_when_cli_specifies_nonexistent_profile_then_lookup_returns_none() {
        let config = TomlConfig::parse("[view.exists]\nformat = \"json\"\n").unwrap();
        let cli = Cli::try_parse_from(["tokmd", "--profile", "nonexistent"]).unwrap();

        let result = config.view.get(cli.profile.as_deref().unwrap());
        assert!(result.is_none());
    }
}

// =========================================================================
// Scenario: TOML type strictness
// =========================================================================

mod toml_type_strictness {
    use super::*;

    #[test]
    fn given_integer_where_boolean_expected_then_error() {
        assert!(TomlConfig::parse("[scan]\nhidden = 1").is_err());
    }

    #[test]
    fn given_float_where_boolean_expected_then_error() {
        assert!(TomlConfig::parse("[scan]\nhidden = 1.0").is_err());
    }

    #[test]
    fn given_object_where_string_expected_then_error() {
        assert!(TomlConfig::parse("[context]\nbudget = { value = 128 }").is_err());
    }

    #[test]
    fn given_boolean_where_string_expected_then_error() {
        assert!(TomlConfig::parse("[export]\nformat = true").is_err());
    }

    #[test]
    fn given_boolean_where_array_expected_then_error() {
        assert!(TomlConfig::parse("[module]\nroots = true").is_err());
    }

    #[test]
    fn given_integer_where_string_expected_then_error() {
        assert!(TomlConfig::parse("[context]\nbudget = 128").is_err());
    }
}
