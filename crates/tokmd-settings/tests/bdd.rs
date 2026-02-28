//! BDD-style scenario tests for tokmd-settings.
//!
//! Each test follows Given / When / Then structure to verify
//! settings behavior for common configuration scenarios.

use std::io::Write;
use tempfile::NamedTempFile;
use tokmd_settings::*;

// =============================================================================
// Scenario: Default settings produce valid, expected values
// =============================================================================

mod defaults {
    use super::*;

    #[test]
    fn scan_options_default_is_permissive() {
        // Given: no configuration
        // When: we construct default ScanOptions
        let opts = ScanOptions::default();
        // Then: all exclusions are empty and ignore flags are off
        assert!(opts.excluded.is_empty());
        assert!(!opts.hidden);
        assert!(!opts.no_ignore);
        assert!(!opts.no_ignore_parent);
        assert!(!opts.no_ignore_dot);
        assert!(!opts.no_ignore_vcs);
        assert!(!opts.treat_doc_strings_as_comments);
    }

    #[test]
    fn scan_settings_default_has_no_paths() {
        // Given/When: default ScanSettings
        let s = ScanSettings::default();
        // Then: paths list is empty (not pre-filled)
        assert!(s.paths.is_empty());
    }

    #[test]
    fn lang_settings_default_shows_all_rows_collapsed() {
        // Given/When: default LangSettings
        let s = LangSettings::default();
        // Then: top=0 means all rows, files off, collapse children, no redaction
        assert_eq!(s.top, 0);
        assert!(!s.files);
        assert!(matches!(s.children, ChildrenMode::Collapse));
        assert!(s.redact.is_none());
    }

    #[test]
    fn module_settings_default_has_standard_roots() {
        // Given/When: default ModuleSettings
        let s = ModuleSettings::default();
        // Then: roots include "crates" and "packages", depth=2, separate children
        assert_eq!(s.module_roots, vec!["crates", "packages"]);
        assert_eq!(s.module_depth, 2);
        assert!(matches!(s.children, ChildIncludeMode::Separate));
        assert!(s.redact.is_none());
        assert_eq!(s.top, 0);
    }

    #[test]
    fn export_settings_default_is_jsonl_with_meta() {
        // Given/When: default ExportSettings
        let s = ExportSettings::default();
        // Then: format is JSONL, meta enabled, no redaction, no limits
        assert!(matches!(s.format, ExportFormat::Jsonl));
        assert!(s.meta);
        assert!(matches!(s.redact, RedactMode::None));
        assert_eq!(s.min_code, 0);
        assert_eq!(s.max_rows, 0);
        assert!(s.strip_prefix.is_none());
    }

    #[test]
    fn analyze_settings_default_is_receipt_preset() {
        // Given/When: default AnalyzeSettings
        let s = AnalyzeSettings::default();
        // Then: preset is "receipt", granularity is "module", all limits are None
        assert_eq!(s.preset, "receipt");
        assert_eq!(s.granularity, "module");
        assert!(s.window.is_none());
        assert!(s.git.is_none());
        assert!(s.max_files.is_none());
        assert!(s.max_bytes.is_none());
        assert!(s.max_file_bytes.is_none());
        assert!(s.max_commits.is_none());
        assert!(s.max_commit_files.is_none());
    }

    #[test]
    fn cockpit_settings_default_targets_main_to_head() {
        // Given/When: default CockpitSettings
        let s = CockpitSettings::default();
        // Then: base=main, head=HEAD, two-dot mode, no baseline
        assert_eq!(s.base, "main");
        assert_eq!(s.head, "HEAD");
        assert_eq!(s.range_mode, "two-dot");
        assert!(s.baseline.is_none());
    }

    #[test]
    fn diff_settings_default_has_empty_refs() {
        // Given/When: default DiffSettings
        let s = DiffSettings::default();
        // Then: both refs are empty strings
        assert!(s.from.is_empty());
        assert!(s.to.is_empty());
    }

