use tokmd_config as cli;

pub(crate) fn badge_metric_label(metric: cli::BadgeMetric) -> &'static str {
    match metric {
        cli::BadgeMetric::Lines => "lines",
        cli::BadgeMetric::Tokens => "tokens",
        cli::BadgeMetric::Bytes => "bytes",
        cli::BadgeMetric::Doc => "doc",
        cli::BadgeMetric::Blank => "blank",
        cli::BadgeMetric::Hotspot => "hotspot",
    }
}

pub(crate) fn badge_svg(label: &str, value: &str) -> String {
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
    use super::*;

    #[test]
    fn test_badge_metric_label_all_variants() {
        assert_eq!(badge_metric_label(cli::BadgeMetric::Lines), "lines");
        assert_eq!(badge_metric_label(cli::BadgeMetric::Tokens), "tokens");
        assert_eq!(badge_metric_label(cli::BadgeMetric::Bytes), "bytes");
        assert_eq!(badge_metric_label(cli::BadgeMetric::Doc), "doc");
        assert_eq!(badge_metric_label(cli::BadgeMetric::Blank), "blank");
        assert_eq!(badge_metric_label(cli::BadgeMetric::Hotspot), "hotspot");
    }

    #[test]
    fn test_badge_svg_contains_label_and_value() {
        let svg = badge_svg("lines", "1234");
        assert!(svg.contains("lines"));
        assert!(svg.contains("1234"));
    }

    #[test]
    fn test_badge_svg_is_valid_svg() {
        let svg = badge_svg("test", "42");
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    #[test]
    fn test_badge_svg_dimensions_calculated_correctly() {
        // Test with short label "ab" (2 chars): 2*7+20 = 34, but min is 60
        let svg = badge_svg("ab", "1");
        assert!(svg.contains("width=\"120\"")); // 60 + 60 = 120

        // Test with longer label "longlabel" (9 chars): 9*7+20 = 83
        let svg = badge_svg("longlabel", "longvalue");
        // label_width = 9*7+20 = 83, value_width = 9*7+20 = 83
        assert!(svg.contains("width=\"166\"")); // 83 + 83 = 166
    }

    #[test]
    fn test_badge_svg_label_x_centered() {
        // label_width = 60 (min), label_x = 60/2 = 30
        let svg = badge_svg("ab", "1");
        // Check that label text element is positioned correctly
        assert!(svg.contains("x=\"30\"")); // label_x
    }

    #[test]
    fn test_badge_svg_value_x_positioned_after_label() {
        // label_width = 60, value_width = 60
        // value_x = 60 + 60/2 = 90
        let svg = badge_svg("ab", "1");
        assert!(svg.contains("x=\"90\"")); // value_x
    }

    #[test]
    fn test_badge_svg_height_is_24() {
        let svg = badge_svg("label", "value");
        assert!(svg.contains("height=\"24\""));
    }

    #[test]
    fn test_badge_svg_arithmetic_operations() {
        // This test ensures the arithmetic in badge_svg is correct
        // by verifying specific calculated values

        // For label "test" (4 chars) and value "100" (3 chars):
        // label_width = 4*7 + 20 = 48, max(48, 60) = 60
        // value_width = 3*7 + 20 = 41, max(41, 60) = 60
        // width = 60 + 60 = 120
        // label_x = 60/2 = 30
        // value_x = 60 + 60/2 = 90
        let svg = badge_svg("test", "100");

        assert!(svg.contains("width=\"120\""));
        assert!(svg.contains("x=\"30\""));
        assert!(svg.contains("x=\"90\""));
    }

    #[test]
    fn test_badge_svg_width_scales_with_text() {
        // Short text uses minimum width
        let short_svg = badge_svg("a", "1");

        // Longer text should have larger width
        let long_svg = badge_svg("averylonglabel", "averylongvalue");

        // Extract width from both SVGs
        let short_width = extract_svg_width(&short_svg);
        let long_width = extract_svg_width(&long_svg);

        assert!(
            long_width > short_width,
            "Longer text should have wider badge"
        );
    }

    fn extract_svg_width(svg: &str) -> i32 {
        // Simple extraction of first width attribute value
        let start = svg.find("width=\"").unwrap() + 7;
        let end = svg[start..].find('"').unwrap() + start;
        svg[start..end].parse().unwrap()
    }
}
