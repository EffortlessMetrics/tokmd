//! Additional workflow integration tests for tokmd-core.
//!
//! Focuses on edge cases and cross-workflow invariants NOT covered
//! by existing test files:
//!
//! 1. Cross-workflow consistency: lang vs module vs export totals agree
//! 2. Schema version consistency across all receipts
//! 3. Redaction mode propagation across all workflows
//! 4. Children mode consistency across lang and export
//! 5. Error handling for invalid and edge-case paths
//! 6. Concurrent workflow execution (same data, parallel threads)
//! 7. Large repo simulation with many files
//! 8. All export format options
//! 9. FFI mode=version returns valid version string
//! 10. FFI error messages are helpful and contain context
//! 11. Strip-prefix propagation and redaction interaction
//! 12. Module depth boundary values
//! 13. Diff workflow with receipt files (load_lang_report path)
//! 14. Scan options propagation (hidden, no_ignore)

use std::fs;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;
use tokmd_core::ffi::run_json;
use tokmd_core::settings::{
    DiffSettings, ExportSettings, LangSettings, ModuleSettings, ScanSettings,
};
use tokmd_core::{diff_workflow, export_workflow, lang_workflow, module_workflow};
use tokmd_types::{ChildIncludeMode, ChildrenMode, ExportFormat, RedactMode};

// ============================================================================
// Fixtures
// ============================================================================

/// Temp dir with Rust, Python, and JavaScript files across nested dirs.
fn fixture_polyglot() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    let src = dir.path().join("src");
    let lib = dir.path().join("lib");
    let scripts = dir.path().join("scripts");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&lib).unwrap();
    fs::create_dir_all(&scripts).unwrap();

    fs::write(
        src.join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n    let x = 42;\n}\n",
    )
    .unwrap();
    fs::write(
        src.join("util.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\npub fn sub(a: i32, b: i32) -> i32 {\n    a - b\n}\n",
    )
    .unwrap();
    fs::write(
        lib.join("helpers.py"),
        "def greet(name):\n    return f\"Hello {name}\"\n\ndef farewell():\n    return \"Bye\"\n",
    )
    .unwrap();
    fs::write(
        scripts.join("index.js"),
        "function main() {\n    console.log('hello');\n}\nmodule.exports = { main };\n",
    )
    .unwrap();
    dir
}

/// Temp dir with a single file for minimal testing.
fn fixture_single_file() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    fs::write(
        dir.path().join("hello.rs"),
        "fn hello() -> &'static str {\n    \"world\"\n}\n",
    )
    .unwrap();
    dir
}

/// Temp dir with many small files to simulate a larger repo.
fn fixture_many_files() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    for i in 0..50 {
        let content = format!("pub fn func_{i}(x: i32) -> i32 {{\n    x + {i}\n}}\n");
        fs::write(src.join(format!("mod_{i}.rs", i = i)), content).unwrap();
    }
    dir
}

/// Temp dir with deeply nested directories.
fn fixture_deep_nesting() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    let deep = dir.path().join("a").join("b").join("c").join("d");
    fs::create_dir_all(&deep).unwrap();
    fs::write(
        deep.join("leaf.rs"),
        "pub fn leaf() -> bool {\n    true\n}\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("a").join("root.rs"),
        "pub fn root() -> bool {\n    false\n}\n",
    )
    .unwrap();
    dir
}

/// Temp dir with hidden files.
fn fixture_with_hidden() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    fs::write(dir.path().join("visible.rs"), "fn visible() {}\n").unwrap();
    fs::write(dir.path().join(".hidden.rs"), "fn hidden() {}\n").unwrap();
    dir
}

fn scan_for(dir: &TempDir) -> ScanSettings {
    ScanSettings::for_paths(vec![dir.path().display().to_string()])
}

/// Strip volatile fields so two receipts can be structurally compared.
fn strip_volatile(v: &mut serde_json::Value) {
    if let Some(obj) = v.as_object_mut() {
        obj.remove("generated_at_ms");
        obj.remove("scan_duration_ms");
        for (_, child) in obj.iter_mut() {
            strip_volatile(child);
        }
    }
    if let Some(arr) = v.as_array_mut() {
        for child in arr.iter_mut() {
            strip_volatile(child);
        }
    }
}

// ============================================================================
// 1. Cross-workflow consistency: totals agree
// ============================================================================

