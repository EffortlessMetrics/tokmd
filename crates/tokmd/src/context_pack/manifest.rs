//! Context bundle directory receipt and manifest writing.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use blake3::Hasher;
use tokmd_types::{
    ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION,
    ContextBundleManifest, ContextExcludedPath, ContextFileRow, ContextReceipt, ToolInfo,
};

use crate::cli;

use super::{CountingWriter, SelectResult, write_bundle_output};

/// Write bundle to a directory with manifest.
///
/// Streams bundle.txt directly to avoid memory blowup and returns the total
/// bytes of bundle.txt (the main output).
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_bundle_directory(
    dir: &Path,
    args: &cli::CliContextArgs,
    selected: &[ContextFileRow],
    budget: usize,
    used_tokens: usize,
    utilization: f64,
    force: bool,
    excluded_paths: &[ContextExcludedPath],
    excluded_patterns: &[String],
    select_result: &SelectResult,
) -> Result<usize> {
    // Check if directory exists and is non-empty.
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

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    let now_ms = js_sys::Date::now().max(1.0) as u128;

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    // Compute token estimation from selected file bytes.
    let total_file_bytes: usize = selected.iter().map(|f| f.bytes).sum();
    let token_estimation = tokmd_types::TokenEstimationMeta::from_bytes(total_file_bytes, 4.0);

    // Write receipt.json.
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
        bundle_audit: None,
    };
    // Write initial receipt.json (bundle_audit populated after bundle is written).
    let initial_receipt_json = serde_json::to_string_pretty(&receipt)?;
    fs::write(&receipt_path, &initial_receipt_json)
        .with_context(|| format!("Failed to write receipt: {}", receipt_path.display()))?;

    // Write bundle.txt (concatenated content) - stream directly to file.
    let bundle_path = dir.join("bundle.txt");
    let bundle_file = File::create(&bundle_path)
        .with_context(|| format!("Failed to create bundle file: {}", bundle_path.display()))?;
    let mut counter = CountingWriter::new(bundle_file);
    write_bundle_output(&mut counter, selected, args.compress)?;
    counter.flush()?;
    let bundle_bytes = counter.bytes() as usize;
    let bundle_hash = hash_file(&bundle_path)?;

    // Deferred write: rewrite receipt.json with bundle audit.
    let receipt_audit =
        tokmd_types::TokenAudit::from_output(bundle_bytes as u64, total_file_bytes as u64);
    let mut receipt = receipt;
    receipt.bundle_audit = Some(receipt_audit);
    let receipt_json = serde_json::to_string_pretty(&receipt)?;
    fs::write(&receipt_path, &receipt_json)
        .with_context(|| format!("Failed to rewrite receipt: {}", receipt_path.display()))?;

    // Build artifacts list.
    let artifacts = vec![
        ArtifactEntry {
            name: "manifest".to_string(),
            path: "manifest.json".to_string(),
            description: "Context bundle manifest".to_string(),
            bytes: 0,
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

    // Write manifest.json (authoritative index).
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
