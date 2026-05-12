use clap::ValueEnum;

use crate::cli;

pub(super) fn parse_table_format(value: Option<&str>) -> Option<tokmd_types::TableFormat> {
    value
        .and_then(|s| cli::TableFormat::from_str(s, true).ok())
        .map(Into::into)
}

pub(super) fn parse_children_mode(value: Option<&str>) -> Option<tokmd_types::ChildrenMode> {
    value
        .and_then(|s| cli::ChildrenMode::from_str(s, true).ok())
        .map(Into::into)
}

pub(super) fn parse_child_include_mode(
    value: Option<&str>,
) -> Option<tokmd_types::ChildIncludeMode> {
    value
        .and_then(|s| cli::ChildIncludeMode::from_str(s, true).ok())
        .map(Into::into)
}

pub(super) fn parse_export_format(value: Option<&str>) -> Option<tokmd_types::ExportFormat> {
    value
        .and_then(|s| cli::ExportFormat::from_str(s, true).ok())
        .map(Into::into)
}

pub(super) fn parse_redact_mode(value: Option<&str>) -> Option<tokmd_types::RedactMode> {
    value
        .and_then(|s| cli::RedactMode::from_str(s, true).ok())
        .map(Into::into)
}
