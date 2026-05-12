use std::fs::File;
use std::io::{self, BufWriter, Write};

use anyhow::Result;

use tokmd_settings::ScanOptions;
use tokmd_types::{ExportArgs, ExportData, ExportFormat, RedactMode};

// -----------------
// Export (datasets)
// -----------------

mod csv;
mod cyclonedx;
mod json;
mod jsonl;
mod redact;

use csv::write_export_csv;
use cyclonedx::{write_export_cyclonedx, write_export_cyclonedx_impl};
use json::write_export_json;
use jsonl::write_export_jsonl;
use redact::redact_rows;

pub use jsonl::write_export_jsonl_to_file;

pub fn write_export(export: &ExportData, global: &ScanOptions, args: &ExportArgs) -> Result<()> {
    match &args.output {
        Some(path) => {
            let file = File::create(path)?;
            let mut out = BufWriter::new(file);
            write_export_to(&mut out, export, global, args)?;
            out.flush()?;
        }
        None => {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            write_export_to(&mut out, export, global, args)?;
            out.flush()?;
        }
    }
    Ok(())
}

fn write_export_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    match args.format {
        ExportFormat::Csv => write_export_csv(out, export, args),
        ExportFormat::Jsonl => write_export_jsonl(out, export, global, args),
        ExportFormat::Json => write_export_json(out, export, global, args),
        ExportFormat::Cyclonedx => write_export_cyclonedx(out, export, args.redact),
    }
}

// =============================================================================
// Public test helpers - expose internal functions for integration tests
// =============================================================================

/// Write CSV export to a writer (exposed for testing).
#[doc(hidden)]
pub fn write_export_csv_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    args: &ExportArgs,
) -> Result<()> {
    write_export_csv(out, export, args)
}

/// Write JSONL export to a writer (exposed for testing).
#[doc(hidden)]
pub fn write_export_jsonl_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    write_export_jsonl(out, export, global, args)
}

/// Write JSON export to a writer (exposed for testing).
#[doc(hidden)]
pub fn write_export_json_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    write_export_json(out, export, global, args)
}

/// Write CycloneDX export to a writer (exposed for testing).
#[doc(hidden)]
pub fn write_export_cyclonedx_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    redact: RedactMode,
) -> Result<()> {
    write_export_cyclonedx(out, export, redact)
}

/// Write CycloneDX export to a writer with explicit options (exposed for testing).
#[doc(hidden)]
pub fn write_export_cyclonedx_with_options<W: Write>(
    out: &mut W,
    export: &ExportData,
    redact: RedactMode,
    serial_number: Option<String>,
    timestamp: Option<String>,
) -> Result<()> {
    write_export_cyclonedx_impl(out, export, redact, serial_number, timestamp)
}
