//! Property-based tests for tokmd-settings using proptest.
//!
//! Verifies invariants that must hold for *all* inputs:
//! - Serde round-trips are lossless
//! - Default impls produce deserializable JSON
//! - Field writes are preserved through serialization

use proptest::prelude::*;
use tokmd_settings::*;

// =============================================================================
// Strategies for generating arbitrary settings values
// =============================================================================

fn arb_config_mode() -> impl Strategy<Value = ConfigMode> {
    prop_oneof![Just(ConfigMode::Auto), Just(ConfigMode::None),]
}

fn arb_children_mode() -> impl Strategy<Value = ChildrenMode> {
    prop_oneof![Just(ChildrenMode::Collapse), Just(ChildrenMode::Separate),]
}

fn arb_child_include_mode() -> impl Strategy<Value = ChildIncludeMode> {
    prop_oneof![
        Just(ChildIncludeMode::Separate),
        Just(ChildIncludeMode::ParentsOnly),
    ]
}

fn arb_redact_mode() -> impl Strategy<Value = RedactMode> {
    prop_oneof![
        Just(RedactMode::None),
        Just(RedactMode::Paths),
        Just(RedactMode::All),
    ]
}

fn arb_export_format() -> impl Strategy<Value = ExportFormat> {
    prop_oneof![
        Just(ExportFormat::Csv),
        Just(ExportFormat::Jsonl),
        Just(ExportFormat::Json),
        Just(ExportFormat::Cyclonedx),
    ]
}

/// Arbitrary non-control-character string (JSON-safe).
fn arb_safe_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_./ -]{0,64}"
}

fn arb_string_vec() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_safe_string(), 0..5)
}

// =============================================================================
// Property: ScanOptions serde round-trip
// =============================================================================

proptest! {
    #[test]
    fn scan_options_roundtrip(
        excluded in arb_string_vec(),
        config in arb_config_mode(),
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        no_ignore_parent in any::<bool>(),
        no_ignore_dot in any::<bool>(),
        no_ignore_vcs in any::<bool>(),
        treat_doc in any::<bool>(),
    ) {
        let opts = ScanOptions {
            excluded: excluded.clone(),
            config,
            hidden,
            no_ignore,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            treat_doc_strings_as_comments: treat_doc,
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: ScanOptions = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.excluded, &excluded);
        prop_assert_eq!(back.hidden, hidden);
        prop_assert_eq!(back.no_ignore, no_ignore);
        prop_assert_eq!(back.no_ignore_parent, no_ignore_parent);
        prop_assert_eq!(back.no_ignore_dot, no_ignore_dot);
        prop_assert_eq!(back.no_ignore_vcs, no_ignore_vcs);
        prop_assert_eq!(back.treat_doc_strings_as_comments, treat_doc);
    }
}

// =============================================================================
// Property: ScanSettings serde round-trip (with flattened options)
// =============================================================================

proptest! {
    #[test]
    fn scan_settings_roundtrip(
        paths in arb_string_vec(),
        hidden in any::<bool>(),
        excluded in arb_string_vec(),
    ) {
        let s = ScanSettings {
            paths: paths.clone(),
            options: ScanOptions {
                excluded: excluded.clone(),
                hidden,
                ..Default::default()
            },
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ScanSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.paths, &paths);
        prop_assert_eq!(back.options.hidden, hidden);
        prop_assert_eq!(&back.options.excluded, &excluded);
    }
}

// =============================================================================
// Property: LangSettings serde round-trip
// =============================================================================

proptest! {
    #[test]
    fn lang_settings_roundtrip(
        top in any::<usize>(),
        files in any::<bool>(),
        children in arb_children_mode(),
        has_redact in any::<bool>(),
        redact in arb_redact_mode(),
    ) {
        let redact_opt = if has_redact { Some(redact) } else { Option::None };
        let s = LangSettings { top, files, children, redact: redact_opt };
        let json = serde_json::to_string(&s).unwrap();
        let back: LangSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.top, top);
        prop_assert_eq!(back.files, files);
    }
}

