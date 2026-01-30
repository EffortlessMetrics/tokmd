//! Interactive init wizard.

use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use std::path::Path;

/// Result of the init wizard.
#[derive(Debug, Clone)]
pub struct WizardResult {
    /// Project type selected.
    pub project_type: ProjectType,

    /// Module roots (comma-separated directories).
    pub module_roots: Vec<String>,

    /// Module depth.
    pub module_depth: usize,

    /// Context budget (token count).
    pub context_budget: String,

    /// Whether to write a tokmd.toml file.
    pub write_config: bool,

    /// Whether to write a .tokeignore file.
    pub write_tokeignore: bool,
}

/// Supported project types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Cpp,
    Mono,
    Other,
}

impl ProjectType {
    fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Rust => "rust",
            ProjectType::Node => "node",
            ProjectType::Python => "python",
            ProjectType::Go => "go",
            ProjectType::Cpp => "cpp",
            ProjectType::Mono => "mono",
            ProjectType::Other => "default",
        }
    }

    fn default_module_roots(&self) -> Vec<String> {
        match self {
            ProjectType::Rust => vec!["crates".to_string(), "src".to_string()],
            ProjectType::Node => vec![
                "packages".to_string(),
                "apps".to_string(),
                "src".to_string(),
            ],
            ProjectType::Python => vec!["src".to_string(), "lib".to_string()],
            ProjectType::Go => vec!["cmd".to_string(), "pkg".to_string(), "internal".to_string()],
            ProjectType::Cpp => vec!["src".to_string(), "include".to_string(), "lib".to_string()],
            ProjectType::Mono => vec![
                "packages".to_string(),
                "apps".to_string(),
                "libs".to_string(),
            ],
            ProjectType::Other => vec!["src".to_string()],
        }
    }
}

/// Run the interactive init wizard.
///
/// Returns `Some(WizardResult)` if the user completes the wizard,
/// or `None` if they cancel.
pub fn run_init_wizard(_dir: &Path) -> Result<Option<WizardResult>> {
    let theme = ColorfulTheme::default();

    // Welcome message
    eprintln!();
    eprintln!("{}", style("Welcome to tokmd init wizard!").bold().cyan());
    eprintln!("This wizard will help you configure tokmd for your project.");
    eprintln!();

    // Project type selection
    let project_types = &[
        "Rust (crates/, src/)",
        "Node.js (packages/, apps/, src/)",
        "Python (src/, lib/)",
        "Go (cmd/, pkg/, internal/)",
        "C/C++ (src/, include/, lib/)",
        "Monorepo (packages/, apps/, libs/)",
        "Other",
    ];

    let selection = Select::with_theme(&theme)
        .with_prompt("What type of project is this?")
        .items(project_types)
        .default(0)
        .interact_opt()
        .context("Failed to get project type selection")?;

    let project_type = match selection {
        Some(0) => ProjectType::Rust,
        Some(1) => ProjectType::Node,
        Some(2) => ProjectType::Python,
        Some(3) => ProjectType::Go,
        Some(4) => ProjectType::Cpp,
        Some(5) => ProjectType::Mono,
        Some(6) => ProjectType::Other,
        None => return Ok(None), // User cancelled
        _ => ProjectType::Other,
    };

    // Module roots
    let default_roots = project_type.default_module_roots().join(", ");
    let roots_input: String = Input::with_theme(&theme)
        .with_prompt("Module roots (comma-separated directories)")
        .default(default_roots)
        .interact_text()
        .context("Failed to get module roots")?;

    let module_roots: Vec<String> = roots_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Module depth
    let module_depth: usize = Input::with_theme(&theme)
        .with_prompt("Module depth")
        .default(2)
        .interact_text()
        .context("Failed to get module depth")?;

    // Context budget
    let context_budget: String = Input::with_theme(&theme)
        .with_prompt("Context budget (tokens)")
        .default("128k".to_string())
        .interact_text()
        .context("Failed to get context budget")?;

    // Confirmation
    eprintln!();
    eprintln!("{}", style("Configuration summary:").bold());
    eprintln!("  Project type: {}", project_type.as_str());
    eprintln!("  Module roots: {}", module_roots.join(", "));
    eprintln!("  Module depth: {}", module_depth);
    eprintln!("  Context budget: {}", context_budget);
    eprintln!();

    let write_config = Confirm::with_theme(&theme)
        .with_prompt("Write tokmd.toml configuration file?")
        .default(true)
        .interact()
        .context("Failed to get config confirmation")?;

    let write_tokeignore = Confirm::with_theme(&theme)
        .with_prompt("Write .tokeignore file?")
        .default(true)
        .interact()
        .context("Failed to get tokeignore confirmation")?;

    if !write_config && !write_tokeignore {
        eprintln!("No files to write. Init cancelled.");
        return Ok(None);
    }

    Ok(Some(WizardResult {
        project_type,
        module_roots,
        module_depth,
        context_budget,
        write_config,
        write_tokeignore,
    }))
}

