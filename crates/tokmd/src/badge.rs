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
