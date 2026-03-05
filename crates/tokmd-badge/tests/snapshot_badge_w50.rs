//! Snapshot tests for tokmd-badge – wave 50.
//!
//! Covers: various label/value combos, color thresholds, empty strings,
//! unicode text, and numeric formatting.

use tokmd_badge::badge_svg;

// ── 1. Standard code lines badge ────────────────────────────────────

#[test]
fn snapshot_badge_code_lines() {
    let svg = badge_svg("code lines", "9,300");
    insta::assert_snapshot!(svg);
}

// ── 2. Languages count badge ────────────────────────────────────────

#[test]
fn snapshot_badge_languages() {
    let svg = badge_svg("languages", "6");
    insta::assert_snapshot!(svg);
}

// ── 3. Doc density percentage badge ─────────────────────────────────

#[test]
fn snapshot_badge_doc_density() {
    let svg = badge_svg("doc density", "15.0%");
    insta::assert_snapshot!(svg);
}

// ── 4. Empty label badge ────────────────────────────────────────────

#[test]
fn snapshot_badge_empty_label() {
    let svg = badge_svg("", "42");
    insta::assert_snapshot!(svg);
}

// ── 5. Empty value badge ────────────────────────────────────────────

#[test]
fn snapshot_badge_empty_value() {
    let svg = badge_svg("tokmd", "");
    insta::assert_snapshot!(svg);
}

// ── 6. Unicode label ────────────────────────────────────────────────

#[test]
fn snapshot_badge_unicode() {
    let svg = badge_svg("コード行", "1,234");
    insta::assert_snapshot!(svg);
}

// ── 7. XML special characters ───────────────────────────────────────

#[test]
fn snapshot_badge_xml_special() {
    let svg = badge_svg("a<b", "1&2");
    insta::assert_snapshot!(svg);
}

// ── 8. Very short values ────────────────────────────────────────────

#[test]
fn snapshot_badge_single_char() {
    let svg = badge_svg("v", "1");
    insta::assert_snapshot!(svg);
}
