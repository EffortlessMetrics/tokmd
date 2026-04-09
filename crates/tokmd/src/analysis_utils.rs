use std::io::IsTerminal;
use std::path::Path;

use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_analysis_grid::PresetKind;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;
use tokmd_core::analysis_facade::{RenderedOutput, render};

pub(crate) fn child_include_to_string(mode: tokmd_types::ChildIncludeMode) -> String {
    match mode {
        tokmd_types::ChildIncludeMode::Separate => "separate".to_string(),
        tokmd_types::ChildIncludeMode::ParentsOnly => "parents-only".to_string(),
    }
}

pub(crate) fn preset_to_string(preset: cli::AnalysisPreset) -> String {
    let key = format!("{:?}", preset).to_lowercase();
    PresetKind::from_str(&key)
        .map(|preset| preset.as_str().to_string())
        .unwrap_or(key)
}

pub(crate) fn format_to_string(format: tokmd_types::AnalysisFormat) -> String {
    match format {
        tokmd_types::AnalysisFormat::Md => "md".to_string(),
        tokmd_types::AnalysisFormat::Json => "json".to_string(),
        tokmd_types::AnalysisFormat::Jsonld => "jsonld".to_string(),
        tokmd_types::AnalysisFormat::Xml => "xml".to_string(),
        tokmd_types::AnalysisFormat::Svg => "svg".to_string(),
        tokmd_types::AnalysisFormat::Mermaid => "mermaid".to_string(),
        tokmd_types::AnalysisFormat::Obj => "obj".to_string(),
        tokmd_types::AnalysisFormat::Midi => "midi".to_string(),
        tokmd_types::AnalysisFormat::Tree => "tree".to_string(),
        tokmd_types::AnalysisFormat::Html => "html".to_string(),
    }
}

pub(crate) fn granularity_to_string(granularity: cli::ImportGranularity) -> String {
    match granularity {
        cli::ImportGranularity::Module => "module".to_string(),
        cli::ImportGranularity::File => "file".to_string(),
    }
}

pub(crate) fn map_preset(preset: cli::AnalysisPreset) -> analysis::AnalysisPreset {
    match preset {
        cli::AnalysisPreset::Receipt => analysis::AnalysisPreset::Receipt,
        cli::AnalysisPreset::Estimate => analysis::AnalysisPreset::Estimate,
        cli::AnalysisPreset::Health => analysis::AnalysisPreset::Health,
        cli::AnalysisPreset::Risk => analysis::AnalysisPreset::Risk,
        cli::AnalysisPreset::Supply => analysis::AnalysisPreset::Supply,
        cli::AnalysisPreset::Architecture => analysis::AnalysisPreset::Architecture,
        cli::AnalysisPreset::Topics => analysis::AnalysisPreset::Topics,
        cli::AnalysisPreset::Security => analysis::AnalysisPreset::Security,
        cli::AnalysisPreset::Identity => analysis::AnalysisPreset::Identity,
        cli::AnalysisPreset::Git => analysis::AnalysisPreset::Git,
        cli::AnalysisPreset::Deep => analysis::AnalysisPreset::Deep,
        cli::AnalysisPreset::Fun => analysis::AnalysisPreset::Fun,
    }
}

pub(crate) fn map_granularity(granularity: cli::ImportGranularity) -> analysis::ImportGranularity {
    match granularity {
        cli::ImportGranularity::Module => analysis::ImportGranularity::Module,
        cli::ImportGranularity::File => analysis::ImportGranularity::File,
    }
}

fn analysis_output_filename(format: tokmd_types::AnalysisFormat) -> &'static str {
    match format {
        tokmd_types::AnalysisFormat::Md => "analysis.md",
        tokmd_types::AnalysisFormat::Json => "analysis.json",
        tokmd_types::AnalysisFormat::Jsonld => "analysis.jsonld",
        tokmd_types::AnalysisFormat::Xml => "analysis.xml",
        tokmd_types::AnalysisFormat::Svg => "analysis.svg",
        tokmd_types::AnalysisFormat::Mermaid => "analysis.mmd",
        tokmd_types::AnalysisFormat::Obj => "analysis.obj",
        tokmd_types::AnalysisFormat::Midi => "analysis.mid",
        tokmd_types::AnalysisFormat::Tree => "analysis.tree.txt",
        tokmd_types::AnalysisFormat::Html => "analysis.html",
    }
}

pub(crate) fn write_analysis_output(
    receipt: &analysis_types::AnalysisReceipt,
    output_dir: &Path,
    format: tokmd_types::AnalysisFormat,
) -> Result<()> {
    let rendered = render(receipt, format)?;
    let out_path = output_dir.join(analysis_output_filename(format));
    match rendered {
        RenderedOutput::Text(text) => {
            std::fs::write(&out_path, text)?;
        }
        RenderedOutput::Binary(bytes) => {
            std::fs::write(&out_path, bytes)?;
        }
    }
    Ok(())
}

