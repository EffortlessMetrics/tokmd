//! # tokmd-scan
//!
//! **Tier 1 (Adapter)**
//!
//! This crate adapts the `tokei` library for use within `tokmd`.
//! It isolates the dependency on `tokei` to a single location.
//!
//! ## What belongs here
//! * Tokei configuration and invocation
//! * Mapping `tokmd` args to `tokei` config
//!
//! ## What does NOT belong here
//! * Business logic (filtering, sorting, aggregation)
//! * Output formatting
//! * Receipt construction

use anyhow::Result;
use std::path::PathBuf;
use tokei::{Config, Languages};

use tokmd_settings::ScanOptions;
use tokmd_types::ConfigMode;

pub fn scan(paths: &[PathBuf], args: &ScanOptions) -> Result<Languages> {
    let mut cfg = match args.config {
        ConfigMode::Auto => Config::from_config_files(),
        ConfigMode::None => Config::default(),
    };

    // Only override config file settings when the user explicitly asked for it.
    if args.hidden {
        cfg.hidden = Some(true);
    }
    if args.no_ignore {
        cfg.no_ignore = Some(true);
        cfg.no_ignore_dot = Some(true);
        cfg.no_ignore_parent = Some(true);
        cfg.no_ignore_vcs = Some(true);
    }
    if args.no_ignore_dot {
        cfg.no_ignore_dot = Some(true);
    }
    if args.no_ignore_parent {
        cfg.no_ignore_parent = Some(true);
    }
    if args.no_ignore_vcs {
        cfg.no_ignore_vcs = Some(true);
    }
    if args.treat_doc_strings_as_comments {
        cfg.treat_doc_strings_as_comments = Some(true);
    }

    let ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();

    for path in paths {
        if !path.exists() {
            anyhow::bail!("Path not found: {}", path.display());
        }
    }

    let mut languages = Languages::new();
    languages.get_statistics(paths, &ignores, &cfg);

    Ok(languages)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_scan_options() -> ScanOptions {
        ScanOptions {
            excluded: vec![],
            config: ConfigMode::Auto,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        }
    }

    // Get a valid test path - the crate's own source directory
    fn test_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
    }

    // ========================
    // Basic Scan Tests
    // ========================

    #[test]
    fn scan_finds_rust_files() -> Result<()> {
        let args = default_scan_options();
        let paths = vec![test_path()];
        let result = scan(&paths, &args)?;
        // Should find at least this lib.rs file
        assert!(!result.is_empty());
        assert!(result.get(&tokei::LanguageType::Rust).is_some());
        Ok(())
    }

    #[test]
    fn scan_with_nonexistent_path_returns_error() -> Result<()> {
        let args = default_scan_options();
        let dir = tempfile::tempdir()?;
        let nonexistent = dir.path().join("definitely-not-created");
        let paths = vec![nonexistent];
        let result = scan(&paths, &args);
        // Should return an error for nonexistent paths
        assert!(result.is_err());
        assert!(
            result
                .expect_err("should have failed")
                .to_string()
                .contains("Path not found")
        );
        Ok(())
    }

    // ========================
    // Config Flag Tests
    // ========================

    #[test]
    fn scan_with_hidden_flag() -> Result<()> {
        let mut args = default_scan_options();
        args.hidden = true;
        let paths = vec![test_path()];
        let result = scan(&paths, &args);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn scan_with_no_ignore_flag() -> Result<()> {
        let mut args = default_scan_options();
        args.no_ignore = true;
        let paths = vec![test_path()];
        // no_ignore should imply all other no_ignore_* flags
        let result = scan(&paths, &args);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn scan_with_individual_no_ignore_flags() -> Result<()> {
        let mut args = default_scan_options();
        args.no_ignore_parent = true;
        args.no_ignore_dot = true;
        args.no_ignore_vcs = true;
        let paths = vec![test_path()];
        let result = scan(&paths, &args);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn scan_with_treat_doc_strings_as_comments() -> Result<()> {
        let mut args = default_scan_options();
        args.treat_doc_strings_as_comments = true;
        let paths = vec![test_path()];
        let result = scan(&paths, &args);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn scan_with_config_mode_none() -> Result<()> {
        let mut args = default_scan_options();
        args.config = ConfigMode::None;
        let paths = vec![test_path()];
        let result = scan(&paths, &args);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn scan_with_excluded_patterns() -> Result<()> {
        let mut args = default_scan_options();
        args.excluded = vec!["target".to_string(), "*.min.js".to_string()];
        let paths = vec![test_path()];
        let result = scan(&paths, &args);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn scan_with_all_flags_combined() -> Result<()> {
        let args = ScanOptions {
            excluded: vec!["node_modules".to_string()],
            config: ConfigMode::None,
            hidden: true,
            no_ignore: true,
            no_ignore_parent: true,
            no_ignore_dot: true,
            no_ignore_vcs: true,
            treat_doc_strings_as_comments: true,
        };
        let paths = vec![test_path()];
        // Should handle all flags without panicking
        let result = scan(&paths, &args);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn scan_returns_code_stats() -> Result<()> {
        let args = default_scan_options();
        let paths = vec![test_path()];
        let result = scan(&paths, &args)?;

        let rust = result
            .get(&tokei::LanguageType::Rust)
            .expect("should find rust in src/lib.rs");
        // The lib.rs file should have some code
        assert!(rust.code > 0);
        assert!(rust.lines() > 0);
        Ok(())
    }

    // ========================
    // Edge-case & boundary tests
    // ========================

    #[test]
    fn scan_with_empty_paths_returns_empty_languages() -> Result<()> {
        // Scanning zero paths should succeed but find nothing
        let args = default_scan_options();
        let result = scan(&[], &args)?;
        assert!(result.is_empty());
        Ok(())
    }

    #[test]
    fn scan_with_multiple_paths() -> Result<()> {
        let args = default_scan_options();
        let paths = vec![test_path(), test_path()];
        // Scanning the same path twice should still succeed
        let result = scan(&paths, &args)?;
        assert!(result.get(&tokei::LanguageType::Rust).is_some());
        Ok(())
    }

    #[test]
    fn scan_config_mode_none_vs_auto_both_find_rust() -> Result<()> {
        let paths = vec![test_path()];

        let mut args_auto = default_scan_options();
        args_auto.config = ConfigMode::Auto;
        let result_auto = scan(&paths, &args_auto)?;

        let mut args_none = default_scan_options();
        args_none.config = ConfigMode::None;
        let result_none = scan(&paths, &args_none)?;

        // Both modes must find Rust
        assert!(result_auto.get(&tokei::LanguageType::Rust).is_some());
        assert!(result_none.get(&tokei::LanguageType::Rust).is_some());
        Ok(())
    }

    #[test]
    fn scan_excluded_patterns_are_collected_as_str_slices() {
        // Verify the ignores conversion produces the right count
        let args = ScanOptions {
            excluded: vec!["a".into(), "b".into(), "c".into()],
            config: ConfigMode::None,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        };
        let ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();
        assert_eq!(ignores.len(), 3);
        assert_eq!(ignores, vec!["a", "b", "c"]);
    }
}
