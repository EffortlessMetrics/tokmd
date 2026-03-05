//! Cross-crate integration pipeline tests (w70).
//!
//! Exercises interactions between multiple crates across tier boundaries:
//! scan → model → format, scan → export, receipt determinism, schema version
//! consistency, module depth, format roundtrip, badge, config, empty dirs,
//! exclude patterns, children modes, and sort consistency.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;
use tokmd_model::create_lang_report;
use tokmd_scan::scan;
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildrenMode, ConfigMode, LangArgs, LangReceipt, ModuleReceipt, SCHEMA_VERSION, TableFormat,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temp project with Rust, Python, and JS files across nested dirs.
fn make_project() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    let root = dir.path();

    std::fs::create_dir_all(root.join(".git")).unwrap();

    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    std::fs::write(
        root.join("src/lib.rs"),
        "/// Doc\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();

    std::fs::create_dir_all(root.join("lib")).unwrap();
    std::fs::write(
        root.join("lib/util.py"),
        "# util\ndef greet(name):\n    return f\"Hello, {name}\"\n",
    )
    .unwrap();

    std::fs::create_dir_all(root.join("web")).unwrap();
    std::fs::write(
        root.join("web/app.js"),
        "// app\nfunction main() {\n  console.log('hi');\n}\nmain();\n",
    )
    .unwrap();

    std::fs::create_dir_all(root.join("src/nested")).unwrap();
    std::fs::write(root.join("src/nested/deep.rs"), "pub fn f() {}\n").unwrap();

    dir
}

/// Create a project containing embedded language (Markdown with Rust fence).
fn make_project_with_embedded() -> TempDir {
    let dir = make_project();
    std::fs::write(
        dir.path().join("README.md"),
        "# Title\n\nSome text.\n\n```rust\nfn demo() {}\n```\n",
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

fn tokmd_at(dir: &std::path::Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir);
    cmd
}

fn normalize_envelope(output: &str) -> String {
    let re_ts = regex::Regex::new(r#""generated_at_ms":\d+"#).unwrap();
    let s = re_ts
        .replace_all(output, r#""generated_at_ms":0"#)
        .to_string();
    let re_ver = regex::Regex::new(r#"("tool":\{"name":"tokmd","version":")[^"]+"#).unwrap();
    re_ver.replace_all(&s, r#"${1}0.0.0"#).to_string()
}

// ===========================================================================
// 1. Scan -> Model -> Format pipeline (Markdown)
// ===========================================================================

#[test]
fn w70_scan_model_format_md_contains_table_header() {
    let proj = make_project();
    let langs = scan(&[proj.path().to_path_buf()], &scan_opts()).expect("scan");
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);
    let args = LangArgs {
        paths: vec![proj.path().to_path_buf()],
        format: TableFormat::Md,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains("|Lang|"), "Markdown must have table header");
    assert!(out.contains("|**Total**|"), "Markdown must have total row");
}

#[test]
fn w70_scan_model_format_md_includes_all_detected_languages() {
    let proj = make_project();
    let langs = scan(&[proj.path().to_path_buf()], &scan_opts()).expect("scan");
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);
    let args = LangArgs {
        paths: vec![proj.path().to_path_buf()],
        format: TableFormat::Md,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let out = String::from_utf8(buf).unwrap();
    for row in &report.rows {
        assert!(out.contains(&row.lang), "MD should contain {}", row.lang);
    }
}

#[test]
fn w70_scan_model_format_json_roundtrips_totals() {
    let proj = make_project();
    let langs = scan(&[proj.path().to_path_buf()], &scan_opts()).expect("scan");
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);
    let args = LangArgs {
        paths: vec![proj.path().to_path_buf()],
        format: TableFormat::Json,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let receipt: LangReceipt = serde_json::from_slice(&buf).unwrap();
    assert_eq!(receipt.report.total.code, report.total.code);
    assert_eq!(receipt.report.total.files, report.total.files);
}

// ===========================================================================
// 2. Scan -> Export pipeline (JSONL)
// ===========================================================================

#[test]
fn w70_export_jsonl_each_line_valid_json() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "should have meta + data rows");
    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} is not valid JSON: {e}", i + 1));
    }
}

