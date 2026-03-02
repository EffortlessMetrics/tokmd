//! Unit tests for `tokmd-analysis-entropy` — classification, edge cases, limits.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::EntropyClass;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn export_for(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn parent_row(path: &str, module: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: "Text".to_string(),
        kind: FileKind::Parent,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    }
}

fn child_row(path: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "(root)".to_string(),
        lang: "Text".to_string(),
        kind: FileKind::Child,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    }
}

fn write_repeated(path: &Path, byte: u8, len: usize) {
    fs::write(path, vec![byte; len]).unwrap();
}

fn write_pseudorandom(path: &Path, seed: u32, len: usize) {
    let mut data = Vec::with_capacity(len);
    let mut x = seed;
    for _ in 0..len {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x >> 16) as u8);
    }
    fs::write(path, data).unwrap();
}

// ── 1. All-zero file → Low ─────────────────────────────────────

#[test]
fn all_zeros_classified_low() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("zeros.bin");
    write_repeated(&f, 0x00, 1024);

    let export = export_for(vec![parent_row("zeros.bin", "(root)")]);
    let files = vec![PathBuf::from("zeros.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
    assert!(report.suspects[0].entropy_bits_per_byte < 0.01);
}

// ── 2. High-entropy pseudorandom → High ────────────────────────

#[test]
fn pseudorandom_classified_high() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("rand.bin");
    write_pseudorandom(&f, 0xDEADBEEF, 4096);

    let export = export_for(vec![parent_row("rand.bin", "(root)")]);
    let files = vec![PathBuf::from("rand.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::High);
    assert!(report.suspects[0].entropy_bits_per_byte > 7.5);
}

// ── 3. Empty file skipped (not in suspects) ────────────────────

#[test]
fn empty_file_not_a_suspect() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("empty.txt"), b"").unwrap();

    let export = export_for(vec![parent_row("empty.txt", "(root)")]);
    let files = vec![PathBuf::from("empty.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.is_empty(), "empty files should be skipped");
}

// ── 4. Single-byte file → Low ──────────────────────────────────

#[test]
fn single_byte_file_classified_low() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("one.bin"), [0xAB]).unwrap();

    let export = export_for(vec![parent_row("one.bin", "(root)")]);
    let files = vec![PathBuf::from("one.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
}

// ── 5. Normal text excluded from suspects ──────────────────────

#[test]
fn normal_english_text_not_in_suspects() {
    let dir = tempdir().unwrap();
    let text = "Hello world. This is a perfectly normal sentence. ".repeat(40);
    fs::write(dir.path().join("prose.txt"), text).unwrap();

    let export = export_for(vec![parent_row("prose.txt", "(root)")]);
    let files = vec![PathBuf::from("prose.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.is_empty(),
        "normal text should not be a suspect"
    );
}

// ── 6. Module mapping preserved in findings ────────────────────

#[test]
fn module_from_export_propagated_to_finding() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("key.bin");
    write_pseudorandom(&f, 0x1234, 2048);

    let export = export_for(vec![parent_row("key.bin", "secrets/keys")]);
    let files = vec![PathBuf::from("key.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].module, "secrets/keys");
}

// ── 7. File not in export gets "(unknown)" module ──────────────

#[test]
fn unmapped_file_gets_unknown_module() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("orphan.bin");
    write_pseudorandom(&f, 0xCAFE, 2048);

    let export = export_for(vec![]); // no rows at all
    let files = vec![PathBuf::from("orphan.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].module, "(unknown)");
}

// ── 8. Child rows in export are ignored for module lookup ──────

#[test]
fn child_rows_not_used_for_module_lookup() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("data.bin");
    write_pseudorandom(&f, 0xBEEF, 2048);

    // Only a Child row exists for this path — should not match
    let export = export_for(vec![child_row("data.bin")]);
    let files = vec![PathBuf::from("data.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(
        report.suspects[0].module, "(unknown)",
        "child rows should be filtered out"
    );
}

// ── 9. No files → empty report ─────────────────────────────────

#[test]
fn no_files_yields_empty_report() {
    let dir = tempdir().unwrap();
    let export = export_for(vec![]);
    let report =
        build_entropy_report(dir.path(), &[], &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.is_empty());
}

// ── 10. Suspects sorted descending by entropy ──────────────────

#[test]
fn suspects_ordered_highest_entropy_first() {
    let dir = tempdir().unwrap();
    // Low entropy
    let lo = dir.path().join("lo.bin");
    write_repeated(&lo, 0xFF, 1024);
    // High entropy
    let hi = dir.path().join("hi.bin");
    write_pseudorandom(&hi, 0xAAAA, 4096);

    let export = export_for(vec![
        parent_row("lo.bin", "(root)"),
        parent_row("hi.bin", "(root)"),
    ]);
    let files = vec![PathBuf::from("lo.bin"), PathBuf::from("hi.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.len() >= 2);
    assert!(
        report.suspects[0].entropy_bits_per_byte >= report.suspects[1].entropy_bits_per_byte,
        "first suspect should have higher entropy"
    );
}

// ── 11. Tie-breaking: same entropy → alphabetical path ─────────

#[test]
fn same_entropy_tiebreaks_by_path_ascending() {
    let dir = tempdir().unwrap();
    // Two identical low-entropy files
    let a = dir.path().join("beta.bin");
    let b = dir.path().join("alpha.bin");
    write_repeated(&a, 0x00, 512);
    write_repeated(&b, 0x00, 512);

    let export = export_for(vec![
        parent_row("alpha.bin", "(root)"),
        parent_row("beta.bin", "(root)"),
    ]);
    let files = vec![PathBuf::from("beta.bin"), PathBuf::from("alpha.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 2);
    assert_eq!(report.suspects[0].path, "alpha.bin");
    assert_eq!(report.suspects[1].path, "beta.bin");
}

// ── 12. MAX_SUSPECTS (50) truncation ───────────────────────────

#[test]
fn suspects_capped_at_fifty() {
    let dir = tempdir().unwrap();
    let mut rows = Vec::new();
    let mut files = Vec::new();
    for i in 0..60 {
        let name = format!("f{i:03}.bin");
        let f = dir.path().join(&name);
        write_pseudorandom(&f, i as u32, 2048);
        rows.push(parent_row(&name, "(root)"));
        files.push(PathBuf::from(name));
    }

    let export = export_for(rows);
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.len() <= 50,
        "should truncate to MAX_SUSPECTS=50, got {}",
        report.suspects.len()
    );
}

// ── 13. max_bytes budget halts scanning ────────────────────────

#[test]
fn max_bytes_budget_limits_scanning() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("first.bin");
    let b = dir.path().join("second.bin");
    write_pseudorandom(&a, 1, 512);
    write_pseudorandom(&b, 2, 512);

    let export = export_for(vec![
        parent_row("first.bin", "(root)"),
        parent_row("second.bin", "(root)"),
    ]);
    let files = vec![PathBuf::from("first.bin"), PathBuf::from("second.bin")];
    // Budget allows only the first file (512 bytes)
    let limits = AnalysisLimits {
        max_bytes: Some(512),
        ..AnalysisLimits::default()
    };
    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

    // At most 1 file should appear because budget is consumed after the first
    assert!(
        report.suspects.len() <= 1,
        "budget should stop after first file, got {} suspects",
        report.suspects.len()
    );
}

// ── 14. max_file_bytes limits sample size ──────────────────────

#[test]
fn max_file_bytes_limits_sample_size() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("big.bin");
    write_pseudorandom(&f, 0x7777, 8192);

    let export = export_for(vec![parent_row("big.bin", "(root)")]);
    let files = vec![PathBuf::from("big.bin")];
    let limits = AnalysisLimits {
        max_file_bytes: Some(128),
        ..AnalysisLimits::default()
    };
    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert!(
        report.suspects[0].sample_bytes <= 128,
        "sample_bytes should respect max_file_bytes, got {}",
        report.suspects[0].sample_bytes
    );
}

// ── 15. sample_bytes field matches actual bytes read ────────────

#[test]
fn sample_bytes_reflects_file_size_when_smaller_than_limit() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("small.bin");
    // Use a low-entropy file so it always appears as a suspect
    write_repeated(&f, 0x00, 64);

    let export = export_for(vec![parent_row("small.bin", "(root)")]);
    let files = vec![PathBuf::from("small.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(
        report.suspects[0].sample_bytes, 64,
        "sample_bytes should equal actual file size for small files"
    );
}
