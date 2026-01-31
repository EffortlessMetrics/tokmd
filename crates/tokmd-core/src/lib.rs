//! # tokmd-core
//!
//! **Tier 4 (Library Facade)**
//!
//! This crate is the **primary library interface** for `tokmd`.
//! It coordinates scanning, aggregation, and modeling to produce code inventory receipts.
//!
//! If you are embedding `tokmd` into another Rust application, depend on this crate
//! and `tokmd-types`. Avoid depending on `tokmd-scan` or `tokmd-model` directly unless necessary.
//!
//! ## What belongs here
//! * High-level workflow coordination
//! * Simplified API for library consumers
//! * Re-exports for convenience
//!
//! ## What does NOT belong here
//! * CLI argument parsing (use tokmd crate)
//! * Low-level scanning logic (use tokmd-scan)
//! * Aggregation details (use tokmd-model)
//!
//! ## Example
//!
//! ```rust,no_run
//! use tokmd_core::scan_workflow;
//! use tokmd_config::GlobalArgs;
//! use tokmd_types::{ChildrenMode, LangArgs, RedactMode, TableFormat};
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

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

// Re-export types for convenience
pub use tokmd_config as config;
pub use tokmd_types as types;

use tokmd_config::GlobalArgs;
use tokmd_format::scan_args;
use tokmd_types::{LangArgs, LangArgsMeta, LangReceipt, RedactMode, ScanStatus, ToolInfo};

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
    let scan_args = scan_args(&lang.paths, global, redact);

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
