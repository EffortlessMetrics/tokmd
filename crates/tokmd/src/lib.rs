//! # tokmd
//!
//! **CLI Binary**
//!
//! This is the entry point for the `tokmd` command-line application.
//! It orchestrates the other crates to perform the requested actions.
//!
//! ## Responsibilities
//! * Parse command line arguments
//! * Load configuration
//! * Dispatch commands to appropriate handlers
//! * Handle errors and exit codes
//!
//! This crate should contain minimal business logic.

use tokmd_analysis as analysis;
use tokmd_analysis_format as analysis_format;
use tokmd_config as cli;
use tokmd_format as format;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_tokeignore as tokeignore;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, ValueEnum};
use std::path::PathBuf;

use cli::{Cli, Commands, UserConfig};

fn load_config() -> Option<UserConfig> {
    let config_dir = dirs::config_dir()?.join("tokmd");
    let config_path = config_dir.join("config.json");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Entry point used by the `tokmd` (and optional `tok`) binaries.
fn make_scan_args(paths: &[PathBuf], global: &cli::GlobalArgs) -> tokmd_types::ScanArgs {
    tokmd_types::ScanArgs {
        paths: paths.iter().map(|p| p.display().to_string()).collect(),
        excluded: global.excluded.clone(),
        config: global.config,
        hidden: global.hidden,
        no_ignore: global.no_ignore,
        no_ignore_parent: global.no_ignore || global.no_ignore_parent,
        no_ignore_dot: global.no_ignore || global.no_ignore_dot,
        no_ignore_vcs: global.no_ignore || global.no_ignore_vcs,
        treat_doc_strings_as_comments: global.treat_doc_strings_as_comments,
    }
}

pub fn resolve_profile<'a>(
    config: &'a Option<UserConfig>,
    name: Option<&String>,
) -> Option<&'a cli::Profile> {
    config.as_ref().and_then(|c| {
        let key = name.map(|s| s.as_str()).unwrap_or("default");
        c.profiles.get(key)
    })
}

pub fn resolve_lang(
    cli: &cli::CliLangArgs,
    profile: Option<&cli::Profile>,
) -> tokmd_types::LangArgs {
    tokmd_types::LangArgs {
        paths: cli
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli
            .format
            .or_else(|| {
                profile
                    .and_then(|p| p.format.as_deref())
                    .and_then(|s| cli::TableFormat::from_str(s, true).ok())
            })
            .unwrap_or(cli::TableFormat::Md),
        top: cli.top.or_else(|| profile.and_then(|p| p.top)).unwrap_or(0),
        files: cli.files || profile.and_then(|p| p.files).unwrap_or(false),
        children: cli
            .children
            .or_else(|| {
                profile
                    .and_then(|p| p.children.as_deref())
                    .and_then(|s| cli::ChildrenMode::from_str(s, true).ok())
            })
            .unwrap_or(cli::ChildrenMode::Collapse),
    }
}

pub fn resolve_module(
    cli: &cli::CliModuleArgs,
    profile: Option<&cli::Profile>,
) -> tokmd_types::ModuleArgs {
    tokmd_types::ModuleArgs {
        paths: cli
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli
            .format
            .or_else(|| {
                profile
                    .and_then(|p| p.format.as_deref())
                    .and_then(|s| cli::TableFormat::from_str(s, true).ok())
            })
            .unwrap_or(cli::TableFormat::Md),
        top: cli.top.or_else(|| profile.and_then(|p| p.top)).unwrap_or(0),
        module_roots: cli
            .module_roots
            .clone()
            .or_else(|| profile.and_then(|p| p.module_roots.clone()))
            .unwrap_or_else(|| vec!["crates".into(), "packages".into()]),
        module_depth: cli
            .module_depth
            .or_else(|| profile.and_then(|p| p.module_depth))
            .unwrap_or(2),
        children: cli
            .children
            .or_else(|| {
                profile
                    .and_then(|p| p.children.as_deref())
                    .and_then(|s| cli::ChildIncludeMode::from_str(s, true).ok())
            })
            .unwrap_or(cli::ChildIncludeMode::Separate),
    }
}

