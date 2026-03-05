//! Golden snapshot tests for badge SVG output (W54).
//!
//! Pins exact SVG output for a range of badge configurations so that
//! any rendering change is caught at review time.

use tokmd_badge::badge_svg;

// ===========================================================================
// Basic badges
// ===========================================================================

#[test]
fn w54_badge_lines_metric() {
    insta::assert_snapshot!("w54_badge_lines_metric", badge_svg("lines", "12345"));
}

#[test]
fn w54_badge_languages_metric() {
    insta::assert_snapshot!("w54_badge_languages_metric", badge_svg("languages", "7"));
}

#[test]
fn w54_badge_files_metric() {
    insta::assert_snapshot!("w54_badge_files_metric", badge_svg("files", "142"));
}

#[test]
fn w54_badge_code_metric() {
    insta::assert_snapshot!("w54_badge_code_metric", badge_svg("code", "8,901"));
}

#[test]
fn w54_badge_tokens_metric() {
    insta::assert_snapshot!("w54_badge_tokens_metric", badge_svg("tokens", "250k"));
}

// ===========================================================================
// Edge cases: zero and very large values
// ===========================================================================

#[test]
fn w54_badge_zero_lines() {
    insta::assert_snapshot!("w54_badge_zero_lines", badge_svg("lines", "0"));
}

#[test]
fn w54_badge_zero_files() {
    insta::assert_snapshot!("w54_badge_zero_files", badge_svg("files", "0"));
}

#[test]
fn w54_badge_very_large_value() {
    insta::assert_snapshot!(
        "w54_badge_very_large_value",
        badge_svg("total bytes", "9,999,999,999")
    );
}

#[test]
fn w54_badge_very_long_label() {
    insta::assert_snapshot!(
        "w54_badge_very_long_label",
        badge_svg("cyclomatic complexity average", "42")
    );
}

// ===========================================================================
// Special characters & escaping
// ===========================================================================

#[test]
fn w54_badge_xml_special_chars() {
    insta::assert_snapshot!(
        "w54_badge_xml_special_chars",
        badge_svg("a<b&c", "1>2\"3'4")
    );
}

#[test]
fn w54_badge_unicode_label() {
    insta::assert_snapshot!("w54_badge_unicode_label", badge_svg("コード", "5000"));
}

// ===========================================================================
// Empty / boundary
// ===========================================================================

#[test]
fn w54_badge_empty_label() {
    insta::assert_snapshot!("w54_badge_empty_label", badge_svg("", "100"));
}

#[test]
fn w54_badge_empty_value() {
    insta::assert_snapshot!("w54_badge_empty_value", badge_svg("score", ""));
}

#[test]
fn w54_badge_both_empty() {
    insta::assert_snapshot!("w54_badge_both_empty", badge_svg("", ""));
}
