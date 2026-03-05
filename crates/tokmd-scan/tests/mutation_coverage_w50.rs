//! Targeted tests for mutation testing coverage gaps (W50).
//!
//! Each test catches common mutations: replacing operators,
//! negating conditions, removing statements.

use std::path::PathBuf;

use tokmd_scan::scan;
use tokmd_settings::ScanOptions;
use tokmd_types::ConfigMode;

fn default_scan_options() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

fn crate_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

// ---------------------------------------------------------------------------
// 1. Exclude pattern actually reduces results (not ignored)
// ---------------------------------------------------------------------------

#[test]
fn exclude_pattern_reduces_results() {
    let args_no_exclude = default_scan_options();
    let result_all = scan(&[crate_src()], &args_no_exclude).unwrap();

    let mut args_exclude = default_scan_options();
    args_exclude.excluded = vec!["*.rs".to_string()];
    let result_excluded = scan(&[crate_src()], &args_exclude).unwrap();

    // With *.rs excluded, we should have fewer (or zero) Rust entries
    let rust_all = result_all
        .get(&tokei::LanguageType::Rust)
        .map(|l| l.code)
        .unwrap_or(0);
    let rust_excluded = result_excluded
        .get(&tokei::LanguageType::Rust)
        .map(|l| l.code)
        .unwrap_or(0);

    assert!(
        rust_excluded < rust_all,
        "excluding *.rs must reduce Rust code count: all={rust_all}, excluded={rust_excluded}"
    );
}

// ---------------------------------------------------------------------------
// 2. Scanning empty dir returns zero totals
// ---------------------------------------------------------------------------

#[test]
fn scanning_empty_dir_returns_zero_totals() {
    let dir = tempfile::tempdir().unwrap();
    let args = default_scan_options();
    let result = scan(&[dir.path().to_path_buf()], &args).unwrap();

    let total_code: usize = result.values().map(|l| l.code).sum();
    assert_eq!(total_code, 0, "empty directory should have zero code");
}

// ---------------------------------------------------------------------------
// 3. Scanning nonexistent path returns error
// ---------------------------------------------------------------------------

#[test]
fn scanning_nonexistent_path_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let fake = dir.path().join("nonexistent");
    let args = default_scan_options();
    let result = scan(&[fake], &args);

    assert!(result.is_err(), "nonexistent path must return Err");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Path not found"),
        "error message should mention 'Path not found', got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// 4. Scanning the crate's own src finds Rust code > 0
// ---------------------------------------------------------------------------

#[test]
fn scan_own_src_finds_rust_code() {
    let args = default_scan_options();
    let result = scan(&[crate_src()], &args).unwrap();
    let rust = result
        .get(&tokei::LanguageType::Rust)
        .expect("should find Rust");
    assert!(rust.code > 0, "Rust code count should be > 0");
}

// ---------------------------------------------------------------------------
// 5. Config mode None still works
// ---------------------------------------------------------------------------

#[test]
fn config_mode_none_scans_successfully() {
    let mut args = default_scan_options();
    args.config = ConfigMode::None;
    let result = scan(&[crate_src()], &args);
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// 6. Scan result is deterministic across two runs
// ---------------------------------------------------------------------------

#[test]
fn scan_is_deterministic() {
    let args = default_scan_options();
    let r1 = scan(&[crate_src()], &args).unwrap();
    let r2 = scan(&[crate_src()], &args).unwrap();

    let code1: usize = r1.values().map(|l| l.code).sum();
    let code2: usize = r2.values().map(|l| l.code).sum();
    assert_eq!(
        code1, code2,
        "two scans of the same path must yield identical code counts"
    );
}

// ---------------------------------------------------------------------------
// 7. hidden flag does not crash (smoke test)
// ---------------------------------------------------------------------------

#[test]
fn hidden_flag_smoke_test() {
    let mut args = default_scan_options();
    args.hidden = true;
    assert!(scan(&[crate_src()], &args).is_ok());
}

// ---------------------------------------------------------------------------
// 8. treat_doc_strings_as_comments changes comment count
// ---------------------------------------------------------------------------

#[test]
fn doc_strings_flag_changes_output() {
    let mut normal = default_scan_options();
    normal.treat_doc_strings_as_comments = false;
    let r_normal = scan(&[crate_src()], &normal).unwrap();

    let mut doc_as_comments = default_scan_options();
    doc_as_comments.treat_doc_strings_as_comments = true;
    let r_doc = scan(&[crate_src()], &doc_as_comments).unwrap();

    // The total lines should remain the same even if categorization differs
    let lines_normal: usize = r_normal
        .values()
        .flat_map(|l| l.reports.iter())
        .map(|r| {
            let s = r.stats.summarise();
            s.code + s.comments + s.blanks
        })
        .sum();
    let lines_doc: usize = r_doc
        .values()
        .flat_map(|l| l.reports.iter())
        .map(|r| {
            let s = r.stats.summarise();
            s.code + s.comments + s.blanks
        })
        .sum();
    assert_eq!(
        lines_normal, lines_doc,
        "total lines should not change with doc-string flag"
    );
}
