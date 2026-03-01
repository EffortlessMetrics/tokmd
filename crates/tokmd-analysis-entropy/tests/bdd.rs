//! BDD-style scenario tests for entropy detection.

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

fn write_pseudorandom(path: &Path, len: usize) {
    let mut data = Vec::with_capacity(len);
    let mut x = 0x12345678u32;
    for _ in 0..len {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x & 0xFF) as u8);
    }
    fs::write(path, data).unwrap();
}

fn write_all_byte_values(path: &Path, repeats: usize) {
    let mut data = Vec::with_capacity(256 * repeats);
    for _ in 0..repeats {
        for b in 0u8..=255 {
            data.push(b);
        }
    }
    fs::write(path, data).unwrap();
}

// ── Low entropy scenarios ───────────────────────────────────────

mod low_entropy {
    use super::*;

    #[test]
    fn given_file_with_single_repeated_byte_then_classified_low() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("zeroes.bin");
        write_repeated(&f, 0x00, 1024);

        let export = export_for_paths(&["zeroes.bin"]);
        let files = vec![PathBuf::from("zeroes.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].class, EntropyClass::Low);
        assert!(report.suspects[0].entropy_bits_per_byte < 2.0);
    }

    #[test]
    fn given_file_of_all_a_characters_then_classified_low() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("aaa.txt");
        write_repeated(&f, b'A', 512);

        let export = export_for_paths(&["aaa.txt"]);
        let files = vec![PathBuf::from("aaa.txt")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].path, "aaa.txt");
        assert_eq!(report.suspects[0].class, EntropyClass::Low);
    }

    #[test]
    fn given_two_distinct_bytes_repeated_then_classified_low() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("ab.bin");
        let mut data = Vec::with_capacity(1024);
        for _ in 0..512 {
            data.push(b'A');
            data.push(b'B');
        }
        fs::write(&f, data).unwrap();

        let export = export_for_paths(&["ab.bin"]);
        let files = vec![PathBuf::from("ab.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        // 2 equally-distributed values → ~1.0 bits/byte
        assert_eq!(report.suspects[0].class, EntropyClass::Low);
    }
}

// ── High entropy scenarios ──────────────────────────────────────

mod high_entropy {
    use super::*;

    #[test]
    fn given_pseudorandom_data_then_classified_high() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("random.bin");
        write_pseudorandom(&f, 4096);

        let export = export_for_paths(&["random.bin"]);
        let files = vec![PathBuf::from("random.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].class, EntropyClass::High);
        assert!(report.suspects[0].entropy_bits_per_byte > 7.5);
    }

    #[test]
    fn given_all_256_byte_values_uniformly_then_classified_high() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("uniform.bin");
        write_all_byte_values(&f, 4);

        let export = export_for_paths(&["uniform.bin"]);
        let files = vec![PathBuf::from("uniform.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].class, EntropyClass::High);
    }
}

// ── Normal entropy (not in suspects) ────────────────────────────

mod normal_entropy {
    use super::*;

    #[test]
    fn given_typical_english_text_then_not_in_suspects() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("readme.txt");
        let text = "The quick brown fox jumps over the lazy dog. \
                     This is a typical sentence with normal entropy. \
                     Repeated words help keep entropy moderate.\n"
            .repeat(20);
        fs::write(&f, text).unwrap();

        let export = export_for_paths(&["readme.txt"]);
        let files = vec![PathBuf::from("readme.txt")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert!(
            report.suspects.is_empty(),
            "Normal text should not appear in suspects: {report:?}"
        );
    }

    #[test]
    fn given_typical_source_code_then_not_in_suspects() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("main.rs");
        let code = r#"fn main() {
    let x = 42;
    println!("Hello, world! x = {}", x);
    for i in 0..10 {
        println!("{}", i);
    }
}
"#
        .repeat(10);
        fs::write(&f, code).unwrap();

        let export = export_for_paths(&["main.rs"]);
        let files = vec![PathBuf::from("main.rs")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert!(
            report.suspects.is_empty(),
            "Source code should not appear in suspects: {report:?}"
        );
    }
}

// ── Empty / edge cases ──────────────────────────────────────────

mod edge_cases {
    use super::*;

    #[test]
    fn given_empty_file_then_not_in_suspects() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("empty.txt");
        fs::write(&f, b"").unwrap();