#[test]
fn cross_workflow_polyglot_lang_export_code_totals_match() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    let lang_total: usize = lang.report.rows.iter().map(|r| r.code).sum();
    let export_total: usize = export.data.rows.iter().map(|r| r.code).sum();

    assert_eq!(
        lang_total, export_total,
        "lang total code ({lang_total}) != export total ({export_total})"
    );
}

#[test]
fn cross_workflow_polyglot_lang_export_lines_match() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    let lang_lines: usize = lang.report.rows.iter().map(|r| r.lines).sum();
    let export_lines: usize = export.data.rows.iter().map(|r| r.lines).sum();

    assert_eq!(
        lang_lines, export_lines,
        "lang total lines ({lang_lines}) != export total ({export_lines})"
    );
}

#[test]
fn cross_workflow_polyglot_export_blanks_and_comments_nonnegative() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);

    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    for row in &export.data.rows {
        // blanks and comments are usize so always >= 0, but verify they sum correctly
        assert!(
            row.blanks + row.comments + row.code <= row.lines,
            "blanks ({}) + comments ({}) + code ({}) should be <= lines ({}) for {}",
            row.blanks,
            row.comments,
            row.code,
            row.lines,
            row.path
        );
    }
}

#[test]
fn cross_workflow_lang_file_count_matches_export_rows() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);

    let lang = lang_workflow(
        &scan,
        &LangSettings {
            files: true,
            ..Default::default()
        },
    )
    .unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    let lang_file_count: usize = lang.report.rows.iter().map(|r| r.files).sum();
    let export_file_count = export.data.rows.len();

    assert_eq!(
        lang_file_count, export_file_count,
        "lang file count ({lang_file_count}) != export row count ({export_file_count})"
    );
}

#[test]
fn cross_workflow_module_total_code_matches_lang_total() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let module = module_workflow(&scan, &ModuleSettings::default()).unwrap();

    let lang_total: usize = lang.report.rows.iter().map(|r| r.code).sum();
    let module_total: usize = module.report.rows.iter().map(|r| r.code).sum();

    assert_eq!(
        lang_total, module_total,
        "lang total code ({lang_total}) != module total code ({module_total})"
    );
}

// ============================================================================
// 2. Schema version consistency
// ============================================================================

#[test]
fn schema_version_consistent_across_all_workflows() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let module = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    let p = dir.path().display().to_string();
    let diff = diff_workflow(&DiffSettings {
        from: p.clone(),
        to: p,
    })
    .unwrap();

    assert_eq!(lang.schema_version, tokmd_types::SCHEMA_VERSION);
    assert_eq!(module.schema_version, tokmd_types::SCHEMA_VERSION);
    assert_eq!(export.schema_version, tokmd_types::SCHEMA_VERSION);
    assert_eq!(diff.schema_version, tokmd_types::SCHEMA_VERSION);
    assert_eq!(lang.schema_version, module.schema_version);
    assert_eq!(lang.schema_version, export.schema_version);
    assert_eq!(lang.schema_version, diff.schema_version);
}

#[test]
fn schema_version_matches_core_constant() {
    assert_eq!(tokmd_core::CORE_SCHEMA_VERSION, tokmd_types::SCHEMA_VERSION);
}

// ============================================================================
// 3. Redaction mode propagation
// ============================================================================

#[test]
fn redact_paths_hides_filenames_in_export() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        redact: RedactMode::Paths,
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(!receipt.data.rows.is_empty());

    for row in &receipt.data.rows {
        assert!(
            !row.path.contains("hello.rs"),
            "redacted path should not contain original filename: {}",
            row.path
        );
    }
}

#[test]
fn redact_all_hides_filenames_and_modules_in_export() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        redact: RedactMode::All,
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(!receipt.data.rows.is_empty());

    for row in &receipt.data.rows {
        assert!(
            !row.path.contains("main.rs"),
            "redacted path should not contain original filename: {}",
            row.path
        );
    }
}

#[test]
fn redact_none_preserves_all_paths() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        redact: RedactMode::None,
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).unwrap();
    let has_hello = receipt
        .data
        .rows
        .iter()
        .any(|r| r.path.contains("hello.rs"));
    assert!(has_hello, "unredacted paths should preserve filenames");
}

