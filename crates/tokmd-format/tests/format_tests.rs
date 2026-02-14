//! Integration tests to kill surviving mutants in tokmd-format
//!
//! These tests verify:
//! - The `scan_args` function correctly handles redaction modes
//! - All formatting functions produce non-trivial output
//! - Edge cases in conditional logic
//! - I/O wrappers (print_lang_report, print_module_report, write_export) produce output

use std::path::PathBuf;

use tokmd_format::{
    compute_diff_rows, compute_diff_totals, normalize_scan_input, render_diff_md, scan_args,
};
use tokmd_settings::{ChildIncludeMode, ChildrenMode, ScanOptions};
use tokmd_types::{
    ConfigMode, ExportArgs, ExportArgsMeta, ExportData, ExportFormat, FileKind, FileRow,
    LangArgsMeta, LangReport, LangRow, ModuleArgsMeta, ModuleReport, ModuleRow, RedactMode,
    ScanArgs, Totals,
};

// ============================================================================
// scan_args tests - Kill mutants on lines 69-70 and related operators
// ============================================================================

/// Test scan_args with RedactMode::Paths - should redact paths
#[test]
fn test_scan_args_redact_paths_mode() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let global = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };

    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    // With Paths mode, paths should be redacted (hashed)
    assert!(!args.paths.is_empty());
    // Path should be a hash (16 chars) + extension
    assert!(args.paths[0].ends_with(".rs"));
    assert_ne!(args.paths[0], "src/lib.rs");

    // Excluded should also be redacted
    assert!(!args.excluded.is_empty());
    assert_ne!(args.excluded[0], "target");

    // excluded_redacted should be true
    assert!(args.excluded_redacted);
}

/// Test scan_args with RedactMode::All - should redact all
#[test]
fn test_scan_args_redact_all_mode() {
    let paths = vec![PathBuf::from("src/main.rs")];
    let global = ScanOptions {
        excluded: vec!["node_modules".to_string()],
        ..Default::default()
    };

    let args = scan_args(&paths, &global, Some(RedactMode::All));

    // With All mode, paths should be redacted
    assert!(args.paths[0].ends_with(".rs"));
    assert_ne!(args.paths[0], "src/main.rs");

    // Excluded should be redacted
    assert_ne!(args.excluded[0], "node_modules");

    // excluded_redacted should be true
    assert!(args.excluded_redacted);
}

/// Test scan_args with RedactMode::None - should NOT redact
#[test]
fn test_scan_args_redact_none_mode() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let global = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };

    let args = scan_args(&paths, &global, Some(RedactMode::None));

    // With None mode, paths should NOT be redacted
    assert_eq!(args.paths[0], "src/lib.rs");

    // Excluded should NOT be redacted
    assert_eq!(args.excluded[0], "target");

    // excluded_redacted should be false
    assert!(!args.excluded_redacted);
}

/// Test scan_args with None redact option - should NOT redact (default behavior)
#[test]
fn test_scan_args_no_redact_option() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let global = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };

    let args = scan_args(&paths, &global, None);

    // With no redact option, paths should NOT be redacted
    assert_eq!(args.paths[0], "src/lib.rs");

    // Excluded should NOT be redacted
    assert_eq!(args.excluded[0], "target");

    // excluded_redacted should be false
    assert!(!args.excluded_redacted);
}

/// Test scan_args with empty excluded list - excluded_redacted should be false even with redaction
#[test]
fn test_scan_args_empty_excluded_with_redact() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let global = ScanOptions {
        excluded: vec![], // empty
        ..Default::default()
    };

    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    // Paths should still be redacted
    assert!(args.paths[0].ends_with(".rs"));
    assert_ne!(args.paths[0], "src/lib.rs");

    // excluded_redacted should be false because excluded is empty
    assert!(!args.excluded_redacted);
}

/// Test scan_args preserves all global flags
#[test]
fn test_scan_args_preserves_global_flags() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        hidden: true,
        no_ignore: true,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };

    let args = scan_args(&paths, &global, None);

    assert!(args.hidden);
    assert!(args.no_ignore);
    // When no_ignore is true, all sub-flags should be true
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
    assert!(args.treat_doc_strings_as_comments);
}

/// Test scan_args with no_ignore implies sub-flags
#[test]
fn test_scan_args_no_ignore_implies_sub_flags() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        no_ignore: true,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        ..Default::default()
    };

    let args = scan_args(&paths, &global, None);

    // no_ignore should imply all sub-flags
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

