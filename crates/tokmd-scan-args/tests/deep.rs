//! Deep tests for `tokmd-scan-args`.
//!
//! Exercises `scan_args()` and `normalize_scan_input()` across redaction modes,
//! flag propagation, serialization roundtrips, determinism, and edge cases.

use std::path::{Path, PathBuf};

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::{ConfigMode, RedactMode, ScanArgs};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_opts() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: ConfigMode::None,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

fn dot_paths() -> Vec<PathBuf> {
    vec![PathBuf::from(".")]
}

// ===========================================================================
// 1. normalize_scan_input
// ===========================================================================

#[test]
fn normalize_strips_single_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("./src")), "src");
}

#[test]
fn normalize_strips_repeated_dot_slash() {
    assert_eq!(
        normalize_scan_input(Path::new("././src/lib.rs")),
        "src/lib.rs"
    );
}

#[test]
fn normalize_keeps_dot_for_bare_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("./")), ".");
}

#[test]
fn normalize_keeps_dot_for_bare_dot() {
    assert_eq!(normalize_scan_input(Path::new(".")), ".");
}

#[test]
fn normalize_converts_backslashes_to_forward() {
    assert_eq!(
        normalize_scan_input(Path::new("src\\main.rs")),
        "src/main.rs"
    );
}

#[test]
fn normalize_handles_absolute_path() {
    let result = normalize_scan_input(Path::new("/home/user/project"));
    assert!(!result.contains('\\'));
    assert!(result.contains("home"));
}

#[test]
fn normalize_is_idempotent() {
    let once = normalize_scan_input(Path::new("./src/lib.rs"));
    let twice = normalize_scan_input(Path::new(&once));
    assert_eq!(once, twice);
}

#[test]
fn normalize_preserves_deep_path() {
    assert_eq!(normalize_scan_input(Path::new("a/b/c/d/e")), "a/b/c/d/e");
}

#[test]
fn normalize_empty_string_becomes_dot() {
    assert_eq!(normalize_scan_input(Path::new("")), ".");
}

// ===========================================================================
// 2. scan_args construction — no redaction
// ===========================================================================

#[test]
fn scan_args_none_redact_preserves_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let args = scan_args(&paths, &default_opts(), None);
    assert_eq!(args.paths, vec!["src/lib.rs"]);
}

#[test]
fn scan_args_none_redact_preserves_exclusions() {
    let mut opts = default_opts();
    opts.excluded = vec!["target".to_string()];
    let args = scan_args(&dot_paths(), &opts, None);
    assert_eq!(args.excluded, vec!["target"]);
    assert!(!args.excluded_redacted);
}

#[test]
fn scan_args_redact_mode_none_preserves_paths() {
    let paths = vec![PathBuf::from("src/main.rs")];
    let args = scan_args(&paths, &default_opts(), Some(RedactMode::None));
    assert_eq!(args.paths, vec!["src/main.rs"]);
}

#[test]
fn scan_args_normalizes_backslash_paths() {
    let paths = vec![PathBuf::from(r".\src\main.rs")];
    let args = scan_args(&paths, &default_opts(), None);
    assert_eq!(args.paths, vec!["src/main.rs"]);
}

// ===========================================================================
// 3. Redaction wiring
// ===========================================================================

#[test]
fn redact_paths_hashes_scan_paths() {
    let paths = vec![PathBuf::from("secret/src/lib.rs")];
    let args = scan_args(&paths, &default_opts(), Some(RedactMode::Paths));
    assert_ne!(args.paths[0], "secret/src/lib.rs");
    // redact_path preserves extension: hash.ext — verify hex portion
    let hex_part = args.paths[0].split('.').next().unwrap();
    assert!(
        hex_part.chars().all(|c| c.is_ascii_hexdigit()),
        "hex portion should be hex: {}",
        args.paths[0]
    );
}

#[test]
fn redact_all_hashes_scan_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let args = scan_args(&paths, &default_opts(), Some(RedactMode::All));
    assert_ne!(args.paths[0], "src/lib.rs");
}

#[test]
fn redact_paths_hashes_exclusions() {
    let mut opts = default_opts();
    opts.excluded = vec!["**/private/**".to_string()];
    let args = scan_args(&dot_paths(), &opts, Some(RedactMode::Paths));
    assert_ne!(args.excluded[0], "**/private/**");
    assert!(args.excluded_redacted);
}

