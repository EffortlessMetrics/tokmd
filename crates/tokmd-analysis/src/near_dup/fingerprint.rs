//! Winnowing fingerprint construction for near-duplicate detection.

use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use rustc_hash::FxHasher;

/// Default k-gram size (number of tokens per shingle).
pub(super) const K: usize = 25;
/// Winnowing window size.
pub(super) const W: usize = 4;
/// Skip fingerprints appearing in more than this many files (common boilerplate).
pub(super) const MAX_POSTINGS: usize = 50;

/// Read a file and compute its Winnowing fingerprints.
pub(super) fn read_and_fingerprint(path: &Path) -> Result<Vec<u64>> {
    let mut content = String::new();
    let mut file = std::fs::File::open(path)?;
    file.read_to_string(&mut content)?;

    Ok(winnow(&content))
}

/// Tokenize text by splitting on non-alphanumeric/underscore boundaries.
pub(super) fn tokenize(text: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let mut start = None;

    for (i, &b) in bytes.iter().enumerate() {
        let is_token_char = b.is_ascii_alphanumeric() || b == b'_';
        match (start, is_token_char) {
            (None, true) => start = Some(i),
            (Some(s), false) => {
                tokens.push(&text[s..i]);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        tokens.push(&text[s..]);
    }
    tokens
}

/// Hash a k-gram (slice of tokens) using FxHash.
fn hash_kgram(tokens: &[&str]) -> u64 {
    let mut hasher = FxHasher::default();
    for t in tokens {
        t.hash(&mut hasher);
    }
    hasher.finish()
}

/// Apply the Winnowing algorithm to extract fingerprints from text.
pub(super) fn winnow(text: &str) -> Vec<u64> {
    let tokens = tokenize(text);
    if tokens.len() < K {
        return Vec::new();
    }

    let kgram_count = tokens.len() - K + 1;
    let hashes: Vec<u64> = (0..kgram_count)
        .map(|i| hash_kgram(&tokens[i..i + K]))
        .collect();

    if hashes.len() < W {
        return hashes;
    }

    let mut fingerprints = Vec::new();
    let mut prev_min_idx: Option<usize> = None;

    for window_start in 0..=(hashes.len() - W) {
        let window = &hashes[window_start..window_start + W];
        let mut min_val = window[0];
        let mut min_idx = window_start;
        for (offset, &h) in window.iter().enumerate() {
            if h <= min_val {
                min_val = h;
                min_idx = window_start + offset;
            }
        }

        if prev_min_idx != Some(min_idx) {
            fingerprints.push(min_val);
            prev_min_idx = Some(min_idx);
        }
    }

    fingerprints
}
