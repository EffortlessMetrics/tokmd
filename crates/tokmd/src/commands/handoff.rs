//! Handoff command: Bundle codebase for LLM handoff.
//!
//! Creates a `.handoff/` directory with four artifacts:
//! - `manifest.json`: Bundle metadata, budgets, capabilities
//! - `map.jsonl`: Complete file inventory (streaming)
//! - `intelligence.json`: Tree + hotspots + complexity + derived
//! - `code.txt`: Token-budgeted code bundle

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use blake3::Hasher;
use tokmd_config as cli;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_types::{
    ArtifactEntry, ArtifactHash, CapabilityState, CapabilityStatus, ExportData, FileKind, FileRow,
    HANDOFF_SCHEMA_VERSION, HandoffComplexity, HandoffDerived, HandoffExcludedPath, HandoffHotspot,
    HandoffIntelligence, HandoffManifest, ToolInfo,
};

use crate::context_pack;
use tokmd_progress::Progress;

const DEFAULT_TREE_DEPTH: usize = 4;

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

    let root = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));

    // Scan and create export data
    progress.set_message("Scanning codebase...");
    let mut scan_args = global.clone();
    let excluded_paths = exclude_output_dir(&root, &args.out_dir, &mut scan_args);
    let scan_opts = tokmd_settings::ScanOptions::from(&scan_args);
    let languages = scan::scan(&paths, &scan_opts)?;
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

    // Detect capabilities
    progress.set_message("Detecting capabilities...");
    let capabilities = detect_capabilities(&root, &args);

    // Compute git scores if needed
    progress.set_message("Computing git scores...");
    let git_scores = if should_compute_git(&capabilities) {
        tokmd_context_git::compute_git_scores(
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
    let select_result = context_pack::select_files_with_options(
        &export.rows,
        budget,
        args.strategy,
        args.rank_by,
        git_scores.as_ref(),
        &context_pack::SelectOptions {
            no_smart_exclude: args.no_smart_exclude,
            max_file_pct: args.max_file_pct,
            max_file_tokens: args.max_file_tokens,
            ..Default::default()
        },
    );
    let selected = select_result.selected;
    let smart_excluded_files = select_result.smart_excluded;

    let used_tokens: usize = selected
        .iter()
        .map(|f| f.effective_tokens.unwrap_or(f.tokens))
        .sum();
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
    let map_hash = hash_file(&map_path)?;

    // Write intelligence.json
    let intel_path = args.out_dir.join("intelligence.json");
    let intel_json = serde_json::to_string_pretty(&intelligence)?;
    fs::write(&intel_path, &intel_json)
        .with_context(|| format!("Failed to write {}", intel_path.display()))?;
    let intel_bytes = intel_json.len() as u64;
    let intel_hash = hash_bytes(intel_json.as_bytes());

    // Write code.txt
    let code_path = args.out_dir.join("code.txt");
    let code_bytes = write_code_bundle(&code_path, &selected, args.compress)?;
    let code_hash = hash_file(&code_path)?;

    // Build artifacts list
    let artifacts = vec![
        ArtifactEntry {
            name: "manifest".to_string(),
            path: "manifest.json".to_string(),
            description: "Bundle metadata and capabilities".to_string(),
            bytes: 0, // Self-referential hash is omitted
            hash: None,
        },
        ArtifactEntry {
            name: "map".to_string(),
            path: "map.jsonl".to_string(),
            description: "Complete file inventory".to_string(),
            bytes: map_bytes,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: map_hash,
            }),
        },
        ArtifactEntry {
            name: "intelligence".to_string(),
            path: "intelligence.json".to_string(),
            description: "Tree, hotspots, complexity, and derived metrics".to_string(),
            bytes: intel_bytes,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: intel_hash,
            }),
        },
        ArtifactEntry {
            name: "code".to_string(),
            path: "code.txt".to_string(),
            description: "Token-budgeted code bundle".to_string(),
            bytes: code_bytes,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: code_hash,
            }),
        },
    ];

    // Compute token estimation and audit
    let total_file_bytes: usize = selected.iter().map(|f| f.bytes).sum();
    let token_estimation = tokmd_types::TokenEstimationMeta::from_bytes(total_file_bytes, 4.0);
    let code_audit =
        tokmd_types::TokenAudit::from_output(code_bytes as u64, total_file_bytes as u64);

    // Write manifest.json
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: timestamp,
        tool: ToolInfo::current(),
        mode: "handoff".to_string(),
        inputs: paths.iter().map(|p| p.display().to_string()).collect(),
        output_dir: args.out_dir.display().to_string(),
        budget_tokens: budget,
        used_tokens,
        utilization_pct: round_f64(utilization, 2),
        strategy: format!("{:?}", args.strategy).to_lowercase(),
        rank_by: format!("{:?}", args.rank_by).to_lowercase(),
        capabilities: capabilities.clone(),
        artifacts,
        included_files: selected.clone(),
        excluded_paths: excluded_paths.clone(),
        excluded_patterns: scan_args.excluded.clone(),
        smart_excluded_files,
        total_files: export
            .rows
            .iter()
            .filter(|r| r.kind == FileKind::Parent)
            .count(),
        bundled_files: selected.len(),
        intelligence_preset: format!("{:?}", args.preset).to_lowercase(),
        rank_by_effective: if select_result.fallback_reason.is_some() {
            Some(select_result.rank_by_effective.clone())
        } else {
            None
        },
        fallback_reason: select_result.fallback_reason.clone(),
        excluded_by_policy: select_result.excluded_by_policy.clone(),
        token_estimation: Some(token_estimation),
        code_audit: Some(code_audit),
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
fn capability_state(capabilities: &[CapabilityStatus], name: &str) -> Option<CapabilityState> {
    capabilities
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.status)
}