#[test]
fn redact_all_hashes_exclusions() {
    let mut opts = default_opts();
    opts.excluded = vec!["secret_dir".to_string()];
    let args = scan_args(&dot_paths(), &opts, Some(RedactMode::All));
    assert_ne!(args.excluded[0], "secret_dir");
    assert!(args.excluded_redacted);
}

#[test]
fn redact_with_empty_exclusions_not_marked_redacted() {
    let args = scan_args(&dot_paths(), &default_opts(), Some(RedactMode::Paths));
    assert!(!args.excluded_redacted);
}

#[test]
fn redact_is_deterministic() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = default_opts();
    let a1 = scan_args(&paths, &opts, Some(RedactMode::Paths));
    let a2 = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_eq!(a1.paths, a2.paths);
}

#[test]
fn different_paths_produce_different_hashes() {
    let p1 = vec![PathBuf::from("src/a.rs")];
    let p2 = vec![PathBuf::from("src/b.rs")];
    let opts = default_opts();
    let a1 = scan_args(&p1, &opts, Some(RedactMode::Paths));
    let a2 = scan_args(&p2, &opts, Some(RedactMode::Paths));
    assert_ne!(
        a1.paths[0], a2.paths[0],
        "different inputs -> different hashes"
    );
}

// ===========================================================================
// 4. Flag propagation
// ===========================================================================

#[test]
fn no_ignore_implies_all_sub_flags() {
    let mut opts = default_opts();
    opts.no_ignore = true;
    let args = scan_args(&dot_paths(), &opts, None);
    assert!(args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

#[test]
fn individual_sub_flags_propagated() {
    let mut opts = default_opts();
    opts.no_ignore_parent = true;
    opts.no_ignore_dot = false;
    opts.no_ignore_vcs = false;
    let args = scan_args(&dot_paths(), &opts, None);
    assert!(args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
}

#[test]
fn hidden_flag_propagated() {
    let mut opts = default_opts();
    opts.hidden = true;
    let args = scan_args(&dot_paths(), &opts, None);
    assert!(args.hidden);
}

#[test]
fn treat_doc_strings_flag_propagated() {
    let mut opts = default_opts();
    opts.treat_doc_strings_as_comments = true;
    let args = scan_args(&dot_paths(), &opts, None);
    assert!(args.treat_doc_strings_as_comments);
}

#[test]
fn config_mode_auto_propagated() {
    let mut opts = default_opts();
    opts.config = ConfigMode::Auto;
    let args = scan_args(&dot_paths(), &opts, None);
    assert_eq!(args.config, ConfigMode::Auto);
}

#[test]
fn config_mode_none_propagated() {
    let opts = default_opts(); // already None
    let args = scan_args(&dot_paths(), &opts, None);
    assert_eq!(args.config, ConfigMode::None);
}

#[test]
fn all_false_flags_propagated() {
    let opts = default_opts();
    let args = scan_args(&dot_paths(), &opts, None);
    assert!(!args.hidden);
    assert!(!args.no_ignore);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
    assert!(!args.treat_doc_strings_as_comments);
}

// ===========================================================================
// 5. Serialization roundtrip
// ===========================================================================

#[test]
fn scan_args_serialization_roundtrip() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let mut opts = default_opts();
    opts.excluded = vec!["target".to_string()];
    opts.hidden = true;
    let args = scan_args(&paths, &opts, None);

    let json = serde_json::to_string(&args).expect("serialize");
    let back: ScanArgs = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(back.paths, args.paths);
    assert_eq!(back.excluded, args.excluded);
    assert_eq!(back.hidden, args.hidden);
    assert_eq!(back.config, args.config);
}

#[test]
fn scan_args_redacted_serialization_roundtrip() {
    let paths = vec![PathBuf::from("secret/src")];
    let mut opts = default_opts();
    opts.excluded = vec!["private".to_string()];
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));

    let json = serde_json::to_string(&args).expect("serialize");
    let back: ScanArgs = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(back.paths, args.paths);
    assert_eq!(back.excluded_redacted, args.excluded_redacted);
    assert!(back.excluded_redacted);
}

#[test]
fn excluded_redacted_false_skipped_in_serialization() {
    let args = scan_args(&dot_paths(), &default_opts(), None);
    let json = serde_json::to_string(&args).expect("serialize");
    // excluded_redacted is skip_serializing_if = "Not::not" (i.e., false)
    assert!(
        !json.contains("excluded_redacted"),
        "excluded_redacted=false should be omitted from JSON: {json}"
    );
}

