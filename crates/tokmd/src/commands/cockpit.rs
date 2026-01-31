//! Handler for the `tokmd cockpit` command.
//!
//! Generates PR cockpit metrics for code review automation.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokmd_config as cli;

/// Cockpit receipt schema version.
const SCHEMA_VERSION: u32 = 1;

/// Handle the cockpit command.
pub(crate) fn handle(args: cli::CockpitArgs, _global: &cli::GlobalArgs) -> Result<()> {
    #[cfg(not(feature = "git"))]
    {
        bail!("The cockpit command requires the 'git' feature. Rebuild with --features git");
    }

    #[cfg(feature = "git")]
    {
        if !tokmd_git::git_available() {
            bail!("git is not available on PATH");
        }

        let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
        let repo_root = tokmd_git::repo_root(&cwd)
            .ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;

        let receipt = compute_cockpit(&repo_root, &args.base, &args.head)?;

        let output = match args.format {
            cli::CockpitFormat::Json => render_json(&receipt)?,
            cli::CockpitFormat::Md => render_markdown(&receipt),
            cli::CockpitFormat::Sections => render_sections(&receipt),
        };

        if let Some(output_path) = &args.output {
            let mut file = std::fs::File::create(output_path).with_context(|| {
                format!("Failed to create output file: {}", output_path.display())
            })?;
            file.write_all(output.as_bytes())?;
        } else {
            print!("{}", output);
        }

        Ok(())
    }
}

/// Cockpit receipt containing all PR metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CockpitReceipt {
    pub schema_version: u32,
    pub generated_at_ms: u64,
    pub base_ref: String,
    pub head_ref: String,
    pub change_surface: ChangeSurface,
    pub composition: Composition,
    pub code_health: CodeHealth,
    pub risk: Risk,
    pub contracts: Contracts,
    pub review_plan: Vec<ReviewItem>,
}

/// Change surface metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSurface {
    pub commits: usize,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub net_lines: i64,
    /// Churn velocity: average lines changed per commit
    pub churn_velocity: f64,
    /// Change concentration: what % of changes are in top 20% of files
    pub change_concentration: f64,
}

/// File composition breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub code_pct: f64,
    pub test_pct: f64,
    pub docs_pct: f64,
    pub config_pct: f64,
    /// Test-to-code ratio (tests / code files)
    pub test_ratio: f64,
}

/// Code health indicators for DevEx.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeHealth {
    /// Overall health score (0-100)
    pub score: u32,
    /// Health grade (A-F)
    pub grade: String,
    /// Number of large files (>500 lines) being changed
    pub large_files_touched: usize,
    /// Average file size in changed files
    pub avg_file_size: usize,
    /// Complexity indicator based on file patterns
    pub complexity_indicator: ComplexityIndicator,
    /// Files with potential issues
    pub warnings: Vec<HealthWarning>,
}

/// Complexity indicator levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityIndicator {
    Low,
    Medium,
    High,
    Critical,
}

/// Health warning for specific files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthWarning {
    pub path: String,
    pub warning_type: WarningType,
    pub message: String,
}

/// Types of health warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningType {
    LargeFile,
    HighChurn,
    LowTestCoverage,
    ComplexChange,
    BusFactor,
}

/// Risk indicators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub hotspots_touched: Vec<String>,
    pub bus_factor_warnings: Vec<String>,
    /// Overall risk level for this PR
    pub level: RiskLevel,
    /// Risk score (0-100)
    pub score: u32,
}

/// Risk level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Contract change indicators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contracts {
    pub api_changed: bool,
    pub cli_changed: bool,
    pub schema_changed: bool,
    /// Number of breaking change indicators
    pub breaking_indicators: usize,
}

/// Review plan item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub path: String,
    pub reason: String,
    pub priority: u32,
    /// Estimated review complexity (1-5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<u8>,
    /// Lines changed in this file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_changed: Option<usize>,
}