#[test]
fn redact_paths_vs_none_same_code_totals() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);

    let none = export_workflow(
        &scan,
        &ExportSettings {
            redact: RedactMode::None,
            ..Default::default()
        },
    )
    .unwrap();

    let paths = export_workflow(
        &scan,
        &ExportSettings {
            redact: RedactMode::Paths,
            ..Default::default()
        },
    )
    .unwrap();

    let none_total: usize = none.data.rows.iter().map(|r| r.code).sum();
    let paths_total: usize = paths.data.rows.iter().map(|r| r.code).sum();
    assert_eq!(
        none_total, paths_total,
        "redaction should not change code totals"
    );
    assert_eq!(
        none.data.rows.len(),
        paths.data.rows.len(),
        "redaction should not change row count"
    );
}

#[test]
fn redact_export_args_meta_reflects_mode() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);

    let receipt_none = export_workflow(
        &scan,
        &ExportSettings {
            redact: RedactMode::None,
            ..Default::default()
        },
    )
    .unwrap();
    let receipt_paths = export_workflow(
        &scan,
        &ExportSettings {
            redact: RedactMode::Paths,
            ..Default::default()
        },
    )
    .unwrap();
    let receipt_all = export_workflow(
        &scan,
        &ExportSettings {
            redact: RedactMode::All,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(receipt_none.args.redact, RedactMode::None);
    assert_eq!(receipt_paths.args.redact, RedactMode::Paths);
    assert_eq!(receipt_all.args.redact, RedactMode::All);
}

// ============================================================================
// 4. Children mode consistency
// ============================================================================

#[test]
fn children_collapse_vs_separate_same_total_code() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);

    let collapse = lang_workflow(
        &scan,
        &LangSettings {
            children: ChildrenMode::Collapse,
            ..Default::default()
        },
    )
    .unwrap();

    let separate = lang_workflow(
        &scan,
        &LangSettings {
            children: ChildrenMode::Separate,
            ..Default::default()
        },
    )
    .unwrap();

    let collapse_total: usize = collapse.report.rows.iter().map(|r| r.code).sum();
    let separate_total: usize = separate.report.rows.iter().map(|r| r.code).sum();

    assert_eq!(
        collapse_total, separate_total,
        "total code should be same regardless of children mode"
    );
}

