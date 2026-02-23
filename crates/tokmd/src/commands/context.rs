use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use blake3::Hasher;

/// A writer wrapper that counts bytes written.
struct CountingWriter<W: Write> {
    inner: W,
    bytes: u64,
}

impl<W: Write> CountingWriter<W> {
    fn new(inner: W) -> Self {
        Self { inner, bytes: 0 }
    }

    fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.bytes += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
use tokmd_config as cli;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_types::{
    ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION,
    ContextBundleManifest, ContextExcludedPath, ContextFileRow, ContextLogRecord, ContextReceipt,
    InclusionPolicy, SCHEMA_VERSION, ToolInfo,
};

use crate::context_pack;
use tokmd_progress::Progress;

pub(crate) fn handle(args: cli::CliContextArgs, global: &cli::GlobalArgs) -> Result<()> {
    let progress = Progress::new(!global.no_progress);

    let paths = args
        .paths
        .clone()
        .unwrap_or_else(|| vec![PathBuf::from(".")]);

    // Parse budget
    let budget = context_pack::parse_budget(&args.budget)?;

    let root = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));

    // Scan and create export data
    progress.set_message("Scanning codebase...");
    let mut scan_args = global.clone();
    let mut excluded_paths: Vec<ContextExcludedPath> = Vec::new();
    add_excluded_path(
        &root,
        args.output.as_ref(),
        "out_file",
        &mut scan_args,
        &mut excluded_paths,
    );
    add_excluded_path(
        &root,
        args.bundle_dir.as_ref(),
        "bundle_dir",
        &mut scan_args,
        &mut excluded_paths,
    );
    add_excluded_path(
        &root,
        args.log.as_ref(),
        "log_file",
        &mut scan_args,
        &mut excluded_paths,
    );
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

    // Compute git scores if using churn/hotspot ranking
    progress.set_message("Computing scores...");
    let needs_git = matches!(
        args.rank_by,
        cli::ValueMetric::Churn | cli::ValueMetric::Hotspot
    );
    let git_scores = if needs_git && !args.no_git {
        let root = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));
        match tokmd_context_git::compute_git_scores(
            &root,
            &export.rows,
            args.max_commits,
            args.max_commit_files,
        ) {
            Some(scores) => {
                if scores.hotspots.is_empty() && args.git {
                    eprintln!("Warning: no git history found for scanned files");
                }
                Some(scores)
            }
            None => {
                if args.git {
                    eprintln!("Warning: git data unavailable, falling back to code lines");
                }
                None
            }
        }
    } else {
        None
    };

    // Select files based on strategy
    progress.set_message("Selecting files for context...");
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
            require_git_scores: args.require_git_scores,
            ..Default::default()
        },
    );

    // Error if require_git_scores is set and a fallback occurred
    if args.require_git_scores && select_result.fallback_reason.is_some() {
        anyhow::bail!(
            "Git scores required but unavailable: {}",
            select_result
                .fallback_reason
                .as_deref()
                .unwrap_or("unknown")
        );
    }

    let selected = &select_result.selected;

    let used_tokens: usize = selected
        .iter()
        .map(|f| f.effective_tokens.unwrap_or(f.tokens))
        .sum();
    let utilization = if budget > 0 {
        (used_tokens as f64 / budget as f64) * 100.0
    } else {
        0.0
    };

    progress.finish_and_clear();

    // Determine output destination for logging
    let output_destination = determine_output_destination(&args);

    // Write output and get total bytes written
    let total_bytes = if let Some(ref bundle_dir) = args.bundle_dir {
        // Handle bundle directory mode - streams directly to files
        write_bundle_directory(
            bundle_dir,
            &args,
            selected,
            budget,
            used_tokens,
            utilization,
            args.force,
            &excluded_paths,
            &scan_args.excluded,
            &select_result,
        )?
    } else {
        // For bundle output mode, stream directly to destination
        // For list/json output modes, build string (small outputs)
        write_to_destination(
            &args,
            selected,
            budget,
            used_tokens,
            utilization,
            &select_result,
        )?
    };

    // Check size threshold and emit warning if exceeded (after writing)
    let max_bytes = args.max_output_bytes;
    if max_bytes > 0 && total_bytes as u64 > max_bytes {
        eprintln!(
            "Warning: output size ({} bytes) exceeds threshold ({} bytes). Consider using --bundle-dir for large outputs.",
            total_bytes, max_bytes
        );
    }

    // Handle log append
    if let Some(ref log_path) = args.log {
        let log_record = ContextLogRecord {
            schema_version: SCHEMA_VERSION,
            generated_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            tool: ToolInfo::current(),
            budget_tokens: budget,
            used_tokens,
            utilization_pct: utilization,
            strategy: format!("{:?}", args.strategy).to_lowercase(),
            rank_by: format!("{:?}", args.rank_by).to_lowercase(),
            file_count: selected.len(),
            total_bytes,
            output_destination,
        };
        append_log_record(log_path, &log_record)?;
    }

    Ok(())
}

