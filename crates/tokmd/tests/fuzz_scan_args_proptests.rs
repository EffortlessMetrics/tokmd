//! Deterministic property tests extracted from `fuzz_scan_args`.
//!
//! Validates invariants for:
//! - Path normalization
//! - Redaction mode toggles
//! - Deterministic output
//! - Ignore-flag fan-out behavior

use proptest::prelude::*;
use std::path::PathBuf;
use tokmd_format::scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

proptest! {
    #[test]
    fn scan_args_invariants(
        paths in prop::collection::vec("[a-zA-Z0-9_\\-\\./\\\\]+", 1..10),
        excluded in prop::collection::vec("[a-zA-Z0-9_\\-\\.*]+", 0..5),
        redact_mode in prop::sample::select(vec![None, Some(RedactMode::None), Some(RedactMode::Paths), Some(RedactMode::All)]),
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        no_ignore_parent in any::<bool>(),
        no_ignore_dot in any::<bool>(),
        no_ignore_vcs in any::<bool>(),
        treat_doc_strings_as_comments in any::<bool>()
    ) {
        let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

        let scan_options = ScanOptions {
            excluded: excluded.clone(),
            config: tokmd_settings::ConfigMode::Auto,
            hidden,
            no_ignore,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            treat_doc_strings_as_comments,
        };

        let args = scan_args(&path_bufs, &scan_options, redact_mode);
        let args2 = scan_args(&path_bufs, &scan_options, redact_mode);

        // Determinism: same input must produce same output.
        prop_assert_eq!(&args.paths, &args2.paths);
        prop_assert_eq!(&args.excluded, &args2.excluded);
        prop_assert_eq!(args.excluded_redacted, args2.excluded_redacted);
        prop_assert_eq!(args.no_ignore_parent, args2.no_ignore_parent);
        prop_assert_eq!(args.no_ignore_dot, args2.no_ignore_dot);
        prop_assert_eq!(args.no_ignore_vcs, args2.no_ignore_vcs);

        // Paths are always one-to-one with input paths.
        prop_assert_eq!(args.paths.len(), paths.len());

        // Normalization invariant: output paths never contain backslashes.
        for p in &args.paths {
            prop_assert!(!p.contains('\\'));
        }

        let should_redact = matches!(redact_mode, Some(RedactMode::Paths | RedactMode::All));
        let expected_excluded_redacted = should_redact && !excluded.is_empty();
        prop_assert_eq!(args.excluded_redacted, expected_excluded_redacted);

        if should_redact {
            // Redacted paths are hash-based and should not include separators.
            for p in &args.paths {
                prop_assert!(!p.contains('/'));
                prop_assert!(!p.contains('\\'));
            }

            // Excluded patterns are short hashes (16 lowercase hex chars).
            prop_assert_eq!(args.excluded.len(), excluded.len());
            for value in &args.excluded {
                prop_assert_eq!(value.len(), 16);
                for c in value.chars() {
                    prop_assert!(c.is_ascii_hexdigit());
                    prop_assert!(!c.is_ascii_uppercase());
                }
            }
        } else {
            let expected_paths: Vec<String> = path_bufs.iter().map(|p| normalize_scan_input(p)).collect();
            prop_assert_eq!(&args.paths, &expected_paths);
            prop_assert_eq!(&args.excluded, &excluded);
        }

        // no_ignore should force sub-flags true.
        if no_ignore {
            prop_assert!(args.no_ignore_parent);
            prop_assert!(args.no_ignore_dot);
            prop_assert!(args.no_ignore_vcs);
        }
    }
}
