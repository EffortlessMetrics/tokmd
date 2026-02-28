use tokmd_badge::badge_svg;

// ── Insta snapshot tests for representative badges ──────────────────

#[test]
fn snapshot_lines_badge() {
    let svg = badge_svg("lines", "1234");
    insta::assert_snapshot!("lines_badge", svg);
}

#[test]
fn snapshot_language_pct_badge() {
    let svg = badge_svg("Rust", "78%");
    insta::assert_snapshot!("language_pct_badge", svg);
}

#[test]
fn snapshot_zero_value_badge() {
    let svg = badge_svg("files", "0");
    insta::assert_snapshot!("zero_value_badge", svg);
}

#[test]
fn snapshot_empty_label_badge() {
    let svg = badge_svg("", "42");
    insta::assert_snapshot!("empty_label_badge", svg);
}

#[test]
fn snapshot_long_text_badge() {
    let svg = badge_svg("cyclomatic complexity", "very high");
    insta::assert_snapshot!("long_text_badge", svg);
}

#[test]
fn snapshot_xml_escape_badge() {
    let svg = badge_svg("a<b", "1&2");
    insta::assert_snapshot!("xml_escape_badge", svg);
}

#[test]
fn snapshot_unicode_badge() {
    let svg = badge_svg("言語", "日本語");
    insta::assert_snapshot!("unicode_badge", svg);
}

#[test]
fn snapshot_minimum_width_badge() {
    let svg = badge_svg("a", "1");
    insta::assert_snapshot!("minimum_width_badge", svg);
}

// ── Property: badge always produces valid SVG structure ──────────────

mod properties {
    use proptest::prelude::*;
    use tokmd_badge::badge_svg;

    proptest! {
        #[test]
        fn badge_always_starts_with_svg_tag(
            label in "\\PC{0,50}",
            value in "\\PC{0,50}"
        ) {
            let svg = badge_svg(&label, &value);
            prop_assert!(svg.starts_with("<svg"), "SVG must start with <svg");
            prop_assert!(svg.ends_with("</svg>"), "SVG must end with </svg>");
        }

        #[test]
        fn badge_always_contains_xmlns(
            label in "\\PC{0,30}",
            value in "\\PC{0,30}"
        ) {
            let svg = badge_svg(&label, &value);
            prop_assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        }

        #[test]
        fn badge_always_has_two_rects(
            label in "\\PC{0,30}",
            value in "\\PC{0,30}"
        ) {
            let svg = badge_svg(&label, &value);
            let count = svg.matches("<rect").count();
            prop_assert_eq!(count, 2);
        }

        #[test]
        fn badge_always_has_two_text_elements(
            label in "\\PC{0,30}",
            value in "\\PC{0,30}"
        ) {
            let svg = badge_svg(&label, &value);
            let count = svg.matches("<text").count();
            prop_assert_eq!(count, 2);
        }

        #[test]
        fn badge_width_is_positive(
            label in "\\PC{0,30}",
            value in "\\PC{0,30}"
        ) {
            let svg = badge_svg(&label, &value);
            // Extract the top-level width
            let start = svg.find("width=\"").unwrap() + 7;
            let end = svg[start..].find('"').unwrap() + start;
            let w: i32 = svg[start..end].parse().unwrap();
            prop_assert!(w >= 120, "width {} must be at least 120", w);
        }

        #[test]
        fn badge_never_contains_raw_xml_specials_in_text_nodes(
            label in "[&<>\"']{1,10}",
            value in "[&<>\"']{1,10}"
        ) {
            let svg = badge_svg(&label, &value);
            // The raw label/value should NOT appear literally between > and <
            // (they should be escaped).  A quick check: the raw string with
            // specials must not appear outside of attribute values.
            // We verify the escape entities are present instead.
            if label.contains('&') {
                prop_assert!(svg.contains("&amp;"));
            }
            if label.contains('<') {
                prop_assert!(svg.contains("&lt;"));
            }
        }
    }
}