// =============================================================================
// Property: ModuleSettings serde round-trip
// =============================================================================

proptest! {
    #[test]
    fn module_settings_roundtrip(
        top in any::<usize>(),
        roots in arb_string_vec(),
        depth in 0usize..100,
        children in arb_child_include_mode(),
        has_redact in any::<bool>(),
        redact in arb_redact_mode(),
    ) {
        let redact_opt = if has_redact { Some(redact) } else { Option::None };
        let s = ModuleSettings {
            top,
            module_roots: roots.clone(),
            module_depth: depth,
            children,
            redact: redact_opt,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ModuleSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.top, top);
        prop_assert_eq!(&back.module_roots, &roots);
        prop_assert_eq!(back.module_depth, depth);
    }
}

// =============================================================================
// Property: ExportSettings serde round-trip
// =============================================================================

proptest! {
    #[test]
    fn export_settings_roundtrip(
        format in arb_export_format(),
        roots in arb_string_vec(),
        depth in 0usize..100,
        children in arb_child_include_mode(),
        min_code in any::<usize>(),
        max_rows in any::<usize>(),
        redact in arb_redact_mode(),
        meta in any::<bool>(),
        has_prefix in any::<bool>(),
        prefix in arb_safe_string(),
    ) {
        let strip = if has_prefix { Some(prefix.clone()) } else { Option::None };
        let s = ExportSettings {
            format, module_roots: roots.clone(), module_depth: depth,
            children, min_code, max_rows, redact, meta, strip_prefix: strip.clone(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ExportSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.min_code, min_code);
        prop_assert_eq!(back.max_rows, max_rows);
        prop_assert_eq!(back.meta, meta);
        prop_assert_eq!(&back.module_roots, &roots);
        prop_assert_eq!(back.module_depth, depth);
        prop_assert_eq!(&back.strip_prefix, &strip);
    }
}

// =============================================================================
// Property: AnalyzeSettings serde round-trip
// =============================================================================

proptest! {
    #[test]
    fn analyze_settings_roundtrip(
        preset in arb_safe_string(),
        window in proptest::option::of(any::<usize>()),
        git in proptest::option::of(any::<bool>()),
        max_files in proptest::option::of(any::<usize>()),
        max_bytes in proptest::option::of(any::<u64>()),
        max_file_bytes in proptest::option::of(any::<u64>()),
        max_commits in proptest::option::of(any::<usize>()),
        max_commit_files in proptest::option::of(any::<usize>()),
        granularity in arb_safe_string(),
    ) {
        let s = AnalyzeSettings {
            preset: preset.clone(),
            window, git, max_files, max_bytes,
            max_file_bytes, max_commits, max_commit_files,
            granularity: granularity.clone(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.preset, &preset);
        prop_assert_eq!(back.window, window);
        prop_assert_eq!(back.git, git);
        prop_assert_eq!(back.max_files, max_files);
        prop_assert_eq!(back.max_bytes, max_bytes);
        prop_assert_eq!(back.max_file_bytes, max_file_bytes);
        prop_assert_eq!(back.max_commits, max_commits);
        prop_assert_eq!(back.max_commit_files, max_commit_files);
        prop_assert_eq!(&back.granularity, &granularity);
    }
}

// =============================================================================
// Property: CockpitSettings serde round-trip
// =============================================================================

proptest! {
    #[test]
    fn cockpit_settings_roundtrip(
        base in arb_safe_string(),
        head in arb_safe_string(),
        range_mode in arb_safe_string(),
        has_baseline in any::<bool>(),
        baseline_val in arb_safe_string(),
    ) {
        let baseline = if has_baseline { Some(baseline_val.clone()) } else { Option::None };
        let s = CockpitSettings { base: base.clone(), head: head.clone(), range_mode: range_mode.clone(), baseline: baseline.clone() };
        let json = serde_json::to_string(&s).unwrap();
        let back: CockpitSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.base, &base);
        prop_assert_eq!(&back.head, &head);
        prop_assert_eq!(&back.range_mode, &range_mode);
        prop_assert_eq!(&back.baseline, &baseline);
    }
}

// =============================================================================
// Property: DiffSettings serde round-trip
// =============================================================================

proptest! {
    #[test]
    fn diff_settings_roundtrip(
        from in arb_safe_string(),
        to in arb_safe_string(),
    ) {
        let s = DiffSettings { from: from.clone(), to: to.clone() };
        let json = serde_json::to_string(&s).unwrap();
        let back: DiffSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.from, &from);
        prop_assert_eq!(&back.to, &to);
    }
}

