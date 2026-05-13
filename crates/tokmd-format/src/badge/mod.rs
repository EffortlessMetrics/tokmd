//! SVG badge rendering helpers.

fn escape_xml_text(s: &str) -> String {
    // Minimal XML escaping for text nodes to keep SVG valid and safe.
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Build a compact two-segment SVG badge.
pub fn badge_svg(label: &str, value: &str) -> String {
    // Width is heuristic; char count avoids UTF-8 byte-length drift.
    let label_chars = label.chars().count() as i32;
    let value_chars = value.chars().count() as i32;
    let label_width = (label_chars * 7 + 20).max(60);
    let value_width = (value_chars * 7 + 20).max(60);
    let width = label_width + value_width;
    let height = 24;
    let label_x = label_width / 2;
    let value_x = label_width + value_width / 2;
    let label_escaped = escape_xml_text(label);
    let value_escaped = escape_xml_text(value);
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" role=\"img\"><rect width=\"{label_width}\" height=\"{height}\" fill=\"#555\"/><rect x=\"{label_width}\" width=\"{value_width}\" height=\"{height}\" fill=\"#4c9aff\"/><text x=\"{label_x}\" y=\"16\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"11\" text-anchor=\"middle\">{label}</text><text x=\"{value_x}\" y=\"16\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"11\" text-anchor=\"middle\">{value}</text></svg>",
        width = width,
        height = height,
        label_width = label_width,
        value_width = value_width,
        label_x = label_x,
        value_x = value_x,
        label = label_escaped,
        value = value_escaped
    )
}

#[cfg(test)]
mod tests {
    use super::{badge_svg, escape_xml_text};

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
    fn badge_svg_width_scales_with_text() -> Result<(), String> {
        let short_svg = badge_svg("a", "1");
        let long_svg = badge_svg("averylonglabel", "averylongvalue");
        assert!(extract_svg_width(&long_svg)? > extract_svg_width(&short_svg)?);
        Ok(())
    }

    #[test]
    fn badge_svg_escapes_xml_text_nodes() {
        let label = "a<&>\"'";
        let value = "b<&>\"'";
        let svg = badge_svg(label, value);
        assert!(svg.contains(&escape_xml_text(label)));
        assert!(svg.contains(&escape_xml_text(value)));
        assert!(!svg.contains(label));
        assert!(!svg.contains(value));
    }

    #[test]
    fn extract_svg_width_reports_malformed_svg() {
        assert!(extract_svg_width("<svg></svg>").is_err());
        assert!(extract_svg_width("<svg width=\"abc\"></svg>").is_err());
    }

    fn extract_svg_width(svg: &str) -> Result<i32, String> {
        let start = svg
            .find("width=\"")
            .ok_or_else(|| "missing SVG width attribute".to_string())?
            + 7;
        let end = svg[start..]
            .find('"')
            .map(|offset| offset + start)
            .ok_or_else(|| "unterminated SVG width attribute".to_string())?;
        svg[start..end]
            .parse()
            .map_err(|error| format!("invalid SVG width `{}`: {error}", &svg[start..end]))
    }
}
