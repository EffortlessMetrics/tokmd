//! Deep tests for tokmd-io-port (W76).
//!
//! ~15 tests covering: ReadFs trait contract for HostFs and MemFs,
//! file enumeration via MemFs, error display messages, binary vs text
//! round-trips, directory inference from nested paths, and isolation.

use std::path::{Path, PathBuf};

use tokmd_io_port::{HostFs, MemFs, ReadFs};

// ===========================================================================
// 1. MemFs - text file round-trip
// ===========================================================================

#[test]
fn w76_memfs_add_file_read_to_string() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("hello.txt"), "hello world");
    assert_eq!(
        fs.read_to_string(Path::new("hello.txt")).unwrap(),
        "hello world"
    );
}

#[test]
fn w76_memfs_empty_file_reads_empty_string() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("empty.txt"), "");
    assert_eq!(fs.read_to_string(Path::new("empty.txt")).unwrap(), "");
}

// ===========================================================================
// 2. MemFs - binary file round-trip
// ===========================================================================

#[test]
fn w76_memfs_add_bytes_read_bytes() {
    let mut fs = MemFs::new();
    let data: Vec<u8> = (0..=255).collect();
    fs.add_bytes(PathBuf::from("all_bytes.bin"), data.clone());
    assert_eq!(fs.read_bytes(Path::new("all_bytes.bin")).unwrap(), data);
}

#[test]
fn w76_memfs_read_bytes_on_text_file() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("text.txt"), "abc");
    assert_eq!(
        fs.read_bytes(Path::new("text.txt")).unwrap(),
        b"abc".to_vec()
    );
}

// ===========================================================================
// 3. MemFs - error paths
// ===========================================================================

#[test]
fn w76_memfs_read_string_not_found_includes_path() {
    let fs = MemFs::new();
    let err = fs.read_to_string(Path::new("missing/file.rs")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("not found"), "error: {msg}");
    assert!(msg.contains("missing"), "error should mention path: {msg}");
}

#[test]
fn w76_memfs_read_bytes_not_found_error() {
    let fs = MemFs::new();
    let err = fs.read_bytes(Path::new("gone.bin")).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn w76_memfs_invalid_utf8_error_includes_path() {
    let mut fs = MemFs::new();
    fs.add_bytes(PathBuf::from("data/bad.txt"), vec![0x80, 0x81, 0x82]);
    let err = fs.read_to_string(Path::new("data/bad.txt")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("invalid UTF-8"), "error: {msg}");
    assert!(msg.contains("bad.txt"), "error should mention path: {msg}");
}

// ===========================================================================
// 4. MemFs - directory inference and file enumeration
// ===========================================================================

#[test]
fn w76_memfs_deep_nested_directory_inference() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("a/b/c/d/e.rs"), "fn e() {}");
    assert!(fs.is_dir(Path::new("a")));
    assert!(fs.is_dir(Path::new("a/b")));
    assert!(fs.is_dir(Path::new("a/b/c")));
    assert!(fs.is_dir(Path::new("a/b/c/d")));
    assert!(!fs.is_dir(Path::new("a/b/c/d/e.rs")));
    assert!(fs.is_file(Path::new("a/b/c/d/e.rs")));
}

#[test]
fn w76_memfs_multiple_files_same_directory() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("src/lib.rs"), "pub mod a;");
    fs.add_file(PathBuf::from("src/a.rs"), "pub fn a() {}");
    fs.add_file(PathBuf::from("src/b.rs"), "pub fn b() {}");
    assert!(fs.is_dir(Path::new("src")));
    assert!(fs.is_file(Path::new("src/lib.rs")));
    assert!(fs.is_file(Path::new("src/a.rs")));
    assert!(fs.is_file(Path::new("src/b.rs")));
    assert!(!fs.is_file(Path::new("src/c.rs")));
}

#[test]
fn w76_memfs_exists_covers_both_files_and_dirs() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("pkg/mod.rs"), "");
    assert!(fs.exists(Path::new("pkg/mod.rs")));
    assert!(fs.exists(Path::new("pkg")));
    assert!(!fs.exists(Path::new("other")));
}

// ===========================================================================
// 5. MemFs - overwrite and isolation
// ===========================================================================

#[test]
fn w76_memfs_overwrite_replaces_content() {
    let mut fs = MemFs::new();
    fs.add_file(PathBuf::from("cfg.toml"), "[old]");
    fs.add_file(PathBuf::from("cfg.toml"), "[new]");
    assert_eq!(fs.read_to_string(Path::new("cfg.toml")).unwrap(), "[new]");
}

#[test]
fn w76_memfs_instances_are_isolated() {
    let mut a = MemFs::new();
    let b = MemFs::new();
    a.add_file(PathBuf::from("only_in_a.txt"), "data");
    assert!(a.exists(Path::new("only_in_a.txt")));
    assert!(!b.exists(Path::new("only_in_a.txt")));
}

// ===========================================================================
// 6. HostFs - real filesystem via tempfile
// ===========================================================================

#[test]
fn w76_hostfs_read_write_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("greeting.txt");
    std::fs::write(&file, "good morning").unwrap();

    let fs = HostFs;
    assert_eq!(fs.read_to_string(&file).unwrap(), "good morning");
    assert_eq!(fs.read_bytes(&file).unwrap(), b"good morning".to_vec());
    assert!(fs.exists(&file));
    assert!(fs.is_file(&file));
    assert!(!fs.is_dir(&file));
    assert!(fs.is_dir(dir.path()));
}

#[test]
fn w76_hostfs_missing_file_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does_not_exist.txt");
    assert!(HostFs.read_to_string(&missing).is_err());
    assert!(HostFs.read_bytes(&missing).is_err());
    assert!(!HostFs.exists(&missing));
}

#[test]
fn w76_hostfs_binary_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("data.bin");
    let bytes: Vec<u8> = (0..=127).collect();
    std::fs::write(&file, &bytes).unwrap();
    assert_eq!(HostFs.read_bytes(&file).unwrap(), bytes);
}
