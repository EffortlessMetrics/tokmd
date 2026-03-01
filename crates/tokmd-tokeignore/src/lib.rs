//! # tokmd-tokeignore
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

use tokmd_config::{InitArgs, InitProfile};

const TEMPLATE_DEFAULT: &str = r#"# .tokeignore
# Patterns here use gitignore syntax.
#
# Goal: keep LOC summaries focused on *your* code, not build artifacts or vendored blobs.
# Tune aggressively for your repos.

# --- Rust / Cargo ---
target/
**/target/

# --- Node / JS tooling ---
node_modules/
**/node_modules/
dist/
out/
build/
**/build/

# --- Python ---
__pycache__/
**/__pycache__/
.venv/
**/.venv/
venv/
**/venv/
.tox/
**/.tox/

# --- Common vendored / third-party dirs ---
vendor/
**/vendor/
third_party/
**/third_party/
external/
**/external/

# --- Generated code ---
generated/
**/generated/
*.generated.*
*.gen.*

# --- Coverage / reports ---
coverage/
**/coverage/
.coverage
lcov.info

# --- tokmd outputs ---
.runs/
**/.runs/

# --- Tree-sitter (common "big files" when vendored) ---
# Adjust to match your vendor layout.
**/tree-sitter*/src/parser.c
**/tree-sitter*/src/scanner.c
**/tree-sitter*/src/*_scanner.c
"#;

const TEMPLATE_RUST: &str = r#"# .tokeignore (Rust)
# Focus: ignore build outputs and generated artifacts.

target/
**/target/

**/*.rs.bk

# Coverage
coverage/
**/coverage/

# tokmd outputs
.runs/
**/.runs/
"#;

const TEMPLATE_NODE: &str = r#"# .tokeignore (Node)
node_modules/
**/node_modules/
dist/
**/dist/
out/
**/out/
build/
**/build/
coverage/
**/coverage/

# tokmd outputs
.runs/
**/.runs/
"#;

const TEMPLATE_MONO: &str = r#"# .tokeignore (Monorepo)
# A conservative monorepo template. Tune to your reality.

# Rust
target/
**/target/

# Node
node_modules/
**/node_modules/
dist/
**/dist/
out/
**/out/
build/
**/build/

# Python
__pycache__/
**/__pycache__/
.venv/
**/.venv/
venv/
**/venv/
.tox/
**/.tox/

# Common vendored / third-party
vendor/
**/vendor/
third_party/
**/third_party/
external/
**/external/

# Generated code
generated/
**/generated/
*.generated.*
*.gen.*

# Coverage / reports
coverage/
**/coverage/
.coverage
lcov.info

# tokmd outputs
.runs/
**/.runs/

# Tree-sitter vendoring (common big files)
**/tree-sitter*/src/parser.c
**/tree-sitter*/src/scanner.c
**/tree-sitter*/src/*_scanner.c
"#;

const TEMPLATE_PYTHON: &str = r#"# .tokeignore (Python)
__pycache__/
**/__pycache__/
*.pyc
.venv/
**/.venv/
venv/
**/venv/
.tox/
**/.tox/
.pytest_cache/
**/.pytest_cache/
htmlcov/
**/htmlcov/
.coverage

# tokmd outputs
.runs/
**/.runs/
"#;

const TEMPLATE_GO: &str = r#"# .tokeignore (Go)
vendor/
**/vendor/
bin/
**/bin/

# tokmd outputs
.runs/
**/.runs/
"#;

const TEMPLATE_CPP: &str = r#"# .tokeignore (C++)
build/
**/build/
cmake-build-*/
**/cmake-build-*/
out/
**/out/
.cache/
**/.cache/

# tokmd outputs
.runs/
**/.runs/
"#;

pub fn init_tokeignore(args: &InitArgs) -> Result<Option<PathBuf>> {
    let template = match args.template {
        InitProfile::Default => TEMPLATE_DEFAULT,
        InitProfile::Rust => TEMPLATE_RUST,
        InitProfile::Node => TEMPLATE_NODE,
        InitProfile::Mono => TEMPLATE_MONO,
        InitProfile::Python => TEMPLATE_PYTHON,
        InitProfile::Go => TEMPLATE_GO,
        InitProfile::Cpp => TEMPLATE_CPP,
    };

    if args.print {
        print!("{template}");
        return Ok(None);
    }

    let dir: PathBuf = args.dir.clone();
    if !dir.exists() {
        bail!("Directory does not exist: {}", dir.display());
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
    fn test_default_template_contains_expected_sections() {
        assert!(TEMPLATE_DEFAULT.contains("# .tokeignore"));
        assert!(TEMPLATE_DEFAULT.contains("target/"));
        assert!(TEMPLATE_DEFAULT.contains("node_modules/"));
        assert!(TEMPLATE_DEFAULT.contains("__pycache__/"));
        assert!(TEMPLATE_DEFAULT.contains(".runs/"));
    }

    #[test]
    fn test_rust_template_is_rust_specific() {
        assert!(TEMPLATE_RUST.contains("(Rust)"));
        assert!(TEMPLATE_RUST.contains("target/"));
        assert!(!TEMPLATE_RUST.contains("node_modules/"));
    }

    #[test]
    fn test_node_template_is_node_specific() {
        assert!(TEMPLATE_NODE.contains("(Node)"));
        assert!(TEMPLATE_NODE.contains("node_modules/"));
        assert!(!TEMPLATE_NODE.contains("__pycache__/"));
    }

    #[test]
    fn test_python_template_is_python_specific() {
        assert!(TEMPLATE_PYTHON.contains("(Python)"));
        assert!(TEMPLATE_PYTHON.contains("__pycache__/"));
        assert!(TEMPLATE_PYTHON.contains(".venv/"));
    }

    #[test]
    fn test_go_template_is_go_specific() {
        assert!(TEMPLATE_GO.contains("(Go)"));
        assert!(TEMPLATE_GO.contains("vendor/"));
    }

    #[test]
    fn test_cpp_template_is_cpp_specific() {
        assert!(TEMPLATE_CPP.contains("(C++)"));
        assert!(TEMPLATE_CPP.contains("cmake-build-*/"));
    }

    #[test]
    fn test_mono_template_covers_multiple_ecosystems() {
        assert!(TEMPLATE_MONO.contains("(Monorepo)"));
        assert!(TEMPLATE_MONO.contains("target/"));
        assert!(TEMPLATE_MONO.contains("node_modules/"));
        assert!(TEMPLATE_MONO.contains("__pycache__/"));
        assert!(TEMPLATE_MONO.contains("vendor/"));
    }

    #[test]
    fn test_all_templates_end_with_newline() {
        for template in [
            TEMPLATE_DEFAULT,
            TEMPLATE_RUST,
            TEMPLATE_NODE,
            TEMPLATE_MONO,
            TEMPLATE_PYTHON,
            TEMPLATE_GO,
            TEMPLATE_CPP,
        ] {
            assert!(template.ends_with('\n'), "template should end with newline");
        }
    }

    #[test]
    fn test_all_templates_contain_runs_dir() {
        for template in [
            TEMPLATE_DEFAULT,
            TEMPLATE_RUST,
            TEMPLATE_NODE,
            TEMPLATE_MONO,
            TEMPLATE_PYTHON,
            TEMPLATE_GO,
            TEMPLATE_CPP,
        ] {
            assert!(
                template.contains(".runs/"),
                "every template should exclude .runs/"
            );
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
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
}
