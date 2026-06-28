//! Byte-mode FFI entrypoint for archive (ZIP) uploads (`feature = "archive-zip"`).
//!
//! This is the transport seam named by `docs/specs/wasm-ffi-byte-mode.md`. It
//! lets a host hand raw archive bytes plus a small JSON options object into
//! tokmd and receive the same response envelope the JSON `{ path, text }` modes
//! return.
//!
//! Untrusted archive bytes are admitted fail-closed by the single authoritative
//! admission engine (`tokmd_scan::inputs_from_zip_bytes`, which delegates to
//! `tokmd_io_port::archive`), and the resulting ordered inputs are routed
//! through the existing mode dispatch. There is no second scan path and no
//! second admission path, so byte/JSON-mode parity holds by construction.

use serde_json::Value;
use tokmd_scan::{ArchiveLimits, inputs_from_zip_bytes};

use super::envelope::json_response;
use super::modes::run_mode;
use super::parse::{parse_optional_string, parse_optional_u64, parse_optional_usize};
use super::settings_parse::parse_scan_settings;
use crate::error::TokmdError;

/// Logical repository root the admitted archive entries are rooted under when
/// the caller omits the `root` option.
const DEFAULT_ARCHIVE_ROOT: &str = "repo";

/// Run a tokmd scan over an in-memory archive (currently ZIP) byte buffer.
///
/// This is the byte-mode counterpart to [`run_json`](super::run_json): instead
/// of `{ path, text }` inputs or host paths, the input source is the archive
/// `archive_bytes` buffer. The `options_json` object carries the same scan and
/// per-mode settings the JSON modes accept, plus two byte-mode-only options:
///
/// * `root` — the logical repository root admitted entries are rooted under
///   (default `"repo"`).
/// * `archive_limits` — an optional object overriding any of the conservative
///   ingestion caps (`max_entry_size`, `max_total_size`, `max_entries`,
///   `max_ratio`); omitted fields keep the engine defaults.
///
/// # Arguments
///
/// * `mode` — one of `"lang"`, `"module"`, `"export"`, or `"analyze"`. Host-only
///   modes (`diff`/`cockpit`/`version`) have no archive meaning and are rejected.
/// * `options_json` — JSON options object (must not carry `inputs` or `paths`).
/// * `archive_bytes` — the raw archive buffer, copied/owned by the caller.
///
/// # Returns
///
/// The same envelope shape [`run_json`](super::run_json) returns:
/// `{"ok": true, "data": {...receipt...}}` on success, or
/// `{"ok": false, "error": {...}}` on failure. A rejected archive fails closed
/// with no partial receipt.
///
/// # Example
///
/// ```ignore
/// use tokmd_core::ffi::run_json_bytes;
///
/// let result = run_json_bytes("lang", r#"{"lang": {"files": true}}"#, &zip_bytes);
/// let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
/// assert_eq!(parsed["data"]["mode"], "lang");
/// ```
pub fn run_json_bytes(mode: &str, options_json: &str, archive_bytes: &[u8]) -> String {
    json_response(run_json_bytes_inner(mode, options_json, archive_bytes))
}

fn run_json_bytes_inner(
    mode: &str,
    options_json: &str,
    archive_bytes: &[u8],
) -> Result<Value, TokmdError> {
    let args: Value = serde_json::from_str(options_json)
        .map_err(|err| TokmdError::invalid_json(err.to_string()))?;
    if !args.is_object() {
        return Err(TokmdError::invalid_json(
            "Top-level JSON value must be an object",
        ));
    }

    // The archive is the sole input source. The `{ path, text }` and host-path
    // conventions are mutually exclusive with byte mode, mirroring the existing
    // "paths cannot be combined with in-memory inputs" rule in ffi/inputs.rs.
    if args.get("inputs").is_some_and(|value| !value.is_null()) {
        return Err(TokmdError::invalid_field(
            "inputs",
            "must not be combined with archive bytes; the archive is the input source",
        ));
    }
    if args.get("paths").is_some_and(|value| !value.is_null()) {
        return Err(TokmdError::invalid_field(
            "paths",
            "cannot be combined with archive bytes",
        ));
    }

    // Byte mode only serves the input-consuming scan modes.
    if !matches!(mode, "lang" | "module" | "export" | "analyze") {
        return Err(TokmdError::invalid_field(
            "mode",
            "one of 'lang', 'module', 'export', or 'analyze' for archive byte mode",
        ));
    }

    let root =
        parse_optional_string(&args, "root")?.unwrap_or_else(|| DEFAULT_ARCHIVE_ROOT.to_string());
    let limits = parse_archive_limits(&args)?;

    // Untrusted bytes cross the trust boundary here: all path-safety and
    // zip-bomb admission is delegated to the single authoritative engine, which
    // fails closed on the first violated entry.
    let inputs = inputs_from_zip_bytes(&root, archive_bytes, &limits)?;

    let scan = parse_scan_settings(&args)?;
    run_mode(mode, &args, &scan, Some(&inputs))
}

/// Build [`ArchiveLimits`] from an optional `archive_limits` options object,
/// keeping the conservative engine defaults for any omitted field.
fn parse_archive_limits(args: &Value) -> Result<ArchiveLimits, TokmdError> {
    let defaults = ArchiveLimits::default();
    let obj = match args.get("archive_limits") {
        None | Some(Value::Null) => return Ok(defaults),
        Some(value) if value.is_object() => value,
        Some(_) => return Err(TokmdError::invalid_field("archive_limits", "a JSON object")),
    };

    Ok(ArchiveLimits {
        max_entry_size: parse_optional_u64(obj, "max_entry_size")?
            .unwrap_or(defaults.max_entry_size),
        max_total_size: parse_optional_u64(obj, "max_total_size")?
            .unwrap_or(defaults.max_total_size),
        max_entries: parse_optional_usize(obj, "max_entries")?.unwrap_or(defaults.max_entries),
        max_ratio: parse_optional_u64(obj, "max_ratio")?.unwrap_or(defaults.max_ratio),
    })
}
