//! Handoff command: Bundle codebase for LLM handoff.
//!
//! Creates a `.handoff/` directory with four artifacts:
//! - `manifest.json`: Bundle metadata, budgets, capabilities
//! - `map.jsonl`: Complete file inventory (streaming)
//! - `intelligence.json`: Tree + hotspots + complexity + derived
//! - `code.txt`: Token-budgeted code bundle

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cli;
use anyhow::{Context, Result, bail};
use blake3::Hasher;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_scan::{add_exclude_pattern, normalize_exclude_pattern};
use tokmd_types::{
    ArtifactEntry, ArtifactHash, ExportData, FileKind, HANDOFF_SCHEMA_VERSION, HandoffExcludedPath,
    HandoffManifest, ToolInfo,
};

use crate::context_pack;
use crate::progress::Progress;

mod capabilities;
mod intelligence;

use capabilities::{detect_capabilities, should_compute_git};
use intelligence::build_intelligence;

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
        tokmd_types::ChildIncludeMode::ParentsOnly,
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
        tokmd_core::context_git::compute_git_scores(
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
    if !pattern.is_empty() {
        let _ = add_exclude_pattern(&mut scan_args.excluded, pattern.clone());
    }
    vec![HandoffExcludedPath {
        path: pattern,
        reason: "output_dir".to_string(),
    }]
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

/// Round a float to N decimal places.
fn round_f64(value: f64, decimals: u32) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_scan::normalize_slashes as normalize_path;
    use tokmd_types::FileRow;

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
            children: tokmd_types::ChildIncludeMode::ParentsOnly,
        };
        let tree = tokmd_format::render_handoff_tree(&export, DEFAULT_TREE_DEPTH);
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
            children: tokmd_types::ChildIncludeMode::ParentsOnly,
        };
        let tree = tokmd_format::render_handoff_tree(&export, 1);
        assert!(tree.contains("a/"));
        assert!(!tree.contains("b/"));
        assert!(!tree.contains("file.rs"));
    }
}
