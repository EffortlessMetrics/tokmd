//! Property-based tests for `tokmd-analysis-api-surface`.
//!
//! These tests verify invariants that must hold for *any* valid input,
//! regardless of specific file content.

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

// ---------------------------------------------------------------------------
// Property: public_items + internal_items == total_items
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn total_equals_public_plus_internal(
        lines in prop::collection::vec(rust_item_line(), 0..50)
    ) {
        let code = lines.join("\n") + "\n";
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lib.rs");
        fs::write(&path, &code).unwrap();

        let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
        let paths = vec![PathBuf::from("lib.rs")];
        let report = build_api_surface_report(
            dir.path(), &paths, &export, &default_limits(),
        ).unwrap();

        prop_assert_eq!(
            report.total_items,
            report.public_items + report.internal_items,
            "total must equal public + internal"
        );
    }
}

// ---------------------------------------------------------------------------
// Property: public_ratio is in [0.0, 1.0]
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn public_ratio_in_unit_range(
        lines in prop::collection::vec(rust_item_line(), 0..50)
    ) {
        let code = lines.join("\n") + "\n";
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("lib.rs"), &code).unwrap();

        let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
        let paths = vec![PathBuf::from("lib.rs")];
        let report = build_api_surface_report(
            dir.path(), &paths, &export, &default_limits(),
        ).unwrap();

        prop_assert!(report.public_ratio >= 0.0, "ratio must be >= 0");
        prop_assert!(report.public_ratio <= 1.0, "ratio must be <= 1");
    }
}

// ---------------------------------------------------------------------------
// Property: documented_ratio is in [0.0, 1.0]
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn documented_ratio_in_unit_range(
        lines in prop::collection::vec(rust_item_line(), 0..50)
    ) {
        let code = lines.join("\n") + "\n";
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("lib.rs"), &code).unwrap();

        let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
        let paths = vec![PathBuf::from("lib.rs")];
        let report = build_api_surface_report(
            dir.path(), &paths, &export, &default_limits(),
        ).unwrap();

        prop_assert!(report.documented_ratio >= 0.0, "doc ratio must be >= 0");
        prop_assert!(report.documented_ratio <= 1.0, "doc ratio must be <= 1");
    }
}

// ---------------------------------------------------------------------------
// Property: by_language totals sum to top-level totals
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn lang_breakdown_sums_to_totals(
        lines in prop::collection::vec(rust_item_line(), 0..30)
    ) {
        let code = lines.join("\n") + "\n";
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("lib.rs"), &code).unwrap();

        let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
        let paths = vec![PathBuf::from("lib.rs")];
        let report = build_api_surface_report(
            dir.path(), &paths, &export, &default_limits(),
        ).unwrap();

        let lang_total: usize = report.by_language.values().map(|l| l.total_items).sum();
        let lang_pub: usize = report.by_language.values().map(|l| l.public_items).sum();
        let lang_int: usize = report.by_language.values().map(|l| l.internal_items).sum();

        prop_assert_eq!(report.total_items, lang_total);
        prop_assert_eq!(report.public_items, lang_pub);
        prop_assert_eq!(report.internal_items, lang_int);
    }
}

// ---------------------------------------------------------------------------
// Property: by_module is sorted descending by total_items
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn modules_sorted_descending(
        lines_a in prop::collection::vec(rust_item_line(), 0..20),
        lines_b in prop::collection::vec(rust_item_line(), 0..20),
    ) {
        let code_a = lines_a.join("\n") + "\n";
        let code_b = lines_b.join("\n") + "\n";
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a")).unwrap();
        fs::create_dir_all(dir.path().join("b")).unwrap();
        fs::write(dir.path().join("a/lib.rs"), &code_a).unwrap();
        fs::write(dir.path().join("b/lib.rs"), &code_b).unwrap();

        let export = make_export(vec![
            make_row("a/lib.rs", "mod_a", "Rust"),
            make_row("b/lib.rs", "mod_b", "Rust"),
        ]);
        let paths = vec![
            PathBuf::from("a/lib.rs"),
            PathBuf::from("b/lib.rs"),
        ];
        let report = build_api_surface_report(
            dir.path(), &paths, &export, &default_limits(),
        ).unwrap();

        for w in report.by_module.windows(2) {
            prop_assert!(
                w[0].total_items >= w[1].total_items,
                "modules must be sorted descending by total_items"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property: top_exporters only contains files with public_items > 0
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn top_exporters_have_public_items(
        lines in prop::collection::vec(rust_item_line(), 0..30)
    ) {
        let code = lines.join("\n") + "\n";
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("lib.rs"), &code).unwrap();

        let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
        let paths = vec![PathBuf::from("lib.rs")];
        let report = build_api_surface_report(
            dir.path(), &paths, &export, &default_limits(),
        ).unwrap();

        for item in &report.top_exporters {
            prop_assert!(item.public_items > 0, "top exporter must have public items");
        }
    }
}

// ---------------------------------------------------------------------------
// Property: top_exporters sorted descending by public_items
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn top_exporters_sorted_descending(
        lines_a in prop::collection::vec(rust_item_line(), 0..20),
        lines_b in prop::collection::vec(rust_item_line(), 0..20),
    ) {
        let code_a = lines_a.join("\n") + "\n";
        let code_b = lines_b.join("\n") + "\n";
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), &code_a).unwrap();
        fs::write(dir.path().join("b.rs"), &code_b).unwrap();

        let export = make_export(vec![
            make_row("a.rs", ".", "Rust"),
            make_row("b.rs", ".", "Rust"),
        ]);
        let paths = vec![
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
        ];
        let report = build_api_surface_report(
            dir.path(), &paths, &export, &default_limits(),
        ).unwrap();

        for w in report.top_exporters.windows(2) {
            prop_assert!(
                w[0].public_items >= w[1].public_items,
                "top exporters must be sorted descending by public_items"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property: empty input always yields zero-valued report
// ---------------------------------------------------------------------------

#[test]
fn empty_input_always_yields_zeros() {
    let dir = tempfile::tempdir().unwrap();
    let export = make_export(vec![]);
    let report =
        build_api_surface_report(dir.path(), &[], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 0);
    assert_eq!(report.public_ratio, 0.0);
    assert_eq!(report.documented_ratio, 0.0);
}
