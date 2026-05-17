//! Shared helpers for export receipt metadata and redacted payloads.
//!
//! JSON and JSONL exports both need the same envelope fields and redaction
//! decisions. Keeping that logic here prevents the two wire formats from
//! drifting apart.

use std::{borrow::Cow, path::Path};

use tokmd_types::{ExportArgs, ExportArgsMeta, ExportData, FileRow, RedactMode};

use crate::{redact_module_roots, redact_path};

use super::redact_rows;

pub(super) fn args_meta_from_export(export: &ExportData, args: &ExportArgs) -> ExportArgsMeta {
    let should_redact = redact_paths(args.redact);

    ExportArgsMeta {
        format: args.format,
        module_roots: redact_module_roots(&export.module_roots, args.redact),
        module_depth: export.module_depth,
        children: export.children,
        min_code: args.min_code,
        max_rows: args.max_rows,
        redact: args.redact,
        strip_prefix: args
            .strip_prefix
            .as_deref()
            .map(|path| normalized_strip_prefix(path, should_redact)),
        strip_prefix_redacted: should_redact && args.strip_prefix.is_some(),
    }
}

pub(super) fn redacted_args_meta(args_meta: &ExportArgsMeta) -> ExportArgsMeta {
    let mut final_args = args_meta.clone();
    final_args.module_roots = redact_module_roots(&final_args.module_roots, args_meta.redact);
    final_args
}

pub(super) fn redacted_export_data(export: &ExportData, redact: RedactMode) -> ExportData {
    ExportData {
        rows: redacted_rows_for_json(export, redact)
            .into_iter()
            .map(|row| row.into_owned())
            .collect(),
        module_roots: redact_module_roots(&export.module_roots, redact),
        module_depth: export.module_depth,
        children: export.children,
    }
}

pub(super) fn redacted_rows_for_json(
    export: &ExportData,
    redact: RedactMode,
) -> Vec<Cow<'_, FileRow>> {
    redact_rows(&export.rows, redact).collect()
}

fn normalized_strip_prefix(path: &Path, should_redact: bool) -> String {
    let normalized = path.display().to_string().replace('\\', "/");
    if should_redact {
        redact_path(&normalized)
    } else {
        normalized
    }
}

fn redact_paths(redact: RedactMode) -> bool {
    redact == RedactMode::Paths || redact == RedactMode::All
}