/// Test scan_args with individual sub-flags
#[test]
fn test_scan_args_individual_sub_flags() {
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: true,
        ..Default::default()
    };

    let args = scan_args(&paths, &global, None);

    // Individual flags should be preserved
    assert!(args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

// ============================================================================
// normalize_scan_input tests - Additional edge cases
// ============================================================================

#[test]
fn test_normalize_scan_input_multiple_dot_slash() {
    let p = std::path::Path::new("././src/lib.rs");
    let normalized = normalize_scan_input(p);
    assert_eq!(normalized, "src/lib.rs");
}

#[test]
fn test_normalize_scan_input_empty_after_strip() {
    let p = std::path::Path::new("./");
    let normalized = normalize_scan_input(p);
    assert_eq!(normalized, ".");
}

// ============================================================================
// Diff computation tests - Kill remaining mutants
// ============================================================================

fn make_lang_row(lang: &str, code: usize, lines: usize, files: usize) -> LangRow {
    LangRow {
        lang: lang.to_string(),
        code,
        lines,
        files,
        bytes: code * 10,
        tokens: code / 4,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

fn make_totals(code: usize, lines: usize, files: usize) -> Totals {
    Totals {
        code,
        lines,
        files,
        bytes: code * 10,
        tokens: code / 4,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

fn make_lang_report(rows: Vec<LangRow>, totals: Totals) -> LangReport {
    LangReport {
        rows,
        total: totals,
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    }
}

#[test]
fn test_compute_diff_rows_delta_calculation() {
    // Tests delta_code = new.code - old.code
    let from = make_lang_report(
        vec![make_lang_row("Rust", 100, 120, 5)],
        make_totals(100, 120, 5),
    );

    let to = make_lang_report(
        vec![make_lang_row("Rust", 150, 180, 7)],
        make_totals(150, 180, 7),
    );

    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].delta_code, 50); // 150 - 100 = 50, not 250 if + was used
    assert_eq!(rows[0].delta_lines, 60); // 180 - 120 = 60
    assert_eq!(rows[0].delta_files, 2); // 7 - 5 = 2
}

#[test]
fn test_compute_diff_rows_negative_delta() {
    // Tests negative deltas (code decreased)
    let from = make_lang_report(
        vec![make_lang_row("Go", 200, 240, 10)],
        make_totals(200, 240, 10),
    );

    let to = make_lang_report(vec![make_lang_row("Go", 50, 60, 3)], make_totals(50, 60, 3));

    let rows = compute_diff_rows(&from, &to);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].delta_code, -150); // 50 - 200 = -150
    assert_eq!(rows[0].delta_lines, -180); // 60 - 240 = -180
    assert_eq!(rows[0].delta_files, -7); // 3 - 10 = -7
}

#[test]
fn test_compute_diff_totals_accumulation() {
    // Tests += operations in compute_diff_totals
    let from = make_lang_report(
        vec![
            make_lang_row("Rust", 100, 120, 5),
            make_lang_row("Go", 200, 240, 10),
        ],
        make_totals(300, 360, 15),
    );

    let to = make_lang_report(
        vec![
            make_lang_row("Rust", 150, 180, 7),
            make_lang_row("Go", 220, 260, 11),
        ],
        make_totals(370, 440, 18),
    );

    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);

    // old_code = 100 + 200 = 300
    assert_eq!(totals.old_code, 300);
    // new_code = 150 + 220 = 370
    assert_eq!(totals.new_code, 370);
    // delta_code = 50 + 20 = 70
    assert_eq!(totals.delta_code, 70);

    // Similar for other fields
    assert_eq!(totals.old_lines, 360);
    assert_eq!(totals.new_lines, 440);
    assert_eq!(totals.delta_lines, 80);
}

#[test]
fn test_compute_diff_totals_with_subtraction() {
    // Ensures -= would fail
    let from = make_lang_report(
        vec![make_lang_row("Python", 500, 600, 20)],
        make_totals(500, 600, 20),
    );

    let to = make_lang_report(
        vec![make_lang_row("Python", 600, 700, 25)],
        make_totals(600, 700, 25),
    );

    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);

    // With += the totals should be positive sums
    assert_eq!(totals.old_code, 500);
    assert_eq!(totals.new_code, 600);
    assert_eq!(totals.delta_code, 100);

    // If -= was used, delta would be -100
    assert!(totals.delta_code > 0);
}

#[test]
fn test_render_diff_md_non_empty() {
    // Kills mutants that replace render_diff_md with String::new() or "xyzzy"
    let from = make_lang_report(
        vec![make_lang_row("Rust", 100, 120, 5)],
        make_totals(100, 120, 5),
    );

    let to = make_lang_report(
        vec![make_lang_row("Rust", 200, 240, 10)],
        make_totals(200, 240, 10),
    );

    let rows = compute_diff_rows(&from, &to);
    let totals = compute_diff_totals(&rows);

    let md = render_diff_md("baseline", "current", &rows, &totals);

    // Must not be empty
    assert!(!md.is_empty());

    // Must contain expected structure
    assert!(md.contains("baseline"));
    assert!(md.contains("current"));
    assert!(md.contains("Rust"));
    assert!(md.contains("|Language|"));
    assert!(md.contains("|**Total**|"));

    // Must contain actual values
    assert!(md.contains("100")); // old_code
    assert!(md.contains("200")); // new_code
    assert!(md.contains("+100")); // delta
}

