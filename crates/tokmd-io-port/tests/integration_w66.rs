//! Integration tests: filesystem snapshots and bulk operations.

use std::collections::BTreeMap;
use std::path::Path;
use tokmd_io_port::{MemFs, ReadFs};

fn project_snapshot() -> MemFs {
    let mut fs = MemFs::new();
    fs.add_file("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"");
    fs.add_file("src/lib.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }");
    fs.add_file("src/main.rs", "fn main() { println!(\"{}\", demo::add(1, 2)); }");
    fs.add_file("tests/test_add.rs", "#[test] fn it_works() { assert_eq!(demo::add(2, 2), 4); }");
    fs.add_file("README.md", "# Demo\nA sample project.");
    fs.add_file(".gitignore", "/target\n");
    fs.add_bytes("assets/icon.png", vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    fs
}

#[test]
fn snapshot_all_files_readable() {
    let fs = project_snapshot();
    let expected_files = [
        "Cargo.toml", "src/lib.rs", "src/main.rs", "tests/test_add.rs",
        "README.md", ".gitignore", "assets/icon.png",
    ];
    for path in expected_files {
        assert!(fs.exists(Path::new(path)), "expected {path} to exist");
        assert!(fs.is_file(Path::new(path)), "expected {path} to be a file");
        assert!(fs.read_bytes(Path::new(path)).is_ok(), "expected {path} to be readable");
    }
}

#[test]
fn snapshot_directories_inferred() {
    let fs = project_snapshot();
    let expected_dirs = ["src", "tests", "assets"];
    for dir in expected_dirs {
        assert!(fs.is_dir(Path::new(dir)), "expected {dir} to be a directory");
    }
}

#[test]
fn snapshot_text_files_are_valid_utf8() {
    let fs = project_snapshot();
    let text_files = ["Cargo.toml", "src/lib.rs", "src/main.rs", "tests/test_add.rs", "README.md", ".gitignore"];
    for path in text_files {
        assert!(fs.read_to_string(Path::new(path)).is_ok(), "expected {path} to be valid UTF-8");
    }
}

#[test]
fn snapshot_binary_file_is_not_utf8() {
    let fs = project_snapshot();
    assert!(fs.read_to_string(Path::new("assets/icon.png")).is_err());
}

#[test]
fn bulk_insert_from_btreemap() {
    let mut entries = BTreeMap::new();
    entries.insert("a/1.txt", "one");
    entries.insert("a/2.txt", "two");
    entries.insert("b/3.txt", "three");
    entries.insert("c.txt", "four");

    let mut fs = MemFs::new();
    for (path, content) in &entries {
        fs.add_file(*path, *content);
    }
    for (path, content) in &entries {
        assert_eq!(fs.read_to_string(Path::new(path)).unwrap(), *content, "mismatch for {path}");
    }
}

#[test]
fn bulk_overwrite_cycle() {
    let mut fs = MemFs::new();
    let path = "data/file.txt";
    for i in 0..50 {
        let content = format!("version {i}");
        fs.add_file(path, &content);
        assert_eq!(fs.read_to_string(Path::new(path)).unwrap(), content);
    }
}

#[test]
fn bulk_many_directories() {
    let mut fs = MemFs::new();
    for i in 0..50 {
        let path = format!("modules/mod_{i}/lib.rs");
        fs.add_file(&path, format!("mod {i}"));
    }
    assert!(fs.is_dir(Path::new("modules")));
    for i in 0..50 {
        let dir = format!("modules/mod_{i}");
        let file = format!("modules/mod_{i}/lib.rs");
        assert!(fs.is_dir(Path::new(&dir)), "dir {dir} should exist");
        assert!(fs.is_file(Path::new(&file)), "file {file} should exist");
    }
}

#[test]
fn clone_is_independent() {
    let mut fs = project_snapshot();
    let fs2 = fs.clone();
    fs.add_file("src/lib.rs", "MODIFIED");
    assert_eq!(fs2.read_to_string(Path::new("src/lib.rs")).unwrap(), "pub fn add(a: i32, b: i32) -> i32 { a + b }");
    assert_eq!(fs.read_to_string(Path::new("src/lib.rs")).unwrap(), "MODIFIED");
}

#[test]
fn mixed_text_and_binary_same_directory() {
    let mut fs = MemFs::new();
    fs.add_file("data/config.json", "{\"key\": \"value\"}");
    fs.add_bytes("data/blob.dat", vec![0x00; 256]);
    assert!(fs.is_dir(Path::new("data")));
    assert_eq!(fs.read_to_string(Path::new("data/config.json")).unwrap(), "{\"key\": \"value\"}");
    assert_eq!(fs.read_bytes(Path::new("data/blob.dat")).unwrap().len(), 256);
}

#[test]
fn root_level_files_no_directories() {
    let mut fs = MemFs::new();
    fs.add_file("a.txt", "a");
    fs.add_file("b.txt", "b");
    assert!(fs.is_file(Path::new("a.txt")));
    assert!(fs.is_file(Path::new("b.txt")));
    assert!(!fs.is_dir(Path::new("a.txt")));
    assert!(!fs.is_dir(Path::new("b.txt")));
}

#[test]
fn memfs_debug_impl() {
    let fs = MemFs::new();
    let debug = format!("{:?}", fs);
    assert!(debug.contains("MemFs"));
}

#[test]
fn error_display_and_error_trait() {
    let fs = MemFs::new();
    let err = fs.read_to_string(Path::new("missing")).unwrap_err();
    let display = format!("{err}");
    assert!(!display.is_empty());
    let _: &dyn std::error::Error = &err;
}