//! CycloneDX SBOM export rendering.
//!
//! This module owns the CycloneDX data transfer shape and JSON writer for
//! file-level export rows. The parent export module keeps command dispatch and
//! shared row redaction.

use std::io::Write;

use anyhow::Result;
use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use tokmd_types::{ExportData, FileKind, RedactMode};

use super::redact_rows;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CycloneDxBom {
    bom_format: &'static str,
    spec_version: &'static str,
    serial_number: String,
    version: u32,
    metadata: CycloneDxMetadata,
    components: Vec<CycloneDxComponent>,
}

#[derive(Debug, Clone, Serialize)]
struct CycloneDxMetadata {
    timestamp: String,
    tools: Vec<CycloneDxTool>,
}

#[derive(Debug, Clone, Serialize)]
struct CycloneDxTool {
    vendor: &'static str,
    name: &'static str,
    version: String,
}

#[derive(Debug, Clone, Serialize)]
struct CycloneDxComponent {
    #[serde(rename = "type")]
    ty: &'static str,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    properties: Vec<CycloneDxProperty>,
}

#[derive(Debug, Clone, Serialize)]
struct CycloneDxProperty {
    name: &'static str,
    value: String,
}

pub(super) fn write_export_cyclonedx<W: Write>(
    out: &mut W,
    export: &ExportData,
    redact: RedactMode,
) -> Result<()> {
    write_export_cyclonedx_impl(out, export, redact, None, None)
}

pub(super) fn write_export_cyclonedx_impl<W: Write>(
    out: &mut W,
    export: &ExportData,
    redact: RedactMode,
    serial_number: Option<String>,
    timestamp: Option<String>,
) -> Result<()> {
    let timestamp = timestamp.unwrap_or_else(|| {
        OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
    });

    let components: Vec<CycloneDxComponent> = redact_rows(&export.rows, redact)
        .map(|row| {
            let mut properties = vec![
                CycloneDxProperty {
                    name: "tokmd:lang",
                    value: row.lang.clone(),
                },
                CycloneDxProperty {
                    name: "tokmd:code",
                    value: row.code.to_string(),
                },
                CycloneDxProperty {
                    name: "tokmd:comments",
                    value: row.comments.to_string(),
                },
                CycloneDxProperty {
                    name: "tokmd:blanks",
                    value: row.blanks.to_string(),
                },
                CycloneDxProperty {
                    name: "tokmd:lines",
                    value: row.lines.to_string(),
                },
                CycloneDxProperty {
                    name: "tokmd:bytes",
                    value: row.bytes.to_string(),
                },
                CycloneDxProperty {
                    name: "tokmd:tokens",
                    value: row.tokens.to_string(),
                },
            ];

            if row.kind == FileKind::Child {
                properties.push(CycloneDxProperty {
                    name: "tokmd:kind",
                    value: "child".to_string(),
                });
            }

            CycloneDxComponent {
                ty: "file",
                name: row.path.clone(),
                group: if row.module.is_empty() {
                    None
                } else {
                    Some(row.module.clone())
                },
                properties,
            }
        })
        .collect();

    let bom = CycloneDxBom {
        bom_format: "CycloneDX",
        spec_version: "1.6",
        serial_number: serial_number
            .unwrap_or_else(|| format!("urn:uuid:{}", uuid::Uuid::new_v4())),
        version: 1,
        metadata: CycloneDxMetadata {
            timestamp,
            tools: vec![CycloneDxTool {
                vendor: "tokmd",
                name: "tokmd",
                version: env!("CARGO_PKG_VERSION").to_string(),
            }],
        },
        components,
    };

    writeln!(out, "{}", serde_json::to_string_pretty(&bom)?)?;
    Ok(())
}
