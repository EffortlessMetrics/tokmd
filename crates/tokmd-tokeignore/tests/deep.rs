//! Deep tests for tokmd-tokeignore: template content, init logic, and edge cases.

use std::fs;
use std::path::PathBuf;

use tokmd_cli_args::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

// ── Helper ─────────────────────────────────────────────────────────────

fn make_args(profile: InitProfile, print: bool, force: bool, dir: PathBuf) -> InitArgs {
    InitArgs {
        dir,
        force,
        print,
        template: profile,
        non_interactive: true,
    }
}

// ── Template content verification ──────────────────────────────────────

#[test]
fn all_profiles_write_distinct_content() {
    let profiles = [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ];
    let mut contents = Vec::new();
    for profile in profiles {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(profile, false, false, dir.path().to_path_buf());
        init_tokeignore(&args).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        contents.push(content);
    }
    // Each profile should produce unique content
    for (i, a) in contents.iter().enumerate() {
        for (j, b) in contents.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "profiles {i} and {j} should differ");
            }
        }
    }
}

#[test]
fn default_template_has_all_ecosystem_sections() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(content.contains("target/"));
    assert!(content.contains("node_modules/"));
    assert!(content.contains("__pycache__/"));
    assert!(content.contains("vendor/"));
    assert!(content.contains(".runs/"));
}

#[test]
fn rust_template_excludes_target_only() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Rust, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(content.contains("target/"));
    assert!(content.contains("(Rust)"));
    assert!(!content.contains("node_modules/"));
    assert!(!content.contains("__pycache__/"));
}

#[test]
fn node_template_excludes_node_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Node, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(content.contains("node_modules/"));
    assert!(content.contains("dist/"));
    assert!(content.contains("(Node)"));
    assert!(!content.contains("__pycache__/"));
}

#[test]
fn python_template_excludes_python_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Python, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(content.contains("__pycache__/"));
    assert!(content.contains(".venv/"));
    assert!(content.contains(".pytest_cache/"));
    assert!(content.contains("(Python)"));
}

#[test]
fn go_template_excludes_vendor() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Go, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(content.contains("vendor/"));
    assert!(content.contains("(Go)"));
}

#[test]
fn cpp_template_excludes_build_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Cpp, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(content.contains("cmake-build-*/"));
    assert!(content.contains("build/"));
    assert!(content.contains("(C++)"));
}

#[test]
fn mono_template_covers_all_ecosystems() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Mono, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(content.contains("target/"));
    assert!(content.contains("node_modules/"));
    assert!(content.contains("__pycache__/"));
    assert!(content.contains("vendor/"));
    assert!(content.contains("(Monorepo)"));
}

// ── Template properties ────────────────────────────────────────────────

#[test]
fn all_templates_end_with_newline() {
    let profiles = [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ];
    for profile in profiles {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(profile, false, false, dir.path().to_path_buf());
        init_tokeignore(&args).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        assert!(
            content.ends_with('\n'),
            "template for {profile:?} should end with newline"
        );
    }
}

#[test]
fn all_templates_contain_runs_exclusion() {
    let profiles = [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ];
    for profile in profiles {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(profile, false, false, dir.path().to_path_buf());
        init_tokeignore(&args).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        assert!(
            content.contains(".runs/"),
            "template for {profile:?} must exclude .runs/"
        );
    }
}

#[test]
fn all_templates_start_with_comment() {
    let profiles = [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ];
    for profile in profiles {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(profile, false, false, dir.path().to_path_buf());
        init_tokeignore(&args).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        assert!(
            content.starts_with('#'),
            "template for {profile:?} should start with a comment"
        );
    }
}

// ── Init logic ─────────────────────────────────────────────────────────

#[test]
fn init_creates_tokeignore_file() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let result = init_tokeignore(&args).unwrap();
    assert!(result.is_some());
    let path = result.unwrap();
    assert!(path.exists());
    assert!(path.ends_with(".tokeignore"));
}

#[test]
fn init_refuses_overwrite_without_force() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".tokeignore"), "existing content").unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let err = init_tokeignore(&args).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn init_force_overwrites_existing() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".tokeignore"), "old content").unwrap();
    let args = make_args(InitProfile::Default, false, true, dir.path().to_path_buf());
    let result = init_tokeignore(&args).unwrap();
    assert!(result.is_some());
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(content.contains("# .tokeignore"));
    assert!(!content.contains("old content"));
}

#[test]
fn init_print_returns_none_no_file_written() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, true, false, dir.path().to_path_buf());
    let result = init_tokeignore(&args).unwrap();
    assert!(result.is_none());
    assert!(!dir.path().join(".tokeignore").exists());
}

#[test]
fn init_nonexistent_directory_errors() {
    let args = make_args(
        InitProfile::Default,
        false,
        false,
        PathBuf::from("/tmp/this_path_absolutely_does_not_exist_12345"),
    );
    let err = init_tokeignore(&args).unwrap_err();
    assert!(err.to_string().contains("does not exist"));
}

#[test]
fn init_idempotent_with_force() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Rust, false, true, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content1 = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    init_tokeignore(&args).unwrap();
    let content2 = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert_eq!(
        content1, content2,
        "repeated init with force should be idempotent"
    );
}

#[test]
fn init_different_profiles_produce_different_files() {
    let dir1 = tempfile::tempdir().unwrap();
    let dir2 = tempfile::tempdir().unwrap();
    init_tokeignore(&make_args(
        InitProfile::Rust,
        false,
        false,
        dir1.path().to_path_buf(),
    ))
    .unwrap();
    init_tokeignore(&make_args(
        InitProfile::Node,
        false,
        false,
        dir2.path().to_path_buf(),
    ))
    .unwrap();
    let c1 = fs::read_to_string(dir1.path().join(".tokeignore")).unwrap();
    let c2 = fs::read_to_string(dir2.path().join(".tokeignore")).unwrap();
    assert_ne!(c1, c2);
}

// ── Pattern content checks ─────────────────────────────────────────────

#[test]
fn default_template_has_recursive_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    // Verify recursive glob patterns exist
    assert!(content.contains("**/target/"));
    assert!(content.contains("**/node_modules/"));
    assert!(content.contains("**/__pycache__/"));
}

#[test]
fn templates_no_empty_non_comment_lines_at_start() {
    let profiles = [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ];
    for profile in profiles {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(profile, false, false, dir.path().to_path_buf());
        init_tokeignore(&args).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        let first_line = content.lines().next().unwrap();
        assert!(
            first_line.starts_with('#'),
            "first line of {profile:?} template should be a comment, got: {first_line}"
        );
    }
}

#[test]
fn default_template_contains_generated_code_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(content.contains("generated/"));
    assert!(content.contains("*.generated.*"));
}

#[test]
fn default_template_contains_coverage_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(content.contains("coverage/"));
    assert!(content.contains("lcov.info"));
}
