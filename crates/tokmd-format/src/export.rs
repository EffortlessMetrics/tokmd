use std::borrow::Cow;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use tokmd_settings::ScanOptions;
use tokmd_types::{
    ExportArgs, ExportArgsMeta, ExportData, ExportFormat, ExportReceipt, FileKind, FileRow,
    RedactMode, ScanArgs, ScanStatus, ToolInfo,
};

use crate::{now_ms, redact_module_roots, redact_path, scan_args, short_hash};

// -----------------
// Export (datasets)
// -----------------

mod cyclonedx;

use cyclonedx::{write_export_cyclonedx, write_export_cyclonedx_impl};

#[derive(Debug, Clone, Serialize)]
struct ExportMeta {
    #[serde(rename = "type")]
    ty: &'static str,
    schema_version: u32,
    generated_at_ms: u128,
    tool: ToolInfo,
    mode: String,
    status: ScanStatus,
    warnings: Vec<String>,
    scan: ScanArgs,
    args: ExportArgsMeta,
}

#[derive(Debug, Clone, Serialize)]
struct JsonlRow<'a> {
    #[serde(rename = "type")]
    ty: &'static str,
    #[serde(flatten)]
    row: &'a FileRow,
}

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

fn write_export_csv<W: Write>(out: &mut W, export: &ExportData, args: &ExportArgs) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new().has_headers(true).from_writer(out);
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

fn write_export_jsonl<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    let module_roots = redact_module_roots(&export.module_roots, args.redact);

    if args.meta {
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();

        let meta = ExportMeta {
            ty: "meta",
            schema_version: tokmd_types::SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: ToolInfo::current(),
            mode: "export".to_string(),
            status: ScanStatus::Complete,
            warnings: vec![],
            scan: scan_args(&args.paths, global, Some(args.redact)),
            args: ExportArgsMeta {
                format: args.format,
                module_roots: module_roots.clone(),
                module_depth: export.module_depth,
                children: export.children,
                min_code: args.min_code,
                max_rows: args.max_rows,
                redact: args.redact,
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| p.display().to_string().replace('\\', "/"))
                },
                strip_prefix_redacted,
            },
        };
        writeln!(out, "{}", serde_json::to_string(&meta)?)?;
    }

    for row in redact_rows(&export.rows, args.redact) {
        let wrapper = JsonlRow {
            ty: "row",
            row: &row,
        };
        writeln!(out, "{}", serde_json::to_string(&wrapper)?)?;
    }
    Ok(())
}

fn write_export_json<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
    let module_roots = redact_module_roots(&export.module_roots, args.redact);

    if args.meta {
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();

        let receipt = ExportReceipt {
            schema_version: tokmd_types::SCHEMA_VERSION,
            generated_at_ms: now_ms(),
            tool: ToolInfo::current(),
            mode: "export".to_string(),
            status: ScanStatus::Complete,
            warnings: vec![],
            scan: scan_args(&args.paths, global, Some(args.redact)),
            args: ExportArgsMeta {
                format: args.format,
                module_roots: module_roots.clone(),
                module_depth: export.module_depth,
                children: export.children,
                min_code: args.min_code,
                max_rows: args.max_rows,
                redact: args.redact,
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| p.display().to_string().replace('\\', "/"))
                },
                strip_prefix_redacted,
            },
            data: ExportData {
                rows: redact_rows(&export.rows, args.redact)
                    .map(|c| c.into_owned())
                    .collect(),
                module_roots: module_roots.clone(),
                module_depth: export.module_depth,
                children: export.children,
            },
        };
        writeln!(out, "{}", serde_json::to_string(&receipt)?)?;
    } else {
        writeln!(
            out,
            "{}",
            serde_json::to_string(&redact_rows(&export.rows, args.redact).collect::<Vec<_>>())?
        )?;
    }
    Ok(())
}

fn redact_rows(rows: &[FileRow], mode: RedactMode) -> impl Iterator<Item = Cow<'_, FileRow>> {
    rows.iter().map(move |r| match mode {
        RedactMode::None => Cow::Borrowed(r),
        RedactMode::Paths => Cow::Owned(FileRow {
            path: redact_path(&r.path),
            module: r.module.clone(),
            lang: r.lang.clone(),
            kind: r.kind,
            code: r.code,
            comments: r.comments,
            blanks: r.blanks,
            lines: r.lines,
            bytes: r.bytes,
            tokens: r.tokens,
        }),
        RedactMode::All => Cow::Owned(FileRow {
            path: redact_path(&r.path),
            module: short_hash(&r.module),
            lang: r.lang.clone(),
            kind: r.kind,
            code: r.code,
            comments: r.comments,
            blanks: r.blanks,
            lines: r.lines,
            bytes: r.bytes,
            tokens: r.tokens,
        }),
    })
}

