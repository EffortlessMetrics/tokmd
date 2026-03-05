//! Contract tests for tokmd-badge SVG rendering (w64).
//!
//! Coverage:
//! - SVG badge generation for various values
//! - Color thresholds
//! - Label customization
//! - Deterministic SVG output
//! - Property: badge always contains valid SVG structure
//! - Edge: zero value, max value, empty label
//! - UTF-8 safety in SVG output

use tokmd_badge::badge_svg;

// ============================================================================
// Helpers
// ============================================================================

fn extract_width(svg: &str) -> i32 {
    let start = svg.find("width=\"").expect("width attr") + 7;
    let end = svg[start..].find('"').expect("width close") + start;
    svg[start..end].parse().expect("numeric width")
}

fn extract_height(svg: &str) -> i32 {
    let start = svg.find("height=\"").expect("height attr") + 8;
    let end = svg[start..].find('"').expect("height close") + start;
    svg[start..end].parse().expect("numeric height")
}

// ============================================================================
// 1. Basic SVG structure
// ============================================================================

#[test]
fn badge_starts_with_svg_tag() {
    let svg = badge_svg("lines", "1234");
    assert!(svg.starts_with("<svg"), "Badge must start with <svg");
}

#[test]
fn badge_ends_with_closing_svg() {
    let svg = badge_svg("lines", "1234");
    assert!(svg.ends_with("</svg>"), "Badge must end with </svg>");
}

#[test]
fn badge_contains_xmlns() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
}

#[test]
fn badge_contains_role_img() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("role=\"img\""));
}

#[test]
fn badge_has_two_rects() {
    let svg = badge_svg("test", "42");
    assert_eq!(svg.matches("<rect").count(), 2, "Expected 2 rect elements");
}

#[test]
fn badge_has_two_text_elements() {
    let svg = badge_svg("test", "42");
    assert_eq!(svg.matches("<text").count(), 2, "Expected 2 text elements");
}

#[test]
fn badge_has_font_family_verdana() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("font-family=\"Verdana\""));
}

#[test]
fn badge_has_font_size_11() {
    let svg = badge_svg("test", "42");
    assert!(svg.contains("font-size=\"11\""));
}

#[test]
fn badge_text_anchor_is_middle() {
    let svg = badge_svg("test", "42");
    assert_eq!(
        svg.matches("text-anchor=\"middle\"").count(),
        2,
        "Both text elements need text-anchor=middle"
    );
}

#[test]
fn badge_height_is_24() {
    let svg = badge_svg("test", "42");
    assert_eq!(extract_height(&svg), 24);
}

// ============================================================================
// 2. Label and value presence
// ============================================================================

#[test]
fn badge_contains_label_text() {
    let svg = badge_svg("coverage", "87%");
    assert!(svg.contains("coverage"));
}

#[test]
fn badge_contains_value_text() {
    let svg = badge_svg("coverage", "87%");
    assert!(svg.contains("87%"));
}

#[test]
fn badge_various_labels() {
    for label in ["lines", "coverage", "grade", "LOC", "tokens"] {
        let svg = badge_svg(label, "42");
        assert!(svg.contains(label), "Badge should contain label: {label}");
    }
}

#[test]
fn badge_various_values() {
    for value in ["0", "42", "1000", "99.9%", "A+", "N/A"] {
        let svg = badge_svg("metric", value);
        assert!(
            svg.contains(value) || svg.contains(&value.replace('%', "")),
            "Badge should contain value: {value}"
        );
    }
}

// ============================================================================
// 3. Color segments
// ============================================================================

#[test]
fn badge_label_background_is_grey() {
    let svg = badge_svg("test", "42");
    assert!(
        svg.contains("fill=\"#555\""),
        "Label segment should be grey #555"
    );
}

#[test]
fn badge_value_background_is_blue() {
    let svg = badge_svg("test", "42");
    assert!(
        svg.contains("fill=\"#4c9aff\""),
        "Value segment should be blue #4c9aff"
    );
}

#[test]
fn badge_text_fill_is_white() {
    let svg = badge_svg("test", "42");
    assert_eq!(
        svg.matches("fill=\"#fff\"").count(),
        2,
        "Both texts should be white"
    );
}

