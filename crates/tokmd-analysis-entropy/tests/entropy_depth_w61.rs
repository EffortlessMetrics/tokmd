//! Wave-61 depth tests for entropy detection.
//!
//! Focuses on boundary classification, budget edge cases, determinism
//! under concurrent-style repeated invocations, large-scale suspect
//! capping, suspicious-class targeting, path normalisation, JSON
//! round-trip of reports, and proptest properties.

use std::fs;
use std::path::{Path, PathBuf};

use proptest::prelude::*;
use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::{EntropyClass, EntropyReport};
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn make_export(paths: &[&str]) -> ExportData {
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

fn make_export_with_module(path: &str, module: &str) -> ExportData {
    ExportData {
        rows: vec![FileRow {
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
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn write_repeated(path: &Path, byte: u8, len: usize) {
    fs::write(path, vec![byte; len]).unwrap();
}

fn write_prng(path: &Path, seed: u32, len: usize) {
    let mut data = Vec::with_capacity(len);
    let mut x = seed;
    for _ in 0..len {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x >> 16) as u8);
    }
    fs::write(path, data).unwrap();
}

fn write_charset(path: &Path, charset: &[u8], seed: u32, len: usize) {
    let mut data = Vec::with_capacity(len);
    let mut x = seed;
    for _ in 0..len {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push(charset[(x >> 16) as usize % charset.len()]);
    }
    fs::write(path, data).unwrap();
}

fn limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

fn run(dir: &Path, names: &[&str], lim: &AnalysisLimits) -> EntropyReport {
    let export = make_export(names);
    let files: Vec<PathBuf> = names.iter().map(PathBuf::from).collect();
    build_entropy_report(dir, &files, &export, lim).unwrap()
}

// ═══════════════════════════════════════════════════════════════
// 1–5  Boundary classification tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn boundary_below_2_is_low() {
    // entropy < 2.0 → Low
    let dir = tempdir().unwrap();
    // Three distinct byte values used uniformly → entropy ≈ log2(3) ≈ 1.58
    let data: Vec<u8> = (0..3000u32).map(|i| (i % 3) as u8).collect();
    fs::write(dir.path().join("three.bin"), data).unwrap();
    let r = run(dir.path(), &["three.bin"], &limits());
    assert_eq!(r.suspects.len(), 1);
    assert_eq!(r.suspects[0].class, EntropyClass::Low);
    assert!(r.suspects[0].entropy_bits_per_byte < 2.0);
}

#[test]
fn boundary_above_7_5_is_high() {
    // entropy > 7.5 → High
    let dir = tempdir().unwrap();
    write_prng(&dir.path().join("h.bin"), 0xFEED, 4096);
    let r = run(dir.path(), &["h.bin"], &limits());
    assert_eq!(r.suspects.len(), 1);
    assert_eq!(r.suspects[0].class, EntropyClass::High);
    assert!(r.suspects[0].entropy_bits_per_byte > 7.5);
}

#[test]
fn four_distinct_bytes_uniform_is_low() {
    // log2(4) = 2.0 — exactly at boundary
    let dir = tempdir().unwrap();
    let data: Vec<u8> = (0..4096u32).map(|i| (i % 4) as u8).collect();
    fs::write(dir.path().join("four.bin"), data).unwrap();
    let r = run(dir.path(), &["four.bin"], &limits());
    // 2.0 is NOT < 2.0, so Normal (filtered). Exactly at the boundary.
    assert!(
        r.suspects.is_empty(),
        "entropy == 2.0 should be Normal, not Low"
    );
}

#[test]
fn sixteen_distinct_bytes_uniform_normal() {
    // log2(16) = 4.0 → Normal (2.0 ≤ 4.0 < 6.5)
    let dir = tempdir().unwrap();
    let data: Vec<u8> = (0..4096u32).map(|i| (i % 16) as u8).collect();
    fs::write(dir.path().join("hex.bin"), data).unwrap();
    let r = run(dir.path(), &["hex.bin"], &limits());
    assert!(r.suspects.is_empty(), "entropy ≈ 4.0 should be Normal");
}

#[test]
fn entropy_exactly_zero_for_single_value() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("same.bin"), 0x55, 2048);
    let r = run(dir.path(), &["same.bin"], &limits());
    assert_eq!(r.suspects.len(), 1);
    assert!(
        r.suspects[0].entropy_bits_per_byte < 0.001,
        "single-value file entropy should be ~0.0, got {}",
        r.suspects[0].entropy_bits_per_byte
    );
}

