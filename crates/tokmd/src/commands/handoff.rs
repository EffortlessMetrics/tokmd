//! Handoff command: Bundle codebase for LLM handoff.
//!
//! Creates a `.handoff/` directory with four artifacts:
//! - `manifest.json`: Bundle metadata, budgets, capabilities
//! - `map.jsonl`: Complete file inventory (streaming)
//! - `intelligence.json`: Tree + hotspots + complexity + derived
//! - `code.txt`: Token-budgeted code bundle

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use tokmd_config as cli;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_types::{
    ArtifactEntry, CapabilityState, CapabilityStatus, ExportData, FileKind, FileRow,
    HANDOFF_SCHEMA_VERSION, HandoffComplexity, HandoffDerived, HandoffHotspot, HandoffIntelligence,
    HandoffManifest, ToolInfo,
};

use crate::context_pack;
use crate::git_scoring;
use crate::progress::Progress;

/// Handle the handoff command.
pub(crate) fn handle(args: cli::HandoffArgs, global: &cli::GlobalArgs) -> Result<()> {
    let progress = Progress::new(!global.no_progress);

    let paths = args
        .paths
        .clone()
        .unwrap_or_else(|| vec![PathBuf::from(".")]);

    // Check output directory
    if args.out_dir.exists() {
        let is_empty = args
            .out_dir
            .read_dir()
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);
        if !is_empty && !args.force {
            bail!(
                "Output directory is not empty: {}. Use --force to overwrite.",
                args.out_dir.display()
            );
        }
    }

    // Parse budget
    let budget = context_pack::parse_budget(&args.budget)?;

    // Scan and create export data
    progress.set_message("Scanning codebase...");
    let languages = scan::scan(&paths, global)?;
    let module_roots = args.module_roots.clone().unwrap_or_default();
    let module_depth = args.module_depth.unwrap_or(2);

    progress.set_message("Building export data...");
    let export = model::create_export_data(
        &languages,
        &module_roots,
        module_depth,
        cli::ChildIncludeMode::ParentsOnly,
        None,
        0, // no min_code filter
        0, // no max_rows limit
    );

    let root = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));

    // Detect capabilities
    progress.set_message("Detecting capabilities...");
    let capabilities = detect_capabilities(&root, &args);

    // Compute git scores if needed
    progress.set_message("Computing git scores...");
    let git_scores = if should_compute_git(&capabilities) {
        git_scoring::compute_git_scores(
            &root,
            &export.rows,
            args.max_commits,
            args.max_commit_files,
        )
    } else {
        None
    };

    // Select files for code bundle
    progress.set_message("Selecting files for code bundle...");
    let selected = context_pack::select_files(
        &export.rows,
        budget,
        args.strategy,
        args.rank_by,
        git_scores.as_ref(),
    );

    let used_tokens: usize = selected.iter().map(|f| f.tokens).sum();
    let utilization = if budget > 0 {
        (used_tokens as f64 / budget as f64) * 100.0
    } else {
        0.0
    };

    // Build intelligence
    progress.set_message("Building intelligence...");
    let intelligence = build_intelligence(&export, &args, &capabilities, git_scores.as_ref());

    // Write output directory
    progress.set_message("Writing handoff bundle...");
    fs::create_dir_all(&args.out_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            args.out_dir.display()
        )
    })?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    // Write map.jsonl
    let map_path = args.out_dir.join("map.jsonl");
    let map_bytes = write_map_jsonl(&map_path, &export)?;

    // Write intelligence.json
    let intel_path = args.out_dir.join("intelligence.json");
    let intel_json = serde_json::to_string_pretty(&intelligence)?;
    fs::write(&intel_path, &intel_json)
        .with_context(|| format!("Failed to write {}", intel_path.display()))?;
    let intel_bytes = intel_json.len() as u64;

    // Write code.txt
    let code_path = args.out_dir.join("code.txt");
    let code_bytes = write_code_bundle(&code_path, &selected, args.compress)?;

    // Build artifacts list
    let artifacts = vec![
        ArtifactEntry {
            name: "manifest".to_string(),
            path: "manifest.json".to_string(),
            description: "Bundle metadata and capabilities".to_string(),
            bytes: 0, // Will be updated after writing
        },
        ArtifactEntry {
            name: "map".to_string(),
            path: "map.jsonl".to_string(),
            description: "Complete file inventory".to_string(),
            bytes: map_bytes,
        },
        ArtifactEntry {
            name: "intelligence".to_string(),
            path: "intelligence.json".to_string(),
            description: "Tree, hotspots, complexity, and derived metrics".to_string(),
            bytes: intel_bytes,
        },
        ArtifactEntry {
            name: "code".to_string(),
            path: "code.txt".to_string(),
            description: "Token-budgeted code bundle".to_string(),
            bytes: code_bytes,
        },
    ];

    // Write manifest.json
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: timestamp,
        tool: ToolInfo::current(),
        mode: "handoff".to_string(),
        inputs: paths.iter().map(|p| p.display().to_string()).collect(),
        budget_tokens: budget,
        used_tokens,
        utilization_pct: round_f64(utilization, 2),
        strategy: format!("{:?}", args.strategy).to_lowercase(),
        rank_by: format!("{:?}", args.rank_by).to_lowercase(),
        capabilities: capabilities.clone(),
        artifacts,
        total_files: export
            .rows
            .iter()
            .filter(|r| r.kind == FileKind::Parent)
            .count(),
        bundled_files: selected.len(),
        intelligence_preset: format!("{:?}", args.preset).to_lowercase(),
    };

    let manifest_path = args.out_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, &manifest_json)
        .with_context(|| format!("Failed to write {}", manifest_path.display()))?;

    progress.finish_and_clear();

    // Print summary
    eprintln!("Wrote handoff bundle to {}", args.out_dir.display());
    eprintln!("  - manifest.json ({} bytes)", manifest_json.len());
    eprintln!("  - map.jsonl ({} bytes)", map_bytes);
    eprintln!("  - intelligence.json ({} bytes)", intel_bytes);
    eprintln!("  - code.txt ({} bytes)", code_bytes);
    eprintln!(
        "  - Token usage: {}/{} ({:.1}%)",
        used_tokens, budget, utilization
    );
    eprintln!(
        "  - Files: {}/{} bundled",
        selected.len(),
        manifest.total_files
    );

    Ok(())
}

