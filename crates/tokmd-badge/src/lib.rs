//! SVG badge rendering helpers.

/// Build a compact two-segment SVG badge.
pub fn badge_svg(label: &str, value: &str) -> String {
    let label_width = (label.len() as i32 * 7 + 20).max(60);
    let value_width = (value.len() as i32 * 7 + 20).max(60);
    let width = label_width + value_width;
    let height = 24;
    let label_x = label_width / 2;
    let value_x = label_width + value_width / 2;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" role=\"img\"><rect width=\"{label_width}\" height=\"{height}\" fill=\"#555\"/><rect x=\"{label_width}\" width=\"{value_width}\" height=\"{height}\" fill=\"#4c9aff\"/><text x=\"{label_x}\" y=\"16\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"11\" text-anchor=\"middle\">{label}</text><text x=\"{value_x}\" y=\"16\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"11\" text-anchor=\"middle\">{value}</text></svg>",
        width = width,
        height = height,
        label_width = label_width,
        value_width = value_width,
        label_x = label_x,
        value_x = value_x,
        label = label,
        value = value
    )
}

#[cfg(test)]
mod tests {
    use super::badge_svg;

    #[test]
    fn badge_svg_contains_label_and_value() {
        let svg = badge_svg("lines", "1234");
        assert!(svg.contains("lines"));
        assert!(svg.contains("1234"));
    }

    #[test]
    fn badge_svg_is_valid_svg() {
        let svg = badge_svg("test", "42");
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    #[test]
    fn badge_svg_dimensions_calculated_correctly() {
        let svg = badge_svg("ab", "1");
        assert!(svg.contains("width=\"120\""));

        let svg = badge_svg("longlabel", "longvalue");
        assert!(svg.contains("width=\"166\""));
    }

    #[test]
    fn badge_svg_positions_are_centered() {
        let svg = badge_svg("ab", "1");
        assert!(svg.contains("x=\"30\""));
        assert!(svg.contains("x=\"90\""));
    }

    #[test]
    fn badge_svg_width_scales_with_text() {
        let short_svg = badge_svg("a", "1");
        let long_svg = badge_svg("averylonglabel", "averylongvalue");
        assert!(extract_svg_width(&long_svg) > extract_svg_width(&short_svg));
    }

    fn extract_svg_width(svg: &str) -> i32 {
        let start = svg.find("width=\"").expect("width attr") + 7;
        let end = svg[start..].find('"').expect("width close") + start;
        svg[start..end].parse().expect("numeric width")
    }
}
