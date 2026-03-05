//! Wave 72 property-based invariant tests for tokmd-badge.
//!
//! Covers: SVG structural validity, width positivity, label/value presence,
//! XML escaping, determinism, and dimension scaling.

use proptest::prelude::*;
use tokmd_badge::badge_svg;

/// Extract the root-level SVG `width` attribute value.
fn extract_width(svg: &str) -> Option<i32> {
    let start = svg.find("width=\"")? + 7;
    let end = svg[start..].find('"')? + start;
    svg[start..end].parse().ok()
}

/// Extract the root-level SVG `height` attribute value.
fn extract_height(svg: &str) -> Option<i32> {
    let start = svg.find("height=\"")? + 8;
    let end = svg[start..].find('"')? + start;
    svg[start..end].parse().ok()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // ========================================================================
    // 1. Badge SVG always contains valid XML structure
    // ========================================================================

    #[test]
    fn svg_starts_and_ends_correctly(
        label in "[A-Za-z0-9 ]{1,30}",
        value in "[A-Za-z0-9 ]{1,30}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.starts_with("<svg "), "Must start with <svg");
        prop_assert!(svg.ends_with("</svg>"), "Must end with </svg>");
    }

    #[test]
    fn svg_contains_xmlns(
        label in "[A-Za-z]{1,20}",
        value in "[0-9]{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    #[test]
    fn svg_has_two_rects(
        label in "[A-Za-z ]{1,20}",
        value in "[0-9.%]{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert_eq!(svg.matches("<rect").count(), 2);
    }

    #[test]
    fn svg_has_two_texts(
        label in "[A-Za-z ]{1,20}",
        value in "[0-9.%]{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert_eq!(svg.matches("<text").count(), 2);
    }

    // ========================================================================
    // 2. Badge width is always positive
    // ========================================================================

    #[test]
    fn width_is_positive(
        label in "[A-Za-z0-9]{1,30}",
        value in "[A-Za-z0-9]{1,30}",
    ) {
        let svg = badge_svg(&label, &value);
        let w = extract_width(&svg).expect("width attribute should exist");
        prop_assert!(w > 0, "Width must be positive, got {}", w);
    }

    #[test]
    fn height_is_24(
        label in "[A-Za-z]{1,15}",
        value in "[0-9]{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        let h = extract_height(&svg).expect("height attribute should exist");
        prop_assert_eq!(h, 24);
    }

    // ========================================================================
    // 3. Badge label appears in SVG output
    // ========================================================================

    #[test]
    fn label_appears_in_svg(
        label in "[A-Za-z0-9]{1,20}",
        value in "[A-Za-z0-9]{1,20}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains(&label), "SVG should contain label '{}'", label);
    }

    #[test]
    fn value_appears_in_svg(
        label in "[A-Za-z0-9]{1,20}",
        value in "[A-Za-z0-9]{1,20}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains(&value), "SVG should contain value '{}'", value);
    }

    // ========================================================================
    // 4. XML escaping: special chars don't appear raw
    // ========================================================================

    #[test]
    fn xml_special_chars_are_escaped(
        base in "[A-Za-z]{1,5}",
    ) {
        let dangerous = format!("{}<>&\"'", base);
        let svg = badge_svg(&dangerous, &dangerous);
        // Raw dangerous chars should NOT appear as-is in the label/value text
        prop_assert!(!svg.contains(&dangerous), "Raw special chars should be escaped");
        // Escaped forms should appear
        prop_assert!(svg.contains("&amp;"));
        prop_assert!(svg.contains("&lt;"));
        prop_assert!(svg.contains("&gt;"));
    }

    // ========================================================================
    // 5. Determinism
    // ========================================================================

    #[test]
    fn badge_is_deterministic(
        label in "[A-Za-z0-9 ]{0,20}",
        value in "[A-Za-z0-9 ]{0,20}",
    ) {
        let a = badge_svg(&label, &value);
        let b = badge_svg(&label, &value);
        prop_assert_eq!(a, b);
    }

    // ========================================================================
    // 6. Width scales with text length
    // ========================================================================

    #[test]
    fn longer_label_wider_badge(
        short in "[A-Za-z]{1,3}",
        value in "[0-9]{1,5}",
    ) {
        let long_label = format!("{}{}{}{}", short, short, short, short);
        let short_svg = badge_svg(&short, &value);
        let long_svg = badge_svg(&long_label, &value);
        let w_short = extract_width(&short_svg).unwrap();
        let w_long = extract_width(&long_svg).unwrap();
        prop_assert!(w_long >= w_short, "Longer label should not shrink badge");
    }

    #[test]
    fn longer_value_wider_badge(
        label in "[A-Za-z]{1,5}",
        short_val in "[0-9]{1,3}",
    ) {
        let long_val = format!("{}{}{}{}", short_val, short_val, short_val, short_val);
        let short_svg = badge_svg(&label, &short_val);
        let long_svg = badge_svg(&label, &long_val);
        let w_short = extract_width(&short_svg).unwrap();
        let w_long = extract_width(&long_svg).unwrap();
        prop_assert!(w_long >= w_short, "Longer value should not shrink badge");
    }

    // ========================================================================
    // 7. Width minimum bound
    // ========================================================================

    #[test]
    fn minimum_width_120(
        label in "[A-Za-z]{0,2}",
        value in "[0-9]{0,2}",
    ) {
        let svg = badge_svg(&label, &value);
        let w = extract_width(&svg).unwrap();
        // Each segment has min 60, so total min is 120
        prop_assert!(w >= 120, "Minimum badge width should be 120, got {}", w);
    }
}