fn capability_reason(capabilities: &[CapabilityStatus], name: &str) -> Option<String> {
    capabilities
        .iter()
        .find(|c| c.name == name)
        .and_then(|c| c.reason.clone())
}

fn should_compute_git(capabilities: &[CapabilityStatus]) -> bool {
    capability_state(capabilities, "git_history") == Some(CapabilityState::Available)
}

/// Build intelligence data for the handoff.
fn build_intelligence(
    export: &ExportData,
    args: &cli::HandoffArgs,
    capabilities: &[CapabilityStatus],
    git_scores: Option<&tokmd_context_git::GitScores>,
) -> HandoffIntelligence {
    let mut warnings = Vec::new();

    // Build tree (always included)
    let tree = Some(build_tree(export, DEFAULT_TREE_DEPTH));
    let tree_depth = tree.as_ref().map(|_| DEFAULT_TREE_DEPTH);

    // Build hotspots (Risk/Deep presets)
    let wants_hotspots = matches!(
        args.preset,
        cli::HandoffPreset::Risk | cli::HandoffPreset::Deep
    );
    let hotspots = if wants_hotspots {
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
                let state = capability_state(capabilities, "git_history");
                if wants_hotspots {
                    let reason = capability_reason(capabilities, "git_history");
                    match state {
                        Some(CapabilityState::Available) => {
                            warnings.push("hotspots unavailable: no git history found".to_string());
                        }
                        Some(CapabilityState::Skipped) => {
                            let msg = if let Some(r) = reason {
                                format!("hotspots unavailable: git history skipped ({})", r)
                            } else {
                                "hotspots unavailable: git history skipped".to_string()
                            };
                            warnings.push(msg);
                        }
                        Some(CapabilityState::Unavailable) => {
                            let msg = if let Some(r) = reason {
                                format!("hotspots unavailable: git history unavailable ({})", r)
                            } else {
                                "hotspots unavailable: git history unavailable".to_string()
                            };
                            warnings.push(msg);
                        }
                        None => {}
                    }
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
        tree,
        tree_depth,
        hotspots,
        complexity,
        derived,
        warnings,
    }
}

/// Build a simple directory tree from export data.
fn build_tree(export: &ExportData, max_depth: usize) -> String {
    #[derive(Default)]
    struct Node {
        children: BTreeMap<String, Node>,
        files: usize,
        lines: usize,
        tokens: usize,
    }

    fn insert(node: &mut Node, parts: &[&str], lines: usize, tokens: usize) {
        node.files += 1;
        node.lines += lines;
        node.tokens += tokens;
        if let Some((head, tail)) = parts.split_first()
            && !tail.is_empty()
        {
            let child = node.children.entry(head.to_string()).or_default();
            insert(child, tail, lines, tokens);
        }
    }

    fn render(
        node: &Node,
        name: &str,
        indent: &str,
        depth: usize,
        max_depth: usize,
        out: &mut String,
    ) {
        let display = if name.is_empty() {
            "".to_string()
        } else if name == "(root)" {
            name.to_string()
        } else {
            format!("{}/", name)
        };
        if !display.is_empty() {
            out.push_str(&format!(
                "{}{} (files: {}, lines: {}, tokens: {})\n",
                indent, display, node.files, node.lines, node.tokens
            ));
        }
        if depth >= max_depth {
            return;
        }
        let next_indent = format!("{}  ", indent);
        for (child_name, child) in &node.children {
            render(child, child_name, &next_indent, depth + 1, max_depth, out);
        }
    }

    let mut root = Node::default();
    let parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .collect();
    if parents.is_empty() {
        return String::new();
    }
    for row in parents {
        let parts: Vec<&str> = row.path.split('/').filter(|seg| !seg.is_empty()).collect();
        insert(&mut root, &parts, row.lines, row.tokens);
    }

    let mut out = String::new();
    render(&root, "(root)", "", 0, max_depth, &mut out);
    out
}

/// Maximum number of files to analyze for complexity.
const MAX_COMPLEXITY_FILES: usize = 50;
/// Maximum bytes to read per file for complexity analysis.
const MAX_COMPLEXITY_BYTES: usize = 128 * 1024;

/// Build complexity metrics by reading source files and counting functions/branching.
fn build_simple_complexity(export: &ExportData) -> HandoffComplexity {
    let mut parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .filter(|r| is_analyzable_lang(&r.lang))
        .collect();

    if parents.is_empty() {
        return HandoffComplexity {
            total_functions: 0,
            avg_function_length: 0.0,
            max_function_length: 0,
            avg_cyclomatic: 0.0,
            max_cyclomatic: 0,
            high_risk_files: 0,
        };
    }

    // Sort by code lines descending, take top files
    parents.sort_by(|a, b| b.code.cmp(&a.code));
    parents.truncate(MAX_COMPLEXITY_FILES);

    let mut total_functions: usize = 0;
    let mut all_function_lengths: Vec<usize> = Vec::new();
    let mut max_function_length: usize = 0;
    let mut file_cyclomatic: Vec<usize> = Vec::new();
    let mut max_cyclomatic: usize = 0;
    let mut high_risk_files: usize = 0;

    for row in &parents {
        let path = PathBuf::from(&row.path);
        let content = match read_file_capped(&path, MAX_COMPLEXITY_BYTES) {
            Some(c) => c,
            None => continue,
        };

        let (fn_count, fn_max_len) = count_functions_simple(&row.lang, &content);
        let cyclomatic = estimate_cyclomatic_simple(&row.lang, &content);

        total_functions += fn_count;
        if fn_max_len > 0 {
            all_function_lengths.push(fn_max_len);
        }
        max_function_length = max_function_length.max(fn_max_len);
        file_cyclomatic.push(cyclomatic);
        max_cyclomatic = max_cyclomatic.max(cyclomatic);

        // High risk: high cyclomatic OR very long functions
        if cyclomatic > 20 || fn_max_len > 100 {
            high_risk_files += 1;
        }
    }

    let avg_function_length = if total_functions == 0 {
        0.0
    } else {
        let total_len: usize = all_function_lengths.iter().sum();
        total_len as f64 / all_function_lengths.len().max(1) as f64
    };

    let avg_cyclomatic = if file_cyclomatic.is_empty() {
        0.0
    } else {
        let total: usize = file_cyclomatic.iter().sum();
        total as f64 / file_cyclomatic.len() as f64
    };

    HandoffComplexity {
        total_functions,
        avg_function_length: round_f64(avg_function_length, 2),
        max_function_length,
        avg_cyclomatic: round_f64(avg_cyclomatic, 2),
        max_cyclomatic,
        high_risk_files,
    }
}

/// Check if a language is analyzable for complexity.
fn is_analyzable_lang(lang: &str) -> bool {
    matches!(
        lang.to_lowercase().as_str(),
        "rust"
            | "javascript"
            | "typescript"
            | "python"
            | "go"
            | "c"
            | "c++"
            | "java"
            | "c#"
            | "php"
            | "ruby"
    )
}

/// Read file contents up to a byte cap. Returns None if unreadable.
fn read_file_capped(path: &Path, max_bytes: usize) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut buf = vec![0u8; max_bytes];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);
    String::from_utf8(buf).ok()
}

