//! Markdown rendering for diff receipts.
//!
//! This module owns high-level diff Markdown composition. SRP submodules own
//! color/style formatting, numeric helpers, language movement classification,
//! and individual Markdown table writers.

use std::fmt::Write as FmtWrite;

use tokmd_types::{DiffRow, DiffTotals};

mod math;
mod movement;
mod style;
mod tables;

pub use style::{DiffColorMode, DiffRenderOptions};

use movement::LanguageMovement;
use tables::{
    write_compact_summary, write_language_breakdown, write_movement_table, write_summary_table,
};

/// Render diff as Markdown table with optional compact/color behavior.
pub fn render_diff_md_with_options(
    from_source: &str,
    to_source: &str,
    rows: &[DiffRow],
    totals: &DiffTotals,
    options: DiffRenderOptions,
) -> String {
    // Heuristic: (rows + 20) * 80 chars per row
    let mut s = String::with_capacity((rows.len() + 20) * 80);

    let _ = writeln!(s, "## Diff: {} → {}", from_source, to_source);
    s.push('\n');

    let movement = LanguageMovement::from_rows(rows);

    if options.compact {
        write_compact_summary(&mut s, totals, movement, options);
        return s;
    }

    write_summary_table(&mut s, totals, options);
    write_movement_table(&mut s, movement);
    write_language_breakdown(&mut s, rows, totals, options);

    s
}

/// Render diff as Markdown table.
pub fn render_diff_md(
    from_source: &str,
    to_source: &str,
    rows: &[DiffRow],
    totals: &DiffTotals,
) -> String {
    render_diff_md_with_options(
        from_source,
        to_source,
        rows,
        totals,
        DiffRenderOptions::default(),
    )
}
