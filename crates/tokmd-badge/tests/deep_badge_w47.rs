//! Deep tests for SVG badge rendering (Wave 47).
//!
//! Covers: width proportionality, formula edge-cases, monotonicity,
//! URL-like characters, mixed Unicode/ASCII, large inputs,
//! determinism stress, and structural SVG invariants.

use tokmd_badge::badge_svg;

// ===========================================================================
// 1. Width proportionality to text length
// ===========================================================================

#[test]
fn width_proportional_to_label_char_count() {
    let w5 = width(&badge_svg(&"x".repeat(5), "v"));
    let w10 = width(&badge_svg(&"x".repeat(10), "v"));
    let w20 = width(&badge_svg(&"x".repeat(20), "v"));
    // Longer label → wider badge (monotonic)
    assert!(w10 > w5);
    assert!(w20 > w10);
}

#[test]
fn width_proportional_to_value_char_count() {
    let w5 = width(&badge_svg("l", &"y".repeat(5)));
    let w10 = width(&badge_svg("l", &"y".repeat(10)));
    let w20 = width(&badge_svg("l", &"y".repeat(20)));
    assert!(w10 > w5);
    assert!(w20 > w10);
}

#[test]
fn width_monotonically_increases_with_label_length() {
    let mut prev = 0;
    for n in [1, 5, 10, 20, 50] {
        let w = width(&badge_svg(&"a".repeat(n), "v"));
        assert!(
            w >= prev,
            "width should not decrease: n={n}, w={w}, prev={prev}"
        );
        prev = w;
    }
}

// ===========================================================================
// 2. Formula exact verification
// ===========================================================================

#[test]
fn formula_both_below_min() {
    // 3 chars each => 3*7+20 = 41, clamped to 60 each
    assert_eq!(width(&badge_svg("abc", "xyz")), 120);
}

#[test]
fn formula_label_above_value_below() {
    // label 8 chars => 8*7+20 = 76, value 2 chars => 60 min
    assert_eq!(width(&badge_svg(&"L".repeat(8), "ab")), 76 + 60);
}

#[test]
fn formula_both_above_min() {
    // label 12 chars => 12*7+20 = 104, value 9 chars => 9*7+20 = 83
    assert_eq!(width(&badge_svg(&"A".repeat(12), &"B".repeat(9))), 104 + 83);
}

#[test]
fn formula_empty_strings_use_min() {
    // 0 chars => 0*7+20 = 20, clamped to 60
    assert_eq!(width(&badge_svg("", "")), 120);
}

#[test]
fn formula_boundary_at_6_chars() {
    // 6 chars => 6*7+20 = 62 (first value > 60)
    let svg = badge_svg("abcdef", "abcdef");
    assert_eq!(width(&svg), 62 + 62);
}

// ===========================================================================
// 3. Color scheme invariants
// ===========================================================================

#[test]
fn color_scheme_unchanged_across_inputs() {
    let inputs = [
        ("a", "1"),
        ("long label here", "big value"),
        ("", ""),
        ("日本", "中文"),
    ];
    for (label, value) in inputs {
        let svg = badge_svg(label, value);
        assert!(
            svg.contains("fill=\"#555\""),
            "label bg missing for ({label}, {value})"
        );
        assert!(
            svg.contains("fill=\"#4c9aff\""),
            "value bg missing for ({label}, {value})"
        );
        assert_eq!(
            svg.matches("fill=\"#fff\"").count(),
            2,
            "text fill count wrong for ({label}, {value})"
        );
    }
}

// ===========================================================================
// 4. XML escaping for URL-like and special characters
// ===========================================================================

#[test]
fn escapes_url_like_characters() {
    let svg = badge_svg("http://example.com", "val");
    // No raw < but the :// is fine; only XML-special chars get escaped
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn escapes_query_string_ampersand() {
    let svg = badge_svg("a=1&b=2", "val");
    assert!(svg.contains("a=1&amp;b=2"));
    assert!(!svg.contains("a=1&b=2"));
}

#[test]
fn escapes_html_tags() {
    let svg = badge_svg("<b>bold</b>", "val");
    assert!(svg.contains("&lt;b&gt;bold&lt;/b&gt;"));
}

#[test]
fn escapes_mixed_special_chars() {
    let svg = badge_svg("a&b<c>d\"e'f", "ok");
    assert!(svg.contains("a&amp;b&lt;c&gt;d&quot;e&apos;f"));
}

// ===========================================================================
// 5. Unicode handling
// ===========================================================================

#[test]
fn unicode_cjk_width_uses_char_count() {
    // 4 CJK chars => 4*7+20 = 48, clamped to 60
    let svg = badge_svg("漢字仮名", "v");
    assert_eq!(width(&svg), 120);
}

#[test]
fn unicode_emoji_produces_valid_svg() {
    let svg = badge_svg("🚀🔥💯", "✅");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("🚀🔥💯"));
}

