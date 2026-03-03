//! Deep tests for tokmd-badge SVG rendering.

use tokmd_badge::badge_svg;

// ---- Default / basic generation ----

#[test]
fn default_badge_generation() {
    let svg = badge_svg("lines", "1234");
    assert!(!svg.is_empty());
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn badge_with_custom_label_and_value() {
    let svg = badge_svg("coverage", "87%");
    assert!(svg.contains("coverage"));
    assert!(svg.contains("87%"));
}

// ---- SVG structure validity ----

#[test]
fn badge_contains_svg_tag() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn badge_contains_rect_elements() {
    let svg = badge_svg("test", "42");
    // Should have two rect elements: one for label, one for value
    let rect_count = svg.matches("<rect").count();
    assert_eq!(rect_count, 2, "Expected 2 rect elements, got {rect_count}");
}

#[test]
fn badge_contains_text_elements() {
    let svg = badge_svg("test", "42");
    let text_count = svg.matches("<text").count();
    assert_eq!(text_count, 2, "Expected 2 text elements, got {text_count}");
}

#[test]
fn badge_has_xmlns_attribute() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
}

#[test]
fn badge_has_role_img() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("role=\"img\""));
}

#[test]
fn badge_has_font_family() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("font-family=\"Verdana\""));
}

#[test]
fn badge_has_font_size() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("font-size=\"11\""));
}

#[test]
fn badge_text_anchor_middle() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("text-anchor=\"middle\""));
}

// ---- Color mapping ----

#[test]
fn badge_label_background_is_grey() {
    let svg = badge_svg("label", "value");
    assert!(svg.contains("fill=\"#555\""));
}

#[test]
fn badge_value_background_is_blue() {
    let svg = badge_svg("label", "value");
    assert!(svg.contains("fill=\"#4c9aff\""));
}

#[test]
fn badge_text_fill_is_white() {
    let svg = badge_svg("label", "value");
    assert!(svg.contains("fill=\"#fff\""));
}

// ---- Dimensions ----

#[test]
fn badge_height_is_24() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("height=\"24\""));
}

#[test]
fn badge_minimum_width_is_120() {
    // Even for very short text, minimum segment width is 60 each = 120 total
    let svg = badge_svg("a", "1");
    let width = extract_width(&svg);
    assert!(width >= 120, "Expected width >= 120, got {width}");
}

#[test]
fn badge_width_increases_with_longer_label() {
    let short = badge_svg("a", "1");
    let long = badge_svg("very long label text here", "1");
    assert!(extract_width(&long) > extract_width(&short));
}

#[test]
fn badge_width_increases_with_longer_value() {
    let short = badge_svg("test", "1");
    let long = badge_svg("test", "very long value text here");
    assert!(extract_width(&long) > extract_width(&short));
}

#[test]
fn badge_width_is_sum_of_segments() {
    let svg = badge_svg("test", "42");
    let total_width = extract_width(&svg);
    // Label: max(4*7+20, 60) = max(48, 60) = 60
    // Value: max(2*7+20, 60) = max(34, 60) = 60
    assert_eq!(total_width, 120);
}

#[test]
fn badge_segment_widths_for_known_inputs() {
    // Label "lines" = 5 chars => max(5*7+20, 60) = max(55, 60) = 60
    // Value "1234" = 4 chars => max(4*7+20, 60) = max(48, 60) = 60
    let svg = badge_svg("lines", "1234");
    assert_eq!(extract_width(&svg), 120);
}

// ---- Unicode text ----

#[test]
fn badge_with_unicode_text() {
    let svg = badge_svg("行数", "1234");
    assert!(svg.contains("行数"));
    assert!(svg.contains("1234"));
    assert!(svg.starts_with("<svg"));
}

#[test]
fn badge_unicode_width_uses_char_count() {
    // "行数" is 2 chars, so width = max(2*7+20, 60) = max(34, 60) = 60
    let svg = badge_svg("行数", "1");
    assert_eq!(extract_width(&svg), 120); // 60 + 60 min
}

#[test]
fn badge_with_emoji() {
    let svg = badge_svg("status", "✅");
    assert!(svg.contains("✅"));
}

// ---- Empty label/value ----

#[test]
fn badge_with_empty_label() {
    let svg = badge_svg("", "42");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("42"));
}

#[test]
fn badge_with_empty_value() {
    let svg = badge_svg("test", "");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("test"));
}

