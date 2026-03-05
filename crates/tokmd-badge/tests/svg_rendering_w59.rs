//! W59 – SVG badge rendering depth tests.
//!
//! Covers structure validation, dimension calculation, XML escaping,
//! determinism, and edge-case inputs for `badge_svg`.

use tokmd_badge::badge_svg;

// ── Helpers ────────────────────────────────────────────────────────────

/// Extract the root-level SVG `width` attribute as an integer.
fn extract_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("width close") + start;
    svg[start..end].parse().expect("numeric width")
}

/// Count occurrences of a substring.
fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

// ── SVG structure ──────────────────────────────────────────────────────

#[test]
fn svg_opens_and_closes() {
    let svg = badge_svg("lang", "Rust");
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn svg_has_xmlns() {
    let svg = badge_svg("k", "v");
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
}

#[test]
fn svg_has_role_img() {
    let svg = badge_svg("k", "v");
    assert!(svg.contains("role=\"img\""));
}

#[test]
fn svg_contains_exactly_two_rects() {
    let svg = badge_svg("lines", "42");
    assert_eq!(count_occurrences(&svg, "<rect"), 2);
}

#[test]
fn svg_contains_exactly_two_text_elements() {
    let svg = badge_svg("lines", "42");
    assert_eq!(count_occurrences(&svg, "<text"), 2);
}

#[test]
fn svg_height_is_always_24() {
    for (l, v) in [("a", "1"), ("long label", "long value"), ("", "")] {
        let svg = badge_svg(l, v);
        assert!(
            svg.contains("height=\"24\""),
            "height should be 24 for ({l:?}, {v:?})"
        );
    }
}

#[test]
fn svg_is_single_line() {
    let svg = badge_svg("lines", "42");
    assert_eq!(svg.lines().count(), 1, "SVG should be a single line");
}

// ── Colours ────────────────────────────────────────────────────────────

#[test]
fn label_rect_is_grey() {
    let svg = badge_svg("k", "v");
    assert!(svg.contains("fill=\"#555\""));
}

#[test]
fn value_rect_is_blue() {
    let svg = badge_svg("k", "v");
    assert!(svg.contains("fill=\"#4c9aff\""));
}

#[test]
fn text_is_white() {
    let svg = badge_svg("k", "v");
    // Both text elements use white fill
    assert_eq!(count_occurrences(&svg, "fill=\"#fff\""), 2);
}

// ── Font ───────────────────────────────────────────────────────────────

#[test]
fn text_uses_verdana() {
    let svg = badge_svg("k", "v");
    assert_eq!(count_occurrences(&svg, "font-family=\"Verdana\""), 2);
}

#[test]
fn text_size_is_11() {
    let svg = badge_svg("k", "v");
    assert_eq!(count_occurrences(&svg, "font-size=\"11\""), 2);
}

#[test]
fn text_anchor_is_middle() {
    let svg = badge_svg("k", "v");
    assert_eq!(count_occurrences(&svg, "text-anchor=\"middle\""), 2);
}

// ── Width calculations ─────────────────────────────────────────────────

#[test]
fn minimum_width_is_120() {
    let svg = badge_svg("a", "1");
    assert!(extract_width(&svg) >= 120, "minimum width must be ≥ 120");
}

#[test]
fn empty_strings_still_reach_minimum_width() {
    let svg = badge_svg("", "");
    assert!(extract_width(&svg) >= 120);
}

#[test]
fn width_grows_monotonically() {
    let w1 = extract_width(&badge_svg("ab", "12"));
    let w2 = extract_width(&badge_svg("abcdef", "123456"));
    let w3 = extract_width(&badge_svg("abcdefghijklmno", "1234567890abcde"));
    assert!(w2 >= w1);
    assert!(w3 >= w2);
}

#[test]
fn width_formula_spot_check() {
    // label "ab" → 2*7+20 = 34 → max(34,60) = 60
    // value "1"  → 1*7+20 = 27 → max(27,60) = 60
    // total = 120
    assert_eq!(extract_width(&badge_svg("ab", "1")), 120);
}

#[test]
fn width_formula_longer_text() {
    // label "longlabel" → 9*7+20 = 83
    // value "longvalue" → 9*7+20 = 83
    // total = 166
    assert_eq!(extract_width(&badge_svg("longlabel", "longvalue")), 166);
}

// ── Text centering ─────────────────────────────────────────────────────

#[test]
fn label_text_is_centered_in_label_rect() {
    // label_width = max(2*7+20, 60) = 60, label_x = 60/2 = 30
    let svg = badge_svg("ab", "1");
    assert!(svg.contains("x=\"30\""));
}

#[test]
fn value_text_is_centered_in_value_rect() {
    // label_width = 60, value_width = 60, value_x = 60 + 60/2 = 90
    let svg = badge_svg("ab", "1");
    assert!(svg.contains("x=\"90\""));
}

// ── XML escaping ───────────────────────────────────────────────────────

#[test]
fn ampersand_is_escaped() {
    let svg = badge_svg("A&B", "ok");
    assert!(svg.contains("A&amp;B"));
    // The raw `&B` must not appear (it would be invalid XML)
    assert!(!svg.contains("A&B"));
}

#[test]
fn less_than_is_escaped() {
    let svg = badge_svg("<b>", "ok");
    assert!(svg.contains("&lt;b&gt;"));
}

#[test]
fn double_quote_is_escaped() {
    let svg = badge_svg("a\"b", "ok");
    assert!(svg.contains("a&quot;b"));
}

#[test]
fn single_quote_is_escaped() {
    let svg = badge_svg("a'b", "ok");
    assert!(svg.contains("a&apos;b"));
}

#[test]
fn all_special_chars_escaped_together() {
    let svg = badge_svg("<&>\"'", "x");
    assert!(svg.contains("&lt;&amp;&gt;&quot;&apos;"));
}

#[test]
fn plain_text_is_not_escaped() {
    let svg = badge_svg("hello", "world");
    assert!(svg.contains(">hello</text>"));
    assert!(svg.contains(">world</text>"));
}

// ── Determinism ────────────────────────────────────────────────────────

#[test]
fn same_input_produces_identical_output() {
    let a = badge_svg("lines", "1234");
    let b = badge_svg("lines", "1234");
    assert_eq!(a, b);
}

#[test]
fn deterministic_across_1000_calls() {
    let reference = badge_svg("coverage", "87%");
    for _ in 0..1000 {
        assert_eq!(badge_svg("coverage", "87%"), reference);
    }
}

// ── Unicode ────────────────────────────────────────────────────────────

#[test]
fn unicode_label_renders() {
    let svg = badge_svg("日本語", "ok");
    assert!(svg.contains("日本語"));
}

#[test]
fn unicode_value_renders() {
    let svg = badge_svg("lang", "中文");
    assert!(svg.contains("中文"));
}

#[test]
fn emoji_in_badge() {
    let svg = badge_svg("status", "✅");
    assert!(svg.contains("✅"));
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn unicode_width_uses_char_count() {
    // "ab" (2 chars) and "日本" (2 chars) should produce same width
    let ascii = extract_width(&badge_svg("ab", "v"));
    let cjk = extract_width(&badge_svg("日本", "v"));
    assert_eq!(
        ascii, cjk,
        "width should be based on char count, not byte length"
    );
}

// ── Edge cases ─────────────────────────────────────────────────────────

#[test]
fn very_long_label() {
    let label = "a".repeat(500);
    let svg = badge_svg(&label, "v");
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>"));
    assert!(extract_width(&svg) > 120);
}

#[test]
fn very_long_value() {
    let value = "9".repeat(500);
    let svg = badge_svg("k", &value);
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>"));
    assert!(extract_width(&svg) > 120);
}

#[test]
fn whitespace_only_strings() {
    let svg = badge_svg("   ", "   ");
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn newlines_in_input() {
    let svg = badge_svg("line\none", "two\nthree");
    // Should still produce valid SVG (newlines in text nodes are fine in SVG)
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn tab_in_input() {
    let svg = badge_svg("a\tb", "c\td");
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>"));
}
