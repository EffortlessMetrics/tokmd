//! CSV file-level export rendering.
//!
//! This module owns the CSV header and row serialization for export receipts.
//! The parent export module keeps dispatch and the public test-helper facade
//! stable.

use std::io::Write;

use anyhow::Result;

use tokmd_types::{ExportArgs, ExportData, FileKind};

use super::redact_rows;

pub(super) fn write_export_csv<W: Write>(
    out: &mut W,
    export: &ExportData,
    args: &ExportArgs,
) -> Result<()> {
    let mut wtr = ::csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(out);
    wtr.write_record([
        "path", "module", "lang", "kind", "code", "comments", "blanks", "lines", "bytes", "tokens",
    ])?;

    for r in redact_rows(&export.rows, args.redact) {
        let code = r.code.to_string();
        let comments = r.comments.to_string();
        let blanks = r.blanks.to_string();
        let lines = r.lines.to_string();
        let bytes = r.bytes.to_string();
        let tokens = r.tokens.to_string();
        let kind = match r.kind {
            FileKind::Parent => "parent",
            FileKind::Child => "child",
        };

        wtr.write_record([
            r.path.as_str(),
            r.module.as_str(),
            r.lang.as_str(),
            kind,
            &code,
            &comments,
            &blanks,
            &lines,
            &bytes,
            &tokens,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}