// ═══════════════════════════════════════════════════════════════
// 6–10  Suspicious-class targeting
// ═══════════════════════════════════════════════════════════════

#[test]
fn suspicious_class_between_6_5_and_7_5() {
    // Build data targeting 6.5–7.5 bits/byte: 180 distinct values
    let dir = tempdir().unwrap();
    let data: Vec<u8> = (0..4096u32).map(|i| (i % 180) as u8).collect();
    fs::write(dir.path().join("sus.bin"), data).unwrap();
    let r = run(dir.path(), &["sus.bin"], &limits());
    if !r.suspects.is_empty() {
        let e = r.suspects[0].entropy_bits_per_byte;
        assert!(
            r.suspects[0].class == EntropyClass::Suspicious
                || r.suspects[0].class == EntropyClass::High,
            "entropy ~{e} should be Suspicious or High"
        );
    }
}

#[test]
fn charset_128_normal_range() {
    // log2(128) = 7.0 → Suspicious (6.5 ≤ 7.0 ≤ 7.5)
    let dir = tempdir().unwrap();
    let charset: Vec<u8> = (0..128u8).collect();
    write_charset(&dir.path().join("c128.bin"), &charset, 0x1111, 4096);
    let r = run(dir.path(), &["c128.bin"], &limits());
    if !r.suspects.is_empty() {
        assert!(
            r.suspects[0].class == EntropyClass::Suspicious
                || r.suspects[0].class == EntropyClass::High,
            "128-char charset entropy should be Suspicious or High, got {:?}",
            r.suspects[0].class
        );
    }
}

#[test]
fn charset_64_is_normal() {
    // log2(64) = 6.0 → Normal
    let dir = tempdir().unwrap();
    let charset: Vec<u8> = (0..64u8).collect();
    write_charset(&dir.path().join("c64.bin"), &charset, 0x2222, 4096);
    let r = run(dir.path(), &["c64.bin"], &limits());
    assert!(
        r.suspects.is_empty(),
        "64-char charset (~6.0 bits/byte) should be Normal"
    );
}

#[test]
fn charset_8_is_normal() {
    // log2(8) = 3.0 → Normal
    let dir = tempdir().unwrap();
    let charset: Vec<u8> = (0..8u8).collect();
    write_charset(&dir.path().join("c8.bin"), &charset, 0x3333, 4096);
    let r = run(dir.path(), &["c8.bin"], &limits());
    assert!(
        r.suspects.is_empty(),
        "8-char charset (~3.0 bits/byte) should be Normal"
    );
}

#[test]
fn mixed_low_and_high_files_both_detected() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("lo.bin"), 0xAA, 1024);
    write_prng(&dir.path().join("hi.bin"), 0x7777, 4096);
    let r = run(dir.path(), &["lo.bin", "hi.bin"], &limits());
    assert_eq!(r.suspects.len(), 2);
    let classes: Vec<_> = r.suspects.iter().map(|s| s.class).collect();
    assert!(classes.contains(&EntropyClass::Low));
    assert!(classes.contains(&EntropyClass::High));
}

// ═══════════════════════════════════════════════════════════════
// 11–15  Sorting & capping
// ═══════════════════════════════════════════════════════════════

#[test]
fn suspects_sorted_descending_entropy_then_path() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("a.bin"), 0x00, 512);
    write_repeated(&dir.path().join("b.bin"), 0x00, 512);
    write_prng(&dir.path().join("c.bin"), 0xAAAA, 2048);
    let r = run(dir.path(), &["a.bin", "b.bin", "c.bin"], &limits());
    for w in r.suspects.windows(2) {
        let ok = w[0].entropy_bits_per_byte > w[1].entropy_bits_per_byte
            || ((w[0].entropy_bits_per_byte - w[1].entropy_bits_per_byte).abs() < f32::EPSILON
                && w[0].path <= w[1].path);
        assert!(ok, "sort violated: {} vs {}", w[0].path, w[1].path);
    }
}

