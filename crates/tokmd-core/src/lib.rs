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
//! use tokmd_config::{ChildrenMode, GlobalArgs, TableFormat};
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
//! // Run pipeline
//! let receipt = scan_workflow(&global, &lang).expect("Scan failed");
//! println!("Scanned {} languages", receipt.report.rows.len());
//! ```

use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

// Re-export types for convenience
pub use tokmd_config as config;
pub use tokmd_types as types;

use tokmd_config::GlobalArgs;
use tokmd_types::{LangArgs, LangArgsMeta, LangReceipt, ScanArgs, ScanStatus, ToolInfo};

/// Runs the complete scan workflow: Scan -> Model -> Receipt.
///
/// This is the high-level entry point for generating a language inventory.
pub fn scan_workflow(global: &GlobalArgs, lang: &LangArgs) -> Result<LangReceipt> {
    // 1. Scan
    let languages = tokmd_scan::scan(&lang.paths, global)?;

    // 2. Model (Aggregation & Filtering)
    // create_lang_report handles filtering (top N) and children mode
    let report = tokmd_model::create_lang_report(&languages, lang.top, lang.files, lang.children);

    // 3. Receipt Construction
    // We construct the receipt manually as it's just a data carrier.
    let receipt = LangReceipt {
        schema_version: tokmd_types::SCHEMA_VERSION,
        generated_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        tool: ToolInfo {
            name: "tokmd".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![], // Tokei scan might have warnings but scan() doesn't return them currently
        scan: ScanArgs {
            paths: lang
                .paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            excluded: global.excluded.clone(),
            config: global.config,
            hidden: global.hidden,
            no_ignore: global.no_ignore,
            no_ignore_parent: global.no_ignore || global.no_ignore_parent,
            no_ignore_dot: global.no_ignore || global.no_ignore_dot,
            no_ignore_vcs: global.no_ignore || global.no_ignore_vcs,
            treat_doc_strings_as_comments: global.treat_doc_strings_as_comments,
        },
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