/// Count functions and estimate max function length in lines.
/// Simplified inline version that avoids heavy dependencies.
fn count_functions_simple(lang: &str, text: &str) -> (usize, usize) {
    let lines: Vec<&str> = text.lines().collect();
    match lang.to_lowercase().as_str() {
        "rust" => count_brace_functions(&lines, is_rust_fn_start_simple),
        "go" => count_brace_functions(&lines, |t| t.starts_with("func ")),
        "javascript" | "typescript" => count_brace_functions(&lines, |t| {
            t.starts_with("function ")
                || t.starts_with("async function ")
                || t.starts_with("export function ")
                || t.starts_with("export async function ")
                || (t.contains("=> {") && !t.starts_with("//"))
        }),
        "c" | "c++" | "java" | "c#" | "php" => count_brace_functions(&lines, |t| {
            (t.ends_with(") {") || t.ends_with("){"))
                && !t.starts_with("if ")
                && !t.starts_with("if(")
                && !t.starts_with("while ")
                && !t.starts_with("while(")
                && !t.starts_with("for ")
                && !t.starts_with("for(")
                && !t.starts_with("switch ")
                && !t.starts_with("//")
        }),
        "python" => count_python_functions_simple(&lines),
        "ruby" => count_ruby_functions_simple(&lines),
        _ => (0, 0),
    }
}

