//! # tokmd-core
//!
//! This crate is the **primary library interface** for `tokmd`.
//! It coordinates scanning, aggregation, and modeling to produce code inventory receipts.
//!
//! If you are embedding `tokmd` into another Rust application, depend on this crate
//! and `tokmd-types`. Avoid depending on `tokmd-scan` or `tokmd-model` directly unless necessary.
//!
//! ## Example
//!
//! ```rust,no_run
//! use tokmd_core::scan_workflow;
//! use tokmd_config::{ChildrenMode, GlobalArgs, RedactMode, TableFormat};
//! use tokmd_types::LangArgs;
//!
//! // Configure scan
//! let global = GlobalArgs::default(); // needs proper init
//! let lang = LangArgs {
//!     paths: vec![],
//!     format: TableFormat::Json,
//!     top: 10,
//!     files: false,
//!     children: ChildrenMode::Collapse,
//! };
//!
//! // Run pipeline (without redaction)
//! let receipt = scan_workflow(&global, &lang, None).expect("Scan failed");
//! println!("Scanned {} languages", receipt.report.rows.len());
//!
//! // Run pipeline (with path redaction for safer LLM sharing)
//! let redacted = scan_workflow(&global, &lang, Some(RedactMode::Paths)).expect("Scan failed");
//! ```

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

// Re-export types for convenience
pub use tokmd_config as config;
pub use tokmd_types as types;

use tokmd_config::{GlobalArgs, RedactMode};
use tokmd_redact::{redact_path, short_hash};
use tokmd_types::{LangArgs, LangArgsMeta, LangReceipt, ScanArgs, ScanStatus, ToolInfo};

/// Runs the complete scan workflow: Scan -> Model -> Receipt.
///
/// This is the high-level entry point for generating a language inventory.
///
/// # Arguments
///
/// * `global` - Global scan configuration (excluded patterns, ignore settings, etc.)
/// * `lang` - Language-specific arguments (format, top N, etc.)
/// * `redact` - Optional redaction mode for safer output (e.g., when sharing with LLMs)
///
/// # Redaction Modes
///
/// * `None` or `Some(RedactMode::None)` - No redaction, paths shown as-is
/// * `Some(RedactMode::Paths)` - Redact file paths (replaced with hashed values preserving extension)
/// * `Some(RedactMode::All)` - Redact paths and excluded patterns
pub fn scan_workflow(
    global: &GlobalArgs,
    lang: &LangArgs,
    redact: Option<RedactMode>,
) -> Result<LangReceipt> {
    // 1. Scan
    let languages = tokmd_scan::scan(&lang.paths, global)?;

    // 2. Model (Aggregation & Filtering)
    // create_lang_report handles filtering (top N) and children mode
    let report = tokmd_model::create_lang_report(&languages, lang.top, lang.files, lang.children);

    // 3. Receipt Construction
    // We construct the receipt manually as it's just a data carrier.
    let scan_args = make_scan_args(&lang.paths, global, redact);

    let receipt = LangReceipt {
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![], // Tokei scan might have warnings but scan() doesn't return them currently
        scan: scan_args,
        args: LangArgsMeta {
            format: format!("{:?}", lang.format), // Enums might need Display impl or conversion
            top: lang.top,
            with_files: lang.files,
            children: lang.children,
        },
        report,
    };

    Ok(receipt)
}

// -------------------------
// Redaction helpers
// -------------------------

/// Normalize a path to forward slashes and strip leading `./` for cross-platform stability.
fn normalize_scan_input(p: &Path) -> String {
    let s = p.display().to_string().replace('\\', "/");
    s.strip_prefix("./").unwrap_or(&s).to_string()
}

/// Constructs `ScanArgs` with optional redaction applied.
fn make_scan_args(
    paths: &[std::path::PathBuf],
    global: &GlobalArgs,
    redact: Option<RedactMode>,
) -> ScanArgs {
    let redact = redact.unwrap_or(RedactMode::None);
    let should_redact = redact == RedactMode::Paths || redact == RedactMode::All;
    let excluded_redacted = should_redact && !global.excluded.is_empty();

    let mut args = ScanArgs {
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
        excluded: if should_redact {
            global.excluded.iter().map(|p| short_hash(p)).collect()
        } else {
            global.excluded.clone()
        },
        excluded_redacted,
        config: global.config,
        hidden: global.hidden,
        no_ignore: global.no_ignore,
        no_ignore_parent: global.no_ignore || global.no_ignore_parent,
        no_ignore_dot: global.no_ignore || global.no_ignore_dot,
        no_ignore_vcs: global.no_ignore || global.no_ignore_vcs,
        treat_doc_strings_as_comments: global.treat_doc_strings_as_comments,
    };

    if should_redact {
        args.paths = args.paths.iter().map(|p| redact_path(p)).collect();
    }

    args
}

