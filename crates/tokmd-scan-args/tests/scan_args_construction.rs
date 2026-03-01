//! Deeper tests for `ScanArgs` metadata construction, redaction wiring,
//! field defaults, and path normalization within ScanArgs.

use std::path::{Path, PathBuf};

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

// ── Deterministic ScanArgs building ─────────────────────────────

#[test]
fn scan_args_default_options_produce_all_false_flags() {
    let args = scan_args(&[PathBuf::from(".")], &ScanOptions::default(), None);

    assert!(!args.hidden);
    assert!(!args.no_ignore);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
    assert!(!args.treat_doc_strings_as_comments);
    assert!(!args.excluded_redacted);
    assert!(args.excluded.is_empty());
}

#[test]
fn scan_args_is_deterministic_across_repeated_calls() {
    let paths = vec![PathBuf::from("alpha/src"), PathBuf::from("beta/lib")];
    let opts = ScanOptions {
        excluded: vec!["target".to_string(), "vendor".to_string()],
        hidden: true,
        no_ignore: true,
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };

    let results: Vec<_> = (0..5)
        .map(|_| scan_args(&paths, &opts, Some(RedactMode::Paths)))
        .collect();

    for r in &results[1..] {
        assert_eq!(r.paths, results[0].paths);
        assert_eq!(r.excluded, results[0].excluded);
        assert_eq!(r.excluded_redacted, results[0].excluded_redacted);
        assert_eq!(r.hidden, results[0].hidden);
        assert_eq!(r.no_ignore, results[0].no_ignore);
    }
}

#[test]
fn scan_args_json_serialization_is_stable() {
    let paths = vec![PathBuf::from("src"), PathBuf::from("tests")];
    let opts = ScanOptions {
        excluded: vec!["build".to_string()],
        ..Default::default()
    };

    let args = scan_args(&paths, &opts, None);
    let json1 = serde_json::to_string(&args).unwrap();
    let json2 = serde_json::to_string(&args).unwrap();
    assert_eq!(json1, json2);

    let restored: tokmd_types::ScanArgs = serde_json::from_str(&json1).unwrap();
    assert_eq!(restored.paths, args.paths);
    assert_eq!(restored.excluded, args.excluded);
}

// ── Redaction wiring ────────────────────────────────────────────

#[test]
fn redact_paths_hashes_paths_but_not_booleans() {
    let paths = vec![PathBuf::from("secret/project/src")];
    let opts = ScanOptions {
        excluded: vec!["internal".to_string()],
        hidden: true,
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };

    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));

    // Paths and exclusions are redacted
    assert_ne!(args.paths[0], "secret/project/src");
    assert_ne!(args.excluded[0], "internal");
    assert!(args.excluded_redacted);

    // Booleans are unaffected by redaction
    assert!(args.hidden);
    assert!(args.treat_doc_strings_as_comments);
}

#[test]
fn redact_all_matches_redact_paths_behavior() {
    let paths = vec![PathBuf::from("corp/repo")];
    let opts = ScanOptions {
        excluded: vec!["private".to_string()],
        ..Default::default()
    };

    let paths_result = scan_args(&paths, &opts, Some(RedactMode::Paths));
    let all_result = scan_args(&paths, &opts, Some(RedactMode::All));

    assert_eq!(paths_result.paths, all_result.paths);
    assert_eq!(paths_result.excluded, all_result.excluded);
    assert_eq!(paths_result.excluded_redacted, all_result.excluded_redacted);
}

#[test]
fn redact_none_and_option_none_behave_identically() {
    let paths = vec![PathBuf::from("src/main.rs")];
    let opts = ScanOptions {
        excluded: vec!["out".to_string()],
        hidden: true,
        ..Default::default()
    };

    let with_none_mode = scan_args(&paths, &opts, Some(RedactMode::None));
    let with_option_none = scan_args(&paths, &opts, None);

    assert_eq!(with_none_mode.paths, with_option_none.paths);
    assert_eq!(with_none_mode.excluded, with_option_none.excluded);
    assert_eq!(
        with_none_mode.excluded_redacted,
        with_option_none.excluded_redacted
    );
    assert_eq!(with_none_mode.hidden, with_option_none.hidden);
}

