use std::path::{Path, PathBuf};

use proptest::prelude::*;
use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

fn pathish_string() -> impl Strategy<Value = String> {
    let alphabet: Vec<char> = "/\\._abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
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
    fn normalize_scan_input_never_contains_backslashes(input in pathish_string()) {
        let normalized = normalize_scan_input(Path::new(&input));
        prop_assert!(!normalized.contains('\\'));
    }

    #[test]
    fn normalize_scan_input_is_idempotent(input in pathish_string()) {
        let once = normalize_scan_input(Path::new(&input));
        let twice = normalize_scan_input(Path::new(&once));
        prop_assert_eq!(once, twice);
    }

    #[test]
    fn scan_args_is_deterministic_for_same_input(
        path_values in prop::collection::vec(pathish_string(), 0..8),
        excluded in prop::collection::vec(pathish_string(), 0..8),
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        no_ignore_parent in any::<bool>(),
        no_ignore_dot in any::<bool>(),
        no_ignore_vcs in any::<bool>(),
        treat_doc_strings_as_comments in any::<bool>(),
        redact in redact_mode_strategy(),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions {
            excluded,
            hidden,
            no_ignore,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            treat_doc_strings_as_comments,
            ..Default::default()
        };

        let a = scan_args(&paths, &scan_options, redact);
        let b = scan_args(&paths, &scan_options, redact);

        prop_assert_eq!(a.paths, b.paths);
        prop_assert_eq!(a.excluded, b.excluded);
        prop_assert_eq!(a.excluded_redacted, b.excluded_redacted);
        prop_assert_eq!(a.hidden, b.hidden);
        prop_assert_eq!(a.no_ignore, b.no_ignore);
        prop_assert_eq!(a.no_ignore_parent, b.no_ignore_parent);
        prop_assert_eq!(a.no_ignore_dot, b.no_ignore_dot);
        prop_assert_eq!(a.no_ignore_vcs, b.no_ignore_vcs);
        prop_assert_eq!(a.treat_doc_strings_as_comments, b.treat_doc_strings_as_comments);
    }

    #[test]
    fn scan_args_excluded_redacted_matches_mode_and_exclusions(
        path_values in prop::collection::vec(pathish_string(), 0..8),
        excluded in prop::collection::vec(pathish_string(), 0..8),
        redact in redact_mode_strategy(),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions {
            excluded: excluded.clone(),
            ..Default::default()
        };

        let args = scan_args(&paths, &scan_options, redact);
        let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
        let expected = should_redact && !excluded.is_empty();
        prop_assert_eq!(args.excluded_redacted, expected);
    }

    #[test]
    fn scan_args_without_redaction_keeps_normalized_paths(
        path_values in prop::collection::vec(pathish_string(), 0..8),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions::default();

        let args = scan_args(&paths, &scan_options, Some(RedactMode::None));
        let expected: Vec<String> = paths
            .iter()
            .map(|p| normalize_scan_input(p))
            .collect();

        prop_assert_eq!(args.paths, expected);
    }

    // ── Normalization properties ─────────────────────────────────────

    #[test]
    fn normalize_scan_input_never_starts_with_dot_slash(input in pathish_string()) {
        let normalized = normalize_scan_input(Path::new(&input));
        prop_assert!(!normalized.starts_with("./"));
    }

    #[test]
    fn normalize_scan_input_never_empty(input in pathish_string()) {
        let normalized = normalize_scan_input(Path::new(&input));
        prop_assert!(!normalized.is_empty());
    }

    // ── Redaction properties ─────────────────────────────────────────

    #[test]
    fn scan_args_redact_all_behaves_like_redact_paths_for_paths(
        path_values in prop::collection::vec(pathish_string(), 1..8),
        excluded in prop::collection::vec(pathish_string(), 0..8),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions {
            excluded,
            ..Default::default()
        };

        let paths_mode = scan_args(&paths, &scan_options, Some(RedactMode::Paths));
        let all_mode = scan_args(&paths, &scan_options, Some(RedactMode::All));

        // Both modes produce identical path and exclusion redaction
        prop_assert_eq!(paths_mode.paths, all_mode.paths);
        prop_assert_eq!(paths_mode.excluded, all_mode.excluded);
        prop_assert_eq!(paths_mode.excluded_redacted, all_mode.excluded_redacted);
    }

    #[test]
    fn scan_args_redacted_paths_contain_no_backslashes(
        path_values in prop::collection::vec(pathish_string(), 0..8),
        redact in redact_mode_strategy(),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions::default();

        let args = scan_args(&paths, &scan_options, redact);
        for p in &args.paths {
            prop_assert!(!p.contains('\\'), "path contained backslash: {}", p);
        }
    }

    #[test]
    fn scan_args_preserves_path_count(
        path_values in prop::collection::vec(pathish_string(), 0..16),
        redact in redact_mode_strategy(),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions::default();

        let args = scan_args(&paths, &scan_options, redact);
        prop_assert_eq!(args.paths.len(), paths.len());
    }

    #[test]
    fn scan_args_preserves_exclusion_count(
        excluded in prop::collection::vec(pathish_string(), 0..16),
        redact in redact_mode_strategy(),
    ) {
        let paths = vec![PathBuf::from(".")];
        let scan_options = ScanOptions {
            excluded: excluded.clone(),
            ..Default::default()
        };

        let args = scan_args(&paths, &scan_options, redact);
        prop_assert_eq!(args.excluded.len(), excluded.len());
    }

    // ── Boolean flag properties ──────────────────────────────────────

    #[test]
    fn scan_args_no_ignore_implies_all_sub_flags(
        path_values in prop::collection::vec(pathish_string(), 1..4),
        no_ignore_parent in any::<bool>(),
        no_ignore_dot in any::<bool>(),
        no_ignore_vcs in any::<bool>(),
        redact in redact_mode_strategy(),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions {
            no_ignore: true,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            ..Default::default()
        };

        let args = scan_args(&paths, &scan_options, redact);
        // When no_ignore is true, all sub-flags must be true
        prop_assert!(args.no_ignore_parent);
        prop_assert!(args.no_ignore_dot);
        prop_assert!(args.no_ignore_vcs);
    }

    #[test]
    fn scan_args_boolean_flags_independent_of_redaction(
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        treat_doc_strings_as_comments in any::<bool>(),
        redact in redact_mode_strategy(),
    ) {
        let paths = vec![PathBuf::from(".")];
        let scan_options = ScanOptions {
            hidden,
            no_ignore,
            treat_doc_strings_as_comments,
            ..Default::default()
        };

        let args = scan_args(&paths, &scan_options, redact);
        prop_assert_eq!(args.hidden, hidden);
        prop_assert_eq!(args.no_ignore, no_ignore);
        prop_assert_eq!(args.treat_doc_strings_as_comments, treat_doc_strings_as_comments);
    }

    // ── Non-redaction preserves exact exclusion strings ───────────────

    #[test]
    fn scan_args_none_redact_preserves_exclusions_verbatim(
        excluded in prop::collection::vec(pathish_string(), 0..8),
    ) {
        let paths = vec![PathBuf::from(".")];
        let scan_options = ScanOptions {
            excluded: excluded.clone(),
            ..Default::default()
        };

        let args_none = scan_args(&paths, &scan_options, Some(RedactMode::None));
        let args_opt_none = scan_args(&paths, &scan_options, None);

        prop_assert_eq!(&args_none.excluded, &excluded);
        prop_assert_eq!(&args_opt_none.excluded, &excluded);
    }

    // NEW property tests

    #[test]
    fn normalize_idempotent(path in pathish_string()) {
        let once = normalize_scan_input(Path::new(&path));
        let twice = normalize_scan_input(Path::new(&once));
        prop_assert_eq!(once, twice);
    }

    #[test]
    fn normalize_no_backslash(path in pathish_string()) {
        let normed = normalize_scan_input(Path::new(&path));
        prop_assert!(!normed.contains('\'));
    }

    #[test]
    fn scan_args_deterministic(path in pathish_string(), mode in redact_mode_strategy()) {
        let paths = vec![PathBuf::from(&path)];
        let opts = ScanOptions::default();
        let a = scan_args(&paths, &opts, mode);
        let b = scan_args(&paths, &opts, mode);
        prop_assert_eq!(a.paths, b.paths);
    }

    #[test]
    fn scan_args_paths_vec(paths_raw in prop::collection::vec(pathish_string(), 1..5)) {
        let paths: Vec<PathBuf> = paths_raw.iter().map(PathBuf::from).collect();
        let opts = ScanOptions::default();
        let args = scan_args(&paths, &opts, None);
        prop_assert_eq!(args.paths.len(), paths.len());
    }

    #[test]
    fn normalize_leading_dot_slash(segment in "[a-zA-Z][a-zA-Z0-9_]{0,20}") {
        let with_dot = format!("./{}", segment);
        let a = normalize_scan_input(Path::new(&with_dot));
        let b = normalize_scan_input(Path::new(&segment));
        prop_assert_eq!(a, b, "dot-slash should be stripped");
    }

}
