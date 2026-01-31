use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

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
use tokmd_types::{ContextFileRow, ContextLogRecord, ContextReceipt, SCHEMA_VERSION, ToolInfo};

use crate::context_pack;
use crate::git_scoring;
use crate::progress::Progress;

pub(crate) fn handle(args: cli::CliContextArgs, global: &cli::GlobalArgs) -> Result<()> {
    let progress = Progress::new(!global.no_progress);

    let paths = args
        .paths
        .clone()
        .unwrap_or_else(|| vec![PathBuf::from(".")]);

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

    // Compute git scores if using churn/hotspot ranking
    progress.set_message("Computing scores...");
    let needs_git = matches!(
        args.rank_by,
        cli::ValueMetric::Churn | cli::ValueMetric::Hotspot
    );
    let git_scores = if needs_git && !args.no_git {
        let root = paths.first().cloned().unwrap_or_else(|| PathBuf::from("."));
        match git_scoring::compute_git_scores(
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

    progress.finish_and_clear();

    // Determine output destination for logging
    let output_destination = determine_output_destination(&args);

    // Write output and get total bytes written
    let total_bytes = if let Some(ref bundle_dir) = args.bundle_dir {
        // Handle bundle directory mode - streams directly to files
        write_bundle_directory(
            bundle_dir,
            &args,
            &selected,
            budget,
            used_tokens,
            utilization,
            args.force,
        )?
    } else {
        // For bundle output mode, stream directly to destination
        // For list/json output modes, build string (small outputs)
        write_to_destination(&args, &selected, budget, used_tokens, utilization)?
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
    } else if let Some(ref out_path) = args.out {
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
) -> Result<usize> {
    match args.output {
        cli::ContextOutput::Bundle => {
            // Stream bundle output directly to destination
            write_bundle_to_destination(args, selected)
        }
        cli::ContextOutput::List | cli::ContextOutput::Json => {
            // Build string for list/json (small outputs)
            let content = match args.output {
                cli::ContextOutput::List => {
                    format_list_output(selected, budget, used_tokens, utilization, args.strategy)
                }
                cli::ContextOutput::Json => {
                    format_json_output(selected, budget, used_tokens, utilization, args)?
                }
                cli::ContextOutput::Bundle => unreachable!(),
            };
            let total_bytes = content.len();

            if let Some(ref out_path) = args.out {
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
    if let Some(ref out_path) = args.out {
        // Open file with proper semantics: create_new fails if exists (unless --force)
        let file = if args.force {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(out_path)
        } else {
            OpenOptions::new().write(true).create_new(true).open(out_path)
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

        writeln!(w, "// === {} ===", file.path)?;

        if compress {
            // Strip blank lines only (safe for all languages).
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
            // Stream with 16KB buffer
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
    Ok(())
}

/// Format JSON receipt output.
fn format_json_output(
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    args: &cli::CliContextArgs,
) -> Result<String> {
    let receipt = ContextReceipt {
        schema_version: SCHEMA_VERSION,
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
fn write_bundle_directory(
    dir: &Path,
    args: &cli::CliContextArgs,
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    force: bool,
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

    // Write receipt.json
    let receipt_path = dir.join("receipt.json");
    let receipt = ContextReceipt {
        schema_version: SCHEMA_VERSION,
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
    };
    let receipt_json = serde_json::to_string_pretty(&receipt)?;
    fs::write(&receipt_path, &receipt_json)
        .with_context(|| format!("Failed to write receipt: {}", receipt_path.display()))?;

    // Write bundle.txt (concatenated content) - stream directly to file
    let bundle_path = dir.join("bundle.txt");
    let bundle_file = File::create(&bundle_path)
        .with_context(|| format!("Failed to create bundle file: {}", bundle_path.display()))?;
    let mut counter = CountingWriter::new(bundle_file);
    write_bundle_output(&mut counter, selected, args.compress)?;
    counter.flush()?;
    let bundle_bytes = counter.bytes() as usize;

    // Write manifest.json (list of files with metadata)
    let manifest_path = dir.join("manifest.json");
    let manifest = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "generated_at_ms": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        "budget_tokens": budget,
        "used_tokens": used_tokens,
        "utilization_pct": utilization,
        "file_count": selected.len(),
        "bundle_bytes": bundle_bytes,
        "files": selected.iter().map(|f| {
            serde_json::json!({
                "path": f.path,
                "module": f.module,
                "lang": f.lang,
                "tokens": f.tokens,
                "code": f.code,
                "lines": f.lines,
                "bytes": f.bytes,
            })
        }).collect::<Vec<_>>(),
    });
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, &manifest_json)
        .with_context(|| format!("Failed to write manifest: {}", manifest_path.display()))?;

    eprintln!("Wrote bundle to {}", dir.display());
    eprintln!("  - receipt.json ({} bytes)", receipt_json.len());
    eprintln!("  - bundle.txt ({} bytes)", bundle_bytes);
    eprintln!("  - manifest.json ({} bytes)", manifest_json.len());

    Ok(bundle_bytes)
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
