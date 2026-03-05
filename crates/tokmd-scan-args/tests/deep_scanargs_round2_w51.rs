//! Deep round-2 tests for tokmd-scan-args (W51).

use std::path::{Path, PathBuf};

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

// ---------------------------------------------------------------------------
// ScanArgs construction
// ---------------------------------------------------------------------------

#[test]
fn scan_args_single_dot_path() {
    let args = scan_args(&[PathBuf::from(".")], &ScanOptions::default(), None);
    assert_eq!(args.paths, vec!["."]);
}

#[test]
fn scan_args_multiple_paths() {
    let paths = vec![PathBuf::from("src"), PathBuf::from("tests")];
    let args = scan_args(&paths, &ScanOptions::default(), None);
    assert_eq!(args.paths, vec!["src", "tests"]);
}

#[test]
fn scan_args_preserves_exclusions_without_redaction() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string(), "dist".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert_eq!(args.excluded, vec!["target", "dist"]);
    assert!(!args.excluded_redacted);
}

#[test]
fn scan_args_propagates_boolean_flags() {
    let opts = ScanOptions {
        hidden: true,
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.hidden);
    assert!(args.treat_doc_strings_as_comments);
}

// ---------------------------------------------------------------------------
// Path normalization
// ---------------------------------------------------------------------------

#[test]
fn normalize_backslashes_to_forward() {
    let result = normalize_scan_input(Path::new("src\\lib.rs"));
    assert_eq!(result, "src/lib.rs");
}

#[test]
fn normalize_strips_dot_slash_prefix() {
    let result = normalize_scan_input(Path::new("./src/main.rs"));
    assert_eq!(result, "src/main.rs");
}

#[test]
fn normalize_bare_dot_preserved() {
    let result = normalize_scan_input(Path::new("."));
    assert_eq!(result, ".");
}

// ---------------------------------------------------------------------------
// Redaction wiring
// ---------------------------------------------------------------------------

#[test]
fn redact_paths_mode_hashes_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::Paths));
    assert_ne!(args.paths[0], "src/lib.rs", "path should be redacted");
    assert!(!args.paths[0].is_empty());
}

#[test]
fn redact_all_mode_hashes_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::All));
    assert_ne!(args.paths[0], "src/lib.rs", "path should be redacted");
}

#[test]
fn redact_none_mode_preserves_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::None));
    assert_eq!(args.paths[0], "src/lib.rs");
}

#[test]
fn redact_exclusions_sets_flag() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, Some(RedactMode::Paths));
    assert!(args.excluded_redacted);
    assert_ne!(args.excluded[0], "target");
}

#[test]
fn redact_empty_exclusions_flag_stays_false() {
    let args = scan_args(
        &[PathBuf::from(".")],
        &ScanOptions::default(),
        Some(RedactMode::Paths),
    );
    assert!(
        !args.excluded_redacted,
        "no exclusions means flag stays false"
    );
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn deterministic_same_input_same_output() {
    let paths = vec![PathBuf::from("src/main.rs"), PathBuf::from("tests/")];
    let opts = ScanOptions {
        excluded: vec!["target".to_string()],
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
fn deterministic_redaction_hash_stable() {
    let paths = vec![PathBuf::from("my/secret/path.rs")];
    let a = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::Paths));
    let b = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::Paths));
    assert_eq!(a.paths[0], b.paths[0], "same input must produce same hash");
}

// ---------------------------------------------------------------------------
// no_ignore flag composition
// ---------------------------------------------------------------------------

#[test]
fn no_ignore_forces_all_sub_flags() {
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
fn sub_flags_independent_without_no_ignore() {
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
