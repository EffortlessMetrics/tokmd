//! W59 — property tests for scan-args determinism and redaction invariants.

use std::path::{Path, PathBuf};

use proptest::prelude::*;
use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

fn pathish() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::sample::select(
            "abcdefghijklmnopqrstuvwxyz0123456789_/.\\"
                .chars()
                .collect::<Vec<_>>(),
        ),
        0..40,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

fn redact_mode() -> impl Strategy<Value = Option<RedactMode>> {
    prop_oneof![
        Just(None),
        Just(Some(RedactMode::None)),
        Just(Some(RedactMode::Paths)),
        Just(Some(RedactMode::All)),
    ]
}

proptest! {
    // ── Normalization ────────────────────────────────────────────────

    #[test]
    fn normalize_never_contains_backslash(input in pathish()) {
        let result = normalize_scan_input(Path::new(&input));
        prop_assert!(!result.contains('\\'));
    }

    #[test]
    fn normalize_never_empty(input in pathish()) {
        let result = normalize_scan_input(Path::new(&input));
        prop_assert!(!result.is_empty());
    }

    #[test]
    fn normalize_never_starts_with_dot_slash(input in pathish()) {
        let result = normalize_scan_input(Path::new(&input));
        prop_assert!(!result.starts_with("./"));
    }

    #[test]
    fn normalize_idempotent(input in pathish()) {
        let once = normalize_scan_input(Path::new(&input));
        let twice = normalize_scan_input(Path::new(&once));
        prop_assert_eq!(once, twice);
    }

    // ── Determinism ──────────────────────────────────────────────────

    #[test]
    fn scan_args_deterministic(
        path in pathish(),
        mode in redact_mode(),
    ) {
        let paths = vec![PathBuf::from(&path)];
        let opts = ScanOptions::default();
        let a = scan_args(&paths, &opts, mode);
        let b = scan_args(&paths, &opts, mode);
        prop_assert_eq!(a.paths, b.paths);
        prop_assert_eq!(a.excluded, b.excluded);
        prop_assert_eq!(a.excluded_redacted, b.excluded_redacted);
    }

    // ── Path count preservation ──────────────────────────────────────

    #[test]
    fn scan_args_preserves_path_count(
        raw in prop::collection::vec(pathish(), 0..10),
        mode in redact_mode(),
    ) {
        let paths: Vec<PathBuf> = raw.iter().map(PathBuf::from).collect();
        let opts = ScanOptions::default();
        let args = scan_args(&paths, &opts, mode);
        prop_assert_eq!(args.paths.len(), paths.len());
    }

    // ── Exclusion count preservation ─────────────────────────────────

    #[test]
    fn scan_args_preserves_exclusion_count(
        excluded in prop::collection::vec(pathish(), 0..10),
        mode in redact_mode(),
    ) {
        let paths = vec![PathBuf::from(".")];
        let opts = ScanOptions {
            excluded: excluded.clone(),
            ..Default::default()
        };
        let args = scan_args(&paths, &opts, mode);
        prop_assert_eq!(args.excluded.len(), excluded.len());
    }

    // ── No backslashes in output ─────────────────────────────────────

    #[test]
    fn scan_args_no_backslash_in_paths(
        raw in prop::collection::vec(pathish(), 1..5),
        mode in redact_mode(),
    ) {
        let paths: Vec<PathBuf> = raw.iter().map(PathBuf::from).collect();
        let opts = ScanOptions::default();
        let args = scan_args(&paths, &opts, mode);
        for p in &args.paths {
            prop_assert!(!p.contains('\\'), "backslash: {p}");
        }
    }

    // ── excluded_redacted consistency ────────────────────────────────

    #[test]
    fn excluded_redacted_correct(
        excluded in prop::collection::vec(pathish(), 0..5),
        mode in redact_mode(),
    ) {
        let paths = vec![PathBuf::from(".")];
        let opts = ScanOptions {
            excluded: excluded.clone(),
            ..Default::default()
        };
        let args = scan_args(&paths, &opts, mode);
        let should_redact = matches!(mode, Some(RedactMode::Paths | RedactMode::All));
        let expected = should_redact && !excluded.is_empty();
        prop_assert_eq!(args.excluded_redacted, expected);
    }

    // ── no_ignore implies all sub-flags ──────────────────────────────

    #[test]
    fn no_ignore_implies_sub_flags(
        no_ignore_parent in any::<bool>(),
        no_ignore_dot in any::<bool>(),
        no_ignore_vcs in any::<bool>(),
    ) {
        let paths = vec![PathBuf::from(".")];
        let opts = ScanOptions {
            no_ignore: true,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            ..Default::default()
        };
        let args = scan_args(&paths, &opts, None);
        prop_assert!(args.no_ignore_parent);
        prop_assert!(args.no_ignore_dot);
        prop_assert!(args.no_ignore_vcs);
    }

    // ── Boolean flags independent of redaction ───────────────────────

    #[test]
    fn boolean_flags_survive_redaction(
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        docstrings in any::<bool>(),
        mode in redact_mode(),
    ) {
        let paths = vec![PathBuf::from(".")];
        let opts = ScanOptions {
            hidden,
            no_ignore,
            treat_doc_strings_as_comments: docstrings,
            ..Default::default()
        };
        let args = scan_args(&paths, &opts, mode);
        prop_assert_eq!(args.hidden, hidden);
        prop_assert_eq!(args.no_ignore, no_ignore);
        prop_assert_eq!(args.treat_doc_strings_as_comments, docstrings);
    }

    // ── Non-redact modes preserve exclusions verbatim ────────────────

    #[test]
    fn non_redact_preserves_exclusions(
        excluded in prop::collection::vec("[a-z]{1,10}", 0..5),
    ) {
        let paths = vec![PathBuf::from(".")];
        let opts = ScanOptions {
            excluded: excluded.clone(),
            ..Default::default()
        };
        let args_none = scan_args(&paths, &opts, None);
        let args_mode_none = scan_args(&paths, &opts, Some(RedactMode::None));
        prop_assert_eq!(&args_none.excluded, &excluded);
        prop_assert_eq!(&args_mode_none.excluded, &excluded);
    }

    // ── Redact Paths == Redact All for path/exclusion output ─────────

    #[test]
    fn redact_paths_equals_all_for_paths(
        raw in prop::collection::vec("[a-z]{2,8}", 1..4),
        excluded in prop::collection::vec("[a-z]{2,8}", 0..3),
    ) {
        let paths: Vec<PathBuf> = raw.iter().map(PathBuf::from).collect();
        let opts = ScanOptions {
            excluded,
            ..Default::default()
        };
        let a = scan_args(&paths, &opts, Some(RedactMode::Paths));
        let b = scan_args(&paths, &opts, Some(RedactMode::All));
        prop_assert_eq!(a.paths, b.paths);
        prop_assert_eq!(a.excluded, b.excluded);
    }

    // ── Redacted paths don't leak long original segments ─────────────

    #[test]
    fn redacted_paths_hide_originals(
        segments in prop::collection::vec("[a-z]{5,10}", 1..4),
    ) {
        let path_str = segments.join("/") + ".rs";
        let paths = vec![PathBuf::from(&path_str)];
        let opts = ScanOptions::default();
        let args = scan_args(&paths, &opts, Some(RedactMode::Paths));
        let redacted = &args.paths[0];
        for seg in &segments {
            prop_assert!(
                !redacted.contains(seg.as_str()),
                "segment {seg:?} leaked in {redacted:?}"
            );
        }
    }
}
