//! Public FFI byte-entrypoint coverage for the WASM FFI byte-mode seam.
//!
//! These tests prove the public-entrypoint slice of
//! `docs/specs/wasm-ffi-byte-mode.md`: the `tokmd_core::ffi::run_json_bytes`
//! transport entrypoint admits ZIP bytes through the single authoritative
//! engine and routes them through the existing mode dispatch, so the byte-mode
//! envelope matches the equivalent `{ path, text }` JSON-mode envelope and
//! hostile/malformed archives fail closed with no partial receipt.
//!
//! Out of scope for this slice (intentionally untested here): the `tokmd-wasm`
//! `Uint8Array` binding + `wasm-bindgen-test` coverage, and any browser
//! capability promotion.
#![cfg(feature = "archive-zip")]

use std::io::{Cursor, Write};

use serde_json::{Value, json};
use tokmd_core::ffi::{run_json, run_json_bytes};
use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

type BoxedError = Box<dyn std::error::Error>;

/// Fixture entries in normalized-sorted path order, matching the order the ZIP
/// admission engine yields so the JSON-mode parity inputs line up exactly.
const FIXTURE: &[(&str, &str)] = &[
    ("src/lib.rs", "pub fn alpha() -> usize { 1 }\n"),
    ("src/main.rs", "fn main() {}\n"),
    ("tests/basic.py", "# TODO: keep smoke\nprint('ok')\n"),
];

fn fixture_zip() -> Result<Vec<u8>, BoxedError> {
    build_zip(|writer| {
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (name, body) in FIXTURE {
            writer.start_file(*name, options)?;
            writer.write_all(body.as_bytes())?;
        }
        Ok(())
    })
}

fn build_zip(
    build: impl FnOnce(&mut ZipWriter<Cursor<Vec<u8>>>) -> zip::result::ZipResult<()>,
) -> Result<Vec<u8>, BoxedError> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    build(&mut writer)?;
    Ok(writer.finish()?.into_inner())
}

/// Equivalent JSON-mode options carrying the same logical file set as inline
/// `{ path, text }` inputs, in the same normalized order.
fn json_mode_options() -> Value {
    let inputs: Vec<Value> = FIXTURE
        .iter()
        .map(|(name, body)| json!({ "path": name, "text": body }))
        .collect();
    json!({ "lang": { "files": true }, "inputs": inputs })
}

fn byte_mode_lang_options() -> String {
    json!({ "lang": { "files": true } }).to_string()
}

fn json_mode_module_options() -> Value {
    let inputs: Vec<Value> = FIXTURE
        .iter()
        .map(|(name, body)| json!({ "path": name, "text": body }))
        .collect();
    json!({ "module": {}, "inputs": inputs })
}

fn byte_mode_module_options() -> String {
    json!({ "module": {} }).to_string()
}

fn json_mode_export_options() -> Value {
    let inputs: Vec<Value> = FIXTURE
        .iter()
        .map(|(name, body)| json!({ "path": name, "text": body }))
        .collect();
    json!({ "export": { "format": "json" }, "inputs": inputs })
}

fn byte_mode_export_options() -> String {
    json!({ "export": { "format": "json" } }).to_string()
}

fn parse_envelope(raw: &str) -> Result<Value, BoxedError> {
    Ok(serde_json::from_str(raw)?)
}

/// Strip the volatile `generated_at_ms` field so two otherwise-identical
/// receipts compare equal.
fn scrub_timestamp(envelope: &mut Value) {
    if let Some(data) = envelope.get_mut("data")
        && let Some(obj) = data.as_object_mut()
    {
        obj.remove("generated_at_ms");
    }
}

#[test]
fn byte_mode_lang_envelope_matches_json_mode_inputs() -> Result<(), BoxedError> {
    let bytes = fixture_zip()?;

    let mut from_zip = parse_envelope(&run_json_bytes("lang", &byte_mode_lang_options(), &bytes))?;
    let mut from_json = parse_envelope(&run_json("lang", &json_mode_options().to_string()))?;

    assert_eq!(from_zip["ok"], json!(true), "byte-mode call should succeed");
    assert_eq!(
        from_json["ok"],
        json!(true),
        "json-mode call should succeed"
    );
    assert_eq!(from_zip["data"]["mode"], json!("lang"));
    // `LangReceipt.report` is `#[serde(flatten)]`, so its `total` sits directly
    // on `data`.
    assert_eq!(from_zip["data"]["total"]["files"], json!(3));

    scrub_timestamp(&mut from_zip);
    scrub_timestamp(&mut from_json);
    assert_eq!(
        from_zip, from_json,
        "byte-mode envelope diverged from the equivalent JSON-mode envelope"
    );
    Ok(())
}

