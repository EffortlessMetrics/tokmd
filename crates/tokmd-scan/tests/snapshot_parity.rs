//! Parity oracle for the repo-snapshot scan consumer.
//!
//! Scanning a fixture through the host filesystem (`scan`) and through a
//! captured `RepoSnapshot` (`scan_snapshot`) must yield the same per-language
//! inventory: identical report (file) counts and identical code/comment/blank
//! totals. This is the snapshot-backed slice of the host/in-memory parity
//! contract described in `docs/specs/repo-snapshot.md`.

use anyhow::Result;
use std::fs;

use tokmd_io_port::{MemFs, RepoSnapshot};
use tokmd_scan::{scan, scan_snapshot};
use tokmd_settings::{ConfigMode, ScanOptions};

/// Relative path -> contents fixture shared by both scan paths.
const FIXTURE: &[(&str, &str)] = &[
    (
        "src/lib.rs",
        "// crate root\npub fn alpha() -> usize { 1 }\n\n",
    ),
    ("src/util.py", "# helper\nprint('ok')\n"),
    ("docs/README.md", "# Title\n\nSome prose.\n"),
];

/// Content-driven scan options: neutralize ambient ignore files so the two
/// distinct temp roots are compared purely on their captured contents.
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

#[test]
fn host_and_snapshot_scans_produce_equivalent_inventory() -> Result<()> {
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

    // Snapshot path: capture the same fixture through MemFs into a RepoSnapshot
    // and scan via the snapshot consumer.
    let mut mem = MemFs::new();
    for (rel, contents) in FIXTURE {
        mem.add_file(*rel, *contents);
    }
    let mut builder = RepoSnapshot::builder(&mem, ".");
    builder
        .add_paths(FIXTURE.iter().map(|(rel, _)| *rel))
        .map_err(|err| anyhow::anyhow!("snapshot capture failed: {err}"))?;
    let snapshot = builder.build();
    let snapshot_scan = scan_snapshot(&snapshot, &opts)?;

    let host_summary = summarize(&host_languages);
    let snapshot_summary = summarize(snapshot_scan.languages());

    // Sanity: the fixture really exercised more than one language.
    assert!(
        host_summary.len() >= 2,
        "fixture should produce multiple languages: {host_summary:?}"
    );
    assert_eq!(
        host_summary, snapshot_summary,
        "snapshot scan inventory must match the host scan inventory"
    );

    Ok(())
}
