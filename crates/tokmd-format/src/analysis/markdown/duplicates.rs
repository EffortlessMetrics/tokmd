//! Duplicate and near-duplicate Markdown rendering.
//!
//! This module owns exact duplicate groups, duplicate density, near-duplicate
//! clusters, pair detail, truncation warnings, and runtime stats formatting.

use std::fmt::Write;

use super::{fmt_f64, fmt_pct};
use tokmd_analysis_types::DuplicateReport;

pub(super) fn render_duplicate_report(out: &mut String, dup: &DuplicateReport) {
    out.push_str("## Duplicates\n\n");
    let _ = writeln!(
        out,
        "- Wasted bytes: `{}`\n- Strategy: `{}`\n",
        dup.wasted_bytes, dup.strategy
    );
    if let Some(density) = &dup.density {
        out.push_str("### Duplication density\n\n");
        let _ = writeln!(
            out,
            "- Duplicate groups: `{}`\n- Duplicate files: `{}`\n- Duplicated bytes: `{}`\n- Waste vs codebase: `{}`\n",
            density.duplicate_groups,
            density.duplicate_files,
            density.duplicated_bytes,
            fmt_pct(density.wasted_pct_of_codebase)
        );
        if !density.by_module.is_empty() {
            out.push_str(
                "|Module|Dup files|Wasted files|Dup bytes|Wasted bytes|Module bytes|Density|\n",
            );
            out.push_str("|---|---:|---:|---:|---:|---:|---:|\n");
            for row in density.by_module.iter().take(10) {
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{}|{}|{}|{}|",
                    row.module,
                    row.duplicate_files,
                    row.wasted_files,
                    row.duplicated_bytes,
                    row.wasted_bytes,
                    row.module_bytes,
                    fmt_pct(row.density)
                );
            }
            out.push('\n');
        }
    }
    if !dup.groups.is_empty() {
        out.push_str("|Hash|Bytes|Files|\n");
        out.push_str("|---|---:|---:|\n");
        for row in dup.groups.iter().take(10) {
            let _ = writeln!(out, "|{}|{}|{}|", row.hash, row.bytes, row.files.len());
        }
        out.push('\n');
    }

    if let Some(near) = &dup.near {
        out.push_str("### Near duplicates\n\n");
        let _ = writeln!(
            out,
            "- Files analyzed: `{}`\n- Files skipped: `{}`\n- Threshold: `{}`\n- Scope: `{:?}`",
            near.files_analyzed,
            near.files_skipped,
            fmt_f64(near.params.threshold, 2),
            near.params.scope
        );
        if let Some(eligible) = near.eligible_files {
            let _ = writeln!(out, "- Eligible files: `{}`", eligible);
        }
        if near.truncated {
            out.push_str("- **Warning**: Pair list truncated by `max_pairs` limit.\n");
        }
        out.push('\n');

        // Clusters (primary human-facing view)
        if let Some(clusters) = &near.clusters
            && !clusters.is_empty()
        {
            out.push_str("#### Clusters\n\n");
            out.push_str("|#|Files|Max Similarity|Representative|Pairs|\n");
            out.push_str("|---:|---:|---:|---|---:|\n");
            for (i, cluster) in clusters.iter().enumerate() {
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{}|{}|",
                    i + 1,
                    cluster.files.len(),
                    fmt_pct(cluster.max_similarity),
                    cluster.representative,
                    cluster.pair_count
                );
            }
            out.push('\n');
        }

        // Pairs (detail view)
        if near.pairs.is_empty() {
            out.push_str("- No near-duplicate pairs detected.\n\n");
        } else {
            out.push_str("#### Pairs\n\n");
            out.push_str("|Left|Right|Similarity|Shared FPs|\n");
            out.push_str("|---|---|---:|---:|\n");
            for pair in near.pairs.iter().take(20) {
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{}|",
                    pair.left,
                    pair.right,
                    fmt_pct(pair.similarity),
                    pair.shared_fingerprints
                );
            }
            out.push('\n');
        }

        // Runtime stats footer
        if let Some(stats) = &near.stats {
            let _ = writeln!(
                out,
                "> Near-dup stats: fingerprinting {}ms, pairing {}ms, {} bytes processed\n",
                stats.fingerprinting_ms, stats.pairing_ms, stats.bytes_processed
            );
        }
    }
}
