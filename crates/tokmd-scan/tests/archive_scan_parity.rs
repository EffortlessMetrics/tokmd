//! Parity oracle for the archive-backed scan consumer (`feature = "archive-zip"`).
//!
//! Scanning a fixture through the host filesystem (`scan`) and through an
//! in-memory ZIP archive (`scan_snapshot_from_zip`) must yield the same
//! per-language inventory: identical report (file) counts and identical
//! code/comment/blank totals. This is the archive-upload slice of the
//! host/snapshot parity contract described in `docs/specs/repo-snapshot.md`:
//! `ZIP bytes -> RepoSnapshot -> scan aggregation` is indistinguishable from a
//! host-extracted run for in-scope files, and a hostile entry fails closed.
#![cfg(feature = "archive-zip")]

use anyhow::Result;
use std::fs;
use std::io::{Cursor, Write};

use tokmd_scan::{ArchiveError, ArchiveLimits, scan, scan_snapshot_from_zip};
use tokmd_settings::{ConfigMode, ScanOptions};
use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

/// Relative path -> contents fixture shared by both scan paths.
const FIXTURE: &[(&str, &str)] = &[
    (
        "src/lib.rs",
        "// crate root\npub fn alpha() -> usize { 1 }\n\n",
    ),
    ("src/util.py", "# helper\nprint('ok')\n"),
    ("docs/README.md", "# Title\n\nSome prose.\n"),
];

/// Content-driven scan options: neutralize ambient ignore files so the host
/// temp root and the materialized snapshot root are compared purely on their
/// captured contents.
fn parity_scan_options() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: ConfigMode::None,
        hidden: false,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: false,
    }
}

/// Summarize a scan into a deterministic, path-prefix-independent inventory:
/// `(language name, file count, code, comments, blanks)` sorted by language.
fn summarize(languages: &tokei::Languages) -> Vec<(String, usize, usize, usize, usize)> {
    let mut rows: Vec<(String, usize, usize, usize, usize)> = languages
        .iter()
        .map(|(lang_type, language)| {
            (
                lang_type.name().to_string(),
                language.reports.len(),
                language.code,
                language.comments,
                language.blanks,
            )
        })
        .collect();
    rows.sort();
    rows
}

/// Build an in-memory ZIP of the shared fixture, deflate-compressed.
fn fixture_zip() -> Result<Vec<u8>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (rel, contents) in FIXTURE {
        writer.start_file(*rel, options)?;
        writer.write_all(contents.as_bytes())?;
    }
    Ok(writer.finish()?.into_inner())
}

#[test]
fn host_and_zip_scans_produce_equivalent_inventory() -> Result<()> {
    let opts = parity_scan_options();

    // Host path: materialize the fixture on disk and scan via std::fs/tokei.
    let dir = tempfile::tempdir()?;
    for (rel, contents) in FIXTURE {
        let full = dir.path().join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full, contents)?;
    }
    let host_languages = scan(&[dir.path().to_path_buf()], &opts)?;

    // Archive path: pack the same fixture into a ZIP and scan via the codec
    // consumer (ZIP bytes -> admitted RepoSnapshot -> scan aggregation).
    let zip_bytes = fixture_zip()?;
    let zip_scan = scan_snapshot_from_zip("repo", &zip_bytes, &ArchiveLimits::default(), &opts)?;

    let host_summary = summarize(&host_languages);
    let zip_summary = summarize(zip_scan.languages());

    // Sanity: the fixture really exercised more than one language.
    assert!(
        host_summary.len() >= 2,
        "fixture should produce multiple languages: {host_summary:?}"
    );
    assert_eq!(
        host_summary, zip_summary,
        "ZIP-backed scan inventory must match the host scan inventory"
    );

    Ok(())
}

#[test]
fn hostile_zip_entry_fails_closed() -> Result<()> {
    // A benign entry followed by a traversal entry: admission must fail the
    // whole build rather than scanning a partial, misleadingly "complete" view.
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    writer.start_file("src/ok.rs", options)?;
    writer.write_all(b"pub fn ok() {}\n")?;
    writer.start_file("nested/../../evil.rs", options)?;
    writer.write_all(b"pub fn evil() {}\n")?;
    let zip_bytes = writer.finish()?.into_inner();

    let result = scan_snapshot_from_zip(
        "repo",
        &zip_bytes,
        &ArchiveLimits::default(),
        &parity_scan_options(),
    );

    match result {
        Ok(scan) => anyhow::bail!(
            "hostile traversal entry must fail the snapshot build, but scan admitted {} files",
            scan.languages().iter().count()
        ),
        Err(err) => match err.downcast_ref::<ArchiveError>() {
            Some(ArchiveError::Traversal { .. }) => Ok(()),
            other => anyhow::bail!("expected traversal rejection, got {other:?}"),
        },
    }
}
