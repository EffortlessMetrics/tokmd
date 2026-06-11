//! # tokmd-model
//!
//! **Tier 1 (Logic)**
//!
//! This crate contains the core business logic for aggregating and transforming code statistics.
//! It handles the conversion from raw Tokei scan results into `tokmd` receipts.
//!
//! ## What belongs here
//! * Aggregation logic (rolling up stats to modules/languages)
//! * Deterministic sorting and filtering
//! * Path normalization rules
//! * Receipt generation logic
//!
//! ## What does NOT belong here
//! * CLI argument parsing
//! * Output formatting (printing to stdout/file)
//! * Tokei interaction (use tokmd-scan)

use std::borrow::Cow;
use std::path::Path;

mod aggregate;
mod children;
pub mod module_key;
mod rows;
mod sorting;

pub use aggregate::{
    create_export_data, create_export_data_from_rows, create_lang_report,
    create_lang_report_from_rows, create_module_report, create_module_report_from_rows,
};
pub use rows::{
    InMemoryRowInput, collect_file_rows, collect_in_memory_file_rows, unique_parent_file_count,
    unique_parent_file_count_from_rows,
};

/// Compute the average of `lines` over `files`, rounding to nearest integer.
///
/// Returns 0 if `files` is zero.
///
/// # Examples
///
/// ```
/// use tokmd_model::avg;
///
/// assert_eq!(avg(300, 3), 100);
/// assert_eq!(avg(0, 5), 0);
/// assert_eq!(avg(100, 0), 0);
/// // Rounds to nearest: 7 / 2 = 3.5 → 4
/// assert_eq!(avg(7, 2), 4);
/// ```
#[inline]
pub fn avg(lines: usize, files: usize) -> usize {
    if files == 0 {
        return 0;
    }

    let quotient = lines / files;
    let remainder = lines % files;

    // Round half up without forming `lines + files / 2`, which can overflow
    // for very large repositories. The comparison is equivalent to
    // `remainder * 2 >= files` but avoids overflowing when `files` is large.
    if remainder >= files - remainder {
        quotient + 1
    } else {
        quotient
    }
}

/// Normalize a path for portable output.
///
/// - Uses `/` separators
/// - Strips leading `./`
/// - Optionally strips a user-provided prefix (after normalization)
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use tokmd_model::normalize_path;
///
/// // Normalizes backslashes to forward slashes
/// let p = Path::new("src\\main.rs");
/// assert_eq!(normalize_path(p, None), "src/main.rs");
///
/// // Strips a prefix
/// let p = Path::new("project/src/lib.rs");
/// let prefix = Path::new("project");
/// assert_eq!(normalize_path(&p, Some(&prefix)), "src/lib.rs");
/// ```
#[inline]
pub fn normalize_path(path: &Path, strip_prefix: Option<&Path>) -> String {
    let s_cow = path.to_string_lossy();
    let s: Cow<str> = if s_cow.contains('\\') {
        Cow::Owned(s_cow.replace('\\', "/"))
    } else {
        s_cow
    };

    let mut slice: &str = &s;

    // Strip leading ./ first, so strip_prefix can match against "src/" instead of "./src/"
    if let Some(stripped) = slice.strip_prefix("./") {
        slice = stripped;
    }

    if let Some(prefix) = strip_prefix {
        let p_cow = prefix.to_string_lossy();
        // Strip leading ./ from prefix so it can match normalized paths
        let p_source = p_cow.as_ref();
        let mut p_slice = p_source.strip_prefix("./").unwrap_or(p_source);
        let normalized_prefix;
        if p_slice.contains('\\') {
            normalized_prefix = p_slice.replace('\\', "/");
            p_slice = &normalized_prefix;
        }

        if let Some(stripped) = strip_path_prefix(slice, p_slice) {
            slice = stripped;
        }
    }

    slice = slice.trim_start_matches('/');

    // After trimming slashes, we might be left with a leading ./ (e.g. from "/./")
    if let Some(stripped) = slice.strip_prefix("./") {
        slice = stripped;
    }
    slice = slice.trim_start_matches('/');

    if slice.len() == s.len() {
        s.into_owned()
    } else {
        slice.to_string()
    }
}

fn strip_path_prefix<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.ends_with('/') {
        path.strip_prefix(prefix)
    } else {
        path.strip_prefix(prefix)
            .and_then(|stripped| stripped.strip_prefix('/'))
    }
}