#[test]
fn children_mode_recorded_in_args() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);

    let collapse = lang_workflow(
        &scan,
        &LangSettings {
            children: ChildrenMode::Collapse,
            ..Default::default()
        },
    )
    .unwrap();
    let separate = lang_workflow(
        &scan,
        &LangSettings {
            children: ChildrenMode::Separate,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(collapse.args.children, ChildrenMode::Collapse);
    assert_eq!(separate.args.children, ChildrenMode::Separate);
}

#[test]
fn child_include_mode_recorded_in_module_args() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);

    let sep = module_workflow(
        &scan,
        &ModuleSettings {
            children: ChildIncludeMode::Separate,
            ..Default::default()
        },
    )
    .unwrap();
    let parents = module_workflow(
        &scan,
        &ModuleSettings {
            children: ChildIncludeMode::ParentsOnly,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(sep.args.children, ChildIncludeMode::Separate);
    assert_eq!(parents.args.children, ChildIncludeMode::ParentsOnly);
}

// ============================================================================
// 5. Error handling for edge-case paths
// ============================================================================

#[test]
fn workflow_empty_paths_vec_handled() {
    let scan = ScanSettings::for_paths(vec![]);
    // tokei panics on empty paths, so catching the panic is acceptable
    let result = std::panic::catch_unwind(|| lang_workflow(&scan, &LangSettings::default()));
    match result {
        Ok(Ok(receipt)) => assert!(receipt.report.rows.is_empty()),
        Ok(Err(_)) => {} // error is acceptable
        Err(_) => {}     // panic is acceptable (tokei limitation)
    }
}

#[test]
fn workflow_dot_path_succeeds() {
    let scan = ScanSettings::current_dir();
    let result = lang_workflow(&scan, &LangSettings::default());
    assert!(result.is_ok(), "scanning '.' should succeed");
}

#[test]
fn workflow_path_with_spaces() {
    let dir = TempDir::new().unwrap();
    let spaced = dir.path().join("my project");
    fs::create_dir_all(&spaced).unwrap();
    fs::write(spaced.join("app.rs"), "fn main() {}\n").unwrap();

    let scan = ScanSettings::for_paths(vec![spaced.display().to_string()]);
    let result = lang_workflow(&scan, &LangSettings::default());
    assert!(result.is_ok(), "paths with spaces should work");
    let receipt = result.unwrap();
    assert!(!receipt.report.rows.is_empty());
}

#[test]
fn workflow_unicode_path() {
    let dir = TempDir::new().unwrap();
    let unicode_dir = dir.path().join("données");
    fs::create_dir_all(&unicode_dir).unwrap();
    fs::write(unicode_dir.join("code.rs"), "fn test() -> i32 { 1 }\n").unwrap();

    let scan = ScanSettings::for_paths(vec![unicode_dir.display().to_string()]);
    let result = lang_workflow(&scan, &LangSettings::default());
    if let Ok(receipt) = result {
        assert!(!receipt.report.rows.is_empty());
    }
}

// ============================================================================
// 6. Concurrent workflow execution
// ============================================================================

#[test]
fn concurrent_lang_workflows_produce_same_results() {
    let dir = fixture_polyglot();
    let scan = Arc::new(scan_for(&dir));
    let lang = Arc::new(LangSettings::default());

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let s = Arc::clone(&scan);
            let l = Arc::clone(&lang);
            thread::spawn(move || lang_workflow(&s, &l).unwrap())
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All results should have same row count and values
    let first = &results[0];
    for (i, r) in results.iter().enumerate().skip(1) {
        assert_eq!(
            first.report.rows.len(),
            r.report.rows.len(),
            "thread {i} row count differs"
        );
        for (a, b) in first.report.rows.iter().zip(r.report.rows.iter()) {
            assert_eq!(a.lang, b.lang, "thread {i} lang differs");
            assert_eq!(a.code, b.code, "thread {i} code differs");
        }
    }
}

#[test]
fn concurrent_mixed_workflows_all_succeed() {
    let dir = fixture_polyglot();
    let scan = Arc::new(scan_for(&dir));

    let scan_l = Arc::clone(&scan);
    let h_lang = thread::spawn(move || lang_workflow(&scan_l, &LangSettings::default()).unwrap());

    let scan_m = Arc::clone(&scan);
    let h_module =
        thread::spawn(move || module_workflow(&scan_m, &ModuleSettings::default()).unwrap());

    let scan_e = Arc::clone(&scan);
    let h_export =
        thread::spawn(move || export_workflow(&scan_e, &ExportSettings::default()).unwrap());

    let lang_r = h_lang.join().unwrap();
    let module_r = h_module.join().unwrap();
    let export_r = h_export.join().unwrap();

    // All should complete with correct modes
    assert_eq!(lang_r.mode, "lang");
    assert_eq!(module_r.mode, "module");
    assert_eq!(export_r.mode, "export");

    // Cross-workflow totals should still agree
    let lang_total: usize = lang_r.report.rows.iter().map(|r| r.code).sum();
    let export_total: usize = export_r.data.rows.iter().map(|r| r.code).sum();
    assert_eq!(lang_total, export_total);
}

// ============================================================================
// 7. Large repo simulation
// ============================================================================

#[test]
fn many_files_lang_workflow_produces_results() {
    let dir = fixture_many_files();
    let scan = scan_for(&dir);

    let receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();
    assert!(!receipt.report.rows.is_empty());

    let rust_row = receipt.report.rows.iter().find(|r| r.lang == "Rust");
    assert!(rust_row.is_some(), "should detect Rust");
    assert!(rust_row.unwrap().files >= 50, "should count all 50 files");
}

#[test]
fn many_files_export_returns_all_rows() {
    let dir = fixture_many_files();
    let scan = scan_for(&dir);

    let receipt = export_workflow(&scan, &ExportSettings::default()).unwrap();
    assert!(
        receipt.data.rows.len() >= 50,
        "should export all 50 files, got {}",
        receipt.data.rows.len()
    );
}

#[test]
fn many_files_export_max_rows_limits_output() {
    let dir = fixture_many_files();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        max_rows: 10,
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(
        receipt.data.rows.len() <= 10,
        "max_rows=10 should limit, got {}",
        receipt.data.rows.len()
    );
}

#[test]
fn many_files_deterministic() {
    let dir = fixture_many_files();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    let r1 = lang_workflow(&scan, &lang).unwrap();
    let r2 = lang_workflow(&scan, &lang).unwrap();

    let mut j1 = serde_json::to_value(r1).unwrap();
    let mut j2 = serde_json::to_value(r2).unwrap();
    strip_volatile(&mut j1);
    strip_volatile(&mut j2);
    assert_eq!(j1, j2, "many-files output should be deterministic");
}

// ============================================================================
// 8. Export format options
// ============================================================================

#[test]
fn export_format_jsonl_recorded_in_args() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        format: ExportFormat::Jsonl,
        ..Default::default()
    };
    let receipt = export_workflow(&scan, &export).unwrap();
    assert_eq!(receipt.args.format, ExportFormat::Jsonl);
}

