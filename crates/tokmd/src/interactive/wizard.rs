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
            ProjectType::Node => vec!["packages".to_string(), "apps".to_string(), "src".to_string()],
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
    println!();
    println!(
        "{}",
        style("Welcome to tokmd init wizard!").bold().cyan()
    );
    println!("This wizard will help you configure tokmd for your project.");
    println!();

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
    println!();
    println!("{}", style("Configuration summary:").bold());
    println!("  Project type: {}", project_type.as_str());
    println!("  Module roots: {}", module_roots.join(", "));
    println!("  Module depth: {}", module_depth);
    println!("  Context budget: {}", context_budget);
    println!();

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
        println!("No files to write. Init cancelled.");
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
pub fn generate_toml_config(result: &WizardResult) -> String {
    let mut config = String::new();

    config.push_str("# tokmd configuration\n");
    config.push_str("# Generated by tokmd init\n\n");

    // Module section
    config.push_str("[module]\n");
    config.push_str(&format!(
        "roots = [{}]\n",
        result
            .module_roots
            .iter()
            .map(|r| format!("\"{}\"", r))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    config.push_str(&format!("depth = {}\n", result.module_depth));
    config.push('\n');

    // Export section
    config.push_str("[export]\n");
    config.push_str("format = \"jsonl\"\n");
    config.push_str("min_code = 10\n");
    config.push('\n');

    // Context section
    config.push_str("[context]\n");
    config.push_str(&format!("budget = \"{}\"\n", result.context_budget));
    config.push_str("strategy = \"greedy\"\n");
    config.push('\n');

    // Analyze section
    config.push_str("[analyze]\n");
    config.push_str("preset = \"receipt\"\n");

    config
}

/// Map project type to InitProfile.
pub fn project_type_to_profile(
    project_type: ProjectType,
) -> tokmd_config::InitProfile {
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

        let config = generate_toml_config(&result);
        assert!(config.contains("[module]"));
        assert!(config.contains("roots = [\"crates\", \"src\"]"));
        assert!(config.contains("depth = 2"));
    }
}
