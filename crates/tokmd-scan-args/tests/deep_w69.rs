//! Deep tests for tokmd-scan-args — W69
//!
//! Covers: normalize_scan_input, scan_args construction, redaction
//! wiring, flag propagation, and determinism.

use std::path::{Path, PathBuf};

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

// ── normalize_scan_input ────────────────────────────────────────────

#[test]
fn normalize_dot_only() {
    assert_eq!(normalize_scan_input(Path::new(".")), ".");
}

#[test]
fn normalize_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("./")), ".");
}

#[test]
fn normalize_double_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("././")), ".");
}

#[test]
fn normalize_strips_leading_dot_slash_deeply() {
    assert_eq!(
        normalize_scan_input(Path::new("./././src/lib.rs")),
        "src/lib.rs"
    );
}

#[test]
fn normalize_preserves_parent_dir() {
    assert_eq!(normalize_scan_input(Path::new("../src")), "../src");
}

#[test]
fn normalize_absolute_path_unchanged() {
    // On Windows this will include the drive letter; just verify no crash
    let p = Path::new("/usr/local/src");
    let n = normalize_scan_input(p);
    assert!(!n.is_empty());
    assert!(!n.contains('\\'));
}

#[test]
fn normalize_forward_slash_preserved() {
    assert_eq!(
        normalize_scan_input(Path::new("a/b/c")),
        "a/b/c"
    );
}

// ── scan_args — basic construction ──────────────────────────────────

fn default_opts() -> ScanOptions {
    ScanOptions::default()
}

#[test]
fn scan_args_single_path() {
    let args = scan_args(&[PathBuf::from("src")], &default_opts(), None);
    assert_eq!(args.paths, vec!["src"]);
    assert!(args.excluded.is_empty());
    assert!(!args.excluded_redacted);
}

#[test]
fn scan_args_multiple_paths() {
    let paths = vec![PathBuf::from("src"), PathBuf::from("tests")];
    let args = scan_args(&paths, &default_opts(), None);
    assert_eq!(args.paths, vec!["src", "tests"]);
}

#[test]
fn scan_args_dot_path_normalized() {
    let args = scan_args(&[PathBuf::from("./")], &default_opts(), None);
    assert_eq!(args.paths, vec!["."]);
}

// ── scan_args — flag propagation ────────────────────────────────────

#[test]
fn scan_args_hidden_flag_propagated() {
    let opts = ScanOptions {
        hidden: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.hidden);
}

#[test]
fn scan_args_treat_doc_strings_propagated() {
    let opts = ScanOptions {
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.treat_doc_strings_as_comments);
}

#[test]
fn scan_args_no_ignore_cascades_to_sub_flags() {
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
fn scan_args_individual_sub_flags_independent() {
    let opts = ScanOptions {
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(!args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
}

// ── scan_args — redaction ───────────────────────────────────────────

#[test]
fn scan_args_redact_none_keeps_paths_verbatim() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from("src")], &opts, Some(RedactMode::None));
    assert_eq!(args.paths, vec!["src"]);
    assert_eq!(args.excluded, vec!["target"]);
    assert!(!args.excluded_redacted);
}

#[test]
fn scan_args_redact_paths_hides_paths_and_exclusions() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string(), "vendor".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from("src/lib.rs")], &opts, Some(RedactMode::Paths));
    assert_ne!(args.paths[0], "src/lib.rs", "path should be redacted");
    assert_ne!(args.excluded[0], "target", "exclusion should be redacted");
    assert_ne!(args.excluded[1], "vendor", "exclusion should be redacted");
    assert!(args.excluded_redacted);
}

#[test]
fn scan_args_redact_all_hides_paths_and_exclusions() {
    let opts = ScanOptions {
        excluded: vec!["build".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from("lib")], &opts, Some(RedactMode::All));
    assert_ne!(args.paths[0], "lib");
    assert_ne!(args.excluded[0], "build");
    assert!(args.excluded_redacted);
}

#[test]
fn scan_args_redact_without_excluded_not_flagged() {
    let opts = default_opts(); // no exclusions
    let args = scan_args(&[PathBuf::from("src")], &opts, Some(RedactMode::Paths));
    assert!(!args.excluded_redacted, "no exclusions means flag stays false");
}

// ── determinism ─────────────────────────────────────────────────────

#[test]
fn scan_args_deterministic_across_calls() {
    let paths = vec![PathBuf::from("a"), PathBuf::from("b")];
    let opts = ScanOptions {
        excluded: vec!["x".to_string()],
        hidden: true,
        ..Default::default()
    };
    let a = scan_args(&paths, &opts, Some(RedactMode::Paths));
    let b = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_eq!(a.paths, b.paths);
    assert_eq!(a.excluded, b.excluded);
    assert_eq!(a.excluded_redacted, b.excluded_redacted);
}

#[test]
fn scan_args_redaction_is_deterministic() {
    let opts = ScanOptions {
        excluded: vec!["secret_dir".to_string()],
        ..Default::default()
    };
    let a = scan_args(&[PathBuf::from("my/path")], &opts, Some(RedactMode::Paths));
    let b = scan_args(&[PathBuf::from("my/path")], &opts, Some(RedactMode::Paths));
    assert_eq!(a.paths, b.paths);
    assert_eq!(a.excluded, b.excluded);
}

// ── JSON round-trip ─────────────────────────────────────────────────

#[test]
fn scan_args_json_round_trip() {
    let opts = ScanOptions {
        excluded: vec!["dist".to_string()],
        hidden: true,
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from("src")], &opts, None);
    let json = serde_json::to_string(&args).unwrap();
    let back: tokmd_types::ScanArgs = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths, args.paths);
    assert_eq!(back.excluded, args.excluded);
    assert_eq!(back.hidden, args.hidden);
    assert_eq!(back.treat_doc_strings_as_comments, args.treat_doc_strings_as_comments);
}

#[test]
fn scan_args_excluded_redacted_skipped_when_false() {
    let args = scan_args(&[PathBuf::from(".")], &default_opts(), None);
    let json = serde_json::to_string(&args).unwrap();
    assert!(
        !json.contains("excluded_redacted"),
        "excluded_redacted should be skip_serializing_if false"
    );
}