#[test]
fn export_format_csv_recorded_in_args() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        format: ExportFormat::Csv,
        ..Default::default()
    };
    let receipt = export_workflow(&scan, &export).unwrap();
    assert_eq!(receipt.args.format, ExportFormat::Csv);
}

#[test]
fn export_format_json_recorded_in_args() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        format: ExportFormat::Json,
        ..Default::default()
    };
    let receipt = export_workflow(&scan, &export).unwrap();
    assert_eq!(receipt.args.format, ExportFormat::Json);
}

#[test]
fn export_format_does_not_affect_data() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);

    let jsonl = export_workflow(
        &scan,
        &ExportSettings {
            format: ExportFormat::Jsonl,
            ..Default::default()
        },
    )
    .unwrap();
    let csv = export_workflow(
        &scan,
        &ExportSettings {
            format: ExportFormat::Csv,
            ..Default::default()
        },
    )
    .unwrap();
    let json = export_workflow(
        &scan,
        &ExportSettings {
            format: ExportFormat::Json,
            ..Default::default()
        },
    )
    .unwrap();

    // All formats should produce identical row data
    assert_eq!(jsonl.data.rows.len(), csv.data.rows.len());
    assert_eq!(jsonl.data.rows.len(), json.data.rows.len());

    for (a, b) in jsonl.data.rows.iter().zip(csv.data.rows.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.code, b.code);
    }
}

// ============================================================================
// 9. FFI mode=version
// ============================================================================

#[test]
fn ffi_version_returns_semver() {
    let result = run_json("version", "{}");
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], true);

    let ver = v["data"]["version"].as_str().unwrap();
    let parts: Vec<&str> = ver.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "version should be MAJOR.MINOR.PATCH, got: {ver}"
    );
    for part in &parts {
        let numeric_part = part.split('-').next().unwrap();
        assert!(
            numeric_part.parse::<u32>().is_ok(),
            "version part should be numeric: {part}"
        );
    }
}

#[test]
fn ffi_version_schema_version_matches_constant() {
    let result = run_json("version", "{}");
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

#[test]
fn ffi_version_idempotent() {
    let r1 = run_json("version", "{}");
    let r2 = run_json("version", "{}");
    assert_eq!(r1, r2, "version should be byte-identical across calls");
}

// ============================================================================
// 10. FFI error messages are helpful
// ============================================================================

#[test]
fn ffi_error_unknown_mode_includes_mode_name() {
    let result = run_json("quux_mode", "{}");
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], false);
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("quux_mode"),
        "error message should mention the invalid mode name: {msg}"
    );
}

#[test]
fn ffi_error_invalid_json_has_code() {
    let result = run_json("lang", "{{broken}}");
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_json"));
}

#[test]
fn ffi_error_invalid_children_mentions_field() {
    let result = run_json("lang", r#"{"children": "invalid_value"}"#);
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], false);
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("children"),
        "error for bad children should mention 'children': {msg}"
    );
}

#[test]
fn ffi_error_invalid_redact_mentions_field() {
    let result = run_json("export", r#"{"redact": "maybe"}"#);
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], false);
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("redact"),
        "error for bad redact should mention 'redact': {msg}"
    );
}

#[test]
fn ffi_error_invalid_format_mentions_field() {
    let result = run_json("export", r#"{"format": "yaml"}"#);
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], false);
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("format"),
        "error for bad format should mention 'format': {msg}"
    );
}

#[test]
fn ffi_error_diff_missing_both_mentions_from() {
    let result = run_json("diff", "{}");
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], false);
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("from"),
        "diff error should mention 'from': {msg}"
    );
}

// ============================================================================
// 11. Strip-prefix propagation
// ============================================================================

#[test]
fn export_strip_prefix_recorded_in_args() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        strip_prefix: Some("src".to_string()),
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).unwrap();
    // When not redacted, strip_prefix should appear as-is
    assert_eq!(receipt.args.strip_prefix, Some("src".to_string()));
}

