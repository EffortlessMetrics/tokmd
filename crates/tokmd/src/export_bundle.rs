use std::path::PathBuf;

use anyhow::{Context, Result};
use tokmd_config as cli;
use tokmd_model as model;
use tokmd_scan as scan;

#[derive(Debug, Clone)]
pub(crate) struct ExportMetaLite {
    pub(crate) schema_version: Option<u32>,
    pub(crate) generated_at_ms: Option<u128>,
    pub(crate) module_roots: Vec<String>,
    pub(crate) module_depth: usize,
    pub(crate) children: cli::ChildIncludeMode,
}

impl Default for ExportMetaLite {
    fn default() -> Self {
        Self {
            schema_version: None,
            generated_at_ms: None,
            // Expanded defaults to cover standard project structures for ad-hoc scans
            module_roots: vec!["crates".into(), "packages".into(), "src".into()],
            module_depth: 2,
            children: cli::ChildIncludeMode::Separate,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ExportBundle {
    pub(crate) export: tokmd_types::ExportData,
    pub(crate) meta: ExportMetaLite,
    /// The path to the actual data file (e.g., export.jsonl)
    pub(crate) export_path: Option<PathBuf>,
    /// The user-provided entry point (e.g., receipt.json or the run directory)
    pub(crate) entry_point: Option<PathBuf>,
    pub(crate) root: PathBuf,
}

pub(crate) fn load_export_from_inputs(
    inputs: &[PathBuf],
    global: &cli::GlobalArgs,
) -> Result<ExportBundle> {
    if inputs.len() > 1 {
        return scan_export_from_paths(inputs, global);
    }

    let input = inputs
        .first()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("."));

    // Case 1: Input is a directory (Run Directory)
    if input.is_dir() {
        let run_receipt = input.join("receipt.json");
        let export_jsonl = input.join("export.jsonl");
        let export_json = input.join("export.json");

        // Priority 1: receipt.json (The manifest)
        if run_receipt.exists() {
            return load_export_from_receipt(&run_receipt, Some(input.clone()), global);
        }
        // Priority 2: export.jsonl (The raw data)
        if export_jsonl.exists() {
            return load_export_from_file(&export_jsonl, Some(input), global);
        }
        // Priority 3: export.json
        if export_json.exists() {
            return load_export_from_file(&export_json, Some(input), global);
        }
    }

    // Case 2: Input is a file (Receipt or Data)
    if input.is_file() {
        return load_export_from_file(&input, None, global);
    }

    // Case 3: Input is paths to scan (or "." default)
    scan_export_from_paths(inputs, global)
}

fn scan_export_from_paths(paths: &[PathBuf], global: &cli::GlobalArgs) -> Result<ExportBundle> {
    let scan_opts = tokmd_settings::ScanOptions::from(global);
    let languages = scan::scan(paths, &scan_opts)?;
    let meta = ExportMetaLite::default();
    let export = model::create_export_data(
        &languages,
        &meta.module_roots,
        meta.module_depth,
        meta.children,
        None,
        0,
        0,
    );
    Ok(ExportBundle {
        export,
        meta,
        export_path: None,
        entry_point: None,
        root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    })
}

fn load_export_from_receipt(
    path: &PathBuf,
    run_dir: Option<PathBuf>,
    global: &cli::GlobalArgs,
) -> Result<ExportBundle> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let receipt: tokmd_types::RunReceipt =
        serde_json::from_str(&content).context("Failed to parse run receipt")?;

    let base = run_dir.unwrap_or_else(|| path.parent().unwrap_or(path).to_path_buf());
    let export_path = base.join(&receipt.export_file);

    // Recurse to load the data file referenced by the receipt
    let mut bundle = load_export_from_file(&export_path, Some(base), global)?;

    // Fix the entry point to point to the receipt we loaded
    bundle.entry_point = Some(path.clone());
    Ok(bundle)
}

fn load_export_from_file(
    path: &PathBuf,
    run_dir: Option<PathBuf>,
    global: &cli::GlobalArgs,
) -> Result<ExportBundle> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Fast path: Scan if not JSON-like (e.g. tokmd analyze my_script.py)
    if ext != "json" && ext != "jsonl" {
        return scan_export_from_paths(std::slice::from_ref(path), global);
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    // Strategy 1: Try parsing as RunReceipt (receipt.json)
    // This handles the case where user runs `tokmd analyze receipt.json` directly
    if ext == "json"
        && let Ok(receipt) = serde_json::from_str::<tokmd_types::RunReceipt>(&content)
    {
        let base = run_dir
            .clone()
            .unwrap_or_else(|| path.parent().unwrap_or(path).to_path_buf());
        let export_file_path = base.join(&receipt.export_file);

        let mut bundle = load_export_from_file(&export_file_path, Some(base), global)?;
        bundle.entry_point = Some(path.clone());
        return Ok(bundle);
    }

    // Strategy 2: Load Export Data (jsonl or json)
    let (mut export, meta) = if ext == "jsonl" {
        load_export_jsonl_content(&content)?
    } else {
        load_export_json_content(&content)?
    };

    export.module_roots = meta.module_roots.clone();
    export.module_depth = meta.module_depth;
    export.children = meta.children;

    Ok(ExportBundle {
        export,
        meta,
        export_path: Some(path.clone()),
        entry_point: Some(path.clone()),
        root: run_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    })
}

fn load_export_jsonl_content(content: &str) -> Result<(tokmd_types::ExportData, ExportMetaLite)> {
    let mut rows = Vec::new();
    let mut meta = ExportMetaLite::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)?;
        let ty = value.get("type").and_then(|v| v.as_str()).unwrap_or("row");
        if ty == "meta" {
            if let Some(schema) = value.get("schema_version").and_then(|v| v.as_u64()) {
                meta.schema_version = Some(schema as u32);
            }
            if let Some(generated) = value.get("generated_at_ms").and_then(|v| v.as_u64()) {
                meta.generated_at_ms = Some(generated as u128);
            }
            if let Some(args) = value.get("args") {
                let parsed: tokmd_types::ExportArgsMeta = serde_json::from_value(args.clone())?;
                meta.module_roots = parsed.module_roots.clone();
                meta.module_depth = parsed.module_depth;
                meta.children = parsed.children;
            }
            continue;
        }

        let row: tokmd_types::FileRow = serde_json::from_value(value)?;
        rows.push(row);
    }

    Ok((
        tokmd_types::ExportData {
            rows,
            module_roots: meta.module_roots.clone(),
            module_depth: meta.module_depth,
            children: meta.children,
        },
        meta,
    ))
}

fn load_export_json_content(content: &str) -> Result<(tokmd_types::ExportData, ExportMetaLite)> {
    // Try ExportReceipt wrapper first
    if let Ok(receipt) = serde_json::from_str::<tokmd_types::ExportReceipt>(content) {
        let meta = ExportMetaLite {
            schema_version: Some(receipt.schema_version),
            generated_at_ms: Some(receipt.generated_at_ms),
            module_roots: receipt.args.module_roots.clone(),
            module_depth: receipt.args.module_depth,
            children: receipt.args.children,
        };
        return Ok((receipt.data, meta));
    }

    // Fallback to raw list of rows
    let rows: Vec<tokmd_types::FileRow> =
        serde_json::from_str(content).context("Failed to parse export rows")?;
    let meta = ExportMetaLite::default();

    Ok((
        tokmd_types::ExportData {
            rows,
            module_roots: meta.module_roots.clone(),
            module_depth: meta.module_depth,
            children: meta.children,
        },
        meta,
    ))
}
