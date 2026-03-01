use tokmd_badge::badge_svg;

// ── Given typical metrics, badge renders label and value ────────────

#[test]
fn given_lines_metric_when_badge_rendered_then_label_and_value_appear() {
    let svg = badge_svg("lines", "1234");
    assert!(svg.contains("lines"));
    assert!(svg.contains("1234"));
}

#[test]
fn given_language_metric_when_badge_rendered_then_both_segments_present() {
    let svg = badge_svg("Rust", "78%");
    assert!(svg.contains("Rust"));
    assert!(svg.contains("78%"));
}

// ── Given any input, badge is structurally valid SVG ────────────────

#[test]
fn given_any_input_when_badge_rendered_then_svg_envelope_is_valid() {
    let svg = badge_svg("test", "42");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("role=\"img\""));
}

#[test]
fn given_any_input_when_badge_rendered_then_two_rects_are_present() {
    let svg = badge_svg("label", "value");
    let rect_count = svg.matches("<rect").count();
    assert_eq!(rect_count, 2, "badge must have exactly two <rect> segments");
}

#[test]
fn given_any_input_when_badge_rendered_then_two_text_elements_exist() {
    let svg = badge_svg("label", "value");
    let text_count = svg.matches("<text").count();
    assert_eq!(text_count, 2, "badge must have exactly two <text> elements");
}

// ── Given default styling, colors are correct ───────────────────────

#[test]
fn given_badge_when_rendered_then_label_rect_has_gray_fill() {
    let svg = badge_svg("x", "y");
    assert!(
        svg.contains("fill=\"#555\""),
        "label segment should be gray"
    );
}

#[test]
fn given_badge_when_rendered_then_value_rect_has_blue_fill() {
    let svg = badge_svg("x", "y");
    assert!(
        svg.contains("fill=\"#4c9aff\""),
        "value segment should be blue"
    );
}

#[test]
fn given_badge_when_rendered_then_text_is_white() {
    let svg = badge_svg("x", "y");
    assert!(svg.contains("fill=\"#fff\""));
}

// ── Given text with XML-special characters, output is escaped ───────

#[test]
fn given_label_with_ampersand_when_rendered_then_it_is_escaped() {
    let svg = badge_svg("a&b", "1");
    assert!(svg.contains("a&amp;b"));
    assert!(!svg.contains("a&b\"") && !svg.contains(">a&b<"));
}

#[test]
fn given_value_with_angle_brackets_when_rendered_then_they_are_escaped() {
    let svg = badge_svg("k", "<script>");
    assert!(svg.contains("&lt;script&gt;"));
    assert!(!svg.contains("<script>"));
}

#[test]
fn given_label_with_quotes_when_rendered_then_they_are_escaped() {
    let svg = badge_svg("a\"b'c", "1");
    assert!(svg.contains("a&quot;b&apos;c"));
}

// ── Given edge-case inputs, badge still renders valid SVG ───────────

#[test]
fn given_zero_value_when_badge_rendered_then_svg_is_valid() {
    let svg = badge_svg("lines", "0");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("0"));
}

#[test]
fn given_very_large_number_when_badge_rendered_then_svg_is_valid() {
    let svg = badge_svg("lines", "999999999");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("999999999"));
}

#[test]
fn given_empty_label_when_badge_rendered_then_svg_is_still_valid() {
    let svg = badge_svg("", "value");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn given_empty_value_when_badge_rendered_then_svg_is_still_valid() {
    let svg = badge_svg("label", "");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn given_both_empty_when_badge_rendered_then_svg_is_still_valid() {
    let svg = badge_svg("", "");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

// ── Given different lengths, width scales proportionally ────────────

#[test]
fn given_short_text_when_badge_rendered_then_minimum_width_is_applied() {
    let svg = badge_svg("a", "1");
    // Each segment min is 60, so total min is 120
    assert!(svg.contains("width=\"120\""));
}

#[test]
fn given_long_text_when_badge_rendered_then_width_exceeds_minimum() {
    let svg = badge_svg("verylonglabel", "verylongvalue");
    // Extract top-level SVG width
    let width = extract_width(&svg);
    assert!(width > 120, "long text badge must be wider than minimum");
}

#[test]
fn given_longer_label_when_compared_to_shorter_then_width_is_larger() {
    let short = badge_svg("a", "1");
    let long = badge_svg("complexity", "high");
    assert!(extract_width(&long) > extract_width(&short));
}

// ── Given unicode text, badge handles it gracefully ─────────────────

#[test]
fn given_unicode_label_when_badge_rendered_then_text_appears() {
    let svg = badge_svg("言語", "42");
    assert!(svg.contains("言語"));
    assert!(svg.contains("42"));
}

#[test]
fn given_emoji_value_when_badge_rendered_then_svg_is_valid() {
    let svg = badge_svg("status", "✅");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("✅"));
}

// ── Given height, it is always 24 ──────────────────────────────────

#[test]
fn given_any_badge_when_rendered_then_height_is_24() {
    let svg = badge_svg("anything", "here");
    assert!(svg.contains("height=\"24\""));
}

// ── Given font properties, Verdana 11 is used ──────────────────────

#[test]
fn given_any_badge_when_rendered_then_font_is_verdana_11() {
    let svg = badge_svg("a", "b");
    assert!(svg.contains("font-family=\"Verdana\""));
    assert!(svg.contains("font-size=\"11\""));
}

// ── Given text elements, they are middle-anchored ───────────────────

#[test]
fn given_any_badge_when_rendered_then_text_is_center_anchored() {
    let svg = badge_svg("a", "b");
    assert!(svg.contains("text-anchor=\"middle\""));
}

// ── Given text elements, y-position is 16 ──────────────────────────

#[test]
fn given_any_badge_when_rendered_then_text_y_is_16() {
    let svg = badge_svg("a", "b");
    let y16_count = svg.matches("y=\"16\"").count();
    assert_eq!(y16_count, 2, "both text elements should have y=16");
}

// ── Given a language count, badge renders valid SVG with count ───────

#[test]
fn given_language_count_when_badge_rendered_then_svg_is_valid_and_contains_count() {
    let svg = badge_svg("languages", "5");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("languages"));
    assert!(svg.contains("5"));
}

// ── Given zero languages, badge shows "0" ───────────────────────────

#[test]
fn given_zero_languages_when_badge_rendered_then_badge_shows_zero() {
    let svg = badge_svg("languages", "0");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("languages"));
    assert!(svg.contains(">0<"));
}

// ── Given very large count, badge still produces valid SVG ──────────

#[test]
fn given_very_large_language_count_when_badge_rendered_then_svg_is_valid() {
    let svg = badge_svg("languages", "9999999");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("9999999"));
    assert_eq!(svg.matches("<rect").count(), 2);
    assert_eq!(svg.matches("<text").count(), 2);
}

// ── Given a custom label, badge contains the label text ─────────────

#[test]
fn given_custom_label_when_badge_rendered_then_badge_contains_label() {
    let svg = badge_svg("my custom metric", "excellent");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("my custom metric"));
    assert!(svg.contains("excellent"));
}

#[test]
fn given_another_custom_label_when_badge_rendered_then_badge_contains_label() {
    let svg = badge_svg("code quality", "A+");
    assert!(svg.contains("code quality"));
    assert!(svg.contains("A+"));
}

// ── helpers ─────────────────────────────────────────────────────────

fn extract_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("width close") + start;
    svg[start..end].parse().expect("numeric width")
}