/// Determine the output destination string for logging.
fn determine_output_destination(args: &cli::CliContextArgs) -> String {
    if let Some(ref bundle_dir) = args.bundle_dir {
        format!("bundle:{}", bundle_dir.display())
    } else if let Some(ref out_path) = args.output {
        format!("file:{}", out_path.display())
    } else {
        "stdout".to_string()
    }
}

/// Write output to destination and return total bytes written.
/// For bundle output, streams directly to avoid memory blowup.
/// For list/json output, builds string first (small outputs).
fn write_to_destination(
    args: &cli::CliContextArgs,
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    select_result: &context_pack::SelectResult,
) -> Result<usize> {
    match args.output_mode {
        cli::ContextOutput::Bundle => {
            // Stream bundle output directly to destination
            write_bundle_to_destination(args, selected)
        }
        cli::ContextOutput::List | cli::ContextOutput::Json => {
            // Build string for list/json (small outputs)
            let content = match args.output_mode {
                cli::ContextOutput::List => {
                    format_list_output(selected, budget, used_tokens, utilization, args.strategy)
                }
                cli::ContextOutput::Json => format_json_output(
                    selected,
                    budget,
                    used_tokens,
                    utilization,
                    args,
                    select_result,
                )?,
                cli::ContextOutput::Bundle => unreachable!(),
            };
            let total_bytes = content.len();

            if let Some(ref out_path) = args.output {
                write_output_file(out_path, &content, args.force)?;
            } else {
                print!("{}", content);
            }

            Ok(total_bytes)
        }
    }
}

/// Write bundle output directly to destination (file or stdout).
/// Streams content to avoid loading entire bundle into memory.
fn write_bundle_to_destination(
    args: &cli::CliContextArgs,
    selected: &[ContextFileRow],
) -> Result<usize> {
    if let Some(ref out_path) = args.output {
        // Open file with proper semantics: create_new fails if exists (unless --force)
        let file = if args.force {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(out_path)
        } else {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out_path)
        }
        .with_context(|| {
            if !args.force && out_path.exists() {
                format!(
                    "Output file already exists: {}. Use --force to overwrite.",
                    out_path.display()
                )
            } else {
                format!("Failed to create output file: {}", out_path.display())
            }
        })?;

        let mut counter = CountingWriter::new(file);
        write_bundle_output(&mut counter, selected, args.compress)?;
        counter.flush()?;

        let bytes = counter.bytes() as usize;
        eprintln!("Wrote {}", out_path.display());
        Ok(bytes)
    } else {
        // Stream to stdout
        let stdout = std::io::stdout();
        let mut counter = CountingWriter::new(stdout.lock());
        write_bundle_output(&mut counter, selected, args.compress)?;
        counter.flush()?;
        Ok(counter.bytes() as usize)
    }
}

