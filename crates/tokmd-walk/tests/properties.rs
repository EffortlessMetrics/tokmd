//! Property-based tests for tokmd-walk.
//!
//! These tests verify the correctness, determinism, and consistency
//! of the `license_candidates` function.

use proptest::prelude::*;
use std::path::PathBuf;
use tokmd_walk::license_candidates;

// ============================================================================
// Strategies
// ============================================================================

/// Strategy for generating license-like filenames.
fn arb_license_filename() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("LICENSE".to_string()),
        Just("LICENSE.md".to_string()),
        Just("LICENSE.txt".to_string()),
        Just("LICENSE-MIT".to_string()),
        Just("LICENSE-APACHE".to_string()),
        Just("license".to_string()),
        Just("license.md".to_string()),
        Just("COPYING".to_string()),
        Just("COPYING.txt".to_string()),
        Just("copying".to_string()),
        Just("NOTICE".to_string()),
        Just("NOTICE.md".to_string()),
        Just("notice".to_string()),
        Just("notice.txt".to_string()),
    ]
}

/// Strategy for generating metadata filenames.
fn arb_metadata_filename() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("Cargo.toml".to_string()),
        Just("cargo.toml".to_string()),
        Just("CARGO.TOML".to_string()),
        Just("package.json".to_string()),
        Just("Package.json".to_string()),
        Just("PACKAGE.JSON".to_string()),
        Just("pyproject.toml".to_string()),
        Just("Pyproject.toml".to_string()),
        Just("PYPROJECT.TOML".to_string()),
    ]
}

/// Strategy for generating non-matching filenames.
fn arb_non_matching_filename() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("README.md".to_string()),
        Just("main.rs".to_string()),
        Just("lib.rs".to_string()),
        Just("index.js".to_string()),
        Just("setup.py".to_string()),
        Just("Makefile".to_string()),
        Just("Dockerfile".to_string()),
        Just(".gitignore".to_string()),
        Just("config.yaml".to_string()),
        Just("data.json".to_string()),
        // Tricky names that should NOT match
        Just("UNLICENSE".to_string()), // Doesn't start with "license"
        Just("my-license-helper.rs".to_string()),
        Just("noticing.txt".to_string()), // "noticing" != starts_with("notice")
        Just("copyingfile.txt".to_string()), // Should match - starts with "copying"
    ]
}

/// Strategy for generating arbitrary directory paths.
fn arb_directory_path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9_.-]+", 0..=4).prop_map(|parts| {
        if parts.is_empty() {
            String::new()
        } else {
            parts.join("/")
        }
    })
}

/// Strategy for generating paths with various separators.
fn arb_path_with_separators() -> impl Strategy<Value = (String, String)> {
    (arb_directory_path(), arb_license_filename()).prop_map(|(dir, file)| {
        let unix_path = if dir.is_empty() {
            file.clone()
        } else {
            format!("{}/{}", dir, file)
        };
        let windows_path = unix_path.replace('/', "\\");
        (unix_path, windows_path)
    })
}

/// Strategy for paths with special characters in parent directories.
fn arb_special_char_path() -> impl Strategy<Value = String> {
    (
        prop::collection::vec("[a-zA-Z0-9_ .-]+", 1..=3),
        arb_license_filename(),
    )
        .prop_map(|(dirs, file)| {
            let dir = dirs.join("/");
            format!("{}/{}", dir, file)
        })
}

// ============================================================================
// Determinism tests
// ============================================================================

proptest! {
    /// Same input list always produces the same output (determinism).
    #[test]
    fn license_candidates_is_deterministic(
        files in prop::collection::vec(arb_license_filename(), 0..=10)
    ) {
        let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();

        let result1 = license_candidates(&paths);
        let result2 = license_candidates(&paths);

        prop_assert_eq!(
            result1.license_files,
            result2.license_files,
            "License files must be deterministic"
        );
        prop_assert_eq!(
            result1.metadata_files,
            result2.metadata_files,
            "Metadata files must be deterministic"
        );
    }

    /// Determinism with mixed file types.
    #[test]
    fn license_candidates_deterministic_mixed(
        licenses in prop::collection::vec(arb_license_filename(), 0..=5),
        metadata in prop::collection::vec(arb_metadata_filename(), 0..=5),
        other in prop::collection::vec(arb_non_matching_filename(), 0..=5),
    ) {
        let mut files: Vec<PathBuf> = Vec::new();
        files.extend(licenses.iter().map(PathBuf::from));
        files.extend(metadata.iter().map(PathBuf::from));
        files.extend(other.iter().map(PathBuf::from));

        let result1 = license_candidates(&files);
        let result2 = license_candidates(&files);

        prop_assert_eq!(result1.license_files, result2.license_files);
        prop_assert_eq!(result1.metadata_files, result2.metadata_files);
    }
}

