use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum CliTableFormat {
    /// Markdown table (great for pasting into ChatGPT).
    Md,
    /// Tab-separated values (good for piping to other tools).
    Tsv,
    /// JSON (compact).
    Json,
}

impl From<CliTableFormat> for tokmd_types::TableFormat {
    fn from(v: CliTableFormat) -> Self {
        match v {
            CliTableFormat::Md => tokmd_types::TableFormat::Md,
            CliTableFormat::Tsv => tokmd_types::TableFormat::Tsv,
            CliTableFormat::Json => tokmd_types::TableFormat::Json,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum CliExportFormat {
    /// CSV with a header row.
    Csv,
    /// One JSON object per line.
    Jsonl,
    /// A single JSON array.
    Json,
    /// CycloneDX 1.6 JSON SBOM format.
    Cyclonedx,
}

impl From<CliExportFormat> for tokmd_types::ExportFormat {
    fn from(v: CliExportFormat) -> Self {
        match v {
            CliExportFormat::Csv => tokmd_types::ExportFormat::Csv,
            CliExportFormat::Jsonl => tokmd_types::ExportFormat::Jsonl,
            CliExportFormat::Json => tokmd_types::ExportFormat::Json,
            CliExportFormat::Cyclonedx => tokmd_types::ExportFormat::Cyclonedx,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CliConfigMode {
    /// Read scan config files (`tokei.toml` / `.tokeirc`) if present.
    #[default]
    Auto,
    /// Ignore config files.
    None,
}

impl From<CliConfigMode> for tokmd_types::ConfigMode {
    fn from(v: CliConfigMode) -> Self {
        match v {
            CliConfigMode::Auto => tokmd_types::ConfigMode::Auto,
            CliConfigMode::None => tokmd_types::ConfigMode::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum CliChildrenMode {
    /// Merge embedded content into the parent language totals.
    Collapse,
    /// Show embedded languages as separate "(embedded)" rows.
    Separate,
}

impl From<CliChildrenMode> for tokmd_types::ChildrenMode {
    fn from(v: CliChildrenMode) -> Self {
        match v {
            CliChildrenMode::Collapse => tokmd_types::ChildrenMode::Collapse,
            CliChildrenMode::Separate => tokmd_types::ChildrenMode::Separate,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum CliChildIncludeMode {
    /// Include embedded languages as separate contributions.
    Separate,
    /// Ignore embedded languages.
    ParentsOnly,
}

impl From<CliChildIncludeMode> for tokmd_types::ChildIncludeMode {
    fn from(v: CliChildIncludeMode) -> Self {
        match v {
            CliChildIncludeMode::Separate => tokmd_types::ChildIncludeMode::Separate,
            CliChildIncludeMode::ParentsOnly => tokmd_types::ChildIncludeMode::ParentsOnly,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum CliRedactMode {
    /// Do not redact.
    None,
    /// Redact file paths.
    Paths,
    /// Redact file paths and module names.
    All,
}

impl From<CliRedactMode> for tokmd_types::RedactMode {
    fn from(v: CliRedactMode) -> Self {
        match v {
            CliRedactMode::None => tokmd_types::RedactMode::None,
            CliRedactMode::Paths => tokmd_types::RedactMode::Paths,
            CliRedactMode::All => tokmd_types::RedactMode::All,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum CliAnalysisFormat {
    /// Markdown report.
    Md,
    /// JSON receipt.
    Json,
    /// JSON-LD document.
    Jsonld,
    /// XML document.
    Xml,
    /// SVG graphic.
    Svg,
    /// Mermaid diagram.
    Mermaid,
    /// Wavefront OBJ export.
    Obj,
    /// MIDI export.
    Midi,
    /// Text tree output.
    Tree,
    /// HTML report.
    Html,
}

impl From<CliAnalysisFormat> for tokmd_types::AnalysisFormat {
    fn from(v: CliAnalysisFormat) -> Self {
        match v {
            CliAnalysisFormat::Md => tokmd_types::AnalysisFormat::Md,
            CliAnalysisFormat::Json => tokmd_types::AnalysisFormat::Json,
            CliAnalysisFormat::Jsonld => tokmd_types::AnalysisFormat::Jsonld,
            CliAnalysisFormat::Xml => tokmd_types::AnalysisFormat::Xml,
            CliAnalysisFormat::Svg => tokmd_types::AnalysisFormat::Svg,
            CliAnalysisFormat::Mermaid => tokmd_types::AnalysisFormat::Mermaid,
            CliAnalysisFormat::Obj => tokmd_types::AnalysisFormat::Obj,
            CliAnalysisFormat::Midi => tokmd_types::AnalysisFormat::Midi,
            CliAnalysisFormat::Tree => tokmd_types::AnalysisFormat::Tree,
            CliAnalysisFormat::Html => tokmd_types::AnalysisFormat::Html,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_config_mode_defaults_to_auto() {
        assert_eq!(CliConfigMode::default(), CliConfigMode::Auto);
    }

    #[test]
    fn cli_table_format_maps_to_core_type() {
        assert_eq!(tokmd_types::TableFormat::from(CliTableFormat::Md), tokmd_types::TableFormat::Md);
        assert_eq!(tokmd_types::TableFormat::from(CliTableFormat::Tsv), tokmd_types::TableFormat::Tsv);
        assert_eq!(tokmd_types::TableFormat::from(CliTableFormat::Json), tokmd_types::TableFormat::Json);
    }

    #[test]
    fn cli_export_format_maps_to_core_type() {
        assert_eq!(
            tokmd_types::ExportFormat::from(CliExportFormat::Cyclonedx),
            tokmd_types::ExportFormat::Cyclonedx
        );
        assert_eq!(tokmd_types::ExportFormat::from(CliExportFormat::Csv), tokmd_types::ExportFormat::Csv);
    }

    #[test]
    fn cli_analysis_format_maps_to_core_type() {
        assert_eq!(tokmd_types::AnalysisFormat::from(CliAnalysisFormat::Md), tokmd_types::AnalysisFormat::Md);
        assert_eq!(tokmd_types::AnalysisFormat::from(CliAnalysisFormat::Html), tokmd_types::AnalysisFormat::Html);
        assert_eq!(
            tokmd_types::AnalysisFormat::from(CliAnalysisFormat::Mermaid),
            tokmd_types::AnalysisFormat::Mermaid
        );
    }

    #[test]
    fn cli_enum_serialization_uses_kebab_case() {
        assert_eq!(
            serde_json::to_string(&CliChildIncludeMode::ParentsOnly).expect("serializes"),
            "\"parents-only\""
        );
        assert_eq!(serde_json::to_string(&CliConfigMode::Auto).expect("serializes"), "\"auto\"");
    }

    #[test]
    fn cli_enum_deserialization_accepts_kebab_case() {
        assert_eq!(
            serde_json::from_str::<CliRedactMode>("\"paths\"").expect("deserializes"),
            CliRedactMode::Paths
        );
        assert_eq!(
            serde_json::from_str::<CliAnalysisFormat>("\"jsonld\"").expect("deserializes"),
            CliAnalysisFormat::Jsonld
        );
    }
}