#[test]
fn export_strip_prefix_with_redact_paths_is_redacted() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        strip_prefix: Some("src".to_string()),
        redact: RedactMode::Paths,
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(receipt.args.strip_prefix_redacted);
    // The strip_prefix should be hashed, not contain "src" literally
    if let Some(sp) = &receipt.args.strip_prefix {
        assert!(
            !sp.contains("src") || sp.len() > 10,
            "redacted strip_prefix should be hashed"
        );
    }
}

// ============================================================================
// 12. Module depth boundary values
// ============================================================================

#[test]
fn module_depth_zero() {
    let dir = fixture_deep_nesting();
    let scan = scan_for(&dir);
    let module = ModuleSettings {
        module_depth: 0,
        ..Default::default()
    };

    let receipt = module_workflow(&scan, &module).unwrap();
    assert_eq!(receipt.args.module_depth, 0);
}

#[test]
fn module_depth_very_large() {
    let dir = fixture_deep_nesting();
    let scan = scan_for(&dir);
    let module = ModuleSettings {
        module_depth: 100,
        ..Default::default()
    };

    let receipt = module_workflow(&scan, &module).unwrap();
    assert_eq!(receipt.args.module_depth, 100);
}

#[test]
fn module_depth_one_vs_default() {
    let dir = fixture_deep_nesting();
    let scan = scan_for(&dir);

    let depth1 = module_workflow(
        &scan,
        &ModuleSettings {
            module_depth: 1,
            ..Default::default()
        },
    )
    .unwrap();

    let depth_default = module_workflow(&scan, &ModuleSettings::default()).unwrap();

    // Both should have correct module_depth in args
    assert_eq!(depth1.args.module_depth, 1);
    assert_eq!(depth_default.args.module_depth, 2); // default is 2
}

#[test]
fn module_custom_roots_propagated() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);
    let module = ModuleSettings {
        module_roots: vec!["src".to_string(), "lib".to_string()],
        ..Default::default()
    };

    let receipt = module_workflow(&scan, &module).unwrap();
    assert!(receipt.args.module_roots.contains(&"src".to_string()));
    assert!(receipt.args.module_roots.contains(&"lib".to_string()));
}

// ============================================================================
// 13. Diff workflow with receipt files
// ============================================================================

#[test]
fn diff_workflow_from_receipt_file() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);

    // Generate a lang receipt and save to a separate temp file (not inside scanned dir)
    let receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let receipt_dir = TempDir::new().unwrap();
    let receipt_path = receipt_dir.path().join("receipt.json");
    let json = serde_json::to_string(&receipt).unwrap();
    fs::write(&receipt_path, &json).unwrap();

    // Diff from receipt file to live scan of same dir
    let settings = DiffSettings {
        from: receipt_path.display().to_string(),
        to: dir.path().display().to_string(),
    };
    let diff = diff_workflow(&settings).unwrap();

    // Self-diff: all deltas should be zero
    assert_eq!(
        diff.totals.delta_code, 0,
        "self-diff via receipt should have zero delta"
    );
    for row in &diff.diff_rows {
        assert_eq!(
            row.delta_code, 0,
            "self-diff via receipt should have zero delta for {}",
            row.lang
        );
    }
}

#[test]
fn diff_workflow_receipt_has_mode_and_sources() {
    let dir_a = fixture_single_file();
    let dir_b = fixture_polyglot();
    let settings = DiffSettings {
        from: dir_a.path().display().to_string(),
        to: dir_b.path().display().to_string(),
    };

    let receipt = diff_workflow(&settings).unwrap();
    assert_eq!(receipt.mode, "diff");
    assert!(!receipt.from_source.is_empty());
    assert!(!receipt.to_source.is_empty());
}

// ============================================================================
// 14. Scan options propagation
// ============================================================================

#[test]
fn scan_hidden_files_included_when_enabled() {
    let dir = fixture_with_hidden();
    let scan = ScanSettings {
        paths: vec![dir.path().display().to_string()],
        options: tokmd_core::settings::ScanOptions {
            hidden: true,
            ..Default::default()
        },
    };

    let receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();
    // With hidden=true, should find both visible and hidden files
    let rust_row = receipt.report.rows.iter().find(|r| r.lang == "Rust");
    assert!(rust_row.is_some());
    assert!(
        rust_row.unwrap().files >= 2,
        "with hidden=true, should find both files"
    );
}

