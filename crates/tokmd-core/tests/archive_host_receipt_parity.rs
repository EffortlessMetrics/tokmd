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
//! This oracle closes that gap for `lang`, `module`, and `export` receipts. It
//! scans an extracted fixture through the host workflow and the equivalent ZIP
//! through the archive workflow (`inputs_from_zip_bytes` + `*_workflow_from_inputs`)
//! and asserts the aggregated reports are byte-for-byte identical. Host `module`
//! scans strip a single scan root automatically; host `export` scans use
//! `strip_prefix` equal to the materialized fixture root so paths align with
//! archive-relative virtual paths.
//!
//! Out of scope (intentionally not asserted here): per-file path strings (host
//! paths are temp-rooted, the language-level report is path-independent), the
//! receipt envelope's volatile `generated_at_ms`, and any browser/WASM
//! capability claim.
#![cfg(feature = "archive-zip")]

use std::fs;
use std::io::{Cursor, Write};

use tokmd_core::{
    InMemoryFile, export_workflow, export_workflow_from_inputs, lang_workflow,
    lang_workflow_from_inputs, module_workflow, module_workflow_from_inputs,
    settings::{ExportSettings, LangSettings, ModuleSettings, ScanOptions, ScanSettings},
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
    let (_dir, host_scan) = materialize_fixture_host_scan()?;
    let host = lang_workflow(&host_scan, &lang)?;
    let archive = lang_workflow_from_inputs(&archive_inputs()?, &parity_scan_options(), &lang)?;

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

#[test]
fn archive_module_report_matches_host_module_report() -> Result<(), BoxedError> {
    let module = ModuleSettings::default();
    let (_dir, host_scan) = materialize_fixture_host_scan()?;
    let host = module_workflow(&host_scan, &module)?;

    let inputs = archive_inputs()?;
    let archive = module_workflow_from_inputs(&inputs, &parity_scan_options(), &module)?;

    assert!(
        host.report.rows.len() >= 2,
        "fixture should produce multiple modules: {:?}",
        host.report.rows
    );
    assert!(
        host.report.total.bytes > 0,
        "host report should carry model-layer byte totals"
    );

    assert_eq!(
        serde_json::to_value(&host.report)?,
        serde_json::to_value(&archive.report)?,
        "archive-backed ModuleReport diverged from the host ModuleReport"
    );
    Ok(())
}

#[test]
fn archive_export_report_matches_host_export_report() -> Result<(), BoxedError> {
    let (_dir, host_scan) = materialize_fixture_host_scan()?;
    let strip = _dir.path().to_string_lossy().into_owned();
    let export = ExportSettings {
        strip_prefix: Some(strip),
        ..Default::default()
    };
    let host = export_workflow(&host_scan, &export)?;

    let inputs = archive_inputs()?;
    let archive = export_workflow_from_inputs(&inputs, &parity_scan_options(), &export)?;

    assert!(
        host.data.rows.len() >= 2,
        "fixture should produce multiple file rows: {:?}",
        host.data.rows
    );
    assert!(
        host.data.rows.iter().any(|row| row.bytes > 0),
        "host export rows should carry model-layer byte totals"
    );

    assert_eq!(
        serde_json::to_value(&host.data)?,
        serde_json::to_value(&archive.data)?,
        "archive-backed ExportData diverged from the host ExportData"
    );
    Ok(())
}

fn materialize_fixture_host_scan() -> Result<(tempfile::TempDir, ScanSettings), BoxedError> {
    let dir = tempfile::tempdir()?;
    for (rel, contents) in FIXTURE {
        let full = dir.path().join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full, contents)?;
    }
    let scan = ScanSettings {
        paths: vec![dir.path().to_string_lossy().into_owned()],
        options: parity_scan_options(),
    };
    Ok((dir, scan))
}

fn archive_inputs() -> Result<Vec<InMemoryFile>, BoxedError> {
    let bytes = fixture_zip()?;
    Ok(inputs_from_zip_bytes(
        "repo",
        &bytes,
        &ArchiveLimits::default(),
    )?)
}