/// Detect available capabilities for the handoff.
fn detect_capabilities(root: &Path, args: &cli::HandoffArgs) -> Vec<CapabilityStatus> {
    let mut capabilities = Vec::new();

    // Check git availability
    let git_available = std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if args.no_git {
        capabilities.push(CapabilityStatus {
            name: "git".to_string(),
            status: CapabilityState::Skipped,
            reason: Some("disabled via --no-git flag".to_string()),
        });
    } else if !git_available {
        capabilities.push(CapabilityStatus {
            name: "git".to_string(),
            status: CapabilityState::Unavailable,
            reason: Some("git command not found".to_string()),
        });
    } else {
        capabilities.push(CapabilityStatus {
            name: "git".to_string(),
            status: CapabilityState::Available,
            reason: None,
        });
    }

    // Check if we're in a git repository
    #[cfg(feature = "git")]
    let in_repo = tokmd_git::repo_root(root).is_some();
    #[cfg(not(feature = "git"))]
    let in_repo = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if args.no_git {
        capabilities.push(CapabilityStatus {
            name: "git_repository".to_string(),
            status: CapabilityState::Skipped,
            reason: Some("disabled via --no-git flag".to_string()),
        });
    } else if !in_repo {
        capabilities.push(CapabilityStatus {
            name: "git_repository".to_string(),
            status: CapabilityState::Unavailable,
            reason: Some("not inside a git repository".to_string()),
        });
    } else {
        capabilities.push(CapabilityStatus {
            name: "git_repository".to_string(),
            status: CapabilityState::Available,
            reason: None,
        });
    }

    // Check for shallow clone
    let shallow = std::process::Command::new("git")
        .args(["rev-parse", "--is-shallow-repository"])
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false);

    if args.no_git || !in_repo {
        capabilities.push(CapabilityStatus {
            name: "git_history".to_string(),
            status: CapabilityState::Skipped,
            reason: Some(if args.no_git {
                "disabled via --no-git flag".to_string()
            } else {
                "not in a git repository".to_string()
            }),
        });
    } else if shallow {
        capabilities.push(CapabilityStatus {
            name: "git_history".to_string(),
            status: CapabilityState::Unavailable,
            reason: Some("shallow clone detected; limited history available".to_string()),
        });
    } else {
        capabilities.push(CapabilityStatus {
            name: "git_history".to_string(),
            status: CapabilityState::Available,
            reason: None,
        });
    }

    capabilities
}

/// Check if we should compute git scores based on capabilities.
fn should_compute_git(capabilities: &[CapabilityStatus]) -> bool {
    capabilities
        .iter()
        .any(|c| c.name == "git_repository" && c.status == CapabilityState::Available)
}

