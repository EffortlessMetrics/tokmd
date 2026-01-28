use std::path::Path;

use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_analysis_format as analysis_format;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;

pub(crate) fn child_include_to_string(mode: cli::ChildIncludeMode) -> String {
    match mode {
        cli::ChildIncludeMode::Separate => "separate".to_string(),
        cli::ChildIncludeMode::ParentsOnly => "parents-only".to_string(),
    }
}

pub(crate) fn preset_to_string(preset: cli::AnalysisPreset) -> String {
    match preset {
        cli::AnalysisPreset::Receipt => "receipt".to_string(),
        cli::AnalysisPreset::Health => "health".to_string(),
        cli::AnalysisPreset::Risk => "risk".to_string(),
        cli::AnalysisPreset::Supply => "supply".to_string(),
        cli::AnalysisPreset::Architecture => "architecture".to_string(),
        cli::AnalysisPreset::Topics => "topics".to_string(),
        cli::AnalysisPreset::Security => "security".to_string(),
        cli::AnalysisPreset::Identity => "identity".to_string(),
        cli::AnalysisPreset::Git => "git".to_string(),
        cli::AnalysisPreset::Deep => "deep".to_string(),
        cli::AnalysisPreset::Fun => "fun".to_string(),
    }
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
