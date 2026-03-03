use tokmd_badge::badge_svg;

#[test]
fn snapshot_badge_small_project() {
    let svg = badge_svg("lines", "100");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_medium_project() {
    let svg = badge_svg("lines", "10,000");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_large_project() {
    let svg = badge_svg("lines", "1,000,000");
    insta::assert_snapshot!(svg);
}

#[test]
fn snapshot_badge_xml_escape() {
    let svg = badge_svg("a<b", "1&2");
    insta::assert_snapshot!(svg);
}
