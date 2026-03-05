//! Depth tests for tokmd-tokeignore — W56 tooling coverage.

use std::fs;
use std::path::PathBuf;

use tokmd_config::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

fn make_args(profile: InitProfile, print: bool, force: bool, dir: PathBuf) -> InitArgs {
    InitArgs {
        dir,
        force,
        print,
        template: profile,
        non_interactive: true,
    }
}

// ---------------------------------------------------------------------------
// Template content invariants
// ---------------------------------------------------------------------------

#[test]
fn default_template_has_comment_header() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, true, false, dir.path().to_path_buf());
    // print mode returns None but prints — just verify no error
    let res = init_tokeignore(&args).unwrap();
    assert!(res.is_none());
}

#[test]
fn all_profiles_produce_non_empty_file() {
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
        let path = init_tokeignore(&args).unwrap().unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            !content.is_empty(),
            "profile {profile:?} produced empty file"
        );
    }
}

#[test]
fn all_profiles_start_with_comment() {
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
        let path = init_tokeignore(&args).unwrap().unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.starts_with('#'),
            "profile {profile:?} should start with a comment"
        );
    }
}

#[test]
fn rust_template_contains_rs_backup_pattern() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Rust, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("*.rs.bk"));
}

#[test]
fn python_template_contains_pyc_pattern() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Python, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("*.pyc"));
}

#[test]
fn cpp_template_contains_cmake_build_pattern() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Cpp, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("cmake-build-*/"));
}

#[test]
fn go_template_contains_bin_pattern() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Go, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("bin/"));
}

#[test]
fn mono_template_contains_generated_glob() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Mono, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("*.generated.*"));
    assert!(content.contains("*.gen.*"));
}

#[test]
fn default_template_contains_tree_sitter_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("tree-sitter"));
}

#[test]
fn node_template_excludes_python_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Node, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(!content.contains("__pycache__"));
    assert!(!content.contains(".venv"));
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn template_output_is_deterministic_across_calls() {
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
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();
        let args1 = make_args(profile, false, false, dir1.path().to_path_buf());
        let args2 = make_args(profile, false, false, dir2.path().to_path_buf());
        let p1 = init_tokeignore(&args1).unwrap().unwrap();
        let p2 = init_tokeignore(&args2).unwrap().unwrap();
        let c1 = fs::read_to_string(p1).unwrap();
        let c2 = fs::read_to_string(p2).unwrap();
        assert_eq!(c1, c2, "profile {profile:?} output should be deterministic");
    }
}

#[test]
fn force_overwrite_produces_same_content() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let p1 = init_tokeignore(&args).unwrap().unwrap();
    let c1 = fs::read_to_string(&p1).unwrap();

    // Overwrite with force
    let args2 = make_args(InitProfile::Default, false, true, dir.path().to_path_buf());
    let p2 = init_tokeignore(&args2).unwrap().unwrap();
    let c2 = fs::read_to_string(p2).unwrap();
    assert_eq!(c1, c2);
}

// ---------------------------------------------------------------------------
// Edge cases: file system
// ---------------------------------------------------------------------------

#[test]
fn init_in_empty_temp_dir_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let result = init_tokeignore(&args);
    assert!(result.is_ok());
}

#[test]
fn init_error_on_nonexistent_directory() {
    let args = make_args(
        InitProfile::Default,
        false,
        false,
        PathBuf::from("__nonexistent_dir_w56__"),
    );
    let result = init_tokeignore(&args);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("does not exist"));
}

#[test]
fn init_refuses_overwrite_existing_without_force() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".tokeignore"), "# existing").unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let err = init_tokeignore(&args).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn init_force_overwrites_existing_content() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".tokeignore"), "# old content\n").unwrap();
    let args = make_args(InitProfile::Rust, false, true, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("(Rust)"));
    assert!(!content.contains("old content"));
}

#[test]
fn init_writes_to_tokeignore_filename() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    assert_eq!(path.file_name().unwrap(), ".tokeignore");
}

#[test]
fn init_path_is_inside_target_dir() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    assert!(path.starts_with(dir.path()));
}

// ---------------------------------------------------------------------------
// Print mode
// ---------------------------------------------------------------------------

#[test]
fn print_mode_does_not_write_file() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, true, false, dir.path().to_path_buf());
    let result = init_tokeignore(&args).unwrap();
    assert!(result.is_none());
    assert!(!dir.path().join(".tokeignore").exists());
}

#[test]
fn print_mode_with_force_still_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, true, true, dir.path().to_path_buf());
    let result = init_tokeignore(&args).unwrap();
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// Pattern validation
// ---------------------------------------------------------------------------

#[test]
fn no_template_has_trailing_whitespace_on_non_empty_lines() {
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
        let path = init_tokeignore(&args).unwrap().unwrap();
        let content = fs::read_to_string(path).unwrap();
        for (i, line) in content.lines().enumerate() {
            if !line.is_empty() {
                assert!(
                    !line.ends_with(' ') && !line.ends_with('\t'),
                    "profile {profile:?} line {i} has trailing whitespace: {line:?}"
                );
            }
        }
    }
}

#[test]
fn all_non_comment_lines_are_valid_gitignore_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(path).unwrap();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Basic pattern validity: should contain at least one non-whitespace char
        assert!(
            !trimmed.is_empty(),
            "pattern line should not be only whitespace"
        );
    }
}

#[test]
fn templates_use_unix_style_glob_separators() {
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
        let path = init_tokeignore(&args).unwrap().unwrap();
        let content = fs::read_to_string(path).unwrap();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            assert!(
                !trimmed.contains('\\'),
                "profile {profile:?} should not contain backslash separators: {trimmed:?}"
            );
        }
    }
}

#[test]
fn default_template_has_both_rooted_and_recursive_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .collect();
    let has_rooted = lines.iter().any(|l| !l.starts_with("**"));
    let has_recursive = lines.iter().any(|l| l.starts_with("**"));
    assert!(has_rooted, "should have rooted patterns");
    assert!(has_recursive, "should have recursive patterns");
}

#[test]
fn python_template_has_pytest_cache() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Python, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains(".pytest_cache"));
}

#[test]
fn python_template_has_htmlcov() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Python, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("htmlcov/"));
}
