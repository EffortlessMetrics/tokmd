use std::path::PathBuf;

use tokmd_settings::Profile;

use crate::cli;
use crate::config::ResolvedConfig;

use super::parse::{parse_children_mode, parse_table_format};

/// Resolve CLI `lang` arguments combined with a legacy JSON profile.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use tokmd::cli::CliLangArgs;
/// use tokmd_settings::Profile;
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
