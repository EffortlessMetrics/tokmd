//! Golden snapshot tests for badge SVG output (W70).
//!
//! Pins exact SVG output for a range of badge configurations so that
//! any rendering change is caught at review time.

use tokmd_badge::badge_svg;

// ===========================================================================
// Basic badges
// ===========================================================================

#[test]
fn w70_badge_lines_metric() {
    insta::assert_snapshot!("w70_badge_lines_metric", badge_svg("lines", "9876"));
}

#[test]
fn w70_badge_languages_metric() {
    insta::assert_snapshot!("w70_badge_languages_metric", badge_svg("languages", "12"));
}

#[test]
fn w70_badge_tokens_metric() {
    insta::assert_snapshot!("w70_badge_tokens_metric", badge_svg("tokens", "180k"));
}

#[test]
fn w70_badge_doc_density() {
    insta::assert_snapshot!("w70_badge_doc_density", badge_svg("doc density", "34.2%"));
}

// ===========================================================================
// Edge cases
// ===========================================================================

#[test]
fn w70_badge_empty_both() {
    insta::assert_snapshot!("w70_badge_empty_both", badge_svg("", ""));
}

#[test]
fn w70_badge_single_char() {
    insta::assert_snapshot!("w70_badge_single_char", badge_svg("x", "1"));
}

#[test]
fn w70_badge_long_text() {
    insta::assert_snapshot!(
        "w70_badge_long_text",
        badge_svg("cyclomatic complexity", "extremely high value here")
    );
}

#[test]
fn w70_badge_xml_special_chars() {
    insta::assert_snapshot!(
        "w70_badge_xml_special_chars",
        badge_svg("a<b&c", "1>2\"3'4")
    );
}

// ===========================================================================
// Numeric formatting
// ===========================================================================

#[test]
fn w70_badge_zero_value() {
    insta::assert_snapshot!("w70_badge_zero_value", badge_svg("files", "0"));
}

#[test]
fn w70_badge_large_number() {
    insta::assert_snapshot!(
        "w70_badge_large_number",
        badge_svg("total bytes", "1,234,567,890")
    );
}