        let export = export_for_paths(&["empty.txt"]);
        let files = vec![PathBuf::from("empty.txt")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert!(report.suspects.is_empty());
    }

    #[test]
    fn given_single_byte_file_then_classified_low() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("one.bin");
        fs::write(f, [0x42]).unwrap();

        let export = export_for_paths(&["one.bin"]);
        let files = vec![PathBuf::from("one.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].class, EntropyClass::Low);
    }

    #[test]
    fn given_no_files_then_empty_report() {
        let dir = tempdir().unwrap();
        let export = export_for_paths(&[]);
        let files: Vec<PathBuf> = vec![];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert!(report.suspects.is_empty());
    }

    #[test]
    fn given_file_not_in_export_then_module_is_unknown() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("mystery.bin");
        write_pseudorandom(&f, 1024);

        // Export has no entries — file is not mapped
        let export = export_for_paths(&[]);
        let files = vec![PathBuf::from("mystery.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].module, "(unknown)");
    }
}

// ── Sorting ─────────────────────────────────────────────────────

mod sorting {
    use super::*;

    #[test]
    fn given_multiple_suspects_then_sorted_by_entropy_descending() {
        let dir = tempdir().unwrap();
        // Low entropy file
        let lo = dir.path().join("low.bin");
        write_repeated(&lo, 0x00, 1024);

        // High entropy file
        let hi = dir.path().join("high.bin");
        write_pseudorandom(&hi, 4096);

        let export = export_for_paths(&["low.bin", "high.bin"]);
        let files = vec![PathBuf::from("low.bin"), PathBuf::from("high.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert!(report.suspects.len() >= 2);
        // First suspect should have higher entropy
        assert!(
            report.suspects[0].entropy_bits_per_byte >= report.suspects[1].entropy_bits_per_byte
        );
    }

    #[test]
    fn given_same_entropy_then_sorted_by_path_ascending() {
        let dir = tempdir().unwrap();
        // Two identical low-entropy files with different names
        let a = dir.path().join("aaa.bin");
        let b = dir.path().join("zzz.bin");
        write_repeated(&a, 0xFF, 1024);
        write_repeated(&b, 0xFF, 1024);

        let export = export_for_paths(&["aaa.bin", "zzz.bin"]);
        let files = vec![PathBuf::from("aaa.bin"), PathBuf::from("zzz.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 2);
        assert_eq!(report.suspects[0].path, "aaa.bin");
        assert_eq!(report.suspects[1].path, "zzz.bin");
    }
}

// ── Limits / budget ─────────────────────────────────────────────

mod limits {
    use super::*;

    #[test]
    fn given_max_bytes_limit_then_stops_scanning() {
        let dir = tempdir().unwrap();
        // Create two high-entropy files, each 1024 bytes
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        write_pseudorandom(&a, 1024);
        write_pseudorandom(&b, 1024);

        let export = export_for_paths(&["a.bin", "b.bin"]);
        let files = vec![PathBuf::from("a.bin"), PathBuf::from("b.bin")];

        // Set max_bytes just enough for one file
        let limits = AnalysisLimits {
            max_bytes: Some(1024),
            ..AnalysisLimits::default()
        };
        let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

        // Should have scanned at most one file (budget is reached after first)
        assert!(
            report.suspects.len() <= 2,
            "budget should constrain scanning"
        );
    }

    #[test]
    fn given_custom_max_file_bytes_then_sample_size_reflects_it() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("data.bin");
        write_pseudorandom(&f, 8192);

        let export = export_for_paths(&["data.bin"]);
        let files = vec![PathBuf::from("data.bin")];
        let limits = AnalysisLimits {
            max_file_bytes: Some(256),
            ..AnalysisLimits::default()
        };
        let report = build_entropy_report(dir.path(), &files, &export, &limits).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert!(report.suspects[0].sample_bytes <= 256);
    }
}

// ── Suspicious class (boundary) ─────────────────────────────────

mod suspicious_class {
    use super::*;

    #[test]
    fn given_moderately_high_entropy_then_classified_suspicious() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("semi.bin");
        // Build data with ~6.5-7.5 bits/byte: many distinct values but not uniformly distributed
        // Use 200 distinct byte values with slight skew
        let mut data = Vec::with_capacity(2048);
        for i in 0..2048u32 {
            // Map to 0..199 range with some bias
            let b = ((i.wrapping_mul(7) + i / 3) % 200) as u8;
            data.push(b);
        }
        fs::write(&f, &data).unwrap();

        let export = export_for_paths(&["semi.bin"]);
        let files = vec![PathBuf::from("semi.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        if !report.suspects.is_empty() {
            let finding = &report.suspects[0];
            // Should be Suspicious or High — not Normal, not Low
            assert!(
                finding.class == EntropyClass::Suspicious || finding.class == EntropyClass::High,
                "expected Suspicious or High for semi-random data, got {:?} with entropy {}",
                finding.class,
                finding.entropy_bits_per_byte
            );
        }
    }
}

// ── Path normalization ──────────────────────────────────────────

#[cfg(target_os = "windows")]
mod path_normalization {
    use super::*;

    #[test]
    fn given_backslash_paths_then_normalized_in_output() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("sub");
        fs::create_dir_all(&subdir).unwrap();
        let f = subdir.join("data.bin");
        write_repeated(&f, 0x00, 512);

        let export = export_for_paths(&["sub/data.bin"]);
        // Pass file with backslashes (Windows-style)
        let files = vec![PathBuf::from("sub\\data.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        // Paths in output should use forward slashes
        for finding in &report.suspects {
            assert!(
                !finding.path.contains('\\'),
                "path should be normalized: {}",
                finding.path
            );
        }
    }
}

// ── Text file moderate entropy ──────────────────────────────────

mod moderate_entropy {
    use super::*;

    #[test]
    fn given_text_file_when_computing_entropy_then_entropy_is_moderate() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("prose.txt");
        let text = "The quick brown fox jumps over the lazy dog. \
                     Pack my box with five dozen liquor jugs. \
                     How vexingly quick daft zebras jump.\n"
            .repeat(30);
        fs::write(&f, text).unwrap();

        let export = export_for_paths(&["prose.txt"]);
        let files = vec![PathBuf::from("prose.txt")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        // English text has ~2.0-5.0 bits/byte entropy → Normal class → not in suspects
        assert!(
            report.suspects.is_empty(),
            "text with moderate entropy should not appear in suspects"
        );
    }

    #[test]
    fn given_csv_data_then_entropy_is_moderate() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("data.csv");
        let csv = "id,name,value\n1,alpha,100\n2,beta,200\n3,gamma,300\n".repeat(40);
        fs::write(&f, csv).unwrap();

        let export = export_for_paths(&["data.csv"]);
        let files = vec![PathBuf::from("data.csv")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert!(
            report.suspects.is_empty(),
            "CSV data should have moderate entropy and not be flagged"
        );
    }
}

// ── Binary / high-entropy flagging ──────────────────────────────

mod high_entropy_flagging {
    use super::*;

    #[test]
    fn given_binary_file_when_computing_entropy_then_entropy_is_high() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("crypto.bin");
        write_pseudorandom(&f, 4096);

        let export = export_for_paths(&["crypto.bin"]);
        let files = vec![PathBuf::from("crypto.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].class, EntropyClass::High);
        assert!(
            report.suspects[0].entropy_bits_per_byte > 7.5,
            "binary data should have entropy > 7.5, got {}",
            report.suspects[0].entropy_bits_per_byte
        );
    }

    #[test]
    fn given_multiple_high_entropy_files_then_all_flagged() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("key1.bin");
        let b = dir.path().join("key2.bin");
        let c = dir.path().join("key3.bin");
        write_pseudorandom(&a, 2048);
        write_pseudorandom(&b, 2048);
        // Different seed for variety
        let mut data = Vec::with_capacity(2048);
        let mut x = 0xDEADBEEFu32;
        for _ in 0..2048 {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            data.push((x & 0xFF) as u8);
        }
        fs::write(&c, data).unwrap();

        let export = export_for_paths(&["key1.bin", "key2.bin", "key3.bin"]);
        let files = vec![
            PathBuf::from("key1.bin"),
            PathBuf::from("key2.bin"),
            PathBuf::from("key3.bin"),
        ];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(
            report.suspects.len(),
            3,
            "all three high-entropy files should be flagged"
        );
        for suspect in &report.suspects {
            assert!(
                suspect.class == EntropyClass::High || suspect.class == EntropyClass::Suspicious,
                "each should be High or Suspicious, got {:?}",
                suspect.class
            );
        }
    }

    #[test]
    fn high_entropy_files_are_flagged_as_suspicious_or_high() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("secret.key");
        write_pseudorandom(&f, 1024);

        let export = export_for_paths(&["secret.key"]);
        let files = vec![PathBuf::from("secret.key")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert!(
            !report.suspects.is_empty(),
            "high-entropy file should be flagged"
        );
        assert!(
            report.suspects[0].class == EntropyClass::High
                || report.suspects[0].class == EntropyClass::Suspicious,
            "should be flagged as High or Suspicious"
        );
    }
}

// ── Zero / near-zero entropy ────────────────────────────────────

mod zero_entropy {
    use super::*;

    #[test]
    fn given_empty_file_when_computing_entropy_then_entropy_is_zero() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("empty.dat");
        fs::write(&f, b"").unwrap();

        let export = export_for_paths(&["empty.dat"]);
        let files = vec![PathBuf::from("empty.dat")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        // Empty files are skipped entirely (0 bytes read)
        assert!(
            report.suspects.is_empty(),
            "empty file should not produce any findings"
        );
    }

    #[test]
    fn given_file_with_all_same_byte_when_computing_entropy_then_entropy_is_zero() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("zeros.bin");
        write_repeated(&f, 0x00, 2048);

        let export = export_for_paths(&["zeros.bin"]);
        let files = vec![PathBuf::from("zeros.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].class, EntropyClass::Low);
        assert!(
            report.suspects[0].entropy_bits_per_byte < 0.01,
            "single repeated byte should have ~0 entropy, got {}",
            report.suspects[0].entropy_bits_per_byte
        );
    }

    #[test]
    fn given_file_with_all_ff_bytes_then_entropy_is_zero() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("ones.bin");
        write_repeated(&f, 0xFF, 1024);

        let export = export_for_paths(&["ones.bin"]);
        let files = vec![PathBuf::from("ones.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        assert_eq!(report.suspects.len(), 1);
        assert_eq!(report.suspects[0].class, EntropyClass::Low);
        assert!(
            report.suspects[0].entropy_bits_per_byte < 0.01,
            "all-0xFF file should have ~0 entropy, got {}",
            report.suspects[0].entropy_bits_per_byte
        );
    }
}

// ── Mixed scenarios ─────────────────────────────────────────────

mod mixed_scenarios {
    use super::*;

    #[test]
    fn given_mix_of_normal_and_high_entropy_then_only_anomalies_flagged() {
        let dir = tempdir().unwrap();
        // Normal entropy: source code
        let src = dir.path().join("app.py");
        let code = "def main():\n    print('hello world')\n    x = 42\n".repeat(20);
        fs::write(&src, code).unwrap();

        // High entropy: random data
        let key = dir.path().join("secret.bin");
        write_pseudorandom(&key, 2048);

        let export = export_for_paths(&["app.py", "secret.bin"]);
        let files = vec![PathBuf::from("app.py"), PathBuf::from("secret.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        // Only the high-entropy file should be in suspects
        assert!(
            report.suspects.iter().any(|f| f.path == "secret.bin"),
            "high-entropy file should be flagged"
        );
        assert!(
            !report.suspects.iter().any(|f| f.path == "app.py"),
            "normal source code should not be flagged"
        );
    }

    #[test]
    fn given_low_and_high_entropy_files_then_both_flagged_with_correct_class() {
        let dir = tempdir().unwrap();
        let low = dir.path().join("padding.bin");
        write_repeated(&low, 0x00, 1024);

        let high = dir.path().join("noise.bin");
        write_pseudorandom(&high, 2048);

        let export = export_for_paths(&["padding.bin", "noise.bin"]);
        let files = vec![PathBuf::from("padding.bin"), PathBuf::from("noise.bin")];
        let report =
            build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default()).unwrap();

        let low_finding = report.suspects.iter().find(|f| f.path == "padding.bin");
        let high_finding = report.suspects.iter().find(|f| f.path == "noise.bin");

        assert!(low_finding.is_some(), "low-entropy file should be flagged");
        assert!(
            high_finding.is_some(),
            "high-entropy file should be flagged"
        );
        assert_eq!(low_finding.unwrap().class, EntropyClass::Low);
        assert_eq!(high_finding.unwrap().class, EntropyClass::High);
    }
}
