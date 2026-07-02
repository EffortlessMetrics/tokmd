//! wasm-bindgen bindings for tokmd.
//!
//! This crate intentionally stays thin: it reuses `tokmd_core::ffi::run_json`
//! plus the shared envelope helpers so the browser surface matches the other
//! binding products instead of reimplementing parsing and validation.

#![forbid(unsafe_code)]

#[cfg(feature = "archive-zip")]
use js_sys::Uint8Array;
use js_sys::{Error as JsError, JSON};
use wasm_bindgen::prelude::*;

#[cfg(test)]
use serde_json::Value;
#[cfg(feature = "analysis")]
use tokmd_core::CORE_ANALYSIS_SCHEMA_VERSION;
use tokmd_core::error::{ResponseEnvelope, TokmdError};

#[cfg(feature = "analysis")]
const ROOTLESS_ANALYZE_PRESETS: &[&str] = &["receipt", "estimate"];

fn to_js_error(message: impl Into<String>) -> JsValue {
    JsError::new(&message.into()).into()
}

#[cfg(test)]
fn serialize_args(args: &Value) -> Result<String, String> {
    serde_json::to_string(args).map_err(|err| format!("JSON encode error: {err}"))
}

fn extract_mode_data_json(mode: &str, args_json: &str) -> Result<String, String> {
    validate_mode_args_json(mode, args_json).map_err(|err| err.to_string())?;
    extract_mode_data_json_after_validation(mode, args_json)
}

fn extract_mode_data_json_after_validation(mode: &str, args_json: &str) -> Result<String, String> {
    let result_json = tokmd_core::ffi::run_json(mode, args_json);
    tokmd_envelope::ffi::extract_data_json(&result_json).map_err(|err| err.to_string())
}

#[cfg(test)]
fn run_mode_value(mode: &str, args: &Value) -> Result<Value, String> {
    let args_json = serialize_args(args)?;
    let data_json = extract_mode_data_json(mode, &args_json)?;
    serde_json::from_str(&data_json).map_err(|err| format!("JSON decode error: {err}"))
}

fn js_args_to_json(args: JsValue) -> Result<String, JsValue> {
    if args.is_null() || args.is_undefined() {
        return Ok("{}".to_string());
    }

    if let Some(raw_json) = args.as_string() {
        return normalize_raw_json_args(&raw_json).map_err(to_js_error);
    }

    JSON::stringify(&args)
        .map_err(|_| to_js_error("failed to serialize JS arguments"))?
        .as_string()
        .ok_or_else(|| to_js_error("failed to serialize JS arguments"))
}

fn normalize_raw_json_args(raw_json: &str) -> Result<String, String> {
    serde_json::from_str::<serde_json::Value>(raw_json)
        .map_err(|err| format!("failed to parse JSON string arguments: {err}"))?;
    Ok(raw_json.to_string())
}

fn run_mode_js(mode: &str, args: JsValue) -> Result<JsValue, JsValue> {
    let args_json = js_args_to_json(args)?;
    let data_json = extract_mode_data_json(mode, &args_json).map_err(to_js_error)?;
    JSON::parse(&data_json).map_err(|_| to_js_error("failed to parse tokmd result JSON"))
}

#[cfg(feature = "analysis")]
fn validate_analyze_args_json(args_json: &str) -> Result<(), TokmdError> {
    let args: serde_json::Value =
        serde_json::from_str(args_json).map_err(TokmdError::invalid_json)?;
    let obj = args.get("analyze").unwrap_or(&args);

    match obj.get("preset").and_then(serde_json::Value::as_str) {
        Some(preset) if tokmd_core::supports_rootless_in_memory_analyze_preset(preset) => Ok(()),
        Some(preset) => Err(TokmdError::not_implemented(format!(
            "tokmd-wasm currently supports analyze only with preset=\"receipt\" or preset=\"estimate\" for in-memory inputs; got {preset:?}"
        ))),
        None => Ok(()),
    }
}

fn validate_mode_args_json(mode: &str, args_json: &str) -> Result<(), TokmdError> {
    #[cfg(feature = "analysis")]
    if mode == "analyze" {
        return validate_analyze_args_json(args_json);
    }

    let _ = (mode, args_json);
    Ok(())
}

#[cfg(feature = "analysis")]
fn run_analyze_js(args: JsValue) -> Result<JsValue, JsValue> {
    let args_json = js_args_to_json(args)?;
    validate_analyze_args_json(&args_json).map_err(|err| to_js_error(err.to_string()))?;
    let data_json =
        extract_mode_data_json_after_validation("analyze", &args_json).map_err(to_js_error)?;
    JSON::parse(&data_json).map_err(|_| to_js_error("failed to parse tokmd result JSON"))
}

/// Return the tokmd package version.
#[wasm_bindgen]
pub fn version() -> String {
    tokmd_core::ffi::version().to_string()
}

/// Return the current core receipt schema version for `lang`, `module`, and `export`.
#[wasm_bindgen(js_name = schemaVersion)]
pub fn schema_version() -> u32 {
    tokmd_core::ffi::schema_version()
}

/// Return the current analysis receipt schema version for `runAnalyze`.
#[cfg(feature = "analysis")]
#[wasm_bindgen(js_name = analysisSchemaVersion)]
pub fn analysis_schema_version() -> u32 {
    CORE_ANALYSIS_SCHEMA_VERSION
}

fn capabilities_json() -> String {
    #[cfg(feature = "analysis")]
    let modes = vec!["lang", "module", "export", "analyze"];
    #[cfg(not(feature = "analysis"))]
    let modes = vec!["lang", "module", "export"];

    #[cfg(feature = "analysis")]
    let rootless_presets = ROOTLESS_ANALYZE_PRESETS;
    #[cfg(not(feature = "analysis"))]
    let rootless_presets: &[&str] = &[];

    serde_json::json!({
        "modes": modes,
        "analyze": {
            "rootlessPresets": rootless_presets,
        },
    })
    .to_string()
}

/// Return the rootless in-memory capability surface for browser callers.
#[wasm_bindgen(js_name = capabilities)]
pub fn capabilities() -> Result<JsValue, JsValue> {
    JSON::parse(&capabilities_json())
        .map_err(|_| to_js_error("failed to parse tokmd wasm capabilities JSON"))
}

/// Run a tokmd mode and return the raw JSON response envelope.
#[wasm_bindgen(js_name = runJson)]
pub fn run_json(mode: &str, args_json: &str) -> String {
    if let Err(err) = validate_mode_args_json(mode, args_json) {
        return ResponseEnvelope::error(&err).to_json();
    }
    tokmd_core::ffi::run_json(mode, args_json)
}

