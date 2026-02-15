//! # tokmd-content
//!
//! **Tier 2 (Utilities)**
//!
//! Content scanning helpers for analysis. Provides file content inspection
//! capabilities including reading, hashing, and entropy calculation.
//!
//! ## What belongs here
//! * File content reading (head, tail, lines)
//! * Text detection
//! * File integrity hashing (BLAKE3)
//! * Tag counting (TODOs, FIXMEs)
//! * Entropy calculation
//! * Function-level complexity metrics
//!
//! ## What does NOT belong here
//! * File listing (use tokmd-walk)
//! * File modification

pub mod complexity;

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{Context, Result};

fn read_head_from_file(file: &mut File, max_bytes: usize) -> Result<Vec<u8>> {
    use std::io::Read as _;
    let mut buf = Vec::with_capacity(max_bytes);
    file.take(max_bytes as u64).read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn read_head(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    let mut file =
        File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    read_head_from_file(&mut file, max_bytes)
}

pub fn read_head_tail(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    if max_bytes == 0 {
        return Ok(Vec::new());
    }
    let mut file =
        File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let size = file
        .metadata()
        .with_context(|| format!("Failed to get metadata for {}", path.display()))?
        .len();
    if size as usize <= max_bytes {
        return read_head_from_file(&mut file, max_bytes);
    }

    let half = max_bytes / 2;
    let head_len = half.max(1);
    let tail_len = max_bytes.saturating_sub(head_len);

    let mut head = vec![0u8; head_len];
    file.read_exact(&mut head)?;

    if tail_len == 0 {
        return Ok(head);
    }

    let tail_start = size.saturating_sub(tail_len as u64);
    file.seek(SeekFrom::Start(tail_start))?;
    let mut tail = vec![0u8; tail_len];
    file.read_exact(&mut tail)?;

    head.extend_from_slice(&tail);
    Ok(head)
}

pub fn read_lines(path: &Path, max_lines: usize, max_bytes: usize) -> Result<Vec<String>> {
    let file = File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut bytes = 0usize;

    for line in reader.lines() {
        let line = line?;
        bytes += line.len();
        lines.push(line);
        if lines.len() >= max_lines || bytes >= max_bytes {
            break;
        }
    }

    Ok(lines)
}

pub fn read_text_capped(path: &Path, max_bytes: usize) -> Result<String> {
    let bytes = read_head(path, max_bytes)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

pub fn is_text_like(bytes: &[u8]) -> bool {
    if bytes.contains(&0) {
        return false;
    }
    std::str::from_utf8(bytes).is_ok()
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn hash_file(path: &Path, max_bytes: usize) -> Result<String> {
    let bytes = read_head(path, max_bytes)?;
    Ok(hash_bytes(&bytes))
}

pub fn count_tags(text: &str, tags: &[&str]) -> Vec<(String, usize)> {
    let upper = text.to_uppercase();
    tags.iter()
        .map(|tag| {
            let needle = tag.to_uppercase();
            let count = upper.matches(&needle).count();
            (tag.to_string(), count)
        })
        .collect()
}

pub fn entropy_bits_per_byte(bytes: &[u8]) -> f32 {
    if bytes.is_empty() {
        return 0.0;
    }
    let mut counts = [0u32; 256];
    for b in bytes {
        counts[*b as usize] += 1;
    }
    let len = bytes.len() as f32;
    let mut entropy = 0.0f32;
    for count in counts {
        if count == 0 {
            continue;
        }
        let p = count as f32 / len;
        entropy -= p * p.log2();
    }
    entropy
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_read_head_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("empty");
        File::create(&path).unwrap();

        let bytes = read_head(&path, 10).unwrap();
        assert!(bytes.is_empty());
    }

    #[test]
    fn test_read_head_small() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("small");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"hello").unwrap();

        let bytes = read_head(&path, 10).unwrap();
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn test_read_head_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("limit");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"hello world").unwrap();

        let bytes = read_head(&path, 5).unwrap();
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn test_read_head_tail_small() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("small");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"hello").unwrap();

        let bytes = read_head_tail(&path, 10).unwrap();
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn test_read_head_tail_large() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("large");
        let mut f = File::create(&path).unwrap();
        // 0123456789
        f.write_all(b"0123456789").unwrap();

        // max_bytes = 4. half=2. head=2 ("01"), tail=2 ("89").
        let bytes = read_head_tail(&path, 4).unwrap();
        assert_eq!(bytes, b"0189");
    }

    #[test]
    fn test_read_head_tail_odd() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("odd");
        let mut f = File::create(&path).unwrap();
        // 0123456789
        f.write_all(b"0123456789").unwrap();

        // max_bytes = 5. half=2. head=2 ("01"), tail=3 ("789").
        let bytes = read_head_tail(&path, 5).unwrap();
        assert_eq!(bytes, b"01789");
    }

    // ========================
    // read_lines tests
    // ========================

    #[test]
    fn test_read_lines_returns_actual_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("lines.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "first line").unwrap();
        writeln!(f, "second line").unwrap();
        writeln!(f, "third line").unwrap();

        let lines = read_lines(&path, 10, 10000).unwrap();
        // Verify actual content, not empty or dummy values
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "first line");
        assert_eq!(lines[1], "second line");
        assert_eq!(lines[2], "third line");
    }

    #[test]
    fn test_read_lines_respects_max_lines_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("many_lines.txt");
        let mut f = File::create(&path).unwrap();
        for i in 0..10 {
            writeln!(f, "line {}", i).unwrap();
        }

        // Request only 3 lines
        let lines = read_lines(&path, 3, 10000).unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line 0");
        assert_eq!(lines[1], "line 1");
        assert_eq!(lines[2], "line 2");
    }

    #[test]
    fn test_read_lines_respects_max_bytes_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bytes_limited.txt");
        let mut f = File::create(&path).unwrap();
        // Each line is 10 chars: "line 0\n" etc
        for i in 0..10 {
            writeln!(f, "line {:04}", i).unwrap();
        }

        // Limit to 25 bytes - should get ~2-3 lines (each ~10 bytes)
        let lines = read_lines(&path, 100, 25).unwrap();
        // With byte limit of 25 and lines of ~10 bytes each,
        // we should stop after accumulating >= 25 bytes
        assert!(
            lines.len() >= 2 && lines.len() <= 4,
            "Expected 2-4 lines, got {}",
            lines.len()
        );
        // Verify first line content
        assert_eq!(lines[0], "line 0000");
    }

    #[test]
    fn test_read_lines_bytes_accumulate_correctly() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("accumulate.txt");
        let mut f = File::create(&path).unwrap();
        // Write lines with known sizes
        writeln!(f, "12345").unwrap(); // 5 bytes (without newline in result)
        writeln!(f, "67890").unwrap(); // 5 more = 10 total
        writeln!(f, "abcde").unwrap(); // 5 more = 15 total
        writeln!(f, "fghij").unwrap(); // 5 more = 20 total

        // Stop at exactly 10 bytes - should get 2 lines
        let lines = read_lines(&path, 100, 10).unwrap();
        assert_eq!(lines.len(), 2, "Should stop after reaching 10 bytes");
        assert_eq!(lines[0], "12345");
        assert_eq!(lines[1], "67890");
    }

    #[test]
    fn test_read_lines_single_line_at_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("single.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "exactlyten").unwrap(); // 10 chars

        // max_lines = 1 should stop after first line
        let lines = read_lines(&path, 1, 10000).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "exactlyten");
    }

    #[test]
    fn test_read_lines_bytes_limit_stops_after_reaching_threshold() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("threshold.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "aaaaa").unwrap(); // 5 bytes
        writeln!(f, "bbbbb").unwrap(); // 5 bytes (total 10)
        writeln!(f, "ccccc").unwrap(); // should not be read if limit is 9

        // With limit of 9 bytes, we should get exactly 2 lines
        // because after first line (5 bytes), bytes=5 < 9, continue
        // after second line (5 bytes), bytes=10 >= 9, break
        let lines = read_lines(&path, 100, 9).unwrap();
        assert_eq!(lines.len(), 2);
    }

    // ========================
    // read_text_capped tests
    // ========================

    #[test]
    fn test_read_text_capped_returns_actual_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("text.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"Hello, World!").unwrap();

        let text = read_text_capped(&path, 100).unwrap();
        // Verify we get actual content, not empty or "xyzzy"
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_read_text_capped_respects_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("long_text.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"The quick brown fox jumps over the lazy dog")
            .unwrap();

        let text = read_text_capped(&path, 9).unwrap();
        assert_eq!(text, "The quick");
    }

    // ========================
    // hash_file tests
    // ========================

    #[test]
    fn test_hash_file_returns_correct_blake3_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("hash_test.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"test content").unwrap();

        let hash = hash_file(&path, 1000).unwrap();

        // Verify it's not empty
        assert!(!hash.is_empty());
        // Verify it's 64 hex chars (BLAKE3 output)
        assert_eq!(hash.len(), 64);
        // Verify it matches expected BLAKE3 hash
        let expected = hash_bytes(b"test content");
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hash_file_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("deterministic.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"same content every time").unwrap();

        let hash1 = hash_file(&path, 1000).unwrap();
        let hash2 = hash_file(&path, 1000).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_file_different_content_different_hash() {
        let tmp = tempfile::tempdir().unwrap();

        let path1 = tmp.path().join("file1.txt");
        let mut f1 = File::create(&path1).unwrap();
        f1.write_all(b"content A").unwrap();

        let path2 = tmp.path().join("file2.txt");
        let mut f2 = File::create(&path2).unwrap();
        f2.write_all(b"content B").unwrap();

        let hash1 = hash_file(&path1, 1000).unwrap();
        let hash2 = hash_file(&path2, 1000).unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_file_respects_max_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("long_file.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"abcdefghij").unwrap();

        // Hash only first 5 bytes
        let hash_limited = hash_file(&path, 5).unwrap();
        let expected = hash_bytes(b"abcde");
        assert_eq!(hash_limited, expected);

        // Full hash should be different
        let hash_full = hash_file(&path, 1000).unwrap();
        assert_ne!(hash_limited, hash_full);
    }
}