/// Check if a trimmed line starts a Rust function definition.
/// Handles all visibility qualifiers including `pub(in path)`, extern "ABI", etc.
fn is_rust_fn_start_simple(trimmed: &str) -> bool {
    let Some(fn_pos) = trimmed.find("fn ") else {
        return false;
    };
    let prefix = trimmed[..fn_pos].trim();
    if prefix.is_empty() {
        return true;
    }
    let mut rest = prefix;
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with("pub(") {
            if let Some(close) = rest.find(')') {
                rest = &rest[close + 1..];
            } else {
                return false;
            }
        } else if let Some(r) = rest.strip_prefix("pub") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("async") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("unsafe") {
            rest = r;
        } else if let Some(r) = rest.strip_prefix("const") {
            rest = r;
        } else if rest.starts_with("extern") {
            rest = rest["extern".len()..].trim_start();
            if rest.starts_with('"') {
                if let Some(close) = rest[1..].find('"') {
                    rest = &rest[close + 2..];
                } else {
                    return false;
                }
            }
        } else {
            return false;
        }
    }
    true
}

/// Count functions in brace-delimited languages.
fn count_brace_functions(lines: &[&str], is_fn_start: impl Fn(&str) -> bool) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut in_fn = false;
    let mut fn_start = 0;
    let mut brace_depth: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !in_fn && is_fn_start(trimmed) {
            count += 1;
            in_fn = true;
            fn_start = i;
            brace_depth = 0;
        }
        if in_fn {
            brace_depth += line.chars().filter(|&c| c == '{').count();
            brace_depth = brace_depth.saturating_sub(line.chars().filter(|&c| c == '}').count());
            if brace_depth == 0 && line.contains('}') {
                let fn_len = i - fn_start + 1;
                max_len = max_len.max(fn_len);
                in_fn = false;
            }
        }
    }

    (count, max_len)
}

