use std::path::PathBuf;
use tokmd_model::normalize_path;

#[test]
fn normalize_path_prefix_partial_match() {
    let p = PathBuf::from("project_extra/file.rs");
    let prefix = PathBuf::from("project");
    assert_eq!(normalize_path(&p, Some(&prefix)), "project_extra/file.rs");
}

#[test]
fn normalize_path_prefix_mixed_slashes() {
    let p = PathBuf::from("my/prefix/dir/file.rs");
    let prefix = PathBuf::from("my\\prefix/"); // needs_replace=true, needs_slash=false
    assert_eq!(normalize_path(&p, Some(&prefix)), "dir/file.rs");
}
