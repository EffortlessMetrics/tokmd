//! Additional coverage tests for `tokmd-badge`.
//!
//! Targets determinism, structural invariants, and edge cases
//! not covered by the existing BDD / unit / snapshot suites.

use tokmd_badge::badge_svg;

// ── Determinism: same input always produces exact same output ────────────

#[test]
fn given_same_inputs_when_badge_called_twice_then_output_is_byte_identical() {
    let a = badge_svg("lines", "1234");
    let b = badge_svg("lines", "1234");
    assert_eq!(a, b, "badge_svg must be deterministic");
}

#[test]
fn given_same_unicode_inputs_when_called_twice_then_output_identical() {
    let a = badge_svg("言語", "✅");
    let b = badge_svg("言語", "✅");
    assert_eq!(a, b);
}

#[test]
fn given_same_special_char_inputs_when_called_twice_then_output_identical() {
    let a = badge_svg("<a>&\"'", "x&y");
    let b = badge_svg("<a>&\"'", "x&y");
    assert_eq!(a, b);
}

// ── Structural: output always has exactly one SVG root ──────────────────

#[test]
fn given_any_input_when_badge_rendered_then_exactly_one_svg_open_tag() {
    for (label, value) in [("a", "b"), ("", ""), ("long label here", "42")] {
        let svg = badge_svg(label, value);
        assert_eq!(
            svg.matches("<svg ").count(),
            1,
            "must have exactly one <svg> open tag"
        );
        assert_eq!(
            svg.matches("</svg>").count(),
            1,
            "must have exactly one </svg> close tag"
        );
    }
}

// ── Edge: control characters in text ────────────────────────────────────

#[test]
fn given_tab_in_label_when_badge_rendered_then_svg_is_valid() {
    let svg = badge_svg("a\tb", "1");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(
        svg.contains("a\tb"),
        "tab should pass through (not XML-special)"
    );
}

#[test]
fn given_null_byte_in_label_when_badge_rendered_then_svg_is_valid() {
    let svg = badge_svg("a\0b", "x");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

// ── Edge: whitespace-only inputs ────────────────────────────────────────

#[test]
fn given_whitespace_only_label_when_badge_rendered_then_svg_is_valid() {
    let svg = badge_svg("   ", "ok");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("   "));
}

// ── Structural: second rect starts at label_width ───────────────────────

#[test]
fn given_badge_when_rendered_then_second_rect_x_equals_first_rect_width() {
    let svg = badge_svg("test", "value");
    // First <rect has width="W1", second has x="W1"
    let first_rect_start = svg.find("<rect").unwrap();
    let first_rect = &svg[first_rect_start..];
    let w_start = first_rect.find("width=\"").unwrap() + 7;
    let w_end = first_rect[w_start..].find('"').unwrap() + w_start;
    let first_width: &str = &first_rect[w_start..w_end];

    let second_rect_start =
        svg[first_rect_start + 5..].find("<rect").unwrap() + first_rect_start + 5;
    let second_rect = &svg[second_rect_start..];
    let x_start = second_rect.find("x=\"").unwrap() + 3;
    let x_end = second_rect[x_start..].find('"').unwrap() + x_start;
    let second_x: &str = &second_rect[x_start..x_end];

    assert_eq!(
        first_width, second_x,
        "second rect x must equal first rect width"
    );
}

// ── Return value: never empty ───────────────────────────────────────────

#[test]
fn given_any_input_when_badge_rendered_then_result_is_non_empty() {
    for (l, v) in [("", ""), ("x", "y"), ("日本語", "中文")] {
        let svg = badge_svg(l, v);
        assert!(!svg.is_empty());
    }
}

// ── Width scaling: asymmetric label/value lengths ───────────────────────

#[test]
fn given_long_label_short_value_when_badge_rendered_then_width_grows() {
    let short = badge_svg("a", "b");
    let asym = badge_svg("this is a very long label", "x");
    assert!(
        extract_width(&asym) > extract_width(&short),
        "asymmetric long label should produce wider badge"
    );
}

#[test]
fn given_short_label_long_value_when_badge_rendered_then_width_grows() {
    let short = badge_svg("a", "b");
    let asym = badge_svg("x", "this is a very long value");
    assert!(
        extract_width(&asym) > extract_width(&short),
        "asymmetric long value should produce wider badge"
    );
}

fn extract_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("width close") + start;
    svg[start..end].parse().expect("numeric width")
}