// ============================================================================
// Sorting tests
// ============================================================================

proptest! {
    /// Results are always sorted alphabetically.
    #[test]
    fn license_files_are_sorted(
        files in prop::collection::vec(arb_license_filename(), 0..=20)
    ) {
        let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
        let result = license_candidates(&paths);

        let license_strs: Vec<String> = result.license_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let mut sorted = license_strs.clone();
        sorted.sort();

        prop_assert_eq!(
            license_strs,
            sorted,
            "License files must be sorted alphabetically"
        );
    }

    /// Metadata files are always sorted alphabetically.
    #[test]
    fn metadata_files_are_sorted(
        files in prop::collection::vec(arb_metadata_filename(), 0..=20)
    ) {
        let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
        let result = license_candidates(&paths);

        let metadata_strs: Vec<String> = result.metadata_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let mut sorted = metadata_strs.clone();
        sorted.sort();

        prop_assert_eq!(
            metadata_strs,
            sorted,
            "Metadata files must be sorted alphabetically"
        );
    }
}

// ============================================================================
// License file matching tests
// ============================================================================

proptest! {
    /// All license filenames are correctly identified.
    #[test]
    fn license_files_match_patterns(filename in arb_license_filename()) {
        let paths = vec![PathBuf::from(&filename)];
        let result = license_candidates(&paths);

        prop_assert!(
            result.license_files.contains(&PathBuf::from(&filename)),
            "License file '{}' should be identified as a license file",
            filename
        );
        prop_assert!(
            result.metadata_files.is_empty(),
            "License file '{}' should not be in metadata files",
            filename
        );
    }

    /// License files in subdirectories are correctly identified.
    #[test]
    fn license_files_in_subdirs((unix_path, _windows_path) in arb_path_with_separators()) {
        let paths = vec![PathBuf::from(&unix_path)];
        let result = license_candidates(&paths);

        prop_assert!(
            result.license_files.contains(&PathBuf::from(&unix_path)),
            "License file at '{}' should be identified",
            unix_path
        );
    }

    /// License files with special characters in parent directories work.
    #[test]
    fn license_files_with_special_parent_dirs(path in arb_special_char_path()) {
        let paths = vec![PathBuf::from(&path)];
        let result = license_candidates(&paths);

        prop_assert!(
            result.license_files.contains(&PathBuf::from(&path)),
            "License file at '{}' should be identified despite special chars in path",
            path
        );
    }
}

// ============================================================================
// Metadata file matching tests
// ============================================================================

proptest! {
    /// All metadata filenames are correctly identified.
    #[test]
    fn metadata_files_match_exactly(filename in arb_metadata_filename()) {
        let paths = vec![PathBuf::from(&filename)];
        let result = license_candidates(&paths);

        prop_assert!(
            result.metadata_files.contains(&PathBuf::from(&filename)),
            "Metadata file '{}' should be identified as a metadata file",
            filename
        );
        prop_assert!(
            result.license_files.is_empty(),
            "Metadata file '{}' should not be in license files",
            filename
        );
    }

    /// Metadata files in subdirectories are correctly identified.
    #[test]
    fn metadata_files_in_subdirs(
        dir in arb_directory_path(),
        filename in arb_metadata_filename()
    ) {
        let path = if dir.is_empty() {
            filename.clone()
        } else {
            format!("{}/{}", dir, filename)
        };
        let paths = vec![PathBuf::from(&path)];
        let result = license_candidates(&paths);

        prop_assert!(
            result.metadata_files.contains(&PathBuf::from(&path)),
            "Metadata file at '{}' should be identified",
            path
        );
    }
}

// ============================================================================
// No overlaps test
// ============================================================================

proptest! {
    /// No file appears in both license_files and metadata_files.
    #[test]
    fn no_overlap_between_license_and_metadata(
        licenses in prop::collection::vec(arb_license_filename(), 0..=10),
        metadata in prop::collection::vec(arb_metadata_filename(), 0..=10),
        other in prop::collection::vec(arb_non_matching_filename(), 0..=10),
    ) {
        let mut files: Vec<PathBuf> = Vec::new();
        files.extend(licenses.iter().map(PathBuf::from));
        files.extend(metadata.iter().map(PathBuf::from));
        files.extend(other.iter().map(PathBuf::from));

        let result = license_candidates(&files);

        for license_file in &result.license_files {
            prop_assert!(
                !result.metadata_files.contains(license_file),
                "File '{}' appears in both license_files and metadata_files",
                license_file.display()
            );
        }

        for metadata_file in &result.metadata_files {
            prop_assert!(
                !result.license_files.contains(metadata_file),
                "File '{}' appears in both metadata_files and license_files",
                metadata_file.display()
            );
        }
    }
}

// ============================================================================
// Empty input test
// ============================================================================

