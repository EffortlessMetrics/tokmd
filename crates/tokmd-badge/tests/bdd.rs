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

// ── helpers ─────────────────────────────────────────────────────────

fn extract_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("width close") + start;
    svg[start..end].parse().expect("numeric width")
}

// ── Given numeric label, badge renders correctly ────────────────────

#[test]
fn given_numeric_label_when_badge_rendered_then_both_numbers_appear() {
    let svg = badge_svg("42", "99");
    assert!(svg.contains(">42<"));
    assert!(svg.contains(">99<"));
}

// ── Given identical label and value, badge still valid ──────────────

#[test]
fn given_identical_label_and_value_when_badge_rendered_then_both_text_elements_present() {
    let svg = badge_svg("same", "same");
    let text_count = svg.matches("<text").count();
    assert_eq!(text_count, 2, "even with identical text, two text elements");
    assert!(svg.contains(">same<"));
}

// ── Given very long text, width grows accordingly ───────────────────

#[test]
fn given_100_char_label_when_badge_rendered_then_width_exceeds_minimum_greatly() {
    let long_label = "a".repeat(100);
    let svg = badge_svg(&long_label, "x");
    let width = extract_width(&svg);
    assert!(width > 500, "100 char label should produce a wide badge: got {width}");
}

// ── Given all XML special chars combined, all escaped ───────────────

#[test]
fn given_all_xml_special_chars_when_badge_rendered_then_all_escaped() {
    let svg = badge_svg("<&>\"'", "<&>\"'");
    // No raw special chars in text content
    assert!(!svg.contains(">&<"));
    assert!(svg.contains("&lt;"));
    assert!(svg.contains("&amp;"));
    assert!(svg.contains("&gt;"));
    assert!(svg.contains("&quot;"));
    assert!(svg.contains("&apos;"));
}

// ── Given newlines in text, badge still valid SVG ───────────────────

#[test]
fn given_newline_in_label_when_badge_rendered_then_svg_is_valid() {
    let svg = badge_svg("line\none", "val");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

// ── Given spaces in text, badge renders correctly ───────────────────

#[test]
fn given_spaces_in_label_when_badge_rendered_then_text_preserved() {
    let svg = badge_svg("code lines", "1 234");
    assert!(svg.contains("code lines"));
    assert!(svg.contains("1 234"));
}

// ── Given width calculation, label and value segments sum to total ──

#[test]
fn given_any_badge_when_rendered_then_total_width_equals_sum_of_segments() {
    let svg = badge_svg("test", "value");
    // Extract total width
    let total_width = extract_width(&svg);

    // Extract individual rect widths (label rect width="X" and value rect width="Y")
    let rects: Vec<i32> = svg
        .match_indices("width=\"")
        .skip(1) // skip the SVG element width
        .take(2) // take the two rect widths
        .map(|(pos, _)| {
            let start = pos + 7;
            let end = svg[start..].find('"').unwrap() + start;
            svg[start..end].parse::<i32>().unwrap()
        })
        .collect();

    assert_eq!(rects.len(), 2, "should find two rect width attributes");
    assert_eq!(
        rects[0] + rects[1],
        total_width,
        "segment widths should sum to total"
    );
}

// ── Given minimum width enforcement, both segments at least 60 ─────

#[test]
fn given_single_char_inputs_when_badge_rendered_then_each_segment_at_least_60() {
    let svg = badge_svg("a", "b");
    // Total width should be exactly 120 (60 + 60)
    assert_eq!(extract_width(&svg), 120);
}
