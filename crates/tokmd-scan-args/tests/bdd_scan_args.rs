use std::path::PathBuf;

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

#[test]
fn given_paths_mode_when_building_scan_args_then_paths_and_exclusions_are_redacted() {
    // Given: user-provided scan paths and exclusion patterns
    let paths = vec![PathBuf::from("secret/src/lib.rs")];
    let global = ScanOptions {
        excluded: vec!["**/private/**".to_string()],
        ..Default::default()
    };

    // When: scan args are constructed with redact paths mode
    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    // Then: raw paths and exclusion patterns are not present in metadata
    assert_ne!(args.paths[0], "secret/src/lib.rs");
    assert_ne!(args.excluded[0], "**/private/**");
    assert!(args.excluded_redacted);
}

#[test]
fn given_no_redaction_when_building_scan_args_then_normalized_paths_are_preserved() {
    // Given: relative scan inputs with slash variants
    let paths = vec![PathBuf::from(r".\src\main.rs")];
    let global = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };

    // When: scan args are constructed without redaction
    let args = scan_args(&paths, &global, None);

    // Then: paths are normalized but not hashed, and exclusions remain visible
    assert_eq!(args.paths, vec!["src/main.rs".to_string()]);
    assert_eq!(args.excluded, vec!["target".to_string()]);
    assert!(!args.excluded_redacted);
}

#[test]
fn given_no_ignore_when_building_scan_args_then_ignore_sub_flags_are_forced_true() {
    // Given: no_ignore is enabled but sub-flags are false in settings
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        no_ignore: true,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        ..Default::default()
    };

    // When: scan args are constructed
    let args = scan_args(&paths, &global, Some(RedactMode::None));

    // Then: all ignore-family flags are true for deterministic behavior
    assert!(args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

// ── Redact-mode coverage ─────────────────────────────────────────────

#[test]
fn given_all_mode_when_building_scan_args_then_paths_and_exclusions_are_redacted() {
    // Given: user paths and exclusions with RedactMode::All
    let paths = vec![PathBuf::from("corp/repo/src")];
    let global = ScanOptions {
        excluded: vec!["vendor".to_string(), "third_party".to_string()],
        ..Default::default()
    };

    // When: scan args are constructed with redact all mode
    let args = scan_args(&paths, &global, Some(RedactMode::All));

    // Then: paths and exclusions are both redacted
    assert_ne!(args.paths[0], "corp/repo/src");
    assert_ne!(args.excluded[0], "vendor");
    assert_ne!(args.excluded[1], "third_party");
    assert!(args.excluded_redacted);
}

#[test]
fn given_none_mode_when_building_scan_args_then_nothing_is_redacted() {
    // Given: explicit RedactMode::None
    let paths = vec![PathBuf::from("src/main.rs")];
    let global = ScanOptions {
        excluded: vec!["build".to_string()],
        ..Default::default()
    };

    // When: scan args built with None mode
    let args = scan_args(&paths, &global, Some(RedactMode::None));

    // Then: paths and exclusions are preserved verbatim (after normalization)
    assert_eq!(args.paths[0], "src/main.rs");
    assert_eq!(args.excluded[0], "build");
    assert!(!args.excluded_redacted);
}

#[test]
fn given_none_option_when_building_scan_args_then_nothing_is_redacted() {
    // Given: redact parameter is Rust None (not RedactMode::None)
    let paths = vec![PathBuf::from("lib/core.rs")];
    let global = ScanOptions {
        excluded: vec!["tmp".to_string()],
        ..Default::default()
    };

    // When: scan args built with Option::None
    let args = scan_args(&paths, &global, None);

    // Then: same as RedactMode::None — no redaction
    assert_eq!(args.paths[0], "lib/core.rs");
    assert_eq!(args.excluded[0], "tmp");
    assert!(!args.excluded_redacted);
}

// ── Redaction with empty exclusions ──────────────────────────────────

#[test]
fn given_paths_mode_and_empty_exclusions_then_excluded_redacted_is_false() {
    // Given: redact paths mode but no exclusion patterns
    let paths = vec![PathBuf::from("src")];
    let global = ScanOptions {
        excluded: vec![],
        ..Default::default()
    };

    // When: scan args are constructed
    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    // Then: excluded_redacted is false because there is nothing to redact
    assert!(!args.excluded_redacted);
    assert!(args.excluded.is_empty());
}

#[test]
fn given_all_mode_and_empty_exclusions_then_excluded_redacted_is_false() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions::default();

    let args = scan_args(&paths, &global, Some(RedactMode::All));

    assert!(!args.excluded_redacted);
}

