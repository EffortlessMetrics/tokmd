use std::path::PathBuf;

use crate::cli;
use clap::ValueEnum;
use tokmd_settings::{Profile, TomlConfig, UserConfig, ViewProfile};

/// Configuration context combining TOML config, JSON config, and resolved profile.
///
/// # Example
///
/// ```rust
/// use tokmd::config::load_config;
///
/// let config = load_config();
/// // config.toml and config.json will be loaded from the environment if present
/// ```
#[derive(Debug, Default)]
pub struct ConfigContext {
    /// TOML configuration (tokmd.toml)
    pub toml: Option<TomlConfig>,
    /// Path where TOML config was found
    pub toml_path: Option<PathBuf>,
    /// Legacy JSON configuration (config.json)
    pub json: Option<UserConfig>,
}

impl ConfigContext {
    /// Get view profile from TOML config by name.
    pub fn get_toml_view(&self, name: &str) -> Option<&ViewProfile> {
        self.toml.as_ref().and_then(|t| t.view.get(name))
    }

    /// Get profile from JSON config by name.
    pub fn get_json_profile(&self, name: &str) -> Option<&Profile> {
        self.json.as_ref().and_then(|c| c.profiles.get(name))
    }
}

/// Load all configuration sources.
pub fn load_config() -> ConfigContext {
    let toml_result = discover_toml_config();
    let json = load_json_config();

    ConfigContext {
        toml: toml_result.as_ref().map(|(config, _)| config.clone()),
        toml_path: toml_result.map(|(_, path)| path),
        json,
    }
}

fn sanitize_selector(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_control) {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Discover TOML configuration following the precedence chain:
/// 1. TOKMD_CONFIG env var (explicit path)
/// 2. ./tokmd.toml (current directory)
/// 3. Parent directories up to root
/// 4. ~/.config/tokmd/tokmd.toml (user config)
fn discover_toml_config() -> Option<(TomlConfig, PathBuf)> {
    // 1. Check TOKMD_CONFIG environment variable
    if let Ok(config_path) = std::env::var("TOKMD_CONFIG")
        && let Some(config_path) = sanitize_selector(&config_path)
    {
        let path = PathBuf::from(&config_path);
        if let Some(result) = try_load_toml(&path) {
            return Some(result);
        }
    }

    // 2. Check current directory and walk up to root
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = Some(cwd.as_path());
        while let Some(d) = dir {
            let config_path = d.join("tokmd.toml");
            if let Some(result) = try_load_toml(&config_path) {
                return Some(result);
            }
            dir = d.parent();
        }
    }

    // 3. Check user config directory
    if let Some(config_dir) = dirs::config_dir() {
        let user_config_path = config_dir.join("tokmd").join("tokmd.toml");
        if let Some(result) = try_load_toml(&user_config_path) {
            return Some(result);
        }
    }

    None
}

/// Try to load a TOML config file if it exists.
fn try_load_toml(path: &std::path::Path) -> Option<(TomlConfig, PathBuf)> {
    if path.exists() {
        TomlConfig::from_file(path)
            .ok()
            .map(|config| (config, path.to_path_buf()))
    } else {
        None
    }
}

/// Load legacy JSON configuration from user config directory.
fn load_json_config() -> Option<UserConfig> {
    let config_dir = dirs::config_dir()?.join("tokmd");
    let config_path = config_dir.join("config.json");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

/// Get the profile name from CLI arg, env var, or default.
///
/// # Examples
///
/// ```
/// use tokmd::config::get_profile_name;
///
/// let cli_profile = "  ci-profile  ".to_string();
/// assert_eq!(
///     get_profile_name(Some(&cli_profile)).as_deref(),
///     Some("ci-profile")
/// );
/// ```
pub fn get_profile_name(cli_profile: Option<&String>) -> Option<String> {
    // CLI argument takes precedence
    if let Some(name) = cli_profile
        && let Some(name) = sanitize_selector(name)
    {
        return Some(name);
    }

    // Then check TOKMD_PROFILE environment variable
    std::env::var("TOKMD_PROFILE")
        .ok()
        .and_then(|name| sanitize_selector(&name))
}

/// Resolve a JSON profile by name (legacy).
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
///
/// use tokmd::config::resolve_profile;
/// use tokmd_settings::{Profile, UserConfig};
///
/// let mut profiles = BTreeMap::new();
/// let profile = Profile {
///     top: Some(10),
///     ..Default::default()
/// };
/// profiles.insert("default".to_string(), profile);
///
/// let config = Some(UserConfig {
///     profiles,
///     repos: BTreeMap::new(),
/// });
///
/// let resolved = resolve_profile(&config, None).expect("default profile");
/// assert_eq!(resolved.top, Some(10));
/// ```
pub fn resolve_profile<'a>(
    config: &'a Option<UserConfig>,
    name: Option<&String>,
) -> Option<&'a Profile> {
    config.as_ref().and_then(|c| {
        let key = name.map(|s| s.as_str()).unwrap_or("default");
        c.profiles.get(key)
    })
}