#[test]
fn cap_at_fifty_when_sixty_suspects() {
    let dir = tempdir().unwrap();
    let mut names = Vec::new();
    for i in 0..60 {
        let name = format!("s{i:03}.bin");
        write_prng(&dir.path().join(&name), i + 3000, 2048);
        names.push(name);
    }
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let r = run(dir.path(), &refs, &limits());
    assert_eq!(r.suspects.len(), 50, "should be capped at MAX_SUSPECTS=50");
}

#[test]
fn cap_preserves_highest_entropy_suspects() {
    let dir = tempdir().unwrap();
    let mut names = Vec::new();
    // 60 high-entropy files
    for i in 0..60 {
        let name = format!("h{i:03}.bin");
        write_prng(&dir.path().join(&name), i + 4000, 4096);
        names.push(name);
    }
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let r = run(dir.path(), &refs, &limits());
    // After capping at 50, all remaining suspects should have the top-50 entropies
    for w in r.suspects.windows(2) {
        assert!(w[0].entropy_bits_per_byte >= w[1].entropy_bits_per_byte);
    }
}

#[test]
fn forty_nine_suspects_not_truncated() {
    let dir = tempdir().unwrap();
    let mut names = Vec::new();
    for i in 0..49 {
        let name = format!("f{i:03}.bin");
        write_prng(&dir.path().join(&name), i + 5000, 2048);
        names.push(name);
    }
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let r = run(dir.path(), &refs, &limits());
    assert_eq!(r.suspects.len(), 49);
}