#[test]
fn excluded_redacted_true_present_in_serialization() {
    let mut opts = default_opts();
    opts.excluded = vec!["secret".to_string()];
    let args = scan_args(&dot_paths(), &opts, Some(RedactMode::Paths));
    let json = serde_json::to_string(&args).expect("serialize");
    assert!(
        json.contains("excluded_redacted"),
        "excluded_redacted=true should appear in JSON: {json}"
    );
}

// ===========================================================================
// 6. Deterministic output
// ===========================================================================

#[test]
fn scan_args_deterministic_without_redaction() {
    let paths = vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")];
    let mut opts = default_opts();
    opts.excluded = vec!["target".to_string()];

    let a = scan_args(&paths, &opts, None);
    let b = scan_args(&paths, &opts, None);

    let json_a = serde_json::to_string(&a).unwrap();
    let json_b = serde_json::to_string(&b).unwrap();
    assert_eq!(json_a, json_b, "same inputs -> identical JSON");
}

#[test]
fn scan_args_deterministic_with_redaction() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let mut opts = default_opts();
    opts.excluded = vec!["node_modules".to_string()];

    let a = scan_args(&paths, &opts, Some(RedactMode::All));
    let b = scan_args(&paths, &opts, Some(RedactMode::All));

    let json_a = serde_json::to_string(&a).unwrap();
    let json_b = serde_json::to_string(&b).unwrap();
    assert_eq!(json_a, json_b, "same inputs + redaction -> identical JSON");
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn empty_paths_slice() {
    let args = scan_args(&[], &default_opts(), None);
    assert!(args.paths.is_empty());
}

#[test]
fn empty_paths_with_redaction() {
    let args = scan_args(&[], &default_opts(), Some(RedactMode::Paths));
    assert!(args.paths.is_empty());
    assert!(!args.excluded_redacted); // no exclusions => not marked
}

#[test]
fn many_paths() {
    let paths: Vec<PathBuf> = (0..100)
        .map(|i| PathBuf::from(format!("dir{i}/file.rs")))
        .collect();
    let args = scan_args(&paths, &default_opts(), None);
    assert_eq!(args.paths.len(), 100);
}

#[test]
fn many_exclusions() {
    let mut opts = default_opts();
    opts.excluded = (0..50).map(|i| format!("exclude_{i}")).collect();
    let args = scan_args(&dot_paths(), &opts, None);
    assert_eq!(args.excluded.len(), 50);
}

#[test]
fn many_exclusions_redacted() {
    let mut opts = default_opts();
    opts.excluded = (0..50).map(|i| format!("secret_{i}")).collect();
    let args = scan_args(&dot_paths(), &opts, Some(RedactMode::Paths));
    assert_eq!(args.excluded.len(), 50);
    assert!(args.excluded_redacted);
    // All exclusions should be hex hashes
    for exc in &args.excluded {
        assert!(
            exc.chars().all(|c| c.is_ascii_hexdigit()),
            "expected hex hash, got: {exc}"
        );
    }
}

#[test]
fn paths_with_spaces() {
    let paths = vec![PathBuf::from("my project/src/lib.rs")];
    let args = scan_args(&paths, &default_opts(), None);
    assert_eq!(args.paths[0], "my project/src/lib.rs");
}

#[test]
fn paths_with_unicode() {
    let paths = vec![PathBuf::from("проект/src/lib.rs")];
    let args = scan_args(&paths, &default_opts(), None);
    assert!(args.paths[0].contains("src/lib.rs"));
}

#[test]
fn redaction_of_path_with_extension_preserves_extension() {
    let paths = vec![PathBuf::from("secret.rs")];
    let args = scan_args(&paths, &default_opts(), Some(RedactMode::Paths));
    assert!(
        args.paths[0].ends_with(".rs"),
        "redacted path should preserve .rs extension: {}",
        args.paths[0]
    );
}

#[test]
fn redaction_of_path_without_extension() {
    let paths = vec![PathBuf::from("Makefile")];
    let args = scan_args(&paths, &default_opts(), Some(RedactMode::Paths));
    // No extension to preserve, should be pure hex
    assert!(
        args.paths[0].chars().all(|c| c.is_ascii_hexdigit()),
        "path without extension should be pure hex: {}",
        args.paths[0]
    );
}

#[test]
fn redact_mode_none_vs_option_none() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = default_opts();

    let with_none_option = scan_args(&paths, &opts, None);
    let with_none_mode = scan_args(&paths, &opts, Some(RedactMode::None));

    // Both should preserve paths
    assert_eq!(with_none_option.paths, with_none_mode.paths);
    assert_eq!(with_none_option.paths[0], "src/lib.rs");
}
