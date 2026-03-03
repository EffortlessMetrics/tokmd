//! Fuzz target for SVG badge rendering.
//!
//! Tests `badge_svg()` with arbitrary label and value strings to verify:
//! - No panics on any valid UTF-8 input
//! - Deterministic output (same inputs → same SVG)
//! - Output is well-formed (starts with `<svg` and ends with `</svg>`)

#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_badge::badge_svg;

/// Cap input size — badge rendering is fast but we focus on realistic lengths.
const MAX_INPUT_SIZE: usize = 4 * 1024; // 4 KB

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_SIZE {
        return;
    }
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    // Split on first newline: label\nvalue
    let (label, value) = match input.find('\n') {
        Some(pos) => (&input[..pos], &input[pos + 1..]),
        None => (input, ""),
    };

    // Generate badge — must never panic
    let svg = badge_svg(label, value);

    // Invariant: deterministic output
    let svg2 = badge_svg(label, value);
    assert_eq!(svg, svg2, "badge_svg must be deterministic");

    // Invariant: output is non-empty and looks like SVG
    assert!(!svg.is_empty(), "badge_svg must not return empty string");
    assert!(
        svg.starts_with("<svg"),
        "badge_svg must start with <svg, got: {}",
        &svg[..svg.len().min(40)]
    );
    assert!(svg.ends_with("</svg>"), "badge_svg must end with </svg>");
});
