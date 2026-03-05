//! W67 deep tests for SVG badge rendering.
//!
//! ~25 tests covering: SVG structure, dimension heuristics, XML escaping,
//! color constants, text positioning, edge cases, Unicode, determinism,
//! and snapshot tests.

use tokmd_badge::badge_svg;

// ═══════════════════════════════════════════════════════════════════════════
// Helper
// ═══════════════════════════════════════════════════════════════════════════

fn extract_outer_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("close quote") + start;
    svg[start..end].parse().expect("numeric width")
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. SVG structural validity
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_svg_opens_with_svg_element() {
    assert!(badge_svg("loc", "99").starts_with("<svg "));
}

#[test]
fn w67_svg_closes_properly() {
    assert!(badge_svg("loc", "99").ends_with("</svg>"));
}

#[test]
fn w67_svg_contains_xmlns_declaration() {
    assert!(badge_svg("x", "y").contains("xmlns=\"http://www.w3.org/2000/svg\""));
}

#[test]
fn w67_svg_contains_role_img() {
    assert!(badge_svg("x", "y").contains("role=\"img\""));
}

#[test]
fn w67_svg_has_two_rects_and_two_texts() {
    let svg = badge_svg("metric", "value");
    assert_eq!(svg.matches("<rect").count(), 2, "expected 2 rects");
    assert_eq!(svg.matches("<text").count(), 2, "expected 2 text elements");
    assert_eq!(
        svg.matches("</text>").count(),
        2,
        "expected 2 closing text tags"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Color constants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_label_rect_is_grey() {
    let svg = badge_svg("metric", "val");
    // First rect should be the grey label background
    assert!(svg.contains("fill=\"#555\""));
}

#[test]
fn w67_value_rect_is_blue() {
    let svg = badge_svg("metric", "val");
    assert!(svg.contains("fill=\"#4c9aff\""));
}

#[test]
fn w67_both_texts_are_white() {
    let svg = badge_svg("a", "b");
    assert_eq!(svg.matches("fill=\"#fff\"").count(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Font and text positioning
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_font_family_verdana() {
    assert!(badge_svg("a", "b").contains("font-family=\"Verdana\""));
}

#[test]
fn w67_font_size_eleven() {
    assert!(badge_svg("a", "b").contains("font-size=\"11\""));
}

#[test]
fn w67_text_anchor_middle_for_both() {
    let svg = badge_svg("a", "b");
    assert_eq!(svg.matches("text-anchor=\"middle\"").count(), 2);
}

#[test]
fn w67_text_baseline_y_is_16() {
    let svg = badge_svg("a", "b");
    assert_eq!(svg.matches("y=\"16\"").count(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Width / dimension heuristics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_minimum_total_width_is_120() {
    // Each segment min 60 → total min 120
    assert!(extract_outer_width(&badge_svg("", "")) >= 120);
    assert!(extract_outer_width(&badge_svg("a", "b")) >= 120);
}

#[test]
fn w67_width_formula_explicit() {
    // label "0123456789" = 10 chars → 10*7+20 = 90 (>60)
    // value "ab"         =  2 chars →  2*7+20 = 34 → clamped to 60
    // total = 90 + 60 = 150
    assert_eq!(extract_outer_width(&badge_svg("0123456789", "ab")), 150);
}

#[test]
fn w67_width_grows_with_longer_label() {
    let short = extract_outer_width(&badge_svg("ab", "x"));
    let long = extract_outer_width(&badge_svg("abcdefghijklmnopqrst", "x"));
    assert!(long > short, "longer label should widen badge");
}

#[test]
fn w67_width_grows_with_longer_value() {
    let short = extract_outer_width(&badge_svg("x", "1"));
    let long = extract_outer_width(&badge_svg("x", "1234567890123456"));
    assert!(long > short, "longer value should widen badge");
}

#[test]
fn w67_height_always_24() {
    for (l, v) in [("", ""), ("abc", "def"), ("x".repeat(100).as_str(), "y")] {
        let svg = badge_svg(l, v);
        assert!(svg.contains("height=\"24\""), "height must be 24");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. XML escaping
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_escapes_ampersand_in_label_and_value() {
    let svg = badge_svg("R&D", "1&2");
    assert!(svg.contains("R&amp;D"));
    assert!(svg.contains("1&amp;2"));
}

#[test]
fn w67_escapes_angle_brackets() {
    let svg = badge_svg("<in>", "<out>");
    assert!(svg.contains("&lt;in&gt;"));
    assert!(svg.contains("&lt;out&gt;"));
}

#[test]
fn w67_escapes_quotes() {
    let svg = badge_svg("a\"b", "c'd");
    assert!(svg.contains("a&quot;b"));
    assert!(svg.contains("c&apos;d"));
}

#[test]
fn w67_all_five_xml_special_chars() {
    let svg = badge_svg("&<>\"'", "&<>\"'");
    let escaped = "&amp;&lt;&gt;&quot;&apos;";
    // Should appear twice (once per text element)
    assert_eq!(svg.matches(escaped).count(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_empty_label_valid_svg() {
    let svg = badge_svg("", "42");
    assert!(svg.starts_with("<svg ") && svg.ends_with("</svg>"));
}

#[test]
fn w67_empty_value_valid_svg() {
    let svg = badge_svg("metric", "");
    assert!(svg.starts_with("<svg ") && svg.ends_with("</svg>"));
}

#[test]
fn w67_both_empty_valid_svg() {
    let svg = badge_svg("", "");
    assert!(svg.starts_with("<svg ") && svg.ends_with("</svg>"));
    assert_eq!(svg.matches("<rect").count(), 2);
}

#[test]
fn w67_very_long_label_renders() {
    let label = "a".repeat(1_000);
    let svg = badge_svg(&label, "v");
    assert!(svg.starts_with("<svg "));
    assert!(svg.ends_with("</svg>"));
    // Width should be large: 1000*7+20 = 7020 for label + 60 for value
    assert_eq!(extract_outer_width(&svg), 7080);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Unicode handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_unicode_label_appears_in_output() {
    let svg = badge_svg("コード行", "42");
    assert!(svg.contains("コード行"));
}

#[test]
fn w67_emoji_value_appears_in_output() {
    let svg = badge_svg("status", "🟢 pass");
    assert!(svg.contains("🟢 pass"));
}

#[test]
fn w67_unicode_width_uses_char_count_not_byte_len() {
    // "日本語" = 3 chars → 3*7+20 = 41 → clamped to 60
    let svg = badge_svg("日本語", "x");
    assert_eq!(extract_outer_width(&svg), 120); // 60 + 60
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Determinism
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_deterministic_same_input_same_output() {
    let a = badge_svg("lines", "1234");
    let b = badge_svg("lines", "1234");
    assert_eq!(a, b);
}

#[test]
fn w67_different_label_different_output() {
    assert_ne!(badge_svg("foo", "42"), badge_svg("bar", "42"));
}

#[test]
fn w67_different_value_different_output() {
    assert_ne!(badge_svg("lines", "1"), badge_svg("lines", "2"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Snapshot tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_snapshot_lines_metric() {
    insta::assert_snapshot!("w67_lines_metric", badge_svg("lines", "12345"));
}

#[test]
fn w67_snapshot_empty_badge() {
    insta::assert_snapshot!("w67_empty_badge", badge_svg("", ""));
}

#[test]
fn w67_snapshot_xml_escape() {
    insta::assert_snapshot!("w67_xml_escape", badge_svg("a<b", "c&d"));
}

#[test]
fn w67_snapshot_unicode_badge() {
    insta::assert_snapshot!("w67_unicode_badge", badge_svg("言語数", "5"));
}
