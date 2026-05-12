use std::path::PathBuf;

use tokmd_settings::Profile;

use crate::cli;
use crate::config::ResolvedConfig;

use super::parse::{parse_child_include_mode, parse_export_format, parse_redact_mode};

/// Resolve CLI `export` arguments combined with a legacy JSON profile.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use tokmd::cli::{CliExportArgs, Profile};
/// use tokmd::resolve_export;
///
/// let cli_args = CliExportArgs {
///     paths: None,
///     format: None,
///     output: None,
///     module_roots: None,
///     module_depth: None,
///     children: None,
///     min_code: None,
///     max_rows: None,
///     redact: None,
///     meta: None,
///     strip_prefix: None,
/// };
/// let profile = Profile::default();
///
/// let export_args = resolve_export(&cli_args, Some(&profile));
///
/// assert_eq!(export_args.paths, vec![PathBuf::from(".")]);
/// ```
pub fn resolve_export(
    cli_args: &cli::CliExportArgs,
    profile: Option<&Profile>,
) -> tokmd_types::ExportArgs {
    tokmd_types::ExportArgs {
        paths: cli_args
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli_args
            .format
            .map(Into::into)
            .or_else(|| parse_export_format(profile.and_then(|p| p.format.as_deref())))
            .unwrap_or(tokmd_types::ExportFormat::Jsonl),
        output: cli_args.output.clone(),
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
            .map(Into::into)
            .or_else(|| parse_child_include_mode(profile.and_then(|p| p.children.as_deref())))
            .unwrap_or(tokmd_types::ChildIncludeMode::Separate),
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
            .map(Into::into)
            .or(profile.and_then(|p| p.redact))
            .unwrap_or(tokmd_types::RedactMode::None),
        meta: cli_args
            .meta
            .or(profile.and_then(|p| p.meta))
            .unwrap_or(true),
        strip_prefix: cli_args.strip_prefix.clone(),
    }
}

/// Resolve export args using ConfigContext.
///
/// # Examples
///
/// ```
/// use tokmd::{resolve_export_with_config, ConfigContext};
/// use tokmd::cli::CliExportArgs;
/// use tokmd_types::ExportFormat;
/// use tokmd_settings::{TomlConfig, ExportConfig};
///
/// // Create config with specific export format
/// let mut toml = TomlConfig::default();
/// let mut exp_cfg = ExportConfig::default();
/// exp_cfg.format = Some("csv".to_string());
/// toml.export = exp_cfg;
///
/// let ctx = ConfigContext { toml: Some(toml), toml_path: None, json: None };
/// let resolved = tokmd::resolve_config(&ctx, None);
///
/// // Empty CLI arg uses the format from config
/// let cli_args_empty = CliExportArgs {
///     paths: None,
///     format: None,
///     output: None,
///     module_roots: None,
///     module_depth: None,
///     children: None,
///     min_code: None,
///     max_rows: None,
///     redact: None,
///     meta: None,
///     strip_prefix: None,
/// };
/// let export_args_1 = resolve_export_with_config(&cli_args_empty, &resolved);
/// assert_eq!(export_args_1.format, ExportFormat::Csv);
///
/// // CLI arg overrides config
/// let cli_args_override = CliExportArgs {
///     paths: None,
///     format: Some(tokmd::cli::ExportFormat::Jsonl),
///     output: None,
///     module_roots: None,
///     module_depth: None,
///     children: None,
///     min_code: None,
///     max_rows: None,
///     redact: None,
///     meta: None,
///     strip_prefix: None,
/// };
/// let export_args_2 = resolve_export_with_config(&cli_args_override, &resolved);
/// assert_eq!(export_args_2.format, ExportFormat::Jsonl);
/// ```
pub fn resolve_export_with_config(
    cli_args: &cli::CliExportArgs,
    resolved: &ResolvedConfig,
) -> tokmd_types::ExportArgs {
    tokmd_types::ExportArgs {
        paths: cli_args
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli_args
            .format
            .map(Into::into)
            .or_else(|| parse_export_format(resolved.format()))
            .or_else(|| parse_export_format(resolved.toml.and_then(|t| t.export.format.as_deref())))
            .unwrap_or(tokmd_types::ExportFormat::Jsonl),
        output: cli_args.output.clone(),
        module_roots: cli_args
            .module_roots
            .clone()
            .or(resolved.module_roots())
            .unwrap_or_else(|| vec!["crates".into(), "packages".into()]),
        module_depth: cli_args
            .module_depth
            .or(resolved.module_depth())
            .unwrap_or(2),
        children: cli_args
            .children
            .map(Into::into)
            .or_else(|| parse_child_include_mode(resolved.children()))
            .or_else(|| {
                parse_child_include_mode(resolved.toml.and_then(|t| t.export.children.as_deref()))
            })
            .unwrap_or(tokmd_types::ChildIncludeMode::Separate),
        min_code: cli_args.min_code.or(resolved.min_code()).unwrap_or(0),
        max_rows: cli_args.max_rows.or(resolved.max_rows()).unwrap_or(0),
        redact: cli_args
            .redact
            .map(Into::into)
            .or_else(|| parse_redact_mode(resolved.redact()))
            .unwrap_or(tokmd_types::RedactMode::None),
        meta: cli_args.meta.or(resolved.meta()).unwrap_or(true),
        strip_prefix: cli_args.strip_prefix.clone(),
    }
}
