use std::path::Path;

use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_analysis_format as analysis_format;
use tokmd_analysis_grid::PresetKind;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;

pub(crate) fn child_include_to_string(mode: cli::ChildIncludeMode) -> String {
    match mode {
        cli::ChildIncludeMode::Separate => "separate".to_string(),
        cli::ChildIncludeMode::ParentsOnly => "parents-only".to_string(),
    }
}

pub(crate) fn preset_to_string(preset: cli::AnalysisPreset) -> String {
    let key = format!("{:?}", preset).to_lowercase();
    PresetKind::from_str(&key)
        .map(|preset| preset.as_str().to_string())
        .unwrap_or(key)
}

pub(crate) fn format_to_string(format: cli::AnalysisFormat) -> String {
    match format {
        cli::AnalysisFormat::Md => "md".to_string(),
        cli::AnalysisFormat::Json => "json".to_string(),
        cli::AnalysisFormat::Jsonld => "jsonld".to_string(),
        cli::AnalysisFormat::Xml => "xml".to_string(),
        cli::AnalysisFormat::Svg => "svg".to_string(),
        cli::AnalysisFormat::Mermaid => "mermaid".to_string(),
        cli::AnalysisFormat::Obj => "obj".to_string(),
        cli::AnalysisFormat::Midi => "midi".to_string(),
        cli::AnalysisFormat::Tree => "tree".to_string(),
        cli::AnalysisFormat::Html => "html".to_string(),
    }
}

pub(crate) fn granularity_to_string(granularity: cli::ImportGranularity) -> String {
    match granularity {
        cli::ImportGranularity::Module => "module".to_string(),
        cli::ImportGranularity::File => "file".to_string(),
    }
}

pub(crate) fn map_preset(preset: cli::AnalysisPreset) -> analysis::AnalysisPreset {
    PresetKind::from_str(&format!("{:?}", preset).to_lowercase()).expect("unknown analysis preset")
}

pub(crate) fn map_granularity(granularity: cli::ImportGranularity) -> analysis::ImportGranularity {
    match granularity {
        cli::ImportGranularity::Module => analysis::ImportGranularity::Module,
        cli::ImportGranularity::File => analysis::ImportGranularity::File,
    }
}

fn analysis_output_filename(format: cli::AnalysisFormat) -> &'static str {
    match format {
        cli::AnalysisFormat::Md => "analysis.md",
        cli::AnalysisFormat::Json => "analysis.json",
        cli::AnalysisFormat::Jsonld => "analysis.jsonld",
        cli::AnalysisFormat::Xml => "analysis.xml",
        cli::AnalysisFormat::Svg => "analysis.svg",
        cli::AnalysisFormat::Mermaid => "analysis.mmd",
        cli::AnalysisFormat::Obj => "analysis.obj",
        cli::AnalysisFormat::Midi => "analysis.mid",
        cli::AnalysisFormat::Tree => "analysis.tree.txt",
        cli::AnalysisFormat::Html => "analysis.html",
    }
}

pub(crate) fn write_analysis_output(
    receipt: &analysis_types::AnalysisReceipt,
    output_dir: &Path,
    format: cli::AnalysisFormat,
) -> Result<()> {
    let rendered = analysis_format::render(receipt, format)?;
    let out_path = output_dir.join(analysis_output_filename(format));
    match rendered {
        analysis_format::RenderedOutput::Text(text) => {
            std::fs::write(&out_path, text)?;
        }
        analysis_format::RenderedOutput::Binary(bytes) => {
            std::fs::write(&out_path, bytes)?;
        }
    }
    Ok(())
}

pub(crate) fn write_analysis_stdout(
    receipt: &analysis_types::AnalysisReceipt,
    format: cli::AnalysisFormat,
) -> Result<()> {
    let rendered = analysis_format::render(receipt, format)?;
    match rendered {
        analysis_format::RenderedOutput::Text(text) => {
            print!("{}", text);
        }
        analysis_format::RenderedOutput::Binary(bytes) => {
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
            child_include_to_string(cli::ChildIncludeMode::Separate),
            "separate"
        );
    }

    #[test]
    fn test_child_include_to_string_parents_only() {
        assert_eq!(
            child_include_to_string(cli::ChildIncludeMode::ParentsOnly),
            "parents-only"
        );
    }

    #[test]
    fn test_preset_to_string_all_variants() {
        assert_eq!(preset_to_string(cli::AnalysisPreset::Receipt), "receipt");
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
        assert_eq!(format_to_string(cli::AnalysisFormat::Md), "md");
        assert_eq!(format_to_string(cli::AnalysisFormat::Json), "json");
        assert_eq!(format_to_string(cli::AnalysisFormat::Jsonld), "jsonld");
        assert_eq!(format_to_string(cli::AnalysisFormat::Xml), "xml");
        assert_eq!(format_to_string(cli::AnalysisFormat::Svg), "svg");
        assert_eq!(format_to_string(cli::AnalysisFormat::Mermaid), "mermaid");
        assert_eq!(format_to_string(cli::AnalysisFormat::Obj), "obj");
        assert_eq!(format_to_string(cli::AnalysisFormat::Midi), "midi");
        assert_eq!(format_to_string(cli::AnalysisFormat::Tree), "tree");
        assert_eq!(format_to_string(cli::AnalysisFormat::Html), "html");
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
            analysis_output_filename(cli::AnalysisFormat::Md),
            "analysis.md"
        );
        assert_eq!(
            analysis_output_filename(cli::AnalysisFormat::Json),
            "analysis.json"
        );
        assert_eq!(
            analysis_output_filename(cli::AnalysisFormat::Jsonld),
            "analysis.jsonld"
        );
        assert_eq!(
            analysis_output_filename(cli::AnalysisFormat::Xml),
            "analysis.xml"
        );
        assert_eq!(
            analysis_output_filename(cli::AnalysisFormat::Svg),
            "analysis.svg"
        );
        assert_eq!(
            analysis_output_filename(cli::AnalysisFormat::Mermaid),
            "analysis.mmd"
        );
        assert_eq!(
            analysis_output_filename(cli::AnalysisFormat::Obj),
            "analysis.obj"
        );
        assert_eq!(
            analysis_output_filename(cli::AnalysisFormat::Midi),
            "analysis.mid"
        );
        assert_eq!(
            analysis_output_filename(cli::AnalysisFormat::Tree),
            "analysis.tree.txt"
        );
        assert_eq!(
            analysis_output_filename(cli::AnalysisFormat::Html),
            "analysis.html"
        );
    }
}