    #[test]
    fn toml_config_default_has_empty_sections() {
        // Given/When: default TomlConfig
        let c = TomlConfig::default();
        // Then: all sections are default (None-valued) and view map is empty
        assert!(c.scan.paths.is_none());
        assert!(c.scan.exclude.is_none());
        assert!(c.module.roots.is_none());
        assert!(c.export.min_code.is_none());
        assert!(c.analyze.preset.is_none());
        assert!(c.context.budget.is_none());
        assert!(c.badge.metric.is_none());
        assert!(c.gate.policy.is_none());
        assert!(c.view.is_empty());
    }
}

// =============================================================================
// Scenario: Constructing ScanSettings via helper methods
// =============================================================================

mod scan_constructors {
    use super::*;

    #[test]
    fn current_dir_scans_dot() {
        // Given: user wants to scan current directory
        // When: using the convenience constructor
        let s = ScanSettings::current_dir();
        // Then: paths=["."], options are default
        assert_eq!(s.paths, vec!["."]);
        assert!(!s.options.hidden);
    }

    #[test]
    fn for_paths_accepts_multiple_dirs() {
        // Given: user wants to scan specific directories
        let dirs = vec!["src".to_string(), "lib".to_string(), "tests".to_string()];
        // When: constructing with for_paths
        let s = ScanSettings::for_paths(dirs.clone());
        // Then: all paths are preserved, options are default
        assert_eq!(s.paths, dirs);
        assert!(s.options.excluded.is_empty());
    }

    #[test]
    fn for_paths_with_empty_list() {
        // Given: an empty path list
        // When: constructing with for_paths
        let s = ScanSettings::for_paths(vec![]);
        // Then: paths is empty
        assert!(s.paths.is_empty());
    }
}

// =============================================================================
// Scenario: JSON serde round-trips preserve all fields
// =============================================================================

mod json_roundtrip {
    use super::*;

    #[test]
    fn scan_options_all_flags_enabled() {
        // Given: ScanOptions with every flag turned on
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
        // When: serialized to JSON and deserialized back
        let json = serde_json::to_string(&opts).unwrap();
        let back: ScanOptions = serde_json::from_str(&json).unwrap();
        // Then: all fields match
        assert_eq!(back.excluded, opts.excluded);
        assert!(back.hidden);
        assert!(back.no_ignore);
        assert!(back.no_ignore_parent);
        assert!(back.no_ignore_dot);
        assert!(back.no_ignore_vcs);
        assert!(back.treat_doc_strings_as_comments);
    }

    #[test]
    fn lang_settings_with_redaction() {
        // Given: LangSettings with all options set
        let s = LangSettings {
            top: 5,
            files: true,
            children: ChildrenMode::Separate,
            redact: Some(RedactMode::All),
        };
        // When: round-tripped through JSON
        let json = serde_json::to_string(&s).unwrap();
        let back: LangSettings = serde_json::from_str(&json).unwrap();
        // Then: all fields preserved
        assert_eq!(back.top, 5);
        assert!(back.files);
        assert!(matches!(back.redact, Some(RedactMode::All)));
    }

    #[test]
    fn module_settings_custom_roots() {
        // Given: ModuleSettings with custom roots
        let s = ModuleSettings {
            top: 3,
            module_roots: vec!["apps".into(), "libs".into()],
            module_depth: 4,
            children: ChildIncludeMode::ParentsOnly,
            redact: Some(RedactMode::Paths),
        };
        // When: round-tripped
        let json = serde_json::to_string(&s).unwrap();
        let back: ModuleSettings = serde_json::from_str(&json).unwrap();
        // Then: custom values preserved
        assert_eq!(back.top, 3);
        assert_eq!(back.module_roots, vec!["apps", "libs"]);
        assert_eq!(back.module_depth, 4);
        assert!(matches!(back.children, ChildIncludeMode::ParentsOnly));
        assert!(matches!(back.redact, Some(RedactMode::Paths)));
    }