#[test]
fn test_diff_row_old_new_distinct() {
    // Tests the == operators in compute_diff_rows for filtering unchanged rows
    let report = make_lang_report(
        vec![make_lang_row("Java", 300, 350, 15)],
        make_totals(300, 350, 15),
    );

    // Same report should produce no diff rows
    let rows = compute_diff_rows(&report, &report);
    assert!(
        rows.is_empty(),
        "identical reports should have no diff rows"
    );
}

#[test]
fn test_diff_row_only_bytes_changed() {
    // Tests that changes in bytes alone trigger a diff row
    let from = make_lang_report(
        vec![LangRow {
            lang: "C".to_string(),
            code: 100,
            lines: 120,
            files: 5,
            bytes: 1000, // different bytes
            tokens: 25,
            avg_lines: 24,
        }],
        make_totals(100, 120, 5),
    );

    let to = make_lang_report(
        vec![LangRow {
            lang: "C".to_string(),
            code: 100,
            lines: 120,
            files: 5,
            bytes: 2000, // changed bytes
            tokens: 25,
            avg_lines: 24,
        }],
        make_totals(100, 120, 5),
    );

    let rows = compute_diff_rows(&from, &to);
    // bytes change should trigger a diff row
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].delta_bytes, 1000);
}

#[test]
fn test_diff_row_only_tokens_changed() {
    // Tests that changes in tokens alone trigger a diff row
    let from = make_lang_report(
        vec![LangRow {
            lang: "Rust".to_string(),
            code: 100,
            lines: 120,
            files: 5,
            bytes: 1000,
            tokens: 100, // different tokens
            avg_lines: 24,
        }],
        make_totals(100, 120, 5),
    );

    let to = make_lang_report(
        vec![LangRow {
            lang: "Rust".to_string(),
            code: 100,
            lines: 120,
            files: 5,
            bytes: 1000,
            tokens: 200, // changed tokens
            avg_lines: 24,
        }],
        make_totals(100, 120, 5),
    );

    let rows = compute_diff_rows(&from, &to);
    // tokens change should trigger a diff row
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].delta_tokens, 100);
}

// ============================================================================
// CSV export format verification
// ============================================================================

#[test]
fn test_write_export_csv_format() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 10,
                lines: 130,
                bytes: 1000,
                tokens: 250,
            },
            FileRow {
                path: "src/main.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Child,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 500,
                tokens: 125,
            },
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let _global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Csv,
        out: None,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::None,
        strip_prefix: None,
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_csv_to(&mut buffer, &export, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();

    // Verify CSV structure
    assert!(output.contains("path,module,lang,kind,code,comments,blanks,lines,bytes,tokens"));
    assert!(output.contains("src/lib.rs"));
    assert!(output.contains("parent"));
    assert!(output.contains("child"));
}

// ============================================================================
// JSONL export format verification
// ============================================================================

#[test]
fn test_write_export_jsonl_with_meta() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "test.rs".to_string(),
            module: "".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 2,
            blanks: 1,
            lines: 13,
            bytes: 100,
            tokens: 25,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Jsonl,
        out: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::None,
        strip_prefix: None,
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_jsonl_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    // Should have meta line and row line
    assert!(lines.len() >= 2);
    assert!(lines[0].contains("\"type\":\"meta\""));
    assert!(lines[1].contains("\"type\":\"row\""));
    assert!(lines[1].contains("test.rs"));
}

#[test]
fn test_write_export_jsonl_without_meta() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "test.rs".to_string(),
            module: "".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 2,
            blanks: 1,
            lines: 13,
            bytes: 100,
            tokens: 25,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Jsonl,
        out: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: false, // No meta
        redact: RedactMode::None,
        strip_prefix: None,
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_jsonl_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    // Should have only row lines, no meta
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("\"type\":\"row\""));
    assert!(!output.contains("\"type\":\"meta\""));
}

// ============================================================================
// JSON export format verification with redaction
// ============================================================================

#[test]
fn test_write_export_json_with_redaction() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/secret.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 2,
            blanks: 1,
            lines: 13,
            bytes: 100,
            tokens: 25,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Json,
        out: None,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::All, // Full redaction
        strip_prefix: None,
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_json_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();

    // Path should be redacted (not contain original)
    assert!(!output.contains("src/secret.rs"));

    // Should still end with .rs
    assert!(output.contains(".rs"));

    // Module should also be redacted in All mode
    assert!(!output.contains("\"module\":\"src\""));
}

