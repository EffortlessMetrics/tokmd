//! Additional property-based tests for tokmd-badge SVG rendering (wave 40).
//!
//! Covers: badge text containment, positive width, non-empty output,
//! empty-string handling, and Unicode width safety.

use proptest::prelude::*;
use tokmd_badge::badge_svg;

// =========================================================================
// Badge text contains the formatted value (after XML escaping)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn badge_contains_label_text(
        label in "[a-zA-Z0-9]{1,20}",
        value in "[a-zA-Z0-9]{1,15}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains(&label),
            "Badge SVG should contain label '{}'", label);
    }

    #[test]
    fn badge_contains_value_text(
        label in "[a-zA-Z0-9]{1,20}",
        value in "[a-zA-Z0-9]{1,15}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains(&value),
            "Badge SVG should contain value '{}'", value);
    }
}

// =========================================================================
// Badge width is always positive
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn badge_width_always_positive(
        label in ".*",
        value in ".*",
    ) {
        let svg = badge_svg(&label, &value);
        let width = extract_svg_width(&svg);
        prop_assert!(width > 0, "Badge width must be positive, got {}", width);
    }
}

// =========================================================================
// Badge output is never empty
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn badge_output_never_empty(
        label in ".{0,50}",
        value in ".{0,30}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(!svg.is_empty(), "Badge SVG must not be empty");
        prop_assert!(svg.len() > 100, "Badge SVG should be a substantial string");
    }
}

// =========================================================================
// Empty inputs produce valid SVG
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn empty_label_produces_valid_svg(
        value in "[a-zA-Z0-9]{0,20}",
    ) {
        let svg = badge_svg("", &value);
        prop_assert!(svg.starts_with("<svg "), "Must start with <svg");
        prop_assert!(svg.ends_with("</svg>"), "Must end with </svg>");
    }

    #[test]
    fn empty_value_produces_valid_svg(
        label in "[a-zA-Z0-9]{0,20}",
    ) {
        let svg = badge_svg(&label, "");
        prop_assert!(svg.starts_with("<svg "), "Must start with <svg");
        prop_assert!(svg.ends_with("</svg>"), "Must end with </svg>");
    }
}

// =========================================================================
// Numeric value formatting: digits appear in badge
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn badge_contains_formatted_number(
        n in 0u64..1_000_000,
    ) {
        let formatted = n.to_string();
        let svg = badge_svg("lines", &formatted);
        prop_assert!(svg.contains(&formatted),
            "Badge should contain formatted number '{}'", formatted);
    }
}

// =========================================================================
// Width formula consistency: label+value width = total
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn width_is_sum_of_segments(
        label in "[a-zA-Z0-9]{0,25}",
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
            "Width {} != expected {} for label='{}' value='{}'",
            actual, expected_total, label, value);
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