#[test]
fn byte_mode_module_envelope_matches_json_mode_inputs() -> Result<(), BoxedError> {
    let bytes = fixture_zip()?;

    let mut from_zip = parse_envelope(&run_json_bytes(
        "module",
        &byte_mode_module_options(),
        &bytes,
    ))?;
    let mut from_json =
        parse_envelope(&run_json("module", &json_mode_module_options().to_string()))?;

    assert_eq!(from_zip["ok"], json!(true), "byte-mode call should succeed");
    assert_eq!(
        from_json["ok"],
        json!(true),
        "json-mode call should succeed"
    );
    assert_eq!(from_zip["data"]["mode"], json!("module"));
    assert_eq!(from_zip["data"]["total"]["files"], json!(3));

    scrub_timestamp(&mut from_zip);
    scrub_timestamp(&mut from_json);
    assert_eq!(
        from_zip, from_json,
        "byte-mode module envelope diverged from the equivalent JSON-mode envelope"
    );
    Ok(())
}

#[test]
fn byte_mode_export_envelope_matches_json_mode_inputs() -> Result<(), BoxedError> {
    let bytes = fixture_zip()?;

    let mut from_zip = parse_envelope(&run_json_bytes(
        "export",
        &byte_mode_export_options(),
        &bytes,
    ))?;
    let mut from_json =
        parse_envelope(&run_json("export", &json_mode_export_options().to_string()))?;

    assert_eq!(from_zip["ok"], json!(true), "byte-mode call should succeed");
    assert_eq!(
        from_json["ok"],
        json!(true),
        "json-mode call should succeed"
    );
    assert_eq!(from_zip["data"]["mode"], json!("export"));
    assert_eq!(
        from_zip["data"]["rows"].as_array().map(Vec::len),
        Some(3),
        "export receipt should list one row per fixture file"
    );

    scrub_timestamp(&mut from_zip);
    scrub_timestamp(&mut from_json);
    assert_eq!(
        from_zip, from_json,
        "byte-mode export envelope diverged from the equivalent JSON-mode envelope"
    );
    Ok(())
}

#[test]
fn byte_mode_rejects_inline_inputs_option() -> Result<(), BoxedError> {
    let bytes = fixture_zip()?;
    let options = json!({ "inputs": [{ "path": "a.rs", "text": "fn a() {}" }] }).to_string();

    let envelope = parse_envelope(&run_json_bytes("lang", &options, &bytes))?;
    assert_eq!(envelope["ok"], json!(false));
    assert_eq!(envelope["error"]["code"], json!("invalid_settings"));
    Ok(())
}

#[test]
fn byte_mode_rejects_paths_option() -> Result<(), BoxedError> {
    let bytes = fixture_zip()?;
    let options = json!({ "paths": ["."] }).to_string();

    let envelope = parse_envelope(&run_json_bytes("lang", &options, &bytes))?;
    assert_eq!(envelope["ok"], json!(false));
    assert_eq!(envelope["error"]["code"], json!("invalid_settings"));
    Ok(())
}

#[test]
fn byte_mode_rejects_host_only_mode() -> Result<(), BoxedError> {
    let bytes = fixture_zip()?;

    let envelope = parse_envelope(&run_json_bytes("diff", &byte_mode_lang_options(), &bytes))?;
    assert_eq!(envelope["ok"], json!(false));
    assert_eq!(envelope["error"]["code"], json!("invalid_settings"));
    Ok(())
}

#[test]
fn byte_mode_fails_closed_on_hostile_entry() -> Result<(), BoxedError> {
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let bytes = build_zip(|writer| {
        writer.start_file("ok.rs", stored)?;
        writer.write_all(b"fn ok() {}\n")?;
        writer.start_file("nested/../../evil.rs", stored)?;
        writer.write_all(b"pwn")?;
        Ok(())
    })?;

    let envelope = parse_envelope(&run_json_bytes("lang", &byte_mode_lang_options(), &bytes))?;
    assert_eq!(
        envelope["ok"],
        json!(false),
        "a traversal entry must fail closed with no partial receipt"
    );
    assert!(
        envelope.get("data").is_none() || envelope["data"].is_null(),
        "a rejected archive must not produce a receipt"
    );
    Ok(())
}

#[test]
fn byte_mode_rejects_malformed_archive() -> Result<(), BoxedError> {
    let envelope = parse_envelope(&run_json_bytes(
        "lang",
        &byte_mode_lang_options(),
        b"not a zip",
    ))?;
    assert_eq!(envelope["ok"], json!(false));
    Ok(())
}

#[test]
fn byte_mode_enforces_archive_limits() -> Result<(), BoxedError> {
    let bytes = fixture_zip()?;
    // The fixture has three entries; a one-entry cap must fail closed.
    let options =
        json!({ "lang": { "files": true }, "archive_limits": { "max_entries": 1 } }).to_string();

    let envelope = parse_envelope(&run_json_bytes("lang", &options, &bytes))?;
    assert_eq!(
        envelope["ok"],
        json!(false),
        "exceeding the entry-count cap must fail closed"
    );
    Ok(())
}
