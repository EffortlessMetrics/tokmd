//! Additional proptest assertions to tighten property coverage.

use std::fs;
use std::path::PathBuf;

use proptest::prelude::*;
use tokmd_analysis_api_surface::build_api_surface_report;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_row(path: &str, module: &str, lang: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes: 100,
        tokens: 30,
    }
}

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

/// Strategy to produce random Rust source lines (pub/internal items).
fn rust_item_line() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("pub fn generated() {}".to_string()),
        Just("fn private_gen() {}".to_string()),
        Just("pub struct GenStruct;".to_string()),
        Just("struct PrivStruct;".to_string()),
        Just("pub enum GenEnum {}".to_string()),
        Just("enum PrivEnum {}".to_string()),
        Just("pub trait GenTrait {}".to_string()),
        Just("trait PrivTrait {}".to_string()),
        Just("pub const GEN_CONST: u32 = 0;".to_string()),
        Just("pub type GenType = u32;".to_string()),
        Just("/// Doc comment".to_string()),
        Just("// regular comment".to_string()),
        Just(String::new()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // Validate that missing files gracefully generate a zero report without panic
    #[test]
    fn prop_missing_files_graceful(
        _files in prop::collection::vec(rust_item_line(), 1..10)
    ) {
        let tmp = tempfile::tempdir().unwrap();
        let export = make_export(vec![make_row("missing.rs", ".", "Rust")]);
        let r = build_api_surface_report(
            tmp.path(),
            &[PathBuf::from("missing.rs")],
            &export,
            &default_limits(),
        ).unwrap();
        prop_assert_eq!(r.total_items, 0);
    }

    // Validate that duplicate files are counted correctly
    #[test]
    fn prop_duplicate_files_are_processed(
        lines in prop::collection::vec(rust_item_line(), 1..10)
    ) {
        let code = lines.join("\n") + "\n";
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("lib.rs"), &code).unwrap();
        fs::write(tmp.path().join("lib2.rs"), &code).unwrap();

        let export1 = make_export(vec![make_row("lib.rs", ".", "Rust")]);
        let r1 = build_api_surface_report(
            tmp.path(),
            &[PathBuf::from("lib.rs")],
            &export1,
            &default_limits(),
        ).unwrap();

        let export2 = make_export(vec![
            make_row("lib.rs", ".", "Rust"),
            make_row("lib2.rs", ".", "Rust"),
        ]);
        let r2 = build_api_surface_report(
            tmp.path(),
            &[PathBuf::from("lib.rs"), PathBuf::from("lib2.rs")],
            &export2,
            &default_limits(),
        ).unwrap();

        prop_assert_eq!(r2.total_items, r1.total_items * 2);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // Validate that number of extracted items matches exactly what was injected
    #[test]
    fn exact_item_count_matches(
        lines in prop::collection::vec(rust_item_line(), 1..20)
    ) {
        let code = lines.join("\n") + "\n";
        let expected_total = lines.iter().filter(|l| l.contains("pub ") || l.contains("fn ") || l.contains("struct ") || l.contains("enum ") || l.contains("trait ") || l.contains("type ") || l.contains("const ") || l.contains("static ") || l.contains("mod ")).count();
        let expected_public = lines.iter().filter(|l| l.contains("pub ")).count();

        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("lib.rs"), &code).unwrap();

        let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
        let r = build_api_surface_report(
            tmp.path(),
            &[PathBuf::from("lib.rs")],
            &export,
            &default_limits(),
        ).unwrap();

        // This validates symbol extraction model
        prop_assert_eq!(r.total_items, expected_total);
        prop_assert_eq!(r.public_items, expected_public);
    }
}
