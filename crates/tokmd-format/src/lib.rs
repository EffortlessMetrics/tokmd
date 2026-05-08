//! # tokmd-format
//!
//! **Tier 2 (Formatting)**
//!
//! This crate handles the rendering and serialization of `tokmd` receipts.
//! It supports Markdown, TSV, JSON, JSONL, CSV, and CycloneDX formats.
//!
//! ## What belongs here
//! * Serialization logic (JSON/CSV/CycloneDX)
//! * Markdown and TSV table rendering
//! * Output file writing
//! * Redaction integration (via internal `redact` module)
//! * ScanArgs integration (via internal `scan_args` module)
//! * Analysis receipt rendering under [`analysis`]
//!
//! ## What does NOT belong here
//! * Business logic (calculating stats)
//! * CLI argument parsing
//! * Analysis computation (use tokmd-analysis)

use std::time::{SystemTime, UNIX_EPOCH};

use tokmd_types::RedactMode;

pub mod analysis;
pub mod badge;
mod diff;
mod export;
pub mod export_tree;
#[cfg(feature = "fun")]
pub mod fun;
pub mod redact;
pub mod scan_args;
mod summary;

pub use badge::badge_svg;
pub use diff::{
    DiffColorMode, DiffRenderOptions, compute_diff_rows, compute_diff_totals, create_diff_receipt,
    render_diff_md, render_diff_md_with_options,
};
pub use export::{
    write_export, write_export_csv_to, write_export_cyclonedx_to,
    write_export_cyclonedx_with_options, write_export_json_to, write_export_jsonl_to,
    write_export_jsonl_to_file,
};
pub use export_tree::{render_analysis_tree, render_handoff_tree};
pub use redact::{redact_path, short_hash};
pub use scan_args::{normalize_scan_input, scan_args};
pub use summary::{
    print_lang_report, print_module_report, write_lang_json_to_file, write_lang_report_to,
    write_module_json_to_file, write_module_report_to,
};

fn redact_module_roots(roots: &[String], redact: RedactMode) -> Vec<String> {
    if redact == RedactMode::All {
        roots.iter().map(|r| short_hash(r)).collect()
    } else {
        roots.to_vec()
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use proptest::prelude::*;

    use super::normalize_scan_input;

    #[test]
    fn normalize_scan_input_forward_slash() {
        let p = Path::new("src/lib.rs");
        let normalized = normalize_scan_input(p);
        assert_eq!(normalized, "src/lib.rs");
    }

    #[test]
    fn normalize_scan_input_backslash_to_forward() {
        let p = Path::new("src\\lib.rs");
        let normalized = normalize_scan_input(p);
        assert_eq!(normalized, "src/lib.rs");
    }

    #[test]
    fn normalize_scan_input_strips_dot_slash() {
        let p = Path::new("./src/lib.rs");
        let normalized = normalize_scan_input(p);
        assert_eq!(normalized, "src/lib.rs");
    }

    #[test]
    fn normalize_scan_input_current_dir() {
        let p = Path::new(".");
        let normalized = normalize_scan_input(p);
        assert_eq!(normalized, ".");
    }

    proptest! {
        #[test]
        fn normalize_scan_input_no_backslash(s in "[a-zA-Z0-9_/\\\\.]+") {
            let p = Path::new(&s);
            let normalized = normalize_scan_input(p);
            prop_assert!(!normalized.contains('\\'), "Should not contain backslash: {}", normalized);
        }

        #[test]
        fn normalize_scan_input_no_leading_dot_slash(s in "[a-zA-Z0-9_/\\\\.]+") {
            let p = Path::new(&s);
            let normalized = normalize_scan_input(p);
            prop_assert!(!normalized.starts_with("./"), "Should not start with ./: {}", normalized);
        }
    }
}
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub mod readme_doctests {}