// ============================================================================
// CycloneDX export verification
// ============================================================================

#[test]
fn test_write_export_cyclonedx_structure() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 10,
                lines: 130,
                bytes: 1000,
                tokens: 250,
            },
            FileRow {
                path: "src/main.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Child,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 500,
                tokens: 125,
            },
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_cyclonedx_to(&mut buffer, &export, RedactMode::None).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();

    // Verify CycloneDX structure (pretty-printed JSON has spaces after colons)
    assert!(output.contains("\"bomFormat\": \"CycloneDX\""));
    assert!(output.contains("\"specVersion\": \"1.6\""));
    assert!(output.contains("\"components\""));
    assert!(output.contains("\"tokmd:lang\""));
    assert!(output.contains("\"tokmd:code\""));

    // Child files should have kind property
    assert!(output.contains("\"tokmd:kind\""));
    assert!(output.contains("\"child\""));
}

#[test]
fn test_cyclonedx_empty_module_no_group() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "standalone.rs".to_string(),
            module: "".to_string(), // Empty module
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 2,
            blanks: 1,
            lines: 13,
            bytes: 100,
            tokens: 25,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_cyclonedx_to(&mut buffer, &export, RedactMode::None).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();

    // Should not have a "group" field for empty module
    // The component should exist but group should be null/missing
    assert!(output.contains("standalone.rs"));

    // Parse and verify
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let components = json["components"].as_array().unwrap();
    assert_eq!(components.len(), 1);

    // group should be null (not present) for empty module
    assert!(components[0].get("group").is_none() || components[0]["group"].is_null());
}

// ============================================================================
// now_ms mutant killers - Verify generated_at_ms is a reasonable timestamp
// ============================================================================

/// Kills mutants: now_ms -> 0 and now_ms -> 1
/// By verifying the generated_at_ms field is a reasonable Unix timestamp.
#[test]
fn test_jsonl_generated_at_ms_is_reasonable() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "test.rs".to_string(),
            module: "".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 2,
            blanks: 1,
            lines: 13,
            bytes: 100,
            tokens: 25,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Jsonl,
        out: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::None,
        strip_prefix: None,
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_jsonl_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let meta_line = output.lines().next().unwrap();
    let meta: serde_json::Value = serde_json::from_str(meta_line).unwrap();

    let generated_at_ms = meta["generated_at_ms"].as_u64().unwrap();

    // Jan 1, 2020 00:00:00 UTC in milliseconds = 1577836800000
    // This kills the mutant that replaces now_ms with 0 or 1
    const JAN_1_2020_MS: u64 = 1_577_836_800_000;

    assert!(
        generated_at_ms > JAN_1_2020_MS,
        "generated_at_ms ({}) should be greater than Jan 1 2020 ({})",
        generated_at_ms,
        JAN_1_2020_MS
    );
}

/// Kills mutants: now_ms -> 0 and now_ms -> 1 via JSON export
#[test]
fn test_json_generated_at_ms_is_reasonable() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "test.rs".to_string(),
            module: "".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 2,
            blanks: 1,
            lines: 13,
            bytes: 100,
            tokens: 25,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Json,
        out: None,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::None,
        strip_prefix: None,
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_json_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    let generated_at_ms = json["generated_at_ms"].as_u64().unwrap();

    // Jan 1, 2020 00:00:00 UTC in milliseconds = 1577836800000
    const JAN_1_2020_MS: u64 = 1_577_836_800_000;

    assert!(
        generated_at_ms > JAN_1_2020_MS,
        "generated_at_ms ({}) should be greater than Jan 1 2020 ({})",
        generated_at_ms,
        JAN_1_2020_MS
    );
}

/// Kills mutants: now_ms -> 0 and now_ms -> 1 via diff receipt
#[test]
fn test_diff_receipt_generated_at_ms_is_reasonable() {
    use tokmd_format::create_diff_receipt;
    use tokmd_types::DiffTotals;

    let receipt = create_diff_receipt("from", "to", vec![], DiffTotals::default());

    // Jan 1, 2020 00:00:00 UTC in milliseconds = 1577836800000
    const JAN_1_2020_MS: u128 = 1_577_836_800_000;

    assert!(
        receipt.generated_at_ms > JAN_1_2020_MS,
        "generated_at_ms ({}) should be greater than Jan 1 2020 ({})",
        receipt.generated_at_ms,
        JAN_1_2020_MS
    );
}

// ============================================================================
// I/O wrapper mutant killers - print_lang_report, print_module_report, write_export
// Kills: print_lang_report -> Ok(()), print_module_report -> Ok(()),
//        write_export -> Ok(()), write_export_to -> Ok(())
// ============================================================================

// NOTE: Tests for print_lang_report and print_module_report are not included here
// because stdout capture with the `gag` crate doesn't work reliably on Windows.
// These functions are thin I/O wrappers that call the already-tested render_* functions.
// They are excluded via `.cargo/mutants.toml` configuration.