#[test]
fn redaction_with_no_exclusions_sets_excluded_redacted_false() {
    let paths = vec![PathBuf::from("src")];
    let opts = ScanOptions::default();

    for mode in [RedactMode::Paths, RedactMode::All] {
        let args = scan_args(&paths, &opts, Some(mode));
        assert!(!args.excluded_redacted);
        assert!(args.excluded.is_empty());
    }
}

#[test]
fn different_inputs_produce_different_redacted_hashes() {
    let opts = ScanOptions::default();

    let a = scan_args(
        &[PathBuf::from("project_x")],
        &opts,
        Some(RedactMode::Paths),
    );
    let b = scan_args(
        &[PathBuf::from("project_y")],
        &opts,
        Some(RedactMode::Paths),
    );

    assert_ne!(a.paths[0], b.paths[0]);
}

// ── Field defaults ──────────────────────────────────────────────

#[test]
fn config_mode_defaults_forwarded() {
    let opts = ScanOptions::default();
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert_eq!(args.config, opts.config);
}

#[test]
fn no_ignore_forces_all_sub_flags_true() {
    let opts = ScanOptions {
        no_ignore: true,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        ..Default::default()
    };

    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

#[test]
fn sub_flags_independent_when_no_ignore_false() {
    let opts = ScanOptions {
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: true,
        ..Default::default()
    };

    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(!args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

// ── Path normalization within ScanArgs ──────────────────────────

#[test]
fn backslash_paths_are_normalized_to_forward_slashes() {
    let paths = vec![PathBuf::from(r"src\nested\file.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), None);

    assert!(!args.paths[0].contains('\\'));
    assert!(args.paths[0].contains("src"));
    assert!(args.paths[0].contains("file.rs"));
}

#[test]
fn dot_slash_prefix_stripped_from_scan_paths() {
    let paths = vec![PathBuf::from("./src/lib.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), None);

    assert!(!args.paths[0].starts_with("./"));
    assert_eq!(args.paths[0], "src/lib.rs");
}

#[test]
fn multiple_dot_slash_prefixes_stripped() {
    let result = normalize_scan_input(Path::new("./././foo/bar"));
    assert_eq!(result, "foo/bar");
}

#[test]
fn empty_path_after_normalization_becomes_dot() {
    let result = normalize_scan_input(Path::new("./"));
    assert_eq!(result, ".");
}

#[test]
fn paths_with_spaces_preserved_through_scan_args() {
    let paths = vec![PathBuf::from("my project/src/lib.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), None);
    assert_eq!(args.paths[0], "my project/src/lib.rs");
}

#[test]
fn all_paths_normalized_in_multi_path_input() {
    let paths = vec![
        PathBuf::from(r".\src\lib.rs"),
        PathBuf::from("tests/unit"),
        PathBuf::from(r"benches\perf.rs"),
    ];
    let args = scan_args(&paths, &ScanOptions::default(), None);

    assert_eq!(args.paths.len(), 3);
    for p in &args.paths {
        assert!(!p.contains('\\'), "path still has backslash: {p}");
        assert!(!p.starts_with("./"), "path still starts with ./: {p}");
    }
}

#[test]
fn empty_paths_vec_produces_empty_scan_args_paths() {
    let args = scan_args(&[], &ScanOptions::default(), None);
    assert!(args.paths.is_empty());
}

#[test]
fn redacted_paths_never_contain_backslashes() {
    let paths = vec![
        PathBuf::from(r"corp\secret\src"),
        PathBuf::from(r"another\path"),
    ];
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::Paths));

    for p in &args.paths {
        assert!(!p.contains('\\'), "redacted path has backslash: {p}");
    }
}

// ── Exclusion count preservation ────────────────────────────────

#[test]
fn exclusion_count_preserved_under_redaction() {
    let opts = ScanOptions {
        excluded: vec![
            "node_modules".to_string(),
            "target".to_string(),
            ".git".to_string(),
        ],
        ..Default::default()
    };

    let args = scan_args(&[PathBuf::from(".")], &opts, Some(RedactMode::All));
    assert_eq!(args.excluded.len(), 3);
    assert!(args.excluded_redacted);
}

#[test]
fn path_count_preserved_under_redaction() {
    let paths: Vec<PathBuf> = (0..10).map(|i| PathBuf::from(format!("dir_{i}"))).collect();
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::Paths));
    assert_eq!(args.paths.len(), 10);
}
