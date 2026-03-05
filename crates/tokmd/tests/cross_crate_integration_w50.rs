//! Cross-crate integration tests verifying complete data flow through the
//! tiered microcrate architecture: types → scan → model → format → analysis.

mod common;

use std::path::PathBuf;

use tokmd_analysis_derived::derive_report;
use tokmd_model::{create_export_data, create_lang_report, create_module_report};
use tokmd_scan::scan;
use tokmd_settings::{ScanOptions, TomlConfig};
use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ConfigMode, ExportArgs, ExportData, ExportFormat,
    ExportReceipt, LangArgs, LangReceipt, LangReport, ModuleReceipt, RedactMode, SCHEMA_VERSION,
    TableFormat,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fixture_path() -> PathBuf {
    common::fixture_root().to_path_buf()
}

fn default_scan_options() -> ScanOptions {
    ScanOptions {
        config: ConfigMode::None,
        no_ignore_vcs: true,
        ..Default::default()
    }
}

fn lang_report_collapse() -> LangReport {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan fixture");
    create_lang_report(&langs, 0, false, ChildrenMode::Collapse)
}

fn lang_report_separate() -> LangReport {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan fixture");
    create_lang_report(&langs, 0, false, ChildrenMode::Separate)
}

fn export_data() -> ExportData {
    let langs = scan(&[fixture_path()], &default_scan_options()).expect("scan fixture");
    create_export_data(
        &langs,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(fixture_path().as_path()),
        0,
        0,
    )
}

fn default_lang_args(fmt: TableFormat) -> LangArgs {
    LangArgs {
        paths: vec![fixture_path()],
        format: fmt,
        top: 0,
        files: false,
        children: ChildrenMode::Collapse,
    }
}

fn default_export_args() -> ExportArgs {
    ExportArgs {
        paths: vec![fixture_path()],
        format: ExportFormat::Csv,
        output: None,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        meta: false,
        strip_prefix: None,
    }
}

// ===========================================================================
// 1. Scan → Model → Format pipeline
// ===========================================================================

#[test]
fn scan_model_format_markdown_has_table_header() {
    let report = lang_report_collapse();
    let scan_opts = default_scan_options();
    let args = default_lang_args(TableFormat::Md);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.contains("|Lang|"),
        "Markdown must contain table header"
    );
    assert!(
        output.contains("|**Total**|"),
        "Markdown must contain total row"
    );
}

#[test]
fn scan_model_format_tsv_column_count() {
    let report = lang_report_collapse();
    let scan_opts = default_scan_options();
    let args = default_lang_args(TableFormat::Tsv);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    for line in output.lines().filter(|l| !l.is_empty()) {
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(
            cols.len() >= 5,
            "TSV line should have ≥5 columns, got {}: {line}",
            cols.len()
        );
    }
}

