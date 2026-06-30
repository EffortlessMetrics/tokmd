//! Receipt-level archive↔host parity oracle (`feature = "archive-zip"`).
//!
//! `docs/specs/repo-snapshot.md` states the archive-ingestion proof obligation
//! as: a benign archive ingested through the archive provider must yield "the
//! same normalized file set and **aggregated receipt** as scanning the
//! equivalent extracted tree through `HostFs`".
//!
//! The existing oracles do not, together, anchor the full tokmd *receipt* to a
//! real host filesystem scan:
//!
//! - `tokmd-scan/tests/archive_scan_parity.rs` proves tokei `Languages`
//!   inventory parity (per-language file/code/comment/blank counts) for
//!   host-vs-ZIP, but stops at the tokei layer — it never exercises the
//!   `tokmd-model` aggregation that computes byte and token totals.
//! - `tokmd-core/tests/archive_zip_bytemode.rs` proves the ZIP-decoded inputs
//!   match the equivalent `{ path, text }` in-memory inputs, but both sides run
//!   the in-memory workflow; neither is a host filesystem scan.
//!
//! This oracle closes that gap. It scans an extracted fixture through the host
//! workflow (`lang_workflow`) and the equivalent ZIP through the archive
//! workflow (`inputs_from_zip_bytes` + `lang_workflow_from_inputs`) and asserts
//! the aggregated `LangReport` — rows plus totals, **including the model-layer
//! `bytes` and `tokens`** — is byte-for-byte identical.
//!
//! Out of scope (intentionally not asserted here): per-file path strings (host
//! paths are temp-rooted, the language-level report is path-independent), the
//! receipt envelope's volatile `generated_at_ms`, and any browser/WASM
//! capability claim.
#![cfg(feature = "archive-zip")]

use std::fs;
use std::io::{Cursor, Write};

use tokmd_core::{
    InMemoryFile, lang_workflow, lang_workflow_from_inputs,
    settings::{LangSettings, ScanOptions, ScanSettings},
};
use tokmd_scan::{ArchiveLimits, inputs_from_zip_bytes};
use tokmd_types::ConfigMode;
use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

type BoxedError = Box<dyn std::error::Error>;

/// Relative path -> contents fixture shared by the host and archive paths.
const FIXTURE: &[(&str, &str)] = &[
    (
        "src/lib.rs",
        "// crate root\npub fn alpha() -> usize { 1 }\n\n",
    ),
    ("src/util.py", "# helper\nprint('ok')\n"),
    ("docs/README.md", "# Title\n\nSome prose.\n"),
];

/// Content-driven scan options: neutralize ambient ignore/config discovery so
/// the host temp root and the archive snapshot are compared purely on their
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

/// Build an in-memory ZIP of the shared fixture, deflate-compressed.
fn fixture_zip() -> Result<Vec<u8>, BoxedError> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, body) in FIXTURE {
        writer.start_file(*name, options)?;
        writer.write_all(body.as_bytes())?;
    }
    Ok(writer.finish()?.into_inner())
}

#[test]
fn archive_lang_report_matches_host_lang_report() -> Result<(), BoxedError> {
    let lang = LangSettings::default();

    // Host path: materialize the fixture on disk and scan via the host
    // workflow (`tokmd_scan::scan` + `tokmd_model::create_lang_report`).
    let dir = tempfile::tempdir()?;
    for (rel, contents) in FIXTURE {
        let full = dir.path().join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full, contents)?;
    }
    let host_scan = ScanSettings {
        paths: vec![dir.path().to_string_lossy().into_owned()],
        options: parity_scan_options(),
    };
    let host = lang_workflow(&host_scan, &lang)?;

    // Archive path: pack the same fixture into a ZIP, admit it fail-closed into
    // the snapshot input set, and run the in-memory workflow
    // (`tokmd_model::create_lang_report_from_rows`).
    let bytes = fixture_zip()?;
    let inputs: Vec<InMemoryFile> =
        inputs_from_zip_bytes("repo", &bytes, &ArchiveLimits::default())?;
    let archive = lang_workflow_from_inputs(&inputs, &parity_scan_options(), &lang)?;

    // Sanity: the fixture really exercised the model layer (bytes + tokens) and
    // more than one language, so the comparison below is not vacuous.
    assert!(
        host.report.rows.len() >= 2,
        "fixture should produce multiple languages: {:?}",
        host.report.rows
    );
    assert!(
        host.report.total.bytes > 0,
        "host report should carry model-layer byte totals"
    );
    assert!(
        host.report.total.tokens > 0,
        "host report should carry model-layer token totals"
    );

    assert_eq!(
        serde_json::to_value(&host.report)?,
        serde_json::to_value(&archive.report)?,
        "archive-backed LangReport diverged from the host LangReport"
    );
    Ok(())
}
