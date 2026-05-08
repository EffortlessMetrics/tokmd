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
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tokei::{Config, Languages};

use crate::path::ValidatedRoot;
use tokmd_settings::ScanOptions;
use tokmd_types::ConfigMode;

/// A single logical file supplied from memory rather than the host filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemoryFile {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

impl InMemoryFile {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            bytes: bytes.into(),
        }
    }
}

/// A scan result that keeps its backing temp root alive for downstream row modeling.
///
/// Keep this wrapper alive while any downstream code needs to read file metadata from
/// the scanned paths. `tokmd-model` uses the underlying paths to compute byte and token
/// counts after the scan phase.
///
/// When converting these scan results into `FileRow`s, pass [`Self::strip_prefix`] as the
/// `strip_prefix` argument so receipts keep the logical in-memory paths rather than the
/// temp backing root.
#[derive(Debug)]
pub struct MaterializedScan {
    languages: Languages,
    logical_paths: Vec<PathBuf>,
    root: tempfile::TempDir,
}

impl MaterializedScan {
    #[must_use]
    pub fn languages(&self) -> &Languages {
        &self.languages
    }

    #[must_use]
    pub fn logical_paths(&self) -> &[PathBuf] {
        &self.logical_paths
    }

    #[must_use]
    pub fn strip_prefix(&self) -> &Path {
        self.root.path()
    }
}

/// Scans a set of paths and computes line counts for each language found.
///
/// # Examples
///
/// ```
/// use std::fs;
/// use std::path::PathBuf;
/// use tokmd_settings::ScanOptions;
/// use tokmd_types::ConfigMode;
/// use tokmd_scan::scan;
///
/// # fn main() -> anyhow::Result<()> {
/// let dir = tempfile::tempdir()?;
/// let file_path = dir.path().join("main.rs");
/// fs::write(&file_path, "fn main() { println!(\"hello\"); }\n")?;
///
/// let paths = vec![file_path];
/// let options = ScanOptions {
///     config: ConfigMode::None,
///     ..Default::default()
/// };
///
/// let languages = scan(&paths, &options)?;
/// let rust_stats = languages.get(&tokei::LanguageType::Rust).unwrap();
///
/// assert_eq!(rust_stats.code, 1);
/// # Ok(())
/// # }
/// ```
pub fn scan(paths: &[PathBuf], args: &ScanOptions) -> Result<Languages> {
    let cfg = config_from_scan_options(args);
    let roots = validated_scan_roots(paths)?;
    let ignores = ignored_patterns(args, &roots);
    let ignore_refs: Vec<_> = ignores.iter().map(String::as_str).collect();
    let scan_paths: Vec<PathBuf> = roots
        .iter()
        .map(|root| root.canonical().to_path_buf())
        .collect();

    let mut languages = Languages::new();
    languages.get_statistics(&scan_paths, &ignore_refs, &cfg);
    rebase_report_paths(&mut languages, &roots);

    Ok(languages)
}

fn validated_scan_roots(paths: &[PathBuf]) -> Result<Vec<ValidatedRoot>> {
    paths
        .iter()
        .map(ValidatedRoot::new)
        .collect::<std::result::Result<_, _>>()
        .map_err(Into::into)
}

fn rebase_report_paths(languages: &mut Languages, roots: &[ValidatedRoot]) {
    for language in languages.values_mut() {
        for report in &mut language.reports {
            report.name = rebase_report_path(&report.name, roots);
        }
        for reports in language.children.values_mut() {
            for report in reports {
                report.name = rebase_report_path(&report.name, roots);
            }
        }
    }
}

fn rebase_report_path(path: &Path, roots: &[ValidatedRoot]) -> PathBuf {
    roots
        .iter()
        .filter_map(|root| {
            path.strip_prefix(root.canonical())
                .ok()
                .map(|relative| (root, relative))
        })
        .max_by_key(|(root, _)| root.canonical().components().count())
        .map_or_else(
            || path.to_path_buf(),
            |(root, relative)| {
                if relative.as_os_str().is_empty() {
                    root.input().to_path_buf()
                } else {
                    root.input().join(relative)
                }
            },
        )
}

/// Build the `tokei` config used for a scan from clap-free `ScanOptions`.
#[must_use]
pub fn config_from_scan_options(args: &ScanOptions) -> Config {
    build_config(args)
}

/// Normalize ordered in-memory inputs into deterministic logical paths.
///
/// This rejects empty, absolute, escaping, and case-only-colliding paths so
/// native and browser callers see the same contract.
pub fn normalize_in_memory_paths(inputs: &[InMemoryFile]) -> Result<Vec<PathBuf>> {
    normalize_logical_paths(inputs, true)
}