// ============================================================================
// 4. Width calculations
// ============================================================================

#[test]
fn badge_width_minimum_120() {
    // Shortest possible label+value each get min 60
    let svg = badge_svg("a", "1");
    assert_eq!(extract_width(&svg), 120, "Minimum badge width is 120");
}

#[test]
fn badge_width_scales_with_text() {
    let short = badge_svg("a", "1");
    let long = badge_svg("averylonglabel", "averylongvalue");
    assert!(
        extract_width(&long) > extract_width(&short),
        "Longer text should produce wider badge"
    );
}

#[test]
fn badge_width_label_half_value_half() {
    // With equal very short text, both halves get min 60
    let svg = badge_svg("ab", "12");
    let width = extract_width(&svg);
    assert_eq!(width, 120);
}

#[test]
fn badge_width_long_label_only() {
    let short_value = badge_svg("thisisalongerlabel", "1");
    let both_short = badge_svg("a", "1");
    assert!(extract_width(&short_value) > extract_width(&both_short));
}

#[test]
fn badge_width_long_value_only() {
    let long_value = badge_svg("a", "thisisalongervalue");
    let both_short = badge_svg("a", "1");
    assert!(extract_width(&long_value) > extract_width(&both_short));
}

// ============================================================================
// 5. Determinism
// ============================================================================

#[test]
fn badge_deterministic_same_input() {
    let a = badge_svg("lines", "1234");
    let b = badge_svg("lines", "1234");
    assert_eq!(a, b, "Same input must produce identical SVG");
}

#[test]
fn badge_deterministic_repeated_calls() {
    let results: Vec<String> = (0..10).map(|_| badge_svg("test", "42")).collect();
    for r in &results {
        assert_eq!(r, &results[0]);
    }
}

// ============================================================================
// 6. XML escaping
// ============================================================================

#[test]
fn badge_escapes_ampersand() {
    let svg = badge_svg("a&b", "1&2");
    assert!(svg.contains("&amp;"));
    assert!(!svg.contains("a&b"));
}

#[test]
fn badge_escapes_less_than() {
    let svg = badge_svg("a<b", "1");
    assert!(svg.contains("&lt;"));
    assert!(!svg.contains("a<b"));
}

#[test]
fn badge_escapes_greater_than() {
    let svg = badge_svg("a>b", "1");
    assert!(svg.contains("&gt;"));
    assert!(!svg.contains("a>b"));
}

#[test]
fn badge_escapes_double_quote() {
    let svg = badge_svg("a\"b", "1");
    assert!(svg.contains("&quot;"));
}

#[test]
fn badge_escapes_single_quote() {
    let svg = badge_svg("a'b", "1");
    assert!(svg.contains("&apos;"));
}

#[test]
fn badge_escapes_all_special_chars() {
    let svg = badge_svg("<>&\"'", "<>&\"'");
    // Ensure no unescaped special chars appear inside text nodes
    // The svg tag attributes contain quotes, so we check text content specifically
    assert!(svg.contains("&lt;&gt;&amp;&quot;&apos;"));
}

// ============================================================================
// 7. Edge: empty strings
// ============================================================================

#[test]
fn badge_empty_label() {
    let svg = badge_svg("", "42");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("42"));
}

#[test]
fn badge_empty_value() {
    let svg = badge_svg("test", "");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("test"));
}

#[test]
fn badge_both_empty() {
    let svg = badge_svg("", "");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    // Minimum width still applies
    assert_eq!(extract_width(&svg), 120);
}

// ============================================================================
// 8. Edge: zero and max values
// ============================================================================

#[test]
fn badge_zero_value() {
    let svg = badge_svg("lines", "0");
    assert!(svg.contains(">0<"));
}

#[test]
fn badge_large_numeric_value() {
    let svg = badge_svg("lines", "999999999");
    assert!(svg.contains("999999999"));
}

#[test]
fn badge_negative_value_text() {
    let svg = badge_svg("delta", "-42");
    assert!(svg.contains("-42"));
}