/// Resolved configuration combining TOML and JSON sources.
///
/// This struct aggregates configurations from both `tokmd.toml` and the legacy
/// `config.json`, preferring TOML settings over JSON ones.
///
/// # Examples
///
/// ```
/// use tokmd::{ConfigContext, ResolvedConfig};
///
/// let ctx = ConfigContext::default();
/// let resolved = tokmd::resolve_config(&ctx, None);
///
/// assert_eq!(resolved.format(), None);
/// assert_eq!(resolved.top(), None);
/// ```
#[derive(Debug, Default)]
pub struct ResolvedConfig<'a> {
    /// TOML view profile (takes precedence).
    pub toml_view: Option<&'a ViewProfile>,
    /// JSON profile (fallback).
    pub json_profile: Option<&'a Profile>,
    /// TOML config sections.
    pub toml: Option<&'a TomlConfig>,
}

impl ResolvedConfig<'_> {
    /// Get format string, preferring TOML view, then JSON profile.
    pub fn format(&self) -> Option<&str> {
        self.toml_view
            .and_then(|v| v.format.as_deref())
            .or_else(|| self.json_profile.and_then(|p| p.format.as_deref()))
    }

    /// Get top value.
    pub fn top(&self) -> Option<usize> {
        self.toml_view
            .and_then(|v| v.top)
            .or_else(|| self.json_profile.and_then(|p| p.top))
    }

    /// Get files flag.
    pub fn files(&self) -> Option<bool> {
        self.toml_view
            .and_then(|v| v.files)
            .or_else(|| self.json_profile.and_then(|p| p.files))
    }

    /// Get module roots.
    pub fn module_roots(&self) -> Option<Vec<String>> {
        self.toml_view
            .and_then(|v| v.module_roots.clone())
            .or_else(|| self.toml.and_then(|t| t.module.roots.clone()))
            .or_else(|| self.json_profile.and_then(|p| p.module_roots.clone()))
    }

    /// Get module depth.
    pub fn module_depth(&self) -> Option<usize> {
        self.toml_view
            .and_then(|v| v.module_depth)
            .or_else(|| self.toml.and_then(|t| t.module.depth))
            .or_else(|| self.json_profile.and_then(|p| p.module_depth))
    }

    /// Get children mode string.
    pub fn children(&self) -> Option<&str> {
        self.toml_view
            .and_then(|v| v.children.as_deref())
            .or_else(|| self.toml.and_then(|t| t.module.children.as_deref()))
            .or_else(|| self.json_profile.and_then(|p| p.children.as_deref()))
    }

    /// Get min_code.
    pub fn min_code(&self) -> Option<usize> {
        self.toml_view
            .and_then(|v| v.min_code)
            .or_else(|| self.toml.and_then(|t| t.export.min_code))
            .or_else(|| self.json_profile.and_then(|p| p.min_code))
    }

    /// Get max_rows.
    pub fn max_rows(&self) -> Option<usize> {
        self.toml_view
            .and_then(|v| v.max_rows)
            .or_else(|| self.toml.and_then(|t| t.export.max_rows))
            .or_else(|| self.json_profile.and_then(|p| p.max_rows))
    }

    /// Get redact mode string.
    pub fn redact(&self) -> Option<&str> {
        self.toml_view
            .and_then(|v| v.redact.as_deref())
            .or_else(|| self.toml.and_then(|t| t.export.redact.as_deref()))
    }

    /// Get meta flag.
    pub fn meta(&self) -> Option<bool> {
        self.toml_view
            .and_then(|v| v.meta)
            .or_else(|| self.json_profile.and_then(|p| p.meta))
    }
}