/// Build intelligence data for the handoff.
fn build_intelligence(
    export: &ExportData,
    args: &cli::HandoffArgs,
    capabilities: &[CapabilityStatus],
    git_scores: Option<&git_scoring::GitScores>,
) -> HandoffIntelligence {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let mut warnings = Vec::new();

    // Build tree (always included)
    let tree = Some(build_tree(export));

    // Build hotspots (Risk/Deep presets)
    let hotspots = if matches!(
        args.preset,
        cli::HandoffPreset::Risk | cli::HandoffPreset::Deep
    ) {
        match git_scores {
            Some(scores) if !scores.hotspots.is_empty() => {
                let mut hotspot_rows: Vec<HandoffHotspot> = scores
                    .hotspots
                    .iter()
                    .map(|(path, &score)| {
                        let commits = scores.commit_counts.get(path).copied().unwrap_or(0);
                        let lines = export
                            .rows
                            .iter()
                            .find(|r| normalize_path(&r.path) == *path)
                            .map(|r| r.lines)
                            .unwrap_or(0);
                        HandoffHotspot {
                            path: path.clone(),
                            commits,
                            lines,
                            score,
                        }
                    })
                    .collect();
                // Sort by score descending, then by path
                hotspot_rows
                    .sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));
                // Limit to top 20
                hotspot_rows.truncate(20);
                Some(hotspot_rows)
            }
            _ => {
                if !args.no_git && should_compute_git(capabilities) {
                    warnings.push("hotspots unavailable: no git history found".to_string());
                }
                None
            }
        }
    } else {
        None
    };

    // Build complexity (Standard/Risk/Deep presets)
    let complexity = if matches!(
        args.preset,
        cli::HandoffPreset::Standard | cli::HandoffPreset::Risk | cli::HandoffPreset::Deep
    ) {
        Some(build_simple_complexity(export))
    } else {
        None
    };

    // Build derived (Standard/Risk/Deep presets)
    let derived = if matches!(
        args.preset,
        cli::HandoffPreset::Standard | cli::HandoffPreset::Risk | cli::HandoffPreset::Deep
    ) {
        Some(build_simple_derived(export))
    } else {
        None
    };

    HandoffIntelligence {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: timestamp,
        tool: ToolInfo::current(),
        tree,
        hotspots,
        complexity,
        derived,
        warnings,
        capabilities: capabilities.to_vec(),
    }
}

/// Build a simple directory tree from export data.
fn build_tree(export: &ExportData) -> String {
    #[derive(Default)]
    struct Node {
        children: BTreeMap<String, Node>,
        lines: usize,
        tokens: usize,
        is_file: bool,
    }

    fn insert(node: &mut Node, parts: &[&str], lines: usize, tokens: usize) {
        node.lines += lines;
        node.tokens += tokens;
        if let Some((head, tail)) = parts.split_first() {
            let child = node.children.entry(head.to_string()).or_default();
            insert(child, tail, lines, tokens);
        } else {
            node.is_file = true;
        }
    }

    fn render(node: &Node, name: &str, indent: &str, out: &mut String) {
        if !name.is_empty() {
            out.push_str(&format!(
                "{}{} (lines: {}, tokens: {})\n",
                indent, name, node.lines, node.tokens
            ));
        }
        let next_indent = if name.is_empty() {
            indent.to_string()
        } else {
            format!("{}  ", indent)
        };
        for (child_name, child) in &node.children {
            render(child, child_name, &next_indent, out);
        }
    }

    let mut root = Node::default();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        let parts: Vec<&str> = row.path.split('/').filter(|seg| !seg.is_empty()).collect();
        insert(&mut root, &parts, row.lines, row.tokens);
    }

    let mut out = String::new();
    render(&root, "", "", &mut out);
    out
}

/// Build simple complexity metrics from export data.
fn build_simple_complexity(export: &ExportData) -> HandoffComplexity {
    let parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .collect();

    let file_count = parents.len();
    if file_count == 0 {
        return HandoffComplexity {
            total_functions: 0,
            avg_function_length: 0.0,
            max_function_length: 0,
            avg_cyclomatic: 0.0,
            max_cyclomatic: 0,
            high_risk_files: 0,
        };
    }

    // Estimate functions based on code lines (rough heuristic: 1 function per 30 lines)
    let total_code: usize = parents.iter().map(|r| r.code).sum();
    let estimated_functions = (total_code / 30).max(1);
    let avg_function_length = total_code as f64 / estimated_functions as f64;
    let max_file_code = parents.iter().map(|r| r.code).max().unwrap_or(0);

    // Rough cyclomatic complexity estimate based on file size
    // This is a placeholder - actual complexity requires parsing
    let avg_cyclomatic = (total_code as f64 / file_count as f64 / 50.0).max(1.0);
    let max_cyclomatic = (max_file_code / 50).max(1);

    // High risk files: > 500 lines of code
    let high_risk_files = parents.iter().filter(|r| r.code > 500).count();

    HandoffComplexity {
        total_functions: estimated_functions,
        avg_function_length: round_f64(avg_function_length, 2),
        max_function_length: max_file_code,
        avg_cyclomatic: round_f64(avg_cyclomatic, 2),
        max_cyclomatic,
        high_risk_files,
    }
}

