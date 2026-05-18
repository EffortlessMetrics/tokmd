//! Test, TODO, boilerplate, polyglot, and reading-time sections.

use std::fmt::Write;

use tokmd_analysis_types::DerivedReport;

use crate::analysis::markdown::{fmt_f64, fmt_pct};

pub(super) fn render_density_sections(out: &mut String, derived: &DerivedReport) {
    render_test_density(out, derived);
    render_todos(out, derived);
    render_boilerplate_ratio(out, derived);
    render_polyglot(out, derived);
    render_reading_time(out, derived);
}

fn render_test_density(out: &mut String, derived: &DerivedReport) {
    out.push_str("## Test density\n\n");
    let _ = writeln!(
        out,
        "- Test lines: `{}`\n- Prod lines: `{}`\n- Test ratio: `{}`\n",
        derived.test_density.test_lines,
        derived.test_density.prod_lines,
        fmt_pct(derived.test_density.ratio)
    );
}

fn render_todos(out: &mut String, derived: &DerivedReport) {
    if let Some(todo) = &derived.todo {
        out.push_str("## TODOs\n\n");
        let _ = writeln!(
            out,
            "- Total: `{}`\n- Density (per KLOC): `{}`\n",
            todo.total,
            fmt_f64(todo.density_per_kloc, 2)
        );
        out.push_str("|Tag|Count|\n");
        out.push_str("|---|---:|\n");
        for tag in &todo.tags {
            let _ = writeln!(out, "|{}|{}|", tag.tag, tag.count);
        }
        out.push('\n');
    }
}

fn render_boilerplate_ratio(out: &mut String, derived: &DerivedReport) {
    out.push_str("## Boilerplate ratio\n\n");
    let _ = writeln!(
        out,
        "- Infra lines: `{}`\n- Logic lines: `{}`\n- Infra ratio: `{}`\n",
        derived.boilerplate.infra_lines,
        derived.boilerplate.logic_lines,
        fmt_pct(derived.boilerplate.ratio)
    );
}

fn render_polyglot(out: &mut String, derived: &DerivedReport) {
    out.push_str("## Polyglot\n\n");
    let _ = writeln!(
        out,
        "- Languages: `{}`\n- Dominant: `{}` ({})\n- Entropy: `{}`\n",
        derived.polyglot.lang_count,
        derived.polyglot.dominant_lang,
        fmt_pct(derived.polyglot.dominant_pct),
        fmt_f64(derived.polyglot.entropy, 4)
    );
}

fn render_reading_time(out: &mut String, derived: &DerivedReport) {
    out.push_str("## Reading time\n\n");
    let _ = writeln!(
        out,
        "- Minutes: `{}` ({} lines/min)\n",
        fmt_f64(derived.reading_time.minutes, 2),
        derived.reading_time.lines_per_minute
    );
}
