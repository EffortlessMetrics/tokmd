//! Wave-38 deep tests for tokmd-badge SVG rendering.
//!
//! Focuses on width formula edge-cases, structural element counts,
//! attribute positioning, and special-character handling not yet
//! covered by existing test suites.

use tokmd_badge::badge_svg;

// ── Width formula: (chars * 7 + 20).max(60) ───────────────────────

#[test]
fn width_formula_below_min_threshold() {
    // 5 chars => 5*7+20 = 55, clamped to 60
    let svg = badge_svg("abcde", "12345");
    let w = width(&svg);
    assert_eq!(w, 120, "both segments at min → 120");
}

#[test]
fn width_formula_exactly_at_min_threshold() {
    // Need n where n*7+20 == 60 → n ≈ 5.7, so 5 chars gives 55 (<60), 6 gives 62 (>60)
    // 6 chars: 6*7+20 = 62 > 60
    let svg = badge_svg("abcdef", "1");
    let label_w = 62; // 6*7+20
    let value_w = 60; // min
    assert_eq!(width(&svg), label_w + value_w);
}

#[test]
fn width_formula_just_above_min_threshold() {
    // 7 chars => 7*7+20 = 69
    let svg = badge_svg("abcdefg", "1");
    assert_eq!(width(&svg), 69 + 60);
}

#[test]
fn width_formula_long_value_dominates() {
    // label 1 char (60 min), value 20 chars => 20*7+20 = 160
    let svg = badge_svg("x", &"v".repeat(20));
    assert_eq!(width(&svg), 60 + 160);
}

#[test]
fn width_formula_both_segments_above_min() {
    // label 10 chars => 10*7+20 = 90, value 15 chars => 15*7+20 = 125
    let svg = badge_svg(&"L".repeat(10), &"V".repeat(15));
    assert_eq!(width(&svg), 90 + 125);
}

// ── Element counts ────────────────────────────────────────────────

#[test]
fn exactly_one_svg_open_and_close() {
    let svg = badge_svg("a", "b");
    assert_eq!(svg.matches("<svg").count(), 1);
    assert_eq!(svg.matches("</svg>").count(), 1);
}

#[test]
fn exactly_two_rect_elements() {
    let svg = badge_svg("label", "value");
    assert_eq!(svg.matches("<rect").count(), 2);
}

#[test]
fn exactly_two_text_open_and_close_tags() {
    let svg = badge_svg("label", "value");
    assert_eq!(svg.matches("<text").count(), 2);
    assert_eq!(svg.matches("</text>").count(), 2);
}

// ── Attribute values ──────────────────────────────────────────────

#[test]
fn text_y_coordinate_is_16() {
    let svg = badge_svg("any", "thing");
    // Both text elements share y="16"
    assert_eq!(svg.matches("y=\"16\"").count(), 2);
}

#[test]
fn value_rect_x_equals_label_width() {
    // label "test" → 4 chars → max(48,60) = 60
    let svg = badge_svg("test", "val");
    assert!(svg.contains("x=\"60\""), "value rect x should be 60");
}

#[test]
fn label_rect_starts_at_zero() {
    // First <rect has no x attribute (implicitly 0)
    let svg = badge_svg("any", "val");
    let first_rect_end = svg.find("/>").unwrap();
    let first_rect = &svg[..first_rect_end];
    assert!(
        !first_rect.contains(" x=\""),
        "label rect should have no x attr (starts at 0)"
    );
}

#[test]
fn both_texts_share_fill_white() {
    let svg = badge_svg("foo", "bar");
    assert_eq!(
        svg.matches("fill=\"#fff\"").count(),
        2,
        "both text nodes should be white"
    );
}

// ── Special characters beyond XML entities ────────────────────────

#[test]
fn badge_handles_newline_in_label() {
    let svg = badge_svg("line\none", "2");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    // The newline should appear literally in SVG text (it's not an XML-special char)
    assert!(svg.contains("line\none"));
}

#[test]
fn badge_handles_tab_in_value() {
    let svg = badge_svg("lbl", "a\tb");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn badge_handles_numeric_only_inputs() {
    let svg = badge_svg("12345", "67890");
    assert!(svg.contains("12345"));
    assert!(svg.contains("67890"));
}

#[test]
fn badge_handles_whitespace_only_label() {
    let svg = badge_svg("   ", "v");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

// ── Centering for known inputs ────────────────────────────────────

#[test]
fn label_x_centered_when_above_min() {
    // label 10 chars → width = 10*7+20 = 90 → x = 90/2 = 45
    let svg = badge_svg(&"A".repeat(10), "v");
    assert!(svg.contains("x=\"45\""), "label text x should be 45");
}

#[test]
fn value_x_centered_when_both_above_min() {
    // label 10 chars → lw=90, value 10 chars → vw=90, value_x = 90+90/2 = 135
    let svg = badge_svg(&"L".repeat(10), &"V".repeat(10));
    assert!(svg.contains("x=\"135\""), "value text x should be 135");
}

// ── Determinism across varied inputs ──────────────────────────────

#[test]
fn deterministic_with_special_chars() {
    let a = badge_svg("a<b&c", "d\"e'f");
    let b = badge_svg("a<b&c", "d\"e'f");
    assert_eq!(a, b);
}

#[test]
fn different_inputs_produce_different_svgs() {
    let a = badge_svg("lines", "100");
    let b = badge_svg("files", "200");
    assert_ne!(a, b);
}

// ── Helper ────────────────────────────────────────────────────────

fn width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("close") + start;
    svg[start..end].parse().expect("int")
}
