use std::path::PathBuf;

use clap::ValueEnum;
use tokmd_config as cli;

/// Configuration context combining TOML config, JSON config, and resolved profile.
#[derive(Debug, Default)]
pub struct ConfigContext {
    /// TOML configuration (tokmd.toml)
    pub toml: Option<cli::TomlConfig>,
    /// Path where TOML config was found
    pub toml_path: Option<PathBuf>,
    /// Legacy JSON configuration (config.json)
    pub json: Option<cli::UserConfig>,
}

impl ConfigContext {
    /// Get view profile from TOML config by name.
    pub fn get_toml_view(&self, name: &str) -> Option<&cli::ViewProfile> {
        self.toml.as_ref().and_then(|t| t.view.get(name))
    }

    /// Get profile from JSON config by name.
    pub fn get_json_profile(&self, name: &str) -> Option<&cli::Profile> {
        self.json.as_ref().and_then(|c| c.profiles.get(name))
    }
}

/// Load all configuration sources.
pub(crate) fn load_config() -> ConfigContext {
    let toml_result = discover_toml_config();
    let json = load_json_config();

    ConfigContext {
        toml: toml_result.as_ref().map(|(config, _)| config.clone()),
        toml_path: toml_result.map(|(_, path)| path),
        json,
    }
}

/// Discover TOML configuration following the precedence chain:
/// 1. TOKMD_CONFIG env var (explicit path)
/// 2. ./tokmd.toml (current directory)
/// 3. Parent directories up to root
/// 4. ~/.config/tokmd/tokmd.toml (user config)
fn discover_toml_config() -> Option<(cli::TomlConfig, PathBuf)> {
    // 1. Check TOKMD_CONFIG environment variable
    if let Ok(config_path) = std::env::var("TOKMD_CONFIG") {
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
fn try_load_toml(path: &std::path::Path) -> Option<(cli::TomlConfig, PathBuf)> {
    if path.exists() {
        cli::TomlConfig::from_file(path)
            .ok()
            .map(|config| (config, path.to_path_buf()))
    } else {
        None
    }
}

/// Load legacy JSON configuration from user config directory.
fn load_json_config() -> Option<cli::UserConfig> {
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
pub fn get_profile_name(cli_profile: Option<&String>) -> Option<String> {
    // CLI argument takes precedence
    if let Some(name) = cli_profile {
        return Some(name.clone());
    }

    // Then check TOKMD_PROFILE environment variable
    std::env::var("TOKMD_PROFILE")
        .ok()
        .filter(|s| !s.is_empty())
}

/// Resolve a JSON profile by name (legacy).
pub fn resolve_profile<'a>(
    config: &'a Option<cli::UserConfig>,
    name: Option<&String>,
) -> Option<&'a cli::Profile> {
    config.as_ref().and_then(|c| {
        let key = name.map(|s| s.as_str()).unwrap_or("default");
        c.profiles.get(key)
    })
}

/// Resolved configuration combining TOML and JSON sources.
#[derive(Debug, Default)]
pub struct ResolvedConfig<'a> {
    /// TOML view profile (takes precedence).
    pub toml_view: Option<&'a cli::ViewProfile>,
    /// JSON profile (fallback).
    pub json_profile: Option<&'a cli::Profile>,
    /// TOML config sections.
    pub toml: Option<&'a cli::TomlConfig>,
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

/// Resolve lang args using ConfigContext.
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

/// Resolve module args using ConfigContext.
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
