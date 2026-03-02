//! BDD-style scenario tests for tokmd-walk.
//!
//! These tests exercise filesystem traversal edge cases:
//! gitignore support, symlinks, hidden files, and empty directories.

use std::path::PathBuf;

use tempfile::TempDir;
use tokmd_walk::{file_size, list_files};

// ============================================================================
// Helpers
// ============================================================================

/// Create a non-git temp directory so `git ls-files` returns None and the
/// WalkBuilder fallback path is exercised.
fn non_git_tempdir() -> TempDir {
    TempDir::new().expect("failed to create tempdir")
}

/// Create a temp directory with `git init` so the `ignore` crate recognises
/// `.gitignore` files. No files are added/committed, so `git ls-files`
/// returns an empty list and falls back to WalkBuilder.
fn git_tempdir() -> TempDir {
    let tmp = TempDir::new().expect("failed to create tempdir");
    std::process::Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("git init failed");
    tmp
}

/// Sorted file name strings from `list_files` output.
fn file_names(files: &[PathBuf]) -> Vec<String> {
    files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

// ============================================================================
// Scenario: .gitignore support (WalkBuilder fallback path)
// ============================================================================

#[test]
fn gitignore_excludes_matching_files() {
    // Given a git directory with a .gitignore that ignores *.log files
    let tmp = git_tempdir();
    std::fs::write(tmp.path().join(".gitignore"), "*.log\n").unwrap();
    std::fs::write(tmp.path().join("app.rs"), "fn main() {}").unwrap();
    std::fs::write(tmp.path().join("debug.log"), "log data").unwrap();
    std::fs::write(tmp.path().join("trace.log"), "trace data").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then .log files are excluded and .rs file is present
    assert!(
        names.iter().any(|n| n.contains("app.rs")),
        "app.rs should be listed"
    );
    assert!(
        !names.iter().any(|n| n.ends_with(".log")),
        "*.log files should be excluded by .gitignore"
    );
    // .gitignore itself is present (it's a regular file, not ignored)
    assert!(
        names.iter().any(|n| n.contains(".gitignore")),
        ".gitignore should be listed"
    );
}

#[test]
fn gitignore_excludes_directories() {
    // Given a .gitignore that ignores a whole directory
    let tmp = git_tempdir();
    std::fs::write(tmp.path().join(".gitignore"), "build/\n").unwrap();
    std::fs::write(tmp.path().join("src.rs"), "code").unwrap();
    std::fs::create_dir_all(tmp.path().join("build")).unwrap();
    std::fs::write(tmp.path().join("build/output.bin"), "binary").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then the build directory contents are excluded
    assert!(names.iter().any(|n| n.contains("src.rs")));
    assert!(
        !names.iter().any(|n| n.contains("build")),
        "build/ dir should be excluded by .gitignore"
    );
}

#[test]
fn gitignore_negation_re_includes_file() {
    // Given a .gitignore with a negation pattern
    let tmp = git_tempdir();
    std::fs::write(tmp.path().join(".gitignore"), "*.log\n!important.log\n").unwrap();
    std::fs::write(tmp.path().join("debug.log"), "debug").unwrap();
    std::fs::write(tmp.path().join("important.log"), "keep me").unwrap();
    std::fs::write(tmp.path().join("app.rs"), "code").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then important.log is re-included while debug.log stays excluded
    assert!(
        !names.iter().any(|n| n.contains("debug.log")),
        "debug.log should be excluded"
    );
    assert!(
        names.iter().any(|n| n.contains("important.log")),
        "important.log should be re-included via negation"
    );
    assert!(names.iter().any(|n| n.contains("app.rs")));
}

#[test]
fn nested_gitignore_applies_to_subdirectory() {
    // Given a nested .gitignore in a subdirectory
    let tmp = git_tempdir();
    std::fs::create_dir_all(tmp.path().join("sub")).unwrap();
    std::fs::write(tmp.path().join("sub/.gitignore"), "*.tmp\n").unwrap();
    std::fs::write(tmp.path().join("sub/code.rs"), "code").unwrap();
    std::fs::write(tmp.path().join("sub/scratch.tmp"), "temp").unwrap();
    // A .tmp at root should NOT be excluded (nested gitignore is scoped)
    std::fs::write(tmp.path().join("root.tmp"), "root temp").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then sub/scratch.tmp is excluded, but root.tmp is not
    assert!(names.iter().any(|n| n.contains("code.rs")));
    assert!(
        !names.iter().any(|n| n.contains("scratch.tmp")),
        "sub/.gitignore should exclude scratch.tmp"
    );
    assert!(
        names.iter().any(|n| n.contains("root.tmp")),
        "root.tmp should not be affected by sub/.gitignore"
    );
}

// ============================================================================
// Scenario: Symlinks (follow_links = false)
// ============================================================================

#[cfg(unix)]
#[test]
fn symlinks_are_not_followed() {
    use std::os::unix::fs::symlink;

    // Given a directory with a symlink to another directory
    let tmp = non_git_tempdir();
    let target_dir = TempDir::new().unwrap();
    std::fs::write(target_dir.path().join("outside.txt"), "external").unwrap();
    std::fs::write(tmp.path().join("local.txt"), "local").unwrap();
    symlink(target_dir.path(), tmp.path().join("link_to_outside")).unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then the symlinked directory contents are not traversed
    assert!(names.iter().any(|n| n.contains("local.txt")));
    assert!(
        !names.iter().any(|n| n.contains("outside.txt")),
        "symlinked dir contents should not be followed"
    );
}

#[cfg(unix)]
#[test]
fn file_symlinks_are_excluded() {
    use std::os::unix::fs::symlink;

    // Given a symlink pointing to a regular file
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("real.txt"), "real").unwrap();
    symlink(tmp.path().join("real.txt"), tmp.path().join("link.txt")).unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then the real file is listed but the symlink is not (follow_links=false
    // means symlink entries have file_type() returning symlink, not file)
    assert!(names.iter().any(|n| n.contains("real.txt")));
    // symlinks are not regular files, so they should be excluded
    assert!(
        !names.iter().any(|n| n.contains("link.txt")),
        "file symlinks should be excluded"
    );
}

// On Windows, symlink creation requires special privileges, so we test conditionally
#[cfg(windows)]
#[test]
fn symlinks_not_followed_windows() {
    use std::os::windows::fs::symlink_dir;

    let tmp = non_git_tempdir();
    let target_dir = TempDir::new().unwrap();
    std::fs::write(target_dir.path().join("outside.txt"), "external").unwrap();
    std::fs::write(tmp.path().join("local.txt"), "local").unwrap();

    // Symlink creation may fail without elevated privileges; skip if so
    if symlink_dir(target_dir.path(), tmp.path().join("link_to_outside")).is_err() {
        eprintln!("Skipping symlink test: insufficient privileges on Windows");
        return;
    }

    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    assert!(names.iter().any(|n| n.contains("local.txt")));
    assert!(
        !names.iter().any(|n| n.contains("outside.txt")),
        "symlinked dir should not be followed"
    );
}

// ============================================================================
// Scenario: Hidden files (hidden = false means DO include them)
// ============================================================================

#[test]
fn hidden_files_are_included() {
    // Given a directory with hidden (dot-prefixed) files
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("visible.txt"), "visible").unwrap();
    std::fs::write(tmp.path().join(".hidden"), "hidden").unwrap();
    std::fs::write(tmp.path().join(".dotfile.cfg"), "config").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then hidden files ARE included (hidden(false) means "don't skip hidden")
    assert!(names.iter().any(|n| n.contains("visible.txt")));
    assert!(
        names.iter().any(|n| n.contains(".hidden")),
        "hidden files should be included"
    );
    assert!(
        names.iter().any(|n| n.contains(".dotfile.cfg")),
        "dotfiles should be included"
    );
}

