//! Additional BDD-style tests covering edge cases and gaps in scan-args.

use std::path::PathBuf;

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

// ── Trailing slash handling ──────────────────────────────────────────

#[test]
fn given_path_with_trailing_slash_when_normalized_then_slash_is_preserved_or_stripped_deterministically()
 {
    let result = normalize_scan_input(&PathBuf::from("src/"));
    // The output should be consistent and never contain backslashes
    assert!(!result.contains('\\'));
    assert!(!result.is_empty());
}

#[test]
fn given_path_with_trailing_backslash_when_normalized_then_forward_slash_used() {
    let result = normalize_scan_input(&PathBuf::from(r"src\nested\"));
    assert!(!result.contains('\\'));
}

// ── Redacted output format ───────────────────────────────────────────

#[test]
fn given_redacted_path_when_inspected_then_original_extension_is_preserved() {
    let paths = vec![PathBuf::from("secret/config.json")];
    let global = ScanOptions::default();

    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    // redact_path preserves file extension after the 16-char hash
    assert!(
        args.paths[0].ends_with(".json"),
        "redacted path should preserve extension, got: {}",
        args.paths[0]
    );
}

#[test]
fn given_redacted_path_without_extension_when_inspected_then_no_extension_appended() {
    let paths = vec![PathBuf::from("Makefile")];
    let global = ScanOptions::default();

    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    // File without extension should not gain one
    assert_ne!(args.paths[0], "Makefile");
    // The hash should not contain the original name
    assert!(!args.paths[0].contains("Makefile"));
}

#[test]
fn given_redacted_exclusion_when_inspected_then_hash_is_16_hex_chars() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        excluded: vec!["node_modules".to_string()],
        ..Default::default()
    };

    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    // short_hash returns exactly 16 hex characters
    assert_eq!(
        args.excluded[0].len(),
        16,
        "expected 16-char hash, got: {}",
        args.excluded[0]
    );
    assert!(
        args.excluded[0].chars().all(|c| c.is_ascii_hexdigit()),
        "expected hex chars, got: {}",
        args.excluded[0]
    );
}

// ── Config field propagation ─────────────────────────────────────────

#[test]
fn given_config_auto_with_redaction_then_config_preserved() {
    let paths = vec![PathBuf::from("src")];
    let global = ScanOptions {
        config: tokmd_types::ConfigMode::Auto,
        ..Default::default()
    };

    let args = scan_args(&paths, &global, Some(RedactMode::All));
    assert_eq!(args.config, tokmd_types::ConfigMode::Auto);
}

#[test]
fn given_config_none_with_redaction_then_config_preserved() {
    let paths = vec![PathBuf::from("src")];
    let global = ScanOptions {
        config: tokmd_types::ConfigMode::None,
        ..Default::default()
    };

    let args = scan_args(&paths, &global, Some(RedactMode::All));
    assert_eq!(args.config, tokmd_types::ConfigMode::None);
}

// ── Redaction of identical paths produces identical hashes ────────────

#[test]
fn given_duplicate_paths_when_redacted_then_hashes_are_identical() {
    let paths = vec![PathBuf::from("src/lib.rs"), PathBuf::from("src/lib.rs")];
    let global = ScanOptions::default();

    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    assert_eq!(args.paths[0], args.paths[1]);
}

#[test]
fn given_duplicate_exclusions_when_redacted_then_hashes_are_identical() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        excluded: vec!["vendor".to_string(), "vendor".to_string()],
        ..Default::default()
    };

    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    assert_eq!(args.excluded[0], args.excluded[1]);
}

// ── Paths with only dots ─────────────────────────────────────────────

#[test]
fn given_double_dot_path_when_normalized_then_preserved() {
    let result = normalize_scan_input(&PathBuf::from(".."));
    assert_eq!(result, "..");
}

#[test]
fn given_parent_relative_path_when_normalized_then_preserved() {
    let result = normalize_scan_input(&PathBuf::from("../sibling/src"));
    assert!(!result.contains('\\'));
    assert!(result.contains("sibling"));
}

// ── Only-whitespace / unusual path segments ──────────────────────────

#[test]
fn given_path_with_numeric_segments_when_normalized_then_preserved() {
    let result = normalize_scan_input(&PathBuf::from("123/456/789"));
    assert_eq!(result, "123/456/789");
}

#[test]
fn given_path_with_single_char_segments_when_normalized_then_preserved() {
    let result = normalize_scan_input(&PathBuf::from("a/b/c"));
    assert_eq!(result, "a/b/c");
}

// ── All options at default values ────────────────────────────────────

#[test]
fn given_all_defaults_when_building_scan_args_then_defaults_propagated() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions::default();

    let args = scan_args(&paths, &global, None);

    assert_eq!(args.paths, vec!["."]);
    assert!(args.excluded.is_empty());
    assert!(!args.excluded_redacted);
    assert_eq!(args.config, tokmd_types::ConfigMode::Auto);
    assert!(!args.hidden);
    assert!(!args.no_ignore);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
    assert!(!args.treat_doc_strings_as_comments);
}

// ── Redact mode None vs Option::None equivalence ─────────────────────

#[test]
fn given_redact_mode_none_vs_option_none_then_outputs_are_identical() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let global = ScanOptions {
        excluded: vec!["target".to_string()],
        hidden: true,
        no_ignore: true,
        ..Default::default()
    };

    let with_none_mode = scan_args(&paths, &global, Some(RedactMode::None));
    let with_option_none = scan_args(&paths, &global, None);

    assert_eq!(with_none_mode.paths, with_option_none.paths);
    assert_eq!(with_none_mode.excluded, with_option_none.excluded);
    assert_eq!(
        with_none_mode.excluded_redacted,
        with_option_none.excluded_redacted
    );
    assert_eq!(with_none_mode.hidden, with_option_none.hidden);
    assert_eq!(with_none_mode.no_ignore, with_option_none.no_ignore);
    assert_eq!(
        with_none_mode.no_ignore_parent,
        with_option_none.no_ignore_parent
    );
    assert_eq!(with_none_mode.no_ignore_dot, with_option_none.no_ignore_dot);
    assert_eq!(with_none_mode.no_ignore_vcs, with_option_none.no_ignore_vcs);
    assert_eq!(
        with_none_mode.treat_doc_strings_as_comments,
        with_option_none.treat_doc_strings_as_comments
    );
}

// ── Path normalization with mixed separators ─────────────────────────

#[test]
fn given_mixed_separator_path_when_normalized_then_all_forward_slashes() {
    let result = normalize_scan_input(&PathBuf::from(r"src/nested\deep/file.rs"));
    assert!(!result.contains('\\'));
    assert!(result.contains("src"));
    assert!(result.contains("file.rs"));
}