pub fn resolve_export(
    cli: &cli::CliExportArgs,
    profile: Option<&cli::Profile>,
) -> tokmd_types::ExportArgs {
    tokmd_types::ExportArgs {
        paths: cli
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli
            .format
            .or_else(|| {
                profile
                    .and_then(|p| p.format.as_deref())
                    .and_then(|s| cli::ExportFormat::from_str(s, true).ok())
            })
            .unwrap_or(cli::ExportFormat::Jsonl),
        out: cli.out.clone(),
        module_roots: cli
            .module_roots
            .clone()
            .or_else(|| profile.and_then(|p| p.module_roots.clone()))
            .unwrap_or_else(|| vec!["crates".into(), "packages".into()]),
        module_depth: cli
            .module_depth
            .or_else(|| profile.and_then(|p| p.module_depth))
            .unwrap_or(2),
        children: cli
            .children
            .or_else(|| {
                profile
                    .and_then(|p| p.children.as_deref())
                    .and_then(|s| cli::ChildIncludeMode::from_str(s, true).ok())
            })
            .unwrap_or(cli::ChildIncludeMode::Separate),
        min_code: cli
            .min_code
            .or(profile.and_then(|p| p.min_code))
            .unwrap_or(0),
        max_rows: cli
            .max_rows
            .or(profile.and_then(|p| p.max_rows))
            .unwrap_or(0),
        redact: cli
            .redact
            .or(profile.and_then(|p| p.redact))
            .unwrap_or(cli::RedactMode::None),
        meta: cli.meta.or(profile.and_then(|p| p.meta)).unwrap_or(true),
        strip_prefix: cli.strip_prefix.clone(),
    }
}

#[derive(Debug, Clone)]
struct ExportMetaLite {
    schema_version: Option<u32>,
    generated_at_ms: Option<u128>,
    module_roots: Vec<String>,
    module_depth: usize,
    children: cli::ChildIncludeMode,
}

impl Default for ExportMetaLite {
    fn default() -> Self {
        Self {
            schema_version: None,
            generated_at_ms: None,
            module_roots: vec!["crates".into(), "packages".into()],
            module_depth: 2,
            children: cli::ChildIncludeMode::Separate,
        }
    }
}

struct ExportBundle {
    export: tokmd_types::ExportData,
    meta: ExportMetaLite,
    export_path: Option<PathBuf>,
    root: PathBuf,
}

fn load_export_from_inputs(inputs: &[PathBuf], global: &cli::GlobalArgs) -> Result<ExportBundle> {
    if inputs.len() > 1 {
        return scan_export_from_paths(inputs, global);
    }

    let input = inputs.first().cloned().unwrap_or_else(|| PathBuf::from("."));
    if input.is_dir() {
        let run_receipt = input.join("receipt.json");
        let export_jsonl = input.join("export.jsonl");
        let export_json = input.join("export.json");

        if run_receipt.exists() {
            return load_export_from_receipt(&run_receipt, Some(input));
        }
        if export_jsonl.exists() {
            return load_export_from_file(&export_jsonl, Some(input));
        }
        if export_json.exists() {
            return load_export_from_file(&export_json, Some(input));
        }
    }

    if input.is_file() {
        return load_export_from_file(&input, None);
    }

    scan_export_from_paths(inputs, global)
}

fn scan_export_from_paths(paths: &[PathBuf], global: &cli::GlobalArgs) -> Result<ExportBundle> {
    let languages = scan::scan(paths, global)?;
    let meta = ExportMetaLite::default();
    let export = model::create_export_data(
        &languages,
        &meta.module_roots,
        meta.module_depth,
        meta.children,
        None,
        0,
        0,
    );
    Ok(ExportBundle {
        export,
        meta,
        export_path: None,
        root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    })
}