/// Resolve configuration from context and profile name.
///
/// # Examples
///
/// ```
/// use tokmd::{resolve_config, ConfigContext};
///
/// let ctx = ConfigContext::default();
/// let resolved = resolve_config(&ctx, Some("default"));
///
/// assert!(resolved.toml_view.is_none());
/// assert!(resolved.json_profile.is_none());
/// ```
pub fn resolve_config<'a>(
    ctx: &'a ConfigContext,
    profile_name: Option<&str>,
) -> ResolvedConfig<'a> {
    let toml_view = profile_name.and_then(|name| ctx.get_toml_view(name));
    let json_profile = profile_name.and_then(|name| ctx.get_json_profile(name));

    ResolvedConfig {
        toml_view,
        json_profile,
        toml: ctx.toml.as_ref(),
    }
}

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
            .or_else(|| {
                resolved
                    .format()
                    .and_then(|s| cli::TableFormat::from_str(s, true).ok())
            })
            .unwrap_or(cli::TableFormat::Md),
        top: cli_args.top.or(resolved.top()).unwrap_or(0),
        files: cli_args.files || resolved.files().unwrap_or(false),
        children: cli_args
            .children
            .or_else(|| {
                resolved
                    .children()
                    .and_then(|s| cli::ChildrenMode::from_str(s, true).ok())
            })
            .unwrap_or(cli::ChildrenMode::Collapse),
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
            .or_else(|| {
                resolved
                    .format()
                    .and_then(|s| cli::TableFormat::from_str(s, true).ok())
            })
            .unwrap_or(cli::TableFormat::Md),
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
            .or_else(|| {
                resolved
                    .children()
                    .and_then(|s| cli::ChildIncludeMode::from_str(s, true).ok())
            })
            .unwrap_or(cli::ChildIncludeMode::Separate),
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
            .or_else(|| {
                profile
                    .and_then(|p| p.format.as_deref())
                    .and_then(|s| cli::ExportFormat::from_str(s, true).ok())
            })
            .unwrap_or(cli::ExportFormat::Jsonl),
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

/// Resolve export args using ConfigContext.
///
/// # Examples
///
/// ```
/// use tokmd::cli::{CliExportArgs, ExportFormat};
/// use tokmd::{resolve_export_with_config, ConfigContext};
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
///     format: Some(ExportFormat::Jsonl),
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
            .or_else(|| {
                resolved
                    .format()
                    .and_then(|s| cli::ExportFormat::from_str(s, true).ok())
            })
            .or_else(|| {
                resolved
                    .toml
                    .and_then(|t| t.export.format.as_deref())
                    .and_then(|s| cli::ExportFormat::from_str(s, true).ok())
            })
            .unwrap_or(cli::ExportFormat::Jsonl),
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
            .or_else(|| {
                resolved
                    .children()
                    .and_then(|s| cli::ChildIncludeMode::from_str(s, true).ok())
            })
            .or_else(|| {
                resolved
                    .toml
                    .and_then(|t| t.export.children.as_deref())
                    .and_then(|s| cli::ChildIncludeMode::from_str(s, true).ok())
            })
            .unwrap_or(cli::ChildIncludeMode::Separate),
        min_code: cli_args.min_code.or(resolved.min_code()).unwrap_or(0),
        max_rows: cli_args.max_rows.or(resolved.max_rows()).unwrap_or(0),
        redact: cli_args
            .redact
            .or_else(|| {
                resolved
                    .redact()
                    .and_then(|s| cli::RedactMode::from_str(s, true).ok())
            })
            .unwrap_or(cli::RedactMode::None),
        meta: cli_args.meta.or(resolved.meta()).unwrap_or(true),
        strip_prefix: cli_args.strip_prefix.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{get_profile_name, sanitize_selector};

    #[test]
    fn sanitize_selector_rejects_empty_and_control_values() {
        assert_eq!(sanitize_selector("   "), None);
        assert_eq!(sanitize_selector("bad\nvalue"), None);
        assert_eq!(sanitize_selector("bad\0value"), None);
    }

    #[test]
    fn sanitize_selector_trims_safe_values() {
        assert_eq!(sanitize_selector("  default  ").as_deref(), Some("default"));
    }

    #[test]
    fn get_profile_name_sanitizes_cli_value() {
        let cli_value = "  secure-profile  ".to_string();
        assert_eq!(
            get_profile_name(Some(&cli_value)).as_deref(),
            Some("secure-profile")
        );
    }

    #[test]
    fn get_profile_name_rejects_control_char_cli_value() {
        let cli_value = "bad\nprofile".to_string();
        assert_eq!(get_profile_name(Some(&cli_value)), None);
    }
}
