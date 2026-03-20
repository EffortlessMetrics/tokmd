use anyhow::Result;
use std::path::PathBuf;

use tokmd_scan::{InMemoryFile, scan_in_memory};
use tokmd_settings::{ConfigMode, ScanOptions};

fn default_scan_options() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: ConfigMode::None,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

#[test]
fn scan_in_memory_preserves_logical_paths_and_materializes_files() -> Result<()> {
    let inputs = vec![
        InMemoryFile::new("src/lib.rs", "pub fn alpha() -> usize { 1 }\n"),
        InMemoryFile::new("tests/basic.py", "print('ok')\n"),
    ];

    let scan = scan_in_memory(&inputs, &default_scan_options())?;
    assert!(scan.languages().get(&tokei::LanguageType::Rust).is_some());
    assert!(scan.languages().get(&tokei::LanguageType::Python).is_some());
    assert_eq!(
        scan.logical_paths(),
        &[PathBuf::from("src/lib.rs"), PathBuf::from("tests/basic.py")]
    );
    assert!(scan.logical_paths().iter().all(|path| path.is_relative()));
    assert!(scan.strip_prefix().join("src/lib.rs").exists());
    assert!(scan.strip_prefix().join("tests/basic.py").exists());

    Ok(())
}

#[test]
fn scan_in_memory_rejects_escaping_paths() {
    let inputs = vec![InMemoryFile::new("../escape.rs", "fn nope() {}\n")];
    let err = scan_in_memory(&inputs, &default_scan_options()).unwrap_err();
    assert!(err.to_string().contains("parent traversal"));
}

#[test]
fn scan_in_memory_rejects_duplicate_logical_paths() {
    let inputs = vec![
        InMemoryFile::new("./src/lib.rs", "fn alpha() {}\n"),
        InMemoryFile::new("src/lib.rs", "fn beta() {}\n"),
    ];
    let err = scan_in_memory(&inputs, &default_scan_options()).unwrap_err();
    assert!(err.to_string().contains("Duplicate in-memory path"));
}

#[test]
fn scan_in_memory_keeps_backing_root_alive_for_metadata_reads() -> Result<()> {
    let scan = scan_in_memory(
        &[InMemoryFile::new("src/lib.rs", "pub fn alpha() {}\n")],
        &default_scan_options(),
    )?;
    let backing_file = scan.strip_prefix().join("src/lib.rs");

    assert!(backing_file.exists());
    assert!(std::fs::metadata(backing_file)?.len() > 0);

    Ok(())
}