fn load_export_from_receipt(path: &PathBuf, run_dir: Option<PathBuf>) -> Result<ExportBundle> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let receipt: tokmd_types::RunReceipt =
        serde_json::from_str(&content).context("Failed to parse run receipt")?;
    let base = run_dir.unwrap_or_else(|| path.parent().unwrap_or(path).to_path_buf());
    let export_path = base.join(&receipt.export_file);
    load_export_from_file(&export_path, Some(base))
}

fn load_export_from_file(path: &PathBuf, run_dir: Option<PathBuf>) -> Result<ExportBundle> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (mut export, meta) = if ext == "jsonl" {
        load_export_jsonl(path)?
    } else if ext == "json" {
        load_export_json(path)?
    } else {
        scan_export_from_paths(&[path.clone()], &cli::GlobalArgs::default())?
            .into_export_and_meta()
    };

    export.module_roots = meta.module_roots.clone();
    export.module_depth = meta.module_depth;
    export.children = meta.children;

    Ok(ExportBundle {
        export,
        meta,
        export_path: Some(path.clone()),
        root: run_dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    })
}

fn load_export_jsonl(path: &PathBuf) -> Result<(tokmd_types::ExportData, ExportMetaLite)> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let mut rows = Vec::new();
    let mut meta = ExportMetaLite::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)?;
        let ty = value
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("row");
        if ty == "meta" {
            if let Some(schema) = value.get("schema_version").and_then(|v| v.as_u64()) {
                meta.schema_version = Some(schema as u32);
            }
            if let Some(generated) = value.get("generated_at_ms").and_then(|v| v.as_u64()) {
                meta.generated_at_ms = Some(generated as u128);
            }
            if let Some(args) = value.get("args") {
                let parsed: tokmd_types::ExportArgsMeta = serde_json::from_value(args.clone())?;
                meta.module_roots = parsed.module_roots.clone();
                meta.module_depth = parsed.module_depth;
                meta.children = parsed.children;
            }
            continue;
        }

        let row: tokmd_types::FileRow = serde_json::from_value(value)?;
        rows.push(row);
    }

    Ok((
        tokmd_types::ExportData {
            rows,
            module_roots: meta.module_roots.clone(),
            module_depth: meta.module_depth,
            children: meta.children,
        },
        meta,
    ))
}

fn load_export_json(path: &PathBuf) -> Result<(tokmd_types::ExportData, ExportMetaLite)> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    if let Ok(receipt) = serde_json::from_str::<tokmd_types::ExportReceipt>(&content) {
        let mut meta = ExportMetaLite::default();
        meta.schema_version = Some(receipt.schema_version);
        meta.generated_at_ms = Some(receipt.generated_at_ms);
        meta.module_roots = receipt.args.module_roots.clone();
        meta.module_depth = receipt.args.module_depth;
        meta.children = receipt.args.children;
        return Ok((receipt.data, meta));
    }

    let rows: Vec<tokmd_types::FileRow> =
        serde_json::from_str(&content).context("Failed to parse export rows")?;
    let meta = ExportMetaLite::default();

    Ok((
        tokmd_types::ExportData {
            rows,
            module_roots: meta.module_roots.clone(),
            module_depth: meta.module_depth,
            children: meta.children,
        },
        meta,
    ))
}

impl ExportBundle {
    fn into_export_and_meta(self) -> (tokmd_types::ExportData, ExportMetaLite) {
        (self.export, self.meta)
    }
}

fn child_include_to_string(mode: cli::ChildIncludeMode) -> String {
    match mode {
        cli::ChildIncludeMode::Separate => "separate".to_string(),
        cli::ChildIncludeMode::ParentsOnly => "parents-only".to_string(),
    }
}