/// Build simple derived metrics from export data.
fn build_simple_derived(export: &ExportData) -> HandoffDerived {
    let parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .collect();

    let total_files = parents.len();
    let total_code: usize = parents.iter().map(|r| r.code).sum();
    let total_lines: usize = parents.iter().map(|r| r.lines).sum();
    let total_tokens: usize = parents.iter().map(|r| r.tokens).sum();

    // Count languages
    let mut lang_counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in &parents {
        *lang_counts.entry(row.lang.clone()).or_insert(0) += row.code;
    }
    let lang_count = lang_counts.len();

    // Find dominant language
    let (dominant_lang, dominant_code) = lang_counts
        .iter()
        .max_by_key(|(_, code)| *code)
        .map(|(lang, code)| (lang.clone(), *code))
        .unwrap_or_else(|| ("Unknown".to_string(), 0));

    let dominant_pct = if total_code > 0 {
        (dominant_code as f64 / total_code as f64) * 100.0
    } else {
        0.0
    };

    HandoffDerived {
        total_files,
        total_code,
        total_lines,
        total_tokens,
        lang_count,
        dominant_lang,
        dominant_pct: round_f64(dominant_pct, 2),
    }
}

/// Write file inventory as JSONL.
fn write_map_jsonl(path: &Path, export: &ExportData) -> Result<u64> {
    let file =
        File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);
    let mut bytes: u64 = 0;

    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        let json = serde_json::to_string(row)?;
        writeln!(writer, "{}", json)?;
        bytes += json.len() as u64 + 1; // +1 for newline
    }

    writer.flush()?;
    Ok(bytes)
}

/// Write code bundle with file contents.
fn write_code_bundle(
    path: &Path,
    selected: &[tokmd_types::ContextFileRow],
    compress: bool,
) -> Result<u64> {
    let file =
        File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);
    let mut bytes: u64 = 0;

    for ctx_file in selected {
        let file_path = PathBuf::from(&ctx_file.path);
        if !file_path.exists() {
            continue;
        }

        let header = format!("// === {} ===\n", ctx_file.path);
        writer.write_all(header.as_bytes())?;
        bytes += header.len() as u64;

        if compress {
            // Strip blank lines
            let f = File::open(&file_path)
                .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
            let reader = BufReader::new(f);
            for line in reader.lines() {
                let line =
                    line.with_context(|| format!("Failed to read file: {}", file_path.display()))?;
                if !line.trim().is_empty() {
                    writeln!(writer, "{}", line)?;
                    bytes += line.len() as u64 + 1;
                }
            }
            writeln!(writer)?;
            bytes += 1;
        } else {
            // Stream content directly
            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
            writer.write_all(content.as_bytes())?;
            bytes += content.len() as u64;
            if !content.ends_with('\n') {
                writeln!(writer)?;
                bytes += 1;
            }
            writeln!(writer)?;
            bytes += 1;
        }
    }

    writer.flush()?;
    Ok(bytes)
}

/// Normalize path for consistent matching.
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Round a float to N decimal places.
fn round_f64(value: f64, decimals: u32) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("foo/bar"), "foo/bar");
        assert_eq!(normalize_path("foo\\bar"), "foo/bar");
        assert_eq!(normalize_path("foo\\bar\\baz"), "foo/bar/baz");
    }

    #[test]
    fn test_round_f64() {
        assert_eq!(round_f64(3.14159, 2), 3.14);
        assert_eq!(round_f64(3.14159, 4), 3.1416);
        assert_eq!(round_f64(100.0, 2), 100.0);
    }

    #[test]
    fn test_build_tree_empty() {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 2,
            children: cli::ChildIncludeMode::ParentsOnly,
        };
        let tree = build_tree(&export);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_build_simple_derived_empty() {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 2,
            children: cli::ChildIncludeMode::ParentsOnly,
        };
        let derived = build_simple_derived(&export);
        assert_eq!(derived.total_files, 0);
        assert_eq!(derived.total_code, 0);
        assert_eq!(derived.lang_count, 0);
    }

    #[test]
    fn test_build_simple_complexity_empty() {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 2,
            children: cli::ChildIncludeMode::ParentsOnly,
        };
        let complexity = build_simple_complexity(&export);
        assert_eq!(complexity.total_functions, 0);
        assert_eq!(complexity.high_risk_files, 0);
    }
}
