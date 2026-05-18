//! Deterministic context/handoff policy helpers.

#![forbid(unsafe_code)]

mod cap;
mod classify;
mod patterns;
mod policy;

pub use cap::compute_file_cap;
pub use classify::classify_file;
pub use patterns::{is_spine_file, smart_exclude_reason};
pub use policy::assign_policy;

/// Default maximum fraction of budget a single file may consume.
pub const DEFAULT_MAX_FILE_PCT: f64 = 0.15;
/// Default hard cap for a single file when no explicit cap is provided.
pub const DEFAULT_MAX_FILE_TOKENS: usize = 16_000;
/// Default tokens-per-line threshold for dense blob detection.
pub const DEFAULT_DENSE_THRESHOLD: f64 = 50.0;

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tokmd_types::{FileClassification, InclusionPolicy};

    #[test]
    fn smart_exclude_reason_detects_lockfiles_and_sourcemaps() {
        assert_eq!(smart_exclude_reason("Cargo.lock"), Some("lockfile"));
        assert_eq!(smart_exclude_reason("dist/app.js.map"), Some("sourcemap"));
        assert_eq!(smart_exclude_reason("src/main.rs"), None);
    }

    #[test]
    fn is_spine_file_matches_basename_and_document_paths() {
        assert!(is_spine_file("README.md"));
        assert!(is_spine_file("nested/docs/architecture.md"));
        assert!(!is_spine_file("src/main.rs"));
    }

    #[test]
    fn classify_file_detects_generated_and_dense_blob() {
        let classes = classify_file("src/node-types.json", 50_000, 5, 50.0);
        assert!(classes.contains(&FileClassification::Generated));
        assert!(classes.contains(&FileClassification::DataBlob));
    }

    #[test]
    fn assign_policy_skips_oversized_generated_files() {
        let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Generated]);
        assert_eq!(policy, InclusionPolicy::Skip);
        assert!(reason.unwrap_or_default().contains("generated"));
    }

    proptest! {
        #[test]
        fn context_policy_invariants_hold_for_arbitrary_inputs(
            path in "\\PC+",
            tokens in 0usize..1_000_000,
            lines in 0usize..1_000_000,
            budget in 0usize..1_000_000,
        ) {
            let _ = is_spine_file(path.as_ref());

            if let Some(reason) = smart_exclude_reason(path.as_ref()) {
                prop_assert!(matches!(reason, "lockfile" | "minified" | "sourcemap"));
            }

            let classes = classify_file(path.as_ref(), tokens, lines, DEFAULT_DENSE_THRESHOLD);
            let mut sorted = classes.clone();
            sorted.sort();
            sorted.dedup();
            prop_assert_eq!(&classes, &sorted);

            let cap_default = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
            let cap_hard = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, Some(4_000));
            prop_assert!(cap_hard <= 4_000 || cap_hard == usize::MAX);

            let (policy, reason) = assign_policy(tokens, cap_default, &classes);
            match policy {
                InclusionPolicy::Full => {
                    prop_assert!(tokens <= cap_default);
                    prop_assert!(reason.is_none());
                }
                InclusionPolicy::HeadTail | InclusionPolicy::Skip => {
                    if cap_default != usize::MAX {
                        prop_assert!(tokens > cap_default);
                        prop_assert!(reason.is_some());
                    }
                }
                InclusionPolicy::Summary => {}
            }
        }
    }
}
