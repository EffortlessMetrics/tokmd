//! Legacy profile configuration contracts.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tokmd_types::RedactMode;

// Legacy profile contract persisted by the historical `config.json` format.
//
// These types are intentionally kept in `tokmd-settings` so that the config
// profile contract is available without CLI parsing dependencies.

/// Legacy profile map used by historical `config.json` files.
///
/// # Example
/// ```rust
/// use tokmd_settings::UserConfig;
///
/// let json_str = r#"
/// {
///   "profiles": {
///     "default": {
///       "format": "json"
///     }
///   },
///   "repos": {}
/// }
/// "#;
/// let config: UserConfig = serde_json::from_str(json_str).unwrap();
/// assert_eq!(config.profiles["default"].format.as_deref(), Some("json"));
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserConfig {
    pub profiles: BTreeMap<String, Profile>,
    pub repos: BTreeMap<String, String>, // "owner/repo" -> "profile_name"
}

/// Legacy profile options shared by configuration consumers.
///
/// # Example
/// ```rust
/// use tokmd_settings::Profile;
///
/// let json_str = r#"
/// {
///   "format": "json",
///   "top": 5,
///   "files": true
/// }
/// "#;
/// let profile: Profile = serde_json::from_str(json_str).unwrap();
/// assert_eq!(profile.format.as_deref(), Some("json"));
/// assert_eq!(profile.top, Some(5));
/// assert_eq!(profile.files, Some(true));
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Profile {
    // Shared
    pub format: Option<String>, // "json", "md", "tsv", "csv", "jsonl"
    pub top: Option<usize>,

    // Lang
    pub files: Option<bool>,

    // Module / Export
    pub module_roots: Option<Vec<String>>,
    pub module_depth: Option<usize>,
    pub min_code: Option<usize>,
    pub max_rows: Option<usize>,
    pub redact: Option<RedactMode>,
    pub meta: Option<bool>,

    // "children" can be ChildrenMode or ChildIncludeMode string
    pub children: Option<String>,
}