/// Kills mutants: write_export -> Ok(()), write_export_to -> Ok(())
/// By writing to a temp file and verifying file exists and has content.
#[test]
fn test_write_export_writes_to_file() {
    use tokmd_format::write_export;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 100,
            comments: 10,
            blanks: 5,
            lines: 115,
            bytes: 1000,
            tokens: 250,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    let temp_path = temp_file.path().to_path_buf();

    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Jsonl,
        out: Some(temp_path.clone()),
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::None,
        strip_prefix: None,
    };

    write_export(&export, &global, &args).expect("write_export should succeed");

    // Verify file was written and has content
    let content = std::fs::read_to_string(&temp_path).expect("read temp file");
    assert!(
        !content.trim().is_empty(),
        "exported file must not be empty"
    );

    // Verify JSONL structure
    let lines: Vec<&str> = content.lines().collect();
    assert!(lines.len() >= 2, "should have meta and row lines");
    assert!(lines[0].contains("\"type\":\"meta\""));
    assert!(lines[1].contains("\"type\":\"row\""));
}

// ============================================================================
// strip_prefix redaction mutant killers
// Kills: should_redact || → &&, strip_prefix_redacted && → ||, == → !=
// ============================================================================

/// Kills mutant: should_redact (Paths || All) → (Paths && All)
/// and strip_prefix_redacted (should_redact && strip_prefix.is_some()) → ||
/// by testing JSONL with Paths redaction mode.
#[test]
fn test_jsonl_strip_prefix_redacted_with_paths_mode() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 1,
            blanks: 1,
            lines: 12,
            bytes: 100,
            tokens: 20,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Jsonl,
        out: None,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::Paths, // Paths mode should trigger redaction
        strip_prefix: Some(PathBuf::from("src")),
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_jsonl_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let meta_line = output.lines().next().unwrap();
    let meta: serde_json::Value = serde_json::from_str(meta_line).unwrap();

    // strip_prefix_redacted must be true (kills && → || mutant)
    assert_eq!(
        meta["args"]["strip_prefix_redacted"], true,
        "strip_prefix_redacted must be true when redact=Paths and strip_prefix is set"
    );

    // strip_prefix must be redacted (16 char hash, not "src")
    let sp = meta["args"]["strip_prefix"].as_str().unwrap();
    assert_ne!(sp, "src", "strip_prefix should be redacted, not literal");
    assert_eq!(sp.len(), 16, "redacted strip_prefix should be 16 chars");
}

/// Kills mutant: should_redact (Paths || All) → (Paths && All)
/// by testing JSONL with All redaction mode.
#[test]
fn test_jsonl_strip_prefix_redacted_with_all_mode() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 1,
            blanks: 1,
            lines: 12,
            bytes: 100,
            tokens: 20,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Jsonl,
        out: None,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::All, // All mode should also trigger redaction
        strip_prefix: Some(PathBuf::from("prefix")),
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_jsonl_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let meta_line = output.lines().next().unwrap();
    let meta: serde_json::Value = serde_json::from_str(meta_line).unwrap();

    // strip_prefix_redacted must be true
    assert_eq!(
        meta["args"]["strip_prefix_redacted"], true,
        "strip_prefix_redacted must be true when redact=All and strip_prefix is set"
    );

    // strip_prefix must be redacted
    let sp = meta["args"]["strip_prefix"].as_str().unwrap();
    assert_ne!(sp, "prefix", "strip_prefix should be redacted");
    assert_eq!(sp.len(), 16, "redacted strip_prefix should be 16 chars");
}

/// Kills mutant: strip_prefix_redacted = should_redact && strip_prefix.is_some() → ||
/// by testing with no strip_prefix (should be false).
#[test]
fn test_jsonl_strip_prefix_redacted_false_when_no_strip_prefix() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 1,
            blanks: 1,
            lines: 12,
            bytes: 100,
            tokens: 20,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Jsonl,
        out: None,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::Paths, // Redaction enabled but no strip_prefix
        strip_prefix: None,        // No strip_prefix
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_jsonl_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let meta_line = output.lines().next().unwrap();
    let meta: serde_json::Value = serde_json::from_str(meta_line).unwrap();

    // strip_prefix_redacted must be false/omitted (kills || mutant - would be true with ||)
    // The field is skipped during serialization when false, so it will be null in the JSON
    assert!(
        meta["args"]["strip_prefix_redacted"].is_null()
            || meta["args"]["strip_prefix_redacted"] == false,
        "strip_prefix_redacted must be false/omitted when strip_prefix is None; got {:?}",
        meta["args"]["strip_prefix_redacted"]
    );

    // strip_prefix should be null
    assert!(
        meta["args"]["strip_prefix"].is_null(),
        "strip_prefix should be null when not set"
    );
}

