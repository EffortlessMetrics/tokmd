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
}