pub(crate) fn write_analysis_stdout(
    receipt: &analysis_types::AnalysisReceipt,
    format: tokmd_types::AnalysisFormat,
) -> Result<()> {
    let rendered = render(receipt, format)?;
    match rendered {
        RenderedOutput::Text(text) => {
            print!("{}", text);
        }
        RenderedOutput::Binary(bytes) => {
            if std::io::stdout().is_terminal() {
                anyhow::bail!(
                    "Refusing to write binary output (format: {:?}) to a terminal to prevent rendering garbage characters. Please redirect stdout to a file (e.g., `> output.bin`) or specify an output directory.",
                    format
                );
            }
            use std::io::Write;
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(&bytes)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_child_include_to_string_separate() {
        assert_eq!(
            child_include_to_string(tokmd_types::ChildIncludeMode::Separate),
            "separate"
        );
    }

    #[test]
    fn test_child_include_to_string_parents_only() {
        assert_eq!(
            child_include_to_string(tokmd_types::ChildIncludeMode::ParentsOnly),
            "parents-only"
        );
    }

    #[test]
    fn test_preset_to_string_all_variants() {
        assert_eq!(preset_to_string(cli::AnalysisPreset::Receipt), "receipt");
        assert_eq!(preset_to_string(cli::AnalysisPreset::Estimate), "estimate");
        assert_eq!(preset_to_string(cli::AnalysisPreset::Health), "health");
        assert_eq!(preset_to_string(cli::AnalysisPreset::Risk), "risk");
        assert_eq!(preset_to_string(cli::AnalysisPreset::Supply), "supply");
        assert_eq!(
            preset_to_string(cli::AnalysisPreset::Architecture),
            "architecture"
        );
        assert_eq!(preset_to_string(cli::AnalysisPreset::Topics), "topics");
        assert_eq!(preset_to_string(cli::AnalysisPreset::Security), "security");
        assert_eq!(preset_to_string(cli::AnalysisPreset::Identity), "identity");
        assert_eq!(preset_to_string(cli::AnalysisPreset::Git), "git");
        assert_eq!(preset_to_string(cli::AnalysisPreset::Deep), "deep");
        assert_eq!(preset_to_string(cli::AnalysisPreset::Fun), "fun");
    }

    #[test]
    fn test_format_to_string_all_variants() {
        assert_eq!(format_to_string(tokmd_types::AnalysisFormat::Md), "md");
        assert_eq!(format_to_string(tokmd_types::AnalysisFormat::Json), "json");
        assert_eq!(
            format_to_string(tokmd_types::AnalysisFormat::Jsonld),
            "jsonld"
        );
        assert_eq!(format_to_string(tokmd_types::AnalysisFormat::Xml), "xml");
        assert_eq!(format_to_string(tokmd_types::AnalysisFormat::Svg), "svg");
        assert_eq!(
            format_to_string(tokmd_types::AnalysisFormat::Mermaid),
            "mermaid"
        );
        assert_eq!(format_to_string(tokmd_types::AnalysisFormat::Obj), "obj");
        assert_eq!(format_to_string(tokmd_types::AnalysisFormat::Midi), "midi");
        assert_eq!(format_to_string(tokmd_types::AnalysisFormat::Tree), "tree");
        assert_eq!(format_to_string(tokmd_types::AnalysisFormat::Html), "html");
    }

    #[test]
    fn test_granularity_to_string() {
        assert_eq!(
            granularity_to_string(cli::ImportGranularity::Module),
            "module"
        );
        assert_eq!(granularity_to_string(cli::ImportGranularity::File), "file");
    }

    #[test]
    fn test_map_preset_all_variants() {
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Receipt),
            analysis::AnalysisPreset::Receipt
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Estimate),
            analysis::AnalysisPreset::Estimate
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Health),
            analysis::AnalysisPreset::Health
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Risk),
            analysis::AnalysisPreset::Risk
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Supply),
            analysis::AnalysisPreset::Supply
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Architecture),
            analysis::AnalysisPreset::Architecture
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Topics),
            analysis::AnalysisPreset::Topics
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Security),
            analysis::AnalysisPreset::Security
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Identity),
            analysis::AnalysisPreset::Identity
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Git),
            analysis::AnalysisPreset::Git
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Deep),
            analysis::AnalysisPreset::Deep
        ));
        assert!(matches!(
            map_preset(cli::AnalysisPreset::Fun),
            analysis::AnalysisPreset::Fun
        ));
    }

    #[test]
    fn test_map_granularity() {
        assert!(matches!(
            map_granularity(cli::ImportGranularity::Module),
            analysis::ImportGranularity::Module
        ));
        assert!(matches!(
            map_granularity(cli::ImportGranularity::File),
            analysis::ImportGranularity::File
        ));
    }

    #[test]
    fn test_analysis_output_filename() {
        assert_eq!(
            analysis_output_filename(tokmd_types::AnalysisFormat::Md),
            "analysis.md"
        );
        assert_eq!(
            analysis_output_filename(tokmd_types::AnalysisFormat::Json),
            "analysis.json"
        );
        assert_eq!(
            analysis_output_filename(tokmd_types::AnalysisFormat::Jsonld),
            "analysis.jsonld"
        );
        assert_eq!(
            analysis_output_filename(tokmd_types::AnalysisFormat::Xml),
            "analysis.xml"
        );
        assert_eq!(
            analysis_output_filename(tokmd_types::AnalysisFormat::Svg),
            "analysis.svg"
        );
        assert_eq!(
            analysis_output_filename(tokmd_types::AnalysisFormat::Mermaid),
            "analysis.mmd"
        );
        assert_eq!(
            analysis_output_filename(tokmd_types::AnalysisFormat::Obj),
            "analysis.obj"
        );
        assert_eq!(
            analysis_output_filename(tokmd_types::AnalysisFormat::Midi),
            "analysis.mid"
        );
        assert_eq!(
            analysis_output_filename(tokmd_types::AnalysisFormat::Tree),
            "analysis.tree.txt"
        );
        assert_eq!(
            analysis_output_filename(tokmd_types::AnalysisFormat::Html),
            "analysis.html"
        );
    }
}
