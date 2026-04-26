//! Cross-crate integration tests (w58) exercising the full pipeline across
//! tier boundaries: types → scan → model → format → analysis-derived.
//!
//! Every test creates a temp directory with known source files so results
//! are deterministic and independent of the host repository layout.

mod common;

use tempfile::TempDir;
use tokmd_analysis::derive_report;
use tokmd_model::{
    collect_file_rows, create_export_data, create_lang_report, create_module_report, normalize_path,
};
use tokmd_scan::scan;
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ConfigMode, ExportArgs, ExportData, ExportFormat,
    ExportReceipt, FileKind, LangArgs, LangReceipt, LangReport, ModuleArgs, ModuleReceipt,
    ModuleReport, RedactMode, SCHEMA_VERSION, TableFormat,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temp directory with a `.git` marker and sample source files.
fn make_sample_project() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    let root = dir.path();

    // .git marker so `ignore` crate respects .gitignore
    std::fs::create_dir_all(root.join(".git")).unwrap();

    // src/main.rs
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src").join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    // src/lib.rs
    std::fs::write(
        root.join("src").join("lib.rs"),
        "/// Doc comment\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn it_works() {\n        assert_eq!(add(2, 2), 4);\n    }\n}\n",
    )
    .unwrap();

    // lib/utils.py
    std::fs::create_dir_all(root.join("lib")).unwrap();
    std::fs::write(
        root.join("lib").join("utils.py"),
        "# utility module\ndef greet(name):\n    return f\"Hello, {name}\"\n\ndef add(a, b):\n    return a + b\n",
    )
    .unwrap();

    // web/app.js
    std::fs::create_dir_all(root.join("web")).unwrap();
    std::fs::write(
        root.join("web").join("app.js"),
        "// main app\nfunction main() {\n  console.log('hello');\n}\nmain();\n",
    )
    .unwrap();

    dir
}

fn scan_opts() -> ScanOptions {
    ScanOptions {
        config: ConfigMode::None,
        no_ignore_vcs: true,
        ..Default::default()
    }
}

fn scan_lang_report(dir: &std::path::Path, children: ChildrenMode) -> LangReport {
    let langs = scan(&[dir.to_path_buf()], &scan_opts()).expect("scan");
    create_lang_report(&langs, 0, true, children)
}

fn scan_module_report(dir: &std::path::Path, children: ChildIncludeMode) -> ModuleReport {
    let langs = scan(&[dir.to_path_buf()], &scan_opts()).expect("scan");
    create_module_report(&langs, &[], 1, children, 0)
}

fn scan_export_data(dir: &std::path::Path, strip: Option<&std::path::Path>) -> ExportData {
    let langs = scan(&[dir.to_path_buf()], &scan_opts()).expect("scan");
    create_export_data(&langs, &[], 1, ChildIncludeMode::Separate, strip, 0, 0)
}

fn make_lang_args(dir: &std::path::Path, fmt: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![dir.to_path_buf()],
        format: fmt,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    }
}

fn make_module_args(dir: &std::path::Path, fmt: TableFormat) -> ModuleArgs {
    ModuleArgs {
        paths: vec![dir.to_path_buf()],
        format: fmt,
        top: 0,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn make_export_args(dir: &std::path::Path) -> ExportArgs {
    ExportArgs {
        paths: vec![dir.to_path_buf()],
        format: ExportFormat::Json,
        output: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: true,
        strip_prefix: Some(dir.to_path_buf()),
    }
}

// ===========================================================================
// 1. Scan → Model → Format pipeline with tempdir
// ===========================================================================

#[test]
fn scan_model_format_lang_md_contains_table() {
    let proj = make_sample_project();
    let report = scan_lang_report(proj.path(), ChildrenMode::Collapse);
    let args = make_lang_args(proj.path(), TableFormat::Md);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.contains("|Lang|"),
        "MD output must have table header"
    );
    assert!(
        output.contains("|**Total**|"),
        "MD output must have total row"
    );
}

#[test]
fn scan_model_format_lang_tsv_columns() {
    let proj = make_sample_project();
    let report = scan_lang_report(proj.path(), ChildrenMode::Collapse);
    let args = make_lang_args(proj.path(), TableFormat::Tsv);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    for line in output.lines().filter(|l| !l.is_empty()) {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            7,
            "TSV with files should have 7 columns: {line}"
        );
    }
}

