//! # tokmd-scan::tokeignore
//!
//! **Tier 1 (Template Generation)**
//!
//! Template generation for `.tokeignore` files. Provides profile-based templates
//! for common project types.
//!
//! ## What belongs here
//! * `.tokeignore` template content by profile
//! * Template writing to disk or stdout
//! * Force overwrite logic
//!
//! ## What does NOT belong here
//! * Parsing or applying ignore patterns (tokei handles this)
//! * Scanning logic
//! * Modifying existing `.tokeignore` files (only create/overwrite)

use std::fs;
use std::path::PathBuf;

use anyhow::{Result, bail};

mod templates;

/// Template selection profile for `.tokeignore` generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitProfile {
    Default,
    Rust,
    Node,
    Mono,
    Python,
    Go,
    Cpp,
}

/// CLI-independent init options for creating `.tokeignore` templates.
#[derive(Debug, Clone)]
pub struct InitArgs {
    pub dir: PathBuf,
    pub force: bool,
    pub print: bool,
    pub template: InitProfile,
    pub non_interactive: bool,
}

pub fn init_tokeignore(args: &InitArgs) -> Result<Option<PathBuf>> {
    let template = templates::template(args.template);

    if args.print {
        print!("{template}");
        return Ok(None);
    }

    let dir: PathBuf = args.dir.clone();
    if !dir.exists() {
        bail!("Path not found: {}", dir.display());
    }

    let path = dir.join(".tokeignore");
    if path.exists() && !args.force {
        bail!(
            "{} already exists. Use --force to overwrite, or --print to just view the template.",
            path.display()
        );
    }

    fs::write(&path, template)?;
    Ok(Some(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_args(profile: InitProfile, print: bool, force: bool, dir: PathBuf) -> InitArgs {
        InitArgs {
            dir,
            force,
            print,
            template: profile,
            non_interactive: true,
        }
    }

    #[test]
    fn test_init_writes_file() {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
        let result = init_tokeignore(&args).unwrap();
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("# .tokeignore"));
    }

    #[test]
    fn test_init_rust_profile_writes_rust_template() {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(InitProfile::Rust, false, false, dir.path().to_path_buf());
        let result = init_tokeignore(&args).unwrap();
        let path = result.unwrap();
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("(Rust)"));
    }

    #[test]
    fn test_init_refuses_overwrite_without_force() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".tokeignore"), "existing").unwrap();
        let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
        let result = init_tokeignore(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_init_overwrites_with_force() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".tokeignore"), "old content").unwrap();
        let args = make_args(InitProfile::Default, false, true, dir.path().to_path_buf());
        let result = init_tokeignore(&args).unwrap();
        assert!(result.is_some());
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        assert!(content.contains("# .tokeignore"));
    }

    #[test]
    fn test_init_print_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(InitProfile::Default, true, false, dir.path().to_path_buf());
        let result = init_tokeignore(&args).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_init_nonexistent_dir_errors() {
        let args = make_args(
            InitProfile::Default,
            false,
            false,
            PathBuf::from("/nonexistent/dir/that/does/not/exist"),
        );
        let result = init_tokeignore(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Path not found"));
    }
}