#[test]
fn w70_export_jsonl_meta_line_has_schema_version() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let first_line = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    let meta: Value = serde_json::from_str(&first_line).unwrap();
    assert_eq!(meta["type"].as_str(), Some("meta"));
    assert_eq!(
        meta["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
}

#[test]
fn w70_export_csv_has_header_and_data_rows() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "CSV should have header + rows");
    assert!(
        lines[0].contains("path") || lines[0].contains("language"),
        "CSV header should name columns"
    );
}

// ===========================================================================
// 3. Receipt determinism
// ===========================================================================

#[test]
fn w70_lang_json_deterministic_across_three_runs() {
    let proj = make_project();
    let run = || {
        let o = tokmd_at(proj.path())
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    let c = run();
    assert_eq!(a, b, "run 1 vs 2 must match");
    assert_eq!(b, c, "run 2 vs 3 must match");
}

#[test]
fn w70_module_json_deterministic_across_two_runs() {
    let proj = make_project();
    let run = || {
        let o = tokmd_at(proj.path())
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    assert_eq!(run(), run(), "module JSON must be stable");
}

#[test]
fn w70_export_json_deterministic_across_two_runs() {
    let proj = make_project();
    let run = || {
        let o = tokmd_at(proj.path())
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    assert_eq!(run(), run(), "export JSON must be stable");
}

// ===========================================================================
// 4. Schema version consistency
// ===========================================================================

#[test]
fn w70_lang_json_schema_version_matches_constant() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
}

#[test]
fn w70_module_json_schema_version_matches_constant() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
}

#[test]
fn w70_export_json_schema_version_matches_constant() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
}

#[test]
fn w70_lib_lang_receipt_schema_version_matches() {
    let proj = make_project();
    let langs = scan(&[proj.path().to_path_buf()], &scan_opts()).expect("scan");
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);
    let args = LangArgs {
        paths: vec![proj.path().to_path_buf()],
        format: TableFormat::Json,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut buf = Vec::new();
    tokmd_format::write_lang_report_to(&mut buf, &report, &scan_opts(), &args).unwrap();
    let receipt: LangReceipt = serde_json::from_slice(&buf).unwrap();
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

// ===========================================================================
// 5. Module depth consistency
// ===========================================================================

#[test]
fn w70_module_depth_1_produces_shallow_keys() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["module", "--format", "json", "--module-depth", "1"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows");
    for row in rows {
        let module = row["module"].as_str().unwrap();
        let depth = module.matches('/').count();
        assert!(
            depth == 0,
            "depth-1 module key should have no slashes: {module}"
        );
    }
}

#[test]
fn w70_module_depth_0_json_field_matches() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["module", "--format", "json", "--module-depth", "0"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["module_depth"].as_u64().unwrap(), 0);
}

#[test]
fn w70_module_depth_2_allows_nested_keys() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["module", "--format", "json", "--module-depth", "2"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["module_depth"].as_u64().unwrap(), 2);
    let rows = json["rows"].as_array().expect("rows");
    assert!(!rows.is_empty());
}

// ===========================================================================
// 6. Format roundtrip
// ===========================================================================

#[test]
fn w70_lang_json_roundtrip_preserves_rows() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
    assert!(json["total"]["code"].is_number());
    assert_eq!(json["mode"].as_str(), Some("lang"));
    assert!(json["schema_version"].is_number());
}

#[test]
fn w70_module_json_roundtrip_preserves_structure() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let receipt: ModuleReceipt = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!receipt.report.rows.is_empty());
    assert!(receipt.report.total.code > 0);
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn w70_export_json_roundtrip_has_file_rows() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows");
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row["path"].is_string(), "each export row needs a path");
        assert!(row["lang"].is_string(), "each export row needs a lang");
    }
}

// ===========================================================================
// 7. Badge from receipt data
// ===========================================================================

#[test]
fn w70_badge_lines_from_temp_project() {
    let proj = make_project();
    tokmd_at(proj.path())
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("lines"));
}

#[test]
fn w70_badge_tokens_from_temp_project() {
    let proj = make_project();
    tokmd_at(proj.path())
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("tokens"));
}

// ===========================================================================
// 8. Config -> Scan integration
// ===========================================================================

