//! CLI argument resolution from legacy JSON profiles and TOML config views.

use std::path::PathBuf;

use tokmd_settings::Profile;

use crate::cli;

use super::ResolvedConfig;

mod parse;

use parse::{
    parse_child_include_mode, parse_children_mode, parse_export_format, parse_redact_mode,
    parse_table_format,
};

/// Resolve CLI `lang` arguments combined with a legacy JSON profile.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use tokmd::cli::{CliLangArgs, Profile};
/// use tokmd::resolve_lang;
///
/// let cli_args = CliLangArgs {
///     paths: None,
///     format: None,
///     top: None,
///     files: false,
///     children: None,
/// };
/// let profile = Profile::default();
///
/// let resolved_lang = resolve_lang(&cli_args, Some(&profile));
///
/// assert_eq!(resolved_lang.paths, vec![PathBuf::from(".")]);
/// assert_eq!(resolved_lang.top, 0);
/// assert_eq!(resolved_lang.files, false);
/// ```
pub fn resolve_lang(
    cli_args: &cli::CliLangArgs,
    profile: Option<&Profile>,
) -> tokmd_types::LangArgs {
    tokmd_types::LangArgs {
        paths: cli_args
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli_args
            .format
            .map(Into::into)
            .or_else(|| parse_table_format(profile.and_then(|p| p.format.as_deref())))
            .unwrap_or(tokmd_types::TableFormat::Md),
        top: cli_args
            .top
            .or_else(|| profile.and_then(|p| p.top))
            .unwrap_or(0),
        files: cli_args.files || profile.and_then(|p| p.files).unwrap_or(false),
        children: cli_args
            .children
            .map(Into::into)
            .or_else(|| parse_children_mode(profile.and_then(|p| p.children.as_deref())))
            .unwrap_or(tokmd_types::ChildrenMode::Collapse),
    }
}

/// Resolve lang args using ConfigContext.
///
/// # Examples
///
/// ```
/// use tokmd::cli::CliLangArgs;
/// use tokmd::{resolve_lang_with_config, ConfigContext};
/// use tokmd_settings::{TomlConfig, ViewProfile};
///
/// // Create a config with a TOML view specifying top = 10
/// let mut toml = TomlConfig::default();
/// let mut view = ViewProfile::default();
/// view.top = Some(10);
/// toml.view.insert("default".to_string(), view);
///
/// let ctx = ConfigContext { toml: Some(toml), toml_path: None, json: None };
/// let resolved = tokmd::resolve_config(&ctx, Some("default"));
///
/// // CLI args have no top, so it falls back to the resolved config (10)
/// let cli_args_empty = CliLangArgs {
///     paths: None,
///     format: None,
///     top: None,
///     files: false,
///     children: None,
/// };
/// let lang_args_1 = resolve_lang_with_config(&cli_args_empty, &resolved);
/// assert_eq!(lang_args_1.top, 10);
///
/// // CLI arg overrides the config
/// let cli_args_override = CliLangArgs {
///     paths: None,
///     format: None,
///     top: Some(5),
///     files: false,
///     children: None,
/// };
/// let lang_args_2 = resolve_lang_with_config(&cli_args_override, &resolved);
/// assert_eq!(lang_args_2.top, 5);
/// ```
pub fn resolve_lang_with_config(
    cli_args: &cli::CliLangArgs,
    resolved: &ResolvedConfig,
) -> tokmd_types::LangArgs {
    tokmd_types::LangArgs {
        paths: cli_args
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli_args
            .format
            .map(Into::into)
            .or_else(|| parse_table_format(resolved.format()))
            .unwrap_or(tokmd_types::TableFormat::Md),
        top: cli_args.top.or(resolved.top()).unwrap_or(0),
        files: cli_args.files || resolved.files().unwrap_or(false),
        children: cli_args
            .children
            .map(Into::into)
            .or_else(|| parse_children_mode(resolved.children()))
            .unwrap_or(tokmd_types::ChildrenMode::Collapse),
    }
}

/// Resolve CLI `module` arguments combined with a legacy JSON profile.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use tokmd::cli::{CliModuleArgs, Profile};
/// use tokmd::resolve_module;
///
/// let cli_args = CliModuleArgs {
///     paths: None,
///     format: None,
///     top: None,
///     module_roots: None,
///     module_depth: None,
///     children: None,
/// };
/// let profile = Profile::default();
///
/// let module_args = resolve_module(&cli_args, Some(&profile));
///
/// assert_eq!(module_args.paths, vec![PathBuf::from(".")]);
/// assert_eq!(
///     module_args.module_roots,
///     vec!["crates".to_string(), "packages".to_string()]
/// );
/// ```
pub fn resolve_module(
    cli_args: &cli::CliModuleArgs,
    profile: Option<&Profile>,
) -> tokmd_types::ModuleArgs {
    tokmd_types::ModuleArgs {
        paths: cli_args
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli_args
            .format
            .map(Into::into)
            .or_else(|| parse_table_format(profile.and_then(|p| p.format.as_deref())))
            .unwrap_or(tokmd_types::TableFormat::Md),
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
            .map(Into::into)
            .or_else(|| parse_child_include_mode(profile.and_then(|p| p.children.as_deref())))
            .unwrap_or(tokmd_types::ChildIncludeMode::Separate),
    }
}

/// Resolve module args using ConfigContext.
///
/// # Examples
///
/// ```
/// use tokmd::cli::CliModuleArgs;
/// use tokmd::{resolve_module_with_config, ConfigContext};
/// use tokmd_settings::{TomlConfig, ModuleConfig};
///
/// // Create a config with a custom module depth
/// let mut toml = TomlConfig::default();
/// let mut mod_cfg = ModuleConfig::default();
/// mod_cfg.depth = Some(4);
/// toml.module = mod_cfg;
///
/// let ctx = ConfigContext { toml: Some(toml), toml_path: None, json: None };
/// let resolved = tokmd::resolve_config(&ctx, None);
///
/// // CLI args have no module_depth, falls back to config (4)
/// let cli_args_empty = CliModuleArgs {
///     paths: None,
///     format: None,
///     top: None,
///     module_roots: None,
///     module_depth: None,
///     children: None,
/// };
/// let module_args_1 = resolve_module_with_config(&cli_args_empty, &resolved);
/// assert_eq!(module_args_1.module_depth, 4);
///
/// // CLI arg overrides config
/// let cli_args_override = CliModuleArgs {
///     paths: None,
///     format: None,
///     top: None,
///     module_roots: None,
///     module_depth: Some(1),
///     children: None,
/// };
/// let module_args_2 = resolve_module_with_config(&cli_args_override, &resolved);
/// assert_eq!(module_args_2.module_depth, 1);
/// assert_eq!(
///     module_args_2.module_roots,
///     vec!["crates".to_string(), "packages".to_string()]
/// );
/// ```
pub fn resolve_module_with_config(
    cli_args: &cli::CliModuleArgs,
    resolved: &ResolvedConfig,
) -> tokmd_types::ModuleArgs {
    tokmd_types::ModuleArgs {
        paths: cli_args
            .paths
            .clone()
            .unwrap_or_else(|| vec![PathBuf::from(".")]),
        format: cli_args
            .format
            .map(Into::into)
            .or_else(|| parse_table_format(resolved.format()))
            .unwrap_or(tokmd_types::TableFormat::Md),
        top: cli_args.top.or(resolved.top()).unwrap_or(0),
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
            .unwrap_or(tokmd_types::ChildIncludeMode::Separate),
    }
}

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
