//! Property-based tests for tokmd-badge SVG rendering.
//!
//! Covers: determinism, structural invariants, dimension monotonicity,
//! XML escaping safety, and width formula verification.

use proptest::prelude::*;
use tokmd_badge::badge_svg;

// =========================================================================
// Determinism: same inputs always produce identical SVG
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn badge_is_deterministic(
        label in "[a-zA-Z0-9 ]{0,30}",
        value in "[a-zA-Z0-9 .%]{0,20}",
    ) {
        let a = badge_svg(&label, &value);
        let b = badge_svg(&label, &value);
        prop_assert_eq!(a, b, "badge_svg must be deterministic");
    }
}

// =========================================================================
// Structural invariants
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn svg_always_well_formed(
        label in "[a-zA-Z0-9 ]{0,30}",
        value in "[a-zA-Z0-9 .%]{0,20}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.starts_with("<svg "), "Must start with <svg");
        prop_assert!(svg.ends_with("</svg>"), "Must end with </svg>");
        prop_assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    #[test]
    fn svg_always_has_two_rects(
        label in "[a-zA-Z0-9 ]{0,20}",
        value in "[a-zA-Z0-9 ]{0,20}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert_eq!(svg.matches("<rect").count(), 2, "Must have exactly 2 rects");
    }

    #[test]
    fn svg_always_has_two_texts(
        label in "[a-zA-Z0-9 ]{0,20}",
        value in "[a-zA-Z0-9 ]{0,20}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert_eq!(svg.matches("<text").count(), 2, "Must have exactly 2 text elements");
        prop_assert_eq!(svg.matches("</text>").count(), 2, "Must have 2 closing text tags");
    }

    #[test]
    fn svg_height_always_24(
        label in "[a-zA-Z0-9 ]{0,20}",
        value in "[a-zA-Z0-9 ]{0,20}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("height=\"24\""), "Height must be 24");
    }
}

// =========================================================================
// Width monotonicity: longer text produces wider badge
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn longer_label_wider_badge(
        short_label in "[a-z]{1,3}",
        extra in "[a-z]{10,20}",
        value in "[a-z]{1,5}",
    ) {
        let long_label = format!("{}{}", short_label, extra);
        let short_svg = badge_svg(&short_label, &value);
        let long_svg = badge_svg(&long_label, &value);
        let short_w = extract_svg_width(&short_svg);
        let long_w = extract_svg_width(&long_svg);
        prop_assert!(long_w >= short_w,
            "Longer label should produce wider badge: {} vs {}", long_w, short_w);
    }

    #[test]
    fn longer_value_wider_badge(
        label in "[a-z]{1,5}",
        short_value in "[a-z]{1,3}",
        extra in "[a-z]{10,20}",
    ) {
        let long_value = format!("{}{}", short_value, extra);
        let short_svg = badge_svg(&label, &short_value);
        let long_svg = badge_svg(&label, &long_value);
        let short_w = extract_svg_width(&short_svg);
        let long_w = extract_svg_width(&long_svg);
        prop_assert!(long_w >= short_w,
            "Longer value should produce wider badge: {} vs {}", long_w, short_w);
    }
}

// =========================================================================
// Width formula: total = label_width + value_width, each >= 60
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn total_width_at_least_120(
        label in "[a-zA-Z0-9]{0,30}",
        value in "[a-zA-Z0-9]{0,20}",
    ) {
        let svg = badge_svg(&label, &value);
        let width = extract_svg_width(&svg);
        prop_assert!(width >= 120, "Total width must be >= 120, got {}", width);
    }

    #[test]
    fn width_matches_char_count_formula(
        label in "[a-zA-Z0-9]{0,30}",
        value in "[a-zA-Z0-9]{0,20}",
    ) {
        let label_chars = label.chars().count() as i32;
        let value_chars = value.chars().count() as i32;
        let expected_label_w = (label_chars * 7 + 20).max(60);
        let expected_value_w = (value_chars * 7 + 20).max(60);
        let expected_total = expected_label_w + expected_value_w;
        let svg = badge_svg(&label, &value);
        let actual = extract_svg_width(&svg);
        prop_assert_eq!(actual, expected_total,
            "Width mismatch for label='{}' value='{}'", label, value);
    }
}

// =========================================================================
// XML escaping safety
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn svg_escapes_ampersand(
        prefix in "[a-z]{1,5}",
        suffix in "[a-z]{1,5}",
    ) {
        let label = format!("{}&{}", prefix, suffix);
        let svg = badge_svg(&label, "ok");
        prop_assert!(
            !svg.contains(&format!(">{}<", label)),
            "Raw ampersand should be escaped in SVG"
        );
        prop_assert!(svg.contains("&amp;"), "Ampersand should be escaped to &amp;");
    }

    #[test]
    fn svg_escapes_angle_brackets(
        prefix in "[a-z]{1,5}",
        suffix in "[a-z]{1,5}",
    ) {
        let label = format!("{}<{}", prefix, suffix);
        let svg = badge_svg(&label, "ok");
        prop_assert!(svg.contains("&lt;"), "< should be escaped to &lt;");
    }
}

// =========================================================================
// Helper
// =========================================================================

fn extract_svg_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("width close") + start;
    svg[start..end].parse().expect("numeric width")
}
