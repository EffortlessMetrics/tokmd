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
    let dir = TempDir::new().unwrap();
    let nonexistent = dir.path().join("definitely-not-created");
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
// Determinism
// ========================

#[test]
fn template_is_deterministic_across_writes() {
    for profile in [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let temp1 = TempDir::new().unwrap();
        let temp2 = TempDir::new().unwrap();
        let args1 = make_args(temp1.path().to_path_buf(), profile, false, false);
        let args2 = make_args(temp2.path().to_path_buf(), profile, false, false);
        init_tokeignore(&args1).unwrap();
        init_tokeignore(&args2).unwrap();
        let c1 = std::fs::read_to_string(temp1.path().join(".tokeignore")).unwrap();
        let c2 = std::fs::read_to_string(temp2.path().join(".tokeignore")).unwrap();
        assert_eq!(c1, c2, "Template for {:?} must be deterministic", profile);
    }
}

// ========================
// Round-trip
// ========================

#[test]
fn round_trip_write_and_reread_is_identical() {
    for profile in [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let temp = TempDir::new().unwrap();
        let args = make_args(temp.path().to_path_buf(), profile, false, false);
        init_tokeignore(&args).unwrap();

        let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();

        // Write a second time with force and confirm byte-for-byte match
        let args2 = make_args(temp.path().to_path_buf(), profile, true, false);
        init_tokeignore(&args2).unwrap();
        let content2 = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();

        assert_eq!(
            content, content2,
            "Round-trip for {:?} must be identical",
            profile
        );
    }
}

// ========================
// Custom pattern appending
// ========================

#[test]
fn custom_patterns_can_be_appended_after_generation() {
    let temp = TempDir::new().unwrap();
    let args = make_args(
        temp.path().to_path_buf(),
        InitProfile::Default,
        false,
        false,
    );
    init_tokeignore(&args).unwrap();

    let path = temp.path().join(".tokeignore");
    let mut content = std::fs::read_to_string(&path).unwrap();

    // Append custom patterns
    content.push_str("\n# Custom\nmy_custom_dir/\n*.log\n");
    std::fs::write(&path, &content).unwrap();

    let final_content = std::fs::read_to_string(&path).unwrap();
    // Original patterns still present
    assert!(final_content.contains("target/"));
    assert!(final_content.contains("node_modules/"));
    // Custom patterns present
    assert!(final_content.contains("my_custom_dir/"));
    assert!(final_content.contains("*.log"));
}

// ========================
// Common pattern coverage
// ========================

#[test]
fn default_template_covers_common_build_artifact_dirs() {
    let temp = TempDir::new().unwrap();
    let args = make_args(
        temp.path().to_path_buf(),
        InitProfile::Default,
        false,
        false,
    );
    init_tokeignore(&args).unwrap();
    let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();

    // Common patterns that should appear in the default template
    assert!(content.contains("target/"), "missing target/");
    assert!(content.contains("node_modules/"), "missing node_modules/");
    assert!(content.contains("build/"), "missing build/");
    assert!(content.contains("dist/"), "missing dist/");
    assert!(content.contains("__pycache__/"), "missing __pycache__/");
    assert!(content.contains("vendor/"), "missing vendor/");
    assert!(content.contains("coverage/"), "missing coverage/");
    assert!(content.contains(".runs/"), "missing .runs/");

    // .git should NOT be in the template (handled natively by git/tokei)
    assert!(
        !content.contains(".git/"),
        ".git/ should not be in the template"
    );
}

#[test]
fn all_templates_contain_runs_output_dir() {
    for profile in [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let temp = TempDir::new().unwrap();
        let args = make_args(temp.path().to_path_buf(), profile, true, false);
        init_tokeignore(&args).unwrap();
        let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
        assert!(
            content.contains(".runs/"),
            "Profile {:?} must contain .runs/",
            profile
        );
    }
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
