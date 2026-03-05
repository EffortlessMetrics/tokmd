//! W53: Property-based tests for `tokmd-badge` SVG generation.
//!
//! Covers: well-formed XML structure, escaping safety, dimension invariants,
//! determinism, and boundary conditions for arbitrary inputs.

use proptest::prelude::*;
use tokmd_badge::badge_svg;

fn extract_svg_width(svg: &str) -> Option<i32> {
    let start = svg.find("width=\"")? + 7;
    let end = svg[start..].find('"')? + start;
    svg[start..end].parse().ok()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 1. SVG always starts with <svg and ends with </svg> for alphanumeric inputs
    #[test]
    fn well_formed_svg_structure(
        label in "[A-Za-z0-9 ]{1,50}",
        value in "[A-Za-z0-9.% ]{1,30}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.starts_with("<svg "), "missing <svg open: {}", &svg[..40.min(svg.len())]);
        prop_assert!(svg.ends_with("</svg>"), "missing </svg> close");
    }

    // 2. SVG contains required xmlns attribute
    #[test]
    fn svg_has_xmlns(label in "\\PC{1,20}", value in "\\PC{1,20}") {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    // 3. SVG has fixed height of 24
    #[test]
    fn svg_fixed_height(label in "\\PC{1,20}", value in "\\PC{1,20}") {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("height=\"24\""));
    }

    // 4. Determinism: same inputs → same output
    #[test]
    fn deterministic_output(label in "\\PC{1,30}", value in "\\PC{1,30}") {
        let a = badge_svg(&label, &value);
        let b = badge_svg(&label, &value);
        prop_assert_eq!(a, b);
    }

    // 5. Width is always positive
    #[test]
    fn positive_width(label in "[A-Za-z]{1,20}", value in "[0-9]{1,10}") {
        let svg = badge_svg(&label, &value);
        let w = extract_svg_width(&svg);
        prop_assert!(w.is_some());
        prop_assert!(w.unwrap() > 0);
    }

    // 6. Width monotonicity: longer label → wider badge
    #[test]
    fn width_monotonic_label(base in "[A-Za-z]{1,5}") {
        let short = badge_svg(&base, "1");
        let long_label = base.repeat(4);
        let long = badge_svg(&long_label, "1");
        let w_short = extract_svg_width(&short).unwrap_or(0);
        let w_long = extract_svg_width(&long).unwrap_or(0);
        prop_assert!(w_long >= w_short, "long={} < short={}", w_long, w_short);
    }

    // 7. XML-special characters are escaped (no raw < > & in text nodes)
    #[test]
    fn xml_special_chars_escaped(
        label in "[<>&\"']{1,10}",
        value in "[<>&\"']{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        // SVG should be well-formed despite special chars in input
        prop_assert!(svg.starts_with("<svg "));
        prop_assert!(svg.ends_with("</svg>"));
        // Raw label/value with specials should NOT appear (they get escaped)
        if label.contains('<') {
            let needle = [">", &label, "<"].concat();
            prop_assert!(!svg.contains(&needle));
        }
    }

    // 8. Contains exactly two <rect elements
    #[test]
    fn contains_two_rects(label in "[A-Za-z]{1,10}", value in "[0-9]{1,5}") {
        let svg = badge_svg(&label, &value);
        let count = svg.matches("<rect").count();
        prop_assert_eq!(count, 2, "expected 2 <rect>, got {}", count);
    }

    // 9. Contains exactly two <text elements
    #[test]
    fn contains_two_texts(label in "[A-Za-z]{1,10}", value in "[0-9]{1,5}") {
        let svg = badge_svg(&label, &value);
        let count = svg.matches("<text").count();
        prop_assert_eq!(count, 2, "expected 2 <text>, got {}", count);
    }

    // 10. Minimum width is 120 (60+60)
    #[test]
    fn minimum_width_120(label in "[A-Za-z]{1,5}", value in "[0-9]{1,3}") {
        let svg = badge_svg(&label, &value);
        let w = extract_svg_width(&svg).unwrap_or(0);
        prop_assert!(w >= 120, "width {} < 120", w);
    }

    // 11. Empty strings produce valid SVG
    #[test]
    fn empty_strings_valid(_dummy in 0..1u8) {
        let svg = badge_svg("", "");
        prop_assert!(svg.starts_with("<svg "));
        prop_assert!(svg.ends_with("</svg>"));
        let w = extract_svg_width(&svg).unwrap_or(0);
        prop_assert!(w >= 120);
    }

    // 12. role="img" accessibility attribute present
    #[test]
    fn has_role_img(label in "\\PC{1,10}", value in "\\PC{1,10}") {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("role=\"img\""));
    }
}