pub fn scan_in_memory(inputs: &[InMemoryFile], args: &ScanOptions) -> Result<MaterializedScan> {
    let root = tempfile::tempdir()?;
    let logical_paths = normalize_in_memory_paths(inputs)?;

    for (logical_path, input) in logical_paths.iter().zip(inputs) {
        let full_path = root.path().join(logical_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, &input.bytes)?;
    }

    let scan_root = vec![root.path().to_path_buf()];
    let languages = scan(&scan_root, args)?;

    Ok(MaterializedScan {
        languages,
        logical_paths,
        root,
    })
}

fn build_config(args: &ScanOptions) -> Config {
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

    cfg
}

fn ignored_patterns(args: &ScanOptions, roots: &[ValidatedRoot]) -> Vec<String> {
    let mut patterns = BTreeSet::new();

    for pattern in &args.excluded {
        patterns.insert(pattern.clone());

        if is_absolute_pattern(pattern) {
            continue;
        }

        let relative = normalize_relative_ignore_pattern(pattern);
        if relative.is_empty() {
            continue;
        }

        if !relative.starts_with("**/") {
            patterns.insert(format!("**/{relative}"));
        }

        for root in roots {
            let canonical = normalize_slashes(&root.canonical().to_string_lossy());
            patterns.insert(format!("{}/{}", canonical.trim_end_matches('/'), relative));
        }
    }

    patterns.into_iter().collect()
}

fn is_absolute_pattern(pattern: &str) -> bool {
    let path = Path::new(pattern);
    path.is_absolute()
        || pattern.starts_with('/')
        || pattern.starts_with('\\')
        || pattern.as_bytes().get(1).is_some_and(|byte| *byte == b':')
}

fn normalize_relative_ignore_pattern(pattern: &str) -> String {
    let mut normalized = normalize_slashes(pattern);
    while let Some(rest) = normalized.strip_prefix("./") {
        normalized = rest.to_string();
    }
    normalized.trim_start_matches('/').to_string()
}

fn normalize_logical_paths(
    inputs: &[InMemoryFile],
    case_insensitive: bool,
) -> Result<Vec<PathBuf>> {
    let mut seen = BTreeSet::new();
    let mut normalized = Vec::with_capacity(inputs.len());

    for input in inputs {
        let logical_path = normalize_logical_path(&input.path)?;
        if !seen.insert(logical_path_key(&logical_path, case_insensitive)) {
            anyhow::bail!("Duplicate in-memory path: {}", logical_path.display());
        }
        normalized.push(logical_path);
    }

    Ok(normalized)
}

fn logical_path_key(path: &Path, case_insensitive: bool) -> String {
    let rendered = path.to_string_lossy();
    if case_insensitive {
        rendered.to_lowercase()
    } else {
        rendered.into_owned()
    }
}