// ── Multiple input paths ─────────────────────────────────────────────

#[test]
fn given_multiple_paths_when_no_redaction_then_all_paths_are_normalized() {
    // Given: several paths with varying separators
    let paths = vec![
        PathBuf::from(r".\src\lib.rs"),
        PathBuf::from("tests/integration"),
        PathBuf::from(r"benches\perf.rs"),
    ];
    let global = ScanOptions::default();

    // When: scan args are constructed without redaction
    let args = scan_args(&paths, &global, None);

    // Then: all paths are forward-slash normalized
    assert_eq!(args.paths.len(), 3);
    assert_eq!(args.paths[0], "src/lib.rs");
    assert_eq!(args.paths[1], "tests/integration");
    assert_eq!(args.paths[2], "benches/perf.rs");
}

#[test]
fn given_multiple_paths_when_redaction_then_all_paths_are_redacted() {
    // Given: multiple scan paths under redaction
    let paths = vec![
        PathBuf::from("alpha/src"),
        PathBuf::from("beta/src"),
        PathBuf::from("gamma/src"),
    ];
    let global = ScanOptions::default();

    // When: scan args built with Paths mode
    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    // Then: none of the original paths appear
    assert_eq!(args.paths.len(), 3);
    for (i, original) in ["alpha/src", "beta/src", "gamma/src"].iter().enumerate() {
        assert_ne!(&args.paths[i], original);
    }
}

#[test]
fn given_empty_paths_when_building_scan_args_then_result_has_empty_paths() {
    // Given: no input paths
    let paths: Vec<PathBuf> = vec![];
    let global = ScanOptions::default();

    // When: scan args constructed
    let args = scan_args(&paths, &global, None);

    // Then: paths vec is empty
    assert!(args.paths.is_empty());
}

// ── Multiple exclusion patterns ──────────────────────────────────────

#[test]
fn given_many_exclusions_when_redacted_then_all_are_hashed() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        excluded: vec![
            "node_modules".to_string(),
            "target".to_string(),
            ".git".to_string(),
            "dist".to_string(),
        ],
        ..Default::default()
    };

    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    assert_eq!(args.excluded.len(), 4);
    assert!(args.excluded_redacted);
    for (i, original) in ["node_modules", "target", ".git", "dist"]
        .iter()
        .enumerate()
    {
        assert_ne!(&args.excluded[i], original);
    }
}

// ── Redaction determinism (same input → same hash) ───────────────────

#[test]
fn given_same_inputs_when_redacted_twice_then_hashes_are_identical() {
    let paths = vec![PathBuf::from("my/secret/project")];
    let global = ScanOptions {
        excluded: vec!["confidential".to_string()],
        ..Default::default()
    };

    let a = scan_args(&paths, &global, Some(RedactMode::Paths));
    let b = scan_args(&paths, &global, Some(RedactMode::Paths));

    assert_eq!(a.paths, b.paths);
    assert_eq!(a.excluded, b.excluded);
}

#[test]
fn given_different_paths_when_redacted_then_hashes_differ() {
    let global = ScanOptions::default();

    let a = scan_args(
        &[PathBuf::from("project_a/src")],
        &global,
        Some(RedactMode::Paths),
    );
    let b = scan_args(
        &[PathBuf::from("project_b/src")],
        &global,
        Some(RedactMode::Paths),
    );

    assert_ne!(a.paths[0], b.paths[0]);
}

// ── no_ignore sub-flag propagation edge cases ────────────────────────

#[test]
fn given_sub_flags_true_but_no_ignore_false_then_sub_flags_are_preserved() {
    // Given: individual sub-flags set but master no_ignore is false
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: false,
        ..Default::default()
    };

    // When: scan args constructed
    let args = scan_args(&paths, &global, None);

    // Then: sub-flags reflect their individual values OR'd with no_ignore
    assert!(!args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
}

