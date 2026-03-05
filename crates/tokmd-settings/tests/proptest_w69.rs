//! W69 deep property-based tests for tokmd-settings.
//!
//! Covers ScanOptions/ScanSettings/LangSettings/ModuleSettings/ExportSettings
//! serde round-trips, default values, enum variant coverage, and builder patterns.

use proptest::prelude::*;
use tokmd_settings::{
    AnalyzeSettings, ChildIncludeMode, ChildrenMode, CockpitSettings, ConfigMode, DiffSettings,
    ExportFormat, ExportSettings, LangSettings, ModuleSettings, RedactMode, ScanOptions,
    ScanSettings, TomlConfig,
};

// =========================================================================
// 1. ScanOptions serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn scan_options_serde_roundtrip(
        hidden in proptest::bool::ANY,
        no_ignore in proptest::bool::ANY,
        no_ignore_parent in proptest::bool::ANY,
        no_ignore_dot in proptest::bool::ANY,
        no_ignore_vcs in proptest::bool::ANY,
        treat_doc in proptest::bool::ANY,
    ) {
        let opts = ScanOptions {
            excluded: vec!["*.log".into(), "target".into()],
            config: ConfigMode::Auto,
            hidden,
            no_ignore,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            treat_doc_strings_as_comments: treat_doc,
        };
        let json = serde_json::to_string(&opts).unwrap();
        let parsed: ScanOptions = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(opts.hidden, parsed.hidden);
        prop_assert_eq!(opts.no_ignore, parsed.no_ignore);
        prop_assert_eq!(opts.no_ignore_parent, parsed.no_ignore_parent);
        prop_assert_eq!(opts.no_ignore_dot, parsed.no_ignore_dot);
        prop_assert_eq!(opts.no_ignore_vcs, parsed.no_ignore_vcs);
        prop_assert_eq!(opts.treat_doc_strings_as_comments, parsed.treat_doc_strings_as_comments);
        prop_assert_eq!(opts.excluded, parsed.excluded);
    }
}

// =========================================================================
// 2. ScanOptions default values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn scan_options_defaults(_dummy in 0..1u8) {
        let opts = ScanOptions::default();
        prop_assert!(opts.excluded.is_empty());
        prop_assert!(!opts.hidden);
        prop_assert!(!opts.no_ignore);
        prop_assert!(!opts.no_ignore_parent);
        prop_assert!(!opts.no_ignore_dot);
        prop_assert!(!opts.no_ignore_vcs);
        prop_assert!(!opts.treat_doc_strings_as_comments);
    }
}

// =========================================================================
// 3. ScanSettings::current_dir has "." path
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn scan_settings_current_dir(_dummy in 0..1u8) {
        let s = ScanSettings::current_dir();
        prop_assert_eq!(s.paths, vec![".".to_string()]);
    }
}

// =========================================================================
// 4. ScanSettings::for_paths preserves paths
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn scan_settings_for_paths(paths in proptest::collection::vec("[a-z/]{1,20}", 1..5)) {
        let s = ScanSettings::for_paths(paths.clone());
        prop_assert_eq!(s.paths, paths);
    }
}

// =========================================================================
// 5. ScanSettings serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn scan_settings_serde_roundtrip(hidden in proptest::bool::ANY) {
        let s = ScanSettings {
            paths: vec!["src".into(), "lib".into()],
            options: ScanOptions {
                hidden,
                ..Default::default()
            },
        };
        let json = serde_json::to_string(&s).unwrap();
        let parsed: ScanSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s.paths, parsed.paths);
        prop_assert_eq!(s.options.hidden, parsed.options.hidden);
    }
}

// =========================================================================
// 6. LangSettings default values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn lang_settings_defaults(_dummy in 0..1u8) {
        let s = LangSettings::default();
        prop_assert_eq!(s.top, 0usize);
        prop_assert!(!s.files);
        prop_assert!(s.redact.is_none());
    }
}

// =========================================================================
// 7. LangSettings serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lang_settings_serde_roundtrip(
        top in 0usize..100,
        files in proptest::bool::ANY,
    ) {
        let s = LangSettings {
            top,
            files,
            children: ChildrenMode::Collapse,
            redact: None,
        };
        let json = serde_json::to_string(&s).unwrap();
        let parsed: LangSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s.top, parsed.top);
        prop_assert_eq!(s.files, parsed.files);
    }
}

// =========================================================================
// 8. ModuleSettings default values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn module_settings_defaults(_dummy in 0..1u8) {
        let s = ModuleSettings::default();
        prop_assert_eq!(s.top, 0usize);
        prop_assert_eq!(s.module_depth, 2usize);
        prop_assert_eq!(s.module_roots, vec!["crates".to_string(), "packages".to_string()]);
        prop_assert!(s.redact.is_none());
    }
}

