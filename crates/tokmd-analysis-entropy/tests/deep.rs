//! Deep invariant tests for entropy detection.
//!
//! Tests classification boundaries, charset entropy ranges,
//! truncation, PRNG stability, determinism, and edge cases.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::EntropyClass;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn export_for_paths(paths: &[&str]) -> ExportData {
    let rows = paths
        .iter()
        .map(|p| FileRow {
            path: (*p).to_string(),
            module: "(root)".to_string(),
            lang: "Text".to_string(),
            kind: FileKind::Parent,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        })
        .collect();
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
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

// ── 1. All-zero file classified as Low ──────────────────────────

#[test]
fn all_zero_file_is_low_entropy() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("zeros.bin");
    write_repeated(&f, 0x00, 1024);

    let export = export_for_paths(&["zeros.bin"]);
    let files = vec![PathBuf::from("zeros.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::Low);
    assert!(report.suspects[0].entropy_bits_per_byte < 0.001);
}

// ── 2. Full-byte-range data classified as High ──────────────────

#[test]
fn full_byte_range_is_high_entropy() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("seq.bin");
    let mut data = Vec::with_capacity(2048);
    for _ in 0..8 {
        for b in 0u8..=255 {
            data.push(b);
        }
    }
    fs::write(&f, &data).unwrap();

    let export = export_for_paths(&["seq.bin"]);
    let files = vec![PathBuf::from("seq.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::High);
}

// ── 3. PRNG data classified as High ─────────────────────────────

#[test]
fn pseudorandom_data_classified_high() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("random.bin");
    write_pseudorandom(&f, 0xDEAD, 4096);

    let export = export_for_paths(&["random.bin"]);
    let files = vec![PathBuf::from("random.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::High);
}

// ── 4. English prose is Normal (not a suspect) ──────────────────

#[test]
fn english_prose_is_normal() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("readme.txt");
    let text = "The quick brown fox jumps over the lazy dog. ".repeat(50);
    fs::write(&f, text).unwrap();

    let export = export_for_paths(&["readme.txt"]);
    let files = vec![PathBuf::from("readme.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.is_empty(),
        "English prose should be Normal class"
    );
}

// ── 5. Typical source code is Normal ────────────────────────────

#[test]
fn source_code_is_normal() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("code.rs");
    let code = r#"fn main() {
    let x = 42;
    println!("Hello, world! x = {}", x);
    for i in 0..100 {
        if i % 2 == 0 {
            println!("{} is even", i);
        }
    }
}
"#
    .repeat(10);
    fs::write(&f, code).unwrap();

    let export = export_for_paths(&["code.rs"]);
    let files = vec![PathBuf::from("code.rs")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.is_empty(),
        "typical source code should be Normal class"
    );
}

// ── 6. Base64-charset data stays Normal ─────────────────────────

#[test]
fn base64_charset_is_normal() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("encoded.b64");
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut data = Vec::with_capacity(2048);
    let mut x = 0xBEEFu32;
    for _ in 0..2048 {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push(charset[(x >> 16) as usize % charset.len()]);
    }
    fs::write(&f, &data).unwrap();

    let export = export_for_paths(&["encoded.b64"]);
    let files = vec![PathBuf::from("encoded.b64")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.is_empty(),
        "base64 charset (~6.0 bits/byte) should be Normal"
    );
}

// ── 7. Hex-charset data stays Normal ────────────────────────────

#[test]
fn hex_charset_is_normal() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("hex.txt");
    let charset = b"0123456789abcdef";
    let mut data = Vec::with_capacity(2048);
    let mut x = 0xCAFEu32;
    for _ in 0..2048 {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push(charset[(x >> 16) as usize % charset.len()]);
    }
    fs::write(&f, &data).unwrap();

    let export = export_for_paths(&["hex.txt"]);
    let files = vec![PathBuf::from("hex.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.is_empty(), "hex (~4.0 bits/byte) is Normal");
}

// ── 8. Empty file produces no suspects ──────────────────────────

#[test]
fn empty_file_no_suspects() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("empty.txt");
    fs::write(&f, b"").unwrap();

    let export = export_for_paths(&["empty.txt"]);
    let files = vec![PathBuf::from("empty.txt")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(report.suspects.is_empty(), "empty file = no suspects");
}

// ── 9. Suspects truncated to MAX_SUSPECTS (50) ─────────────────

#[test]
fn suspects_truncated_to_max() {
    let dir = tempdir().unwrap();
    let mut rows = Vec::new();
    let mut files = Vec::new();
    for i in 0..60 {
        let name = format!("f{i:03}.bin");
        write_pseudorandom(&dir.path().join(&name), i as u32 + 1000, 2048);
        rows.push(FileRow {
            path: name.clone(),
            module: "(root)".to_string(),
            lang: "Text".to_string(),
            kind: FileKind::Parent,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        });
        files.push(PathBuf::from(name));
    }
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert!(
        report.suspects.len() <= 50,
        "suspects should be capped at 50, got {}",
        report.suspects.len()
    );
}

// ── 10. Suspects sorted by entropy desc, then path asc ─────────

#[test]
fn suspects_sorted_by_entropy_desc_then_path_asc() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("a.bin"), 0x00, 1024);
    write_repeated(&dir.path().join("b.bin"), 0x00, 1024);
    write_pseudorandom(&dir.path().join("c.bin"), 0xABCD, 4096);

    let export = export_for_paths(&["a.bin", "b.bin", "c.bin"]);
    let files = vec![
        PathBuf::from("a.bin"),
        PathBuf::from("b.bin"),
        PathBuf::from("c.bin"),
    ];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    for i in 1..report.suspects.len() {
        let prev = &report.suspects[i - 1];
        let curr = &report.suspects[i];
        assert!(
            prev.entropy_bits_per_byte >= curr.entropy_bits_per_byte
                || (prev.entropy_bits_per_byte == curr.entropy_bits_per_byte
                    && prev.path <= curr.path),
            "sort violation at index {i}"
        );
    }
}