// =============================================================================
// Property: All Default impls produce valid JSON that round-trips
// =============================================================================

#[test]
fn default_scan_options_serializes() {
    let json = serde_json::to_string(&ScanOptions::default()).unwrap();
    let _: ScanOptions = serde_json::from_str(&json).unwrap();
}

#[test]
fn default_scan_settings_serializes() {
    let json = serde_json::to_string(&ScanSettings::default()).unwrap();
    let _: ScanSettings = serde_json::from_str(&json).unwrap();
}

#[test]
fn default_lang_settings_serializes() {
    let json = serde_json::to_string(&LangSettings::default()).unwrap();
    let _: LangSettings = serde_json::from_str(&json).unwrap();
}

#[test]
fn default_module_settings_serializes() {
    let json = serde_json::to_string(&ModuleSettings::default()).unwrap();
    let _: ModuleSettings = serde_json::from_str(&json).unwrap();
}

#[test]
fn default_export_settings_serializes() {
    let json = serde_json::to_string(&ExportSettings::default()).unwrap();
    let _: ExportSettings = serde_json::from_str(&json).unwrap();
}

#[test]
fn default_analyze_settings_serializes() {
    let json = serde_json::to_string(&AnalyzeSettings::default()).unwrap();
    let _: AnalyzeSettings = serde_json::from_str(&json).unwrap();
}

#[test]
fn default_cockpit_settings_serializes() {
    let json = serde_json::to_string(&CockpitSettings::default()).unwrap();
    let _: CockpitSettings = serde_json::from_str(&json).unwrap();
}

#[test]
fn default_diff_settings_serializes() {
    let json = serde_json::to_string(&DiffSettings::default()).unwrap();
    let _: DiffSettings = serde_json::from_str(&json).unwrap();
}

#[test]
fn default_toml_config_serializes_to_toml() {
    let toml_str = toml::to_string(&TomlConfig::default()).unwrap();
    let _: TomlConfig = toml::from_str(&toml_str).unwrap();
}

// =============================================================================
// Property: TOML round-trip for config sections
// =============================================================================

proptest! {
    #[test]
    fn scan_config_toml_roundtrip(
        hidden in proptest::option::of(any::<bool>()),
        no_ignore in proptest::option::of(any::<bool>()),
        doc_comments in proptest::option::of(any::<bool>()),
    ) {
        let cfg = ScanConfig {
            hidden,
            no_ignore,
            doc_comments,
            ..Default::default()
        };
        let toml_str = toml::to_string(&cfg).unwrap();
        let back: ScanConfig = toml::from_str(&toml_str).unwrap();
        prop_assert_eq!(back.hidden, hidden);
        prop_assert_eq!(back.no_ignore, no_ignore);
        prop_assert_eq!(back.doc_comments, doc_comments);
    }
}

proptest! {
    #[test]
    fn module_config_toml_roundtrip(
        depth in proptest::option::of(0usize..100),
        children in proptest::option::of(prop_oneof![
            Just("collapse".to_string()),
            Just("separate".to_string()),
        ]),
    ) {
        let cfg = ModuleConfig {
            roots: None,
            depth,
            children: children.clone(),
        };
        let toml_str = toml::to_string(&cfg).unwrap();
        let back: ModuleConfig = toml::from_str(&toml_str).unwrap();
        prop_assert_eq!(back.depth, depth);
        prop_assert_eq!(&back.children, &children);
    }
}

