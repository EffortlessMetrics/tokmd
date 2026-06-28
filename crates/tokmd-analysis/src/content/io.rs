//! Public content I/O facade for analysis.
//!
//! The implementation lives in owner modules under `io/`, while this module
//! preserves the existing `crate::content::io::*` call surface.
//!
//! ## What belongs here
//! * File content reading (head, tail, lines)
//! * Text detection and byte-level classification
//! * File integrity hashing (BLAKE3)
//! * Tag counting (TODOs, FIXMEs)
//! * Entropy calculation
//!
//! ## What does NOT belong here
//! * File listing (use tokmd-scan::walk)
//! * File modification

#![cfg_attr(
    fuzzing,
    expect(
        dead_code,
        reason = "fuzz_entropy includes only hash/tag/entropy facade entry points"
    )
)]

use std::path::Path;

use anyhow::Result;

#[path = "io/bytes.rs"]
mod bytes;
#[path = "io/read.rs"]
mod read;
#[path = "io/tags.rs"]
mod tags;

pub fn read_head(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    read::read_head(path, max_bytes)
}

pub fn read_head_tail(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    read::read_head_tail(path, max_bytes)
}

pub fn read_lines(path: &Path, max_lines: usize, max_bytes: usize) -> Result<Vec<String>> {
    read::read_lines(path, max_lines, max_bytes)
}

pub fn read_text_capped(path: &Path, max_bytes: usize) -> Result<String> {
    read::read_text_capped(path, max_bytes)
}

pub fn as_text(bytes: &[u8]) -> Option<&str> {
    bytes::as_text(bytes)
}

#[cfg_attr(
    not(any(test, fuzzing)),
    expect(
        dead_code,
        reason = "content::io facade API exercised by tests and fuzz_entropy"
    )
)]
pub fn hash_bytes(bytes: &[u8]) -> String {
    bytes::hash_bytes(bytes)
}

#[cfg_attr(
    not(any(test, fuzzing)),
    expect(
        dead_code,
        reason = "content::io facade API exercised by tests and fuzz_entropy"
    )
)]
pub fn hash_file(path: &Path, max_bytes: usize) -> Result<String> {
    bytes::hash_file(path, max_bytes)
}

#[cfg_attr(
    not(any(test, fuzzing)),
    expect(
        dead_code,
        reason = "content::io facade API exercised by tests and fuzz_entropy"
    )
)]
pub fn count_tags(text: &str, tag_names: &[&str]) -> Vec<(String, usize)> {
    tags::count_tags(text, tag_names)
}

pub(crate) fn count_delimited_tags(text: &str, tag_names: &[&str]) -> Vec<(String, usize)> {
    tags::count_delimited_tags(text, tag_names)
}

pub fn entropy_bits_per_byte(bytes: &[u8]) -> f32 {
    bytes::entropy_bits_per_byte(bytes)
}
