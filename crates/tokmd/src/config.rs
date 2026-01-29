use std::path::PathBuf;

use clap::ValueEnum;
use tokmd_config as cli;

pub(crate) fn load_config() -> Option<cli::UserConfig> {
    let config_dir = dirs::config_dir()?.join("tokmd");
    let config_path = config_dir.join("config.json");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn resolve_profile<'a>(
    config: &'a Option<cli::UserConfig>,
    name: Option<&String>,
) -> Option<&'a cli::Profile> {
    config.as_ref().and_then(|c| {
        let key = name.map(|s| s.as_str()).unwrap_or("default");
        c.profiles.get(key)
    })
}

pub fn resolve_lang(
    cli_args: &cli::CliLangArgs,
    profile: Option<&cli::Profile>,
) -> tokmd_types::LangArgs {
    tokmd_types::LangArgs {
        paths: cli_args
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli_args
            .format
            .or_else(|| {
                profile
                    .and_then(|p| p.format.as_deref())
                    .and_then(|s| cli::TableFormat::from_str(s, true).ok())
            })
            .unwrap_or(cli::TableFormat::Md),
        top: cli_args
            .top
            .or_else(|| profile.and_then(|p| p.top))
            .unwrap_or(0),
        files: cli_args.files || profile.and_then(|p| p.files).unwrap_or(false),
        children: cli_args
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
    cli_args: &cli::CliModuleArgs,
    profile: Option<&cli::Profile>,
) -> tokmd_types::ModuleArgs {
    tokmd_types::ModuleArgs {
        paths: cli_args
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli_args
            .format
            .or_else(|| {
                profile
                    .and_then(|p| p.format.as_deref())
                    .and_then(|s| cli::TableFormat::from_str(s, true).ok())
            })
            .unwrap_or(cli::TableFormat::Md),
        top: cli_args
            .top
            .or_else(|| profile.and_then(|p| p.top))
            .unwrap_or(0),
        module_roots: cli_args
            .module_roots
            .clone()
            .or_else(|| profile.and_then(|p| p.module_roots.clone()))
            .unwrap_or_else(|| vec!["crates".into(), "packages".into()]),
        module_depth: cli_args
            .module_depth
            .or_else(|| profile.and_then(|p| p.module_depth))
            .unwrap_or(2),
        children: cli_args
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
    cli_args: &cli::CliExportArgs,
    profile: Option<&cli::Profile>,
) -> tokmd_types::ExportArgs {
    tokmd_types::ExportArgs {
        paths: cli_args
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli_args
            .format
            .or_else(|| {
                profile
                    .and_then(|p| p.format.as_deref())
                    .and_then(|s| cli::ExportFormat::from_str(s, true).ok())
            })
            .unwrap_or(cli::ExportFormat::Jsonl),
        out: cli_args.out.clone(),
        module_roots: cli_args
            .module_roots
            .clone()
            .or_else(|| profile.and_then(|p| p.module_roots.clone()))
            .unwrap_or_else(|| vec!["crates".into(), "packages".into()]),
        module_depth: cli_args
            .module_depth
            .or_else(|| profile.and_then(|p| p.module_depth))
            .unwrap_or(2),
        children: cli_args
            .children
            .or_else(|| {
                profile
                    .and_then(|p| p.children.as_deref())
                    .and_then(|s| cli::ChildIncludeMode::from_str(s, true).ok())
            })
            .unwrap_or(cli::ChildIncludeMode::Separate),
        min_code: cli_args
            .min_code
            .or(profile.and_then(|p| p.min_code))
            .unwrap_or(0),
        max_rows: cli_args
            .max_rows
            .or(profile.and_then(|p| p.max_rows))
            .unwrap_or(0),
        redact: cli_args
            .redact
            .or(profile.and_then(|p| p.redact))
            .unwrap_or(cli::RedactMode::None),
        meta: cli_args
            .meta
            .or(profile.and_then(|p| p.meta))
            .unwrap_or(true),
        strip_prefix: cli_args.strip_prefix.clone(),
    }
}
