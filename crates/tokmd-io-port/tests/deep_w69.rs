//! Deep tests for tokmd-io-port (W69).
//!
//! Covers ReadFs trait implementations for HostFs and MemFs,
//! error handling, directory inference, isolation, and property tests.

use std::path::{Path, PathBuf};

use tokmd_io_port::{HostFs, MemFs, ReadFs};

// ---------------------------------------------------------------------------
// MemFs – basic operations
// ---------------------------------------------------------------------------

#[test]
fn w69_memfs_read_string_roundtrip() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("a.txt"), "hello");
    assert_eq!(fs.read_to_string(Path::new("a.txt")).unwrap(), "hello");
}

#[test]
fn w69_memfs_read_bytes_roundtrip() {
    let mut fs = MemFs::new();
    fs.add_bytes(PathBuf::from("data.bin"), vec![0xCA, 0xFE]);
    assert_eq!(fs.read_bytes(Path::new("data.bin")).unwrap(), vec![0xCA, 0xFE]);
}

#[test]
fn w69_memfs_not_found_error() {
    let fs = MemFs::new();
    let err = fs.read_to_string(Path::new("ghost.txt")).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn w69_memfs_invalid_utf8_error() {
    let mut fs = MemFs::new();
    fs.add_bytes(PathBuf::from("bad.txt"), vec![0xFF, 0xFE]);
    let err = fs.read_to_string(Path::new("bad.txt")).unwrap_err();
    assert!(err.to_string().contains("invalid UTF-8"));
}

#[test]
fn w69_memfs_read_bytes_not_found() {
    let fs = MemFs::new();
    let err = fs.read_bytes(Path::new("missing.bin")).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

// ---------------------------------------------------------------------------
// MemFs – exists / is_file / is_dir
// ---------------------------------------------------------------------------

#[test]
fn w69_memfs_exists_file() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("f.txt"), "x");
    assert!(fs.exists(Path::new("f.txt")));
}

#[test]
fn w69_memfs_exists_returns_false_for_missing() {
    let fs = MemFs::new();
    assert!(!fs.exists(Path::new("nope")));
}

#[test]
fn w69_memfs_is_file_true() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("src/main.rs"), "fn main() {}");
    assert!(fs.is_file(Path::new("src/main.rs")));
}

#[test]
fn w69_memfs_is_file_false_for_dir() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("src/main.rs"), "");
    assert!(!fs.is_file(Path::new("src")));
}

#[test]
fn w69_memfs_is_dir_inferred() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("src/lib.rs"), "");
    assert!(fs.is_dir(Path::new("src")));
}

#[test]
fn w69_memfs_is_dir_false_for_file() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("src/lib.rs"), "");
    assert!(!fs.is_dir(Path::new("src/lib.rs")));
}

#[test]
fn w69_memfs_nested_dir_inference() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("a/b/c/d.txt"), "deep");
    assert!(fs.is_dir(Path::new("a")));
    assert!(fs.is_dir(Path::new("a/b")));
    assert!(fs.is_dir(Path::new("a/b/c")));
    assert!(!fs.is_dir(Path::new("a/b/c/d.txt")));
}

#[test]
fn w69_memfs_exists_for_inferred_dir() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("pkg/mod.rs"), "");
    assert!(fs.exists(Path::new("pkg")));
}

// ---------------------------------------------------------------------------
// MemFs – default / empty state
// ---------------------------------------------------------------------------

#[test]
fn w69_memfs_default_empty() {
    let fs = MemFs::default();
    assert!(!fs.exists(Path::new("anything")));
    assert!(!fs.is_file(Path::new("x")));
    assert!(!fs.is_dir(Path::new("y")));
}

// ---------------------------------------------------------------------------
// MemFs – isolation: two instances share nothing
// ---------------------------------------------------------------------------

#[test]
fn w69_memfs_isolation() {
    let mut fs1 = MemFs::new();
    let fs2 = MemFs::new();
    fs1.add_file(PathBuf::from("a.txt"), "data");
    assert!(fs1.exists(Path::new("a.txt")));
    assert!(!fs2.exists(Path::new("a.txt")));
}

// ---------------------------------------------------------------------------
// MemFs – overwrite semantics
// ---------------------------------------------------------------------------

#[test]
fn w69_memfs_overwrite() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("f.txt"), "v1");
    fs.add_file(PathBuf::from("f.txt"), "v2");
    assert_eq!(fs.read_to_string(Path::new("f.txt")).unwrap(), "v2");
}

// ---------------------------------------------------------------------------
// HostFs – real filesystem operations
// ---------------------------------------------------------------------------

#[test]
fn w69_hostfs_read_to_string() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("hi.txt");
    std::fs::write(&f, "hi").unwrap();
    assert_eq!(HostFs.read_to_string(&f).unwrap(), "hi");
}

#[test]
fn w69_hostfs_read_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("bin");
    std::fs::write(&f, &[1, 2, 3]).unwrap();
    assert_eq!(HostFs.read_bytes(&f).unwrap(), vec![1, 2, 3]);
}

#[test]
fn w69_hostfs_exists_and_predicates() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("x.txt");
    std::fs::write(&f, "").unwrap();
    assert!(HostFs.exists(&f));
    assert!(HostFs.is_file(&f));
    assert!(!HostFs.is_dir(&f));
    assert!(HostFs.is_dir(dir.path()));
}

#[test]
fn w69_hostfs_missing_file_error() {
    let err = HostFs.read_to_string(Path::new("/nonexistent_w69_xyz")).unwrap_err();
    assert!(err.to_string().len() > 0);
}
