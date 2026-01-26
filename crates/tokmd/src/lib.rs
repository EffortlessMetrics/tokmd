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
fn tool_info() -> tokmd_types::ToolInfo {
    tokmd_types::ToolInfo {
        name: "tokmd".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

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
                    schema_version: 2,
                    generated_at_ms: now_ms(),
                    tool: tool_info(),
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
                    schema_version: 2,
                    generated_at_ms: now_ms(),
                    tool: tool_info(),
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
                    "schema_version": 2,
                    "generated_at_ms": now_ms(),
                    "tool": tool_info(),
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
                schema_version: 2,
                generated_at_ms: now_ms(),
                lang_file: "lang.json".to_string(),
                module_file: "module.json".to_string(),
                export_file: "export.jsonl".to_string(),
            };
            let receipt_path = output_dir.join("receipt.json");
            let f = std::fs::File::create(&receipt_path)?;
            serde_json::to_writer(f, &receipt)?;
        }
        Commands::Diff(args) => {
            // 1. Load both receipts
            // For now, assume they are paths to the receipt.json file or the run dir.
            // 1. Load both receipts
            let from_path = std::path::PathBuf::from(&args.from);
            let to_path = std::path::PathBuf::from(&args.to);

            let load_lang = |path: &std::path::PathBuf| -> Result<tokmd_types::LangReceipt> {
                // Try to find lang.json in the same directory if the path points to receipt.json
                let lang_path = if path.ends_with("receipt.json") {
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
        Commands::Init(args) => {
            tokeignore::init_tokeignore(&args)?;
        }
    }

    Ok(())
}