#[cfg(feature = "git")]
fn compute_cockpit(repo_root: &PathBuf, base: &str, head: &str) -> Result<CockpitReceipt> {
    let generated_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Get changed files with their stats
    let file_stats = get_file_stats(repo_root, base, head)?;
    let changed_files: Vec<String> = file_stats.iter().map(|f| f.path.clone()).collect();

    // Get change surface from git
    let change_surface = compute_change_surface(repo_root, base, head, &file_stats)?;

    // Compute composition with test ratio
    let composition = compute_composition(&changed_files);

    // Detect contract changes
    let contracts = detect_contracts(&changed_files);

    // Compute code health
    let code_health = compute_code_health(&file_stats, &contracts);

    // Compute risk based on various factors
    let risk = compute_risk(&file_stats, &contracts, &code_health);

    // Generate review plan with complexity scores
    let review_plan = generate_review_plan(&file_stats, &contracts);

    Ok(CockpitReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms,
        base_ref: base.to_string(),
        head_ref: head.to_string(),
        change_surface,
        composition,
        code_health,
        risk,
        contracts,
        review_plan,
    })
}

/// Per-file statistics from git diff.
#[derive(Debug, Clone)]
struct FileStats {
    path: String,
    insertions: usize,
    deletions: usize,
}

impl FileStats {
    fn total_lines(&self) -> usize {
        self.insertions + self.deletions
    }
}

#[cfg(feature = "git")]
fn get_file_stats(repo_root: &PathBuf, base: &str, head: &str) -> Result<Vec<FileStats>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("diff")
        .arg("--numstat")
        .arg(format!("{}...{}", base, head))
        .output()
        .context("Failed to run git diff --numstat")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff --numstat failed: {}", stderr.trim());
    }

    let stats_str = String::from_utf8_lossy(&output.stdout);
    let mut stats = Vec::new();

    for line in stats_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            // Handle binary files (shown as -)
            let insertions = parts[0].parse().unwrap_or(0);
            let deletions = parts[1].parse().unwrap_or(0);
            let path = parts[2..].join(" ");

            stats.push(FileStats {
                path,
                insertions,
                deletions,
            });
        }
    }

    Ok(stats)
}

#[cfg(feature = "git")]
fn compute_change_surface(repo_root: &PathBuf, base: &str, head: &str) -> Result<ChangeSurface> {
    // Get commit count
    let commits = get_commit_count(repo_root, base, head)?;

    // Get diff stats
    let (files_changed, insertions, deletions) = get_diff_stats(repo_root, base, head)?;

    let net_lines = insertions as i64 - deletions as i64;

    Ok(ChangeSurface {
        commits,
        files_changed,
        insertions,
        deletions,
        net_lines,
    })
}

#[cfg(feature = "git")]
fn get_commit_count(repo_root: &PathBuf, base: &str, head: &str) -> Result<usize> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("rev-list")
        .arg("--count")
        .arg(format!("{}..{}", base, head))
        .output()
        .context("Failed to run git rev-list")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git rev-list failed: {}", stderr.trim());
    }

    let count_str = String::from_utf8_lossy(&output.stdout);
    count_str
        .trim()
        .parse::<usize>()
        .context("Failed to parse commit count")
}

#[cfg(feature = "git")]
fn get_diff_stats(repo_root: &PathBuf, base: &str, head: &str) -> Result<(usize, usize, usize)> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("diff")
        .arg("--shortstat")
        .arg(format!("{}...{}", base, head))
        .output()
        .context("Failed to run git diff --shortstat")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff failed: {}", stderr.trim());
    }

    let stat_str = String::from_utf8_lossy(&output.stdout);
    parse_shortstat(&stat_str)
}

