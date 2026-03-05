//! W59 – Property-based tests for `badge_svg`.

use proptest::prelude::*;
use tokmd_badge::badge_svg;

/// Extract the root-level SVG `width` attribute.
fn extract_width(svg: &str) -> Option<i32> {
    let start = svg.find("width=\"")? + 7;
    let end = svg[start..].find('"')? + start;
    svg[start..end].parse().ok()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    // ── Structure invariants ───────────────────────────────────────────

    #[test]
    fn svg_always_opens_and_closes(
        label in ".*{0,50}",
        value in ".*{0,50}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.starts_with("<svg "));
        prop_assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn svg_always_contains_xmlns(
        label in "[A-Za-z0-9]{0,20}",
        value in "[A-Za-z0-9]{0,20}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    #[test]
    fn svg_always_contains_role_img(
        label in "[A-Za-z]{1,10}",
        value in "[0-9]{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("role=\"img\""));
    }

    #[test]
    fn svg_always_has_two_rects(
        label in "[A-Za-z ]{1,20}",
        value in "[0-9.%]{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        let count = svg.matches("<rect").count();
        prop_assert_eq!(count, 2);
    }

    #[test]
    fn svg_always_has_two_texts(
        label in "[A-Za-z ]{1,20}",
        value in "[0-9.%]{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        let count = svg.matches("<text").count();
        prop_assert_eq!(count, 2);
    }

    // ── Determinism ────────────────────────────────────────────────────

    #[test]
    fn svg_is_deterministic(
        label in ".*{0,30}",
        value in ".*{0,30}",
    ) {
        let a = badge_svg(&label, &value);
        let b = badge_svg(&label, &value);
        prop_assert_eq!(a, b);
    }

    // ── Width invariants ───────────────────────────────────────────────

    #[test]
    fn width_is_at_least_120(
        label in ".*{0,40}",
        value in ".*{0,40}",
    ) {
        let svg = badge_svg(&label, &value);
        if let Some(w) = extract_width(&svg) {
            prop_assert!(w >= 120, "width {} < 120", w);
        }
    }

    #[test]
    fn width_is_positive(
        label in "[A-Za-z]{1,15}",
        value in "[0-9]{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        if let Some(w) = extract_width(&svg) {
            prop_assert!(w > 0);
        }
    }

    #[test]
    fn longer_label_means_wider_or_equal(
        base in "[A-Za-z]{1,5}",
        extra in "[A-Za-z]{1,20}",
    ) {
        let short = badge_svg(&base, "1");
        let long = badge_svg(&format!("{base}{extra}"), "1");
        let w_short = extract_width(&short).unwrap_or(0);
        let w_long = extract_width(&long).unwrap_or(0);
        prop_assert!(w_long >= w_short, "longer label should not shrink width");
    }

    #[test]
    fn longer_value_means_wider_or_equal(
        base in "[0-9]{1,3}",
        extra in "[0-9]{1,15}",
    ) {
        let short = badge_svg("k", &base);
        let long = badge_svg("k", &format!("{base}{extra}"));
        let w_short = extract_width(&short).unwrap_or(0);
        let w_long = extract_width(&long).unwrap_or(0);
        prop_assert!(w_long >= w_short, "longer value should not shrink width");
    }

    // ── Height invariant ───────────────────────────────────────────────

    #[test]
    fn height_is_always_24(
        label in ".*{0,20}",
        value in ".*{0,20}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("height=\"24\""));
    }

    // ── Escaping invariants ────────────────────────────────────────────

    #[test]
    fn no_raw_ampersand_in_text(
        label in "[A-Za-z&]{1,10}",
        value in "[A-Za-z&]{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        // After the opening <svg tag, raw `&` must not appear unless followed
        // by `amp;`, `lt;`, `gt;`, `quot;`, or `apos;`.
        // Simple check: no unescaped `&` in the text content portions.
        // Since the entire SVG is single-line, verify &amp; is used for labels
        // that contain `&`.
        if label.contains('&') {
            prop_assert!(svg.contains("&amp;"), "label & should be escaped");
        }
    }

    #[test]
    fn no_raw_angle_brackets_in_text_content(
        label in "[A-Za-z<>]{1,10}",
        value in "[A-Za-z<>]{1,10}",
    ) {
        let svg = badge_svg(&label, &value);
        if label.contains('<') {
            prop_assert!(svg.contains("&lt;"), "< should be escaped in label");
        }
        if label.contains('>') {
            prop_assert!(svg.contains("&gt;"), "> should be escaped in label");
        }
    }

    // ── Colour invariants ──────────────────────────────────────────────

    #[test]
    fn always_has_label_colour(
        label in "[A-Za-z]{1,10}",
        value in "[0-9]{1,5}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("fill=\"#555\""), "label rect colour missing");
    }

    #[test]
    fn always_has_value_colour(
        label in "[A-Za-z]{1,10}",
        value in "[0-9]{1,5}",
    ) {
        let svg = badge_svg(&label, &value);
        prop_assert!(svg.contains("fill=\"#4c9aff\""), "value rect colour missing");
    }
}
