//! Shared clap value enums used by multiple command parser modules.
//!
//! This module owns CLI-facing enum spelling and conversions to the receipt
//! contract types in `tokmd-types`.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TableFormat {
    /// Markdown table (great for pasting into ChatGPT).
    Md,
    /// Tab-separated values (good for piping to other tools).
    Tsv,
    /// JSON (compact).
    Json,
}

impl From<TableFormat> for tokmd_types::TableFormat {
    fn from(value: TableFormat) -> Self {
        match value {
            TableFormat::Md => Self::Md,
            TableFormat::Tsv => Self::Tsv,
            TableFormat::Json => Self::Json,
        }
    }
}

impl From<tokmd_types::TableFormat> for TableFormat {
    fn from(value: tokmd_types::TableFormat) -> Self {
        match value {
            tokmd_types::TableFormat::Md => Self::Md,
            tokmd_types::TableFormat::Tsv => Self::Tsv,
            tokmd_types::TableFormat::Json => Self::Json,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    /// CSV with a header row.
    Csv,
    /// One JSON object per line.
    Jsonl,
    /// A single JSON array.
    Json,
    /// CycloneDX 1.6 JSON SBOM format.
    Cyclonedx,
}

impl From<ExportFormat> for tokmd_types::ExportFormat {
    fn from(value: ExportFormat) -> Self {
        match value {
            ExportFormat::Csv => Self::Csv,
            ExportFormat::Jsonl => Self::Jsonl,
            ExportFormat::Json => Self::Json,
            ExportFormat::Cyclonedx => Self::Cyclonedx,
        }
    }
}

impl From<tokmd_types::ExportFormat> for ExportFormat {
    fn from(value: tokmd_types::ExportFormat) -> Self {
        match value {
            tokmd_types::ExportFormat::Csv => Self::Csv,
            tokmd_types::ExportFormat::Jsonl => Self::Jsonl,
            tokmd_types::ExportFormat::Json => Self::Json,
            tokmd_types::ExportFormat::Cyclonedx => Self::Cyclonedx,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigMode {
    /// Read scan config files (`tokei.toml` / `.tokeirc`) if present.
    #[default]
    Auto,
    /// Ignore config files.
    None,
}

impl From<ConfigMode> for tokmd_types::ConfigMode {
    fn from(value: ConfigMode) -> Self {
        match value {
            ConfigMode::Auto => Self::Auto,
            ConfigMode::None => Self::None,
        }
    }
}

impl From<tokmd_types::ConfigMode> for ConfigMode {
    fn from(value: tokmd_types::ConfigMode) -> Self {
        match value {
            tokmd_types::ConfigMode::Auto => Self::Auto,
            tokmd_types::ConfigMode::None => Self::None,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildrenMode {
    /// Merge embedded content into the parent language totals.
    Collapse,
    /// Show embedded languages as separate "(embedded)" rows.
    Separate,
}

impl From<ChildrenMode> for tokmd_types::ChildrenMode {
    fn from(value: ChildrenMode) -> Self {
        match value {
            ChildrenMode::Collapse => Self::Collapse,
            ChildrenMode::Separate => Self::Separate,
        }
    }
}

impl From<tokmd_types::ChildrenMode> for ChildrenMode {
    fn from(value: tokmd_types::ChildrenMode) -> Self {
        match value {
            tokmd_types::ChildrenMode::Collapse => Self::Collapse,
            tokmd_types::ChildrenMode::Separate => Self::Separate,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildIncludeMode {
    /// Include embedded languages as separate contributions.
    Separate,
    /// Ignore embedded languages.
    ParentsOnly,
}

impl From<ChildIncludeMode> for tokmd_types::ChildIncludeMode {
    fn from(value: ChildIncludeMode) -> Self {
        match value {
            ChildIncludeMode::Separate => Self::Separate,
            ChildIncludeMode::ParentsOnly => Self::ParentsOnly,
        }
    }
}

impl From<tokmd_types::ChildIncludeMode> for ChildIncludeMode {
    fn from(value: tokmd_types::ChildIncludeMode) -> Self {
        match value {
            tokmd_types::ChildIncludeMode::Separate => Self::Separate,
            tokmd_types::ChildIncludeMode::ParentsOnly => Self::ParentsOnly,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RedactMode {
    /// Do not redact.
    None,
    /// Redact file paths.
    Paths,
    /// Redact file paths and module names.
    All,
}

impl From<RedactMode> for tokmd_types::RedactMode {
    fn from(value: RedactMode) -> Self {
        match value {
            RedactMode::None => Self::None,
            RedactMode::Paths => Self::Paths,
            RedactMode::All => Self::All,
        }
    }
}

impl From<tokmd_types::RedactMode> for RedactMode {
    fn from(value: tokmd_types::RedactMode) -> Self {
        match value {
            tokmd_types::RedactMode::None => Self::None,
            tokmd_types::RedactMode::Paths => Self::Paths,
            tokmd_types::RedactMode::All => Self::All,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnalysisFormat {
    Md,
    Json,
    Jsonld,
    Xml,
    Svg,
    Mermaid,
    Obj,
    Midi,
    Tree,
    Html,
}

impl From<AnalysisFormat> for tokmd_types::AnalysisFormat {
    fn from(value: AnalysisFormat) -> Self {
        match value {
            AnalysisFormat::Md => Self::Md,
            AnalysisFormat::Json => Self::Json,
            AnalysisFormat::Jsonld => Self::Jsonld,
            AnalysisFormat::Xml => Self::Xml,
            AnalysisFormat::Svg => Self::Svg,
            AnalysisFormat::Mermaid => Self::Mermaid,
            AnalysisFormat::Obj => Self::Obj,
            AnalysisFormat::Midi => Self::Midi,
            AnalysisFormat::Tree => Self::Tree,
            AnalysisFormat::Html => Self::Html,
        }
    }
}

impl From<tokmd_types::AnalysisFormat> for AnalysisFormat {
    fn from(value: tokmd_types::AnalysisFormat) -> Self {
        match value {
            tokmd_types::AnalysisFormat::Md => Self::Md,
            tokmd_types::AnalysisFormat::Json => Self::Json,
            tokmd_types::AnalysisFormat::Jsonld => Self::Jsonld,
            tokmd_types::AnalysisFormat::Xml => Self::Xml,
            tokmd_types::AnalysisFormat::Svg => Self::Svg,
            tokmd_types::AnalysisFormat::Mermaid => Self::Mermaid,
            tokmd_types::AnalysisFormat::Obj => Self::Obj,
            tokmd_types::AnalysisFormat::Midi => Self::Midi,
            tokmd_types::AnalysisFormat::Tree => Self::Tree,
            tokmd_types::AnalysisFormat::Html => Self::Html,
        }
    }
}