proptest! {
    #[test]
    fn context_config_toml_roundtrip(
        budget in proptest::option::of(arb_safe_string()),
        strategy in proptest::option::of(prop_oneof![
            Just("greedy".to_string()),
            Just("spread".to_string()),
        ]),
        compress in proptest::option::of(any::<bool>()),
    ) {
        let cfg = ContextConfig {
            budget: budget.clone(),
            strategy: strategy.clone(),
            rank_by: None,
            output: None,
            compress,
        };
        let toml_str = toml::to_string(&cfg).unwrap();
        let back: ContextConfig = toml::from_str(&toml_str).unwrap();
        prop_assert_eq!(&back.budget, &budget);
        prop_assert_eq!(&back.strategy, &strategy);
        prop_assert_eq!(back.compress, compress);
    }
}

// =============================================================================
// Property: ViewProfile partial fields survive TOML round-trip
// =============================================================================

proptest! {
    #[test]
    fn view_profile_toml_roundtrip(
        top in proptest::option::of(0usize..1000),
        files in proptest::option::of(any::<bool>()),
        compress in proptest::option::of(any::<bool>()),
        min_code in proptest::option::of(0usize..10000),
    ) {
        let vp = ViewProfile {
            top,
            files,
            compress,
            min_code,
            ..Default::default()
        };
        let toml_str = toml::to_string(&vp).unwrap();
        let back: ViewProfile = toml::from_str(&toml_str).unwrap();
        prop_assert_eq!(back.top, top);
        prop_assert_eq!(back.files, files);
        prop_assert_eq!(back.compress, compress);
        prop_assert_eq!(back.min_code, min_code);
    }
}

// =============================================================================
// Property: GateRule serde round-trip
// =============================================================================

proptest! {
    #[test]
    fn gate_rule_roundtrip(
        name in arb_safe_string(),
        pointer in "/[a-z]{1,10}(/[a-z]{1,10}){0,3}",
        op in prop_oneof![
            Just("==".to_string()),
            Just("!=".to_string()),
            Just("<=".to_string()),
            Just(">=".to_string()),
            Just("<".to_string()),
            Just(">".to_string()),
        ],
        negate in any::<bool>(),
    ) {
        let rule = GateRule {
            name: name.clone(),
            pointer: pointer.clone(),
            op: op.clone(),
            value: Some(serde_json::json!(42)),
            values: None,
            negate,
            level: Some("error".to_string()),
            message: None,
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: GateRule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.name, &name);
        prop_assert_eq!(&back.pointer, &pointer);
        prop_assert_eq!(&back.op, &op);
        prop_assert_eq!(back.negate, negate);
    }
}

// =============================================================================
// Property: RatchetRuleConfig serde round-trip
// =============================================================================

proptest! {
    #[test]
    fn ratchet_rule_roundtrip(
        pointer in "/[a-z]{1,10}(/[a-z]{1,10}){0,3}",
        max_inc in proptest::option::of(0.0f64..1000.0),
        max_val in proptest::option::of(0.0f64..1000.0),
    ) {
        let rule = RatchetRuleConfig {
            pointer: pointer.clone(),
            max_increase_pct: max_inc,
            max_value: max_val,
            level: Some("warn".to_string()),
            description: Some("test rule".to_string()),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: RatchetRuleConfig = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.pointer, &pointer);
        // Float comparison with tolerance
        match (back.max_increase_pct, max_inc) {
            (Some(a), Some(b)) => prop_assert!((a - b).abs() < 1e-10),
            (None, None) => {}
            _ => prop_assert!(false, "max_increase_pct mismatch"),
        }
        match (back.max_value, max_val) {
            (Some(a), Some(b)) => prop_assert!((a - b).abs() < 1e-10),
            (None, None) => {}
            _ => prop_assert!(false, "max_value mismatch"),
        }
    }
}