#[test]
fn scan_model_format_lang_json_roundtrip() {
    let proj = make_sample_project();
    let report = scan_lang_report(proj.path(), ChildrenMode::Collapse);
    let args = make_lang_args(proj.path(), TableFormat::Json);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let receipt: LangReceipt =
        serde_json::from_slice(&buf).expect("JSON must deserialize into LangReceipt");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert!(
        !receipt.report.rows.is_empty(),
        "receipt must have lang rows"
    );
}

#[test]
fn scan_model_format_module_md_contains_table() {
    let proj = make_sample_project();
    let report = scan_module_report(proj.path(), ChildIncludeMode::Separate);
    let args = make_module_args(proj.path(), TableFormat::Md);
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("|Module|"), "Module MD must have header");
    assert!(output.contains("|**Total**|"), "Module MD must have totals");
}

#[test]
fn scan_model_format_module_json_roundtrip() {
    let proj = make_sample_project();
    let report = scan_module_report(proj.path(), ChildIncludeMode::Separate);
    let args = make_module_args(proj.path(), TableFormat::Json);
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let receipt: ModuleReceipt =
        serde_json::from_slice(&buf).expect("JSON must deserialize into ModuleReceipt");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn scan_model_format_export_json_roundtrip() {
    let proj = make_sample_project();
    let data = scan_export_data(proj.path(), Some(proj.path()));
    let args = make_export_args(proj.path());
    let mut buf = Vec::new();
    tokmd_format::write_export_json_to(&mut buf, &data, &scan_opts(), &args).unwrap();
    let receipt: ExportReceipt =
        serde_json::from_slice(&buf).expect("JSON must deserialize into ExportReceipt");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert!(!receipt.data.rows.is_empty());
}

// ===========================================================================
// 2. Scan → Model → Analysis-derived → Analysis-format pipeline
// ===========================================================================

#[test]
fn analysis_derived_from_tempdir_has_totals() {
    let proj = make_sample_project();
    let data = scan_export_data(proj.path(), Some(proj.path()));
    let derived = derive_report(&data, None);
    assert!(derived.totals.files > 0, "derived must report files");
    assert!(derived.totals.code > 0, "derived must report code lines");
    assert!(derived.totals.lines > 0, "derived must report total lines");
}

#[test]
fn analysis_derived_doc_density_valid() {
    let proj = make_sample_project();
    let data = scan_export_data(proj.path(), Some(proj.path()));
    let derived = derive_report(&data, None);
    let ratio = derived.doc_density.total.ratio;
    assert!(
        (0.0..=1.0).contains(&ratio),
        "doc_density ratio must be in [0,1], got {ratio}"
    );
}

#[test]
fn analysis_derived_integrity_hash_present() {
    let proj = make_sample_project();
    let data = scan_export_data(proj.path(), Some(proj.path()));
    let derived = derive_report(&data, None);
    assert!(
        !derived.integrity.hash.is_empty(),
        "integrity hash must be non-empty"
    );
}

#[test]
fn analysis_derived_cocomo_positive() {
    let proj = make_sample_project();
    let data = scan_export_data(proj.path(), Some(proj.path()));
    let derived = derive_report(&data, None);
    let cocomo = derived.cocomo.expect("COCOMO must be present");
    assert!(cocomo.kloc > 0.0);
    assert!(cocomo.effort_pm > 0.0);
}

#[test]
fn analysis_derived_context_window() {
    let proj = make_sample_project();
    let data = scan_export_data(proj.path(), Some(proj.path()));
    let derived = derive_report(&data, Some(128_000));
    let cw = derived
        .context_window
        .expect("context_window must be present when window_tokens given");
    assert_eq!(cw.window_tokens, 128_000);
    assert!(cw.total_tokens <= cw.window_tokens || !cw.fits);
}

// ===========================================================================
// 3. Module-key consistency between model and format layers
// ===========================================================================

#[test]
fn module_key_consistent_model_vs_export() {
    let proj = make_sample_project();
    let mod_report = scan_module_report(proj.path(), ChildIncludeMode::Separate);
    let exp_data = scan_export_data(proj.path(), None);

    // Every module key from export rows must appear in module report rows
    // (unless it's folded into "Other", which only happens with top > 0).
    let module_keys: std::collections::BTreeSet<_> =
        mod_report.rows.iter().map(|r| r.module.as_str()).collect();
    for row in &exp_data.rows {
        assert!(
            module_keys.contains(row.module.as_str()),
            "Export module key '{}' not found in module report",
            row.module
        );
    }
}

// ===========================================================================
// 4. Path normalization consistency across tiers
// ===========================================================================

#[test]
fn paths_use_forward_slashes_in_export() {
    let proj = make_sample_project();
    let data = scan_export_data(proj.path(), Some(proj.path()));
    for row in &data.rows {
        assert!(
            !row.path.contains('\\'),
            "Export path must use forward slashes: {}",
            row.path
        );
        assert!(
            !row.module.contains('\\'),
            "Module key must use forward slashes: {}",
            row.module
        );
    }
}

#[test]
fn paths_use_forward_slashes_in_collect_file_rows() {
    let proj = make_sample_project();
    let langs = scan(&[proj.path().to_path_buf()], &scan_opts()).expect("scan");
    let rows = collect_file_rows(&langs, &[], 1, ChildIncludeMode::Separate, None);
    for row in &rows {
        assert!(
            !row.path.contains('\\'),
            "collect_file_rows path must use forward slashes: {}",
            row.path
        );
    }
}

#[test]
fn normalize_path_strips_dot_prefix() {
    let p = std::path::Path::new("./src/main.rs");
    let norm = normalize_path(p, None);
    assert_eq!(norm, "src/main.rs");
    assert!(!norm.starts_with("./"));
}

#[test]
fn normalize_path_forward_slashes_on_backslash_input() {
    let p = std::path::Path::new("src\\lib.rs");
    let norm = normalize_path(p, None);
    assert_eq!(norm, "src/lib.rs");
}

// ===========================================================================
// 5. Children mode (collapse/separate) consistency across layers
// ===========================================================================

#[test]
fn children_collapse_no_embedded_label() {
    let proj = make_sample_project();
    let report = scan_lang_report(proj.path(), ChildrenMode::Collapse);
    for row in &report.rows {
        assert!(
            !row.lang.contains("(embedded)"),
            "Collapse mode must not have (embedded) rows, found: {}",
            row.lang
        );
    }
}

#[test]
fn children_separate_has_separate_tag() {
    let proj = make_sample_project();
    let report = scan_lang_report(proj.path(), ChildrenMode::Separate);
    assert_eq!(report.children, ChildrenMode::Separate);
}

#[test]
fn children_collapse_tag_preserved() {
    let proj = make_sample_project();
    let report = scan_lang_report(proj.path(), ChildrenMode::Collapse);
    assert_eq!(report.children, ChildrenMode::Collapse);
}

#[test]
fn children_separate_ge_collapse_rows() {
    let proj = make_sample_project();
    let collapse = scan_lang_report(proj.path(), ChildrenMode::Collapse);
    let separate = scan_lang_report(proj.path(), ChildrenMode::Separate);
    assert!(
        separate.rows.len() >= collapse.rows.len(),
        "Separate should have >= rows than collapse"
    );
}

#[test]
fn children_include_mode_parents_only_no_child_kind() {
    let proj = make_sample_project();
    let langs = scan(&[proj.path().to_path_buf()], &scan_opts()).expect("scan");
    let data = create_export_data(&langs, &[], 1, ChildIncludeMode::ParentsOnly, None, 0, 0);
    for row in &data.rows {
        assert_eq!(
            row.kind,
            FileKind::Parent,
            "ParentsOnly must not contain child rows"
        );
    }
}

// ===========================================================================
// 6. Sorting order maintained through pipeline
// ===========================================================================

#[test]
fn lang_rows_sorted_code_desc_name_asc() {
    let proj = make_sample_project();
    let report = scan_lang_report(proj.path(), ChildrenMode::Collapse);
    for window in report.rows.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let ok = a.code > b.code || (a.code == b.code && a.lang <= b.lang);
        assert!(
            ok,
            "Lang rows not sorted: {} ({}) vs {} ({})",
            a.lang, a.code, b.lang, b.code
        );
    }
}

