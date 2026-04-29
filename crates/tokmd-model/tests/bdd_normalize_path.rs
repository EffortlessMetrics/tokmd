use std::path::PathBuf;

use tokmd_model::normalize_path;

#[test]
fn normalize_path_prefix_partial_match() {
    let path = PathBuf::from("project_extra/file.rs");
    let prefix = PathBuf::from("project");

    assert_eq!(
        normalize_path(&path, Some(&prefix)),
        "project_extra/file.rs"
    );
}

#[test]
fn normalize_path_prefix_mixed_slashes() {
    let path = PathBuf::from("my/prefix/dir/file.rs");
    let prefix = PathBuf::from("my\\prefix/");

    assert_eq!(normalize_path(&path, Some(&prefix)), "dir/file.rs");
}
