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
    /// Get the string representation of the project type.
    pub fn as_str(&self) -> &'static str {
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

    /// Get the default module roots for this project type.
    pub fn default_module_roots(&self) -> Vec<String> {
        let profile = project_type_to_profile(*self);
        let config = profile.default_config();
        config.module.roots.unwrap_or_default()
    }
}

/// Map a selection index to a project type.
///
/// This is the pure logic extracted from `run_init_wizard` to enable deterministic testing.
/// Returns `None` if index is `None` (user cancelled).
pub fn index_to_project_type(index: Option<usize>) -> Option<ProjectType> {
    match index {
        Some(0) => Some(ProjectType::Rust),
        Some(1) => Some(ProjectType::Node),
        Some(2) => Some(ProjectType::Python),
        Some(3) => Some(ProjectType::Go),
        Some(4) => Some(ProjectType::Cpp),
        Some(5) => Some(ProjectType::Mono),
        Some(6) => Some(ProjectType::Other),
        None => None,                        // User cancelled
        Some(_) => Some(ProjectType::Other), // Fallback for out-of-bounds
    }
}

/// Pure logic for determining wizard result from collected answers.
///
/// This is extracted from `run_init_wizard` to enable deterministic testing.
/// Returns `None` if:
/// - `project_type` is `None` (user cancelled selection)
/// - Both `write_config` and `write_tokeignore` are `false` (nothing to write)
pub fn wizard_result_from_answers(
    project_type: Option<ProjectType>,
    module_roots: Vec<String>,
    module_depth: usize,
    context_budget: String,
    write_config: bool,
    write_tokeignore: bool,
) -> Option<WizardResult> {
    let project_type = project_type?;

    if !write_config && !write_tokeignore {
        return None;
    }

    Some(WizardResult {
        project_type,
        module_roots,
        module_depth,
        context_budget,
        write_config,
        write_tokeignore,
    })
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

    let project_type = match index_to_project_type(selection) {
        Some(pt) => pt,
        None => return Ok(None), // User cancelled
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

    let result = wizard_result_from_answers(
        Some(project_type),
        module_roots,
        module_depth,
        context_budget,
        write_config,
        write_tokeignore,
    );

    if result.is_none() {
        eprintln!("No files to write. Init cancelled.");
    }

    Ok(result)
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

    // ==================== ProjectType::as_str() tests ====================

    #[test]
    fn test_as_str_rust() {
        assert_eq!(ProjectType::Rust.as_str(), "rust");
    }

    #[test]
    fn test_as_str_node() {
        assert_eq!(ProjectType::Node.as_str(), "node");
    }

    #[test]
    fn test_as_str_python() {
        assert_eq!(ProjectType::Python.as_str(), "python");
    }

    #[test]
    fn test_as_str_go() {
        assert_eq!(ProjectType::Go.as_str(), "go");
    }

    #[test]
    fn test_as_str_cpp() {
        assert_eq!(ProjectType::Cpp.as_str(), "cpp");
    }

    #[test]
    fn test_as_str_mono() {
        assert_eq!(ProjectType::Mono.as_str(), "mono");
    }

    #[test]
    fn test_as_str_other() {
        assert_eq!(ProjectType::Other.as_str(), "default");
    }

    // ==================== ProjectType::default_module_roots() tests ====================

    #[test]
    fn test_default_roots_rust() {
        assert_eq!(
            ProjectType::Rust.default_module_roots(),
            vec!["crates".to_string(), "src".to_string()]
        );
    }

    #[test]
    fn test_default_roots_node() {
        assert_eq!(
            ProjectType::Node.default_module_roots(),
            vec![
                "packages".to_string(),
                "apps".to_string(),
                "src".to_string()
            ]
        );
    }

    #[test]
    fn test_default_roots_python() {
        assert_eq!(
            ProjectType::Python.default_module_roots(),
            vec!["src".to_string(), "lib".to_string()]
        );
    }

    #[test]
    fn test_default_roots_go() {
        assert_eq!(
            ProjectType::Go.default_module_roots(),
            vec!["cmd".to_string(), "pkg".to_string(), "internal".to_string()]
        );
    }

    #[test]
    fn test_default_roots_cpp() {
        assert_eq!(
            ProjectType::Cpp.default_module_roots(),
            vec!["src".to_string(), "include".to_string(), "lib".to_string()]
        );
    }

    #[test]
    fn test_default_roots_mono() {
        assert_eq!(
            ProjectType::Mono.default_module_roots(),
            vec![
                "packages".to_string(),
                "apps".to_string(),
                "libs".to_string()
            ]
        );
    }

    #[test]
    fn test_default_roots_other() {
        assert_eq!(
            ProjectType::Other.default_module_roots(),
            vec!["src".to_string()]
        );
    }

    // ==================== project_type_to_profile() tests ====================

    #[test]
    fn test_profile_rust() {
        assert_eq!(
            project_type_to_profile(ProjectType::Rust),
            tokmd_config::InitProfile::Rust
        );
    }

    #[test]
    fn test_profile_node() {
        assert_eq!(
            project_type_to_profile(ProjectType::Node),
            tokmd_config::InitProfile::Node
        );
    }

    #[test]
    fn test_profile_python() {
        assert_eq!(
            project_type_to_profile(ProjectType::Python),
            tokmd_config::InitProfile::Python
        );
    }

    #[test]
    fn test_profile_go() {
        assert_eq!(
            project_type_to_profile(ProjectType::Go),
            tokmd_config::InitProfile::Go
        );
    }

    #[test]
    fn test_profile_cpp() {
        assert_eq!(
            project_type_to_profile(ProjectType::Cpp),
            tokmd_config::InitProfile::Cpp
        );
    }

    #[test]
    fn test_profile_mono() {
        assert_eq!(
            project_type_to_profile(ProjectType::Mono),
            tokmd_config::InitProfile::Mono
        );
    }

    #[test]
    fn test_profile_other_maps_to_default() {
        assert_eq!(
            project_type_to_profile(ProjectType::Other),
            tokmd_config::InitProfile::Default
        );
    }

    // ==================== Legacy tests ====================

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

    // ==================== index_to_project_type() tests ====================

    #[test]
    fn test_index_to_project_type_rust() {
        assert_eq!(index_to_project_type(Some(0)), Some(ProjectType::Rust));
    }

    #[test]
    fn test_index_to_project_type_node() {
        assert_eq!(index_to_project_type(Some(1)), Some(ProjectType::Node));
    }

    #[test]
    fn test_index_to_project_type_python() {
        assert_eq!(index_to_project_type(Some(2)), Some(ProjectType::Python));
    }

    #[test]
    fn test_index_to_project_type_go() {
        assert_eq!(index_to_project_type(Some(3)), Some(ProjectType::Go));
    }

    #[test]
    fn test_index_to_project_type_cpp() {
        assert_eq!(index_to_project_type(Some(4)), Some(ProjectType::Cpp));
    }

    #[test]
    fn test_index_to_project_type_mono() {
        assert_eq!(index_to_project_type(Some(5)), Some(ProjectType::Mono));
    }

    #[test]
    fn test_index_to_project_type_other() {
        assert_eq!(index_to_project_type(Some(6)), Some(ProjectType::Other));
    }

    #[test]
    fn test_index_to_project_type_none_returns_none() {
        // Kills "delete None arm" mutant - user cancellation
        assert_eq!(index_to_project_type(None), None);
    }

    #[test]
    fn test_index_to_project_type_out_of_bounds_returns_other() {
        // Kills "delete fallback arm" mutant
        assert_eq!(index_to_project_type(Some(100)), Some(ProjectType::Other));
        assert_eq!(index_to_project_type(Some(7)), Some(ProjectType::Other));
    }

    // ==================== wizard_result_from_answers() tests ====================

    #[test]
    fn test_wizard_result_happy_path() {
        // Kills "wizard_result_from_answers -> None" mutant
        let result = wizard_result_from_answers(
            Some(ProjectType::Rust),
            vec!["crates".to_string()],
            2,
            "128k".to_string(),
            true,
            true,
        );
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.project_type, ProjectType::Rust);
        assert_eq!(r.module_roots, vec!["crates"]);
        assert_eq!(r.module_depth, 2);
        assert_eq!(r.context_budget, "128k");
        assert!(r.write_config);
        assert!(r.write_tokeignore);
    }

    #[test]
    fn test_wizard_result_none_project_type_returns_none() {
        // User cancelled project type selection
        let result = wizard_result_from_answers(
            None,
            vec!["src".to_string()],
            2,
            "128k".to_string(),
            true,
            true,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_wizard_result_no_files_to_write_returns_none() {
        // Kills "delete !write_config && !write_tokeignore check" mutant
        let result = wizard_result_from_answers(
            Some(ProjectType::Rust),
            vec!["src".to_string()],
            2,
            "128k".to_string(),
            false,
            false,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_wizard_result_only_config_returns_some() {
        let result = wizard_result_from_answers(
            Some(ProjectType::Python),
            vec!["src".to_string()],
            1,
            "64k".to_string(),
            true,
            false,
        );
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.write_config);
        assert!(!r.write_tokeignore);
    }

    #[test]
    fn test_wizard_result_only_tokeignore_returns_some() {
        let result = wizard_result_from_answers(
            Some(ProjectType::Node),
            vec!["packages".to_string()],
            3,
            "256k".to_string(),
            false,
            true,
        );
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(!r.write_config);
        assert!(r.write_tokeignore);
    }

    #[test]
    fn test_wizard_result_preserves_all_fields() {
        let result = wizard_result_from_answers(
            Some(ProjectType::Go),
            vec!["cmd".to_string(), "pkg".to_string()],
            5,
            "1m".to_string(),
            true,
            false,
        );
        let r = result.unwrap();
        assert_eq!(r.project_type, ProjectType::Go);
        assert_eq!(r.module_roots, vec!["cmd", "pkg"]);
        assert_eq!(r.module_depth, 5);
        assert_eq!(r.context_budget, "1m");
    }

    // ==================== Mutant killer: as_str not empty/xyzzy ====================

    #[test]
    fn test_as_str_not_empty() {
        // Kills "as_str -> empty string" mutants
        assert!(!ProjectType::Rust.as_str().is_empty());
        assert!(!ProjectType::Node.as_str().is_empty());
        assert!(!ProjectType::Python.as_str().is_empty());
        assert!(!ProjectType::Go.as_str().is_empty());
        assert!(!ProjectType::Cpp.as_str().is_empty());
        assert!(!ProjectType::Mono.as_str().is_empty());
        assert!(!ProjectType::Other.as_str().is_empty());
    }

    #[test]
    fn test_as_str_not_xyzzy() {
        // Kills "as_str -> xyzzy" mutants
        assert_ne!(ProjectType::Rust.as_str(), "xyzzy");
        assert_ne!(ProjectType::Node.as_str(), "xyzzy");
        assert_ne!(ProjectType::Python.as_str(), "xyzzy");
        assert_ne!(ProjectType::Go.as_str(), "xyzzy");
        assert_ne!(ProjectType::Cpp.as_str(), "xyzzy");
        assert_ne!(ProjectType::Mono.as_str(), "xyzzy");
        assert_ne!(ProjectType::Other.as_str(), "xyzzy");
    }

    // ==================== Mutant killer: default_module_roots not empty/xyzzy ====================

    #[test]
    fn test_default_roots_not_empty() {
        // Kills "default_module_roots -> vec![]" mutants
        assert!(!ProjectType::Rust.default_module_roots().is_empty());
        assert!(!ProjectType::Node.default_module_roots().is_empty());
        assert!(!ProjectType::Python.default_module_roots().is_empty());
        assert!(!ProjectType::Go.default_module_roots().is_empty());
        assert!(!ProjectType::Cpp.default_module_roots().is_empty());
        assert!(!ProjectType::Mono.default_module_roots().is_empty());
        assert!(!ProjectType::Other.default_module_roots().is_empty());
    }

    #[test]
    fn test_default_roots_not_xyzzy() {
        // Kills "default_module_roots -> vec![xyzzy]" mutants
        for pt in [
            ProjectType::Rust,
            ProjectType::Node,
            ProjectType::Python,
            ProjectType::Go,
            ProjectType::Cpp,
            ProjectType::Mono,
            ProjectType::Other,
        ] {
            let roots = pt.default_module_roots();
            assert!(
                !roots.contains(&"xyzzy".to_string()),
                "{pt:?} should not contain 'xyzzy'"
            );
        }
    }
}
