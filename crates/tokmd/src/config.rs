use std::path::PathBuf;

use tokmd_settings::{Profile, TomlConfig, UserConfig, ViewProfile};

mod resolve;

pub use resolve::{
    resolve_export, resolve_export_with_config, resolve_lang, resolve_lang_with_config,
    resolve_module, resolve_module_with_config,
};

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
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ConfigContext;
    ///
    /// let ctx = ConfigContext::default();
    /// assert!(ctx.get_toml_view("default").is_none());
    /// ```
    pub fn get_toml_view(&self, name: &str) -> Option<&ViewProfile> {
        self.toml.as_ref().and_then(|t| t.view.get(name))
    }

    /// Get profile from JSON config by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ConfigContext;
    ///
    /// let ctx = ConfigContext::default();
    /// assert!(ctx.get_json_profile("default").is_none());
    /// ```
    pub fn get_json_profile(&self, name: &str) -> Option<&Profile> {
        self.json.as_ref().and_then(|c| c.profiles.get(name))
    }
}

/// Load all configuration sources.
///
/// # Examples
///
/// ```
/// use tokmd::config::load_config;
///
/// let config = load_config();
/// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ResolvedConfig;
    ///
    /// let resolved = ResolvedConfig::default();
    /// assert_eq!(resolved.format(), None);
    /// ```
    pub fn format(&self) -> Option<&str> {
        self.toml_view
            .and_then(|v| v.format.as_deref())
            .or_else(|| self.json_profile.and_then(|p| p.format.as_deref()))
    }

    /// Get top value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ResolvedConfig;
    ///
    /// let resolved = ResolvedConfig::default();
    /// assert_eq!(resolved.top(), None);
    /// ```
    pub fn top(&self) -> Option<usize> {
        self.toml_view
            .and_then(|v| v.top)
            .or_else(|| self.json_profile.and_then(|p| p.top))
    }

    /// Get files flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ResolvedConfig;
    ///
    /// let resolved = ResolvedConfig::default();
    /// assert_eq!(resolved.files(), None);
    /// ```
    pub fn files(&self) -> Option<bool> {
        self.toml_view
            .and_then(|v| v.files)
            .or_else(|| self.json_profile.and_then(|p| p.files))
    }

    /// Get module roots.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ResolvedConfig;
    ///
    /// let resolved = ResolvedConfig::default();
    /// assert_eq!(resolved.module_roots(), None);
    /// ```
    pub fn module_roots(&self) -> Option<Vec<String>> {
        self.toml_view
            .and_then(|v| v.module_roots.clone())
            .or_else(|| self.toml.and_then(|t| t.module.roots.clone()))
            .or_else(|| self.json_profile.and_then(|p| p.module_roots.clone()))
    }

    /// Get module depth.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ResolvedConfig;
    ///
    /// let resolved = ResolvedConfig::default();
    /// assert_eq!(resolved.module_depth(), None);
    /// ```
    pub fn module_depth(&self) -> Option<usize> {
        self.toml_view
            .and_then(|v| v.module_depth)
            .or_else(|| self.toml.and_then(|t| t.module.depth))
            .or_else(|| self.json_profile.and_then(|p| p.module_depth))
    }

    /// Get children mode string.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ResolvedConfig;
    ///
    /// let resolved = ResolvedConfig::default();
    /// assert_eq!(resolved.children(), None);
    /// ```
    pub fn children(&self) -> Option<&str> {
        self.toml_view
            .and_then(|v| v.children.as_deref())
            .or_else(|| self.toml.and_then(|t| t.module.children.as_deref()))
            .or_else(|| self.json_profile.and_then(|p| p.children.as_deref()))
    }

    /// Get min_code.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ResolvedConfig;
    ///
    /// let resolved = ResolvedConfig::default();
    /// assert_eq!(resolved.min_code(), None);
    /// ```
    pub fn min_code(&self) -> Option<usize> {
        self.toml_view
            .and_then(|v| v.min_code)
            .or_else(|| self.toml.and_then(|t| t.export.min_code))
            .or_else(|| self.json_profile.and_then(|p| p.min_code))
    }

    /// Get max_rows.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ResolvedConfig;
    ///
    /// let resolved = ResolvedConfig::default();
    /// assert_eq!(resolved.max_rows(), None);
    /// ```
    pub fn max_rows(&self) -> Option<usize> {
        self.toml_view
            .and_then(|v| v.max_rows)
            .or_else(|| self.toml.and_then(|t| t.export.max_rows))
            .or_else(|| self.json_profile.and_then(|p| p.max_rows))
    }

    /// Get redact mode string.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ResolvedConfig;
    ///
    /// let resolved = ResolvedConfig::default();
    /// assert_eq!(resolved.redact(), None);
    /// ```
    pub fn redact(&self) -> Option<&str> {
        self.toml_view
            .and_then(|v| v.redact.as_deref())
            .or_else(|| self.toml.and_then(|t| t.export.redact.as_deref()))
    }

    /// Get meta flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd::config::ResolvedConfig;
    ///
    /// let resolved = ResolvedConfig::default();
    /// assert_eq!(resolved.meta(), None);
    /// ```
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
