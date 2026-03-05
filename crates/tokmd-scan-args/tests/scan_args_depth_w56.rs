//! Depth tests for tokmd-scan-args — W56 tooling coverage.

use std::path::{Path, PathBuf};

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

// ---------------------------------------------------------------------------
// normalize_scan_input
// ---------------------------------------------------------------------------

#[test]
fn normalize_strips_single_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("./src")), "src");
}

#[test]
fn normalize_strips_repeated_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("././././foo")), "foo");
}

#[test]
fn normalize_empty_to_dot() {
    assert_eq!(normalize_scan_input(Path::new("")), ".");
}

#[test]
fn normalize_dot_slash_only_to_dot() {
    assert_eq!(normalize_scan_input(Path::new("./")), ".");
}

#[test]
fn normalize_preserves_absolute_path() {
    let input = normalize_scan_input(Path::new("/usr/local/src"));
    assert!(input.contains("usr/local/src") || input.contains("usr\\local\\src"));
}

#[test]
fn normalize_preserves_deep_relative_path() {
    let result = normalize_scan_input(Path::new("a/b/c/d/e"));
    assert_eq!(result, "a/b/c/d/e");
}

#[test]
fn normalize_dot_to_dot() {
    assert_eq!(normalize_scan_input(Path::new(".")), ".");
}

#[test]
fn normalize_forward_slashes_on_unix_paths() {
    let result = normalize_scan_input(Path::new("src/main.rs"));
    assert!(!result.contains('\\'));
}

// ---------------------------------------------------------------------------
// scan_args construction — basic
// ---------------------------------------------------------------------------

fn default_scan_opts() -> ScanOptions {
    ScanOptions::default()
}

#[test]
fn scan_args_single_path() {
    let args = scan_args(&[PathBuf::from("src")], &default_scan_opts(), None);
    assert_eq!(args.paths, vec!["src"]);
}

#[test]
fn scan_args_multiple_paths() {
    let paths = vec![PathBuf::from("src"), PathBuf::from("lib")];
    let args = scan_args(&paths, &default_scan_opts(), None);
    assert_eq!(args.paths.len(), 2);
    assert!(args.paths.contains(&"src".to_string()));
    assert!(args.paths.contains(&"lib".to_string()));
}

#[test]
fn scan_args_empty_paths() {
    let args = scan_args(&[], &default_scan_opts(), None);
    assert!(args.paths.is_empty());
}

#[test]
fn scan_args_excluded_passthrough_no_redact() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string(), "vendor".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert_eq!(args.excluded, vec!["target", "vendor"]);
    assert!(!args.excluded_redacted);
}

#[test]
fn scan_args_hidden_flag_passthrough() {
    let opts = ScanOptions {
        hidden: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.hidden);
}

#[test]
fn scan_args_doc_strings_flag_passthrough() {
    let opts = ScanOptions {
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.treat_doc_strings_as_comments);
}

// ---------------------------------------------------------------------------
// no_ignore cascading
// ---------------------------------------------------------------------------

#[test]
fn no_ignore_enables_all_sub_flags() {
    let opts = ScanOptions {
        no_ignore: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

#[test]
fn no_ignore_parent_standalone() {
    let opts = ScanOptions {
        no_ignore_parent: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(!args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
}

#[test]
fn no_ignore_dot_standalone() {
    let opts = ScanOptions {
        no_ignore_dot: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.no_ignore_dot);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_vcs);
}

#[test]
fn no_ignore_vcs_standalone() {
    let opts = ScanOptions {
        no_ignore_vcs: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.no_ignore_vcs);
    assert!(!args.no_ignore_dot);
}

// ---------------------------------------------------------------------------
// Redaction — RedactMode::Paths
// ---------------------------------------------------------------------------

#[test]
fn redact_paths_changes_scan_paths() {
    let args = scan_args(
        &[PathBuf::from("src/lib.rs")],
        &default_scan_opts(),
        Some(RedactMode::Paths),
    );
    assert_ne!(args.paths[0], "src/lib.rs");
}

#[test]
fn redact_paths_hashes_exclusions() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, Some(RedactMode::Paths));
    assert_ne!(args.excluded[0], "target");
    assert!(args.excluded_redacted);
}

#[test]
fn redact_all_also_redacts() {
    let opts = ScanOptions {
        excluded: vec!["vendor".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from("src")], &opts, Some(RedactMode::All));
    assert_ne!(args.paths[0], "src");
    assert_ne!(args.excluded[0], "vendor");
    assert!(args.excluded_redacted);
}

#[test]
fn redact_none_does_not_redact() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from("src")], &opts, Some(RedactMode::None));
    assert_eq!(args.paths[0], "src");
    assert_eq!(args.excluded[0], "target");
    assert!(!args.excluded_redacted);
}

#[test]
fn no_redact_mode_none_passthrough() {
    let args = scan_args(&[PathBuf::from("src")], &default_scan_opts(), None);
    assert_eq!(args.paths[0], "src");
    assert!(!args.excluded_redacted);
}

#[test]
fn redact_empty_exclusions_not_flagged() {
    let args = scan_args(
        &[PathBuf::from("src")],
        &default_scan_opts(),
        Some(RedactMode::Paths),
    );
    assert!(!args.excluded_redacted); // empty exclusions => not flagged
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn scan_args_deterministic_no_redact() {
    let opts = ScanOptions {
        excluded: vec!["vendor".to_string()],
        hidden: true,
        ..Default::default()
    };
    let a = scan_args(&[PathBuf::from("src")], &opts, None);
    let b = scan_args(&[PathBuf::from("src")], &opts, None);
    assert_eq!(a.paths, b.paths);
    assert_eq!(a.excluded, b.excluded);
    assert_eq!(a.hidden, b.hidden);
}

#[test]
fn scan_args_deterministic_with_redact() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };
    let a = scan_args(
        &[PathBuf::from("src/lib.rs")],
        &opts,
        Some(RedactMode::Paths),
    );
    let b = scan_args(
        &[PathBuf::from("src/lib.rs")],
        &opts,
        Some(RedactMode::Paths),
    );
    assert_eq!(a.paths, b.paths);
    assert_eq!(a.excluded, b.excluded);
}

#[test]
fn redacted_path_preserves_extension() {
    let args = scan_args(
        &[PathBuf::from("src/main.rs")],
        &default_scan_opts(),
        Some(RedactMode::Paths),
    );
    assert!(
        args.paths[0].ends_with(".rs"),
        "redacted path should preserve .rs extension, got: {}",
        args.paths[0]
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn dot_slash_path_normalized_before_redaction() {
    let a = scan_args(
        &[PathBuf::from("./src")],
        &default_scan_opts(),
        Some(RedactMode::Paths),
    );
    let b = scan_args(
        &[PathBuf::from("src")],
        &default_scan_opts(),
        Some(RedactMode::Paths),
    );
    assert_eq!(a.paths, b.paths, "./src and src should redact identically");
}

#[test]
fn config_mode_passthrough() {
    let opts = default_scan_opts();
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert_eq!(
        serde_json::to_string(&args.config).unwrap(),
        serde_json::to_string(&opts.config).unwrap()
    );
}