/// Write export data as JSONL to a file path.
///
/// This is a convenience function for the `run` command that accepts
/// pre-constructed `ScanArgs` and `ExportArgsMeta` rather than requiring
/// the full `ScanOptions` and `ExportArgs` structs.
pub fn write_export_jsonl_to_file(
    path: &Path,
    export: &ExportData,
    scan: &ScanArgs,
    args_meta: &ExportArgsMeta,
) -> Result<()> {
    let file = File::create(path)?;
    let mut out = BufWriter::new(file);

    let mut final_args = args_meta.clone();
    final_args.module_roots = redact_module_roots(&final_args.module_roots, args_meta.redact);

    let meta = ExportMeta {
        ty: "meta",
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan.clone(),
        args: final_args,
    };
    writeln!(out, "{}", serde_json::to_string(&meta)?)?;

    for row in redact_rows(&export.rows, args_meta.redact) {
        let wrapper = JsonlRow {
            ty: "row",
            row: &row,
        };
        writeln!(out, "{}", serde_json::to_string(&wrapper)?)?;
    }

    out.flush()?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    fn sample_file_rows() -> Vec<FileRow> {
        vec![
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 10,
                lines: 130,
                bytes: 1000,
                tokens: 250,
            },
            FileRow {
                path: "tests/test.rs".to_string(),
                module: "tests".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 500,
                tokens: 125,
            },
        ]
    }

    // ========================
    // Redaction Tests
    // ========================

    #[test]
    fn redact_rows_none_mode() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::None).collect();

        // Should be identical
        assert_eq!(redacted.len(), rows.len());
        assert_eq!(redacted[0].path, "src/lib.rs");
        assert_eq!(redacted[0].module, "src");
    }

    #[test]
    fn redact_rows_paths_mode() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();

        // Paths should be redacted (16 char hash + extension)
        assert_ne!(redacted[0].path, "src/lib.rs");
        assert!(redacted[0].path.ends_with(".rs"));
        assert_eq!(redacted[0].path.len(), 16 + 3); // hash + ".rs"

        // Module should NOT be redacted
        assert_eq!(redacted[0].module, "src");
    }

    #[test]
    fn redact_rows_all_mode() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::All).collect();

        // Paths should be redacted
        assert_ne!(redacted[0].path, "src/lib.rs");
        assert!(redacted[0].path.ends_with(".rs"));

        // Module should ALSO be redacted (16 char hash)
        assert_ne!(redacted[0].module, "src");
        assert_eq!(redacted[0].module.len(), 16);
    }

    #[test]
    fn redact_rows_preserves_other_fields() {
        let rows = sample_file_rows();
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::All).collect();

        // All other fields should be preserved
        assert_eq!(redacted[0].lang, "Rust");
        assert_eq!(redacted[0].kind, FileKind::Parent);
        assert_eq!(redacted[0].code, 100);
        assert_eq!(redacted[0].comments, 20);
        assert_eq!(redacted[0].blanks, 10);
        assert_eq!(redacted[0].lines, 130);
        assert_eq!(redacted[0].bytes, 1000);
        assert_eq!(redacted[0].tokens, 250);
    }

    proptest! {
        #[test]
        fn redact_rows_preserves_count(
            code in 0usize..10000,
            comments in 0usize..1000,
            blanks in 0usize..500
        ) {
            let rows = vec![FileRow {
                path: "test/file.rs".to_string(),
                module: "test".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code,
                comments,
                blanks,
                lines: code + comments + blanks,
                bytes: 1000,
                tokens: 250,
            }];

            for mode in [RedactMode::None, RedactMode::Paths, RedactMode::All] {
                let redacted: Vec<_> = redact_rows(&rows, mode).collect();
                prop_assert_eq!(redacted.len(), 1);
                prop_assert_eq!(redacted[0].code, code);
                prop_assert_eq!(redacted[0].comments, comments);
                prop_assert_eq!(redacted[0].blanks, blanks);
            }
        }

        #[test]
        fn redact_rows_paths_preserve_allowlisted_extensions(ext in "rs|js|ts|json|md|toml|gz") {
            let path = format!("some/path/file.{}", ext);
            let rows = vec![FileRow {
                path: path.clone(),
                module: "some".to_string(),
                lang: "Test".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 115,
                bytes: 1000,
                tokens: 250,
            }];

            let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();
            prop_assert!(redacted[0].path.ends_with(&format!(".{}", ext)),
                "Redacted path '{}' should end with .{}", redacted[0].path, ext);
        }

        #[test]
        fn redact_rows_paths_strip_untrusted_extensions(ext in "passwd|secret|pass1234|token") {
            let path = format!("some/path/file.{}", ext);
            let rows = vec![FileRow {
                path: path.clone(),
                module: "some".to_string(),
                lang: "Test".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 115,
                bytes: 1000,
                tokens: 250,
            }];

            let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();
            prop_assert_eq!(redacted[0].path.len(), 16);
            prop_assert!(!redacted[0].path.contains('.'));
            prop_assert!(!redacted[0].path.contains(&ext));
        }
    }
}