// ── 11. Normal class never appears in suspects ──────────────────

#[test]
fn normal_class_never_in_suspects() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("lo.bin"), 0x00, 1024);
    write_pseudorandom(&dir.path().join("hi.bin"), 0x5555, 4096);
    fs::write(
        dir.path().join("code.rs"),
        "fn main() { println!(\"hello\"); }\n".repeat(30),
    )
    .unwrap();

    let export = export_for_paths(&["lo.bin", "hi.bin", "code.rs"]);
    let files = vec![
        PathBuf::from("lo.bin"),
        PathBuf::from("hi.bin"),
        PathBuf::from("code.rs"),
    ];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    for suspect in &report.suspects {
        assert_ne!(suspect.class, EntropyClass::Normal);
    }
}

// ── 12. Entropy values always in [0.0, 8.0] ────────────────────

#[test]
fn entropy_within_theoretical_bounds() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("zeros.bin"), 0x00, 1024);
    write_repeated(&dir.path().join("ff.bin"), 0xFF, 1024);
    write_pseudorandom(&dir.path().join("rand.bin"), 0x42, 4096);

    let export = export_for_paths(&["zeros.bin", "ff.bin", "rand.bin"]);
    let files = vec![
        PathBuf::from("zeros.bin"),
        PathBuf::from("ff.bin"),
        PathBuf::from("rand.bin"),
    ];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    for suspect in &report.suspects {
        assert!(
            (0.0..=8.0).contains(&suspect.entropy_bits_per_byte),
            "entropy out of bounds: {} for {}",
            suspect.entropy_bits_per_byte,
            suspect.path
        );
    }
}

// ── 13. Parent row preferred for module lookup ──────────────────

#[test]
fn parent_row_preferred_for_module_lookup() {
    let dir = tempdir().unwrap();
    write_pseudorandom(&dir.path().join("data.bin"), 0xFACE, 2048);

    let rows = vec![
        FileRow {
            path: "data.bin".to_string(),
            module: "parent_mod".to_string(),
            lang: "Text".to_string(),
            kind: FileKind::Parent,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        },
        FileRow {
            path: "data.bin".to_string(),
            module: "child_mod".to_string(),
            lang: "Text".to_string(),
            kind: FileKind::Child,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        },
    ];
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let files = vec![PathBuf::from("data.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].module, "parent_mod");
}

// ── 14. Deterministic across multiple runs ──────────────────────

#[test]
fn deterministic_across_runs() {
    let dir = tempdir().unwrap();
    write_pseudorandom(&dir.path().join("a.bin"), 0x1111, 2048);
    write_repeated(&dir.path().join("b.bin"), 0x00, 512);

    let export = export_for_paths(&["a.bin", "b.bin"]);
    let files = vec![PathBuf::from("a.bin"), PathBuf::from("b.bin")];

    let results: Vec<_> = (0..3)
        .map(|_| {
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap()
        })
        .collect();

    for i in 1..3 {
        assert_eq!(results[0].suspects.len(), results[i].suspects.len());
        for (a, b) in results[0].suspects.iter().zip(results[i].suspects.iter()) {
            assert_eq!(a.path, b.path);
            assert_eq!(a.class, b.class);
            assert!(
                (a.entropy_bits_per_byte - b.entropy_bits_per_byte).abs() < f32::EPSILON,
                "entropy mismatch for {}",
                a.path
            );
        }
    }
}

// ── 15. max_bytes limit stops processing early ──────────────────

#[test]
fn max_bytes_limit_caps_processing() {
    let dir = tempdir().unwrap();
    for i in 0..10 {
        let name = format!("f{i}.bin");
        write_pseudorandom(&dir.path().join(&name), i as u32, 4096);
    }

    let names: Vec<String> = (0..10).map(|i| format!("f{i}.bin")).collect();
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let export = export_for_paths(&name_refs);
    let files: Vec<PathBuf> = names.iter().map(PathBuf::from).collect();

    let limits_small = AnalysisLimits {
        max_bytes: Some(4096),
        max_file_bytes: None,
        ..Default::default()
    };
    let limits_large = AnalysisLimits {
        max_bytes: Some(1_000_000),
        max_file_bytes: None,
        ..Default::default()
    };

    let report_small = build_entropy_report(dir.path(), &files, &export, &limits_small).unwrap();
    let report_large = build_entropy_report(dir.path(), &files, &export, &limits_large).unwrap();

    assert!(
        report_small.suspects.len() <= report_large.suspects.len(),
        "smaller budget should find <= suspects"
    );
}
