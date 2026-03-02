//! Property-based tests for tokmd-badge SVG generation.

use proptest::prelude::*;
use tokmd_badge::badge_svg;

fn extract_svg_width(svg: &str) -> Option<f64> {
    let start = svg.find("width=\"")? + 7;
    let end = svg[start..].find('"')? + start;
    svg[start..end].parse().ok()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn badge_svg_well_formed(
        label in "[A-Za-z0-9 ]{1,30}",
        value in "[A-Za-z0-9.% ]{1,20}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.starts_with("<svg "));
        prop_assert!(svg.trim_end().ends_with("</svg>"));
    }

    #[test]
    fn badge_svg_xmlns(
        label in "[A-Za-z]{1,10}",
        value in "[0-9]{1,5}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    #[test]
    fn badge_svg_height(
        label in "[A-Za-z]{1,10}",
        value in "[0-9]{1,5}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("height=\"20\""));
    }

    #[test]
    fn badge_svg_deterministic(
        label in "[A-Za-z]{1,10}",
        value in "[0-9]{1,5}",
    ) {
        let a = badge_svg(&label, &value);
        let b = badge_svg(&label, &value);
        prop_assert_eq!(a, b);
    }

    #[test]
    fn badge_svg_width(
        label in "[A-Za-z]{1,10}",
        value in "[0-9]{1,5}",
    ) {
        let svg = badge_svg(&label, &value);
        let w = extract_svg_width(&svg);
        prop_assert!(w.is_some());
        prop_assert!(w.unwrap() > 0.0);
    }

    #[test]
    fn badge_svg_escaping(
        label in "[<>&]{1,5}",
        value in "[<>&]{1,5}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.starts_with("<svg "));
        prop_assert!(svg.trim_end().ends_with("</svg>"));
    }

    #[test]
    fn badge_svg_monotonicity(base in "[A-Za-z]{1,5}") {
        let short_svg = badge_svg(&base, "1");
        let long_label = format!("{}{}{}", base, base, base);
        let long_svg = badge_svg(&long_label, "1");
        let w_short = extract_svg_width(&short_svg).unwrap_or(0.0);
        let w_long = extract_svg_width(&long_svg).unwrap_or(0.0);
        prop_assert!(w_long >= w_short);
    }

    #[test]
    fn badge_svg_rects(
        label in "[A-Za-z]{1,10}",
        value in "[0-9]{1,5}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("<rect"));
    }

    #[test]
    fn badge_svg_texts(
        label in "[A-Za-z]{1,10}",
        value in "[0-9]{1,5}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("<text"));
    }
}
