//! Tests for tokmd-tokeignore template generation.

use std::path::PathBuf;

use tempfile::TempDir;
use tokmd_config::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

fn make_args(dir: PathBuf, template: InitProfile, force: bool, print: bool) -> InitArgs {
    InitArgs {
        dir,
        template,
        force,
        print,
        non_interactive: true,
    }
}

// ========================
// Template Selection Tests
// ========================

#[test]
fn template_default_contains_target() {
    let temp = TempDir::new().unwrap();
    let args = make_args(
        temp.path().to_path_buf(),
        InitProfile::Default,
        false,
        false,
    );

    init_tokeignore(&args).unwrap();

    let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
    assert!(content.contains("target/"));
    assert!(content.contains("node_modules/"));
    assert!(content.contains("__pycache__/"));
}

#[test]
fn template_rust_contains_target() {
    let temp = TempDir::new().unwrap();
    let args = make_args(temp.path().to_path_buf(), InitProfile::Rust, false, false);

    init_tokeignore(&args).unwrap();

    let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
    assert!(content.contains("target/"));
    assert!(content.contains(".rs.bk"));
    // Rust template shouldn't have node_modules
    assert!(!content.contains("node_modules/"));
}

#[test]
fn template_node_contains_node_modules() {
    let temp = TempDir::new().unwrap();
    let args = make_args(temp.path().to_path_buf(), InitProfile::Node, false, false);

    init_tokeignore(&args).unwrap();

    let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
    assert!(content.contains("node_modules/"));
    assert!(content.contains("dist/"));
    // Node template shouldn't have target/
    assert!(!content.contains("target/"));
}

#[test]
fn template_mono_contains_multiple() {
    let temp = TempDir::new().unwrap();
    let args = make_args(temp.path().to_path_buf(), InitProfile::Mono, false, false);

    init_tokeignore(&args).unwrap();

    let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
    // Mono should have patterns from multiple languages
    assert!(content.contains("target/"));
    assert!(content.contains("node_modules/"));
    assert!(content.contains("__pycache__/"));
    assert!(content.contains("vendor/"));
}

#[test]
fn template_python_contains_pycache() {
    let temp = TempDir::new().unwrap();
    let args = make_args(temp.path().to_path_buf(), InitProfile::Python, false, false);

    init_tokeignore(&args).unwrap();

    let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
    assert!(content.contains("__pycache__/"));
    assert!(content.contains(".venv/"));
    assert!(content.contains("*.pyc"));
}

#[test]
fn template_go_contains_vendor() {
    let temp = TempDir::new().unwrap();
    let args = make_args(temp.path().to_path_buf(), InitProfile::Go, false, false);

    init_tokeignore(&args).unwrap();

    let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
    assert!(content.contains("vendor/"));
    assert!(content.contains("bin/"));
}

#[test]
fn template_cpp_contains_build() {
    let temp = TempDir::new().unwrap();
    let args = make_args(temp.path().to_path_buf(), InitProfile::Cpp, false, false);

    init_tokeignore(&args).unwrap();

    let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
    assert!(content.contains("build/"));
    assert!(content.contains("cmake-build-*/"));
}

// ========================
// Print Mode Tests
// ========================

#[test]
fn print_mode_does_not_write_file() {
    let temp = TempDir::new().unwrap();
    let args = make_args(temp.path().to_path_buf(), InitProfile::Default, false, true);

    // Should succeed without writing file
    init_tokeignore(&args).unwrap();

    // File should not exist
    assert!(!temp.path().join(".tokeignore").exists());
}

#[test]
fn print_mode_works_with_all_templates() {
    // Print mode should work even if directory doesn't need to exist
    // (but we use a valid temp dir anyway)
    let temp = TempDir::new().unwrap();

    for profile in [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let args = make_args(temp.path().to_path_buf(), profile, false, true);
        init_tokeignore(&args).unwrap();
    }

    // No files should be created
    assert!(!temp.path().join(".tokeignore").exists());
}

// ========================
// Error Handling Tests
// ========================

#[test]
fn error_when_directory_does_not_exist() {
    let nonexistent = PathBuf::from("/nonexistent/path/that/does/not/exist");
    let args = make_args(nonexistent, InitProfile::Default, false, false);

    let result = init_tokeignore(&args);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("does not exist"), "Error message: {}", err);
}

#[test]
fn error_when_file_exists_without_force() {
    let temp = TempDir::new().unwrap();

    // Create existing file
    std::fs::write(temp.path().join(".tokeignore"), "existing content").unwrap();

    let args = make_args(
        temp.path().to_path_buf(),
        InitProfile::Default,
        false,
        false,
    );
    let result = init_tokeignore(&args);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("already exists"), "Error message: {}", err);
}

// ========================
// Force Mode Tests
// ========================

#[test]
fn force_overwrites_existing_file() {
    let temp = TempDir::new().unwrap();

    // Create existing file with different content
    std::fs::write(temp.path().join(".tokeignore"), "old content").unwrap();

    let args = make_args(temp.path().to_path_buf(), InitProfile::Rust, true, false);
    init_tokeignore(&args).unwrap();

    let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
    // Should have new content, not old
    assert!(!content.contains("old content"));
    assert!(content.contains("target/"));
}

#[test]
fn force_works_when_file_does_not_exist() {
    let temp = TempDir::new().unwrap();

    // Force should work even when file doesn't exist
    let args = make_args(temp.path().to_path_buf(), InitProfile::Default, true, false);
    init_tokeignore(&args).unwrap();

    assert!(temp.path().join(".tokeignore").exists());
}

// ========================
// Edge Cases
// ========================

#[test]
fn writes_to_correct_path() {
    let temp = TempDir::new().unwrap();
    let subdir = temp.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();

    let args = make_args(subdir.clone(), InitProfile::Default, false, false);
    init_tokeignore(&args).unwrap();

    // File should be in subdir, not temp root
    assert!(subdir.join(".tokeignore").exists());
    assert!(!temp.path().join(".tokeignore").exists());
}

#[test]
fn all_templates_have_valid_gitignore_syntax() {
    let temp = TempDir::new().unwrap();

    for profile in [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let args = make_args(temp.path().to_path_buf(), profile, true, false);
        init_tokeignore(&args).unwrap();

        let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();

        // Basic validation: should have content
        assert!(!content.is_empty());

        // Should have comment header
        assert!(content.starts_with("#"));

        // Each non-empty, non-comment line should be a valid pattern
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            // Pattern shouldn't have invalid characters (basic check)
            assert!(!trimmed.contains('\t'), "Found tab in pattern: {}", trimmed);
        }
    }
}