/// Compute a "module key" from an input path.
///
/// Rules:
/// - Root-level files become "(root)".
/// - If the first directory segment is in `module_roots`, join `module_depth` *directory* segments.
/// - Otherwise, module key is the top-level directory.
///
/// # Examples
///
/// ```
/// use tokmd_model::module_key;
///
/// let roots = vec!["crates".to_string()];
/// assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 2), "crates/foo");
/// assert_eq!(module_key("src/lib.rs", &roots, 2), "src");
/// assert_eq!(module_key("Cargo.toml", &roots, 2), "(root)");
/// ```
pub fn module_key(path: &str, module_roots: &[String], module_depth: usize) -> String {
    module_key::module_key(path, module_roots, module_depth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn module_key_root_level_file() {
        assert_eq!(module_key("Cargo.toml", &["crates".into()], 2), "(root)");
        assert_eq!(module_key("./Cargo.toml", &["crates".into()], 2), "(root)");
    }

    #[test]
    fn module_key_crates_depth_2() {
        let roots = vec!["crates".into(), "packages".into()];
        assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 2), "crates/foo");
        assert_eq!(
            module_key("packages/bar/src/main.rs", &roots, 2),
            "packages/bar"
        );
    }

    #[test]
    fn module_key_crates_depth_1() {
        let roots = vec!["crates".into(), "packages".into()];
        assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 1), "crates");
    }

    #[test]
    fn module_key_non_root() {
        let roots = vec!["crates".into()];
        assert_eq!(module_key("src/lib.rs", &roots, 2), "src");
        assert_eq!(module_key("tools/gen.rs", &roots, 2), "tools");
    }

    #[test]
    fn module_key_depth_overflow_does_not_include_filename() {
        let roots = vec!["crates".into()];
        // File directly under a root: depth=2 should NOT include the filename
        assert_eq!(module_key("crates/foo.rs", &roots, 2), "crates");
        // Depth exceeds available directories: should stop at deepest directory
        assert_eq!(
            module_key("crates/foo/src/lib.rs", &roots, 10),
            "crates/foo/src"
        );
    }

    #[test]
    fn normalize_path_strips_prefix() {
        let p = PathBuf::from("C:/Code/Repo/src/main.rs");
        let prefix = PathBuf::from("C:/Code/Repo");
        let got = normalize_path(&p, Some(&prefix));
        assert_eq!(got, "src/main.rs");
    }

    #[test]
    fn normalize_path_normalization_slashes() {
        let p = PathBuf::from(r"C:\Code\Repo\src\main.rs");
        let got = normalize_path(&p, None);
        assert_eq!(got, "C:/Code/Repo/src/main.rs");
    }

    mod normalize_properties {
        use super::*;
        use proptest::prelude::*;

        fn arb_path_component() -> impl Strategy<Value = String> {
            "[a-zA-Z0-9_.-]+"
        }

        fn arb_path(max_depth: usize) -> impl Strategy<Value = String> {
            prop::collection::vec(arb_path_component(), 1..=max_depth)
                .prop_map(|comps| comps.join("/"))
        }

        proptest! {
            #[test]
            fn normalize_path_is_idempotent(path in arb_path(5)) {
                let p = PathBuf::from(&path);
                let norm1 = normalize_path(&p, None);
                let p2 = PathBuf::from(&norm1);
                let norm2 = normalize_path(&p2, None);
                prop_assert_eq!(norm1, norm2);
            }

            #[test]
            fn normalize_path_handles_windows_separators(path in arb_path(5)) {
                let win_path = path.replace('/', "\\");
                let p_win = PathBuf::from(&win_path);
                let p_unix = PathBuf::from(&path);

                let norm_win = normalize_path(&p_win, None);
                let norm_unix = normalize_path(&p_unix, None);

                prop_assert_eq!(norm_win, norm_unix);
            }

            #[test]
            fn normalize_path_no_leading_slash(path in arb_path(5)) {
                let p = PathBuf::from(&path);
                let norm = normalize_path(&p, None);
                prop_assert!(!norm.starts_with('/'));
            }

            #[test]
            fn normalize_path_no_leading_dot_slash(path in arb_path(5)) {
                let p = PathBuf::from(&path);
                let norm = normalize_path(&p, None);
                prop_assert!(!norm.starts_with("./"));
            }

            #[test]
            fn module_key_deterministic(
                path in arb_path(5),
                roots in prop::collection::vec(arb_path_component(), 1..3),
                depth in 1usize..5
            ) {
                let k1 = module_key(&path, &roots, depth);
                let k2 = module_key(&path, &roots, depth);
                prop_assert_eq!(k1, k2);
            }
        }
    }

    #[test]
    fn avg_handles_boundaries_and_rounding() {
        assert_eq!(avg(100, 0), 0);
        assert_eq!(avg(10, 2), 5);
        assert_eq!(avg(9, 3), 3);
        assert_eq!(avg(10, 3), 3);
        assert_eq!(avg(11, 3), 4);
    }

    #[test]
    fn avg_handles_maximum_line_counts_without_overflow() {
        assert_eq!(avg(usize::MAX, 1), usize::MAX);
        assert_eq!(avg(usize::MAX, 2), usize::MAX / 2 + 1);
        assert_eq!(avg(usize::MAX - 1, 2), usize::MAX / 2);
    }
}
