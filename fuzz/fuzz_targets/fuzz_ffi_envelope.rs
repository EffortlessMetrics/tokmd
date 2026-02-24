//! Fuzz target for FFI envelope parser/extractor invariants.
//!
//! Validates:
//! - No panics on arbitrary UTF-8 JSON inputs
//! - Deterministic parser/extractor behavior
//! - Equivalence between step-wise and convenience APIs

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use tokmd_ffi_envelope::{
    EnvelopeExtractError, extract_data, extract_data_from_json, parse_envelope,
};

const MAX_INPUT_SIZE: usize = 64 * 1024;

fn assert_result_eq(
    left: &Result<Value, EnvelopeExtractError>,
    right: &Result<Value, EnvelopeExtractError>,
) {
    match (left, right) {
        (Ok(a), Ok(b)) => assert_eq!(a, b),
        (Err(a), Err(b)) => assert_eq!(a.to_string(), b.to_string()),
        (a, b) => panic!("mismatched results: left={a:?} right={b:?}"),
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_SIZE {
        return;
    }
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    let first = extract_data_from_json(input);
    let second = extract_data_from_json(input);
    assert_result_eq(&first, &second);

    if let Ok(parsed) = parse_envelope(input) {
        let via_steps = extract_data(parsed);
        let direct = extract_data_from_json(input);
        assert_result_eq(&via_steps, &direct);
    }
});
