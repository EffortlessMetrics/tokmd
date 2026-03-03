//! Additional property-based tests for tokmd-scan-args (wave 40).
//!
//! Covers: normalize idempotency, path count preservation,
//! redaction flag consistency, and config mode propagation.

use std::path::{Path, PathBuf};

use proptest::prelude::*;
use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

// =========================================================================
// Normalize is idempotent: normalize(normalize(x)) == normalize(x)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn normalize_is_idempotent(
        parts in prop::collection::vec("[a-z]{2,8}", 1..5),
    ) {
        let path = parts.join("/");
        let once = normalize_scan_input(Path::new(&path));
        let twice = normalize_scan_input(Path::new(&once));
        prop_assert_eq!(once, twice, "normalize must be idempotent");
    }
}

// =========================================================================
// Normalize output is never empty
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn normalize_output_never_empty(
        path in ".{0,40}",
    ) {
        let normalized = normalize_scan_input(Path::new(&path));
        prop_assert!(!normalized.is_empty(),
            "Normalized output must never be empty for input '{}'", path);
    }
}

// =========================================================================
// Path count preservation: output paths count matches input
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn scan_args_path_count_matches_input(
        path_values in prop::collection::vec("[a-z]{2,10}", 1..8),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions::default();
        let args = scan_args(&paths, &scan_options, None);
        prop_assert_eq!(args.paths.len(), paths.len(),
            "Output path count {} != input count {}", args.paths.len(), paths.len());
    }

    #[test]
    fn scan_args_path_count_preserved_under_redaction(
        path_values in prop::collection::vec("[a-z]{2,10}\\.[a-z]{1,3}", 1..8),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions::default();
        let args = scan_args(&paths, &scan_options, Some(RedactMode::Paths));
        prop_assert_eq!(args.paths.len(), paths.len(),
            "Redacted path count {} != input count {}", args.paths.len(), paths.len());
    }
}

// =========================================================================
// Excluded count preservation
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn excluded_count_preserved(
        excluded in prop::collection::vec("[a-z]{3,10}", 0..5),
    ) {
        let paths = vec![PathBuf::from(".")];
        let scan_options = ScanOptions {
            excluded: excluded.clone(),
            ..Default::default()
        };
        let args_none = scan_args(&paths, &scan_options, None);
        let args_redact = scan_args(&paths, &scan_options, Some(RedactMode::Paths));
        prop_assert_eq!(args_none.excluded.len(), excluded.len(),
            "Non-redacted excluded count mismatch");
        prop_assert_eq!(args_redact.excluded.len(), excluded.len(),
            "Redacted excluded count mismatch");
    }
}

// =========================================================================
// excluded_redacted flag matches redaction mode
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn excluded_redacted_flag_correct(
        excluded in prop::collection::vec("[a-z]{3,10}", 1..5),
        redact_idx in 0usize..4,
    ) {
        let redact = [None, Some(RedactMode::None), Some(RedactMode::Paths), Some(RedactMode::All)][redact_idx];
        let paths = vec![PathBuf::from(".")];
        let scan_options = ScanOptions {
            excluded: excluded.clone(),
            ..Default::default()
        };
        let args = scan_args(&paths, &scan_options, redact);

        let should_be_redacted = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
        prop_assert_eq!(args.excluded_redacted, should_be_redacted,
            "excluded_redacted flag should be {} for redact mode {:?}", should_be_redacted, redact);
    }
}

// =========================================================================
// Boolean flags propagation
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn hidden_flag_propagated(
        hidden in any::<bool>(),
    ) {
        let paths = vec![PathBuf::from(".")];
        let scan_options = ScanOptions {
            hidden,
            ..Default::default()
        };
        let args = scan_args(&paths, &scan_options, None);
        prop_assert_eq!(args.hidden, hidden, "hidden flag must propagate");
    }

    #[test]
    fn treat_doc_strings_flag_propagated(
        treat_doc in any::<bool>(),
    ) {
        let paths = vec![PathBuf::from(".")];
        let scan_options = ScanOptions {
            treat_doc_strings_as_comments: treat_doc,
            ..Default::default()
        };
        let args = scan_args(&paths, &scan_options, None);
        prop_assert_eq!(args.treat_doc_strings_as_comments, treat_doc,
            "treat_doc_strings_as_comments must propagate");
    }
}

// =========================================================================
// Normalize never contains backslash
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn normalize_forward_slashes_only(
        parts in prop::collection::vec("[a-z]{2,6}", 1..6),
    ) {
        // Mix of forward and backslash separators
        let mixed = parts.join("\\");
        let normalized = normalize_scan_input(Path::new(&mixed));
        prop_assert!(!normalized.contains('\\'),
            "Normalized '{}' must not contain backslash", normalized);
    }
}
