//! # tokmd-content
//!
//! Content scanning helpers for tokmd analysis.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
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

pub fn is_text_like(bytes: &[u8]) -> bool {
    if bytes.iter().any(|b| *b == 0) {
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