// =============================================================================
// Property: Default impls are consistent with serde defaults
// =============================================================================

/// Deserializing an empty JSON object must yield the same result as Default::default()
/// for every settings type that uses `#[serde(default)]`.
#[test]
fn default_consistency_scan_options() {
    let from_default = serde_json::to_value(&ScanOptions::default()).unwrap();
    let from_empty: ScanOptions = serde_json::from_str("{}").unwrap();
    let from_empty_val = serde_json::to_value(&from_empty).unwrap();
    assert_eq!(from_default, from_empty_val);
}

#[test]
fn default_consistency_scan_settings() {
    let from_default = serde_json::to_value(&ScanSettings::default()).unwrap();
    let from_empty: ScanSettings = serde_json::from_str("{}").unwrap();
    let from_empty_val = serde_json::to_value(&from_empty).unwrap();
    assert_eq!(from_default, from_empty_val);
}

#[test]
fn default_consistency_lang_settings() {
    let from_default = serde_json::to_value(&LangSettings::default()).unwrap();
    let from_empty: LangSettings = serde_json::from_str("{}").unwrap();
    let from_empty_val = serde_json::to_value(&from_empty).unwrap();
    assert_eq!(from_default, from_empty_val);
}

#[test]
fn default_consistency_module_settings() {
    let from_default = serde_json::to_value(&ModuleSettings::default()).unwrap();
    let from_empty: ModuleSettings = serde_json::from_str("{}").unwrap();
    let from_empty_val = serde_json::to_value(&from_empty).unwrap();
    assert_eq!(from_default, from_empty_val);
}

#[test]
fn default_consistency_export_settings() {
    let from_default = serde_json::to_value(&ExportSettings::default()).unwrap();
    let from_empty: ExportSettings = serde_json::from_str("{}").unwrap();
    let from_empty_val = serde_json::to_value(&from_empty).unwrap();
    assert_eq!(from_default, from_empty_val);
}

#[test]
fn default_consistency_analyze_settings() {
    let from_default = serde_json::to_value(&AnalyzeSettings::default()).unwrap();
    let from_empty: AnalyzeSettings = serde_json::from_str("{}").unwrap();
    let from_empty_val = serde_json::to_value(&from_empty).unwrap();
    assert_eq!(from_default, from_empty_val);
}

#[test]
fn default_consistency_cockpit_settings() {
    let from_default = serde_json::to_value(&CockpitSettings::default()).unwrap();
    let from_empty: CockpitSettings = serde_json::from_str("{}").unwrap();
    let from_empty_val = serde_json::to_value(&from_empty).unwrap();
    assert_eq!(from_default, from_empty_val);
}

#[test]
fn default_consistency_toml_config() {
    let from_default = toml::to_string(&TomlConfig::default()).unwrap();
    let from_empty: TomlConfig = toml::from_str("").unwrap();
    let from_empty_str = toml::to_string(&from_empty).unwrap();
    assert_eq!(from_default, from_empty_str);
}

// =============================================================================
// Property: ScanSettings helper methods preserve defaults
// =============================================================================

proptest! {
    #[test]
    fn scan_settings_for_paths_preserves_default_options(
        paths in prop::collection::vec(arb_safe_string(), 1..5),
    ) {
        let s = ScanSettings::for_paths(paths.clone());
        let default_opts = ScanOptions::default();
        prop_assert_eq!(&s.paths, &paths);
        prop_assert_eq!(s.options.hidden, default_opts.hidden);
        prop_assert_eq!(s.options.no_ignore, default_opts.no_ignore);
        prop_assert_eq!(s.options.no_ignore_parent, default_opts.no_ignore_parent);
        prop_assert_eq!(s.options.no_ignore_dot, default_opts.no_ignore_dot);
        prop_assert_eq!(s.options.no_ignore_vcs, default_opts.no_ignore_vcs);
        prop_assert_eq!(
            s.options.treat_doc_strings_as_comments,
            default_opts.treat_doc_strings_as_comments
        );
    }
}