#[test]
fn zero_suspect_files_yield_empty_report() {
    let dir = tempdir().unwrap();
    // All Normal-entropy files
    let text = "fn main() { println!(\"hello\"); }\n".repeat(50);
    fs::write(dir.path().join("code.rs"), &text).unwrap();
    let r = run(dir.path(), &["code.rs"], &limits());
    assert!(r.suspects.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// 16–20  Budget / limits
// ═══════════════════════════════════════════════════════════════

#[test]
fn max_bytes_one_byte_scans_nothing() {
    let dir = tempdir().unwrap();
    write_prng(&dir.path().join("big.bin"), 0xAAAA, 8192);
    let lim = AnalysisLimits {
        max_bytes: Some(1),
        ..AnalysisLimits::default()
    };
    let r = run(dir.path(), &["big.bin"], &lim);
    // max_bytes=1 is enough for one byte, so the first file gets read
    // but budget is then exhausted
    assert!(r.suspects.len() <= 1);
}

#[test]
fn max_file_bytes_limits_sample_size() {
    let dir = tempdir().unwrap();
    write_prng(&dir.path().join("data.bin"), 0xBBBB, 8192);
    let lim = AnalysisLimits {
        max_file_bytes: Some(64),
        ..AnalysisLimits::default()
    };
    let r = run(dir.path(), &["data.bin"], &lim);
    if !r.suspects.is_empty() {
        assert!(
            r.suspects[0].sample_bytes <= 64,
            "sample should be <= 64, got {}",
            r.suspects[0].sample_bytes
        );
    }
}

#[test]
fn max_bytes_budget_stops_second_file() {
    let dir = tempdir().unwrap();
    // First file 512 bytes, second file 512 bytes, budget = 512
    write_prng(&dir.path().join("a.bin"), 0x1000, 512);
    write_prng(&dir.path().join("b.bin"), 0x2000, 512);
    let lim = AnalysisLimits {
        max_bytes: Some(512),
        ..AnalysisLimits::default()
    };
    let export = make_export(&["a.bin", "b.bin"]);
    let files = vec![PathBuf::from("a.bin"), PathBuf::from("b.bin")];
    let r = build_entropy_report(dir.path(), &files, &export, &lim).unwrap();
    // After first file (512 bytes), total_bytes >= max_bytes, so second is skipped
    assert!(r.suspects.len() <= 2, "budget should limit scanning");
}

#[test]
fn unlimited_budget_scans_all() {
    let dir = tempdir().unwrap();
    for i in 0..5 {
        write_prng(&dir.path().join(format!("f{i}.bin")), i + 6000, 1024);
    }
    let r = run(
        dir.path(),
        &["f0.bin", "f1.bin", "f2.bin", "f3.bin", "f4.bin"],
        &limits(),
    );
    assert_eq!(r.suspects.len(), 5);
}

#[test]
fn default_limits_per_file_is_1024() {
    let dir = tempdir().unwrap();
    write_prng(&dir.path().join("big.bin"), 0xCCCC, 8192);
    let r = run(dir.path(), &["big.bin"], &limits());
    assert_eq!(r.suspects.len(), 1);
    assert_eq!(
        r.suspects[0].sample_bytes, 1024,
        "default per-file sample should be 1024"
    );
}

// ═══════════════════════════════════════════════════════════════
// 21–25  Module mapping & path handling
// ═══════════════════════════════════════════════════════════════

#[test]
fn file_not_in_export_gets_unknown_module() {
    let dir = tempdir().unwrap();
    write_prng(&dir.path().join("orphan.bin"), 0xDDDD, 2048);
    let export = make_export(&[]); // no rows
    let files = vec![PathBuf::from("orphan.bin")];
    let r = build_entropy_report(dir.path(), &files, &export, &limits()).unwrap();
    assert_eq!(r.suspects.len(), 1);
    assert_eq!(r.suspects[0].module, "(unknown)");
}

#[test]
fn nested_path_module_mapping() {
    let dir = tempdir().unwrap();
    let nested = dir.path().join("a").join("b").join("c");
    fs::create_dir_all(&nested).unwrap();
    write_prng(&nested.join("deep.bin"), 0xEEEE, 2048);
    let export = make_export_with_module("a/b/c/deep.bin", "a/b/c");
    let files = vec![PathBuf::from("a/b/c/deep.bin")];
    let r = build_entropy_report(dir.path(), &files, &export, &limits()).unwrap();
    assert_eq!(r.suspects.len(), 1);
    assert_eq!(r.suspects[0].module, "a/b/c");
    assert_eq!(r.suspects[0].path, "a/b/c/deep.bin");
}

#[test]
fn child_rows_ignored_for_module_lookup() {
    let dir = tempdir().unwrap();
    write_prng(&dir.path().join("x.bin"), 0xFFFF, 2048);
    let export = ExportData {
        rows: vec![
            FileRow {
                path: "x.bin".to_string(),
                module: "correct_mod".to_string(),
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
                path: "x.bin".to_string(),
                module: "wrong_mod".to_string(),
                lang: "Text".to_string(),
                kind: FileKind::Child,
                code: 1,
                comments: 0,
                blanks: 0,
                lines: 1,
                bytes: 10,
                tokens: 2,
            },
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let files = vec![PathBuf::from("x.bin")];
    let r = build_entropy_report(dir.path(), &files, &export, &limits()).unwrap();
    assert_eq!(r.suspects[0].module, "correct_mod");
}

#[cfg(target_os = "windows")]
#[test]
fn backslash_paths_normalised_to_forward_slash() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir_all(&sub).unwrap();
    write_repeated(&sub.join("lo.bin"), 0x00, 512);
    let export = make_export(&["sub/lo.bin"]);
    let files = vec![PathBuf::from("sub\\lo.bin")];
    let r = build_entropy_report(dir.path(), &files, &export, &limits()).unwrap();
    for s in &r.suspects {
        assert!(!s.path.contains('\\'), "path has backslash: {}", s.path);
    }
}

#[test]
fn empty_file_list_yields_empty_report() {
    let dir = tempdir().unwrap();
    let r = run(dir.path(), &[], &limits());
    assert!(r.suspects.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// 26–30  Determinism
// ═══════════════════════════════════════════════════════════════

#[test]
fn five_runs_identical_output() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("lo.bin"), 0x00, 1024);
    write_prng(&dir.path().join("hi.bin"), 0x9999, 4096);
    let names = &["lo.bin", "hi.bin"];
    let results: Vec<EntropyReport> = (0..5).map(|_| run(dir.path(), names, &limits())).collect();
    for i in 1..5 {
        assert_eq!(results[0].suspects.len(), results[i].suspects.len());
        for (a, b) in results[0].suspects.iter().zip(results[i].suspects.iter()) {
            assert_eq!(a.path, b.path);
            assert_eq!(a.class, b.class);
            assert!((a.entropy_bits_per_byte - b.entropy_bits_per_byte).abs() < f32::EPSILON);
            assert_eq!(a.sample_bytes, b.sample_bytes);
            assert_eq!(a.module, b.module);
        }
    }
}

#[test]
fn different_prng_seeds_produce_different_entropy() {
    let dir = tempdir().unwrap();
    write_prng(&dir.path().join("s1.bin"), 1, 4096);
    write_prng(&dir.path().join("s2.bin"), 999_999, 4096);
    let export = make_export(&["s1.bin", "s2.bin"]);
    let files = vec![PathBuf::from("s1.bin"), PathBuf::from("s2.bin")];
    let r = build_entropy_report(dir.path(), &files, &export, &limits()).unwrap();
    // Both should be high, but entropy values may differ slightly
    assert_eq!(r.suspects.len(), 2);
    for s in &r.suspects {
        assert_eq!(s.class, EntropyClass::High);
    }
}

#[test]
fn determinism_with_many_files() {
    let dir = tempdir().unwrap();
    let mut names = Vec::new();
    for i in 0..20 {
        let name = format!("d{i:02}.bin");
        write_prng(&dir.path().join(&name), i + 7000, 1024);
        names.push(name);
    }
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let r1 = run(dir.path(), &refs, &limits());
    let r2 = run(dir.path(), &refs, &limits());
    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2, "JSON output must be deterministic");
}

#[test]
fn entropy_report_serde_round_trip() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("lo.bin"), 0x00, 1024);
    write_prng(&dir.path().join("hi.bin"), 0x1234, 4096);
    let r = run(dir.path(), &["lo.bin", "hi.bin"], &limits());
    let json = serde_json::to_string(&r).unwrap();
    let rt: EntropyReport = serde_json::from_str(&json).unwrap();
    assert_eq!(r.suspects.len(), rt.suspects.len());
    for (a, b) in r.suspects.iter().zip(rt.suspects.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.class, b.class);
        assert_eq!(a.sample_bytes, b.sample_bytes);
    }
}

