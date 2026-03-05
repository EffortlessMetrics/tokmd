//! W47 deep tests for `tokmd-scan-args`.
//!
//! Covers: ScanArgs construction with all fields, deterministic construction,
//! redaction integration, serde roundtrips, path normalization, and
//! property-based invariants.

use std::path::{Path, PathBuf};

use proptest::prelude::*;
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
// 1. ScanArgs construction with all fields
// ===========================================================================

#[test]
fn scan_args_all_fields_populated() {
    let paths = vec![PathBuf::from("src/lib.rs"), PathBuf::from("tests")];
    let opts = ScanOptions {
        excluded: vec!["target".into(), "dist".into()],
        config: ConfigMode::Auto,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };

    let args = scan_args(&paths, &opts, None);

    assert_eq!(args.paths, vec!["src/lib.rs", "tests"]);
    assert_eq!(args.excluded, vec!["target", "dist"]);
    assert!(!args.excluded_redacted);
    assert_eq!(args.config, ConfigMode::Auto);
    assert!(args.hidden);
    assert!(args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
    assert!(args.treat_doc_strings_as_comments);
}

#[test]
fn scan_args_minimal_fields() {
    let args = scan_args(&[], &default_opts(), None);
    assert!(args.paths.is_empty());
    assert!(args.excluded.is_empty());
    assert!(!args.excluded_redacted);
    assert!(!args.hidden);
    assert!(!args.no_ignore);
}

// ===========================================================================
// 2. Deterministic construction (same paths → same args)
// ===========================================================================

#[test]
fn deterministic_without_redaction() {
    let paths = vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")];
    let mut opts = default_opts();
    opts.excluded = vec!["target".into()];

    let a = serde_json::to_string(&scan_args(&paths, &opts, None)).unwrap();
    let b = serde_json::to_string(&scan_args(&paths, &opts, None)).unwrap();
    assert_eq!(a, b);
}

#[test]
fn deterministic_with_redaction() {
    let paths = vec![PathBuf::from("secret/src/lib.rs")];
    let mut opts = default_opts();
    opts.excluded = vec!["private".into()];

    let a = serde_json::to_string(&scan_args(&paths, &opts, Some(RedactMode::Paths))).unwrap();
    let b = serde_json::to_string(&scan_args(&paths, &opts, Some(RedactMode::Paths))).unwrap();
    assert_eq!(a, b);
}

#[test]
fn deterministic_with_all_redaction() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = default_opts();

    let a = serde_json::to_string(&scan_args(&paths, &opts, Some(RedactMode::All))).unwrap();
    let b = serde_json::to_string(&scan_args(&paths, &opts, Some(RedactMode::All))).unwrap();
    assert_eq!(a, b);
}

// ===========================================================================
// 3. Redaction integration
// ===========================================================================

#[test]
fn redaction_paths_mode_hashes_paths() {
    let paths = vec![PathBuf::from("secret/src/lib.rs")];
    let args = scan_args(&paths, &default_opts(), Some(RedactMode::Paths));
    assert_ne!(args.paths[0], "secret/src/lib.rs");
}

#[test]
fn redaction_all_mode_hashes_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let args = scan_args(&paths, &default_opts(), Some(RedactMode::All));
    assert_ne!(args.paths[0], "src/lib.rs");
}

#[test]
fn redaction_none_mode_preserves_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let args = scan_args(&paths, &default_opts(), Some(RedactMode::None));
    assert_eq!(args.paths[0], "src/lib.rs");
}

#[test]
fn redaction_option_none_preserves_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let args = scan_args(&paths, &default_opts(), None);
    assert_eq!(args.paths[0], "src/lib.rs");
}

#[test]
fn redaction_hashes_exclusions() {
    let mut opts = default_opts();
    opts.excluded = vec!["secret_dir".into()];
    let args = scan_args(&dot_paths(), &opts, Some(RedactMode::Paths));
    assert_ne!(args.excluded[0], "secret_dir");
    assert!(args.excluded_redacted);
}

