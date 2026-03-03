//! Deep tests for SVG badge rendering.
//!
//! Covers: XML validity, dimension calculations, color scheme constants,
//! text positioning, metric value formatting (zero, large, decimal),
//! long text handling, and structural invariants.

use tokmd_badge::badge_svg;

// ===========================================================================
// 1. SVG structural validity
// ===========================================================================

#[test]
fn svg_is_well_formed_xml_structure() {
    let svg = badge_svg("label", "value");
    // Must start with <svg and end with </svg>
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>"));

    // Must have xmlns
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));

    // Must have role=img for accessibility
    assert!(svg.contains("role=\"img\""));
}

#[test]
fn svg_contains_exactly_two_rect_elements() {
    let svg = badge_svg("test", "42");
    let rect_count = svg.matches("<rect").count();
    assert_eq!(rect_count, 2, "badge must have exactly 2 rect elements");
}

#[test]
fn svg_contains_exactly_two_text_elements() {
    let svg = badge_svg("test", "42");
    let text_count = svg.matches("<text").count();
    assert_eq!(text_count, 2, "badge must have exactly 2 text elements");
}

#[test]
fn svg_no_unclosed_tags() {
    let svg = badge_svg("test", "42");
    // text elements should be closed with </text>
    let open_text = svg.matches("<text").count();
    let close_text = svg.matches("</text>").count();
    assert_eq!(open_text, close_text);

    // svg element should be opened and closed once
    assert_eq!(svg.matches("<svg ").count(), 1);
    assert_eq!(svg.matches("</svg>").count(), 1);
}

// ===========================================================================
// 2. Dimension constants
// ===========================================================================

#[test]
fn svg_height_is_always_24() {
    for (label, value) in [("a", "1"), ("long label", "long value"), ("", "")] {
        let svg = badge_svg(label, value);
        assert!(
            svg.contains("height=\"24\""),
            "height must be 24 for label={label}"
        );
    }
}

#[test]
fn svg_minimum_total_width_is_120() {
    // Minimum: each segment is at least 60 wide (60+60=120)
    let svg = badge_svg("a", "1");
    let width = extract_width(&svg);
    assert!(width >= 120, "minimum width should be 120, got {width}");
}

#[test]
fn svg_minimum_width_with_empty_strings() {
    let svg = badge_svg("", "");
    let width = extract_width(&svg);
    assert!(
        width >= 120,
        "even empty strings should produce width >= 120"
    );
}

#[test]
fn svg_label_segment_min_width_60() {
    let svg = badge_svg("a", "very long value here");
    // First rect width should be at least 60
    let first_rect_width = extract_first_rect_width(&svg);
    assert!(
        first_rect_width >= 60,
        "label segment width should be at least 60, got {first_rect_width}"
    );
}

#[test]
fn svg_value_segment_min_width_60() {
    let svg = badge_svg("very long label here", "x");
    // Second rect width should be at least 60
    let second_rect_width = extract_second_rect_width(&svg);
    assert!(
        second_rect_width >= 60,
        "value segment width should be at least 60, got {second_rect_width}"
    );
}

// ===========================================================================
// 3. Color scheme
// ===========================================================================

#[test]
fn svg_label_rect_fill_is_dark_gray() {
    let svg = badge_svg("label", "value");
    assert!(svg.contains("fill=\"#555\""));
}

#[test]
fn svg_value_rect_fill_is_blue() {
    let svg = badge_svg("label", "value");
    assert!(svg.contains("fill=\"#4c9aff\""));
}

#[test]
fn svg_text_fill_is_white() {
    let svg = badge_svg("label", "value");
    // Both text elements should use white fill
    assert_eq!(
        svg.matches("fill=\"#fff\"").count(),
        2,
        "both text elements should be white"
    );
}

// ===========================================================================
// 4. Text attributes
// ===========================================================================

#[test]
fn svg_text_font_is_verdana() {
    let svg = badge_svg("x", "y");
    assert!(svg.contains("font-family=\"Verdana\""));
}

#[test]
fn svg_text_font_size_is_11() {
    let svg = badge_svg("x", "y");
    assert!(svg.contains("font-size=\"11\""));
}

#[test]
fn svg_text_anchor_is_middle() {
    let svg = badge_svg("x", "y");
    assert_eq!(
        svg.matches("text-anchor=\"middle\"").count(),
        2,
        "both text elements should be centered"
    );
}

#[test]
fn svg_text_y_position_is_16() {
    let svg = badge_svg("x", "y");
    assert_eq!(
        svg.matches("y=\"16\"").count(),
        2,
        "both text elements should have y=16"
    );
}

// ===========================================================================
// 5. Various metric values
// ===========================================================================

#[test]
fn svg_zero_value() {
    let svg = badge_svg("lines", "0");
    assert!(svg.contains(">0</text>"));
}

#[test]
fn svg_large_number() {
    let svg = badge_svg("lines", "1000000");
    assert!(svg.contains("1000000"));
    let width = extract_width(&svg);
    assert!(width > 120, "large number should produce wider badge");
}

#[test]
fn svg_decimal_value() {
    let svg = badge_svg("coverage", "99.9%");
    assert!(svg.contains("99.9%"));
}

#[test]
fn svg_negative_number() {
    let svg = badge_svg("diff", "-42");
    assert!(svg.contains("-42"));
}

#[test]
fn svg_fraction_value() {
    let svg = badge_svg("ratio", "0.75");
    assert!(svg.contains("0.75"));
}

// ===========================================================================
// 6. Long text handling
// ===========================================================================

