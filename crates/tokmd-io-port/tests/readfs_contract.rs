//! Contract tests for the `ReadFs` trait.
//!
//! Any correct `ReadFs` implementation must pass these tests.
//! We verify with `MemFs` as the concrete backend.

use std::path::Path;
use tokmd_io_port::{MemFs, ReadFs};

/// Build a populated `MemFs` used by most contract tests.
fn sample_fs() -> MemFs {
    let mut fs = MemFs::new();
    fs.add_file("src/lib.rs", "pub fn hello() {}");
    fs.add_file("src/main.rs", "fn main() {}");
    fs.add_file("README.md", "# Hello");
    fs.add_bytes("assets/logo.png", vec![0x89, 0x50, 0x4E, 0x47]);
    fs
}

#[test]
fn contract_read_to_string_exact() {
    let fs = sample_fs();
    assert_eq!(
        fs.read_to_string(Path::new("src/lib.rs")).unwrap(),
        "pub fn hello() {}"
    );
    assert_eq!(
        fs.read_to_string(Path::new("README.md")).unwrap(),
        "# Hello"
    );
}

#[test]
fn contract_read_bytes_exact() {
    let fs = sample_fs();
    assert_eq!(
        fs.read_bytes(Path::new("assets/logo.png")).unwrap(),
        vec![0x89, 0x50, 0x4E, 0x47]
    );
}

#[test]
fn contract_read_bytes_matches_read_to_string_for_utf8() {
    let fs = sample_fs();
    let as_string = fs.read_to_string(Path::new("src/main.rs")).unwrap();
    let as_bytes = fs.read_bytes(Path::new("src/main.rs")).unwrap();
    assert_eq!(as_bytes, as_string.as_bytes());
}

#[test]
fn contract_read_to_string_errors_on_binary() {
    let fs = sample_fs();
    let err = fs.read_to_string(Path::new("assets/logo.png")).unwrap_err();
    assert!(err.to_string().contains("invalid UTF-8"));
}

#[test]
fn contract_read_to_string_missing() {
    let fs = sample_fs();
    assert!(fs.read_to_string(Path::new("nope.txt")).is_err());
}

#[test]
fn contract_read_bytes_missing() {
    let fs = sample_fs();
    assert!(fs.read_bytes(Path::new("nope.txt")).is_err());
}

#[test]
fn contract_exists_file() {
    let fs = sample_fs();
    assert!(fs.exists(Path::new("src/lib.rs")));
}

#[test]
fn contract_exists_directory() {
    let fs = sample_fs();
    assert!(fs.exists(Path::new("src")));
}

#[test]
fn contract_not_exists() {
    let fs = sample_fs();
    assert!(!fs.exists(Path::new("not/here")));
}

#[test]
fn contract_is_file_true() {
    let fs = sample_fs();
    assert!(fs.is_file(Path::new("src/lib.rs")));
}

#[test]
fn contract_is_file_false_for_directory() {
    let fs = sample_fs();
    assert!(!fs.is_file(Path::new("src")));
}

#[test]
fn contract_is_file_false_for_missing() {
    let fs = sample_fs();
    assert!(!fs.is_file(Path::new("missing.rs")));
}

#[test]
fn contract_is_dir_true() {
    let fs = sample_fs();
    assert!(fs.is_dir(Path::new("src")));
    assert!(fs.is_dir(Path::new("assets")));
}

#[test]
fn contract_is_dir_false_for_file() {
    let fs = sample_fs();
    assert!(!fs.is_dir(Path::new("README.md")));
}

#[test]
fn contract_is_dir_false_for_missing() {
    let fs = sample_fs();
    assert!(!fs.is_dir(Path::new("nonexistent")));
}

#[test]
fn contract_exists_equals_file_or_dir() {
    let fs = sample_fs();
    let paths = [
        "src/lib.rs",
        "src/main.rs",
        "README.md",
        "assets/logo.png",
        "src",
        "assets",
        "nope",
        "also/nope",
    ];
    for p in paths {
        let path = Path::new(p);
        assert_eq!(
            fs.exists(path),
            fs.is_file(path) || fs.is_dir(path),
            "exists consistency failed for {p}"
        );
    }
}

#[test]
fn contract_file_and_dir_mutually_exclusive() {
    let fs = sample_fs();
    let paths = [
        "src/lib.rs",
        "src",
        "README.md",
        "assets",
        "assets/logo.png",
        "nothing",
    ];
    for p in paths {
        let path = Path::new(p);
        assert!(
            !(fs.is_file(path) && fs.is_dir(path)),
            "is_file and is_dir both true for {p}"
        );
    }
}

#[test]
fn contract_error_contains_path() {
    let fs = MemFs::new();
    let err = fs.read_to_string(Path::new("some/path.txt")).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("some") || msg.contains("path.txt"),
        "error should mention the path: {msg}"
    );
}

#[test]
fn contract_overwrite_text_with_binary() {
    let mut fs = MemFs::new();
    fs.add_file("data", "text content");
    fs.add_bytes("data", vec![0x00, 0xFF]);
    assert!(fs.read_to_string(Path::new("data")).is_err());
    assert_eq!(fs.read_bytes(Path::new("data")).unwrap(), vec![0x00, 0xFF]);
}

#[test]
fn contract_overwrite_binary_with_text() {
    let mut fs = MemFs::new();
    fs.add_bytes("data", vec![0x00, 0xFF]);
    fs.add_file("data", "now text");
    assert_eq!(fs.read_to_string(Path::new("data")).unwrap(), "now text");
}