/// Run a tokmd scan over raw archive (ZIP) bytes and return the raw JSON response envelope.
///
/// This is the byte-mode counterpart to [`run_json`]: the archive is the sole
/// input source, so `options_json` must not carry `inputs` or `paths`. The
/// `archive_bytes` view is copied into an owned buffer at the boundary (the
/// crate keeps `#![forbid(unsafe_code)]`) and forwarded to
/// [`tokmd_core::ffi::run_json_bytes`], which admits the bytes fail-closed
/// through the single authoritative engine and routes them through the existing
/// mode dispatch.
///
/// Supported modes are `"lang"`, `"module"`, `"export"`, and `"analyze"`
/// (rootless presets only). Host-only modes are rejected with an error
/// envelope.
#[cfg(feature = "archive-zip")]
#[wasm_bindgen(js_name = runJsonBytes)]
pub fn run_json_bytes(mode: &str, options_json: &str, archive_bytes: Uint8Array) -> String {
    if let Err(err) = validate_mode_args_json(mode, options_json) {
        return ResponseEnvelope::error(&err).to_json();
    }
    tokmd_core::ffi::run_json_bytes(mode, options_json, &archive_bytes.to_vec())
}

/// Run a tokmd mode with raw JSON args and return only the extracted data JSON payload.
#[wasm_bindgen(js_name = runDataJson)]
pub fn run_data_json(mode: &str, args_json: &str) -> Result<String, JsValue> {
    extract_mode_data_json(mode, args_json).map_err(to_js_error)
}

/// Run a tokmd mode with a plain JavaScript object and return the extracted data payload.
#[wasm_bindgen(js_name = run)]
pub fn run(mode: &str, args: JsValue) -> Result<JsValue, JsValue> {
    run_mode_js(mode, args)
}

/// Run the `lang` workflow on in-memory inputs.
#[wasm_bindgen(js_name = runLang)]
pub fn run_lang(args: JsValue) -> Result<JsValue, JsValue> {
    run_mode_js("lang", args)
}

/// Run the `module` workflow on in-memory inputs.
#[wasm_bindgen(js_name = runModule)]
pub fn run_module(args: JsValue) -> Result<JsValue, JsValue> {
    run_mode_js("module", args)
}

/// Run the `export` workflow on in-memory inputs.
#[wasm_bindgen(js_name = runExport)]
pub fn run_export(args: JsValue) -> Result<JsValue, JsValue> {
    run_mode_js("export", args)
}

/// Run the `analyze` workflow on in-memory inputs.
///
/// `tokmd-wasm` currently supports only `preset: "receipt"` and
/// `preset: "estimate"` because the richer analysis presets still depend on
/// filesystem-backed content scans. Omitting `preset` defaults to `receipt`,
/// matching `tokmd-core`.
#[cfg(feature = "analysis")]
#[wasm_bindgen(js_name = runAnalyze)]
pub fn run_analyze(args: JsValue) -> Result<JsValue, JsValue> {
    run_analyze_js(args)
}

/// Minimal dependency-free ZIP fixture builder shared by the native and
/// `wasm-bindgen-test` archive-byte coverage.
///
/// The repo convention is to generate archive fixtures at test time rather than
/// committing binaries (no `.zip` files are checked in), so this builds a
/// `Stored` (uncompressed) ZIP by hand with correct CRC-32 checksums. The
/// `zip`-crate reader behind the `archive-zip` admission engine validates the
/// CRC on read, so an incorrect checksum here would fail the test rather than
/// pass silently. Pure-`std`, safe code keeps it `wasm32`-compatible without
/// adding a decompression dev-dependency to the default browser test lanes.
#[cfg(all(test, feature = "archive-zip"))]
mod archive_fixture {
    /// Fixture entries in normalized-sorted path order, matching the order the
    /// ZIP admission engine yields so the inline `{ path, text }` parity inputs
    /// line up exactly.
    pub(crate) const ENTRIES: &[(&str, &str)] = &[
        ("src/lib.rs", "pub fn alpha() -> usize { 1 }\n"),
        ("src/main.rs", "fn main() {}\n"),
        ("tests/basic.py", "# TODO: keep smoke\nprint('ok')\n"),
    ];

