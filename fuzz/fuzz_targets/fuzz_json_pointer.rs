//! Fuzz target for RFC 6901 JSON Pointer resolution.
//!
//! Tests `resolve_pointer()` with arbitrary JSON documents and pointer strings
//! to find panics or unexpected behavior in pointer parsing and navigation.

#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use tokmd_gate::resolve_pointer;

/// Max input sizes to prevent pathological parse times
const MAX_JSON_SIZE: usize = 64 * 1024; // 64KB for JSON
const MAX_POINTER_SIZE: usize = 4 * 1024; // 4KB for pointer strings

#[derive(Debug, Arbitrary)]
struct PointerInput {
    /// Raw bytes for JSON document (will be parsed)
    json_bytes: Vec<u8>,
    /// The JSON pointer string to resolve
    pointer: String,
}

fuzz_target!(|input: PointerInput| {
    if input.json_bytes.len() > MAX_JSON_SIZE || input.pointer.len() > MAX_POINTER_SIZE {
        return;
    }
    // Try to parse JSON from bytes
    let Ok(json_str) = std::str::from_utf8(&input.json_bytes) else {
        return;
    };
    let Ok(doc) = serde_json::from_str::<Value>(json_str) else {
        return;
    };

    // Test pointer resolution - should never panic
    let _ = resolve_pointer(&doc, &input.pointer);

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