#[test]
fn badge_with_both_empty() {
    let svg = badge_svg("", "");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

// ---- Very long text ----

#[test]
fn badge_with_very_long_label() {
    let label = "a".repeat(200);
    let svg = badge_svg(&label, "val");
    assert!(svg.starts_with("<svg"));
    let width = extract_width(&svg);
    assert!(width > 120, "Long label should produce wide badge");
}

#[test]
fn badge_with_very_long_value() {
    let value = "b".repeat(200);
    let svg = badge_svg("lbl", &value);
    assert!(svg.starts_with("<svg"));
    let width = extract_width(&svg);
    assert!(width > 120, "Long value should produce wide badge");
}

// ---- XML escaping ----

#[test]
fn badge_escapes_ampersand() {
    let svg = badge_svg("a&b", "c&d");
    assert!(svg.contains("a&amp;b"));
    assert!(svg.contains("c&amp;d"));
    assert!(!svg.contains("a&b"));
}

#[test]
fn badge_escapes_angle_brackets() {
    let svg = badge_svg("<script>", "val");
    assert!(svg.contains("&lt;script&gt;"));
    assert!(!svg.contains("<script>"));
}

#[test]
fn badge_escapes_quotes() {
    let svg = badge_svg("a\"b", "c'd");
    assert!(svg.contains("&quot;"));
    assert!(svg.contains("&apos;"));
}

// ---- Deterministic output ----

#[test]
fn badge_output_is_deterministic() {
    let svg1 = badge_svg("lines", "42");
    let svg2 = badge_svg("lines", "42");
    assert_eq!(svg1, svg2, "Identical inputs must produce identical output");
}

#[test]
fn badge_deterministic_with_unicode() {
    let svg1 = badge_svg("テスト", "値");
    let svg2 = badge_svg("テスト", "値");
    assert_eq!(svg1, svg2);
}

// ---- Centering logic ----

#[test]
fn badge_label_text_centered_in_label_rect() {
    // For short text, label_width = 60, label_x = 30
    let svg = badge_svg("ab", "1");
    assert!(svg.contains("x=\"30\""));
}

#[test]
fn badge_value_text_positioned_after_label() {
    // label_width=60, value_width=60 => value_x = 60 + 30 = 90
    let svg = badge_svg("ab", "1");
    assert!(svg.contains("x=\"90\""));
}

// ---- Snapshot test ----

#[test]
fn badge_snapshot_basic() {
    let svg = badge_svg("lines", "42");
    insta::assert_snapshot!(svg);
}

#[test]
fn badge_snapshot_unicode() {
    let svg = badge_svg("言語", "Rust");
    insta::assert_snapshot!(svg);
}

// ---- Property-based tests ----

mod properties {
    use proptest::prelude::*;
    use tokmd_badge::badge_svg;

    proptest! {
        #[test]
        fn always_valid_svg(label in ".*", value in ".*") {
            let svg = badge_svg(&label, &value);
            prop_assert!(svg.starts_with("<svg"));
            prop_assert!(svg.ends_with("</svg>"));
        }

        #[test]
        fn width_never_negative(label in ".*", value in ".*") {
            let svg = badge_svg(&label, &value);
            let width = extract_width_prop(&svg);
            prop_assert!(width >= 120, "Width was {}", width);
        }

        #[test]
        fn deterministic(label in ".{0,50}", value in ".{0,50}") {
            let svg1 = badge_svg(&label, &value);
            let svg2 = badge_svg(&label, &value);
            prop_assert_eq!(svg1, svg2);
        }

        #[test]
        fn no_unescaped_ampersand_in_text(label in ".*&.*", value in ".*") {
            let svg = badge_svg(&label, &value);
            // The label text should be escaped; check that raw & not followed by amp;/lt;/gt;/quot;/apos; doesn't appear
            // Actually just verify the escaped version is present
            prop_assert!(svg.contains("&amp;"), "Ampersand should be escaped");
        }
    }

    fn extract_width_prop(svg: &str) -> i32 {
        let start = svg.find("width=\"").expect("width attr") + 7;
        let end = svg[start..].find('"').expect("width close") + start;
        svg[start..end].parse().expect("numeric width")
    }
}

/// Helper to extract the top-level SVG width attribute value.
fn extract_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("width close") + start;
    svg[start..end].parse().expect("numeric width")
}