    /// CRC-32 (IEEE polynomial, reflected) computed bit-by-bit so the fixture
    /// needs no lookup table or external crate.
    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &byte in data {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                let mask = (crc & 1).wrapping_neg();
                crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
            }
        }
        !crc
    }

    /// Build a `Stored` ZIP archive containing [`ENTRIES`].
    pub(crate) fn tiny_zip() -> Vec<u8> {
        let mut body = Vec::new();
        let mut central = Vec::new();
        // DOS date 1980-01-01 (time 0); a zero date field is technically
        // invalid, so use the epoch the format defines.
        const DOS_TIME: u16 = 0;
        const DOS_DATE: u16 = 0x0021;

        for (name, text) in ENTRIES {
            let name_bytes = name.as_bytes();
            let data = text.as_bytes();
            let crc = crc32(data);
            let size = u32::try_from(data.len()).expect("fixture entry fits in u32");
            let offset = u32::try_from(body.len()).expect("fixture offset fits in u32");
            let name_len = u16::try_from(name_bytes.len()).expect("fixture name fits in u16");

            // Local file header.
            body.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
            body.extend_from_slice(&20u16.to_le_bytes()); // version needed
            body.extend_from_slice(&0u16.to_le_bytes()); // flags
            body.extend_from_slice(&0u16.to_le_bytes()); // method: stored
            body.extend_from_slice(&DOS_TIME.to_le_bytes());
            body.extend_from_slice(&DOS_DATE.to_le_bytes());
            body.extend_from_slice(&crc.to_le_bytes());
            body.extend_from_slice(&size.to_le_bytes()); // compressed
            body.extend_from_slice(&size.to_le_bytes()); // uncompressed
            body.extend_from_slice(&name_len.to_le_bytes());
            body.extend_from_slice(&0u16.to_le_bytes()); // extra len
            body.extend_from_slice(name_bytes);
            body.extend_from_slice(data);

            // Central directory header.
            central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
            central.extend_from_slice(&20u16.to_le_bytes()); // version made by
            central.extend_from_slice(&20u16.to_le_bytes()); // version needed
            central.extend_from_slice(&0u16.to_le_bytes()); // flags
            central.extend_from_slice(&0u16.to_le_bytes()); // method: stored
            central.extend_from_slice(&DOS_TIME.to_le_bytes());
            central.extend_from_slice(&DOS_DATE.to_le_bytes());
            central.extend_from_slice(&crc.to_le_bytes());
            central.extend_from_slice(&size.to_le_bytes()); // compressed
            central.extend_from_slice(&size.to_le_bytes()); // uncompressed
            central.extend_from_slice(&name_len.to_le_bytes());
            central.extend_from_slice(&0u16.to_le_bytes()); // extra len
            central.extend_from_slice(&0u16.to_le_bytes()); // comment len
            central.extend_from_slice(&0u16.to_le_bytes()); // disk number start
            central.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
            central.extend_from_slice(&0u32.to_le_bytes()); // external attrs
            central.extend_from_slice(&offset.to_le_bytes());
            central.extend_from_slice(name_bytes);
        }

        let central_offset = u32::try_from(body.len()).expect("central offset fits in u32");
        let central_size = u32::try_from(central.len()).expect("central size fits in u32");
        let entry_count = u16::try_from(ENTRIES.len()).expect("fixture entry count fits in u16");

        let mut archive = body;
        archive.extend_from_slice(&central);
        // End of central directory record.
        archive.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        archive.extend_from_slice(&0u16.to_le_bytes()); // disk number
        archive.extend_from_slice(&0u16.to_le_bytes()); // disk with central dir
        archive.extend_from_slice(&entry_count.to_le_bytes());
        archive.extend_from_slice(&entry_count.to_le_bytes());
        archive.extend_from_slice(&central_size.to_le_bytes());
        archive.extend_from_slice(&central_offset.to_le_bytes());
        archive.extend_from_slice(&0u16.to_le_bytes()); // comment len
        archive
    }

    /// Equivalent inline `{ path, text }` options carrying the same logical file
    /// set, used as the JSON-mode parity oracle for the byte mode.
    pub(crate) fn inline_options_json() -> String {
        let inputs: Vec<serde_json::Value> = ENTRIES
            .iter()
            .map(|(name, text)| serde_json::json!({ "path": name, "text": text }))
            .collect();
        serde_json::json!({ "lang": { "files": true }, "inputs": inputs }).to_string()
    }

    /// Equivalent inline `{ path, text }` options with no mode-specific
    /// settings, used as the JSON-mode parity oracle for byte-mode `module` and
    /// `export` (whose byte call carries empty options `{}`).
    pub(crate) fn inline_bare_options_json() -> String {
        let inputs: Vec<serde_json::Value> = ENTRIES
            .iter()
            .map(|(name, text)| serde_json::json!({ "path": name, "text": text }))
            .collect();
        serde_json::json!({ "inputs": inputs }).to_string()
    }

    /// Equivalent inline `{ path, text }` analyze options for rootless presets.
    #[cfg(feature = "analysis")]
    pub(crate) fn inline_analyze_options_json(preset: &str) -> String {
        let inputs: Vec<serde_json::Value> = ENTRIES
            .iter()
            .map(|(name, text)| serde_json::json!({ "path": name, "text": text }))
            .collect();
        serde_json::json!({ "preset": preset, "inputs": inputs }).to_string()
    }

    /// Byte-mode analyze options (archive bytes carry the file set).
    #[cfg(feature = "analysis")]
    pub(crate) fn byte_analyze_options_json(preset: &str) -> String {
        serde_json::json!({ "preset": preset }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture_inputs() -> Value {
        json!([
            {
                "path": "crates/app/src/lib.rs",
                "text": "pub fn alpha() -> usize { 1 }\n"
            },
            {
                "path": "src/main.rs",
                "text": "fn main() {}\n"
            },
            {
                "path": "tests/basic.py",
                "text": "# TODO: keep smoke\nprint('ok')\n"
            }
        ])
    }

    #[test]
    fn run_json_returns_valid_envelope() {
        let result = run_json("version", "{}");
        let envelope = tokmd_envelope::ffi::parse_envelope(&result).expect("valid JSON envelope");

        assert_eq!(envelope["ok"], true);
        assert_eq!(envelope["data"]["version"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn run_data_json_returns_payload_without_envelope() {
        let payload = run_data_json("version", "{}").expect("version payload");
        let value: Value = serde_json::from_str(&payload).expect("valid payload json");

        assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
        assert!(value.get("schema_version").is_some());
    }

    #[test]
    fn capabilities_reports_rootless_surface() {
        let obj: Value = serde_json::from_str(&capabilities_json()).expect("capabilities JSON");

        assert_eq!(obj["modes"][0], "lang");
        assert_eq!(obj["modes"][1], "module");
        assert_eq!(obj["modes"][2], "export");

        #[cfg(feature = "analysis")]
        {
            assert_eq!(obj["modes"][3], "analyze");
            assert_eq!(
                obj["analyze"]["rootlessPresets"],
                json!(["receipt", "estimate"])
            );
        }

        #[cfg(not(feature = "analysis"))]
        {
            assert_eq!(obj["modes"].as_array().expect("modes").len(), 3);
            assert_eq!(obj["analyze"]["rootlessPresets"], json!([]));
        }
    }

    #[test]
    fn normalize_raw_json_args_accepts_json_object_strings() {
        let raw = r#"{"inputs":[{"path":"src/lib.rs","text":"pub fn alpha() {}\n"}]}"#;

        assert_eq!(
            normalize_raw_json_args(raw).expect("valid raw args"),
            raw.to_string()
        );
    }

    #[test]
    fn normalize_raw_json_args_rejects_invalid_json_strings() {
        let err = normalize_raw_json_args("{not json").expect_err("invalid raw args");

        assert!(err.contains("failed to parse JSON string arguments"));
    }

    #[test]
    fn run_mode_value_lang_supports_in_memory_inputs() {
        let data = run_mode_value(
            "lang",
            &json!({
                "inputs": fixture_inputs(),
                "files": true
            }),
        )
        .expect("lang data");

        assert_eq!(data["mode"], "lang");
        assert_eq!(data["scan"]["paths"][0], "crates/app/src/lib.rs");
        assert_eq!(data["total"]["files"], 3);
    }

    #[test]
    fn run_mode_value_export_preserves_logical_paths() {
        let data = run_mode_value(
            "export",
            &json!({
                "inputs": fixture_inputs()
            }),
        )
        .expect("export data");
        let paths: Vec<&str> = data["rows"]
            .as_array()
            .expect("rows array")
            .iter()
            .map(|row| row["path"].as_str().expect("row path"))
            .collect();

        assert_eq!(data["mode"], "export");
        assert!(paths.contains(&"crates/app/src/lib.rs"));
        assert!(paths.contains(&"tests/basic.py"));
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn run_mode_value_analyze_estimate_returns_effort_payload() {
        let data = run_mode_value(
            "analyze",
            &json!({
                "inputs": fixture_inputs(),
                "preset": "estimate"
            }),
        )
        .expect("analysis data");

        assert_eq!(data["mode"], "analysis");
        assert_eq!(data["source"]["inputs"][1], "src/main.rs");
        assert_eq!(data["effort"]["model"], "cocomo81-basic");
        assert_eq!(data["effort"]["size_basis"]["total_lines"], 3);
        assert!(
            data["effort"]["results"]["effort_pm_p50"]
                .as_f64()
                .expect("effort p50")
                > 0.0
        );
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn run_mode_value_analyze_receipt_returns_rootless_receipt_payload() {
        let data = run_mode_value(
            "analyze",
            &json!({
                "inputs": fixture_inputs(),
                "preset": "receipt"
            }),
        )
        .expect("analysis data");

        assert_eq!(data["mode"], "analysis");
        assert_eq!(data["source"]["inputs"][2], "tests/basic.py");
        assert_eq!(data["derived"]["totals"]["files"], 3);
        assert_eq!(data["effort"], Value::Null);
        assert_eq!(data["git"], Value::Null);
        assert!(
            data["warnings"]
                .as_array()
                .expect("warnings array")
                .iter()
                .filter_map(Value::as_str)
                .any(|warning| warning.contains("no host root") && warning.contains("file-backed"))
        );
        assert!(
            data["warnings"]
                .as_array()
                .expect("warnings array")
                .iter()
                .filter_map(Value::as_str)
                .any(|warning| warning.contains("no host root") && warning.contains("git"))
        );
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn run_mode_value_analyze_without_preset_defaults_to_receipt_payload() {
        let data = run_mode_value(
            "analyze",
            &json!({
                "inputs": fixture_inputs()
            }),
        )
        .expect("analysis data");

        assert_eq!(data["mode"], "analysis");
        assert_eq!(data["source"]["inputs"][0], "crates/app/src/lib.rs");
        assert_eq!(data["derived"]["totals"]["files"], 3);
        assert_eq!(data["effort"], Value::Null);
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn validate_analyze_args_accepts_rootless_receipt_and_estimate() {
        validate_analyze_args_json(
            r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }]
            }"#,
        )
        .expect("missing preset should default to receipt");

        validate_analyze_args_json(
            r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "analyze": { "preset": "Receipt" }
            }"#,
        )
        .expect("nested mixed-case receipt should be allowed");

        validate_analyze_args_json(
            r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "preset": "estimate"
            }"#,
        )
        .expect("estimate should be allowed");

        validate_analyze_args_json(
            r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "analyze": { "preset": "Estimate" }
            }"#,
        )
        .expect("nested mixed-case estimate should be allowed");

        let err = validate_analyze_args_json(
            r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "preset": "health"
            }"#,
        )
        .expect_err("unsupported preset should be rejected");

        assert!(err.message.contains("preset=\"receipt\""));
        assert!(err.message.contains("preset=\"estimate\""));
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn run_json_analyze_rejects_unsupported_presets() {
        let result = run_json(
            "analyze",
            r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "preset": "health"
            }"#,
        );
        let envelope = tokmd_envelope::ffi::parse_envelope(&result).expect("valid JSON envelope");

        assert_eq!(envelope["ok"], false);
        assert_eq!(envelope["error"]["code"], "not_implemented");
        assert!(
            envelope["error"]["message"]
                .as_str()
                .expect("error message")
                .contains("preset=\"receipt\"")
        );
        assert!(
            envelope["error"]["message"]
                .as_str()
                .expect("error message")
                .contains("preset=\"estimate\"")
        );
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn run_mode_value_analyze_accepts_nested_case_insensitive_estimate() {
        let data = run_mode_value(
            "analyze",
            &json!({
                "inputs": fixture_inputs(),
                "analyze": { "preset": "Estimate" }
            }),
        )
        .expect("analysis data");

        assert_eq!(data["mode"], "analysis");
        assert_eq!(data["source"]["inputs"][0], "crates/app/src/lib.rs");
        assert_eq!(data["effort"]["model"], "cocomo81-basic");
    }

    #[test]
    fn run_mode_value_surfaces_upstream_errors() {
        let err = run_mode_value(
            "lang",
            &json!({
                "inputs": fixture_inputs(),
                "paths": ["src"]
            }),
        )
        .expect_err("paths + inputs should error");

        assert!(err.contains("[invalid_settings]"));
        assert!(err.contains("cannot be combined with in-memory inputs"));
    }

    #[test]
    fn schema_version_matches_core_receipts() {
        assert_eq!(schema_version(), tokmd_types::SCHEMA_VERSION);
    }

    #[cfg(feature = "archive-zip")]
    #[test]
    fn core_run_json_bytes_lang_matches_inline_inputs() {
        use crate::archive_fixture::{inline_options_json, tiny_zip};

        let bytes = tiny_zip();
        let from_zip_raw =
            tokmd_core::ffi::run_json_bytes("lang", r#"{"lang":{"files":true}}"#, &bytes);
        let from_json_raw = tokmd_core::ffi::run_json("lang", &inline_options_json());

        let mut from_zip: Value = serde_json::from_str(&from_zip_raw).expect("byte-mode envelope");
        let mut from_json: Value =
            serde_json::from_str(&from_json_raw).expect("json-mode envelope");

        assert_eq!(from_zip["ok"], json!(true), "byte-mode call should succeed");
        assert_eq!(from_zip["data"]["mode"], json!("lang"));
        assert_eq!(from_zip["data"]["total"]["files"], json!(3));

        let langs: Vec<&str> = from_zip["data"]["rows"]
            .as_array()
            .expect("lang rows array")
            .iter()
            .filter_map(|row| row["lang"].as_str())
            .collect();
        assert!(
            langs.contains(&"Rust"),
            "expected a Rust row, got {langs:?}"
        );
        assert!(
            langs.contains(&"Python"),
            "expected a Python row, got {langs:?}"
        );

        // Strongest oracle: the byte-mode envelope must equal the inline
        // `{ path, text }` envelope modulo the volatile timestamp.
        for envelope in [&mut from_zip, &mut from_json] {
            if let Some(obj) = envelope.get_mut("data").and_then(Value::as_object_mut) {
                obj.remove("generated_at_ms");
            }
        }
        assert_eq!(
            from_zip, from_json,
            "byte-mode envelope diverged from the equivalent inline-inputs envelope"
        );
    }

    /// Recursively remove volatile timestamp keys so byte-mode and inline-mode
    /// envelopes can be compared for full structural equality.
    #[cfg(feature = "archive-zip")]
    fn scrub_volatile_timestamps(value: &mut Value) {
        match value {
            Value::Array(items) => {
                for item in items {
                    scrub_volatile_timestamps(item);
                }
            }
            Value::Object(map) => {
                map.remove("generated_at_ms");
                map.remove("export_generated_at_ms");
                for nested in map.values_mut() {
                    scrub_volatile_timestamps(nested);
                }
            }
            _ => {}
        }
    }

    #[cfg(feature = "archive-zip")]
    #[test]
    fn core_run_json_bytes_module_matches_inline_inputs() {
        use crate::archive_fixture::{inline_bare_options_json, tiny_zip};

        let bytes = tiny_zip();
        let from_zip_raw = tokmd_core::ffi::run_json_bytes("module", "{}", &bytes);
        let from_json_raw = tokmd_core::ffi::run_json("module", &inline_bare_options_json());

        let mut from_zip: Value = serde_json::from_str(&from_zip_raw).expect("byte-mode envelope");
        let mut from_json: Value =
            serde_json::from_str(&from_json_raw).expect("json-mode envelope");

        assert_eq!(from_zip["ok"], json!(true), "byte-mode call should succeed");
        assert_eq!(from_zip["data"]["mode"], json!("module"));
        assert!(
            from_zip["data"]["rows"].as_array().is_some(),
            "module receipt should carry rows"
        );

        scrub_volatile_timestamps(&mut from_zip);
        scrub_volatile_timestamps(&mut from_json);
        assert_eq!(
            from_zip, from_json,
            "byte-mode module envelope diverged from the equivalent inline-inputs envelope"
        );
    }

    #[cfg(feature = "archive-zip")]
    #[test]
    fn core_run_json_bytes_export_matches_inline_inputs() {
        use crate::archive_fixture::{inline_bare_options_json, tiny_zip};

        let bytes = tiny_zip();
        let from_zip_raw = tokmd_core::ffi::run_json_bytes("export", "{}", &bytes);
        let from_json_raw = tokmd_core::ffi::run_json("export", &inline_bare_options_json());

        let mut from_zip: Value = serde_json::from_str(&from_zip_raw).expect("byte-mode envelope");
        let mut from_json: Value =
            serde_json::from_str(&from_json_raw).expect("json-mode envelope");

        assert_eq!(from_zip["ok"], json!(true), "byte-mode call should succeed");
        assert_eq!(from_zip["data"]["mode"], json!("export"));
        let paths: Vec<&str> = from_zip["data"]["rows"]
            .as_array()
            .expect("export rows array")
            .iter()
            .filter_map(|row| row["path"].as_str())
            .collect();
        assert!(
            paths.contains(&"src/lib.rs") && paths.contains(&"tests/basic.py"),
            "expected fixture paths, got {paths:?}"
        );

        scrub_volatile_timestamps(&mut from_zip);
        scrub_volatile_timestamps(&mut from_json);
        assert_eq!(
            from_zip, from_json,
            "byte-mode export envelope diverged from the equivalent inline-inputs envelope"
        );
    }

    #[cfg(feature = "archive-zip")]
    #[test]
    fn core_run_json_bytes_rejects_paths_option() {
        use crate::archive_fixture::tiny_zip;

        let bytes = tiny_zip();
        let raw = tokmd_core::ffi::run_json_bytes("lang", r#"{"paths":["."]}"#, &bytes);
        let envelope: Value = serde_json::from_str(&raw).expect("envelope");

        assert_eq!(envelope["ok"], json!(false));
        assert_eq!(envelope["error"]["code"], json!("invalid_settings"));
    }

    #[cfg(all(feature = "archive-zip", feature = "analysis"))]
    #[test]
    fn core_run_json_bytes_analyze_receipt_matches_inline_inputs() {
        use crate::archive_fixture::{
            byte_analyze_options_json, inline_analyze_options_json, tiny_zip,
        };

        let bytes = tiny_zip();
        let from_zip_raw = tokmd_core::ffi::run_json_bytes(
            "analyze",
            &byte_analyze_options_json("receipt"),
            &bytes,
        );
        let from_json_raw =
            tokmd_core::ffi::run_json("analyze", &inline_analyze_options_json("receipt"));

        let mut from_zip: Value = serde_json::from_str(&from_zip_raw).expect("byte-mode envelope");
        let mut from_json: Value =
            serde_json::from_str(&from_json_raw).expect("json-mode envelope");

        assert_eq!(from_zip["ok"], json!(true), "byte-mode call should succeed");
        assert_eq!(from_zip["data"]["mode"], json!("analysis"));
        assert_eq!(from_zip["data"]["derived"]["totals"]["files"], json!(3));
        assert_eq!(from_zip["data"]["effort"], Value::Null);

        scrub_analyze_envelope_timestamps(&mut from_zip);
        scrub_analyze_envelope_timestamps(&mut from_json);
        assert_eq!(
            from_zip, from_json,
            "byte-mode analyze receipt envelope diverged from inline-inputs envelope"
        );
    }

    #[cfg(all(feature = "archive-zip", feature = "analysis"))]
    #[test]
    fn core_run_json_bytes_analyze_estimate_matches_inline_inputs() {
        use crate::archive_fixture::{
            byte_analyze_options_json, inline_analyze_options_json, tiny_zip,
        };

        let bytes = tiny_zip();
        let from_zip_raw = tokmd_core::ffi::run_json_bytes(
            "analyze",
            &byte_analyze_options_json("estimate"),
            &bytes,
        );
        let from_json_raw =
            tokmd_core::ffi::run_json("analyze", &inline_analyze_options_json("estimate"));

        let mut from_zip: Value = serde_json::from_str(&from_zip_raw).expect("byte-mode envelope");
        let mut from_json: Value =
            serde_json::from_str(&from_json_raw).expect("json-mode envelope");

        assert_eq!(from_zip["ok"], json!(true), "byte-mode call should succeed");
        assert_eq!(from_zip["data"]["mode"], json!("analysis"));
        assert_eq!(from_zip["data"]["effort"]["model"], json!("cocomo81-basic"));

        scrub_analyze_envelope_timestamps(&mut from_zip);
        scrub_analyze_envelope_timestamps(&mut from_json);
        assert_eq!(
            from_zip, from_json,
            "byte-mode analyze estimate envelope diverged from inline-inputs envelope"
        );
    }

    fn scrub_analyze_envelope_timestamps(envelope: &mut Value) {
        if let Some(data) = envelope.get_mut("data").and_then(Value::as_object_mut) {
            data.remove("generated_at_ms");
            if let Some(source) = data.get_mut("source").and_then(Value::as_object_mut) {
                source.remove("export_generated_at_ms");
            }
        }
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn analysis_schema_version_matches_analysis_receipts() {
        assert_eq!(analysis_schema_version(), CORE_ANALYSIS_SCHEMA_VERSION);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use serde_json::Value;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    fn parse_js_args(json: &str) -> JsValue {
        JSON::parse(json).expect("valid JS object")
    }

    fn js_value_to_json(value: &JsValue) -> Value {
        let json = JSON::stringify(value)
            .expect("serializable JS value")
            .as_string()
            .expect("JSON string");
        serde_json::from_str(&json).expect("valid JSON value")
    }

    fn core_mode_value(mode: &str, args_json: &str) -> Value {
        let envelope_json = tokmd_core::ffi::run_json(mode, args_json);
        let data_json =
            tokmd_envelope::ffi::extract_data_json(&envelope_json).expect("core data payload");
        serde_json::from_str(&data_json).expect("valid core JSON value")
    }

    fn assert_generated_at_ms_nonzero(label: &str, value: &Value) {
        let timestamp = value
            .get("generated_at_ms")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| panic!("{label} missing numeric generated_at_ms"));
        assert!(timestamp > 0, "{label} generated_at_ms must not be 0");
    }

    fn normalize_volatile_timestamps(value: &mut Value) {
        match value {
            Value::Array(items) => {
                for item in items {
                    normalize_volatile_timestamps(item);
                }
            }
            Value::Object(object) => {
                for (key, value) in object {
                    if key == "generated_at_ms" || key == "export_generated_at_ms" {
                        if !value.is_null() {
                            *value = Value::from(1);
                        }
                    } else {
                        normalize_volatile_timestamps(value);
                    }
                }
            }
            _ => {}
        }
    }

    fn values_match_js_boundary(actual: &Value, expected: &Value) -> bool {
        match (actual, expected) {
            (Value::Null, Value::Null)
            | (Value::Bool(_), Value::Bool(_))
            | (Value::String(_), Value::String(_)) => actual == expected,
            (Value::Number(actual), Value::Number(expected)) => {
                numbers_match_js_boundary(actual, expected)
            }
            (Value::Array(actual), Value::Array(expected)) => {
                actual.len() == expected.len()
                    && actual
                        .iter()
                        .zip(expected.iter())
                        .all(|(actual, expected)| values_match_js_boundary(actual, expected))
            }
            (Value::Object(actual), Value::Object(expected)) => {
                actual.len() == expected.len()
                    && actual.iter().all(|(key, actual_value)| {
                        expected.get(key).is_some_and(|expected_value| {
                            values_match_js_boundary(actual_value, expected_value)
                        })
                    })
            }
            _ => false,
        }
    }

    fn numbers_match_js_boundary(
        actual: &serde_json::Number,
        expected: &serde_json::Number,
    ) -> bool {
        const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

        if actual == expected {
            return true;
        }

        if let (Some(actual), Some(expected)) = (actual.as_i64(), expected.as_i64()) {
            return actual == expected;
        }

        if let (Some(actual), Some(expected)) = (actual.as_u64(), expected.as_u64()) {
            return actual == expected;
        }

        let (Some(actual), Some(expected)) = (actual.as_f64(), expected.as_f64()) else {
            return false;
        };

        if actual != expected {
            return false;
        }

        let both_integral = actual.fract() == 0.0 && expected.fract() == 0.0;
        if both_integral && (actual.abs() > MAX_SAFE_INTEGER || expected.abs() > MAX_SAFE_INTEGER) {
            return false;
        }

        true
    }

    #[wasm_bindgen_test]
    fn run_lang_exercises_js_value_boundary() {
        let args_json = r#"{
            "inputs": [
                { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
                { "path": "tests/basic.py", "text": "print('ok')\n" }
            ],
            "files": true
        }"#;
        let data = run_lang(parse_js_args(args_json)).expect("lang data");
        let mut parsed = js_value_to_json(&data);
        let mut expected = core_mode_value("lang", args_json);

        assert_eq!(parsed["mode"], "lang");
        assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
        assert_eq!(parsed["total"]["files"], 2);
        assert_generated_at_ms_nonzero("lang wasm payload", &parsed);
        assert_generated_at_ms_nonzero("lang core payload", &expected);
        normalize_volatile_timestamps(&mut parsed);
        normalize_volatile_timestamps(&mut expected);
        assert!(
            values_match_js_boundary(&parsed, &expected),
            "wasm payload diverged from core payload\nactual: {parsed}\nexpected: {expected}"
        );
    }

    #[wasm_bindgen_test]
    fn run_module_exercises_js_value_boundary() {
        let args_json = r#"{
            "inputs": [
                { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
                { "path": "tests/basic.py", "text": "print('ok')\n" }
            ]
        }"#;
        let data = run_module(parse_js_args(args_json)).expect("module data");
        let mut parsed = js_value_to_json(&data);
        let mut expected = core_mode_value("module", args_json);

        assert_eq!(parsed["mode"], "module");
        assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
        assert!(parsed["rows"].as_array().is_some());
        assert_generated_at_ms_nonzero("module wasm payload", &parsed);
        assert_generated_at_ms_nonzero("module core payload", &expected);
        normalize_volatile_timestamps(&mut parsed);
        normalize_volatile_timestamps(&mut expected);
        assert!(
            values_match_js_boundary(&parsed, &expected),
            "wasm payload diverged from core payload\nactual: {parsed}\nexpected: {expected}"
        );
    }

    #[wasm_bindgen_test]
    fn run_export_exercises_js_value_boundary() {
        let args_json = r#"{
            "inputs": [
                { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
                { "path": "tests/basic.py", "text": "print('ok')\n" }
            ]
        }"#;
        let data = run_export(parse_js_args(args_json)).expect("export data");
        let mut parsed = js_value_to_json(&data);
        let mut expected = core_mode_value("export", args_json);

        assert_eq!(parsed["mode"], "export");
        assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
        assert_eq!(parsed["rows"][0]["path"], "src/lib.rs");
        assert_generated_at_ms_nonzero("export wasm payload", &parsed);
        assert_generated_at_ms_nonzero("export core payload", &expected);
        normalize_volatile_timestamps(&mut parsed);
        normalize_volatile_timestamps(&mut expected);
        assert!(
            values_match_js_boundary(&parsed, &expected),
            "wasm payload diverged from core payload\nactual: {parsed}\nexpected: {expected}"
        );
    }

    #[wasm_bindgen_test]
    fn run_surfaces_js_facing_errors() {
        let err = run(
            "lang",
            parse_js_args(
                r#"{
                    "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                    "paths": ["src"]
                }"#,
            ),
        )
        .expect_err("conflicting inputs should error")
        .dyn_into::<JsError>()
        .expect("js error");

        let message = err.message().as_string().expect("js string message");
        assert!(message.contains("[invalid_settings]"));
    }

    #[cfg(feature = "analysis")]
    #[wasm_bindgen_test]
    fn run_analyze_estimate_reports_analysis_schema_and_matches_core_payload() {
        let args_json = r#"{
            "inputs": [
                { "path": "crates/app/src/lib.rs", "text": "pub fn alpha() -> usize { 1 }\n" },
                { "path": "src/main.rs", "text": "fn main() {}\n" }
            ],
            "preset": "estimate"
        }"#;
        let data = run_analyze(parse_js_args(args_json)).expect("analysis data");
        let mut parsed = js_value_to_json(&data);
        let mut expected = core_mode_value("analyze", args_json);

        assert_eq!(analysis_schema_version(), CORE_ANALYSIS_SCHEMA_VERSION);
        assert_eq!(parsed["mode"], "analysis");
        assert_eq!(parsed["source"]["inputs"][0], "crates/app/src/lib.rs");
        assert_eq!(parsed["effort"]["model"], "cocomo81-basic");
        assert_generated_at_ms_nonzero("analysis estimate wasm payload", &parsed);
        assert_generated_at_ms_nonzero("analysis estimate core payload", &expected);
        normalize_volatile_timestamps(&mut parsed);
        normalize_volatile_timestamps(&mut expected);
        assert!(
            values_match_js_boundary(&parsed, &expected),
            "wasm payload diverged from core payload\nactual: {parsed}\nexpected: {expected}"
        );
    }

    #[cfg(feature = "analysis")]
    #[wasm_bindgen_test]
    fn run_analyze_receipt_matches_core_payload() {
        let args_json = r#"{
            "inputs": [
                { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }
            ],
            "preset": "receipt"
        }"#;
        let data = run_analyze(parse_js_args(args_json)).expect("analysis data");
        let mut parsed = js_value_to_json(&data);
        let mut expected = core_mode_value("analyze", args_json);

        assert_eq!(parsed["mode"], "analysis");
        assert_eq!(parsed["source"]["inputs"][0], "src/lib.rs");
        assert_eq!(parsed["derived"]["totals"]["files"], 1);
        assert_eq!(parsed["effort"], Value::Null);
        assert_generated_at_ms_nonzero("analysis receipt wasm payload", &parsed);
        assert_generated_at_ms_nonzero("analysis receipt core payload", &expected);
        normalize_volatile_timestamps(&mut parsed);
        normalize_volatile_timestamps(&mut expected);
        assert!(
            values_match_js_boundary(&parsed, &expected),
            "wasm payload diverged from core payload\nactual: {parsed}\nexpected: {expected}"
        );
    }

    #[cfg(feature = "analysis")]
    #[wasm_bindgen_test]
    fn run_analyze_without_preset_defaults_to_receipt() {
        let data = run_analyze(parse_js_args(
            r#"{
                "inputs": [
                    { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }
                ]
            }"#,
        ))
        .expect("analysis data");
        let parsed = js_value_to_json(&data);

        assert_eq!(parsed["mode"], "analysis");
        assert_eq!(parsed["source"]["inputs"][0], "src/lib.rs");
        assert_eq!(parsed["derived"]["totals"]["files"], 1);
        assert_eq!(parsed["effort"], Value::Null);
    }

    #[cfg(feature = "analysis")]
    #[wasm_bindgen_test]
    fn run_analyze_rejects_unsupported_presets() {
        let err = run_analyze(parse_js_args(
            r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "preset": "health"
            }"#,
        ))
        .expect_err("non-estimate preset should be rejected")
        .dyn_into::<JsError>()
        .expect("js error");

        let message = err.message().as_string().expect("js string message");
        assert!(message.contains("preset=\"receipt\""));
        assert!(message.contains("preset=\"estimate\""));
    }

    #[cfg(feature = "analysis")]
    #[wasm_bindgen_test]
    fn run_accepts_nested_case_insensitive_analyze_preset() {
        let data = run(
            "analyze",
            parse_js_args(
                r#"{
                    "inputs": [
                        { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }
                    ],
                    "analyze": { "preset": "Estimate" }
                }"#,
            ),
        )
        .expect("analysis data");
        let parsed = js_value_to_json(&data);

        assert_eq!(parsed["mode"], "analysis");
        assert_eq!(parsed["effort"]["model"], "cocomo81-basic");
    }

    #[cfg(feature = "archive-zip")]
    #[wasm_bindgen_test]
    fn run_json_bytes_lang_matches_inline_inputs_over_js_boundary() {
        use crate::archive_fixture::{inline_options_json, tiny_zip};

        let bytes = tiny_zip();
        // Cross the JS boundary explicitly: build a real Uint8Array view and
        // hand it to the binding, which copies it into an owned buffer.
        let view = Uint8Array::from(bytes.as_slice());
        let from_zip_raw = run_json_bytes("lang", r#"{"lang":{"files":true}}"#, view);
        let from_json_raw = tokmd_core::ffi::run_json("lang", &inline_options_json());

        let mut from_zip: Value =
            serde_json::from_str(&from_zip_raw).expect("byte-mode envelope json");
        let mut from_json: Value =
            serde_json::from_str(&from_json_raw).expect("json-mode envelope json");

        assert_eq!(
            from_zip["ok"],
            Value::Bool(true),
            "byte-mode should succeed"
        );
        assert_eq!(from_zip["data"]["mode"], "lang");
        assert_eq!(from_zip["data"]["total"]["files"], 3);
        assert_generated_at_ms_nonzero("byte-mode lang wasm payload", &from_zip["data"]);

        let langs: Vec<&str> = from_zip["data"]["rows"]
            .as_array()
            .expect("lang rows array")
            .iter()
            .filter_map(|row| row["lang"].as_str())
            .collect();
        assert!(
            langs.contains(&"Rust"),
            "expected a Rust row, got {langs:?}"
        );
        assert!(
            langs.contains(&"Python"),
            "expected a Python row, got {langs:?}"
        );

        normalize_volatile_timestamps(&mut from_zip);
        normalize_volatile_timestamps(&mut from_json);
        assert!(
            values_match_js_boundary(&from_zip, &from_json),
            "byte-mode envelope diverged from inline-inputs envelope\nzip: {from_zip}\njson: {from_json}"
        );
    }

    #[cfg(feature = "archive-zip")]
    #[wasm_bindgen_test]
    fn run_json_bytes_module_matches_inline_inputs_over_js_boundary() {
        use crate::archive_fixture::{inline_bare_options_json, tiny_zip};

        let bytes = tiny_zip();
        let view = Uint8Array::from(bytes.as_slice());
        let from_zip_raw = run_json_bytes("module", "{}", view);
        let from_json_raw = tokmd_core::ffi::run_json("module", &inline_bare_options_json());

        let mut from_zip: Value =
            serde_json::from_str(&from_zip_raw).expect("byte-mode envelope json");
        let mut from_json: Value =
            serde_json::from_str(&from_json_raw).expect("json-mode envelope json");

        assert_eq!(
            from_zip["ok"],
            Value::Bool(true),
            "byte-mode module should succeed"
        );
        assert_eq!(from_zip["data"]["mode"], "module");
        assert!(from_zip["data"]["rows"].as_array().is_some());
        assert_generated_at_ms_nonzero("byte-mode module wasm payload", &from_zip["data"]);

        normalize_volatile_timestamps(&mut from_zip);
        normalize_volatile_timestamps(&mut from_json);
        assert!(
            values_match_js_boundary(&from_zip, &from_json),
            "byte-mode module envelope diverged from inline-inputs envelope\nzip: {from_zip}\njson: {from_json}"
        );
    }

    #[cfg(feature = "archive-zip")]
    #[wasm_bindgen_test]
    fn run_json_bytes_export_matches_inline_inputs_over_js_boundary() {
        use crate::archive_fixture::{inline_bare_options_json, tiny_zip};

        let bytes = tiny_zip();
        let view = Uint8Array::from(bytes.as_slice());
        let from_zip_raw = run_json_bytes("export", "{}", view);
        let from_json_raw = tokmd_core::ffi::run_json("export", &inline_bare_options_json());

        let mut from_zip: Value =
            serde_json::from_str(&from_zip_raw).expect("byte-mode envelope json");
        let mut from_json: Value =
            serde_json::from_str(&from_json_raw).expect("json-mode envelope json");

        assert_eq!(
            from_zip["ok"],
            Value::Bool(true),
            "byte-mode export should succeed"
        );
        assert_eq!(from_zip["data"]["mode"], "export");
        let paths: Vec<&str> = from_zip["data"]["rows"]
            .as_array()
            .expect("export rows array")
            .iter()
            .filter_map(|row| row["path"].as_str())
            .collect();
        assert!(
            paths.contains(&"src/lib.rs") && paths.contains(&"tests/basic.py"),
            "expected fixture paths, got {paths:?}"
        );
        assert_generated_at_ms_nonzero("byte-mode export wasm payload", &from_zip["data"]);

        normalize_volatile_timestamps(&mut from_zip);
        normalize_volatile_timestamps(&mut from_json);
        assert!(
            values_match_js_boundary(&from_zip, &from_json),
            "byte-mode export envelope diverged from inline-inputs envelope\nzip: {from_zip}\njson: {from_json}"
        );
    }

    #[cfg(feature = "archive-zip")]
    #[wasm_bindgen_test]
    fn run_json_bytes_rejects_paths_option() {
        use crate::archive_fixture::tiny_zip;

        let bytes = tiny_zip();
        let view = Uint8Array::from(bytes.as_slice());
        let raw = run_json_bytes("lang", r#"{"paths":["."]}"#, view);
        let envelope: Value = serde_json::from_str(&raw).expect("envelope json");

        assert_eq!(envelope["ok"], Value::Bool(false));
        assert_eq!(envelope["error"]["code"], "invalid_settings");
    }

    #[cfg(all(feature = "archive-zip", feature = "analysis"))]
    #[wasm_bindgen_test]
    fn run_json_bytes_analyze_receipt_matches_inline_inputs_over_js_boundary() {
        use crate::archive_fixture::{
            byte_analyze_options_json, inline_analyze_options_json, tiny_zip,
        };

        let bytes = tiny_zip();
        let view = Uint8Array::from(bytes.as_slice());
        let from_zip_raw = run_json_bytes("analyze", &byte_analyze_options_json("receipt"), view);
        let from_json_raw =
            tokmd_core::ffi::run_json("analyze", &inline_analyze_options_json("receipt"));

        let mut from_zip: Value =
            serde_json::from_str(&from_zip_raw).expect("byte-mode envelope json");
        let mut from_json: Value =
            serde_json::from_str(&from_json_raw).expect("json-mode envelope json");

        assert_eq!(
            from_zip["ok"],
            Value::Bool(true),
            "byte-mode analyze receipt should succeed"
        );
        assert_eq!(from_zip["data"]["mode"], "analysis");
        assert_eq!(from_zip["data"]["derived"]["totals"]["files"], 3);
        assert_eq!(from_zip["data"]["effort"], Value::Null);
        assert_generated_at_ms_nonzero("byte-mode analyze receipt wasm payload", &from_zip["data"]);

        normalize_volatile_timestamps(&mut from_zip);
        normalize_volatile_timestamps(&mut from_json);
        assert!(
            values_match_js_boundary(&from_zip, &from_json),
            "byte-mode analyze receipt envelope diverged from inline-inputs envelope\nzip: {from_zip}\njson: {from_json}"
        );
    }

    #[cfg(all(feature = "archive-zip", feature = "analysis"))]
    #[wasm_bindgen_test]
    fn run_json_bytes_analyze_estimate_matches_inline_inputs_over_js_boundary() {
        use crate::archive_fixture::{
            byte_analyze_options_json, inline_analyze_options_json, tiny_zip,
        };

        let bytes = tiny_zip();
        let view = Uint8Array::from(bytes.as_slice());
        let from_zip_raw = run_json_bytes("analyze", &byte_analyze_options_json("estimate"), view);
        let from_json_raw =
            tokmd_core::ffi::run_json("analyze", &inline_analyze_options_json("estimate"));

        let mut from_zip: Value =
            serde_json::from_str(&from_zip_raw).expect("byte-mode envelope json");
        let mut from_json: Value =
            serde_json::from_str(&from_json_raw).expect("json-mode envelope json");

        assert_eq!(
            from_zip["ok"],
            Value::Bool(true),
            "byte-mode analyze estimate should succeed"
        );
        assert_eq!(from_zip["data"]["mode"], "analysis");
        assert_eq!(from_zip["data"]["effort"]["model"], "cocomo81-basic");
        assert_generated_at_ms_nonzero(
            "byte-mode analyze estimate wasm payload",
            &from_zip["data"],
        );

        normalize_volatile_timestamps(&mut from_zip);
        normalize_volatile_timestamps(&mut from_json);
        assert!(
            values_match_js_boundary(&from_zip, &from_json),
            "byte-mode analyze estimate envelope diverged from inline-inputs envelope\nzip: {from_zip}\njson: {from_json}"
        );
    }

    #[wasm_bindgen_test]
    fn run_lang_accepts_raw_json_string_args() {
        let args_json = r#"{
            "inputs": [
                { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }
            ],
            "files": true
        }"#;
        let data = run_lang(JsValue::from_str(args_json)).expect("lang data");
        let parsed = js_value_to_json(&data);

        assert_eq!(parsed["mode"], "lang");
        assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
    }

    #[cfg(feature = "analysis")]
    #[wasm_bindgen_test]
    fn run_rejects_unsupported_analyze_presets() {
        let err = run(
            "analyze",
            parse_js_args(
                r#"{
                    "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                    "preset": "health"
                }"#,
            ),
        )
        .expect_err("non-estimate preset should be rejected")
        .dyn_into::<JsError>()
        .expect("js error");

        let message = err.message().as_string().expect("js string message");
        assert!(message.contains("preset=\"receipt\""));
        assert!(message.contains("preset=\"estimate\""));
    }
}
