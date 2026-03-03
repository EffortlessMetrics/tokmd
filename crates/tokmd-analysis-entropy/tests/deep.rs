//! Deep invariant tests for entropy detection.
//!
//! Focuses on classification boundary values, charset entropy ranges,
//! truncation boundaries, PRNG seed stability, and determinism.

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

// ── 1. Base64-charset data falls in Normal range (not suspect) ──

#[test]
fn base64_charset_falls_in_normal_range() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("secret.b64");
    // Base64 uses 64 chars → max entropy ≈ log2(64) = 6.0 bits/byte → Normal
    let base64_chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut data = Vec::with_capacity(2048);
    let mut x = 0xDEADBEEFu32;
    for _ in 0..2048 {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push(base64_chars[(x >> 16) as usize % base64_chars.len()]);
    }
    fs::write(&f, &data).unwrap();

    let export = export_for_paths(&["secret.b64"]);
    let files = vec![PathBuf::from("secret.b64")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    // 64-char alphabet → entropy ≈ 6.0 → Normal → filtered out of suspects
    assert!(
        report.suspects.is_empty(),
        "base64 charset data (entropy ≈ 6.0) should be Normal, not a suspect"
    );
}

// ── 2. Hex-charset data falls in Normal range (not suspect) ─────

#[test]
fn hex_charset_falls_in_normal_range() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("apikey.hex");
    // Hex uses 16 chars → max entropy ≈ log2(16) = 4.0 bits/byte → Normal
    let hex_chars = b"0123456789abcdef";
    let mut data = Vec::with_capacity(2048);
    let mut x = 0xCAFEBABEu32;
    for _ in 0..2048 {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push(hex_chars[(x >> 16) as usize % hex_chars.len()]);
    }
    fs::write(&f, &data).unwrap();

    let export = export_for_paths(&["apikey.hex"]);
    let files = vec![PathBuf::from("apikey.hex")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    // 16-char alphabet → entropy ≈ 4.0 → Normal → filtered out of suspects
    assert!(
        report.suspects.is_empty(),
        "hex charset data (entropy ≈ 4.0) should be Normal, not a suspect"
    );
}

// ── 3. Sequential bytes (0,1,2,...,255) repeated ────────────────

#[test]
fn incrementing_byte_sequence_classified_high() {
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

    // 256 equally-distributed byte values → entropy = 8.0 bits/byte → High
    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].class, EntropyClass::High);
}

// ── 4. Exactly 50 suspects: not truncated ───────────────────────

