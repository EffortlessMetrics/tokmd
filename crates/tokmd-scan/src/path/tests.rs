use std::path::{Path, PathBuf};

use super::*;

#[test]
fn validated_root_accepts_existing_directory() {
    let dir = tempfile::tempdir().unwrap();
    let root = ValidatedRoot::new(dir.path()).unwrap();

    assert_eq!(root.input(), dir.path());
    assert!(root.canonical().is_absolute());
}

#[test]
fn validated_root_rejects_missing_path() {
    let dir = tempfile::tempdir().unwrap();
    let err = ValidatedRoot::new(dir.path().join("missing")).unwrap_err();

    assert!(err.to_string().contains("Path not found"));
}

#[test]
fn bounded_relative_path_strips_current_directory_segments() {
    let normalized = normalize_bounded_relative_path(Path::new("./src/./lib.rs")).unwrap();

    assert_eq!(normalized, PathBuf::from("src/lib.rs"));
}

#[test]
fn bounded_relative_path_rejects_absolute_path() {
    let err = normalize_bounded_relative_path(Path::new("/src/lib.rs")).unwrap_err();

    assert!(err.to_string().contains("must be relative"));
}

#[test]
fn bounded_relative_path_rejects_parent_traversal() {
    let err = normalize_bounded_relative_path(Path::new("../secret.txt")).unwrap_err();

    assert!(err.to_string().contains("parent traversal"));
}

#[test]
fn bounded_existing_relative_returns_root_relative_path() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "pub fn lib() {}\n").unwrap();
    let root = ValidatedRoot::new(dir.path()).unwrap();

    let bounded = BoundedPath::existing_relative(&root, Path::new("./src/lib.rs")).unwrap();

    assert_eq!(bounded.relative(), Path::new("src/lib.rs"));
    assert!(bounded.canonical().starts_with(root.canonical()));
}

#[test]
fn bounded_existing_child_rejects_path_outside_root() {
    let root_dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let outside_file = outside.path().join("secret.txt");
    std::fs::write(&outside_file, "secret").unwrap();
    let root = ValidatedRoot::new(root_dir.path()).unwrap();

    let err = BoundedPath::existing_child(&root, &outside_file).unwrap_err();

    assert!(err.to_string().contains("escapes scan root"));
}

#[test]
fn bounded_existing_relative_rejects_symlink_escape_when_supported() {
    let root_dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let outside_file = outside.path().join("secret.txt");
    let link = root_dir.path().join("secret-link.txt");
    std::fs::write(&outside_file, "secret").unwrap();

    if create_file_symlink(&outside_file, &link).is_err() {
        return;
    }

    let root = ValidatedRoot::new(root_dir.path()).unwrap();
    let err = BoundedPath::existing_relative(&root, Path::new("secret-link.txt")).unwrap_err();

    assert!(err.to_string().contains("escapes scan root"));
}

#[test]
fn bounded_existing_relative_reports_missing_only_for_true_not_found() {
    let root_dir = tempfile::tempdir().unwrap();
    let root = ValidatedRoot::new(root_dir.path()).unwrap();

    let err = BoundedPath::existing_relative(&root, Path::new("missing.rs")).unwrap_err();

    assert!(matches!(err, PathViolation::Missing(_)));
}

#[test]
fn bounded_existing_relative_dangling_symlink_is_not_missing_when_supported() {
    let root_dir = tempfile::tempdir().unwrap();
    let missing_target = root_dir.path().join("missing-target.txt");
    let link = root_dir.path().join("dangling-link.txt");

    if create_file_symlink(&missing_target, &link).is_err() {
        return;
    }

    let root = ValidatedRoot::new(root_dir.path()).unwrap();
    let err = BoundedPath::existing_relative(&root, Path::new("dangling-link.txt")).unwrap_err();

    assert!(matches!(err, PathViolation::CanonicalizeFailed { .. }));
    assert!(err.to_string().contains("Failed to resolve bounded path"));
}

#[cfg(unix)]
fn create_file_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(windows)]
fn create_file_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(src, dst)
}