#[test]
fn given_all_ignore_flags_false_then_all_remain_false() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        ..Default::default()
    };

    let args = scan_args(&paths, &global, None);

    assert!(!args.no_ignore);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
}

// ── Boolean flag pass-through ────────────────────────────────────────

#[test]
fn given_hidden_true_when_building_scan_args_then_hidden_is_set() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        hidden: true,
        ..Default::default()
    };

    let args = scan_args(&paths, &global, None);
    assert!(args.hidden);
}

#[test]
fn given_treat_doc_strings_as_comments_then_flag_is_forwarded() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };

    let args = scan_args(&paths, &global, None);
    assert!(args.treat_doc_strings_as_comments);
}

#[test]
fn given_config_none_then_config_is_none() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        config: tokmd_types::ConfigMode::None,
        ..Default::default()
    };

    let args = scan_args(&paths, &global, None);
    assert_eq!(args.config, tokmd_types::ConfigMode::None);
}

// ── normalize_scan_input edge cases ──────────────────────────────────

#[test]
fn given_bare_dot_path_then_normalize_returns_dot() {
    assert_eq!(normalize_scan_input(&PathBuf::from(".")), ".");
}

#[test]
fn given_deeply_nested_dot_slash_then_all_are_stripped() {
    assert_eq!(
        normalize_scan_input(&PathBuf::from("./././deep/path")),
        "deep/path"
    );
}

#[test]
fn given_backslash_path_then_normalize_uses_forward_slashes() {
    let result = normalize_scan_input(&PathBuf::from(r"src\nested\file.rs"));
    assert!(!result.contains('\\'));
    assert!(result.contains("src") && result.contains("file.rs"));
}

#[test]
fn given_absolute_unix_style_path_then_preserved_with_forward_slashes() {
    let result = normalize_scan_input(&PathBuf::from("/home/user/project"));
    assert!(!result.contains('\\'));
    assert!(result.contains("project"));
}

#[test]
fn given_path_with_spaces_then_preserved() {
    let result = normalize_scan_input(&PathBuf::from("my project/src/lib.rs"));
    assert_eq!(result, "my project/src/lib.rs");
}

#[test]
fn given_path_with_unicode_then_preserved() {
    let result = normalize_scan_input(&PathBuf::from("données/résumé.rs"));
    assert!(result.contains("données"));
    assert!(result.contains("résumé.rs"));
}

// ── Combined scenarios ───────────────────────────────────────────────

#[test]
fn given_complex_setup_all_fields_propagate_correctly() {
    // Given: all options set to non-default values
    let paths = vec![PathBuf::from("a"), PathBuf::from("b")];
    let global = ScanOptions {
        excluded: vec!["x".to_string()],
        config: tokmd_types::ConfigMode::None,
        hidden: true,
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };

    // When: no redaction
    let args = scan_args(&paths, &global, Some(RedactMode::None));

    // Then: every field matches expectations
    assert_eq!(args.paths, vec!["a", "b"]);
    assert_eq!(args.excluded, vec!["x"]);
    assert!(!args.excluded_redacted);
    assert_eq!(args.config, tokmd_types::ConfigMode::None);
    assert!(args.hidden);
    assert!(!args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
    assert!(args.treat_doc_strings_as_comments);
}

#[test]
fn given_complex_setup_with_redaction_then_data_fields_redacted_booleans_preserved() {
    let paths = vec![PathBuf::from("secret/a"), PathBuf::from("secret/b")];
    let global = ScanOptions {
        excluded: vec!["private_dir".to_string()],
        hidden: true,
        no_ignore: true,
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };

    let args = scan_args(&paths, &global, Some(RedactMode::All));

    // Paths and exclusions are redacted
    assert_ne!(args.paths[0], "secret/a");
    assert_ne!(args.paths[1], "secret/b");
    assert_ne!(args.excluded[0], "private_dir");
    assert!(args.excluded_redacted);

    // Boolean flags still forwarded
    assert!(args.hidden);
    assert!(args.no_ignore);
    assert!(args.no_ignore_parent); // forced by no_ignore
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
    assert!(args.treat_doc_strings_as_comments);
}
