use std::path::Path;

use tokmd_io_port::MemFs;
use tokmd_walk::{file_size_from_memfs, list_files_from_memfs};

fn sample_memfs() -> MemFs {
    let mut fs = MemFs::new();
    fs.add_file("README.md", "# repo");
    fs.add_file("src/lib.rs", "pub fn lib() {}");
    fs.add_file("src/main.rs", "fn main() {}");
    fs.add_file("src/nested/mod.rs", "pub mod nested;");
    fs.add_bytes("assets/logo.svg", b"<svg />".to_vec());
    fs
}

#[test]
fn list_files_from_memfs_returns_sorted_root_relative_paths() {
    let fs = sample_memfs();

    let files = list_files_from_memfs(&fs, Path::new("src"), None).unwrap();

    assert_eq!(
        files,
        vec![
            Path::new("lib.rs").to_path_buf(),
            Path::new("main.rs").to_path_buf(),
            Path::new("nested/mod.rs").to_path_buf(),
        ]
    );
}

#[test]
fn list_files_from_memfs_root_dot_returns_all_files() {
    let fs = sample_memfs();

    let files = list_files_from_memfs(&fs, Path::new("."), None).unwrap();

    assert_eq!(
        files,
        vec![
            Path::new("README.md").to_path_buf(),
            Path::new("assets/logo.svg").to_path_buf(),
            Path::new("src/lib.rs").to_path_buf(),
            Path::new("src/main.rs").to_path_buf(),
            Path::new("src/nested/mod.rs").to_path_buf(),
        ]
    );
}

#[test]
fn list_files_from_memfs_dot_prefixed_root_matches_plain_root() {
    let fs = sample_memfs();

    let plain = list_files_from_memfs(&fs, Path::new("src"), None).unwrap();
    let dotted = list_files_from_memfs(&fs, Path::new("./src"), None).unwrap();

    assert_eq!(dotted, plain);
}

#[test]
fn list_files_from_memfs_respects_max_files() {
    let fs = sample_memfs();

    let files = list_files_from_memfs(&fs, Path::new("."), Some(2)).unwrap();

    assert_eq!(
        files,
        vec![
            Path::new("README.md").to_path_buf(),
            Path::new("assets/logo.svg").to_path_buf(),
        ]
    );
}

#[test]
fn list_files_from_memfs_zero_limit_returns_empty() {
    let fs = sample_memfs();

    let files = list_files_from_memfs(&fs, Path::new("."), Some(0)).unwrap();

    assert!(files.is_empty());
}

#[test]
fn list_files_from_memfs_missing_root_returns_empty() {
    let fs = sample_memfs();

    let files = list_files_from_memfs(&fs, Path::new("missing"), None).unwrap();

    assert!(files.is_empty());
}

#[test]
fn file_size_from_memfs_reads_relative_bytes() {
    let fs = sample_memfs();

    let size = file_size_from_memfs(&fs, Path::new("src"), Path::new("lib.rs")).unwrap();

    assert_eq!(size, "pub fn lib() {}".len() as u64);
}

#[test]
fn file_size_from_memfs_dot_prefixed_root_matches_plain_root() {
    let fs = sample_memfs();

    let plain = file_size_from_memfs(&fs, Path::new("src"), Path::new("lib.rs")).unwrap();
    let dotted = file_size_from_memfs(&fs, Path::new("./src"), Path::new("./lib.rs")).unwrap();

    assert_eq!(dotted, plain);
}

#[test]
fn file_size_from_memfs_missing_file_errors() {
    let fs = sample_memfs();

    let result = file_size_from_memfs(&fs, Path::new("src"), Path::new("ghost.rs"));

    assert!(result.is_err());
}