// ============================================================================
// 9. UTF-8 safety
// ============================================================================

#[test]
fn badge_utf8_label_cjk() {
    let svg = badge_svg("日本語", "42");
    assert!(svg.contains("日本語"));
    assert!(svg.starts_with("<svg"));
}

#[test]
fn badge_utf8_value_emoji() {
    let svg = badge_svg("status", "✅");
    assert!(svg.contains("✅"));
}

#[test]
fn badge_utf8_mixed() {
    let svg = badge_svg("テスト", "成功");
    assert!(svg.contains("テスト"));
    assert!(svg.contains("成功"));
}

#[test]
fn badge_utf8_width_uses_char_count() {
    // CJK chars: 3 chars but more bytes
    let cjk = badge_svg("日本語", "42");
    let ascii = badge_svg("abc", "42");
    // Same char count should yield same width
    let cjk_width = extract_width(&cjk);
    let ascii_width = extract_width(&ascii);
    assert_eq!(
        cjk_width, ascii_width,
        "Width should be based on char count, not byte length"
    );
}

#[test]
fn badge_utf8_cyrillic() {
    let svg = badge_svg("строки", "42");
    assert!(svg.contains("строки"));
}

#[test]
fn badge_utf8_arabic() {
    let svg = badge_svg("سطور", "42");
    assert!(svg.contains("سطور"));
}

// ============================================================================
// 10. Property: valid SVG structure
// ============================================================================

#[test]
fn property_badge_always_valid_svg_structure() {
    let cases = [
        ("test", "42"),
        ("", ""),
        ("very long label text here", "99999"),
        ("a&<>\"'b", "c&<>\"'d"),
        ("日本語", "テスト"),
        ("lines", "0"),
    ];
    for (label, value) in cases {
        let svg = badge_svg(label, value);
        assert!(
            svg.starts_with("<svg"),
            "Must start with <svg: label={label}"
        );
        assert!(
            svg.ends_with("</svg>"),
            "Must end with </svg>: label={label}"
        );
        assert!(
            svg.contains("xmlns=\"http://www.w3.org/2000/svg\""),
            "Must have xmlns: label={label}"
        );
        assert_eq!(
            svg.matches("<rect").count(),
            2,
            "Must have 2 rects: label={label}"
        );
        assert_eq!(
            svg.matches("<text").count(),
            2,
            "Must have 2 texts: label={label}"
        );
        assert_eq!(
            svg.matches("</text>").count(),
            2,
            "Must close 2 texts: label={label}"
        );
    }
}

#[test]
fn property_badge_width_always_positive() {
    let cases = [("a", "1"), ("", ""), ("test", "42"), ("日本語", "テスト")];
    for (label, value) in cases {
        let svg = badge_svg(label, value);
        let width = extract_width(&svg);
        assert!(
            width > 0,
            "Width must be positive: label={label} value={value}"
        );
    }
}

#[test]
fn property_badge_no_raw_xml_special_in_text() {
    let dangerous = ["<script>", "]]>", "&raw;", "\"onclick=\""];
    for input in dangerous {
        let svg = badge_svg(input, input);
        // Between <text...> and </text>, no raw special chars should appear
        assert!(
            !svg.contains(&format!(">{input}<")),
            "Raw dangerous input must be escaped: {input}"
        );
    }
}

// ============================================================================
// 11. Snapshot tests (insta)
// ============================================================================

#[test]
fn snapshot_badge_basic() {
    let svg = badge_svg("lines", "1234");
    insta::assert_snapshot!("w64_badge_basic", svg);
}

#[test]
fn snapshot_badge_empty() {
    let svg = badge_svg("", "");
    insta::assert_snapshot!("w64_badge_empty", svg);
}

#[test]
fn snapshot_badge_xml_escape() {
    let svg = badge_svg("a&<b", "c>\"d");
    insta::assert_snapshot!("w64_badge_xml_escape", svg);
}

#[test]
fn snapshot_badge_unicode() {
    let svg = badge_svg("日本語", "テスト");
    insta::assert_snapshot!("w64_badge_unicode", svg);
}