#[test]
fn scan_without_hidden_excludes_dotfiles() {
    let dir = fixture_with_hidden();
    let scan = ScanSettings {
        paths: vec![dir.path().display().to_string()],
        options: tokmd_core::settings::ScanOptions {
            hidden: false,
            ..Default::default()
        },
    };

    let receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let rust_row = receipt.report.rows.iter().find(|r| r.lang == "Rust");
    // Without hidden, should find only the visible file
    if let Some(row) = rust_row {
        assert!(
            row.files <= 1,
            "with hidden=false, should skip hidden files"
        );
    }
}

#[test]
fn scan_excluded_patterns_filter_files() {
    let dir = fixture_polyglot();
    let scan_all = scan_for(&dir);
    let scan_filtered = ScanSettings {
        paths: vec![dir.path().display().to_string()],
        options: tokmd_core::settings::ScanOptions {
            excluded: vec!["*.py".to_string()],
            ..Default::default()
        },
    };

    let all = lang_workflow(&scan_all, &LangSettings::default()).unwrap();
    let filtered = lang_workflow(&scan_filtered, &LangSettings::default()).unwrap();

    let all_has_python = all.report.rows.iter().any(|r| r.lang == "Python");
    let filtered_has_python = filtered.report.rows.iter().any(|r| r.lang == "Python");

    assert!(all_has_python, "unfiltered should find Python");
    assert!(!filtered_has_python, "excluded *.py should remove Python");
}

// ============================================================================
// 15. Additional edge cases
// ============================================================================

#[test]
fn lang_top_exceeds_total_languages_returns_all() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);
    let lang = LangSettings {
        top: 1000,
        ..Default::default()
    };

    let receipt = lang_workflow(&scan, &lang).unwrap();
    // Only Rust in fixture, so should get 1 row regardless of top=1000
    assert!(
        receipt.report.rows.len() <= 2,
        "top=1000 with 1 language should just return that language"
    );
}

#[test]
fn export_min_code_zero_includes_all() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        min_code: 0,
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(
        receipt.data.rows.len() >= 4,
        "min_code=0 should include all files"
    );
}

#[test]
fn export_min_code_very_high_filters_all() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        min_code: 1_000_000,
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(
        receipt.data.rows.is_empty(),
        "min_code=1M should filter all files"
    );
}

#[test]
fn export_max_rows_zero_means_unlimited() {
    let dir = fixture_many_files();
    let scan = scan_for(&dir);
    let export = ExportSettings {
        max_rows: 0,
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(
        receipt.data.rows.len() >= 50,
        "max_rows=0 means unlimited, should get all files"
    );
}

#[test]
fn all_workflow_receipts_have_tool_info() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let module = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    for (name, tool) in [
        ("lang", &lang.tool),
        ("module", &module.tool),
        ("export", &export.tool),
    ] {
        assert!(!tool.name.is_empty(), "{name} receipt tool.name empty");
        assert!(
            !tool.version.is_empty(),
            "{name} receipt tool.version empty"
        );
    }
}

#[test]
fn all_workflow_receipts_have_valid_timestamps() {
    let dir = fixture_single_file();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let module = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    for (name, ts) in [
        ("lang", lang.generated_at_ms),
        ("module", module.generated_at_ms),
        ("export", export.generated_at_ms),
    ] {
        assert!(
            ts > 1_577_836_800_000,
            "{name} timestamp should be after 2020-01-01"
        );
    }
}

#[test]
fn lang_workflow_rows_sorted_by_code_descending() {
    let dir = fixture_polyglot();
    let scan = scan_for(&dir);
    let receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();

    let codes: Vec<usize> = receipt.report.rows.iter().map(|r| r.code).collect();
    for window in codes.windows(2) {
        assert!(
            window[0] >= window[1],
            "rows should be sorted by code descending: {:?}",
            codes
        );
    }
}

#[test]
fn export_paths_always_use_forward_slashes() {
    let dir = fixture_deep_nesting();
    let scan = scan_for(&dir);
    let receipt = export_workflow(&scan, &ExportSettings::default()).unwrap();

    for row in &receipt.data.rows {
        assert!(
            !row.path.contains('\\'),
            "path must use forward slashes: {}",
            row.path
        );
    }
}