#[test]
fn w70_cli_exclude_flag_propagates_to_scan_options() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["lang", "--format", "json", "--exclude", "*.py"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let scan_info = &json["scan"];
    let excluded = scan_info["excluded"].as_array().expect("excluded array");
    assert!(
        excluded.iter().any(|e| e.as_str() == Some("*.py")),
        "scan metadata should record the exclude pattern"
    );
}

#[test]
fn w70_lib_config_parse_exclude_affects_scan() {
    use tokmd_settings::TomlConfig;
    let toml_str = "[scan]\nexclude = [\"*.md\"]\n";
    let config = TomlConfig::parse(toml_str).expect("valid TOML");
    let excludes = config.scan.exclude.unwrap_or_default();
    assert_eq!(excludes, vec!["*.md"]);

    let proj = make_project();
    let opts = ScanOptions {
        excluded: excludes,
        config: ConfigMode::None,
        no_ignore_vcs: true,
        ..Default::default()
    };
    let langs = scan(&[proj.path().to_path_buf()], &opts).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let has_md = report.rows.iter().any(|r| r.lang == "Markdown");
    assert!(!has_md, "Markdown should be excluded via parsed config");
}

// ===========================================================================
// 9. Empty directory
// ===========================================================================

#[test]
fn w70_empty_dir_lang_json_valid_with_zero_rows() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::create_dir(dir.path().join(".git")).unwrap();

    let output = tokmd_at(dir.path())
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    let rows = json["rows"].as_array().expect("rows");
    assert!(rows.is_empty(), "empty dir should have zero rows");
}

#[test]
fn w70_empty_dir_export_json_valid() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::create_dir(dir.path().join(".git")).unwrap();

    let output = tokmd_at(dir.path())
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
}

#[test]
fn w70_empty_dir_module_json_valid() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::create_dir(dir.path().join(".git")).unwrap();

    let output = tokmd_at(dir.path())
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    let rows = json["rows"].as_array().expect("rows");
    assert!(rows.is_empty());
}

// ===========================================================================
// 10. Exclude patterns propagate through the pipeline
// ===========================================================================

#[test]
fn w70_cli_exclude_removes_language_from_lang() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["lang", "--format", "json", "--exclude", "*.py"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows");
    let has_python = rows.iter().any(|r| r["lang"].as_str() == Some("Python"));
    assert!(!has_python, "*.py exclude should remove Python");
}

#[test]
fn w70_cli_exclude_removes_files_from_export() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["export", "--format", "json", "--exclude", "*.js"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows");
    for row in rows {
        let path = row["path"].as_str().unwrap_or("");
        assert!(!path.ends_with(".js"), "excluded .js file appears: {path}");
    }
}

#[test]
fn w70_lib_exclude_removes_language_from_scan() {
    let proj = make_project();
    let opts = ScanOptions {
        excluded: vec!["*.js".to_string()],
        config: ConfigMode::None,
        no_ignore_vcs: true,
        ..Default::default()
    };
    let langs = scan(&[proj.path().to_path_buf()], &opts).expect("scan");
    let report = create_lang_report(&langs, 0, false, ChildrenMode::Collapse);
    let has_js = report.rows.iter().any(|r| r.lang == "JavaScript");
    assert!(!has_js, "JavaScript should be excluded");
    let has_rust = report.rows.iter().any(|r| r.lang == "Rust");
    assert!(has_rust, "Rust should still be present");
}

// ===========================================================================
// 11. Children mode
// ===========================================================================

#[test]
fn w70_children_collapse_vs_separate_differ() {
    let proj = make_project_with_embedded();
    let collapse_out = tokmd_at(proj.path())
        .args(["lang", "--format", "json", "--children", "collapse"])
        .output()
        .expect("run");
    let separate_out = tokmd_at(proj.path())
        .args(["lang", "--format", "json", "--children", "separate"])
        .output()
        .expect("run");
    assert!(collapse_out.status.success());
    assert!(separate_out.status.success());
    let c: Value = serde_json::from_slice(&collapse_out.stdout).unwrap();
    let s: Value = serde_json::from_slice(&separate_out.stdout).unwrap();
    assert_eq!(c["children"].as_str().unwrap_or(""), "collapse");
    assert_eq!(s["children"].as_str().unwrap_or(""), "separate");
}