/// Format list output (markdown table).
fn format_list_output(
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    strategy: cli::ContextStrategy,
) -> String {
    let mut out = String::new();
    out.push_str("# Context Pack\n\n");
    out.push_str(&format!("Budget: {} tokens\n", budget));
    out.push_str(&format!(
        "Used: {} tokens ({:.1}%)\n",
        used_tokens, utilization
    ));
    out.push_str(&format!("Files: {}\n", selected.len()));
    out.push_str(&format!("Strategy: {:?}\n\n", strategy));
    out.push_str("|Path|Module|Lang|Tokens|Code|\n");
    out.push_str("|---|---|---|---:|---:|\n");
    for file in selected {
        out.push_str(&format!(
            "|{}|{}|{}|{}|{}|\n",
            file.path, file.module, file.lang, file.tokens, file.code
        ));
    }
    out
}

/// Write bundle output (concatenated file contents) directly to a writer.
/// Streams file content to avoid loading entire bundle into memory.
/// Dispatches based on file inclusion policy (Full / HeadTail / Skip).
fn write_bundle_output<W: Write>(
    w: &mut W,
    selected: &[ContextFileRow],
    compress: bool,
) -> Result<()> {
    for file in selected {
        let path = PathBuf::from(&file.path);
        if !path.exists() {
            continue;
        }

        match file.policy {
            InclusionPolicy::Full => {
                writeln!(w, "// === {} ===", file.path)?;

                if compress {
                    let f = File::open(&path)
                        .with_context(|| format!("Failed to open file: {}", path.display()))?;
                    let reader = BufReader::new(f);
                    for line in reader.lines() {
                        let line = line
                            .with_context(|| format!("Failed to read file: {}", path.display()))?;
                        if !line.trim().is_empty() {
                            writeln!(w, "{line}")?;
                        }
                    }
                    writeln!(w)?;
                } else {
                    let mut f = File::open(&path)
                        .with_context(|| format!("Failed to open file: {}", path.display()))?;
                    let mut buf = [0u8; 16 * 1024];
                    let mut last: Option<u8> = None;
                    loop {
                        let n = f.read(&mut buf)?;
                        if n == 0 {
                            break;
                        }
                        last = Some(buf[n - 1]);
                        w.write_all(&buf[..n])?;
                    }
                    if last != Some(b'\n') {
                        w.write_all(b"\n")?;
                    }
                    w.write_all(b"\n")?;
                }
            }
            InclusionPolicy::HeadTail => {
                writeln!(w, "// === {} ===", file.path)?;
                context_pack::write_head_tail(w, &path, file, compress)?;
                writeln!(w)?;
            }
            InclusionPolicy::Summary | InclusionPolicy::Skip => {
                writeln!(
                    w,
                    "// === {} [skipped: {}] ===",
                    file.path,
                    file.policy_reason.as_deref().unwrap_or("policy")
                )?;
                writeln!(w)?;
            }
        }
    }
    Ok(())
}

/// Format JSON receipt output.
fn format_json_output(
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    args: &cli::CliContextArgs,
    select_result: &context_pack::SelectResult,
) -> Result<String> {
    let total_file_bytes: usize = selected.iter().map(|f| f.bytes).sum();
    let token_estimation = tokmd_types::TokenEstimationMeta::from_bytes(total_file_bytes, 4.0);
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        tool: ToolInfo::current(),
        mode: "context".to_string(),
        budget_tokens: budget,
        used_tokens,
        utilization_pct: utilization,
        strategy: format!("{:?}", args.strategy).to_lowercase(),
        rank_by: format!("{:?}", args.rank_by).to_lowercase(),
        file_count: selected.len(),
        files: selected.to_vec(),
        rank_by_effective: if select_result.fallback_reason.is_some() {
            Some(select_result.rank_by_effective.clone())
        } else {
            None
        },
        fallback_reason: select_result.fallback_reason.clone(),
        excluded_by_policy: select_result.excluded_by_policy.clone(),
        token_estimation: Some(token_estimation),
        bundle_audit: None,
    };
    let json = serde_json::to_string_pretty(&receipt)?;
    Ok(format!("{}\n", json))
}