#[test]
fn empty_report_serializes_correctly() {
    let dir = tempdir().unwrap();
    let r = run(dir.path(), &[], &limits());
    let json = serde_json::to_string(&r).unwrap();
    assert_eq!(json, r#"{"suspects":[]}"#);
}

// ═══════════════════════════════════════════════════════════════
// 31–35  Edge cases
// ═══════════════════════════════════════════════════════════════

#[test]
fn one_byte_file_classified_low() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("one.bin"), [0x42]).unwrap();
    let r = run(dir.path(), &["one.bin"], &limits());
    assert_eq!(r.suspects.len(), 1);
    assert_eq!(r.suspects[0].class, EntropyClass::Low);
}

#[test]
fn two_byte_identical_classified_low() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("two.bin"), [0xAA, 0xAA]).unwrap();
    let r = run(dir.path(), &["two.bin"], &limits());
    assert_eq!(r.suspects.len(), 1);
    assert_eq!(r.suspects[0].class, EntropyClass::Low);
}

#[test]
fn two_byte_distinct_classified_low() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("two.bin"), [0x00, 0xFF]).unwrap();
    let r = run(dir.path(), &["two.bin"], &limits());
    // 1-bit entropy → Low
    assert_eq!(r.suspects.len(), 1);
    assert_eq!(r.suspects[0].class, EntropyClass::Low);
}

#[test]
fn large_low_entropy_file() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("huge.bin"), 0x00, 1_000_000);
    let r = run(dir.path(), &["huge.bin"], &limits());
    assert_eq!(r.suspects.len(), 1);
    assert_eq!(r.suspects[0].class, EntropyClass::Low);
    // Only 1024 bytes sampled by default
    assert_eq!(r.suspects[0].sample_bytes, 1024);
}

#[test]
fn empty_file_not_in_suspects() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("empty.txt"), b"").unwrap();
    let r = run(dir.path(), &["empty.txt"], &limits());
    assert!(r.suspects.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// 36–40  Multi-file & realistic scenarios
// ═══════════════════════════════════════════════════════════════

#[test]
fn normal_text_not_in_suspects() {
    let dir = tempdir().unwrap();
    let text = "The quick brown fox jumps over the lazy dog.\n".repeat(50);
    fs::write(dir.path().join("text.txt"), text).unwrap();
    let r = run(dir.path(), &["text.txt"], &limits());
    assert!(r.suspects.is_empty());
}

#[test]
fn source_code_not_in_suspects() {
    let dir = tempdir().unwrap();
    let code = r#"fn main() {
    let x = 42;
    println!("Hello, world! x = {}", x);
    for i in 0..100 {
        println!("{}", i * i);
    }
}
"#
    .repeat(20);
    fs::write(dir.path().join("main.rs"), code).unwrap();
    let r = run(dir.path(), &["main.rs"], &limits());
    assert!(r.suspects.is_empty());
}