#[test]
fn scan_model_format_json_has_schema_version() {
    let report = lang_report_collapse();
    let scan_opts = default_scan_options();
    let args = default_lang_args(TableFormat::Json);
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts, &args).unwrap();
    let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(
        v["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
}

#[test]
fn scan_model_format_json_total_consistency() {
    let report = lang_report_collapse();
    let row_code_sum: usize = report.rows.iter().map(|r| r.code).sum();
    assert_eq!(
        report.total.code, row_code_sum,
        "total.code must equal sum of row code values"
    );
    let row_files_sum: usize = report.rows.iter().map(|r| r.files).sum();
    assert_eq!(
        report.total.files, row_files_sum,
        "total.files must equal sum of row files values"
    );
}

#[test]
fn scan_model_format_separate_has_more_or_equal_rows() {
    let collapse = lang_report_collapse();
    let separate = lang_report_separate();
    assert!(
        separate.rows.len() >= collapse.rows.len(),
        "Separate mode should produce >= rows than Collapse"
    );
}

#[test]
fn scan_model_format_collapse_children_mode_tag() {
    let report = lang_report_collapse();
    assert_eq!(report.children, ChildrenMode::Collapse);
}

#[test]
fn scan_model_format_separate_children_mode_tag() {
    let report = lang_report_separate();
    assert_eq!(report.children, ChildrenMode::Separate);
}

// ===========================================================================
// 2. Scan → Model → Export pipeline
// ===========================================================================

#[test]
fn export_csv_has_header_and_rows() {
    let data = export_data();
    let args = default_export_args();
    let mut buf = Vec::new();
    tokmd_format::write_export_csv_to(&mut buf, &data, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert!(
        lines.len() >= 2,
        "CSV must have header + at least one data row"
    );
    assert!(
        lines[0].contains("path"),
        "CSV header must contain 'path' column"
    );
}

#[test]
fn export_all_paths_use_forward_slashes() {
    let data = export_data();
    for row in &data.rows {
        assert!(
            !row.path.contains('\\'),
            "Path must use forward slashes: {}",
            row.path
        );
    }
}

#[test]
fn export_deterministic_sort_order() {
    let data = export_data();
    for window in data.rows.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let order_ok = a.code > b.code || (a.code == b.code && a.path <= b.path);
        assert!(
            order_ok,
            "Rows must be sorted by code desc, then path asc: {} ({}) vs {} ({})",
            a.path, a.code, b.path, b.code
        );
    }
}

#[test]
fn export_jsonl_one_object_per_line() {
    let data = export_data();
    let scan_opts = default_scan_options();
    let args = default_export_args();
    let mut buf = Vec::new();
    tokmd_format::write_export_jsonl_to(&mut buf, &data, &scan_opts, &args).unwrap();
    let output = String::from_utf8(buf).unwrap();
    for line in output.lines().filter(|l| !l.is_empty()) {
        let parsed: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("Each JSONL line must be valid JSON: {e}\nLine: {line}"));
        assert!(parsed.is_object(), "Each JSONL line must be a JSON object");
    }
}

// ===========================================================================
// 3. Scan → Analysis pipeline
// ===========================================================================

#[test]
fn analysis_derived_report_has_totals() {
    let data = export_data();
    let derived = derive_report(&data, None);
    assert!(
        derived.totals.files > 0,
        "Derived totals must have at least one file"
    );
    assert!(
        derived.totals.code > 0,
        "Derived totals must have non-zero code lines"
    );
}

#[test]
fn analysis_density_is_valid_ratio() {
    let data = export_data();
    let derived = derive_report(&data, None);
    let ratio = derived.doc_density.total.ratio;
    assert!(
        (0.0..=1.0).contains(&ratio),
        "doc_density ratio must be in [0, 1], got {ratio}"
    );
}

#[test]
fn analysis_cocomo_estimates_positive() {
    let data = export_data();
    let derived = derive_report(&data, None);
    let cocomo = derived
        .cocomo
        .expect("COCOMO must be present for non-empty codebase");
    assert!(cocomo.kloc > 0.0, "COCOMO kloc must be positive");
    assert!(cocomo.effort_pm > 0.0, "COCOMO effort_pm must be positive");
    assert!(
        cocomo.duration_months > 0.0,
        "COCOMO duration_months must be positive"
    );
}

#[test]
fn analysis_integrity_hash_present() {
    let data = export_data();
    let derived = derive_report(&data, None);
    assert!(
        !derived.integrity.hash.is_empty(),
        "Integrity hash must be present"
    );
}

// ===========================================================================
// 4. Config → Scan → Model pipeline
// ===========================================================================

#[test]
fn config_parse_minimal_toml() {
    let toml_str = r#"
[scan]
exclude = ["*.md"]
"#;
    let config = TomlConfig::parse(toml_str).expect("valid TOML");
    let excludes = config.scan.exclude.unwrap_or_default();
    assert_eq!(excludes, vec!["*.md"]);
}