/// Generate tokmd.toml content from wizard result.
///
/// Uses the `TomlConfig` struct to ensure output matches the schema exactly.
pub fn generate_toml_config(result: &WizardResult) -> Result<String> {
    use tokmd_config::{AnalyzeConfig, ContextConfig, ExportConfig, ModuleConfig, TomlConfig};

    let config = TomlConfig {
        module: ModuleConfig {
            roots: Some(result.module_roots.clone()),
            depth: Some(result.module_depth),
            ..Default::default()
        },
        export: ExportConfig {
            format: Some("jsonl".to_string()),
            min_code: Some(10),
            ..Default::default()
        },
        context: ContextConfig {
            budget: Some(result.context_budget.clone()),
            strategy: Some("greedy".to_string()),
            ..Default::default()
        },
        analyze: AnalyzeConfig {
            preset: Some("receipt".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let toml_content =
        toml::to_string_pretty(&config).context("Failed to serialize configuration to TOML")?;

    Ok(format!(
        "# tokmd configuration\n\
         # Generated by tokmd init\n\n\
         {toml_content}"
    ))
}

/// Map project type to InitProfile.
pub fn project_type_to_profile(project_type: ProjectType) -> tokmd_config::InitProfile {
    match project_type {
        ProjectType::Rust => tokmd_config::InitProfile::Rust,
        ProjectType::Node => tokmd_config::InitProfile::Node,
        ProjectType::Python => tokmd_config::InitProfile::Python,
        ProjectType::Go => tokmd_config::InitProfile::Go,
        ProjectType::Cpp => tokmd_config::InitProfile::Cpp,
        ProjectType::Mono => tokmd_config::InitProfile::Mono,
        ProjectType::Other => tokmd_config::InitProfile::Default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type_defaults() {
        assert!(!ProjectType::Rust.default_module_roots().is_empty());
        assert!(!ProjectType::Node.default_module_roots().is_empty());
    }

    #[test]
    fn test_generate_config() {
        let result = WizardResult {
            project_type: ProjectType::Rust,
            module_roots: vec!["crates".to_string(), "src".to_string()],
            module_depth: 2,
            context_budget: "128k".to_string(),
            write_config: true,
            write_tokeignore: true,
        };

        let config = generate_toml_config(&result).expect("should generate config");

        // Check header comment
        assert!(config.contains("# tokmd configuration"));
        assert!(config.contains("# Generated by tokmd init"));

        // Check module section
        assert!(config.contains("[module]"));
        assert!(config.contains("\"crates\""));
        assert!(config.contains("\"src\""));
        assert!(config.contains("depth = 2"));

        // Check export section
        assert!(config.contains("[export]"));
        assert!(config.contains("format = \"jsonl\""));
        assert!(config.contains("min_code = 10"));

        // Check context section
        assert!(config.contains("[context]"));
        assert!(config.contains("budget = \"128k\""));
        assert!(config.contains("strategy = \"greedy\""));

        // Check analyze section
        assert!(config.contains("[analyze]"));
        assert!(config.contains("preset = \"receipt\""));

        // Verify it's valid TOML by parsing
        let parsed: tokmd_config::TomlConfig =
            toml::from_str(&config).expect("generated config should be valid TOML");
        assert_eq!(parsed.module.depth, Some(2));
        assert_eq!(parsed.context.budget, Some("128k".to_string()));
    }
}