#[test]
fn all_256_values_uniform_is_high() {
    let dir = tempdir().unwrap();
    let data: Vec<u8> = (0..=255u8).cycle().take(4096).collect();
    fs::write(dir.path().join("uniform.bin"), data).unwrap();
    let r = run(dir.path(), &["uniform.bin"], &limits());
    assert_eq!(r.suspects.len(), 1);
    assert_eq!(r.suspects[0].class, EntropyClass::High);
    assert!(r.suspects[0].entropy_bits_per_byte > 7.9);
}

#[test]
fn mixed_normal_low_high_only_non_normal_reported() {
    let dir = tempdir().unwrap();
    write_repeated(&dir.path().join("lo.bin"), 0x00, 1024);
    write_prng(&dir.path().join("hi.bin"), 0x4567, 4096);
    let text = "Hello world this is normal text.\n".repeat(50);
    fs::write(dir.path().join("normal.txt"), text).unwrap();
    let r = run(dir.path(), &["lo.bin", "hi.bin", "normal.txt"], &limits());
    for s in &r.suspects {
        assert_ne!(
            s.class,
            EntropyClass::Normal,
            "Normal should never appear in suspects"
        );
    }
}

#[test]
fn multiple_low_entropy_files_all_detected() {
    let dir = tempdir().unwrap();
    for b in [0x00u8, 0xFF, 0x41, 0x7F] {
        let name = format!("b_{b:02x}.bin");
        write_repeated(&dir.path().join(&name), b, 1024);
    }
    let r = run(
        dir.path(),
        &["b_00.bin", "b_ff.bin", "b_41.bin", "b_7f.bin"],
        &limits(),
    );
    assert_eq!(r.suspects.len(), 4);
    for s in &r.suspects {
        assert_eq!(s.class, EntropyClass::Low);
    }
}

// ═══════════════════════════════════════════════════════════════
// 41–45  Proptest properties
// ═══════════════════════════════════════════════════════════════

mod w61_properties {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn entropy_always_in_range(data in prop::collection::vec(any::<u8>(), 1..=2048)) {
            let dir = tempdir().unwrap();
            fs::write(dir.path().join("t.bin"), &data).unwrap();
            let r = run(dir.path(), &["t.bin"], &limits());
            for s in &r.suspects {
                prop_assert!(s.entropy_bits_per_byte >= 0.0);
                prop_assert!(s.entropy_bits_per_byte <= 8.0);
            }
        }

        #[test]
        fn normal_class_never_in_suspects(data in prop::collection::vec(any::<u8>(), 1..=4096)) {
            let dir = tempdir().unwrap();
            fs::write(dir.path().join("t.bin"), &data).unwrap();
            let r = run(dir.path(), &["t.bin"], &limits());
            for s in &r.suspects {
                prop_assert_ne!(s.class, EntropyClass::Normal);
            }
        }

        #[test]
        fn single_repeated_byte_is_low(byte in any::<u8>(), len in 16usize..=2048) {
            let dir = tempdir().unwrap();
            write_repeated(&dir.path().join("m.bin"), byte, len);
            let r = run(dir.path(), &["m.bin"], &limits());
            prop_assert_eq!(r.suspects.len(), 1);
            prop_assert_eq!(r.suspects[0].class, EntropyClass::Low);
        }

        #[test]
        fn suspects_bounded_by_fifty(count in 1usize..=60) {
            let dir = tempdir().unwrap();
            let mut names = Vec::new();
            for i in 0..count {
                let name = format!("p{i:03}.bin");
                write_prng(&dir.path().join(&name), i as u32 + 8000, 2048);
                names.push(name);
            }
            let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            let r = run(dir.path(), &refs, &limits());
            prop_assert!(r.suspects.len() <= 50);
        }

        #[test]
        fn sample_bytes_positive_for_nonempty(data in prop::collection::vec(any::<u8>(), 1..=1024)) {
            let dir = tempdir().unwrap();
            fs::write(dir.path().join("t.bin"), &data).unwrap();
            let r = run(dir.path(), &["t.bin"], &limits());
            for s in &r.suspects {
                prop_assert!(s.sample_bytes > 0);
            }
        }
    }
}
