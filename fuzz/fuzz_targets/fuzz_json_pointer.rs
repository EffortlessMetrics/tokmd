//! Fuzz target for RFC 6901 JSON Pointer resolution.
//!
//! Tests `resolve_pointer()` with arbitrary JSON documents and pointer strings
//! to find panics or unexpected behavior in pointer parsing and navigation.
//!
//! Corpus format: `json_document\npointer_string`
//! The input is split on the first newline to separate JSON from pointer.

#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use tokmd_gate::resolve_pointer;

/// Max input sizes to prevent pathological parse times
const MAX_JSON_SIZE: usize = 64 * 1024; // 64KB for JSON
const MAX_POINTER_SIZE: usize = 4 * 1024; // 4KB for pointer strings

fuzz_target!(|data: &[u8]| {
    // Split on newline: json\npointer
    let Some(pos) = data.iter().position(|&b| b == b'\n') else {
        return;
    };
    let (json_bytes, ptr_bytes) = data.split_at(pos);
    let ptr_bytes = &ptr_bytes[1..]; // skip the newline

    if json_bytes.len() > MAX_JSON_SIZE || ptr_bytes.len() > MAX_POINTER_SIZE {
        return;
    }

    let Ok(json_str) = std::str::from_utf8(json_bytes) else {
        return;
    };
    let Ok(ptr_str) = std::str::from_utf8(ptr_bytes) else {
        return;
    };
    let Ok(doc) = serde_json::from_str::<Value>(json_str) else {
        return;
    };

    // Test pointer resolution - should never panic
    let _ = resolve_pointer(&doc, ptr_str);

    // Also test some edge case pointers
    let _ = resolve_pointer(&doc, "");
    let _ = resolve_pointer(&doc, "/");
    let _ = resolve_pointer(&doc, "//");
    let _ = resolve_pointer(&doc, "/~0");
    let _ = resolve_pointer(&doc, "/~1");
    let _ = resolve_pointer(&doc, "/~01");
    let _ = resolve_pointer(&doc, "/0");
    let _ = resolve_pointer(&doc, "/0/0/0/0/0");
});