/// Write output to a file, checking for existence unless force is true.
fn write_output_file(path: &Path, content: &str, force: bool) -> Result<()> {
    // Open file with proper semantics: create_new fails if exists (unless --force)
    let mut file = if force {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
    } else {
        OpenOptions::new().write(true).create_new(true).open(path)
    }
    .with_context(|| {
        if !force && path.exists() {
            format!(
                "Output file already exists: {}. Use --force to overwrite.",
                path.display()
            )
        } else {
            format!("Failed to write output file: {}", path.display())
        }
    })?;

    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write output file: {}", path.display()))?;
    eprintln!("Wrote {}", path.display());
    Ok(())
}

/// Write bundle to a directory with manifest.
/// Streams bundle.txt directly to avoid memory blowup.
/// Returns the total bytes of bundle.txt (the main output).
#[allow(clippy::too_many_arguments)]
fn write_bundle_directory(
    dir: &Path,
    args: &cli::CliContextArgs,
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    force: bool,
    excluded_paths: &[ContextExcludedPath],
    excluded_patterns: &[String],
    select_result: &context_pack::SelectResult,
) -> Result<usize> {
    // Check if directory exists and is non-empty
    if dir.exists() {
        let is_empty = dir
            .read_dir()
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);
        if !is_empty && !force {
            bail!(
                "Bundle directory is not empty: {}. Use --force to overwrite.",
                dir.display()
            );
        }
    } else {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create bundle directory: {}", dir.display()))?;
    }

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    // Compute token estimation from selected file bytes
    let total_file_bytes: usize = selected.iter().map(|f| f.bytes).sum();
    let token_estimation = tokmd_types::TokenEstimationMeta::from_bytes(total_file_bytes, 4.0);

    // Write receipt.json
    let receipt_path = dir.join("receipt.json");
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: now_ms,
        tool: ToolInfo::current(),
        mode: "context".to_string(),
        budget_tokens: budget,
        used_tokens,
        utilization_pct: utilization,
        strategy: format!("{:?}", args.strategy).to_lowercase(),
        rank_by: format!("{:?}", args.rank_by).to_lowercase(),
        file_count: selected.len(),
        files: selected.to_vec(),
        rank_by_effective: if select_result.fallback_reason.is_some() {
            Some(select_result.rank_by_effective.clone())
        } else {
            None
        },
        fallback_reason: select_result.fallback_reason.clone(),
        excluded_by_policy: select_result.excluded_by_policy.clone(),
        token_estimation: Some(token_estimation),
        bundle_audit: None, // Populated below after bundle is written
    };
    // Write initial receipt.json (bundle_audit populated after bundle is written)
    let initial_receipt_json = serde_json::to_string_pretty(&receipt)?;
    fs::write(&receipt_path, &initial_receipt_json)
        .with_context(|| format!("Failed to write receipt: {}", receipt_path.display()))?;

    // Write bundle.txt (concatenated content) - stream directly to file
    let bundle_path = dir.join("bundle.txt");
    let bundle_file = File::create(&bundle_path)
        .with_context(|| format!("Failed to create bundle file: {}", bundle_path.display()))?;
    let mut counter = CountingWriter::new(bundle_file);
    write_bundle_output(&mut counter, selected, args.compress)?;
    counter.flush()?;
    let bundle_bytes = counter.bytes() as usize;
    let bundle_hash = hash_file(&bundle_path)?;

    // Deferred write: rewrite receipt.json with bundle audit
    let receipt_audit =
        tokmd_types::TokenAudit::from_output(bundle_bytes as u64, total_file_bytes as u64);
    let mut receipt = receipt;
    receipt.bundle_audit = Some(receipt_audit);
    let receipt_json = serde_json::to_string_pretty(&receipt)?;
    fs::write(&receipt_path, &receipt_json)
        .with_context(|| format!("Failed to rewrite receipt: {}", receipt_path.display()))?;

    // Build artifacts list
    let artifacts = vec![
        ArtifactEntry {
            name: "manifest".to_string(),
            path: "manifest.json".to_string(),
            description: "Context bundle manifest".to_string(),
            bytes: 0, // Self-referential hash omitted
            hash: None,
        },
        ArtifactEntry {
            name: "receipt".to_string(),
            path: "receipt.json".to_string(),
            description: "Context selection receipt".to_string(),
            bytes: receipt_json.len() as u64,
            hash: None,
        },
        ArtifactEntry {
            name: "bundle".to_string(),
            path: "bundle.txt".to_string(),
            description: "Token-budgeted code bundle".to_string(),
            bytes: bundle_bytes as u64,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: bundle_hash,
            }),
        },
    ];

    // Write manifest.json (authoritative index)
    let manifest_path = dir.join("manifest.json");
    let total_file_bytes: usize = selected.iter().map(|f| f.bytes).sum();
    let bundle_estimation = tokmd_types::TokenEstimationMeta::from_bytes(total_file_bytes, 4.0);
    let bundle_audit =
        tokmd_types::TokenAudit::from_output(bundle_bytes as u64, total_file_bytes as u64);
    let manifest = ContextBundleManifest {
        schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
        generated_at_ms: now_ms,
        tool: ToolInfo::current(),
        mode: "context_bundle".to_string(),
        budget_tokens: budget,
        used_tokens,
        utilization_pct: utilization,
        strategy: format!("{:?}", args.strategy).to_lowercase(),
        rank_by: format!("{:?}", args.rank_by).to_lowercase(),
        file_count: selected.len(),
        bundle_bytes,
        artifacts,
        included_files: selected.to_vec(),
        excluded_paths: excluded_paths.to_vec(),
        excluded_patterns: excluded_patterns.to_vec(),
        rank_by_effective: if select_result.fallback_reason.is_some() {
            Some(select_result.rank_by_effective.clone())
        } else {
            None
        },
        fallback_reason: select_result.fallback_reason.clone(),
        excluded_by_policy: select_result.excluded_by_policy.clone(),
        token_estimation: Some(bundle_estimation),
        bundle_audit: Some(bundle_audit),
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, &manifest_json)
        .with_context(|| format!("Failed to write manifest: {}", manifest_path.display()))?;

    eprintln!("Wrote bundle to {}", dir.display());
    eprintln!("  - receipt.json ({} bytes)", receipt_json.len());
    eprintln!("  - bundle.txt ({} bytes)", bundle_bytes);
    eprintln!("  - manifest.json ({} bytes)", manifest_json.len());

    Ok(bundle_bytes)
}

fn add_excluded_path(
    root: &Path,
    path: Option<&PathBuf>,
    reason: &str,
    scan_args: &mut cli::GlobalArgs,
    excluded_paths: &mut Vec<ContextExcludedPath>,
) {
    let Some(path) = path else { return };
    let pattern = normalize_exclude_pattern(root, path);
    if pattern.is_empty() {
        return;
    }

    if !scan_args
        .excluded
        .iter()
        .any(|p| normalize_path(p) == pattern)
    {
        scan_args.excluded.push(pattern.clone());
    }

    if !excluded_paths.iter().any(|p| p.path == pattern) {
        excluded_paths.push(ContextExcludedPath {
            path: pattern,
            reason: reason.to_string(),
        });
    }
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

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
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

/// Append a log record to a JSONL file.
fn append_log_record(path: &Path, record: &ContextLogRecord) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open log file: {}", path.display()))?;

    let json = serde_json::to_string(record)?;
    writeln!(file, "{}", json)
        .with_context(|| format!("Failed to append to log file: {}", path.display()))?;

    Ok(())
}