/// Kills mutant: should_redact == RedactMode::Paths → !=
/// by testing with None redaction mode (should NOT redact).
#[test]
fn test_jsonl_no_redaction_with_none_mode() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 1,
            blanks: 1,
            lines: 12,
            bytes: 100,
            tokens: 20,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Jsonl,
        out: None,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::None, // None mode should NOT trigger redaction
        strip_prefix: Some(PathBuf::from("src")),
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_jsonl_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let meta_line = output.lines().next().unwrap();
    let meta: serde_json::Value = serde_json::from_str(meta_line).unwrap();

    // strip_prefix_redacted must be false/omitted
    // The field is skipped during serialization when false, so it will be null in the JSON
    assert!(
        meta["args"]["strip_prefix_redacted"].is_null()
            || meta["args"]["strip_prefix_redacted"] == false,
        "strip_prefix_redacted must be false/omitted when redact=None; got {:?}",
        meta["args"]["strip_prefix_redacted"]
    );

    // strip_prefix should be literal "src", not redacted
    assert_eq!(
        meta["args"]["strip_prefix"].as_str().unwrap(),
        "src",
        "strip_prefix should be literal when redact=None"
    );
}

/// Same tests for JSON export to kill mutants in write_export_json
#[test]
fn test_json_strip_prefix_redacted_with_paths_mode() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 1,
            blanks: 1,
            lines: 12,
            bytes: 100,
            tokens: 20,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Json,
        out: None,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::Paths,
        strip_prefix: Some(PathBuf::from("src")),
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_json_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();

    assert_eq!(
        json["args"]["strip_prefix_redacted"], true,
        "strip_prefix_redacted must be true when redact=Paths and strip_prefix is set"
    );

    let sp = json["args"]["strip_prefix"].as_str().unwrap();
    assert_ne!(sp, "src");
    assert_eq!(sp.len(), 16);
}

#[test]
fn test_json_no_redaction_with_none_mode() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 1,
            blanks: 1,
            lines: 12,
            bytes: 100,
            tokens: 20,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let global = ScanOptions::default();
    let args = ExportArgs {
        paths: vec![PathBuf::from(".")],
        format: ExportFormat::Json,
        out: None,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        meta: true,
        redact: RedactMode::None,
        strip_prefix: Some(PathBuf::from("myprefix")),
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_json_to(&mut buffer, &export, &global, &args).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let json: serde_json::Value = serde_json::from_str(output.trim()).unwrap();

    // strip_prefix_redacted must be false/omitted
    // The field is skipped during serialization when false, so it will be null in the JSON
    assert!(
        json["args"]["strip_prefix_redacted"].is_null()
            || json["args"]["strip_prefix_redacted"] == false,
        "strip_prefix_redacted must be false/omitted when redact=None; got {:?}",
        json["args"]["strip_prefix_redacted"]
    );

    assert_eq!(
        json["args"]["strip_prefix"].as_str().unwrap(),
        "myprefix",
        "strip_prefix should be literal when redact=None"
    );
}

// ============================================================================
// File-writing helper mutant killers
// Kills: write_lang_json_to_file -> Ok(()), write_module_json_to_file -> Ok(()),
//        write_export_jsonl_to_file -> Ok(())
// ============================================================================

/// Kills mutant: write_lang_json_to_file -> Ok(())
/// By writing to a temp file and verifying valid JSON was written.
#[test]
fn test_write_lang_json_to_file_writes_valid_json() {
    use tokmd_format::write_lang_json_to_file;

    let report = LangReport {
        rows: vec![LangRow {
            lang: "Rust".to_string(),
            code: 100,
            lines: 120,
            files: 5,
            bytes: 5000,
            tokens: 250,
            avg_lines: 24,
        }],
        total: Totals {
            code: 100,
            lines: 120,
            files: 5,
            bytes: 5000,
            tokens: 250,
            avg_lines: 24,
        },
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };

    let scan = ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec![],
        excluded_redacted: false,
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    };

    let args_meta = LangArgsMeta {
        format: "json".to_string(),
        top: 0,
        with_files: true,
        children: ChildrenMode::Collapse,
    };

    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let file_path = temp_dir.path().join("lang.json");

    write_lang_json_to_file(&file_path, &report, &scan, &args_meta)
        .expect("write_lang_json_to_file should succeed");

    // Verify file exists and contains valid JSON
    let content = std::fs::read_to_string(&file_path).expect("read file");
    assert!(!content.trim().is_empty(), "file must not be empty");

    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert_eq!(json["mode"], "lang");
    // Note: LangReceipt uses #[serde(flatten)] on report, so rows are at top level
    assert!(json["rows"].is_array());
    assert_eq!(json["rows"][0]["lang"], "Rust");
    assert_eq!(json["rows"][0]["code"], 100);
}