/// Count functions in Python (indentation-based).
fn count_python_functions_simple(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut fn_start = 0;
    let mut fn_indent = 0;
    let mut in_fn = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            if in_fn {
                max_len = max_len.max(i - fn_start);
            }
            count += 1;
            in_fn = true;
            fn_start = i;
            fn_indent = line.len() - line.trim_start().len();
        } else if in_fn && !trimmed.is_empty() && !trimmed.starts_with('#') {
            let indent = line.len() - line.trim_start().len();
            if indent <= fn_indent
                && !trimmed.starts_with("def ")
                && !trimmed.starts_with("async def ")
            {
                max_len = max_len.max(i - fn_start);
                in_fn = false;
            }
        }
    }
    if in_fn {
        max_len = max_len.max(lines.len() - fn_start);
    }

    (count, max_len)
}

/// Count functions in Ruby (end-delimited).
fn count_ruby_functions_simple(lines: &[&str]) -> (usize, usize) {
    let mut count = 0;
    let mut max_len = 0;
    let mut fn_start = 0;
    let mut in_fn = false;
    let mut depth = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("def ") {
            if !in_fn {
                count += 1;
                in_fn = true;
                fn_start = i;
                depth = 1;
            } else {
                depth += 1;
            }
        } else if in_fn {
            if trimmed.starts_with("do")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("module ")
                || trimmed.starts_with("begin")
                || trimmed.starts_with("if ")
                || trimmed.starts_with("unless ")
                || trimmed.starts_with("case ")
            {
                depth += 1;
            }
            if trimmed == "end" || trimmed.starts_with("end ") {
                depth -= 1;
                if depth == 0 {
                    max_len = max_len.max(i - fn_start + 1);
                    in_fn = false;
                }
            }
        }
    }

    (count, max_len)
}

/// Estimate file-level cyclomatic complexity by counting branching keywords.
fn estimate_cyclomatic_simple(lang: &str, text: &str) -> usize {
    let mut complexity: usize = 1; // base

    let keywords: &[&str] = match lang.to_lowercase().as_str() {
        "rust" => &[
            "if ", "else if ", "match ", "while ", "for ", "loop ", "&&", "||",
        ],
        "javascript" | "typescript" => &[
            "if ", "else if ", "switch ", "case ", "while ", "for ", "&&", "||", "catch ",
        ],
        "python" => &["if ", "elif ", "while ", "for ", "except ", " and ", " or "],
        "go" => &[
            "if ", "else if ", "switch ", "case ", "for ", "select ", "&&", "||",
        ],
        "c" | "c++" | "java" | "c#" | "php" => &[
            "if ", "else if ", "switch ", "case ", "while ", "for ", "&&", "||", "catch ",
        ],
        "ruby" => &[
            "if ", "elsif ", "unless ", "while ", "until ", "for ", "case ", "when ", "rescue ",
        ],
        _ => return 1,
    };

    for line in text.lines() {
        let trimmed = line.trim();
        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
            continue;
        }
        for keyword in keywords {
            complexity += trimmed.matches(keyword).count();
        }
    }

    complexity
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

/// Write code bundle with file contents, dispatching based on inclusion policy.
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

        match ctx_file.policy {
            tokmd_types::InclusionPolicy::Full => {
                let header = format!("// === {} ===\n", ctx_file.path);
                writer.write_all(header.as_bytes())?;
                bytes += header.len() as u64;

                if compress {
                    let f = File::open(&file_path)
                        .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
                    let reader = BufReader::new(f);
                    for line in reader.lines() {
                        let line = line.with_context(|| {
                            format!("Failed to read file: {}", file_path.display())
                        })?;
                        if !line.trim().is_empty() {
                            writeln!(writer, "{}", line)?;
                            bytes += line.len() as u64 + 1;
                        }
                    }
                    writeln!(writer)?;
                    bytes += 1;
                } else {
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
            tokmd_types::InclusionPolicy::HeadTail => {
                let header = format!("// === {} ===\n", ctx_file.path);
                writer.write_all(header.as_bytes())?;
                bytes += header.len() as u64;

                // Capture head/tail output to count bytes
                let mut buf = Vec::new();
                crate::context_pack::write_head_tail(&mut buf, &file_path, ctx_file, compress)?;
                writer.write_all(&buf)?;
                bytes += buf.len() as u64;

                writeln!(writer)?;
                bytes += 1;
            }
            tokmd_types::InclusionPolicy::Summary | tokmd_types::InclusionPolicy::Skip => {
                let header = format!(
                    "// === {} [skipped: {}] ===\n\n",
                    ctx_file.path,
                    ctx_file.policy_reason.as_deref().unwrap_or("policy")
                );
                writer.write_all(header.as_bytes())?;
                bytes += header.len() as u64;
            }
        }
    }

    writer.flush()?;
    Ok(bytes)
}

