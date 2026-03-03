//! Extended golden snapshot tests for tokmd-badge SVG output.
//!
//! Covers domain-specific badge scenarios (LOC, languages, ratios)
//! and edge cases to ensure SVG rendering stability.

use tokmd_badge::badge_svg;

// ── Lines-of-code badges ────────────────────────────────────────────

#[test]
fn snapshot_loc_small_project() {
    let svg = badge_svg("lines of code", "342");
    insta::assert_snapshot!("loc_small_project", svg);
}

#[test]
fn snapshot_loc_medium_project() {
    let svg = badge_svg("lines of code", "12,450");
    insta::assert_snapshot!("loc_medium_project", svg);
}

#[test]
fn snapshot_loc_large_project() {
    let svg = badge_svg("lines of code", "1.2M");
    insta::assert_snapshot!("loc_large_project", svg);
}

// ── Language badges ─────────────────────────────────────────────────

#[test]
fn snapshot_lang_rust() {
    let svg = badge_svg("Rust", "92.3%");
    insta::assert_snapshot!("lang_rust", svg);
}

#[test]
fn snapshot_lang_python() {
    let svg = badge_svg("Python", "67.1%");
    insta::assert_snapshot!("lang_python", svg);
}

#[test]
fn snapshot_lang_javascript() {
    let svg = badge_svg("JavaScript", "45.8%");
    insta::assert_snapshot!("lang_javascript", svg);
}

// ── Comment ratio badges ────────────────────────────────────────────

#[test]
fn snapshot_comment_ratio_high() {
    let svg = badge_svg("comment ratio", "38%");
    insta::assert_snapshot!("comment_ratio_high", svg);
}

#[test]
fn snapshot_comment_ratio_low() {
    let svg = badge_svg("comment ratio", "2%");
    insta::assert_snapshot!("comment_ratio_low", svg);
}

// ── Custom color badge (via value text) ─────────────────────────────

#[test]
fn snapshot_complexity_badge() {
    let svg = badge_svg("complexity", "moderate");
    insta::assert_snapshot!("complexity_badge", svg);
}

#[test]
fn snapshot_coverage_badge() {
    let svg = badge_svg("coverage", "87.5%");
    insta::assert_snapshot!("coverage_badge", svg);
}

// ── Multi-word and special content ──────────────────────────────────

#[test]
fn snapshot_multiword_label_value() {
    let svg = badge_svg("tech debt ratio", "12.3%");
    insta::assert_snapshot!("multiword_label_value", svg);
}

#[test]
fn snapshot_special_chars_ampersand() {
    let svg = badge_svg("C & C++", "1,200");
    insta::assert_snapshot!("special_chars_ampersand", svg);
}