#[test]
fn scan_settings_current_dir_has_dot_path() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
    // options must be default
    let d = ScanOptions::default();
    assert_eq!(s.options.hidden, d.hidden);
}

// =============================================================================
// Property: JSON ↔ JSON value stability (double serialization)
// =============================================================================

proptest! {
    /// Serializing, deserializing, then re-serializing must produce identical JSON.
    #[test]
    fn scan_options_double_roundtrip(
        excluded in arb_string_vec(),
        config in arb_config_mode(),
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
    ) {
        let opts = ScanOptions {
            excluded, config, hidden, no_ignore,
            ..Default::default()
        };
        let json1 = serde_json::to_string(&opts).unwrap();
        let mid: ScanOptions = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&mid).unwrap();
        prop_assert_eq!(json1, json2, "double roundtrip must be stable");
    }

    /// ExportSettings double roundtrip stability.
    #[test]
    fn export_settings_double_roundtrip(
        min_code in any::<usize>(),
        max_rows in any::<usize>(),
        meta in any::<bool>(),
        redact in arb_redact_mode(),
    ) {
        let s = ExportSettings {
            min_code, max_rows, meta, redact,
            ..Default::default()
        };
        let json1 = serde_json::to_string(&s).unwrap();
        let mid: ExportSettings = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&mid).unwrap();
        prop_assert_eq!(json1, json2, "double roundtrip must be stable");
    }

    /// AnalyzeSettings double roundtrip stability.
    #[test]
    fn analyze_settings_double_roundtrip(
        preset in arb_safe_string(),
        granularity in arb_safe_string(),
        window in proptest::option::of(any::<usize>()),
    ) {
        let s = AnalyzeSettings {
            preset, granularity, window,
            ..Default::default()
        };
        let json1 = serde_json::to_string(&s).unwrap();
        let mid: AnalyzeSettings = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&mid).unwrap();
        prop_assert_eq!(json1, json2, "double roundtrip must be stable");
    }
}

// =============================================================================
// Property: TomlConfig with arbitrary view profiles round-trips
// =============================================================================

proptest! {
    #[test]
    fn toml_config_with_profiles_roundtrip(
        profile_name in "[a-z]{1,10}",
        top in proptest::option::of(0usize..500),
        files in proptest::option::of(any::<bool>()),
        preset in proptest::option::of(arb_safe_string()),
    ) {
        let mut config = TomlConfig::default();
        config.view.insert(profile_name.clone(), ViewProfile {
            top,
            files,
            preset: preset.clone(),
            ..Default::default()
        });
        let toml_str = toml::to_string(&config).unwrap();
        let back: TomlConfig = toml::from_str(&toml_str).unwrap();
        let profile = back.view.get(&profile_name).expect("profile must survive roundtrip");
        prop_assert_eq!(profile.top, top);
        prop_assert_eq!(profile.files, files);
        prop_assert_eq!(&profile.preset, &preset);
    }
}

// =============================================================================
// Property: ModuleSettings default depth is non-zero
// =============================================================================

#[test]
fn module_settings_default_depth_nonzero() {
    let s = ModuleSettings::default();
    assert!(s.module_depth > 0, "default module_depth must be > 0");
}

#[test]
fn module_settings_default_roots_nonempty() {
    let s = ModuleSettings::default();
    assert!(
        !s.module_roots.is_empty(),
        "default module_roots must be non-empty"
    );
}

#[test]
fn analyze_settings_default_preset_is_receipt() {
    let s = AnalyzeSettings::default();
    assert_eq!(s.preset, "receipt");
}

#[test]
fn cockpit_settings_default_range_mode_is_two_dot() {
    let s = CockpitSettings::default();
    assert_eq!(s.range_mode, "two-dot");
}

#[test]
fn export_settings_default_meta_is_true() {
    let s = ExportSettings::default();
    assert!(s.meta);
}