#[test]
fn svg_long_label_increases_width() {
    let short = badge_svg("x", "1");
    let long = badge_svg("this is a very very long label text here", "1");
    assert!(
        extract_width(&long) > extract_width(&short),
        "longer label must produce wider badge"
    );
}

#[test]
fn svg_long_value_increases_width() {
    let short = badge_svg("x", "1");
    let long = badge_svg("x", "this is a very very long value text here");
    assert!(
        extract_width(&long) > extract_width(&short),
        "longer value must produce wider badge"
    );
}

#[test]
fn svg_very_long_text_still_valid() {
    let long_label = "a".repeat(100);
    let long_value = "b".repeat(100);
    let svg = badge_svg(&long_label, &long_value);
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains(&long_label));
    assert!(svg.contains(&long_value));
}

// ===========================================================================
// 7. XML escaping
// ===========================================================================

#[test]
fn svg_escapes_all_xml_special_chars() {
    let svg = badge_svg("a&b<c>d\"e'f", "x&y");
    assert!(svg.contains("a&amp;b&lt;c&gt;d&quot;e&apos;f"));
    assert!(svg.contains("x&amp;y"));
}

#[test]
fn svg_does_not_double_escape() {
    let svg = badge_svg("&amp;", "value");
    // "&amp;" should be escaped to "&amp;amp;" in the SVG
    assert!(svg.contains("&amp;amp;"));
}

#[test]
fn svg_preserves_normal_text_unescaped() {
    let svg = badge_svg("hello world", "42 lines");
    assert!(svg.contains("hello world"));
    assert!(svg.contains("42 lines"));
}

// ===========================================================================
// 8. Text centering positions
// ===========================================================================

#[test]
fn svg_label_text_x_is_half_of_label_width() {
    let svg = badge_svg("test", "ok");
    let label_chars = "test".chars().count() as i32;
    let label_width = (label_chars * 7 + 20).max(60);
    let expected_x = label_width / 2;

    let first_rect_w = extract_first_rect_width(&svg);
    assert_eq!(first_rect_w, label_width);

    // First text element x should be label_width / 2
    let text_positions = extract_text_x_positions(&svg);
    assert_eq!(
        text_positions[0], expected_x,
        "label text x should be half of label width"
    );
}

#[test]
fn svg_value_text_x_is_label_width_plus_half_value_width() {
    let svg = badge_svg("test", "ok");
    let label_chars = "test".chars().count() as i32;
    let value_chars = "ok".chars().count() as i32;
    let label_width = (label_chars * 7 + 20).max(60);
    let value_width = (value_chars * 7 + 20).max(60);
    let expected_x = label_width + value_width / 2;

    let text_positions = extract_text_x_positions(&svg);
    assert_eq!(
        text_positions[1], expected_x,
        "value text x should be label_width + value_width/2"
    );
}

// ===========================================================================
// 9. Determinism
// ===========================================================================

#[test]
fn badge_is_deterministic_for_various_inputs() {
    let cases = [
        ("lines", "1234"),
        ("", ""),
        ("日本語", "中文"),
        ("a&b", "c<d"),
        ("very long label", "very long value"),
    ];
    for (label, value) in cases {
        let a = badge_svg(label, value);
        let b = badge_svg(label, value);
        assert_eq!(
            a, b,
            "badge_svg must be deterministic for ({label}, {value})"
        );
    }
}

// ===========================================================================
// 10. Width calculation formula verification
// ===========================================================================

#[test]
fn svg_total_width_equals_label_width_plus_value_width() {
    let svg = badge_svg("hello", "world");
    let total = extract_width(&svg);
    let label_w = extract_first_rect_width(&svg);
    let value_w = extract_second_rect_width(&svg);
    assert_eq!(total, label_w + value_w);
}

#[test]
fn svg_width_formula_matches_char_count_heuristic() {
    let label = "test";
    let value = "42";
    let label_chars = label.chars().count() as i32;
    let value_chars = value.chars().count() as i32;
    let expected_label_w = (label_chars * 7 + 20).max(60);
    let expected_value_w = (value_chars * 7 + 20).max(60);
    let expected_total = expected_label_w + expected_value_w;

    let svg = badge_svg(label, value);
    assert_eq!(extract_width(&svg), expected_total);
}

// ===========================================================================
// Helpers
// ===========================================================================

fn extract_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("width close") + start;
    svg[start..end].parse().expect("numeric width")
}

fn extract_first_rect_width(svg: &str) -> i32 {
    let rect_start = svg.find("<rect").unwrap();
    let rect = &svg[rect_start..];
    let w_start = rect.find("width=\"").unwrap() + 7;
    let w_end = rect[w_start..].find('"').unwrap() + w_start;
    rect[w_start..w_end].parse().unwrap()
}

fn extract_second_rect_width(svg: &str) -> i32 {
    let first = svg.find("<rect").unwrap();
    let rest = &svg[first + 5..];
    let second = rest.find("<rect").unwrap();
    let rect = &rest[second..];
    let w_start = rect.find("width=\"").unwrap() + 7;
    let w_end = rect[w_start..].find('"').unwrap() + w_start;
    rect[w_start..w_end].parse().unwrap()
}

fn extract_text_x_positions(svg: &str) -> Vec<i32> {
    let mut positions = Vec::new();
    let mut search_from = 0;
    while let Some(text_start) = svg[search_from..].find("<text") {
        let abs = search_from + text_start;
        let text = &svg[abs..];
        let x_start = text.find("x=\"").unwrap() + 3;
        let x_end = text[x_start..].find('"').unwrap() + x_start;
        positions.push(text[x_start..x_end].parse().unwrap());
        search_from = abs + 5;
    }
    positions
}
