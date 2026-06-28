//! Byte-mode (`archive-zip`) parity coverage for the WASM FFI byte-mode seam.
//!
//! These tests prove the byte-mode proof obligation named in
//! `docs/specs/wasm-ffi-byte-mode.md`: scanning an admitted ZIP archive through
//! the decode primitive (`tokmd_scan::inputs_from_zip_bytes`) and the existing
//! in-memory workflow yields the same receipt as the equivalent `{ path, text }`
//! JSON-mode input set. The decode reuses the single authoritative admission
//! engine, so there is no second scan path and no second admission path.
//!
//! Out of scope for this slice (and intentionally untested here): a public FFI
//! byte entrypoint, the `tokmd-wasm` `Uint8Array` binding + `wasm-bindgen-test`
//! coverage, and any browser capability promotion.
#![cfg(feature = "archive-zip")]

use std::io::{Cursor, Write};

use tokmd_core::{
    InMemoryFile, lang_workflow_from_inputs,
    settings::{LangSettings, ScanOptions},
};
use tokmd_scan::{ArchiveLimits, inputs_from_zip_bytes};
use tokmd_types::ConfigMode;
use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

type BoxedError = Box<dyn std::error::Error>;

const FIXTURE: &[(&str, &str)] = &[
    ("src/lib.rs", "pub fn alpha() -> usize { 1 }\n"),
    ("src/main.rs", "fn main() {}\n"),
    ("tests/basic.py", "# TODO: keep smoke\nprint('ok')\n"),
];

fn scan_options() -> ScanOptions {
    ScanOptions {
        config: ConfigMode::None,
        ..Default::default()
    }
}

fn lang_settings() -> LangSettings {
    LangSettings {
        files: true,
        ..Default::default()
    }
}

fn fixture_zip() -> Result<Vec<u8>, BoxedError> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, body) in FIXTURE {
        writer.start_file(*name, options)?;
        writer.write_all(body.as_bytes())?;
    }
    Ok(writer.finish()?.into_inner())
}

fn direct_inputs() -> Vec<InMemoryFile> {
    FIXTURE
        .iter()
        .map(|(name, body)| InMemoryFile::new(*name, body.as_bytes().to_vec()))
        .collect()
}

#[test]
fn byte_mode_lang_receipt_matches_json_mode_inputs() -> Result<(), BoxedError> {
    let bytes = fixture_zip()?;
    let decoded = inputs_from_zip_bytes("repo", &bytes, &ArchiveLimits::default())?;
    let direct = direct_inputs();

    let opts = scan_options();
    let lang = lang_settings();

    let from_zip = lang_workflow_from_inputs(&decoded, &opts, &lang)?;
    let from_direct = lang_workflow_from_inputs(&direct, &opts, &lang)?;

    // The aggregated language inventory (rows + totals) is the deterministic
    // contract; compare it directly rather than the receipt envelope, which
    // carries a volatile `generated_at_ms`.
    assert_eq!(
        serde_json::to_value(&from_zip.report)?,
        serde_json::to_value(&from_direct.report)?,
        "byte-mode lang report diverged from JSON-mode lang report"
    );
    assert_eq!(from_zip.report.total.files, FIXTURE.len());
    Ok(())
}

#[test]
fn byte_mode_decode_preserves_logical_paths_and_bytes() -> Result<(), BoxedError> {
    let bytes = fixture_zip()?;
    let decoded = inputs_from_zip_bytes("repo", &bytes, &ArchiveLimits::default())?;

    let mut got: Vec<(String, Vec<u8>)> = decoded
        .iter()
        .map(|input| {
            (
                input.path.to_string_lossy().replace('\\', "/"),
                input.bytes.clone(),
            )
        })
        .collect();
    got.sort_by(|left, right| left.0.cmp(&right.0));

    let mut want: Vec<(String, Vec<u8>)> = FIXTURE
        .iter()
        .map(|(name, body)| ((*name).to_string(), body.as_bytes().to_vec()))
        .collect();
    want.sort_by(|left, right| left.0.cmp(&right.0));

    assert_eq!(got, want);
    Ok(())
}