#[test]
fn config_exclude_pattern_honored() {
    let opts = ScanOptions {
        excluded: vec!["*.js".to_string()],
        config: ConfigMode::None,
        no_ignore_vcs: true,
        ..Default::default()
    };
    let languages = scan(&[fixture_path()], &opts).expect("scan with excludes");
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let has_js = report.rows.iter().any(|r| r.lang == "JavaScript");
    assert!(!has_js, "JavaScript should be excluded by *.js pattern");
}

#[test]
fn config_exclude_does_not_affect_other_langs() {
    let opts = ScanOptions {
        excluded: vec!["*.js".to_string()],
        config: ConfigMode::None,
        no_ignore_vcs: true,
        ..Default::default()
    };
    let languages = scan(&[fixture_path()], &opts).expect("scan with excludes");
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let has_rust = report.rows.iter().any(|r| r.lang == "Rust");
    assert!(
        has_rust,
        "Rust should still be present when only *.js excluded"
    );
}

// ===========================================================================
// 5. Types contract verification
// ===========================================================================

#[test]
fn receipt_lang_serialize_has_schema_version() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tokmd_types::ToolInfo {
            name: "tokmd".into(),
            version: "0.0.0-test".into(),
        },
        mode: "lang".into(),
        status: tokmd_types::ScanStatus::Complete,
        warnings: vec![],
        scan: tokmd_types::ScanArgs {
            paths: vec![".".into()],
            excluded: vec![],
            excluded_redacted: false,
            config: ConfigMode::None,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        },
        args: tokmd_types::LangArgsMeta {
            format: "md".into(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![],
            total: tokmd_types::Totals {
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    };
    let json = serde_json::to_value(&receipt).unwrap();
    assert_eq!(json["schema_version"], SCHEMA_VERSION);
    assert_eq!(json["mode"], "lang");
}

#[test]
fn receipt_lang_flatten_rows_at_top_level() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: Default::default(),
        mode: "lang".into(),
        status: tokmd_types::ScanStatus::Complete,
        warnings: vec![],
        scan: tokmd_types::ScanArgs {
            paths: vec![],
            excluded: vec![],
            excluded_redacted: false,
            config: ConfigMode::None,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        },
        args: tokmd_types::LangArgsMeta {
            format: "json".into(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![tokmd_types::LangRow {
                lang: "Rust".into(),
                code: 100,
                lines: 120,
                files: 2,
                bytes: 3000,
                tokens: 500,
                avg_lines: 60,
            }],
            total: tokmd_types::Totals {
                code: 100,
                lines: 120,
                files: 2,
                bytes: 3000,
                tokens: 500,
                avg_lines: 60,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    };
    let json = serde_json::to_value(&receipt).unwrap();
    // #[serde(flatten)] puts rows at top level, not nested under "report"
    assert!(
        json.get("report").is_none(),
        "report key must not exist (flattened)"
    );
    assert!(json.get("rows").is_some(), "rows must be at top level");
    assert!(json.get("total").is_some(), "total must be at top level");
}

#[test]
fn receipt_module_flatten_rows_at_top_level() {
    let receipt = ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: Default::default(),
        mode: "module".into(),
        status: tokmd_types::ScanStatus::Complete,
        warnings: vec![],
        scan: tokmd_types::ScanArgs {
            paths: vec![],
            excluded: vec![],
            excluded_redacted: false,
            config: ConfigMode::None,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        },
        args: tokmd_types::ModuleArgsMeta {
            format: "json".into(),
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
        report: tokmd_types::ModuleReport {
            rows: vec![],
            total: tokmd_types::Totals {
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
    };
    let json = serde_json::to_value(&receipt).unwrap();
    assert!(
        json.get("report").is_none(),
        "report key must not exist (flattened)"
    );
    assert!(json.get("rows").is_some(), "rows must be at top level");
}

#[test]
fn receipt_export_flatten_data_at_top_level() {
    let receipt = ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: Default::default(),
        mode: "export".into(),
        status: tokmd_types::ScanStatus::Complete,
        warnings: vec![],
        scan: tokmd_types::ScanArgs {
            paths: vec![],
            excluded: vec![],
            excluded_redacted: false,
            config: ConfigMode::None,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        },
        args: tokmd_types::ExportArgsMeta {
            format: ExportFormat::Csv,
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        },
        data: ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        },
    };
    let json = serde_json::to_value(&receipt).unwrap();
    assert!(
        json.get("data").is_none(),
        "data key must not exist (flattened)"
    );
    assert!(json.get("rows").is_some(), "rows must be at top level");
}

#[test]
fn receipts_implement_clone_debug_serde() {
    // Verify Clone + Debug + Serialize + Deserialize via round-trip.
    let report = lang_report_collapse();
    let cloned = report.clone();
    let debug_str = format!("{:?}", cloned);
    assert!(!debug_str.is_empty(), "Debug must produce output");

    let json = serde_json::to_string(&cloned).unwrap();
    let deser: LangReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.total.code, report.total.code);
}

// ===========================================================================
// 6. Determinism contracts
// ===========================================================================

#[test]
fn determinism_identical_json_across_runs() {
    let opts = default_scan_options();
    let path = fixture_path();

    let langs1 = scan(std::slice::from_ref(&path), &opts).unwrap();
    let report1 = create_lang_report(&langs1, 0, false, ChildrenMode::Collapse);

    let langs2 = scan(std::slice::from_ref(&path), &opts).unwrap();
    let report2 = create_lang_report(&langs2, 0, false, ChildrenMode::Collapse);

    let json1 = serde_json::to_string(&report1).unwrap();
    let json2 = serde_json::to_string(&report2).unwrap();
    assert_eq!(json1, json2, "Same scan must produce byte-identical JSON");
}

#[test]
fn determinism_export_identical_across_runs() {
    let opts = default_scan_options();
    let path = fixture_path();

    let langs1 = scan(std::slice::from_ref(&path), &opts).unwrap();
    let data1 = create_export_data(
        &langs1,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(path.as_path()),
        0,
        0,
    );

    let langs2 = scan(std::slice::from_ref(&path), &opts).unwrap();
    let data2 = create_export_data(
        &langs2,
        &[],
        2,
        ChildIncludeMode::Separate,
        Some(path.as_path()),
        0,
        0,
    );

    let json1 = serde_json::to_string(&data1).unwrap();
    let json2 = serde_json::to_string(&data2).unwrap();
    assert_eq!(json1, json2, "Same export must produce byte-identical JSON");
}

#[test]
fn determinism_no_backslash_in_paths() {
    let data = export_data();
    let json = serde_json::to_string_pretty(&data).unwrap();
    // Check the raw JSON string for backslashes that are NOT escape sequences
    for line in json.lines() {
        if line.contains("\"path\"") {
            assert!(
                !line.contains("\\\\"),
                "Paths in JSON must not contain backslashes: {line}"
            );
        }
    }
}

#[test]
fn determinism_sort_stability_with_tiebreak() {
    let data = export_data();
    // Collect paths grouped by code count to verify tie-breaking by name.
    let mut prev_code = usize::MAX;
    let mut prev_path = String::new();
    for row in &data.rows {
        if row.code == prev_code {
            assert!(
                row.path >= prev_path,
                "Tie-break must sort by path ascending: {} vs {}",
                prev_path,
                row.path
            );
        } else {
            assert!(
                row.code <= prev_code,
                "Primary sort must be code descending: {} vs {}",
                prev_code,
                row.code
            );
        }
        prev_code = row.code;
        prev_path = row.path.clone();
    }
}

#[test]
fn determinism_module_report_stable() {
    let opts = default_scan_options();
    let path = fixture_path();
    let langs = scan(&[path], &opts).unwrap();

    let r1 = create_module_report(&langs, &[], 2, ChildIncludeMode::Separate, 0);
    let r2 = create_module_report(&langs, &[], 2, ChildIncludeMode::Separate, 0);

    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2, "Module report must be deterministic");
}