fn preset_to_string(preset: cli::AnalysisPreset) -> String {
    match preset {
        cli::AnalysisPreset::Receipt => "receipt".to_string(),
        cli::AnalysisPreset::Health => "health".to_string(),
        cli::AnalysisPreset::Risk => "risk".to_string(),
        cli::AnalysisPreset::Supply => "supply".to_string(),
        cli::AnalysisPreset::Architecture => "architecture".to_string(),
        cli::AnalysisPreset::Deep => "deep".to_string(),
        cli::AnalysisPreset::Fun => "fun".to_string(),
    }
}

fn format_to_string(format: cli::AnalysisFormat) -> String {
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

fn granularity_to_string(granularity: cli::ImportGranularity) -> String {
    match granularity {
        cli::ImportGranularity::Module => "module".to_string(),
        cli::ImportGranularity::File => "file".to_string(),
    }
}

fn map_preset(preset: cli::AnalysisPreset) -> analysis::AnalysisPreset {
    match preset {
        cli::AnalysisPreset::Receipt => analysis::AnalysisPreset::Receipt,
        cli::AnalysisPreset::Health => analysis::AnalysisPreset::Health,
        cli::AnalysisPreset::Risk => analysis::AnalysisPreset::Risk,
        cli::AnalysisPreset::Supply => analysis::AnalysisPreset::Supply,
        cli::AnalysisPreset::Architecture => analysis::AnalysisPreset::Architecture,
        cli::AnalysisPreset::Deep => analysis::AnalysisPreset::Deep,
        cli::AnalysisPreset::Fun => analysis::AnalysisPreset::Fun,
    }
}

fn map_granularity(granularity: cli::ImportGranularity) -> analysis::ImportGranularity {
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

fn write_analysis_output(
    receipt: &tokmd_analysis_types::AnalysisReceipt,
    output_dir: &PathBuf,
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

fn write_analysis_stdout(
    receipt: &tokmd_analysis_types::AnalysisReceipt,
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

fn badge_metric_label(metric: cli::BadgeMetric) -> &'static str {
    match metric {
        cli::BadgeMetric::Lines => "lines",
        cli::BadgeMetric::Tokens => "tokens",
        cli::BadgeMetric::Bytes => "bytes",
        cli::BadgeMetric::Doc => "doc",
        cli::BadgeMetric::Blank => "blank",
        cli::BadgeMetric::Hotspot => "hotspot",
    }
}

fn badge_svg(label: &str, value: &str) -> String {
    let label_width = (label.len() as i32 * 7 + 20).max(60);
    let value_width = (value.len() as i32 * 7 + 20).max(60);
    let width = label_width + value_width;
    let height = 24;
    let label_x = label_width / 2;
    let value_x = label_width + value_width / 2;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" role=\"img\"><rect width=\"{label_width}\" height=\"{height}\" fill=\"#555\"/><rect x=\"{label_width}\" width=\"{value_width}\" height=\"{height}\" fill=\"#4c9aff\"/><text x=\"{label_x}\" y=\"16\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"11\">{label}</text><text x=\"{value_x}\" y=\"16\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"11\">{value}</text></svg>",
        width = width,
        height = height,
        label_width = label_width,
        value_width = value_width,
        label_x = label_x,
        value_x = value_x,
        label = label,
        value = value
    )
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    // Load config and resolve profile
    let user_config = load_config();
    let profile = resolve_profile(&user_config, cli.profile.as_ref());

    match cli.command.unwrap_or(Commands::Lang(cli.lang.clone())) {
        Commands::Completions(args) => {
            use clap_complete::generate;
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            let shell = match args.shell {
                cli::Shell::Bash => clap_complete::Shell::Bash,
                cli::Shell::Elvish => clap_complete::Shell::Elvish,
                cli::Shell::Fish => clap_complete::Shell::Fish,
                cli::Shell::Powershell => clap_complete::Shell::PowerShell,
                cli::Shell::Zsh => clap_complete::Shell::Zsh,
            };
            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
        Commands::Run(args) => {
            // 1. Scan once
            let languages = scan::scan(&args.paths, &cli.global)?;

            // 2. Determine output directory
            let output_dir = if let Some(d) = args.output_dir {
                d
            } else {
                let state_dir = dirs::state_dir()
                    .or_else(dirs::data_local_dir)
                    .unwrap_or_else(std::env::temp_dir);
                let run_id = args.name.unwrap_or_else(|| format!("run-{}", now_ms()));
                state_dir.join("tokmd").join("runs").join(run_id)
            };

            std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;
            println!("Writing run artifacts to: {}", output_dir.display());

            // 3. Generate Reports
            let lang_report =
                model::create_lang_report(&languages, 0, false, cli::ChildrenMode::Collapse);
            let module_report = model::create_module_report(
                &languages,
                &["crates".to_string(), "packages".to_string()],
                2,
                cli::ChildIncludeMode::Separate,
                0,
            );
            let export_data = model::create_export_data(
                &languages,
                &["crates".to_string(), "packages".to_string()],
                2,
                cli::ChildIncludeMode::Separate,
                None,
                0,
                0,
            );

            // 4. Write artifacts
            let lang_path = output_dir.join("lang.json");
            let module_path = output_dir.join("module.json");

            {
                let receipt = tokmd_types::LangReceipt {
                    schema_version: tokmd_types::SCHEMA_VERSION,
                    generated_at_ms: now_ms(),
                    tool: tokmd_types::ToolInfo::current(),
                    mode: "lang".to_string(),
                    status: tokmd_types::ScanStatus::Complete,
                    warnings: vec![],
                    scan: make_scan_args(&args.paths, &cli.global),
                    args: tokmd_types::LangArgsMeta {
                        format: "json".to_string(),
                        top: 0,
                        with_files: false,
                        children: cli::ChildrenMode::Collapse,
                    },
                    report: lang_report,
                };
                let f = std::fs::File::create(&lang_path)?;
                serde_json::to_writer(f, &receipt)?;
            }
            {
                let receipt = tokmd_types::ModuleReceipt {
                    schema_version: tokmd_types::SCHEMA_VERSION,
                    generated_at_ms: now_ms(),
                    tool: tokmd_types::ToolInfo::current(),
                    mode: "module".to_string(),
                    status: tokmd_types::ScanStatus::Complete,
                    warnings: vec![],
                    scan: make_scan_args(&args.paths, &cli.global),
                    args: tokmd_types::ModuleArgsMeta {
                        format: "json".to_string(),
                        top: 0,
                        module_roots: vec!["crates".to_string(), "packages".to_string()],
                        module_depth: 2,
                        children: cli::ChildIncludeMode::Separate,
                    },
                    report: module_report,
                };
                let f = std::fs::File::create(&module_path)?;
                serde_json::to_writer(f, &receipt)?;
            }

            // 4. Write export.jsonl
            let export_path = output_dir.join("export.jsonl");
            {
                let f =
                    std::fs::File::create(&export_path).context("Failed to create export.jsonl")?;
                let mut writer = std::io::BufWriter::new(f);
                use std::io::Write;

                // Header
                let header = serde_json::json!({
                    "type": "meta",
                    "schema_version": tokmd_types::SCHEMA_VERSION,
                    "generated_at_ms": now_ms(),
                    "tool": tokmd_types::ToolInfo::current(),
                    "mode": "export",
                    "status": "complete",
                    "warnings": [],
                    "scan": make_scan_args(&args.paths, &cli.global),
                    "args": tokmd_types::ExportArgsMeta {
                       format: tokmd_config::ExportFormat::Jsonl,
                       module_roots: vec!["crates".to_string(), "packages".to_string()],
                       module_depth: 2,
                       children: cli::ChildIncludeMode::Separate,
                       min_code: 0,
                       max_rows: 0,
                       redact: tokmd_config::RedactMode::None,
                       strip_prefix: None,
                    }
                });
                serde_json::to_writer(&mut writer, &header)?;
                writeln!(&mut writer)?;

                // Rows
                for row in &export_data.rows {
                    let mut val = serde_json::to_value(row)?;
                    if let Some(obj) = val.as_object_mut() {
                        obj.insert("type".to_string(), "row".into());
                    }
                    serde_json::to_writer(&mut writer, &val)?;
                    writeln!(&mut writer)?;
                }
            }

            // 5. Write receipt.json
            let receipt = tokmd_types::RunReceipt {
                schema_version: tokmd_types::SCHEMA_VERSION,
                generated_at_ms: now_ms(),
                lang_file: "lang.json".to_string(),
                module_file: "module.json".to_string(),
                export_file: "export.jsonl".to_string(),
            };
            let receipt_path = output_dir.join("receipt.json");
            let f = std::fs::File::create(&receipt_path)?;
            serde_json::to_writer(f, &receipt)?;

            if let Some(preset) = args.analysis {
                let source = tokmd_analysis_types::AnalysisSource {
                    inputs: args
                        .paths
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect(),
                    export_path: Some("export.jsonl".to_string()),
                    export_schema_version: Some(tokmd_types::SCHEMA_VERSION),
                    export_generated_at_ms: None,
                    module_roots: export_data.module_roots.clone(),
                    module_depth: export_data.module_depth,
                    children: child_include_to_string(export_data.children),
                };
                let args_meta = tokmd_analysis_types::AnalysisArgsMeta {
                    preset: preset_to_string(preset),
                    format: "md+json".to_string(),
                    window_tokens: None,
                    git: None,
                    max_files: None,
                    max_bytes: None,
                    max_file_bytes: None,
                    max_commits: None,
                    max_commit_files: None,
                    import_granularity: "module".to_string(),
                };
                let request = analysis::AnalysisRequest {
                    preset: map_preset(preset),
                    args: args_meta,
                    limits: analysis::AnalysisLimits::default(),
                    window_tokens: None,
                    git: None,
                    import_granularity: analysis::ImportGranularity::Module,
                };
                let ctx = analysis::AnalysisContext {
                    export: export_data.clone(),
                    root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                    source,
                };
                let receipt = analysis::analyze(ctx, request)?;
                write_analysis_output(&receipt, &output_dir, cli::AnalysisFormat::Md)?;
                write_analysis_output(&receipt, &output_dir, cli::AnalysisFormat::Json)?;
            }
        }
        Commands::Diff(args) => {
            // 1. Load both receipts
            // For now, assume they are paths to the receipt.json file or the run dir.
            // 1. Load both receipts
            let from_path = std::path::PathBuf::from(&args.from);
            let to_path = std::path::PathBuf::from(&args.to);

            let load_lang = |path: &std::path::PathBuf| -> Result<tokmd_types::LangReceipt> {
                // Handle directory (implicit run) or receipt.json (implicit run)
                let lang_path = if path.is_dir() {
                    path.join("lang.json")
                } else if path.ends_with("receipt.json") {
                    path.parent().unwrap_or(path).join("lang.json")
                } else {
                    path.clone()
                };
                let content = std::fs::read_to_string(&lang_path)
                    .with_context(|| format!("Failed to read {}", lang_path.display()))?;
                serde_json::from_str(&content).context("Failed to parse lang receipt")
            };

            let from_receipt = load_lang(&from_path)?;
            let to_receipt = load_lang(&to_path)?;

            println!("Diffing Language Summaries:");
            println!(
                "{:<20} {:>10} {:>10} {:>10}",
                "Language", "Old LOC", "New LOC", "Delta"
            );
            println!("{:-<55}", "");

            // Simple map-based diff
            let mut all_langs: Vec<String> = from_receipt
                .report
                .rows
                .iter()
                .chain(to_receipt.report.rows.iter())
                .map(|r| r.lang.clone())
                .collect();
            all_langs.sort();
            all_langs.dedup();

            for lang_name in all_langs {
                let old_row = from_receipt
                    .report
                    .rows
                    .iter()
                    .find(|r| r.lang == lang_name);
                let new_row = to_receipt.report.rows.iter().find(|r| r.lang == lang_name);

                let old_code = old_row.map(|r| r.code).unwrap_or(0);
                let new_code = new_row.map(|r| r.code).unwrap_or(0);

                if old_code == new_code {
                    continue;
                }

                let delta = new_code as i64 - old_code as i64;
                let sign = if delta > 0 { "+" } else { "" };

                println!(
                    "{:<20} {:>10} {:>10} {:>10}{}",
                    lang_name, old_code, new_code, sign, delta
                );
            }
        }
        Commands::Lang(cli_args) => {
            let args = resolve_lang(&cli_args, profile);
            let languages = scan::scan(&args.paths, &cli.global)?;
            let report = model::create_lang_report(&languages, args.top, args.files, args.children);
            format::print_lang_report(&report, &cli.global, &args)?;
        }
        Commands::Module(cli_args) => {
            let args = resolve_module(&cli_args, profile);
            let languages = scan::scan(&args.paths, &cli.global)?;
            let report = model::create_module_report(
                &languages,
                &args.module_roots,
                args.module_depth,
                args.children,
                args.top,
            );
            format::print_module_report(&report, &cli.global, &args)?;
        }
        Commands::Export(cli_args) => {
            let args = resolve_export(&cli_args, profile);
            let languages = scan::scan(&args.paths, &cli.global)?;
            let strip_prefix = args.strip_prefix.as_deref();
            let export = model::create_export_data(
                &languages,
                &args.module_roots,
                args.module_depth,
                args.children,
                strip_prefix,
                args.min_code,
                args.max_rows,
            );
            format::write_export(&export, &cli.global, &args)?;
        }
        Commands::Analyze(args) => {
            let preset = args.preset.unwrap_or(cli::AnalysisPreset::Receipt);
            let format = args.format.unwrap_or(cli::AnalysisFormat::Md);
            let git_flag = if args.git {
                Some(true)
            } else if args.no_git {
                Some(false)
            } else {
                None
            };
            let granularity = args.granularity.unwrap_or(cli::ImportGranularity::Module);

            let bundle = load_export_from_inputs(&args.inputs, &cli.global)?;
            let source = tokmd_analysis_types::AnalysisSource {
                inputs: args
                    .inputs
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect(),
                export_path: bundle.export_path.as_ref().map(|p| p.display().to_string()),
                export_schema_version: bundle.meta.schema_version,
                export_generated_at_ms: bundle.meta.generated_at_ms,
                module_roots: bundle.meta.module_roots.clone(),
                module_depth: bundle.meta.module_depth,
                children: child_include_to_string(bundle.meta.children),
            };
            let args_meta = tokmd_analysis_types::AnalysisArgsMeta {
                preset: preset_to_string(preset),
                format: format_to_string(format),
                window_tokens: args.window,
                git: git_flag,
                max_files: args.max_files,
                max_bytes: args.max_bytes,
                max_file_bytes: args.max_file_bytes,
                max_commits: args.max_commits,
                max_commit_files: args.max_commit_files,
                import_granularity: granularity_to_string(granularity),
            };
            let request = analysis::AnalysisRequest {
                preset: map_preset(preset),
                args: args_meta,
                limits: analysis::AnalysisLimits {
                    max_files: args.max_files,
                    max_bytes: args.max_bytes,
                    max_file_bytes: args.max_file_bytes,
                    max_commits: args.max_commits,
                    max_commit_files: args.max_commit_files,
                },
                window_tokens: args.window,
                git: git_flag,
                import_granularity: map_granularity(granularity),
            };
            let ctx = analysis::AnalysisContext {
                export: bundle.export,
                root: bundle.root,
                source,
            };
            let receipt = analysis::analyze(ctx, request)?;

            if let Some(output_dir) = args.output_dir {
                std::fs::create_dir_all(&output_dir)
                    .context("Failed to create analysis output directory")?;
                write_analysis_output(&receipt, &output_dir, format)?;
            } else {
                write_analysis_stdout(&receipt, format)?;
            }
        }
        Commands::Badge(args) => {
            let metric = args.metric;
            let mut preset = args.preset.unwrap_or(cli::AnalysisPreset::Receipt);
            if metric == cli::BadgeMetric::Hotspot && args.preset.is_none() {
                preset = cli::AnalysisPreset::Risk;
            }
            let git_flag = if args.git {
                Some(true)
            } else if args.no_git {
                Some(false)
            } else if metric == cli::BadgeMetric::Hotspot {
                Some(true)
            } else {
                None
            };

            let bundle = load_export_from_inputs(&args.inputs, &cli.global)?;
            let source = tokmd_analysis_types::AnalysisSource {
                inputs: args
                    .inputs
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect(),
                export_path: bundle.export_path.as_ref().map(|p| p.display().to_string()),
                export_schema_version: bundle.meta.schema_version,
                export_generated_at_ms: bundle.meta.generated_at_ms,
                module_roots: bundle.meta.module_roots.clone(),
                module_depth: bundle.meta.module_depth,
                children: child_include_to_string(bundle.meta.children),
            };
            let args_meta = tokmd_analysis_types::AnalysisArgsMeta {
                preset: preset_to_string(preset),
                format: "badge".to_string(),
                window_tokens: None,
                git: git_flag,
                max_files: None,
                max_bytes: None,
                max_file_bytes: None,
                max_commits: args.max_commits,
                max_commit_files: args.max_commit_files,
                import_granularity: "module".to_string(),
            };
            let request = analysis::AnalysisRequest {
                preset: map_preset(preset),
                args: args_meta,
                limits: analysis::AnalysisLimits {
                    max_files: None,
                    max_bytes: None,
                    max_file_bytes: None,
                    max_commits: args.max_commits,
                    max_commit_files: args.max_commit_files,
                },
                window_tokens: None,
                git: git_flag,
                import_granularity: analysis::ImportGranularity::Module,
            };
            let ctx = analysis::AnalysisContext {
                export: bundle.export,
                root: bundle.root,
                source,
            };
            let receipt = analysis::analyze(ctx, request)?;

            let value = match metric {
                cli::BadgeMetric::Lines => receipt
                    .derived
                    .as_ref()
                    .map(|d| d.totals.lines.to_string())
                    .unwrap_or_else(|| "0".to_string()),
                cli::BadgeMetric::Tokens => receipt
                    .derived
                    .as_ref()
                    .map(|d| d.totals.tokens.to_string())
                    .unwrap_or_else(|| "0".to_string()),
                cli::BadgeMetric::Bytes => receipt
                    .derived
                    .as_ref()
                    .map(|d| d.totals.bytes.to_string())
                    .unwrap_or_else(|| "0".to_string()),
                cli::BadgeMetric::Doc => receipt
                    .derived
                    .as_ref()
                    .map(|d| format!("{:.1}%", d.doc_density.total.ratio * 100.0))
                    .unwrap_or_else(|| "0%".to_string()),
                cli::BadgeMetric::Blank => receipt
                    .derived
                    .as_ref()
                    .map(|d| format!("{:.1}%", d.whitespace.total.ratio * 100.0))
                    .unwrap_or_else(|| "0%".to_string()),
                cli::BadgeMetric::Hotspot => receipt
                    .git
                    .as_ref()
                    .and_then(|g| g.hotspots.first())
                    .map(|h| h.score.to_string())
                    .unwrap_or_else(|| "n/a".to_string()),
            };

            let label = badge_metric_label(metric);
            let svg = badge_svg(label, &value);

            if let Some(out) = args.out {
                std::fs::write(out, svg)?;
            } else {
                print!("{}", svg);
            }
        }
        Commands::Init(args) => {
            tokeignore::init_tokeignore(&args)?;
        }
    }

    Ok(())
}