#[test]
fn hidden_directories_are_traversed() {
    // Given a hidden subdirectory with files
    let tmp = non_git_tempdir();
    std::fs::create_dir_all(tmp.path().join(".config")).unwrap();
    std::fs::write(tmp.path().join(".config/settings.json"), "{}").unwrap();
    std::fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then files inside hidden directories are also included
    assert!(names.iter().any(|n| n.contains("main.rs")));
    assert!(
        names.iter().any(|n| n.contains("settings.json")),
        "files in hidden dirs should be included"
    );
}

#[test]
fn hidden_files_excluded_by_gitignore_are_omitted() {
    // Given a .gitignore that explicitly ignores a hidden file
    let tmp = git_tempdir();
    std::fs::write(tmp.path().join(".gitignore"), ".secret\n").unwrap();
    std::fs::write(tmp.path().join(".secret"), "password").unwrap();
    std::fs::write(tmp.path().join(".visible_hidden"), "visible").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then .secret is excluded (by gitignore) but .visible_hidden remains
    assert!(
        !names
            .iter()
            .any(|n| n.contains(".secret") && !n.contains("visible")),
        ".secret should be excluded by .gitignore"
    );
    assert!(
        names.iter().any(|n| n.contains(".visible_hidden")),
        ".visible_hidden should be included"
    );
}

