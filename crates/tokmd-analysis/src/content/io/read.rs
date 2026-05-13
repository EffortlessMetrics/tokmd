//! Bounded file-reading helpers for content analysis.

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

pub(super) fn read_head(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    let mut file =
        File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    read_head_from_file(&mut file, max_bytes)
}

pub(super) fn read_head_tail(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
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

pub(super) fn read_lines(path: &Path, max_lines: usize, max_bytes: usize) -> Result<Vec<String>> {
    if max_lines == 0 || max_bytes == 0 {
        return Ok(Vec::new());
    }
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

pub(super) fn read_text_capped(path: &Path, max_bytes: usize) -> Result<String> {
    let bytes = read_head(path, max_bytes)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use super::*;

    #[test]
    fn read_head_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("empty");
        File::create(&path).unwrap();

        let bytes = read_head(&path, 10).unwrap();
        assert!(bytes.is_empty());
    }

    #[test]
    fn read_head_small() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("small");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"hello").unwrap();

        let bytes = read_head(&path, 10).unwrap();
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn read_head_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("limit");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"hello world").unwrap();

        let bytes = read_head(&path, 5).unwrap();
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn read_head_tail_small() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("small");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"hello").unwrap();

        let bytes = read_head_tail(&path, 10).unwrap();
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn read_head_tail_large() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("large");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"0123456789").unwrap();

        let bytes = read_head_tail(&path, 4).unwrap();
        assert_eq!(bytes, b"0189");
    }

    #[test]
    fn read_head_tail_odd() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("odd");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"0123456789").unwrap();

        let bytes = read_head_tail(&path, 5).unwrap();
        assert_eq!(bytes, b"01789");
    }

    #[test]
    fn read_lines_returns_actual_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("lines.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "first line").unwrap();
        writeln!(f, "second line").unwrap();
        writeln!(f, "third line").unwrap();

        let lines = read_lines(&path, 10, 10000).unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "first line");
        assert_eq!(lines[1], "second line");
        assert_eq!(lines[2], "third line");
    }

    #[test]
    fn read_lines_respects_max_lines_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("many_lines.txt");
        let mut f = File::create(&path).unwrap();
        for i in 0..10 {
            writeln!(f, "line {}", i).unwrap();
        }

        let lines = read_lines(&path, 3, 10000).unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line 0");
        assert_eq!(lines[1], "line 1");
        assert_eq!(lines[2], "line 2");
    }

    #[test]
    fn read_lines_respects_max_bytes_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bytes_limited.txt");
        let mut f = File::create(&path).unwrap();
        for i in 0..10 {
            writeln!(f, "line {:04}", i).unwrap();
        }

        let lines = read_lines(&path, 100, 25).unwrap();
        assert!(
            lines.len() >= 2 && lines.len() <= 4,
            "Expected 2-4 lines, got {}",
            lines.len()
        );
        assert_eq!(lines[0], "line 0000");
    }

    #[test]
    fn read_lines_bytes_accumulate_correctly() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("accumulate.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "12345").unwrap();
        writeln!(f, "67890").unwrap();
        writeln!(f, "abcde").unwrap();
        writeln!(f, "fghij").unwrap();

        let lines = read_lines(&path, 100, 10).unwrap();
        assert_eq!(lines.len(), 2, "Should stop after reaching 10 bytes");
        assert_eq!(lines[0], "12345");
        assert_eq!(lines[1], "67890");
    }

    #[test]
    fn read_lines_single_line_at_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("single.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "exactlyten").unwrap();

        let lines = read_lines(&path, 1, 10000).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "exactlyten");
    }

    #[test]
    fn read_lines_bytes_limit_stops_after_reaching_threshold() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("threshold.txt");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "aaaaa").unwrap();
        writeln!(f, "bbbbb").unwrap();
        writeln!(f, "ccccc").unwrap();

        let lines = read_lines(&path, 100, 9).unwrap();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn read_text_capped_returns_actual_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("text.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"Hello, World!").unwrap();

        let text = read_text_capped(&path, 100).unwrap();
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn read_text_capped_respects_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("long_text.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"The quick brown fox jumps over the lazy dog")
            .unwrap();

        let text = read_text_capped(&path, 9).unwrap();
        assert_eq!(text, "The quick");
    }
}