#[test]
fn redaction_empty_exclusions_not_marked() {
    let args = scan_args(&dot_paths(), &default_opts(), Some(RedactMode::Paths));
    assert!(!args.excluded_redacted);
}

#[test]
fn redaction_preserves_extension() {
    let paths = vec![PathBuf::from("secret.rs")];
    let args = scan_args(&paths, &default_opts(), Some(RedactMode::Paths));
    assert!(
        args.paths[0].ends_with(".rs"),
        "expected .rs extension in: {}",
        args.paths[0]
    );
}

#[test]
fn redaction_different_inputs_different_hashes() {
    let p1 = vec![PathBuf::from("src/a.rs")];
    let p2 = vec![PathBuf::from("src/b.rs")];
    let opts = default_opts();
    let a = scan_args(&p1, &opts, Some(RedactMode::Paths));
    let b = scan_args(&p2, &opts, Some(RedactMode::Paths));
    assert_ne!(a.paths[0], b.paths[0]);
}

// ===========================================================================
// 4. Property: ScanArgs roundtrips through serde
// ===========================================================================

#[test]
fn serde_roundtrip_no_redaction() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let mut opts = default_opts();
    opts.excluded = vec!["target".into()];
    opts.hidden = true;
    opts.config = ConfigMode::Auto;

    let args = scan_args(&paths, &opts, None);
    let json = serde_json::to_string(&args).unwrap();
    let back: ScanArgs = serde_json::from_str(&json).unwrap();

    assert_eq!(back.paths, args.paths);
    assert_eq!(back.excluded, args.excluded);
    assert_eq!(back.hidden, args.hidden);
    assert_eq!(back.config, args.config);
    assert_eq!(back.no_ignore, args.no_ignore);
    assert_eq!(
        back.treat_doc_strings_as_comments,
        args.treat_doc_strings_as_comments
    );
}

#[test]
fn serde_roundtrip_with_redaction() {
    let paths = vec![PathBuf::from("secret/src")];
    let mut opts = default_opts();
    opts.excluded = vec!["private".into()];
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));

    let json = serde_json::to_string(&args).unwrap();
    let back: ScanArgs = serde_json::from_str(&json).unwrap();

    assert_eq!(back.paths, args.paths);
    assert!(back.excluded_redacted);
}

#[test]
fn serde_excluded_redacted_false_omitted() {
    let args = scan_args(&dot_paths(), &default_opts(), None);
    let json = serde_json::to_string(&args).unwrap();
    assert!(
        !json.contains("excluded_redacted"),
        "excluded_redacted=false should be omitted: {json}"
    );
}

#[test]
fn serde_excluded_redacted_true_present() {
    let mut opts = default_opts();
    opts.excluded = vec!["secret".into()];
    let args = scan_args(&dot_paths(), &opts, Some(RedactMode::Paths));
    let json = serde_json::to_string(&args).unwrap();
    assert!(
        json.contains("excluded_redacted"),
        "excluded_redacted=true should appear: {json}"
    );
}

// ===========================================================================
// 5. Path normalization in scan args
// ===========================================================================

#[test]
fn normalize_strips_dot_slash() {
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
fn normalize_bare_dot() {
    assert_eq!(normalize_scan_input(Path::new(".")), ".");
}

#[test]
fn normalize_bare_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("./")), ".");
}

#[test]
fn normalize_backslash_to_forward() {
    assert_eq!(
        normalize_scan_input(Path::new("src\\main.rs")),
        "src/main.rs"
    );
}

#[test]
fn normalize_empty_becomes_dot() {
    assert_eq!(normalize_scan_input(Path::new("")), ".");
}

#[test]
fn normalize_idempotent() {
    let once = normalize_scan_input(Path::new("./src/lib.rs"));
    let twice = normalize_scan_input(Path::new(&once));
    assert_eq!(once, twice);
}