// ============================================================================
// Scenario: Empty directories
// ============================================================================

#[test]
fn empty_directories_do_not_appear_in_results() {
    // Given a directory structure with empty subdirectories
    let tmp = non_git_tempdir();
    std::fs::create_dir_all(tmp.path().join("empty_dir")).unwrap();
    std::fs::create_dir_all(tmp.path().join("nested/empty")).unwrap();
    std::fs::write(tmp.path().join("file.txt"), "content").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // Then only regular files appear, no directory entries
    assert_eq!(files.len(), 1, "only one real file should be listed");
    assert_eq!(names[0], "file.txt");
}

#[test]
fn deeply_nested_empty_directories_are_invisible() {
    // Given deeply nested empty directory chains
    let tmp = non_git_tempdir();
    std::fs::create_dir_all(tmp.path().join("a/b/c/d/e")).unwrap();
    std::fs::write(tmp.path().join("root.txt"), "root").unwrap();

    // When we list files
    let files = list_files(tmp.path(), None).unwrap();

    // Then only the real file appears
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].to_string_lossy(), "root.txt");
}

// ============================================================================
// Scenario: Mixed edge cases
// ============================================================================

#[test]
fn gitignore_with_hidden_and_empty_dirs() {
    // Given a complex directory layout
    let tmp = git_tempdir();
    std::fs::write(tmp.path().join(".gitignore"), "*.tmp\nbuild/\n").unwrap();
    std::fs::create_dir_all(tmp.path().join("build")).unwrap();
    std::fs::write(tmp.path().join("build/out.bin"), "binary").unwrap();
    std::fs::create_dir_all(tmp.path().join("empty_dir")).unwrap();
    std::fs::create_dir_all(tmp.path().join(".hidden_dir")).unwrap();
    std::fs::write(tmp.path().join(".hidden_dir/config.yml"), "key: val").unwrap();
    std::fs::write(tmp.path().join("app.rs"), "fn main() {}").unwrap();
    std::fs::write(tmp.path().join("scratch.tmp"), "temp").unwrap();

    let files = list_files(tmp.path(), None).unwrap();
    let names = file_names(&files);

    // app.rs and .hidden_dir/config.yml and .gitignore should be listed
    assert!(names.iter().any(|n| n.contains("app.rs")));
    assert!(
        names.iter().any(|n| n.contains("config.yml")),
        "hidden dir contents should be visible"
    );
    assert!(names.iter().any(|n| n.contains(".gitignore")));
    // build/ contents and *.tmp should be excluded
    assert!(
        !names.iter().any(|n| n.contains("out.bin")),
        "build/ should be excluded"
    );
    assert!(
        !names.iter().any(|n| n.contains("scratch.tmp")),
        "*.tmp should be excluded"
    );
}

#[test]
fn max_files_with_gitignore_still_respects_limit() {
    // Given files with some ignored
    let tmp = git_tempdir();
    std::fs::write(tmp.path().join(".gitignore"), "*.log\n").unwrap();
    std::fs::write(tmp.path().join("a.rs"), "a").unwrap();
    std::fs::write(tmp.path().join("b.rs"), "b").unwrap();
    std::fs::write(tmp.path().join("c.rs"), "c").unwrap();
    std::fs::write(tmp.path().join("x.log"), "log").unwrap();

    // When we list with max_files=2
    let files = list_files(tmp.path(), Some(2)).unwrap();

    // Then at most 2 files are returned, and none are .log
    assert!(files.len() <= 2, "max_files should be respected");
    let names = file_names(&files);
    assert!(!names.iter().any(|n| n.ends_with(".log")));
}

// ============================================================================
// Scenario: file_size edge cases
// ============================================================================

#[test]
fn file_size_for_file_in_hidden_directory() {
    let tmp = non_git_tempdir();
    std::fs::create_dir_all(tmp.path().join(".hidden")).unwrap();
    std::fs::write(tmp.path().join(".hidden/data.bin"), "12345").unwrap();

    let size = file_size(tmp.path(), std::path::Path::new(".hidden/data.bin")).unwrap();
    assert_eq!(size, 5);
}

#[test]
fn file_size_for_dotfile() {
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join(".env"), "SECRET=abc").unwrap();

    let size = file_size(tmp.path(), std::path::Path::new(".env")).unwrap();
    assert_eq!(size, 10);
}

