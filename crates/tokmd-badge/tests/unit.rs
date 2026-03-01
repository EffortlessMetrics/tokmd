use tokmd_badge::badge_svg;

#[test]
fn svg_starts_and_ends_with_svg_tags() {
    let svg = badge_svg("label", "value");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn svg_contains_xmlns_attribute() {
    let svg = badge_svg("x", "y");
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
}

#[test]
fn svg_contains_role_img() {
    let svg = badge_svg("lang", "Rust");
    assert!(svg.contains("role=\"img\""));
}

#[test]
fn svg_contains_label_and_value_text() {
    let svg = badge_svg("lines", "1234");
    assert!(svg.contains("lines"));
    assert!(svg.contains("1234"));
}

#[test]
fn svg_escapes_ampersand_in_label() {
    let svg = badge_svg("A&B", "ok");
    assert!(svg.contains("A&amp;B"));
    assert!(!svg.contains("A&B"));
}

#[test]
fn svg_escapes_angle_brackets() {
    let svg = badge_svg("<script>", "val");
    assert!(svg.contains("&lt;script&gt;"));
    assert!(!svg.contains("<script>"));
}

#[test]
fn svg_escapes_quotes() {
    let svg = badge_svg("a\"b", "c'd");
    assert!(svg.contains("a&quot;b"));
    assert!(svg.contains("c&apos;d"));
}

#[test]
fn svg_minimum_width_enforced() {
    // Even a single-char label/value should produce at least width 120 (60+60).
    let svg = badge_svg("a", "1");
    let w = extract_width(&svg);
    assert!(w >= 120, "minimum width should be 120, got {w}");
}

#[test]
fn svg_width_grows_with_longer_text() {
    let short = badge_svg("hi", "1");
    let long = badge_svg("a_very_long_label", "a_very_long_value");
    assert!(
        extract_width(&long) > extract_width(&short),
        "longer text should produce wider badge"
    );
}

#[test]
fn svg_label_segment_uses_dark_fill() {
    let svg = badge_svg("k", "v");
    assert!(svg.contains("fill=\"#555\""));
}

#[test]
fn svg_value_segment_uses_blue_fill() {
    let svg = badge_svg("k", "v");
    assert!(svg.contains("fill=\"#4c9aff\""));
}

#[test]
fn svg_text_uses_verdana_font() {
    let svg = badge_svg("k", "v");
    assert!(svg.contains("font-family=\"Verdana\""));
}

#[test]
fn svg_text_is_white() {
    let svg = badge_svg("k", "v");
    // All text elements should be white.
    assert!(svg.contains("fill=\"#fff\""));
}

#[test]
fn svg_height_is_24() {
    let svg = badge_svg("k", "v");
    assert!(svg.contains("height=\"24\""));
}

#[test]
fn svg_empty_strings_produce_valid_output() {
    let svg = badge_svg("", "");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(extract_width(&svg) >= 120);
}

#[test]
fn svg_unicode_label_and_value() {
    let svg = badge_svg("日本語", "中文");
    assert!(svg.contains("日本語"));
    assert!(svg.contains("中文"));
    // Width should scale with char count, not byte count.
    assert!(extract_width(&svg) >= 120);
}

/// Helper to extract the top-level `width` attribute from the SVG root element.
fn extract_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("width close") + start;
    svg[start..end].parse().expect("numeric width")
}