// =========================================================================
// 9. ModuleSettings serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn module_settings_serde_roundtrip(
        top in 0usize..100,
        depth in 1usize..5,
    ) {
        let s = ModuleSettings {
            top,
            module_roots: vec!["src".into()],
            module_depth: depth,
            children: ChildIncludeMode::Separate,
            redact: None,
        };
        let json = serde_json::to_string(&s).unwrap();
        let parsed: ModuleSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s.top, parsed.top);
        prop_assert_eq!(s.module_depth, parsed.module_depth);
        prop_assert_eq!(s.module_roots, parsed.module_roots);
    }
}

// =========================================================================
// 10. ExportSettings default values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn export_settings_defaults(_dummy in 0..1u8) {
        let s = ExportSettings::default();
        prop_assert_eq!(s.min_code, 0usize);
        prop_assert_eq!(s.max_rows, 0usize);
        prop_assert!(s.meta);
        prop_assert!(s.strip_prefix.is_none());
    }
}

// =========================================================================
// 11. ExportSettings serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn export_settings_serde_roundtrip(
        min_code in 0usize..1000,
        max_rows in 0usize..5000,
        meta in proptest::bool::ANY,
    ) {
        let s = ExportSettings {
            format: ExportFormat::Jsonl,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            min_code,
            max_rows,
            redact: RedactMode::None,
            meta,
            strip_prefix: None,
        };
        let json = serde_json::to_string(&s).unwrap();
        let parsed: ExportSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s.min_code, parsed.min_code);
        prop_assert_eq!(s.max_rows, parsed.max_rows);
        prop_assert_eq!(s.meta, parsed.meta);
    }
}

// =========================================================================
// 12. AnalyzeSettings default values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn analyze_settings_defaults(_dummy in 0..1u8) {
        let s = AnalyzeSettings::default();
        prop_assert_eq!(s.preset, "receipt");
        prop_assert_eq!(s.granularity, "module");
        prop_assert!(s.window.is_none());
        prop_assert!(s.git.is_none());
        prop_assert!(s.max_files.is_none());
        prop_assert!(s.max_bytes.is_none());
        prop_assert!(s.max_file_bytes.is_none());
        prop_assert!(s.max_commits.is_none());
        prop_assert!(s.max_commit_files.is_none());
    }
}

// =========================================================================
// 13. AnalyzeSettings serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn analyze_settings_serde_roundtrip(
        max_files in proptest::option::of(1usize..10000),
        max_commits in proptest::option::of(1usize..5000),
    ) {
        let s = AnalyzeSettings {
            preset: "health".into(),
            window: Some(128000),
            git: Some(true),
            max_files,
            max_bytes: None,
            max_file_bytes: None,
            max_commits,
            max_commit_files: None,
            granularity: "file".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let parsed: AnalyzeSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s.preset, parsed.preset);
        prop_assert_eq!(s.max_files, parsed.max_files);
        prop_assert_eq!(s.max_commits, parsed.max_commits);
    }
}

// =========================================================================
// 14. ChildrenMode enum coverage
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn children_mode_variant_coverage(idx in 0usize..2) {
        let all = [ChildrenMode::Collapse, ChildrenMode::Separate];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: ChildrenMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 15. ChildIncludeMode enum coverage
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn child_include_mode_variant_coverage(idx in 0usize..2) {
        let all = [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: ChildIncludeMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 16. RedactMode enum coverage
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn redact_mode_variant_coverage(idx in 0usize..3) {
        let all = [RedactMode::None, RedactMode::Paths, RedactMode::All];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: RedactMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 17. CockpitSettings default values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn cockpit_settings_defaults(_dummy in 0..1u8) {
        let s = CockpitSettings::default();
        prop_assert_eq!(s.base, "main");
        prop_assert_eq!(s.head, "HEAD");
        prop_assert_eq!(s.range_mode, "two-dot");
        prop_assert!(s.baseline.is_none());
    }
}

// =========================================================================
// 18. DiffSettings serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn diff_settings_serde_roundtrip(
        from in "[a-z]{3,10}",
        to in "[a-z]{3,10}",
    ) {
        let s = DiffSettings { from: from.clone(), to: to.clone() };
        let json = serde_json::to_string(&s).unwrap();
        let parsed: DiffSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(from, parsed.from);
        prop_assert_eq!(to, parsed.to);
    }
}

// =========================================================================
// 19. TomlConfig::parse empty string yields defaults
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn toml_config_empty_string_defaults(_dummy in 0..1u8) {
        let config = TomlConfig::parse("").unwrap();
        prop_assert!(config.scan.paths.is_none());
        prop_assert!(config.scan.exclude.is_none());
        prop_assert!(config.scan.hidden.is_none());
        prop_assert!(config.module.roots.is_none());
        prop_assert!(config.analyze.preset.is_none());
    }
}

// =========================================================================
// 20. TomlConfig serde roundtrip via TOML
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn toml_config_roundtrip(_dummy in 0..1u8) {
        let toml_str = r#"
[scan]
hidden = true

[module]
depth = 3

[analyze]
preset = "health"
"#;
        let config = TomlConfig::parse(toml_str).unwrap();
        prop_assert_eq!(config.scan.hidden, Some(true));
        prop_assert_eq!(config.module.depth, Some(3));
        prop_assert_eq!(config.analyze.preset, Some("health".to_string()));
    }
}