fn normalize_logical_path(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().is_empty() {
        anyhow::bail!("In-memory path must not be empty");
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => normalized.push(segment),
            Component::CurDir => {}
            Component::ParentDir => {
                anyhow::bail!(
                    "In-memory path must not contain parent traversal: {}",
                    path.display()
                );
            }
            Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("In-memory path must be relative: {}", path.display());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        anyhow::bail!("In-memory path must resolve to a file: {}", path.display());
    }

    Ok(normalized)
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

    #[test]
    fn validated_scan_roots_resolve_parent_segments_before_walking() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path().join("repo");
        let src = root.join("src");
        fs::create_dir_all(&src)?;
        fs::write(src.join("lib.rs"), "pub fn lib() {}\n")?;

        let aliased_root = src.join("..");
        let roots = validated_scan_roots(&[aliased_root])?;

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].canonical(), fs::canonicalize(&root)?);
        Ok(())
    }

    #[test]
    fn scan_rebases_canonical_walk_paths_to_input_root() -> Result<()> {
        let args = default_scan_options();
        let dir = tempfile::tempdir()?;
        let root = dir.path().join("repo");
        let src = root.join("src");
        fs::create_dir_all(&src)?;
        fs::write(src.join("lib.rs"), "pub fn lib() {}\n")?;

        let aliased_root = src.join("..");
        let languages = scan(std::slice::from_ref(&aliased_root), &args)?;
        let rust = languages
            .get(&tokei::LanguageType::Rust)
            .expect("scan should find the Rust file");
        let report = rust
            .reports
            .iter()
            .find(|report| report.name.ends_with("src/lib.rs"))
            .expect("scan should preserve a report for src/lib.rs");

        assert!(
            report.name.starts_with(&aliased_root),
            "report path {} should be rebased under input root {}",
            report.name.display(),
            aliased_root.display()
        );
        assert_ne!(
            report.name,
            fs::canonicalize(&root)?.join("src/lib.rs"),
            "report path should preserve the caller-facing input root"
        );
        Ok(())
    }

    #[test]
    fn scan_rebases_embedded_child_report_paths_to_input_root() -> Result<()> {
        let args = default_scan_options();
        let dir = tempfile::tempdir()?;
        let root = dir.path().join("repo");
        let web = root.join("web");
        fs::create_dir_all(&web)?;
        fs::write(
            web.join("index.html"),
            "<html><script>const answer = 42;</script></html>\n",
        )?;

        let aliased_root = web.join("..");
        let languages = scan(std::slice::from_ref(&aliased_root), &args)?;
        let child_report = languages
            .values()
            .flat_map(|language| language.children.values())
            .flatten()
            .find(|report| report.name.ends_with("web/index.html"))
            .expect("scan should preserve embedded child report paths");

        assert!(
            child_report.name.starts_with(&aliased_root),
            "child report path {} should be rebased under input root {}",
            child_report.name.display(),
            aliased_root.display()
        );
        assert_ne!(
            child_report.name,
            fs::canonicalize(&root)?.join("web/index.html"),
            "child report path should preserve the caller-facing input root"
        );
        Ok(())
    }

    #[test]
    fn scan_keeps_relative_excludes_matching_canonical_walk_roots() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path().join("repo");
        let secret = root.join("secret_folder");
        fs::create_dir_all(&secret)?;
        fs::write(secret.join("app.rs"), "fn secret() {}\n")?;
        fs::write(root.join("other.rs"), "fn other() {}\n")?;

        let mut args = default_scan_options();
        args.excluded = vec!["secret_folder/**".to_string()];
        let languages = scan(&[root.join(".")], &args)?;
        let rust = languages
            .get(&tokei::LanguageType::Rust)
            .expect("scan should find the visible Rust file");
        let report_paths: Vec<_> = rust
            .reports
            .iter()
            .map(|report| normalize_slashes(&report.name.to_string_lossy()))
            .collect();

        assert!(
            report_paths.iter().any(|path| path.ends_with("other.rs")),
            "visible file should be scanned: {report_paths:?}"
        );
        assert!(
            !report_paths
                .iter()
                .any(|path| path.contains("secret_folder")),
            "relative exclude should still match after canonicalizing scan roots: {report_paths:?}"
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

    #[test]
    fn normalize_logical_path_strips_dot_segments() -> Result<()> {
        let normalized = normalize_logical_path(Path::new("./src/./lib.rs"))?;
        assert_eq!(normalized, PathBuf::from("src/lib.rs"));
        Ok(())
    }

    #[test]
    fn normalize_logical_path_rejects_absolute_paths() {
        let err = normalize_logical_path(Path::new("/src/lib.rs")).unwrap_err();
        assert!(err.to_string().contains("must be relative"));
    }

    #[test]
    fn normalize_logical_path_rejects_parent_traversal() {
        let err = normalize_logical_path(Path::new("../src/lib.rs")).unwrap_err();
        assert!(err.to_string().contains("parent traversal"));
    }

    #[test]
    fn normalize_logical_paths_rejects_duplicate_after_normalization() {
        let inputs = vec![
            InMemoryFile::new("./src/lib.rs", "fn main() {}\n"),
            InMemoryFile::new("src/lib.rs", "fn main() {}\n"),
        ];

        let err = normalize_logical_paths(&inputs, false).unwrap_err();
        assert!(err.to_string().contains("Duplicate in-memory path"));
    }

    #[test]
    fn normalize_logical_paths_rejects_case_only_collision_on_case_insensitive_fs() {
        let inputs = vec![
            InMemoryFile::new("src/lib.rs", "fn main() {}\n"),
            InMemoryFile::new("SRC/LIB.rs", "fn main() {}\n"),
        ];

        let err = normalize_logical_paths(&inputs, true).unwrap_err();
        assert!(err.to_string().contains("Duplicate in-memory path"));
    }
}

pub mod exclude;
pub mod math;
pub mod path;
pub mod tokeignore;
pub mod walk;

pub use exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};
pub use math::{gini_coefficient, percentile, round_f64, safe_ratio};
pub use path::{normalize_rel_path, normalize_slashes};
pub use tokeignore::{InitArgs, InitProfile, init_tokeignore};