#[test]
fn scan_args_normalizes_backslash_in_paths() {
    let paths = vec![PathBuf::from(r".\src\main.rs")];
    let args = scan_args(&paths, &default_opts(), None);
    assert_eq!(args.paths, vec!["src/main.rs"]);
}

// ===========================================================================
// 6. Flag propagation
// ===========================================================================

#[test]
fn no_ignore_implies_all_sub_flags() {
    let mut opts = default_opts();
    opts.no_ignore = true;
    let args = scan_args(&dot_paths(), &opts, None);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

#[test]
fn individual_sub_flags_independent() {
    let mut opts = default_opts();
    opts.no_ignore_parent = true;
    let args = scan_args(&dot_paths(), &opts, None);
    assert!(args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
}

// ===========================================================================
// 7. Property tests
// ===========================================================================

fn pathish_string() -> impl Strategy<Value = String> {
    let alphabet: Vec<char> = "/\\._abcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    prop::collection::vec(prop::sample::select(alphabet), 0..64)
        .prop_map(|chars| chars.into_iter().collect())
}

fn redact_mode_strategy() -> impl Strategy<Value = Option<RedactMode>> {
    prop_oneof![
        Just(None),
        Just(Some(RedactMode::None)),
        Just(Some(RedactMode::Paths)),
        Just(Some(RedactMode::All)),
    ]
}

proptest! {
    #[test]
    fn prop_serde_roundtrip(
        path_values in prop::collection::vec(pathish_string(), 0..4),
        excluded in prop::collection::vec(pathish_string(), 0..4),
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        redact in redact_mode_strategy(),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let opts = ScanOptions {
            excluded,
            hidden,
            no_ignore,
            ..Default::default()
        };

        let args = scan_args(&paths, &opts, redact);
        let json = serde_json::to_string(&args).unwrap();
        let back: ScanArgs = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(args.paths, back.paths);
        prop_assert_eq!(args.excluded, back.excluded);
        prop_assert_eq!(args.hidden, back.hidden);
        prop_assert_eq!(args.no_ignore, back.no_ignore);
    }

    #[test]
    fn prop_normalize_no_backslash(input in pathish_string()) {
        let normalized = normalize_scan_input(Path::new(&input));
        prop_assert!(!normalized.contains('\\'));
    }

    #[test]
    fn prop_normalize_never_empty(input in pathish_string()) {
        let normalized = normalize_scan_input(Path::new(&input));
        prop_assert!(!normalized.is_empty());
    }

    #[test]
    fn prop_normalize_idempotent(input in pathish_string()) {
        let once = normalize_scan_input(Path::new(&input));
        let twice = normalize_scan_input(Path::new(&once));
        prop_assert_eq!(once, twice);
    }

    #[test]
    fn prop_path_count_preserved(
        path_values in prop::collection::vec(pathish_string(), 0..8),
        redact in redact_mode_strategy(),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let args = scan_args(&paths, &default_opts(), redact);
        prop_assert_eq!(args.paths.len(), paths.len());
    }

    #[test]
    fn prop_exclusion_count_preserved(
        excluded in prop::collection::vec(pathish_string(), 0..8),
        redact in redact_mode_strategy(),
    ) {
        let opts = ScanOptions { excluded: excluded.clone(), ..Default::default() };
        let args = scan_args(&dot_paths(), &opts, redact);
        prop_assert_eq!(args.excluded.len(), excluded.len());
    }

    #[test]
    fn prop_deterministic(
        path in pathish_string(),
        redact in redact_mode_strategy(),
    ) {
        let paths = vec![PathBuf::from(&path)];
        let opts = default_opts();
        let a = scan_args(&paths, &opts, redact);
        let b = scan_args(&paths, &opts, redact);
        prop_assert_eq!(a.paths, b.paths);
        prop_assert_eq!(a.excluded, b.excluded);
    }
}