fn exclude_output_dir(
    root: &Path,
    out_dir: &Path,
    scan_args: &mut cli::GlobalArgs,
) -> Vec<HandoffExcludedPath> {
    let pattern = normalize_exclude_pattern(root, out_dir);
    if !pattern.is_empty()
        && !scan_args
            .excluded
            .iter()
            .any(|p| normalize_path(p) == pattern)
    {
        scan_args.excluded.push(pattern.clone());
    }
    vec![HandoffExcludedPath {
        path: pattern,
        reason: "output_dir".to_string(),
    }]
}

fn normalize_exclude_pattern(root: &Path, path: &Path) -> String {
    let rel = if path.is_absolute() {
        path.strip_prefix(root).unwrap_or(path)
    } else {
        path
    };
    let out = normalize_path(&rel.to_string_lossy());
    out.strip_prefix("./").unwrap_or(&out).to_string()
}

fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let mut hasher = Hasher::new();
    let mut buf = [0u8; 8 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
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
    #[allow(clippy::approx_constant)]
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
        let tree = build_tree(&export, DEFAULT_TREE_DEPTH);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_build_tree_depth_limit_and_no_file_leaves() {
        let export = ExportData {
            rows: vec![FileRow {
                path: "a/b/c/file.rs".to_string(),
                module: "a".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 10,
                comments: 0,
                blanks: 0,
                lines: 10,
                bytes: 100,
                tokens: 20,
            }],
            module_roots: vec![],
            module_depth: 2,
            children: cli::ChildIncludeMode::ParentsOnly,
        };
        let tree = build_tree(&export, 1);
        assert!(tree.contains("a/"));
        assert!(!tree.contains("b/"));
        assert!(!tree.contains("file.rs"));
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

    #[test]
    fn test_count_functions_simple_rust() {
        let code = r#"
fn simple() {
    println!("hello");
}

pub fn public_fn() {
    let x = 1;
    let y = 2;
}

pub async fn async_fn() {
    todo!()
}
"#;
        let (count, max_len) = count_functions_simple("Rust", code);
        assert_eq!(count, 3);
        assert!(max_len >= 3);
    }

    #[test]
    fn test_count_functions_simple_python() {
        let code = r#"
def foo():
    pass

async def bar():
    await something()

def baz():
    x = 1
    y = 2
    return x + y
"#;
        let (count, _max_len) = count_functions_simple("Python", code);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_estimate_cyclomatic_simple_rust() {
        let code = r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            x * 2
        } else {
            x + 1
        }
    } else {
        match x {
            -1 => 0,
            _ => x.abs(),
        }
    }
}
"#;
        let cyclo = estimate_cyclomatic_simple("Rust", code);
        // Base 1 + 2 ifs + 1 else if (none here) + 1 match = 4+
        assert!(cyclo >= 4, "Expected cyclomatic >= 4, got {}", cyclo);
    }

    #[test]
    fn test_is_analyzable_lang() {
        assert!(is_analyzable_lang("Rust"));
        assert!(is_analyzable_lang("javascript"));
        assert!(is_analyzable_lang("Python"));
        assert!(!is_analyzable_lang("Markdown"));
        assert!(!is_analyzable_lang("JSON"));
        assert!(!is_analyzable_lang("TOML"));
    }
}
