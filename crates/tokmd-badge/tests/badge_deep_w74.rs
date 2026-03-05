//! W74 – Badge rendering deep tests.

use tokmd_badge::badge_svg;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the first `width="N"` from the SVG string (the overall SVG width).
fn svg_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("closing quote") + start;
    svg[start..end].parse().expect("numeric width")
}

/// Assert the SVG is well-formed: starts with `<svg`, ends with `</svg>`, and
/// contains the required xmlns attribute.
fn assert_wellformed(svg: &str) {
    assert!(svg.starts_with("<svg"), "must start with <svg");
    assert!(svg.ends_with("</svg>"), "must end with </svg>");
    assert!(
        svg.contains("xmlns=\"http://www.w3.org/2000/svg\""),
        "must include SVG xmlns"
    );
}

// ---------------------------------------------------------------------------
// 1. Basic rendering
// ---------------------------------------------------------------------------

#[test]
fn w74_basic_badge_contains_label_and_value() {
    let svg = badge_svg("lines", "1234");
    assert!(svg.contains("lines"));
    assert!(svg.contains("1234"));
}

#[test]
fn w74_basic_badge_is_wellformed_svg() {
    assert_wellformed(&badge_svg("metric", "42"));
}

#[test]
fn w74_badge_has_role_img() {
    let svg = badge_svg("a", "b");
    assert!(svg.contains("role=\"img\""));
}

// ---------------------------------------------------------------------------
// 2. Width / layout
// ---------------------------------------------------------------------------

#[test]
fn w74_width_scales_with_text_length() {
    let short = svg_width(&badge_svg("a", "1"));
    let long = svg_width(&badge_svg("very_long_label", "very_long_value"));
    assert!(long > short, "longer text must produce wider badge");
}

#[test]
fn w74_minimum_width_enforced() {
    // Even a single character should produce at least 120px (60 + 60 minimums).
    let w = svg_width(&badge_svg("x", "y"));
    assert!(w >= 120);
}

#[test]
fn w74_badge_height_is_24() {
    let svg = badge_svg("h", "v");
    assert!(svg.contains("height=\"24\""));
}

// ---------------------------------------------------------------------------
// 3. Colour segments
// ---------------------------------------------------------------------------

#[test]
fn w74_label_segment_has_grey_fill() {
    let svg = badge_svg("label", "value");
    assert!(svg.contains("fill=\"#555\""));
}

#[test]
fn w74_value_segment_has_blue_fill() {
    let svg = badge_svg("label", "value");
    assert!(svg.contains("fill=\"#4c9aff\""));
}

// ---------------------------------------------------------------------------
// 4. Numeric / zero values
// ---------------------------------------------------------------------------

#[test]
fn w74_zero_value_rendered() {
    let svg = badge_svg("files", "0");
    assert!(svg.contains(">0<"));
}

#[test]
fn w74_large_number_rendered() {
    let svg = badge_svg("lines", "1234567");
    assert!(svg.contains("1234567"));
}

// ---------------------------------------------------------------------------
// 5. XML escaping
// ---------------------------------------------------------------------------

#[test]
fn w74_ampersand_escaped_in_label() {
    let svg = badge_svg("C&C", "10");
    assert!(svg.contains("C&amp;C"));
    assert!(!svg.contains("C&C"));
}

#[test]
fn w74_angle_brackets_escaped() {
    let svg = badge_svg("<tag>", "val");
    assert!(svg.contains("&lt;tag&gt;"));
}

#[test]
fn w74_quotes_escaped() {
    let svg = badge_svg("a\"b'c", "d");
    assert!(svg.contains("&quot;"));
    assert!(svg.contains("&apos;"));
}

// ---------------------------------------------------------------------------
// 6. Very long labels
// ---------------------------------------------------------------------------

#[test]
fn w74_very_long_label_still_wellformed() {
    let long_label = "a".repeat(200);
    let svg = badge_svg(&long_label, "v");
    assert_wellformed(&svg);
    assert!(svg_width(&svg) > 120);
}

// ---------------------------------------------------------------------------
// 7. Empty strings
// ---------------------------------------------------------------------------

#[test]
fn w74_empty_label_and_value_still_wellformed() {
    let svg = badge_svg("", "");
    assert_wellformed(&svg);
    assert!(svg_width(&svg) >= 120);
}