proptest! {
    /// Empty input always produces empty outputs.
    #[test]
    fn empty_input_produces_empty_output(_dummy in 0..1u8) {
        let files: Vec<PathBuf> = Vec::new();
        let result = license_candidates(&files);

        prop_assert!(
            result.license_files.is_empty(),
            "Empty input should produce empty license_files"
        );
        prop_assert!(
            result.metadata_files.is_empty(),
            "Empty input should produce empty metadata_files"
        );
    }
}

// ============================================================================
// Non-matching files test
// ============================================================================

proptest! {
    /// Non-matching files are not included in either output list.
    /// Note: Some "non-matching" files like "copyingfile.txt" actually DO match
    /// because they start with "copying" (case-insensitive).
    #[test]
    fn non_matching_files_excluded(filename in arb_non_matching_filename()) {
        let lower = filename.to_lowercase();

        // Skip files that actually match the patterns
        let starts_with_license = lower.starts_with("license");
        let starts_with_copying = lower.starts_with("copying");
        let starts_with_notice = lower.starts_with("notice");
        let is_metadata = lower == "cargo.toml" || lower == "package.json" || lower == "pyproject.toml";

        prop_assume!(!starts_with_license && !starts_with_copying && !starts_with_notice && !is_metadata);

        let paths = vec![PathBuf::from(&filename)];
        let result = license_candidates(&paths);

        prop_assert!(
            result.license_files.is_empty(),
            "Non-matching file '{}' should not be in license_files",
            filename
        );
        prop_assert!(
            result.metadata_files.is_empty(),
            "Non-matching file '{}' should not be in metadata_files",
            filename
        );
    }
}

// ============================================================================
// Path separator handling tests
// ============================================================================

proptest! {
    /// Files with forward slash paths are handled correctly.
    #[test]
    fn handles_forward_slash_paths(
        parts in prop::collection::vec("[a-zA-Z0-9_]+", 1..=4),
        filename in arb_license_filename()
    ) {
        let path = format!("{}/{}", parts.join("/"), filename);
        let paths = vec![PathBuf::from(&path)];
        let result = license_candidates(&paths);

        prop_assert!(
            result.license_files.contains(&PathBuf::from(&path)),
            "License file at '{}' should be identified",
            path
        );
    }

    /// Files with backslash paths are handled correctly (Windows-style).
    #[test]
    fn handles_backslash_paths(
        parts in prop::collection::vec("[a-zA-Z0-9_]+", 1..=4),
        filename in arb_license_filename()
    ) {
        let path = format!("{}\\{}", parts.join("\\"), filename);
        let paths = vec![PathBuf::from(&path)];
        #[allow(unused_variables)]
        let result = license_candidates(&paths);

        // On Windows, backslash is the separator, so file_name() works correctly
        // On Unix, backslash is part of the filename, so this may or may not match
        // The function should work correctly for the platform it's running on
        #[cfg(windows)]
        {
            prop_assert!(
                result.license_files.contains(&PathBuf::from(&path)),
                "License file at '{}' should be identified on Windows",
                path
            );
        }
    }
}

// ============================================================================
// Case insensitivity tests
// ============================================================================

proptest! {
    /// License detection is case-insensitive.
    #[test]
    fn license_detection_case_insensitive(
        prefix in prop_oneof![
            Just("LICENSE"),
            Just("license"),
            Just("License"),
            Just("LiCeNsE"),
            Just("COPYING"),
            Just("copying"),
            Just("Copying"),
            Just("NOTICE"),
            Just("notice"),
            Just("Notice"),
        ],
        suffix in prop_oneof![
            Just("".to_string()),
            Just(".md".to_string()),
            Just(".txt".to_string()),
            Just("-MIT".to_string()),
        ]
    ) {
        let filename = format!("{}{}", prefix, suffix);
        let paths = vec![PathBuf::from(&filename)];
        let result = license_candidates(&paths);

        prop_assert!(
            result.license_files.contains(&PathBuf::from(&filename)),
            "License file '{}' should be identified (case-insensitive)",
            filename
        );
    }

    /// Metadata detection is case-insensitive.
    #[test]
    fn metadata_detection_case_insensitive(
        filename in prop_oneof![
            Just("cargo.toml"),
            Just("Cargo.toml"),
            Just("CARGO.TOML"),
            Just("Cargo.Toml"),
            Just("package.json"),
            Just("Package.json"),
            Just("PACKAGE.JSON"),
            Just("Package.Json"),
            Just("pyproject.toml"),
            Just("Pyproject.toml"),
            Just("PYPROJECT.TOML"),
            Just("PyProject.Toml"),
        ]
    ) {
        let paths = vec![PathBuf::from(filename)];
        let result = license_candidates(&paths);

        prop_assert!(
            result.metadata_files.contains(&PathBuf::from(filename)),
            "Metadata file '{}' should be identified (case-insensitive)",
            filename
        );
    }
}