#[test]
fn file_size_for_deeply_nested_file() {
    let tmp = non_git_tempdir();
    std::fs::create_dir_all(tmp.path().join("a/b/c/d")).unwrap();
    std::fs::write(tmp.path().join("a/b/c/d/deep.txt"), "deep content!").unwrap();

    let size = file_size(tmp.path(), std::path::Path::new("a/b/c/d/deep.txt")).unwrap();
    assert_eq!(size, 13);
}

// ============================================================================
// Scenario: license_candidates detection
// ============================================================================

#[test]
fn given_license_variants_when_candidates_checked_then_all_detected() {
    use tokmd_walk::license_candidates;

    let files = vec![
        PathBuf::from("LICENSE"),
        PathBuf::from("LICENSE.md"),
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("LICENSE-APACHE"),
        PathBuf::from("COPYING"),
        PathBuf::from("NOTICE"),
        PathBuf::from("license.txt"),
    ];
    let result = license_candidates(&files);
    assert_eq!(
        result.license_files.len(),
        7,
        "all license file variants should be detected"
    );
}

#[test]
fn given_metadata_files_when_candidates_checked_then_correctly_classified() {
    use tokmd_walk::license_candidates;

    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
        PathBuf::from("pyproject.toml"),
        PathBuf::from("src/main.rs"),
        PathBuf::from("README.md"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.metadata_files.len(), 3);
    assert!(result.license_files.is_empty());
}

#[test]
fn given_empty_file_list_when_candidates_checked_then_empty_results() {
    use tokmd_walk::license_candidates;

    let result = license_candidates(&[]);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

#[test]
fn given_nested_license_files_when_candidates_checked_then_sorted_by_path() {
    use tokmd_walk::license_candidates;

    let files = vec![
        PathBuf::from("z/LICENSE"),
        PathBuf::from("a/LICENSE"),
        PathBuf::from("m/LICENSE"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files[0], PathBuf::from("a/LICENSE"));
    assert_eq!(result.license_files[1], PathBuf::from("m/LICENSE"));
    assert_eq!(result.license_files[2], PathBuf::from("z/LICENSE"));
}

// ============================================================================
// Scenario: list_files with max_files=0
// ============================================================================

#[test]
fn given_max_files_zero_when_listing_then_returns_empty() {
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("file.txt"), "content").unwrap();

    let files = list_files(tmp.path(), Some(0)).unwrap();
    assert!(
        files.is_empty(),
        "max_files=0 should always return empty vec"
    );
}

// ============================================================================
// Scenario: list_files with single file
// ============================================================================

#[test]
fn given_single_file_when_listed_then_exactly_one_returned() {
    let tmp = non_git_tempdir();
    std::fs::write(tmp.path().join("only.txt"), "data").unwrap();

    let files = list_files(tmp.path(), None).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].to_string_lossy().contains("only.txt"));
}

// ============================================================================
// Scenario: file_size for various contents
// ============================================================================

#[test]
fn given_unicode_content_when_file_size_checked_then_bytes_not_chars() {
    let tmp = non_git_tempdir();
    // "こんにちは" is 15 bytes in UTF-8 (5 chars × 3 bytes each)
    std::fs::write(tmp.path().join("unicode.txt"), "こんにちは").unwrap();

    let size = file_size(tmp.path(), std::path::Path::new("unicode.txt")).unwrap();
    assert_eq!(size, 15, "file_size should return bytes, not char count");
}

#[test]
fn given_missing_file_when_file_size_checked_then_error_returned() {
    let tmp = non_git_tempdir();
    let result = file_size(tmp.path(), std::path::Path::new("nonexistent.txt"));
    assert!(result.is_err(), "missing file should return error");
}

// ============================================================================
// Scenario: deeply nested file listing
// ============================================================================

#[test]
fn given_deeply_nested_files_when_listed_then_all_found() {
    let tmp = non_git_tempdir();
    std::fs::create_dir_all(tmp.path().join("a/b/c")).unwrap();
    std::fs::write(tmp.path().join("a/b/c/deep.txt"), "deep").unwrap();
    std::fs::write(tmp.path().join("a/shallow.txt"), "shallow").unwrap();
    std::fs::write(tmp.path().join("root.txt"), "root").unwrap();

    let files = list_files(tmp.path(), None).unwrap();
    assert_eq!(files.len(), 3, "all three files should be found");
}
