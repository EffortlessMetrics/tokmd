use std::path::PathBuf;

use tokmd_settings::Profile;

use crate::cli;
use crate::config::ResolvedConfig;

use super::parse::{parse_child_include_mode, parse_table_format};

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
