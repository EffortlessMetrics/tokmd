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
    // `fuzz_entropy` path-includes this module (crate = tokmd-fuzz), where the
    // tokmd-analysis-only `io_trace`/`io_cache` modules do not exist; the fuzz
    // target never exercises the read helpers, so instrumentation is compiled
    // out there.
    #[cfg(not(fuzzing))]
    {
        crate::io_trace::record(crate::io_trace::IoReadMode::Head, max_bytes, path);
        crate::io_cache::get_or_read_bytes(
            crate::io_trace::IoReadMode::Head,
            max_bytes,
            path,
            || read::read_head(path, max_bytes),
        )
    }
    #[cfg(fuzzing)]
    {
        read::read_head(path, max_bytes)
    }
}

pub fn read_head_tail(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    #[cfg(not(fuzzing))]
    {
        crate::io_trace::record(crate::io_trace::IoReadMode::HeadTail, max_bytes, path);
        crate::io_cache::get_or_read_bytes(
            crate::io_trace::IoReadMode::HeadTail,
            max_bytes,
            path,
            || read::read_head_tail(path, max_bytes),
        )
    }
    #[cfg(fuzzing)]
    {
        read::read_head_tail(path, max_bytes)
    }
}

pub fn read_lines(path: &Path, max_lines: usize, max_bytes: usize) -> Result<Vec<String>> {
    #[cfg(not(fuzzing))]
    crate::io_trace::record(crate::io_trace::IoReadMode::Lines, max_bytes, path);
    read::read_lines(path, max_lines, max_bytes)
}

pub fn read_text_capped(path: &Path, max_bytes: usize) -> Result<String> {
    #[cfg(not(fuzzing))]
    crate::io_trace::record(crate::io_trace::IoReadMode::TextCapped, max_bytes, path);
    read::read_text_capped(path, max_bytes)
}

pub fn is_text_like(bytes: &[u8]) -> bool {
    bytes::is_text_like(bytes)
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
