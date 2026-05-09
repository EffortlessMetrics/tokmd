//! Topic cloud Markdown rendering.
//!
//! This module owns overall and per-module topic list rendering.

use std::fmt::Write;

use tokmd_analysis_types::TopicClouds;

pub(super) fn render_topic_clouds(out: &mut String, topics: &TopicClouds) {
    out.push_str("## Topics\n\n");
    if !topics.overall.is_empty() {
        let _ = writeln!(
            out,
            "- Overall: `{}`",
            topics
                .overall
                .iter()
                .map(|t| t.term.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    for (module, terms) in &topics.per_module {
        if terms.is_empty() {
            continue;
        }
        let line = terms
            .iter()
            .map(|t| t.term.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(out, "- `{}`: {}", module, line);
    }
    out.push('\n');
}
