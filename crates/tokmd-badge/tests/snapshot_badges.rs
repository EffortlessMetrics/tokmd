//! Insta snapshot tests for diverse badge configurations.
//!
//! Each test captures the full SVG output so that any rendering
//! regression (widths, escaping, positioning) is caught automatically.

use tokmd_badge::badge_svg;

// ── Metric-flavoured badges ─────────────────────────────────────────

#[test]
fn snapshot_code_density_badge() {
    let svg = badge_svg("code density", "87.3%");
    insta::assert_snapshot!("code_density_badge", svg);
}

#[test]
fn snapshot_comment_ratio_badge() {
    let svg = badge_svg("comment ratio", "12.5%");
    insta::assert_snapshot!("comment_ratio_badge", svg);
}

#[test]
fn snapshot_blank_ratio_badge() {
    let svg = badge_svg("blank ratio", "0.2%");
    insta::assert_snapshot!("blank_ratio_badge", svg);
}

#[test]
fn snapshot_total_files_badge() {
    let svg = badge_svg("files", "314");
    insta::assert_snapshot!("total_files_badge", svg);
}

#[test]
fn snapshot_cocomo_person_months_badge() {
    let svg = badge_svg("COCOMO", "4.2 person-months");
    insta::assert_snapshot!("cocomo_person_months_badge", svg);
}

// ── Zero / boundary values ──────────────────────────────────────────

#[test]
fn snapshot_zero_code_lines_badge() {
    let svg = badge_svg("code lines", "0");
    insta::assert_snapshot!("zero_code_lines_badge", svg);
}

#[test]
fn snapshot_zero_percent_badge() {
    let svg = badge_svg("coverage", "0%");
    insta::assert_snapshot!("zero_percent_badge", svg);
}

#[test]
fn snapshot_one_language_badge() {
    let svg = badge_svg("languages", "1");
    insta::assert_snapshot!("one_language_badge", svg);
}

// ── Very large numbers ──────────────────────────────────────────────

#[test]
fn snapshot_million_lines_badge() {
    let svg = badge_svg("lines", "1000000");
    insta::assert_snapshot!("million_lines_badge", svg);
}

#[test]
fn snapshot_huge_file_count_badge() {
    let svg = badge_svg("files", "123456789");
    insta::assert_snapshot!("huge_file_count_badge", svg);
}

#[test]
fn snapshot_max_u64_badge() {
    let svg = badge_svg("max", &u64::MAX.to_string());
    insta::assert_snapshot!("max_u64_badge", svg);
}

// ── Special characters ──────────────────────────────────────────────

#[test]
fn snapshot_ampersand_in_label_badge() {
    let svg = badge_svg("C & C++", "42");
    insta::assert_snapshot!("ampersand_in_label_badge", svg);
}

#[test]
fn snapshot_angle_brackets_in_value_badge() {
    let svg = badge_svg("tag", "<none>");
    insta::assert_snapshot!("angle_brackets_in_value_badge", svg);
}

#[test]
fn snapshot_quotes_in_label_badge() {
    let svg = badge_svg("it's \"fine\"", "ok");
    insta::assert_snapshot!("quotes_in_label_badge", svg);
}

#[test]
fn snapshot_all_xml_specials_badge() {
    let svg = badge_svg("<&\"'>", "<&\"'>");
    insta::assert_snapshot!("all_xml_specials_badge", svg);
}

// ── Empty / whitespace ──────────────────────────────────────────────

#[test]
fn snapshot_empty_both_badge() {
    let svg = badge_svg("", "");
    insta::assert_snapshot!("empty_both_badge", svg);
}

#[test]
fn snapshot_whitespace_label_badge() {
    let svg = badge_svg("   ", "val");
    insta::assert_snapshot!("whitespace_label_badge", svg);
}

// ── Unicode & emoji ─────────────────────────────────────────────────

#[test]
fn snapshot_emoji_badge() {
    let svg = badge_svg("status", "🟢 pass");
    insta::assert_snapshot!("emoji_badge", svg);
}

#[test]
fn snapshot_cjk_label_badge() {
    let svg = badge_svg("代码行数", "1024");
    insta::assert_snapshot!("cjk_label_badge", svg);
}

#[test]
fn snapshot_mixed_script_badge() {
    let svg = badge_svg("Código", "líneas: 99");
    insta::assert_snapshot!("mixed_script_badge", svg);
}

// ── Realistic metric combos ─────────────────────────────────────────

#[test]
fn snapshot_grade_badge() {
    let svg = badge_svg("quality", "A+");
    insta::assert_snapshot!("grade_badge", svg);
}

#[test]
fn snapshot_risk_score_badge() {
    let svg = badge_svg("risk score", "high");
    insta::assert_snapshot!("risk_score_badge", svg);
}

#[test]
fn snapshot_archetype_badge() {
    let svg = badge_svg("archetype", "web-app");
    insta::assert_snapshot!("archetype_badge", svg);
}