/// Kills mutant: write_module_json_to_file -> Ok(())
/// By writing to a temp file and verifying valid JSON was written.
#[test]
fn test_write_module_json_to_file_writes_valid_json() {
    use tokmd_format::write_module_json_to_file;

    let report = ModuleReport {
        rows: vec![ModuleRow {
            module: "src".to_string(),
            code: 200,
            lines: 240,
            files: 10,
            bytes: 10000,
            tokens: 500,
            avg_lines: 24,
        }],
        total: Totals {
            code: 200,
            lines: 240,
            files: 10,
            bytes: 10000,
            tokens: 500,
            avg_lines: 24,
        },
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        top: 0,
    };

    let scan = ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec![],
        excluded_redacted: false,
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    };

    let args_meta = ModuleArgsMeta {
        format: "json".to_string(),
        top: 0,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let file_path = temp_dir.path().join("module.json");

    write_module_json_to_file(&file_path, &report, &scan, &args_meta)
        .expect("write_module_json_to_file should succeed");

    // Verify file exists and contains valid JSON
    let content = std::fs::read_to_string(&file_path).expect("read file");
    assert!(!content.trim().is_empty(), "file must not be empty");

    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert_eq!(json["mode"], "module");
    // Note: ModuleReceipt uses #[serde(flatten)] on report, so rows are at top level
    assert!(json["rows"].is_array());
    assert_eq!(json["rows"][0]["module"], "src");
    assert_eq!(json["rows"][0]["code"], 200);
}

/// Kills mutant: write_export_jsonl_to_file -> Ok(())
/// By writing to a temp file and verifying valid JSONL was written.
#[test]
fn test_write_export_jsonl_to_file_writes_valid_jsonl() {
    use tokmd_format::write_export_jsonl_to_file;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 50,
            comments: 10,
            blanks: 5,
            lines: 65,
            bytes: 500,
            tokens: 125,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let scan = ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec![],
        excluded_redacted: false,
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    };

    let args_meta = ExportArgsMeta {
        format: ExportFormat::Jsonl,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
        min_code: 0,
        max_rows: 0,
        redact: RedactMode::None,
        strip_prefix: None,
        strip_prefix_redacted: false,
    };

    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let file_path = temp_dir.path().join("export.jsonl");

    write_export_jsonl_to_file(&file_path, &export, &scan, &args_meta)
        .expect("write_export_jsonl_to_file should succeed");

    // Verify file exists and contains valid JSONL
    let content = std::fs::read_to_string(&file_path).expect("read file");
    assert!(!content.trim().is_empty(), "file must not be empty");

    let lines: Vec<&str> = content.lines().collect();
    assert!(lines.len() >= 2, "should have meta and row lines");

    // Verify each line is valid JSON
    let meta: serde_json::Value = serde_json::from_str(lines[0]).expect("meta is valid JSON");
    assert_eq!(meta["type"], "meta");
    assert_eq!(meta["mode"], "export");

    let row: serde_json::Value = serde_json::from_str(lines[1]).expect("row is valid JSON");
    assert_eq!(row["type"], "row");
    assert_eq!(row["path"], "src/main.rs");
    assert_eq!(row["code"], 50);
}

// ============================================================================
// CycloneDX child kind mutant killer - test Parent vs Child distinction
// Kills: row.kind == FileKind::Child -> != or true/false
// ============================================================================

/// Kills mutant: row.kind == FileKind::Child → !=
/// By testing that Parent rows do NOT have tokmd:kind property.
#[test]
fn test_cyclonedx_parent_has_no_kind_property() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent, // Parent, not child
            code: 100,
            comments: 20,
            blanks: 10,
            lines: 130,
            bytes: 1000,
            tokens: 250,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_cyclonedx_to(&mut buffer, &export, RedactMode::None).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    let components = json["components"].as_array().unwrap();
    assert_eq!(components.len(), 1);

    // Parent should NOT have tokmd:kind property
    let properties = components[0]["properties"].as_array().unwrap();
    let has_kind = properties
        .iter()
        .any(|p| p["name"].as_str() == Some("tokmd:kind"));
    assert!(
        !has_kind,
        "Parent files should NOT have tokmd:kind property"
    );
}

/// Complementary test: Child rows SHOULD have tokmd:kind property
#[test]
fn test_cyclonedx_child_has_kind_property() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child, // Child
            code: 100,
            comments: 20,
            blanks: 10,
            lines: 130,
            bytes: 1000,
            tokens: 250,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let mut buffer = Cursor::new(Vec::new());
    tokmd_format::write_export_cyclonedx_to(&mut buffer, &export, RedactMode::None).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    let components = json["components"].as_array().unwrap();
    assert_eq!(components.len(), 1);

    // Child should have tokmd:kind = "child" property
    let properties = components[0]["properties"].as_array().unwrap();
    let kind_prop = properties
        .iter()
        .find(|p| p["name"].as_str() == Some("tokmd:kind"));
    assert!(
        kind_prop.is_some(),
        "Child files should have tokmd:kind property"
    );
    assert_eq!(kind_prop.unwrap()["value"], "child");
}

