//! Snapshot tests for tokmd-badge – wave 45.
//!
//! Covers gaps: whitespace handling, real-world badge values,
//! status indicators, percentage values, and very long text.

use tokmd_badge::badge_svg;

// =========================================================================
// Whitespace handling
// =========================================================================

#[test]
fn snapshot_badge_leading_spaces() {
    let svg = badge_svg("  lines", "1000");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_trailing_spaces() {
    let svg = badge_svg("lines  ", "1000  ");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_internal_whitespace() {
    let svg = badge_svg("total  lines", "1 000");
    insta::assert_snapshot!(svg);
}

// =========================================================================
// Real-world badge values: percentage and status text
// =========================================================================

#[test]
fn snapshot_badge_zero_percent() {
    let svg = badge_svg("coverage", "0%");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_fifty_percent() {
    let svg = badge_svg("coverage", "50%");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_hundred_percent() {
    let svg = badge_svg("coverage", "100%");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_status_pass() {
    let svg = badge_svg("build", "pass");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_status_fail() {
    let svg = badge_svg("build", "fail");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_status_na() {
    let svg = badge_svg("coverage", "N/A");
    insta::assert_snapshot!(svg);
}

// =========================================================================
// Very long text
// =========================================================================

#[test]
fn snapshot_badge_very_long_label() {
    let label = "a]".repeat(80);
    let svg = badge_svg(&label, "42");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_very_long_value() {
    let value = "9".repeat(50);
    let svg = badge_svg("lines", &value);
    insta::assert_snapshot!(svg);
}

// =========================================================================
// Numeric formatting edge cases
// =========================================================================

#[test]
fn snapshot_badge_negative_value() {
    let svg = badge_svg("delta", "-42");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_decimal_value() {
    let svg = badge_svg("ratio", "0.75");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_formatted_number() {
    let svg = badge_svg("lines", "1,234,567");
    insta::assert_snapshot!(svg);
}