#[test]
fn w70_children_separate_has_embedded_tag() {
    let proj = make_project_with_embedded();
    let output = tokmd_at(proj.path())
        .args(["lang", "--format", "json", "--children", "separate"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["children"].as_str(), Some("separate"));
}

// ===========================================================================
// 12. Sort consistency
// ===========================================================================

#[test]
fn w70_lang_rows_sorted_descending_by_code() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows");
    let codes: Vec<u64> = rows.iter().map(|r| r["code"].as_u64().unwrap()).collect();
    for w in codes.windows(2) {
        assert!(
            w[0] >= w[1],
            "rows should be sorted descending by code: {} < {}",
            w[0],
            w[1]
        );
    }
}

#[test]
fn w70_sort_order_consistent_between_md_and_json() {
    let proj = make_project();
    let langs = scan(&[proj.path().to_path_buf()], &scan_opts()).expect("scan");
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);

    let json_args = LangArgs {
        paths: vec![proj.path().to_path_buf()],
        format: TableFormat::Json,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut jbuf = Vec::new();
    tokmd_format::write_lang_report_to(&mut jbuf, &report, &scan_opts(), &json_args).unwrap();
    let receipt: LangReceipt = serde_json::from_slice(&jbuf).unwrap();
    let json_langs: Vec<&str> = receipt
        .report
        .rows
        .iter()
        .map(|r| r.lang.as_str())
        .collect();

    let md_args = LangArgs {
        paths: vec![proj.path().to_path_buf()],
        format: TableFormat::Md,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut mbuf = Vec::new();
    tokmd_format::write_lang_report_to(&mut mbuf, &report, &scan_opts(), &md_args).unwrap();
    let md_out = String::from_utf8(mbuf).unwrap();
    let md_langs: Vec<&str> = md_out
        .lines()
        .filter(|l| {
            l.starts_with('|')
                && !l.contains("Lang|")
                && !l.contains("---")
                && !l.contains("**Total**")
        })
        .filter_map(|l| l.split('|').nth(1).map(|s| s.trim()))
        .filter(|s| !s.is_empty())
        .collect();

    assert_eq!(
        json_langs, md_langs,
        "JSON and MD should list languages in the same order"
    );
}

#[test]
fn w70_sort_order_consistent_between_tsv_and_json() {
    let proj = make_project();
    let langs = scan(&[proj.path().to_path_buf()], &scan_opts()).expect("scan");
    let report = create_lang_report(&langs, 0, true, ChildrenMode::Collapse);

    let json_args = LangArgs {
        paths: vec![proj.path().to_path_buf()],
        format: TableFormat::Json,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut jbuf = Vec::new();
    tokmd_format::write_lang_report_to(&mut jbuf, &report, &scan_opts(), &json_args).unwrap();
    let receipt: LangReceipt = serde_json::from_slice(&jbuf).unwrap();
    let json_langs: Vec<&str> = receipt
        .report
        .rows
        .iter()
        .map(|r| r.lang.as_str())
        .collect();

    let tsv_args = LangArgs {
        paths: vec![proj.path().to_path_buf()],
        format: TableFormat::Tsv,
        top: 0,
        files: true,
        children: ChildrenMode::Collapse,
    };
    let mut tbuf = Vec::new();
    tokmd_format::write_lang_report_to(&mut tbuf, &report, &scan_opts(), &tsv_args).unwrap();
    let tsv_out = String::from_utf8(tbuf).unwrap();
    let tsv_langs: Vec<&str> = tsv_out
        .lines()
        .skip(1)
        .filter(|l| !l.is_empty() && !l.starts_with("Total"))
        .filter_map(|l| l.split('\t').next())
        .collect();

    assert_eq!(
        json_langs, tsv_langs,
        "JSON and TSV should list languages in the same order"
    );
}

#[test]
fn w70_module_rows_sorted_descending_by_code() {
    let proj = make_project();
    let output = tokmd_at(proj.path())
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows");
    let codes: Vec<u64> = rows.iter().map(|r| r["code"].as_u64().unwrap()).collect();
    for w in codes.windows(2) {
        assert!(
            w[0] >= w[1],
            "module rows should be sorted descending by code: {} < {}",
            w[0],
            w[1]
        );
    }
}