fn parse_shortstat(s: &str) -> Result<(usize, usize, usize)> {
    // Example: " 5 files changed, 150 insertions(+), 30 deletions(-)"
    let s = s.trim();
    if s.is_empty() {
        return Ok((0, 0, 0));
    }

    let mut files = 0;
    let mut insertions = 0;
    let mut deletions = 0;

    for part in s.split(", ") {
        let part = part.trim();
        if part.contains("file") {
            if let Some(num) = part.split_whitespace().next() {
                files = num.parse().unwrap_or(0);
            }
        } else if part.contains("insertion")
            && let Some(num) = part.split_whitespace().next()
        {
            insertions = num.parse().unwrap_or(0);
        } else if part.contains("deletion")
            && let Some(num) = part.split_whitespace().next()
        {
            deletions = num.parse().unwrap_or(0);
        }
    }

    Ok((files, insertions, deletions))
}

#[cfg(feature = "git")]
fn get_changed_files(repo_root: &PathBuf, base: &str, head: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("diff")
        .arg("--name-only")
        .arg(format!("{}...{}", base, head))
        .output()
        .context("Failed to run git diff --name-only")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff --name-only failed: {}", stderr.trim());
    }

    let files_str = String::from_utf8_lossy(&output.stdout);
    Ok(files_str
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

/// File classification for composition analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FileCategory {
    Code,
    Test,
    Docs,
    Config,
}

fn classify_file(path: &str) -> FileCategory {
    let path_lower = path.to_lowercase();

    // Test patterns
    if path_lower.contains("/tests/")
        || path_lower.contains("/test/")
        || path_lower.starts_with("tests/")
        || path_lower.starts_with("test/")
        || path_lower.contains("_test.")
        || path_lower.contains(".test.")
        || path_lower.contains("_spec.")
        || path_lower.ends_with("_test.rs")
        || path_lower.ends_with("_tests.rs")
    {
        return FileCategory::Test;
    }

    // Docs patterns
    if path_lower.ends_with(".md")
        || path_lower.starts_with("docs/")
        || path_lower.contains("/docs/")
        || path_lower.contains("readme")
    {
        return FileCategory::Docs;
    }

    // Config/CI patterns
    if path_lower.starts_with(".github/")
        || path_lower.ends_with(".toml")
        || path_lower.ends_with(".yml")
        || path_lower.ends_with(".yaml")
        || path_lower.ends_with(".json")
        || path_lower == "justfile"
        || path_lower == "makefile"
        || path_lower.ends_with(".lock")
    {
        return FileCategory::Config;
    }

    // Everything else is code
    FileCategory::Code
}

fn compute_composition(files: &[String]) -> Composition {
    if files.is_empty() {
        return Composition {
            code_pct: 0.0,
            test_pct: 0.0,
            docs_pct: 0.0,
            config_pct: 0.0,
        };
    }

    let mut counts: BTreeMap<FileCategory, usize> = BTreeMap::new();
    for file in files {
        let cat = classify_file(file);
        *counts.entry(cat).or_insert(0) += 1;
    }

    let total = files.len() as f64;
    let code = *counts.get(&FileCategory::Code).unwrap_or(&0) as f64;
    let test = *counts.get(&FileCategory::Test).unwrap_or(&0) as f64;
    let docs = *counts.get(&FileCategory::Docs).unwrap_or(&0) as f64;
    let config = *counts.get(&FileCategory::Config).unwrap_or(&0) as f64;

    Composition {
        code_pct: round_pct(code / total * 100.0),
        test_pct: round_pct(test / total * 100.0),
        docs_pct: round_pct(docs / total * 100.0),
        config_pct: round_pct(config / total * 100.0),
    }
}