#[test]
fn mixed_ascii_unicode_formula() {
    // "ab漢字" = 4 chars => 4*7+20 = 48, clamped to 60
    let svg = badge_svg("ab漢字", "v");
    assert_eq!(width(&svg), 120);
}

// ===========================================================================
// 6. Large inputs
// ===========================================================================

#[test]
fn large_label_500_chars() {
    let label = "X".repeat(500);
    let svg = badge_svg(&label, "v");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    let expected_label_w = 500 * 7 + 20; // 3520
    assert_eq!(width(&svg), expected_label_w + 60);
}

#[test]
fn large_value_500_chars() {
    let value = "Y".repeat(500);
    let svg = badge_svg("l", &value);
    let expected_value_w = 500 * 7 + 20; // 3520
    assert_eq!(width(&svg), 60 + expected_value_w);
}

// ===========================================================================
// 7. Determinism stress
// ===========================================================================

#[test]
fn determinism_100_iterations() {
    let reference = badge_svg("lines", "42");
    for _ in 0..100 {
        assert_eq!(badge_svg("lines", "42"), reference);
    }
}

#[test]
fn determinism_with_unicode_100_iterations() {
    let reference = badge_svg("テスト", "結果");
    for _ in 0..100 {
        assert_eq!(badge_svg("テスト", "結果"), reference);
    }
}

// ===========================================================================
// 8. Structural SVG invariants
// ===========================================================================

#[test]
fn value_rect_x_equals_label_width() {
    for n in [1, 5, 10, 20] {
        let label = "L".repeat(n);
        let svg = badge_svg(&label, "v");
        let expected_label_w = (n as i32 * 7 + 20).max(60);
        let expected_x = format!("x=\"{expected_label_w}\"");
        assert!(
            svg.contains(&expected_x),
            "value rect x should be {expected_label_w} for {n}-char label"
        );
    }
}

#[test]
fn label_text_x_is_half_label_width() {
    for n in [1, 8, 15] {
        let label = "A".repeat(n);
        let svg = badge_svg(&label, "v");
        let label_w = (n as i32 * 7 + 20).max(60);
        let expected_x = label_w / 2;
        let positions = text_x_positions(&svg);
        assert_eq!(
            positions[0], expected_x,
            "label text x wrong for {n}-char label"
        );
    }
}

#[test]
fn value_text_x_is_label_w_plus_half_value_w() {
    let svg = badge_svg(&"A".repeat(10), &"B".repeat(8));
    let label_w = 10 * 7 + 20; // 90
    let value_w = 8 * 7 + 20; // 76
    let expected = label_w + value_w / 2; // 90 + 38 = 128
    let positions = text_x_positions(&svg);
    assert_eq!(positions[1], expected);
}

#[test]
fn height_always_24() {
    for (l, v) in [("", ""), ("long label", "long value"), ("🎉", "✅")] {
        let svg = badge_svg(l, v);
        assert!(
            svg.contains("height=\"24\""),
            "height not 24 for ({l}, {v})"
        );
    }
}

#[test]
fn all_text_elements_centered() {
    let svg = badge_svg("label", "value");
    assert_eq!(svg.matches("text-anchor=\"middle\"").count(), 2);
}

// ===========================================================================
// 9. Content presence
// ===========================================================================

#[test]
fn percentage_value_displayed() {
    let svg = badge_svg("coverage", "95.3%");
    assert!(svg.contains("95.3%"));
}

#[test]
fn negative_value_displayed() {
    let svg = badge_svg("delta", "-17");
    assert!(svg.contains("-17"));
}

#[test]
fn whitespace_only_inputs() {
    let svg = badge_svg("   ", "\t\t");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

// ===========================================================================
// Helpers
// ===========================================================================

fn width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("close") + start;
    svg[start..end].parse().expect("int")
}

fn text_x_positions(svg: &str) -> Vec<i32> {
    let mut positions = Vec::new();
    let mut from = 0;
    while let Some(idx) = svg[from..].find("<text") {
        let abs = from + idx;
        let text = &svg[abs..];
        let x_start = text.find("x=\"").unwrap() + 3;
        let x_end = text[x_start..].find('"').unwrap() + x_start;
        positions.push(text[x_start..x_end].parse().unwrap());
        from = abs + 5;
    }
    positions
}