#[test]
fn module_rows_sorted_code_desc_module_asc() {
    let proj = make_sample_project();
    let report = scan_module_report(proj.path(), ChildIncludeMode::Separate);
    for window in report.rows.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let ok = a.code > b.code || (a.code == b.code && a.module <= b.module);
        assert!(
            ok,
            "Module rows not sorted: {} ({}) vs {} ({})",
            a.module, a.code, b.module, b.code
        );
    }
}

#[test]
fn export_rows_sorted_code_desc_path_asc() {
    let proj = make_sample_project();
    let data = scan_export_data(proj.path(), Some(proj.path()));
    for window in data.rows.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let ok = a.code > b.code || (a.code == b.code && a.path <= b.path);
        assert!(
            ok,
            "Export rows not sorted: {} ({}) vs {} ({})",
            a.path, a.code, b.path, b.code
        );
    }
}

// ===========================================================================
// 7. Configuration propagation from settings through stack
// ===========================================================================

#[test]
fn exclude_pattern_filters_language() {
    let proj = make_sample_project();
    let opts = ScanOptions {
        excluded: vec!["*.js".to_string()],
        config: ConfigMode::None,
        no_ignore_vcs: true,
        ..Default::default()
    };
    let langs = scan(&[proj.path().to_path_buf()], &opts).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let has_js = report.rows.iter().any(|r| r.lang == "JavaScript");
    assert!(!has_js, "JavaScript must be excluded by *.js pattern");
}

