//! BDD-style scenario tests for `MemFs`.
//!
//! Each test follows the Given/When/Then pattern exercising a single behaviour.

use std::path::Path;
use tokmd_io_port::{MemFs, ReadFs};

// ---------------------------------------------------------------------------
// Scenario: empty filesystem
// ---------------------------------------------------------------------------

#[test]
fn empty_fs_has_no_files() {
    let fs = MemFs::new();
    assert!(!fs.exists(Path::new("anything.txt")));
    assert!(!fs.is_file(Path::new("anything.txt")));
    assert!(!fs.is_dir(Path::new("anything.txt")));
}

#[test]
fn empty_fs_read_to_string_returns_not_found() {
    let fs = MemFs::new();
    let err = fs.read_to_string(Path::new("missing.txt")).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn empty_fs_read_bytes_returns_not_found() {
    let fs = MemFs::new();
    let err = fs.read_bytes(Path::new("missing.bin")).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn default_fs_is_empty() {
    let fs = MemFs::default();
    assert!(!fs.exists(Path::new("x")));
}

// ---------------------------------------------------------------------------
// Scenario: add a single file
// ---------------------------------------------------------------------------

#[test]
fn add_file_then_read_to_string() {
    let mut fs = MemFs::new();
    fs.add_file("hello.txt", "world");
    assert_eq!(fs.read_to_string(Path::new("hello.txt")).unwrap(), "world");
}

#[test]
fn add_file_then_read_bytes() {
    let mut fs = MemFs::new();
    fs.add_file("hello.txt", "world");
    assert_eq!(
        fs.read_bytes(Path::new("hello.txt")).unwrap(),
        b"world".to_vec()
    );
}

#[test]
fn added_file_exists() {
    let mut fs = MemFs::new();
    fs.add_file("a.rs", "fn main() {}");
    assert!(fs.exists(Path::new("a.rs")));
    assert!(fs.is_file(Path::new("a.rs")));
    assert!(!fs.is_dir(Path::new("a.rs")));
}

// ---------------------------------------------------------------------------
// Scenario: overwrite an existing file
// ---------------------------------------------------------------------------

#[test]
fn overwrite_replaces_content() {
    let mut fs = MemFs::new();
    fs.add_file("f.txt", "original");
    fs.add_file("f.txt", "replaced");
    assert_eq!(fs.read_to_string(Path::new("f.txt")).unwrap(), "replaced");
}

#[test]
fn overwrite_with_bytes_replaces_content() {
    let mut fs = MemFs::new();
    fs.add_file("f.bin", "text");
    fs.add_bytes("f.bin", vec![0xCA, 0xFE]);
    assert_eq!(fs.read_bytes(Path::new("f.bin")).unwrap(), vec![0xCA, 0xFE]);
}

// ---------------------------------------------------------------------------
// Scenario: large files
// ---------------------------------------------------------------------------

#[test]
fn large_text_file_roundtrips() {
    let mut fs = MemFs::new();
    let big = "x".repeat(1_000_000);
    fs.add_file("big.txt", big.clone());
    assert_eq!(fs.read_to_string(Path::new("big.txt")).unwrap(), big);
}

#[test]
fn large_binary_file_roundtrips() {
    let mut fs = MemFs::new();
    let big: Vec<u8> = (0..=255).cycle().take(500_000).collect();
    fs.add_bytes("big.bin", big.clone());
    assert_eq!(fs.read_bytes(Path::new("big.bin")).unwrap(), big);
}

// ---------------------------------------------------------------------------
// Scenario: nested paths and directory inference
// ---------------------------------------------------------------------------

#[test]
fn nested_path_creates_implicit_directory() {
    let mut fs = MemFs::new();
    fs.add_file("src/lib.rs", "// lib");
    assert!(fs.is_dir(Path::new("src")));
    assert!(fs.exists(Path::new("src")));
    assert!(!fs.is_file(Path::new("src")));
}

#[test]
fn deeply_nested_path_creates_all_ancestor_dirs() {
    let mut fs = MemFs::new();
    fs.add_file("a/b/c/d/e.txt", "deep");
    assert!(fs.is_dir(Path::new("a")));
    assert!(fs.is_dir(Path::new("a/b")));
    assert!(fs.is_dir(Path::new("a/b/c")));
    assert!(fs.is_dir(Path::new("a/b/c/d")));
    assert!(fs.is_file(Path::new("a/b/c/d/e.txt")));
}

#[test]
fn multiple_files_share_directory() {
    let mut fs = MemFs::new();
    fs.add_file("src/a.rs", "a");
    fs.add_file("src/b.rs", "b");
    assert!(fs.is_dir(Path::new("src")));
    assert_eq!(fs.read_to_string(Path::new("src/a.rs")).unwrap(), "a");
    assert_eq!(fs.read_to_string(Path::new("src/b.rs")).unwrap(), "b");
}

#[test]
fn sibling_directories_are_independent() {
    let mut fs = MemFs::new();
    fs.add_file("src/main.rs", "main");
    fs.add_file("tests/test.rs", "test");
    assert!(fs.is_dir(Path::new("src")));
    assert!(fs.is_dir(Path::new("tests")));
    assert!(!fs.is_dir(Path::new("lib")));
}

// ---------------------------------------------------------------------------
// Scenario: edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_content_file() {
    let mut fs = MemFs::new();
    fs.add_file("empty.txt", "");
    assert!(fs.is_file(Path::new("empty.txt")));
    assert_eq!(fs.read_to_string(Path::new("empty.txt")).unwrap(), "");
    assert_eq!(
        fs.read_bytes(Path::new("empty.txt")).unwrap(),
        Vec::<u8>::new()
    );
}

#[test]
fn empty_bytes_file() {
    let mut fs = MemFs::new();
    fs.add_bytes("empty.bin", Vec::<u8>::new());
    assert!(fs.is_file(Path::new("empty.bin")));
    assert_eq!(fs.read_bytes(Path::new("empty.bin")).unwrap().len(), 0);
}

#[test]
fn binary_content_invalid_utf8() {
    let mut fs = MemFs::new();
    fs.add_bytes("bad.bin", vec![0xFF, 0xFE, 0x00]);
    let err = fs.read_to_string(Path::new("bad.bin")).unwrap_err();
    assert!(err.to_string().contains("invalid UTF-8"));
    assert_eq!(
        fs.read_bytes(Path::new("bad.bin")).unwrap(),
        vec![0xFF, 0xFE, 0x00]
    );
}

#[test]
fn unicode_content_roundtrips() {
    let mut fs = MemFs::new();
    let text = "\u{3053}\u{3093}\u{306b}\u{3061}\u{306f}\u{4e16}\u{754c} \u{1f30d}";
    fs.add_file("uni.txt", text);
    assert_eq!(fs.read_to_string(Path::new("uni.txt")).unwrap(), text);
}

#[test]
fn path_with_special_characters() {
    let mut fs = MemFs::new();
    fs.add_file("dir with spaces/file (1).txt", "ok");
    assert!(fs.is_file(Path::new("dir with spaces/file (1).txt")));
    assert!(fs.is_dir(Path::new("dir with spaces")));
}

#[test]
fn forward_slash_paths_consistency() {
    let mut fs = MemFs::new();
    fs.add_file("a/b/c.txt", "content");
    assert!(fs.exists(Path::new("a/b/c.txt")));
    assert!(fs.is_dir(Path::new("a/b")));
    assert!(fs.is_dir(Path::new("a")));
}

#[test]
fn nonexistent_sibling_does_not_exist() {
    let mut fs = MemFs::new();
    fs.add_file("src/lib.rs", "");
    assert!(!fs.exists(Path::new("src/main.rs")));
    assert!(!fs.is_file(Path::new("src/main.rs")));
}

#[test]
fn file_is_not_directory_even_with_similar_prefix() {
    let mut fs = MemFs::new();
    fs.add_file("src", "I am a file named src");
    fs.add_file("src2/lib.rs", "");
    assert!(fs.is_file(Path::new("src")));
}

#[test]
fn many_files_in_one_directory() {
    let mut fs = MemFs::new();
    for i in 0..100 {
        fs.add_file(format!("dir/file_{i}.txt"), format!("content {i}"));
    }
    assert!(fs.is_dir(Path::new("dir")));
    for i in 0..100 {
        let path_str = format!("dir/file_{i}.txt");
        let expected = format!("content {i}");
        assert_eq!(fs.read_to_string(Path::new(&path_str)).unwrap(), expected);
    }
}
