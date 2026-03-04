//! Snapshot tests for tokmd-badge SVG output – wave 42.
//!
//! Uses `insta` for snapshot testing of badge SVG rendering.
//!
//! Run with: `cargo test -p tokmd-badge --test snapshot_w42`

use tokmd_badge::badge_svg;

// =========================================================================
// Badge with various code line counts
// =========================================================================

#[test]
fn snapshot_badge_zero_lines() {
    let svg = badge_svg("lines", "0");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_single_digit() {
    let svg = badge_svg("lines", "7");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_hundreds() {
    let svg = badge_svg("lines", "500");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_thousands() {
    let svg = badge_svg("lines", "42,000");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_millions() {
    let svg = badge_svg("lines", "1,234,567");
    insta::assert_snapshot!(svg);
}

// =========================================================================
// Badge with special characters in language names
// =========================================================================

#[test]
fn snapshot_badge_lang_csharp() {
    let svg = badge_svg("C#", "1000");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_lang_cpp() {
    let svg = badge_svg("C++", "5000");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_lang_with_ampersand() {
    let svg = badge_svg("HTML & CSS", "300");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_value_with_angle_brackets() {
    let svg = badge_svg("test", "<100>");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_unicode_label() {
    let svg = badge_svg("日本語", "999");
    insta::assert_snapshot!(svg);
}

// =========================================================================
// Badge with zero/max/edge values
// =========================================================================

#[test]
fn snapshot_badge_empty_strings() {
    let svg = badge_svg("", "");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_max_value() {
    let svg = badge_svg("total", "999,999,999");
    insta::assert_snapshot!(svg);
}

// =========================================================================
// Determinism checks
// =========================================================================

#[test]
fn badge_svg_deterministic() {
    let a = badge_svg("lines", "1234");
    let b = badge_svg("lines", "1234");
    let c = badge_svg("lines", "1234");
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn badge_svg_different_inputs_differ() {
    let a = badge_svg("lines", "100");
    let b = badge_svg("lines", "200");
    assert_ne!(a, b);
}