    #[test]
    fn export_settings_csv_with_limits() {
        // Given: ExportSettings for CSV with limits
        let s = ExportSettings {
            format: ExportFormat::Csv,
            module_roots: vec!["src".into()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 10,
            max_rows: 500,
            redact: RedactMode::Paths,
            meta: false,
            strip_prefix: Some("project/".into()),
        };
        // When: round-tripped
        let json = serde_json::to_string(&s).unwrap();
        let back: ExportSettings = serde_json::from_str(&json).unwrap();
        // Then: all fields preserved
        assert!(matches!(back.format, ExportFormat::Csv));
        assert_eq!(back.min_code, 10);
        assert_eq!(back.max_rows, 500);
        assert!(!back.meta);
        assert_eq!(back.strip_prefix, Some("project/".into()));
    }

    #[test]
    fn analyze_settings_with_all_limits() {
        // Given: AnalyzeSettings with every limit set
        let s = AnalyzeSettings {
            preset: "deep".into(),
            window: Some(128_000),
            git: Some(true),
            max_files: Some(10_000),
            max_bytes: Some(100_000_000),
            max_file_bytes: Some(1_000_000),
            max_commits: Some(500),
            max_commit_files: Some(50),
            granularity: "file".into(),
        };
        // When: round-tripped
        let json = serde_json::to_string(&s).unwrap();
        let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
        // Then: all values preserved
        assert_eq!(back.preset, "deep");
        assert_eq!(back.window, Some(128_000));
        assert_eq!(back.git, Some(true));
        assert_eq!(back.max_files, Some(10_000));
        assert_eq!(back.max_bytes, Some(100_000_000));
        assert_eq!(back.max_file_bytes, Some(1_000_000));
        assert_eq!(back.max_commits, Some(500));
        assert_eq!(back.max_commit_files, Some(50));
        assert_eq!(back.granularity, "file");
    }

    #[test]
    fn cockpit_settings_three_dot_with_baseline() {
        // Given: CockpitSettings for a PR comparison
        let s = CockpitSettings {
            base: "v2.0.0".into(),
            head: "feature/new-api".into(),
            range_mode: "three-dot".into(),
            baseline: Some("baselines/v2.json".into()),
        };
        // When: round-tripped
        let json = serde_json::to_string(&s).unwrap();
        let back: CockpitSettings = serde_json::from_str(&json).unwrap();
        // Then: all values preserved
        assert_eq!(back.base, "v2.0.0");
        assert_eq!(back.head, "feature/new-api");
        assert_eq!(back.range_mode, "three-dot");
        assert_eq!(back.baseline.as_deref(), Some("baselines/v2.json"));
    }

    #[test]
    fn diff_settings_roundtrip() {
        // Given: DiffSettings with tag refs
        let s = DiffSettings {
            from: "v1.0.0".into(),
            to: "v2.0.0".into(),
        };
        // When: round-tripped
        let json = serde_json::to_string(&s).unwrap();
        let back: DiffSettings = serde_json::from_str(&json).unwrap();
        // Then: preserved
        assert_eq!(back.from, "v1.0.0");
        assert_eq!(back.to, "v2.0.0");
    }

    #[test]
    fn gate_rule_roundtrip() {
        // Given: a gate rule with all fields
        let rule = GateRule {
            name: "max-files".into(),
            pointer: "/summary/total_files".into(),
            op: "<=".into(),
            value: Some(serde_json::json!(1000)),
            values: None,
            negate: false,
            level: Some("error".into()),
            message: Some("Too many files".into()),
        };
        // When: round-tripped
        let json = serde_json::to_string(&rule).unwrap();
        let back: GateRule = serde_json::from_str(&json).unwrap();
        // Then: preserved
        assert_eq!(back.name, "max-files");
        assert_eq!(back.pointer, "/summary/total_files");
        assert_eq!(back.op, "<=");
        assert_eq!(back.value, Some(serde_json::json!(1000)));
        assert!(!back.negate);
        assert_eq!(back.level.as_deref(), Some("error"));
        assert_eq!(back.message.as_deref(), Some("Too many files"));
    }

    #[test]
    fn gate_rule_with_in_operator() {
        // Given: a gate rule using the "in" operator with multiple values
        let rule = GateRule {
            name: "language-check".into(),
            pointer: "/languages/0/name".into(),
            op: "in".into(),
            value: None,
            values: Some(vec![serde_json::json!("Rust"), serde_json::json!("Python")]),
            negate: true,
            level: Some("warn".into()),
            message: None,
        };
        // When: round-tripped
        let json = serde_json::to_string(&rule).unwrap();
        let back: GateRule = serde_json::from_str(&json).unwrap();
        // Then: preserved
        assert!(back.negate);
        assert_eq!(back.values.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn ratchet_rule_config_roundtrip() {
        // Given: a ratchet rule with all fields
        let rule = RatchetRuleConfig {
            pointer: "/complexity/avg_cyclomatic".into(),
            max_increase_pct: Some(5.0),
            max_value: Some(25.0),
            level: Some("error".into()),
            description: Some("Cyclomatic complexity must not spike".into()),
        };
        // When: round-tripped
        let json = serde_json::to_string(&rule).unwrap();
        let back: RatchetRuleConfig = serde_json::from_str(&json).unwrap();
        // Then: preserved
        assert_eq!(back.pointer, "/complexity/avg_cyclomatic");
        assert!((back.max_increase_pct.unwrap() - 5.0).abs() < f64::EPSILON);
        assert!((back.max_value.unwrap() - 25.0).abs() < f64::EPSILON);
        assert_eq!(back.level.as_deref(), Some("error"));
    }

    #[test]
    fn view_profile_roundtrip() {
        // Given: a fully populated ViewProfile
        let vp = ViewProfile {
            format: Some("json".into()),
            top: Some(20),
            files: Some(true),
            module_roots: Some(vec!["src".into()]),
            module_depth: Some(3),
            min_code: Some(5),
            max_rows: Some(100),
            redact: Some("paths".into()),
            meta: Some(false),
            children: Some("collapse".into()),
            preset: Some("health".into()),
            window: Some(128_000),
            budget: Some("100k".into()),
            strategy: Some("greedy".into()),
            rank_by: Some("churn".into()),
            output: Some("bundle".into()),
            compress: Some(true),
            metric: Some("loc".into()),
        };
        // When: round-tripped
        let json = serde_json::to_string(&vp).unwrap();
        let back: ViewProfile = serde_json::from_str(&json).unwrap();
        // Then: all fields preserved
        assert_eq!(back.format.as_deref(), Some("json"));
        assert_eq!(back.top, Some(20));
        assert_eq!(back.files, Some(true));
        assert_eq!(
            back.module_roots.as_ref().unwrap(),
            &vec!["src".to_string()]
        );
        assert_eq!(back.module_depth, Some(3));
        assert_eq!(back.min_code, Some(5));
        assert_eq!(back.max_rows, Some(100));
        assert_eq!(back.compress, Some(true));
        assert_eq!(back.metric.as_deref(), Some("loc"));
    }
}

// =============================================================================
// Scenario: TOML configuration parsing
// =============================================================================

mod toml_parsing {
    use super::*;

    #[test]
    fn empty_toml_produces_defaults() {
        // Given: an empty TOML string
        // When: parsed
        let config = TomlConfig::parse("").unwrap();
        // Then: all sections use defaults
        assert!(config.scan.hidden.is_none());
        assert!(config.view.is_empty());
    }

    #[test]
    fn full_scan_section() {
        // Given: a TOML with all scan fields
        let toml = r#"
[scan]
paths = ["src", "lib"]
exclude = ["target", "*.bak"]
hidden = true
config = "none"
no_ignore = true
no_ignore_parent = false
no_ignore_dot = true
no_ignore_vcs = false
doc_comments = true
"#;
        // When: parsed
        let config = TomlConfig::parse(toml).unwrap();
        // Then: all fields are present and correct
        assert_eq!(config.scan.paths, Some(vec!["src".into(), "lib".into()]));
        assert_eq!(
            config.scan.exclude,
            Some(vec!["target".into(), "*.bak".into()])
        );
        assert_eq!(config.scan.hidden, Some(true));
        assert_eq!(config.scan.config, Some("none".into()));
        assert_eq!(config.scan.no_ignore, Some(true));
        assert_eq!(config.scan.no_ignore_parent, Some(false));
        assert_eq!(config.scan.no_ignore_dot, Some(true));
        assert_eq!(config.scan.no_ignore_vcs, Some(false));
        assert_eq!(config.scan.doc_comments, Some(true));
    }

    #[test]
    fn module_section() {
        // Given: TOML with module settings
        let toml = r#"
[module]
roots = ["apps", "packages"]
depth = 3
children = "collapse"
"#;
        // When: parsed
        let config = TomlConfig::parse(toml).unwrap();
        // Then: all module fields correct
        assert_eq!(
            config.module.roots,
            Some(vec!["apps".into(), "packages".into()])
        );
        assert_eq!(config.module.depth, Some(3));
        assert_eq!(config.module.children, Some("collapse".into()));
    }

    #[test]
    fn export_section() {
        // Given: TOML with export settings
        let toml = r#"
[export]
min_code = 10
max_rows = 500
redact = "paths"
format = "csv"
children = "separate"
"#;
        // When: parsed
        let config = TomlConfig::parse(toml).unwrap();
        // Then: fields correct
        assert_eq!(config.export.min_code, Some(10));
        assert_eq!(config.export.max_rows, Some(500));
        assert_eq!(config.export.redact, Some("paths".into()));
        assert_eq!(config.export.format, Some("csv".into()));
    }

    #[test]
    fn analyze_section_with_limits() {
        // Given: TOML with analyze limits
        let toml = r#"
[analyze]
preset = "deep"
window = 128000
git = true
max_files = 5000
max_bytes = 50000000
max_file_bytes = 500000
max_commits = 200
max_commit_files = 30
granularity = "file"
format = "json"
"#;
        // When: parsed
        let config = TomlConfig::parse(toml).unwrap();
        // Then: all fields present
        assert_eq!(config.analyze.preset, Some("deep".into()));
        assert_eq!(config.analyze.window, Some(128_000));
        assert_eq!(config.analyze.git, Some(true));
        assert_eq!(config.analyze.max_files, Some(5000));
        assert_eq!(config.analyze.max_bytes, Some(50_000_000));
        assert_eq!(config.analyze.max_file_bytes, Some(500_000));
        assert_eq!(config.analyze.max_commits, Some(200));
        assert_eq!(config.analyze.max_commit_files, Some(30));
        assert_eq!(config.analyze.granularity, Some("file".into()));
    }

    #[test]
    fn context_section() {
        // Given: TOML with context settings
        let toml = r#"
[context]
budget = "100k"
strategy = "spread"
rank_by = "hotspot"
output = "bundle"
compress = true
"#;
        // When: parsed
        let config = TomlConfig::parse(toml).unwrap();
        // Then: all fields present
        assert_eq!(config.context.budget, Some("100k".into()));
        assert_eq!(config.context.strategy, Some("spread".into()));
        assert_eq!(config.context.rank_by, Some("hotspot".into()));
        assert_eq!(config.context.output, Some("bundle".into()));
        assert_eq!(config.context.compress, Some(true));
    }

    #[test]
    fn badge_section() {
        // Given: TOML with badge settings
        let toml = r#"
[badge]
metric = "languages"
"#;
        // When: parsed
        let config = TomlConfig::parse(toml).unwrap();
        // Then: metric is set
        assert_eq!(config.badge.metric, Some("languages".into()));
    }

    #[test]
    fn gate_section_with_inline_rules() {
        // Given: TOML with gate rules
        let toml = r#"
[gate]
policy = "policy.json"
baseline = "baseline.json"
preset = "health"
fail_fast = true
allow_missing_baseline = true
allow_missing_current = false

[[gate.rules]]
name = "max-files"
pointer = "/summary/total_files"
op = "<="
value = 1000

[[gate.ratchet]]
pointer = "/complexity/avg"
max_increase_pct = 5.0
max_value = 20.0
level = "warn"
description = "Keep complexity low"
"#;
        // When: parsed
        let config = TomlConfig::parse(toml).unwrap();
        // Then: gate fields and inline rules present
        assert_eq!(config.gate.policy, Some("policy.json".into()));
        assert_eq!(config.gate.baseline, Some("baseline.json".into()));
        assert_eq!(config.gate.preset, Some("health".into()));
        assert_eq!(config.gate.fail_fast, Some(true));
        assert_eq!(config.gate.allow_missing_baseline, Some(true));
        assert_eq!(config.gate.allow_missing_current, Some(false));

        let rules = config.gate.rules.unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "max-files");
        assert_eq!(rules[0].op, "<=");

        let ratchet = config.gate.ratchet.unwrap();
        assert_eq!(ratchet.len(), 1);
        assert_eq!(ratchet[0].pointer, "/complexity/avg");
        assert!((ratchet[0].max_increase_pct.unwrap() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn multiple_view_profiles() {
        // Given: TOML with multiple named view profiles
        let toml = r#"
[view.llm]
format = "json"
top = 10
budget = "100k"

[view.ci]
format = "tsv"
preset = "health"
compress = true

[view.minimal]
top = 5
"#;
        // When: parsed
        let config = TomlConfig::parse(toml).unwrap();
        // Then: all three profiles exist
        assert_eq!(config.view.len(), 3);

        let llm = config.view.get("llm").unwrap();
        assert_eq!(llm.format.as_deref(), Some("json"));
        assert_eq!(llm.top, Some(10));
        assert_eq!(llm.budget.as_deref(), Some("100k"));

        let ci = config.view.get("ci").unwrap();
        assert_eq!(ci.format.as_deref(), Some("tsv"));
        assert_eq!(ci.preset.as_deref(), Some("health"));
        assert_eq!(ci.compress, Some(true));

        let minimal = config.view.get("minimal").unwrap();
        assert_eq!(minimal.top, Some(5));
        assert!(minimal.format.is_none());
    }

    #[test]
    fn from_file_reads_disk() {
        // Given: a TOML file on disk
        let content = r#"
[scan]
hidden = true
exclude = ["vendor"]

[module]
depth = 4
"#;
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();

        // When: loaded from file
        let config = TomlConfig::from_file(f.path()).unwrap();

        // Then: values are correct
        assert_eq!(config.scan.hidden, Some(true));
        assert_eq!(config.scan.exclude, Some(vec!["vendor".into()]));
        assert_eq!(config.module.depth, Some(4));
    }

    #[test]
    fn from_file_nonexistent_returns_error() {
        // Given: a path that does not exist
        let path = std::path::Path::new("/nonexistent/tokmd.toml");
        // When: attempting to load
        let result = TomlConfig::from_file(path);
        // Then: we get an IO error
        assert!(result.is_err());
    }

    #[test]
    fn invalid_toml_returns_parse_error() {
        // Given: malformed TOML
        let bad_toml = "[scan\nhidden = true";
        // When: parsing
        let result = TomlConfig::parse(bad_toml);
        // Then: we get a parse error
        assert!(result.is_err());
    }
}

// =============================================================================
// Scenario: Deserialization from partial / minimal JSON
// =============================================================================

mod partial_deserialization {
    use super::*;

    #[test]
    fn scan_options_from_empty_json() {
        // Given: an empty JSON object
        // When: deserialized as ScanOptions
        let opts: ScanOptions = serde_json::from_str("{}").unwrap();
        // Then: defaults are applied
        assert!(opts.excluded.is_empty());
        assert!(!opts.hidden);
    }

    #[test]
    fn lang_settings_from_minimal_json() {
        // Given: JSON with only 'top' set
        let json = r#"{"top": 5}"#;
        // When: deserialized
        let s: LangSettings = serde_json::from_str(json).unwrap();
        // Then: top is 5, rest are defaults
        assert_eq!(s.top, 5);
        assert!(!s.files);
        assert!(matches!(s.children, ChildrenMode::Collapse));
    }

    #[test]
    fn export_settings_format_only() {
        // Given: JSON with only format specified
        let json = r#"{"format": "csv"}"#;
        // When: deserialized
        let s: ExportSettings = serde_json::from_str(json).unwrap();
        // Then: format is CSV, rest are defaults
        assert!(matches!(s.format, ExportFormat::Csv));
        assert!(s.meta); // default_meta() = true
        assert_eq!(s.module_depth, 2);
    }

    #[test]
    fn analyze_settings_partial() {
        // Given: JSON with only some limits
        let json = r#"{"preset": "risk", "max_commits": 100}"#;
        // When: deserialized
        let s: AnalyzeSettings = serde_json::from_str(json).unwrap();
        // Then: specified fields set, rest default
        assert_eq!(s.preset, "risk");
        assert_eq!(s.max_commits, Some(100));
        assert!(s.max_files.is_none());
        assert_eq!(s.granularity, "module");
    }

    #[test]
    fn scan_settings_flatten_from_json() {
        // Given: JSON with flattened scan options
        let json = r#"{"paths": ["src"], "hidden": true, "excluded": ["*.log"]}"#;
        // When: deserialized as ScanSettings (options are flattened)
        let s: ScanSettings = serde_json::from_str(json).unwrap();
        // Then: paths and flattened options both work
        assert_eq!(s.paths, vec!["src"]);
        assert!(s.options.hidden);
        assert_eq!(s.options.excluded, vec!["*.log"]);
    }
}

// =============================================================================
// Scenario: Edge cases with special characters and extreme values
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn paths_with_unicode() {
        // Given: paths containing unicode characters
        let s = ScanSettings::for_paths(vec!["src/日本語".into(), "lib/données".into()]);
        // When: round-tripped
        let json = serde_json::to_string(&s).unwrap();
        let back: ScanSettings = serde_json::from_str(&json).unwrap();
        // Then: unicode preserved
        assert_eq!(back.paths[0], "src/日本語");
        assert_eq!(back.paths[1], "lib/données");
    }

    #[test]
    fn paths_with_special_characters() {
        // Given: paths with spaces, dots, and dashes
        let s = ScanSettings::for_paths(vec![
            "my project/src".into(),
            "../parent-dir".into(),
            "./relative.path".into(),
        ]);
        // When: round-tripped
        let json = serde_json::to_string(&s).unwrap();
        let back: ScanSettings = serde_json::from_str(&json).unwrap();
        // Then: all preserved
        assert_eq!(back.paths.len(), 3);
        assert_eq!(back.paths[0], "my project/src");
    }

    #[test]
    fn empty_string_fields() {
        // Given: settings with empty strings
        let s = DiffSettings {
            from: "".into(),
            to: "".into(),
        };
        // When: round-tripped
        let json = serde_json::to_string(&s).unwrap();
        let back: DiffSettings = serde_json::from_str(&json).unwrap();
        // Then: empty strings preserved
        assert!(back.from.is_empty());
        assert!(back.to.is_empty());
    }

    #[test]
    fn large_numeric_values() {
        // Given: AnalyzeSettings with very large values
        let s = AnalyzeSettings {
            max_bytes: Some(u64::MAX),
            max_file_bytes: Some(u64::MAX),
            max_files: Some(usize::MAX),
            max_commits: Some(usize::MAX),
            max_commit_files: Some(usize::MAX),
            window: Some(usize::MAX),
            ..Default::default()
        };
        // When: round-tripped
        let json = serde_json::to_string(&s).unwrap();
        let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
        // Then: values preserved
        assert_eq!(back.max_bytes, Some(u64::MAX));
        assert_eq!(back.max_files, Some(usize::MAX));
    }

    #[test]
    fn exclusion_patterns_with_globs() {
        // Given: complex glob patterns in exclusions
        let opts = ScanOptions {
            excluded: vec![
                "**/*.min.js".into(),
                "vendor/**".into(),
                "*.{bak,tmp,swp}".into(),
                "[Bb]uild/".into(),
            ],
            ..Default::default()
        };
        // When: round-tripped
        let json = serde_json::to_string(&opts).unwrap();
        let back: ScanOptions = serde_json::from_str(&json).unwrap();
        // Then: all glob patterns preserved exactly
        assert_eq!(back.excluded.len(), 4);
        assert_eq!(back.excluded[0], "**/*.min.js");
        assert_eq!(back.excluded[2], "*.{bak,tmp,swp}");
    }

    #[test]
    fn gate_rule_with_json_pointer_special_chars() {
        // Given: a gate rule with RFC 6901 escaped JSON pointer
        let rule = GateRule {
            name: "special-check".into(),
            pointer: "/a~1b/c~0d".into(), // ~ and / escaping per RFC 6901
            op: "==".into(),
            value: Some(serde_json::json!("test")),
            values: None,
            negate: false,
            level: None,
            message: None,
        };
        // When: round-tripped
        let json = serde_json::to_string(&rule).unwrap();
        let back: GateRule = serde_json::from_str(&json).unwrap();
        // Then: pointer preserved
        assert_eq!(back.pointer, "/a~1b/c~0d");
    }

    #[test]
    fn toml_config_roundtrip_through_serialization() {
        // Given: a populated TomlConfig
        let config = TomlConfig {
            scan: ScanConfig {
                hidden: Some(true),
                ..Default::default()
            },
            module: ModuleConfig {
                depth: Some(3),
                ..Default::default()
            },
            ..Default::default()
        };
        // When: serialized to TOML and parsed back
        let toml_str = toml::to_string(&config).unwrap();
        let back = TomlConfig::parse(&toml_str).unwrap();
        // Then: values preserved
        assert_eq!(back.scan.hidden, Some(true));
        assert_eq!(back.module.depth, Some(3));
    }
}

// =============================================================================
// Scenario: Clone and Debug trait verification
// =============================================================================

mod trait_impls {
    use super::*;

    #[test]
    fn all_types_are_cloneable() {
        let scan_opts = ScanOptions::default();
        let _ = scan_opts.clone();

        let scan_settings = ScanSettings::current_dir();
        let _ = scan_settings.clone();

        let lang = LangSettings::default();
        let _ = lang.clone();

        let module = ModuleSettings::default();
        let _ = module.clone();

        let export = ExportSettings::default();
        let _ = export.clone();

        let analyze = AnalyzeSettings::default();
        let _ = analyze.clone();

        let cockpit = CockpitSettings::default();
        let _ = cockpit.clone();

        let diff = DiffSettings::default();
        let _ = diff.clone();

        let toml = TomlConfig::default();
        let _ = toml.clone();

        let vp = ViewProfile::default();
        let _ = vp.clone();
    }

    #[test]
    fn all_types_are_debuggable() {
        // Verify Debug formatting doesn't panic
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
        let _ = format!("{:?}", ScanConfig::default());
        let _ = format!("{:?}", ModuleConfig::default());
        let _ = format!("{:?}", ExportConfig::default());
        let _ = format!("{:?}", AnalyzeConfig::default());
        let _ = format!("{:?}", ContextConfig::default());
        let _ = format!("{:?}", BadgeConfig::default());
        let _ = format!("{:?}", GateConfig::default());
    }
}