#[test]
fn exactly_fifty_suspects_not_truncated() {
    let dir = tempdir().unwrap();
    let mut rows = Vec::new();
    let mut files = Vec::new();
    // Create exactly 50 high-entropy files
    for i in 0..50 {
        let name = format!("f{i:03}.bin");
        let f = dir.path().join(&name);
        write_pseudorandom(&f, i as u32 + 1000, 2048);
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

    // All 50 should be present (MAX_SUSPECTS = 50, not exceeded)
    assert_eq!(
        report.suspects.len(),
        50,
        "exactly 50 suspects should not be truncated"
    );
}

// ── 5. 51 suspects → truncated to 50 ───────────────────────────

#[test]
fn fifty_one_suspects_truncated_to_fifty() {
    let dir = tempdir().unwrap();
    let mut rows = Vec::new();
    let mut files = Vec::new();
    for i in 0..51 {
        let name = format!("f{i:03}.bin");
        let f = dir.path().join(&name);
        write_pseudorandom(&f, i as u32 + 2000, 2048);
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

    assert_eq!(
        report.suspects.len(),
        50,
        "51 suspects should be truncated to MAX_SUSPECTS=50"
    );
}

// ── 6. Multiple seeds produce consistent high-entropy classification

#[test]
fn different_prng_seeds_all_classify_high() {
    let dir = tempdir().unwrap();
    let seeds = [0xAAAAu32, 0xBBBB, 0xCCCC, 0xDDDD, 0xEEEE];
    let mut rows = Vec::new();
    let mut files = Vec::new();

    for (i, &seed) in seeds.iter().enumerate() {
        let name = format!("s{i}.bin");
        let f = dir.path().join(&name);
        write_pseudorandom(&f, seed, 4096);
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

    assert_eq!(report.suspects.len(), 5);
    for suspect in &report.suspects {
        assert_eq!(
            suspect.class,
            EntropyClass::High,
            "seed-varied PRNG data should all be High, {} was {:?}",
            suspect.path,
            suspect.class
        );
    }
}

// ── 7. Classification boundary: all-same bytes → Low ────────────

#[test]
fn classification_boundary_all_same_bytes_is_low() {
    let dir = tempdir().unwrap();
    // Test multiple single-byte values
    for byte in [0x00u8, 0x41, 0xFF, 0x7F] {
        let name = format!("byte_{byte:02x}.bin");
        let f = dir.path().join(&name);
        write_repeated(&f, byte, 1024);

        let export = export_for_paths(&[&name]);
        let files = vec![PathBuf::from(&name)];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(
            report.suspects[0].class,
            EntropyClass::Low,
            "all-same-byte file (0x{byte:02x}) should be Low"
        );
    }
}

// ── 8. Module mapping for files in subdirectories ───────────────

#[test]
fn subdirectory_file_maps_to_correct_module() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("sub").join("dir");
    fs::create_dir_all(&sub).unwrap();
    let f = sub.join("secret.bin");
    write_pseudorandom(&f, 0x1234, 2048);

    let rows = vec![FileRow {
        path: "sub/dir/secret.bin".to_string(),
        module: "sub/dir".to_string(),
        lang: "Text".to_string(),
        kind: FileKind::Parent,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    }];
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let files = vec![PathBuf::from("sub/dir/secret.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert_eq!(report.suspects[0].module, "sub/dir");
    assert_eq!(report.suspects[0].path, "sub/dir/secret.bin");
}

// ── 9. max_bytes budget: zero budget scans nothing ──────────────

#[test]
fn zero_max_bytes_budget_scans_nothing() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("data.bin");
    write_pseudorandom(&f, 0xABCD, 1024);

    let export = export_for_paths(&["data.bin"]);
    let files = vec![PathBuf::from("data.bin")];
    let limits = AnalysisLimits {
        max_bytes: Some(0),
        ..AnalysisLimits::default()
    };
    let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

    assert!(
        report.suspects.is_empty(),
        "zero budget should scan nothing"
    );
}

// ── 10. Determinism: same file, three runs, identical output ────

#[test]
fn three_runs_produce_identical_results() {
    let dir = tempdir().unwrap();

    let lo = dir.path().join("lo.bin");
    let hi = dir.path().join("hi.bin");
    write_repeated(&lo, 0x00, 1024);
    write_pseudorandom(&hi, 0x9999, 4096);

    let export = export_for_paths(&["lo.bin", "hi.bin"]);
    let files = vec![PathBuf::from("lo.bin"), PathBuf::from("hi.bin")];

    let results: Vec<_> = (0..3)
        .map(|_| {
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap()
        })
        .collect();

    for i in 1..3 {
        assert_eq!(results[0].suspects.len(), results[i].suspects.len());
        for (s0, si) in results[0].suspects.iter().zip(results[i].suspects.iter()) {
            assert_eq!(s0.path, si.path);
            assert_eq!(s0.class, si.class);
            assert!(
                (s0.entropy_bits_per_byte - si.entropy_bits_per_byte).abs() < f32::EPSILON,
                "entropy should be identical across runs"
            );
            assert_eq!(s0.sample_bytes, si.sample_bytes);
            assert_eq!(s0.module, si.module);
        }
    }
}

// ── 11. Entropy value for all-zero file is exactly 0.0 ──────────

#[test]
fn all_zero_file_has_zero_entropy() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("zeros.bin");
    write_repeated(&f, 0x00, 2048);

    let export = export_for_paths(&["zeros.bin"]);
    let files = vec![PathBuf::from("zeros.bin")];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 1);
    assert!(
        report.suspects[0].entropy_bits_per_byte < 0.001,
        "all-zero file should have ~0.0 entropy, got {}",
        report.suspects[0].entropy_bits_per_byte
    );
}

// ── 12. Mixed parent and child rows: only parent used for lookup ─

#[test]
fn parent_row_preferred_over_child_for_module_lookup() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("data.bin");
    write_pseudorandom(&f, 0xFACE, 2048);

    let rows = vec![
        FileRow {
            path: "data.bin".to_string(),
            module: "parent_module".to_string(),
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
            module: "child_module".to_string(),
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
    assert_eq!(
        report.suspects[0].module, "parent_module",
        "should use parent row's module, not child row's"
    );
}

// ── 13. Entropy values are always in [0.0, 8.0] ────────────────

#[test]
fn entropy_values_within_theoretical_bounds() {
    let dir = tempdir().unwrap();

    let names = ["zeros.bin", "ones.bin", "random.bin", "text.txt"];
    write_repeated(&dir.path().join("zeros.bin"), 0x00, 1024);
    write_repeated(&dir.path().join("ones.bin"), 0xFF, 1024);
    write_pseudorandom(&dir.path().join("random.bin"), 0x1234, 4096);
    fs::write(dir.path().join("text.txt"), "hello world ".repeat(100)).unwrap();

    let export = export_for_paths(&names);
    let files: Vec<PathBuf> = names.iter().map(PathBuf::from).collect();
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    for suspect in &report.suspects {
        assert!(
            suspect.entropy_bits_per_byte >= 0.0 && suspect.entropy_bits_per_byte <= 8.0,
            "entropy should be in [0, 8], got {} for {}",
            suspect.entropy_bits_per_byte,
            suspect.path
        );
    }
}

// ── 14. Suspect sort stability: deterministic for equal entropy ──

#[test]
fn suspects_with_equal_entropy_sorted_by_path() {
    let dir = tempdir().unwrap();
    // Create multiple files with identical content (same entropy)
    for name in &["c.bin", "a.bin", "b.bin"] {
        let f = dir.path().join(name);
        write_repeated(&f, 0x00, 512);
    }

    let export = export_for_paths(&["c.bin", "a.bin", "b.bin"]);
    let files = vec![
        PathBuf::from("c.bin"),
        PathBuf::from("a.bin"),
        PathBuf::from("b.bin"),
    ];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    assert_eq!(report.suspects.len(), 3);
    // All have same entropy (0.0), so should be sorted alphabetically
    assert_eq!(report.suspects[0].path, "a.bin");
    assert_eq!(report.suspects[1].path, "b.bin");
    assert_eq!(report.suspects[2].path, "c.bin");
}

// ── 15. Normal class never appears in suspects list ─────────────

#[test]
fn normal_class_never_in_suspects() {
    let dir = tempdir().unwrap();
    // Create files across all entropy ranges
    let lo = dir.path().join("lo.bin");
    let hi = dir.path().join("hi.bin");
    let normal = dir.path().join("code.rs");

    write_repeated(&lo, 0x00, 1024);
    write_pseudorandom(&hi, 0x5555, 4096);
    let code = r#"fn main() {
    let x = 42;
    println!("Hello, world! x = {}", x);
    for i in 0..10 {
        println!("{}", i);
    }
}
"#
    .repeat(10);
    fs::write(&normal, code).unwrap();

    let export = export_for_paths(&["lo.bin", "hi.bin", "code.rs"]);
    let files = vec![
        PathBuf::from("lo.bin"),
        PathBuf::from("hi.bin"),
        PathBuf::from("code.rs"),
    ];
    let report =
        build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

    for suspect in &report.suspects {
        assert_ne!(
            suspect.class,
            EntropyClass::Normal,
            "Normal class should never appear in suspects: {} is {:?}",
            suspect.path,
            suspect.class
        );
    }
}