// ============================================================================
// compute_diff_totals multi-row accumulation killers
// Kills: += → -= or *= mutations in compute_diff_totals
// ============================================================================

/// Kills mutants in compute_diff_totals by using non-canceling deltas.
/// Every accumulated total is non-zero, which kills += → -= and += → *= mutations.
/// Key: if totals start at 0 and *= is used, result stays 0 (mutation survives).
/// If deltas cancel to 0, -= can also survive. We prevent both by ensuring all sums are non-zero.
#[test]
fn test_compute_diff_totals_nonzero_deltas() {
    use tokmd_types::DiffRow;

    // Create rows where NO accumulated total is zero
    let rows = vec![
        DiffRow {
            lang: "Rust".to_string(),
            old_code: 100,
            new_code: 160,
            delta_code: 60, // +60
            old_lines: 200,
            new_lines: 260,
            delta_lines: 60, // +60
            old_files: 5,
            new_files: 8,
            delta_files: 3, // +3
            old_bytes: 1000,
            new_bytes: 1800,
            delta_bytes: 800, // +800
            old_tokens: 250,
            new_tokens: 400,
            delta_tokens: 150, // +150
        },
        DiffRow {
            lang: "Go".to_string(),
            old_code: 200,
            new_code: 180,
            delta_code: -20, // sums to 40
            old_lines: 300,
            new_lines: 310,
            delta_lines: 10, // sums to 70
            old_files: 10,
            new_files: 9,
            delta_files: -1, // sums to 2
            old_bytes: 2000,
            new_bytes: 2100,
            delta_bytes: 100, // sums to 900
            old_tokens: 500,
            new_tokens: 520,
            delta_tokens: 20, // sums to 170
        },
    ];

    let totals = compute_diff_totals(&rows);

    // All sums are non-zero - kills *= mutations (would stay 0)
    // old_code = 100 + 200 = 300
    assert_eq!(totals.old_code, 300);
    // new_code = 160 + 180 = 340
    assert_eq!(totals.new_code, 340);
    // delta_code = 60 + (-20) = 40 (non-zero!)
    assert_eq!(totals.delta_code, 40);

    // old_lines = 200 + 300 = 500
    assert_eq!(totals.old_lines, 500);
    // new_lines = 260 + 310 = 570
    assert_eq!(totals.new_lines, 570);
    // delta_lines = 60 + 10 = 70 (non-zero!)
    assert_eq!(totals.delta_lines, 70);

    // old_files = 5 + 10 = 15
    assert_eq!(totals.old_files, 15);
    // new_files = 8 + 9 = 17
    assert_eq!(totals.new_files, 17);
    // delta_files = 3 + (-1) = 2 (non-zero!)
    assert_eq!(totals.delta_files, 2);

    // old_bytes = 1000 + 2000 = 3000
    assert_eq!(totals.old_bytes, 3000);
    // new_bytes = 1800 + 2100 = 3900
    assert_eq!(totals.new_bytes, 3900);
    // delta_bytes = 800 + 100 = 900 (non-zero!)
    assert_eq!(totals.delta_bytes, 900);

    // old_tokens = 250 + 500 = 750
    assert_eq!(totals.old_tokens, 750);
    // new_tokens = 400 + 520 = 920
    assert_eq!(totals.new_tokens, 920);
    // delta_tokens = 150 + 20 = 170 (non-zero!)
    assert_eq!(totals.delta_tokens, 170);
}

// ============================================================================
// Deterministic CycloneDX snapshot tests
// ============================================================================

#[test]
fn test_cyclonedx_snapshot_deterministic() {
    use std::io::Cursor;

    let export = ExportData {
        rows: vec![
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 10,
                lines: 130,
                bytes: 1000,
                tokens: 250,
            },
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let mut buffer = Cursor::new(Vec::new());

    // Use fixed values for determinism
    let serial = Some("urn:uuid:00000000-0000-0000-0000-000000000000".to_string());
    let timestamp = Some("1970-01-01T00:00:00Z".to_string());

    tokmd_format::write_export_cyclonedx_with_options(
        &mut buffer,
        &export,
        RedactMode::None,
        serial,
        timestamp
    ).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();

    // Parse JSON to ensure formatting is consistent before snapshot
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let pretty = serde_json::to_string_pretty(&json).unwrap();

    insta::assert_snapshot!(pretty);
}