fn round_pct(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// Contract detection patterns.
fn detect_contracts(files: &[String]) -> Contracts {
    let mut api_changed = false;
    let mut cli_changed = false;
    let mut schema_changed = false;

    for file in files {
        // API changes: lib.rs files in crates
        if file.contains("crates/") && file.ends_with("/src/lib.rs") {
            api_changed = true;
        }
        if file.ends_with("/mod.rs") {
            api_changed = true;
        }

        // CLI changes
        if file.contains("crates/tokmd/src/commands/") {
            cli_changed = true;
        }
        if file.contains("crates/tokmd-config/") {
            cli_changed = true;
        }

        // Schema changes
        if file == "docs/schema.json" {
            schema_changed = true;
        }
        if file.contains("crates/tokmd-types/") {
            schema_changed = true;
        }
        if file.contains("crates/tokmd-analysis-types/") {
            schema_changed = true;
        }
    }

    Contracts {
        api_changed,
        cli_changed,
        schema_changed,
    }
}

fn generate_review_plan(files: &[String], contracts: &Contracts) -> Vec<ReviewItem> {
    let mut items: Vec<ReviewItem> = Vec::new();
    let mut priority = 1u32;

    // Priority 1: Schema changes (high impact)
    if contracts.schema_changed {
        for file in files {
            if file == "docs/schema.json"
                || file.contains("crates/tokmd-types/")
                || file.contains("crates/tokmd-analysis-types/")
            {
                items.push(ReviewItem {
                    path: file.clone(),
                    reason: "Schema change".to_string(),
                    priority,
                });
            }
        }
        priority += 1;
    }

    // Priority 2: API changes
    if contracts.api_changed {
        for file in files {
            if ((file.contains("crates/") && file.ends_with("/src/lib.rs"))
                || file.ends_with("/mod.rs"))
                && !items.iter().any(|i| i.path == *file)
            {
                items.push(ReviewItem {
                    path: file.clone(),
                    reason: "API surface".to_string(),
                    priority,
                });
            }
        }
        priority += 1;
    }

    // Priority 3: CLI changes
    if contracts.cli_changed {
        for file in files {
            if (file.contains("crates/tokmd/src/commands/")
                || file.contains("crates/tokmd-config/"))
                && !items.iter().any(|i| i.path == *file)
            {
                items.push(ReviewItem {
                    path: file.clone(),
                    reason: "CLI interface".to_string(),
                    priority,
                });
            }
        }
        priority += 1;
    }

    // Priority 4: Test files
    for file in files {
        if classify_file(file) == FileCategory::Test && !items.iter().any(|i| i.path == *file) {
            items.push(ReviewItem {
                path: file.clone(),
                reason: "Test coverage".to_string(),
                priority,
            });
        }
    }
    if items.iter().any(|i| i.reason == "Test coverage") {
        priority += 1;
    }

    // Priority 5: Remaining code files
    for file in files {
        if !items.iter().any(|i| i.path == *file) {
            let cat = classify_file(file);
            let reason = match cat {
                FileCategory::Code => "Implementation".to_string(),
                FileCategory::Docs => "Documentation".to_string(),
                FileCategory::Config => "Configuration".to_string(),
                FileCategory::Test => "Test".to_string(),
            };
            items.push(ReviewItem {
                path: file.clone(),
                reason,
                priority,
            });
        }
    }

    items
}

fn render_json(receipt: &CockpitReceipt) -> Result<String> {
    serde_json::to_string_pretty(receipt).context("Failed to serialize cockpit receipt")
}

fn render_markdown(receipt: &CockpitReceipt) -> String {
    let mut out = String::new();

    out.push_str("## Glass Cockpit\n\n");

    // Change Surface
    out.push_str("### Change Surface\n\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|-------|\n");
    out.push_str(&format!(
        "| Commits | {} |\n",
        receipt.change_surface.commits
    ));
    out.push_str(&format!(
        "| Files changed | {} |\n",
        receipt.change_surface.files_changed
    ));
    out.push_str(&format!(
        "| Insertions | +{} |\n",
        receipt.change_surface.insertions
    ));
    out.push_str(&format!(
        "| Deletions | -{} |\n",
        receipt.change_surface.deletions
    ));
    out.push_str(&format!(
        "| Net lines | {} |\n",
        receipt.change_surface.net_lines
    ));
    out.push('\n');

    // Composition
    out.push_str("### Composition\n\n");
    out.push_str("| Category | Percentage |\n");
    out.push_str("|----------|------------|\n");
    out.push_str(&format!(
        "| Code | {:.1}% |\n",
        receipt.composition.code_pct
    ));
    out.push_str(&format!(
        "| Tests | {:.1}% |\n",
        receipt.composition.test_pct
    ));
    out.push_str(&format!(
        "| Docs | {:.1}% |\n",
        receipt.composition.docs_pct
    ));
    out.push_str(&format!(
        "| Config | {:.1}% |\n",
        receipt.composition.config_pct
    ));
    out.push('\n');

    // Contracts
    out.push_str("### Contracts\n\n");
    out.push_str("| Contract | Changed |\n");
    out.push_str("|----------|:-------:|\n");
    out.push_str(&format!(
        "| API | {} |\n",
        if receipt.contracts.api_changed {
            "Yes"
        } else {
            "No"
        }
    ));
    out.push_str(&format!(
        "| CLI | {} |\n",
        if receipt.contracts.cli_changed {
            "Yes"
        } else {
            "No"
        }
    ));
    out.push_str(&format!(
        "| Schema | {} |\n",
        if receipt.contracts.schema_changed {
            "Yes"
        } else {
            "No"
        }
    ));
    out.push('\n');

    // Review Plan
    out.push_str("### Review Plan\n\n");
    out.push_str("| Priority | File | Reason |\n");
    out.push_str("|:--------:|------|--------|\n");
    for item in &receipt.review_plan {
        out.push_str(&format!(
            "| {} | `{}` | {} |\n",
            item.priority, item.path, item.reason
        ));
    }
    out.push('\n');

    out
}

fn render_sections(receipt: &CockpitReceipt) -> String {
    let mut out = String::new();

    // COCKPIT section (for AI-FILL:COCKPIT)
    out.push_str("<!-- SECTION:COCKPIT -->\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|--------|-------|\n");
    out.push_str("| **Change Surface** | |\n");
    out.push_str(&format!(
        "| Commits | {} |\n",
        receipt.change_surface.commits
    ));
    out.push_str(&format!(
        "| Files changed | {} |\n",
        receipt.change_surface.files_changed
    ));
    out.push_str(&format!(
        "| Lines (+/-) | +{}/-{} |\n",
        receipt.change_surface.insertions, receipt.change_surface.deletions
    ));
    out.push_str(&format!(
        "| Net lines | {} |\n",
        receipt.change_surface.net_lines
    ));
    out.push_str("| **Composition** | |\n");
    out.push_str(&format!(
        "| Code | {:.1}% |\n",
        receipt.composition.code_pct
    ));
    out.push_str(&format!(
        "| Tests | {:.1}% |\n",
        receipt.composition.test_pct
    ));
    out.push_str(&format!(
        "| Docs | {:.1}% |\n",
        receipt.composition.docs_pct
    ));
    out.push_str(&format!(
        "| Config | {:.1}% |\n",
        receipt.composition.config_pct
    ));
    out.push_str("| **Contracts** | |\n");
    out.push_str(&format!(
        "| API changed | {} |\n",
        if receipt.contracts.api_changed {
            "Yes"
        } else {
            "No"
        }
    ));
    out.push_str(&format!(
        "| CLI changed | {} |\n",
        if receipt.contracts.cli_changed {
            "Yes"
        } else {
            "No"
        }
    ));
    out.push_str(&format!(
        "| Schema changed | {} |\n",
        if receipt.contracts.schema_changed {
            "Yes"
        } else {
            "No"
        }
    ));
    out.push_str("<!-- /SECTION:COCKPIT -->\n\n");

    // REVIEW_PLAN section (for AI-FILL:REVIEW_PLAN)
    out.push_str("<!-- SECTION:REVIEW_PLAN -->\n");
    out.push_str("| Priority | File | Reason |\n");
    out.push_str("|----------|------|--------|\n");
    for item in &receipt.review_plan {
        out.push_str(&format!(
            "| {} | `{}` | {} |\n",
            item.priority, item.path, item.reason
        ));
    }
    out.push_str("<!-- /SECTION:REVIEW_PLAN -->\n\n");

    // RECEIPTS section (full JSON)
    out.push_str("<!-- SECTION:RECEIPTS -->\n");
    out.push_str("```json\n");
    if let Ok(json) = serde_json::to_string_pretty(receipt) {
        out.push_str(&json);
    }
    out.push_str("\n```\n");
    out.push_str("<!-- /SECTION:RECEIPTS -->\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_file_code() {
        assert_eq!(classify_file("src/lib.rs"), FileCategory::Code);
        assert_eq!(classify_file("crates/foo/src/main.rs"), FileCategory::Code);
    }

    #[test]
    fn test_classify_file_test() {
        assert_eq!(classify_file("tests/integration.rs"), FileCategory::Test);
        assert_eq!(classify_file("src/foo_test.rs"), FileCategory::Test);
        assert_eq!(classify_file("crates/bar/tests/it.rs"), FileCategory::Test);
    }

    #[test]
    fn test_classify_file_docs() {
        assert_eq!(classify_file("README.md"), FileCategory::Docs);
        assert_eq!(classify_file("docs/guide.md"), FileCategory::Docs);
        assert_eq!(classify_file("CHANGELOG.md"), FileCategory::Docs);
    }

    #[test]
    fn test_classify_file_config() {
        assert_eq!(classify_file("Cargo.toml"), FileCategory::Config);
        assert_eq!(
            classify_file(".github/workflows/ci.yml"),
            FileCategory::Config
        );
        assert_eq!(classify_file("Justfile"), FileCategory::Config);
    }

    #[test]
    fn test_parse_shortstat() {
        let (files, ins, del) =
            parse_shortstat(" 5 files changed, 150 insertions(+), 30 deletions(-)").unwrap();
        assert_eq!(files, 5);
        assert_eq!(ins, 150);
        assert_eq!(del, 30);
    }

    #[test]
    fn test_parse_shortstat_empty() {
        let (files, ins, del) = parse_shortstat("").unwrap();
        assert_eq!(files, 0);
        assert_eq!(ins, 0);
        assert_eq!(del, 0);
    }

    #[test]
    fn test_parse_shortstat_insertions_only() {
        let (files, ins, del) = parse_shortstat(" 2 files changed, 50 insertions(+)").unwrap();
        assert_eq!(files, 2);
        assert_eq!(ins, 50);
        assert_eq!(del, 0);
    }

    #[test]
    fn test_compute_composition() {
        let files = vec![
            "src/lib.rs".to_string(),
            "src/main.rs".to_string(),
            "tests/test.rs".to_string(),
            "README.md".to_string(),
            "Cargo.toml".to_string(),
        ];
        let comp = compute_composition(&files);
        assert_eq!(comp.code_pct, 40.0); // 2/5
        assert_eq!(comp.test_pct, 20.0); // 1/5
        assert_eq!(comp.docs_pct, 20.0); // 1/5
        assert_eq!(comp.config_pct, 20.0); // 1/5
    }

    #[test]
    fn test_detect_contracts_api() {
        let files = vec!["crates/tokmd-types/src/lib.rs".to_string()];
        let contracts = detect_contracts(&files);
        assert!(contracts.api_changed);
        assert!(contracts.schema_changed);
    }

    #[test]
    fn test_detect_contracts_cli() {
        let files = vec!["crates/tokmd/src/commands/cockpit.rs".to_string()];
        let contracts = detect_contracts(&files);
        assert!(contracts.cli_changed);
    }

    #[test]
    fn test_detect_contracts_schema() {
        let files = vec!["docs/schema.json".to_string()];
        let contracts = detect_contracts(&files);
        assert!(contracts.schema_changed);
    }
}
