//! Markdown table writers for diff receipts.

use std::fmt::Write as FmtWrite;

use tokmd_types::{DiffRow, DiffTotals};

use super::math::percent_change;
use super::movement::LanguageMovement;
use super::style::{DiffRenderOptions, format_delta_colored, format_pct_delta_colored};

pub(super) fn write_compact_summary(
    s: &mut String,
    totals: &DiffTotals,
    movement: LanguageMovement,
    options: DiffRenderOptions,
) {
    s.push_str("### Summary\n\n");
    s.push_str("|Metric|Value|\n");
    s.push_str("|---|---:|\n");
    let _ = writeln!(s, "|From LOC|{}|", totals.old_code);
    let _ = writeln!(s, "|To LOC|{}|", totals.new_code);
    let _ = writeln!(
        s,
        "|Delta LOC|{}|",
        format_delta_colored(totals.delta_code, options.color)
    );
    let _ = writeln!(
        s,
        "|LOC Change|{}|",
        format_pct_delta_colored(
            percent_change(totals.old_code, totals.new_code),
            options.color
        )
    );
    let _ = writeln!(
        s,
        "|Delta Lines|{}|",
        format_delta_colored(totals.delta_lines, options.color)
    );
    let _ = writeln!(
        s,
        "|Delta Files|{}|",
        format_delta_colored(totals.delta_files, options.color)
    );
    let _ = writeln!(
        s,
        "|Delta Bytes|{}|",
        format_delta_colored(totals.delta_bytes, options.color)
    );
    let _ = writeln!(
        s,
        "|Delta Tokens|{}|",
        format_delta_colored(totals.delta_tokens, options.color)
    );
    let _ = writeln!(s, "|Languages changed|{}|", movement.changed);
    let _ = writeln!(s, "|Languages added|{}|", movement.added);
    let _ = writeln!(s, "|Languages removed|{}|", movement.removed);
    let _ = writeln!(s, "|Languages modified|{}|", movement.modified);
}

pub(super) fn write_summary_table(s: &mut String, totals: &DiffTotals, options: DiffRenderOptions) {
    s.push_str("### Summary\n\n");
    s.push_str("|Metric|From|To|Delta|Change|\n");
    s.push_str("|---|---:|---:|---:|---:|\n");

    let _ = writeln!(
        s,
        "|LOC|{}|{}|{}|{}|",
        totals.old_code,
        totals.new_code,
        format_delta_colored(totals.delta_code, options.color),
        format_pct_delta_colored(
            percent_change(totals.old_code, totals.new_code),
            options.color
        )
    );
    let _ = writeln!(
        s,
        "|Lines|{}|{}|{}|{}|",
        totals.old_lines,
        totals.new_lines,
        format_delta_colored(totals.delta_lines, options.color),
        format_pct_delta_colored(
            percent_change(totals.old_lines, totals.new_lines),
            options.color
        )
    );
    let _ = writeln!(
        s,
        "|Files|{}|{}|{}|{}|",
        totals.old_files,
        totals.new_files,
        format_delta_colored(totals.delta_files, options.color),
        format_pct_delta_colored(
            percent_change(totals.old_files, totals.new_files),
            options.color
        )
    );
    let _ = writeln!(
        s,
        "|Bytes|{}|{}|{}|{}|",
        totals.old_bytes,
        totals.new_bytes,
        format_delta_colored(totals.delta_bytes, options.color),
        format_pct_delta_colored(
            percent_change(totals.old_bytes, totals.new_bytes),
            options.color
        )
    );
    let _ = writeln!(
        s,
        "|Tokens|{}|{}|{}|{}|",
        totals.old_tokens,
        totals.new_tokens,
        format_delta_colored(totals.delta_tokens, options.color),
        format_pct_delta_colored(
            percent_change(totals.old_tokens, totals.new_tokens),
            options.color
        )
    );
    s.push('\n');
}

pub(super) fn write_movement_table(s: &mut String, movement: LanguageMovement) {
    s.push_str("### Language Movement\n\n");
    s.push_str("|Type|Count|\n");
    s.push_str("|---|---:|\n");
    let _ = writeln!(s, "|Changed|{}|", movement.changed);
    let _ = writeln!(s, "|Added|{}|", movement.added);
    let _ = writeln!(s, "|Removed|{}|", movement.removed);
    let _ = writeln!(s, "|Modified|{}|", movement.modified);
    s.push('\n');
}

pub(super) fn write_language_breakdown(
    s: &mut String,
    rows: &[DiffRow],
    totals: &DiffTotals,
    options: DiffRenderOptions,
) {
    s.push_str("### Language Breakdown\n\n");
    s.push_str("|Language|Old LOC|New LOC|Delta|\n");
    s.push_str("|---|---:|---:|---:|\n");

    for row in rows {
        let _ = writeln!(
            s,
            "|{}|{}|{}|{}|",
            row.lang,
            row.old_code,
            row.new_code,
            format_delta_colored(row.delta_code, options.color)
        );
    }

    let _ = writeln!(
        s,
        "|**Total**|{}|{}|{}|",
        totals.old_code,
        totals.new_code,
        format_delta_colored(totals.delta_code, options.color)
    );
}
