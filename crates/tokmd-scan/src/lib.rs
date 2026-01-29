//! # tokmd-scan
//!
//! **Tier 3 (Adapter)**
//!
//! This crate adapts the `tokei` library for use within `tokmd`.
//! It isolates the dependency on `tokei` to a single location.
//!
//! ## What belongs here
//! * Tokei configuration and invocation
//! * Mapping `tokmd` args to `tokei` config
//!
//! ## What does NOT belong here
//! * Business logic (filtering, sorting)
//! * Output formatting

use anyhow::Result;
use std::path::PathBuf;
use tokei::{Config, Languages};

use tokmd_config::GlobalArgs;
use tokmd_types::ConfigMode;

pub fn scan(paths: &[PathBuf], args: &GlobalArgs) -> Result<Languages> {
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

    fn default_global_args() -> GlobalArgs {
        GlobalArgs {
            excluded: vec![],
            config: ConfigMode::Auto,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
            verbose: 0,
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
    fn scan_finds_rust_files() {
        let args = default_global_args();
        let paths = vec![test_path()];
        let result = scan(&paths, &args).unwrap();
        // Should find at least this lib.rs file
        assert!(!result.is_empty());
        assert!(result.get(&tokei::LanguageType::Rust).is_some());
    }

    #[test]
    fn scan_with_nonexistent_path_returns_empty() {
        let args = default_global_args();
        let paths = vec![PathBuf::from("/nonexistent/path/that/does/not/exist")];
        let result = scan(&paths, &args).unwrap();
        // Tokei returns empty for nonexistent paths
        assert!(result.is_empty());
    }

    // ========================
    // Config Flag Tests
    // ========================

    #[test]
    fn scan_with_hidden_flag() {
        let mut args = default_global_args();
        args.hidden = true;
        let paths = vec![test_path()];
        let result = scan(&paths, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_with_no_ignore_flag() {
        let mut args = default_global_args();
        args.no_ignore = true;
        let paths = vec![test_path()];
        // no_ignore should imply all other no_ignore_* flags
        let result = scan(&paths, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_with_individual_no_ignore_flags() {
        let mut args = default_global_args();
        args.no_ignore_parent = true;
        args.no_ignore_dot = true;
        args.no_ignore_vcs = true;
        let paths = vec![test_path()];
        let result = scan(&paths, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_with_treat_doc_strings_as_comments() {
        let mut args = default_global_args();
        args.treat_doc_strings_as_comments = true;
        let paths = vec![test_path()];
        let result = scan(&paths, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_with_config_mode_none() {
        let mut args = default_global_args();
        args.config = ConfigMode::None;
        let paths = vec![test_path()];
        let result = scan(&paths, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_with_excluded_patterns() {
        let mut args = default_global_args();
        args.excluded = vec!["target".to_string(), "*.min.js".to_string()];
        let paths = vec![test_path()];
        let result = scan(&paths, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_with_all_flags_combined() {
        let args = GlobalArgs {
            excluded: vec!["node_modules".to_string()],
            config: ConfigMode::None,
            hidden: true,
            no_ignore: true,
            no_ignore_parent: true,
            no_ignore_dot: true,
            no_ignore_vcs: true,
            treat_doc_strings_as_comments: true,
            verbose: 2,
        };
        let paths = vec![test_path()];
        // Should handle all flags without panicking
        let result = scan(&paths, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_returns_code_stats() {
        let args = default_global_args();
        let paths = vec![test_path()];
        let result = scan(&paths, &args).unwrap();

        let rust = result.get(&tokei::LanguageType::Rust).unwrap();
        // The lib.rs file should have some code
        assert!(rust.code > 0);
        assert!(rust.lines() > 0);
    }
}