#[test]
fn exclude_pattern_preserves_other_langs() {
    let proj = make_sample_project();
    let opts = ScanOptions {
        excluded: vec!["*.js".to_string()],
        config: ConfigMode::None,
        no_ignore_vcs: true,
        ..Default::default()
    };
    let langs = scan(&[proj.path().to_path_buf()], &opts).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let has_rust = report.rows.iter().any(|r| r.lang == "Rust");
    assert!(
        has_rust,
        "Rust must still be present when only *.js excluded"
    );
}

#[test]
fn top_n_truncates_lang_rows() {
    let proj = make_sample_project();
    let langs = scan(&[proj.path().to_path_buf()], &scan_opts()).expect("scan");
    let report = create_lang_report(&langs, 1, false, ChildrenMode::Collapse);
    // top=1 means 1 real row + 1 "Other" row (if more langs exist)
    assert!(
        report.rows.len() <= 2,
        "top=1 should yield at most 2 rows (1 + Other), got {}",
        report.rows.len()
    );
}

// ===========================================================================
// 8. JSON output from format layer deserializes into types layer
// ===========================================================================

#[test]
fn lang_json_schema_version_matches_constant() {
    let proj = make_sample_project();
    let report = scan_lang_report(proj.path(), ChildrenMode::Collapse);
    let args = make_lang_args(proj.path(), TableFormat::Json);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(
        v["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
}

#[test]
fn module_json_schema_version_matches_constant() {
    let proj = make_sample_project();
    let report = scan_module_report(proj.path(), ChildIncludeMode::Separate);
    let args = make_module_args(proj.path(), TableFormat::Json);
    let mut buf = Vec::new();
    tokmd_format::write_module_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(
        v["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
}

#[test]
fn lang_json_flattened_no_report_key() {
    let proj = make_sample_project();
    let report = scan_lang_report(proj.path(), ChildrenMode::Collapse);
    let args = make_lang_args(proj.path(), TableFormat::Json);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
    assert!(
        v.get("report").is_none(),
        "report key must be flattened away"
    );
    assert!(v.get("rows").is_some(), "rows must be at top level");
    assert!(v.get("total").is_some(), "total must be at top level");
}

// ===========================================================================
// 9. Determinism: same input → identical output across 3 runs
// ===========================================================================

#[test]
fn determinism_lang_report_three_runs() {
    let proj = make_sample_project();
    let mut results = Vec::new();
    for _ in 0..3 {
        let report = scan_lang_report(proj.path(), ChildrenMode::Collapse);
        let row_data: Vec<(String, usize, usize)> = report
            .rows
            .iter()
            .map(|r| (r.lang.clone(), r.code, r.files))
            .collect();
        results.push(row_data);
    }
    assert_eq!(results[0], results[1], "Run 1 vs 2 must be identical");
    assert_eq!(results[1], results[2], "Run 2 vs 3 must be identical");
}

#[test]
fn determinism_module_report_three_runs() {
    let proj = make_sample_project();
    let mut results = Vec::new();
    for _ in 0..3 {
        let report = scan_module_report(proj.path(), ChildIncludeMode::Separate);
        let row_data: Vec<(String, usize, usize)> = report
            .rows
            .iter()
            .map(|r| (r.module.clone(), r.code, r.files))
            .collect();
        results.push(row_data);
    }
    assert_eq!(results[0], results[1]);
    assert_eq!(results[1], results[2]);
}

#[test]
fn determinism_export_data_three_runs() {
    let proj = make_sample_project();
    let mut results = Vec::new();
    for _ in 0..3 {
        let data = scan_export_data(proj.path(), Some(proj.path()));
        let row_data: Vec<(String, String, usize)> = data
            .rows
            .iter()
            .map(|r| (r.path.clone(), r.lang.clone(), r.code))
            .collect();
        results.push(row_data);
    }
    assert_eq!(results[0], results[1]);
    assert_eq!(results[1], results[2]);
}

#[test]
fn determinism_derived_integrity_hash() {
    let proj = make_sample_project();
    let mut hashes = Vec::new();
    for _ in 0..3 {
        let data = scan_export_data(proj.path(), Some(proj.path()));
        let derived = derive_report(&data, None);
        hashes.push(derived.integrity.hash.clone());
    }
    assert_eq!(hashes[0], hashes[1], "Hash run 1 vs 2");
    assert_eq!(hashes[1], hashes[2], "Hash run 2 vs 3");
}

// ===========================================================================
// 10. Feature-gated paths don't leak into default builds
// ===========================================================================

#[test]
fn export_data_rows_only_contain_known_kinds() {
    let proj = make_sample_project();
    let data = scan_export_data(proj.path(), Some(proj.path()));
    for row in &data.rows {
        assert!(
            row.kind == FileKind::Parent || row.kind == FileKind::Child,
            "FileKind must be Parent or Child"
        );
    }
}

#[test]
fn lang_report_totals_equal_row_sums() {
    let proj = make_sample_project();
    let report = scan_lang_report(proj.path(), ChildrenMode::Collapse);
    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(
        report.total.code, row_code,
        "total.code must equal sum of row codes"
    );
}

#[test]
fn module_report_totals_code_consistent() {
    let proj = make_sample_project();
    let report = scan_module_report(proj.path(), ChildIncludeMode::Separate);
    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(
        report.total.code, row_code,
        "module total.code must match row sum"
    );
}
