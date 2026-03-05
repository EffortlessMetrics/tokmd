//! Expanded insta snapshot tests for SVG badge output (W58).
//!
//! Covers various badge configurations including metrics, edge cases,
//! special characters, and extreme values.

use tokmd_badge::badge_svg;

// ===========================================================================
// Standard metric badges
// ===========================================================================

#[test]
fn w58_badge_loc_metric() {
    insta::assert_snapshot!("w58_badge_loc_metric", badge_svg("LOC", "42,000"));
}

#[test]
fn w58_badge_complexity_metric() {
    insta::assert_snapshot!(
        "w58_badge_complexity_metric",
        badge_svg("complexity", "3.5")
    );
}

#[test]
fn w58_badge_coverage_metric() {
    insta::assert_snapshot!("w58_badge_coverage_metric", badge_svg("coverage", "87.3%"));
}

#[test]
fn w58_badge_doc_density() {
    insta::assert_snapshot!("w58_badge_doc_density", badge_svg("doc density", "12.5%"));
}

#[test]
fn w58_badge_tech_debt() {
    insta::assert_snapshot!("w58_badge_tech_debt", badge_svg("tech debt", "low"));
}

// ===========================================================================
// Zero values
// ===========================================================================

#[test]
fn w58_badge_zero_loc() {
    insta::assert_snapshot!("w58_badge_zero_loc", badge_svg("LOC", "0"));
}

#[test]
fn w58_badge_zero_complexity() {
    insta::assert_snapshot!("w58_badge_zero_complexity", badge_svg("complexity", "0.0"));
}

// ===========================================================================
// Very large numbers
// ===========================================================================

#[test]
fn w58_badge_million_loc() {
    insta::assert_snapshot!("w58_badge_million_loc", badge_svg("LOC", "1,234,567"));
}

#[test]
fn w58_badge_huge_tokens() {
    insta::assert_snapshot!(
        "w58_badge_huge_tokens",
        badge_svg("tokens", "99,999,999,999")
    );
}

// ===========================================================================
// Single character edge cases
// ===========================================================================

#[test]
fn w58_badge_single_char_label() {
    insta::assert_snapshot!("w58_badge_single_char_label", badge_svg("x", "1"));
}
