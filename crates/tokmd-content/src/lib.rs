//! # tokmd-content
//!
//! Content scanning helpers for tokmd analysis.

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{Context, Result};

pub fn read_head(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let mut buf = vec![0u8; max_bytes];
    let n = file.read(&mut buf)?;
    buf.truncate(n);
    Ok(buf)
}

pub fn read_head_tail(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    if max_bytes == 0 {
        return Ok(Vec::new());
    }
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    if size as usize <= max_bytes {
        return read_head(path, max_bytes);
    }

    let half = max_bytes / 2;
    let head_len = half.max(1);
    let tail_len = max_bytes.saturating_sub(head_len);

    let mut head = vec![0u8; head_len];
    let n_head = file.read(&mut head)?;
    head.truncate(n_head);

    if tail_len == 0 {
        return Ok(head);
    }

    let tail_start = size.saturating_sub(tail_len as u64);
    file.seek(SeekFrom::Start(tail_start))?;
    let mut tail = vec![0u8; tail_len];
    let n_tail = file.read(&mut tail)?;
    tail.truncate(n_tail);

    head.extend_from_slice(&tail);
    Ok(head)
}

pub fn read_lines(path: &Path, max_lines: usize, max_bytes: usize) -> Result<Vec<String>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
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
