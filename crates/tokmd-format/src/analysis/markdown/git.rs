//! Git metrics Markdown rendering.
//!
//! This module owns the Markdown section for git-derived review and history
//! evidence: hotspots, freshness, age distribution, coupling, and commit intent.

use std::fmt::Write;

use super::{fmt_f64, fmt_pct};
use tokmd_analysis_types::GitReport;

pub(super) fn render_git_report(out: &mut String, git: &GitReport) {
    out.push_str("## Git metrics\n\n");
    let _ = writeln!(
        out,
        "- Commits scanned: `{}`\n- Files seen: `{}`\n",
        git.commits_scanned, git.files_seen
    );
    if !git.hotspots.is_empty() {
        out.push_str("### Hotspots\n\n");
        out.push_str("|File|Commits|Lines|Score|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for row in git.hotspots.iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.path, row.commits, row.lines, row.score
            );
        }
        out.push('\n');
    }
    if !git.bus_factor.is_empty() {
        out.push_str("### Bus factor\n\n");
        out.push_str("|Module|Authors|\n");
        out.push_str("|---|---:|\n");
        for row in git.bus_factor.iter().take(10) {
            let _ = writeln!(out, "|{}|{}|", row.module, row.authors);
        }
        out.push('\n');
    }
    out.push_str("### Freshness\n\n");
    let _ = writeln!(
        out,
        "- Stale threshold (days): `{}`\n- Stale files: `{}` / `{}` ({})\n",
        git.freshness.threshold_days,
        git.freshness.stale_files,
        git.freshness.total_files,
        fmt_pct(git.freshness.stale_pct)
    );
    if !git.freshness.by_module.is_empty() {
        out.push_str("|Module|Avg days|P90 days|Stale%|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for row in git.freshness.by_module.iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.module,
                fmt_f64(row.avg_days, 2),
                fmt_f64(row.p90_days, 2),
                fmt_pct(row.stale_pct)
            );
        }
        out.push('\n');
    }
    if let Some(age) = &git.age_distribution {
        out.push_str("### Code age\n\n");
        let _ = writeln!(
            out,
            "- Refresh trend: `{:?}` (recent: `{}`, prior: `{}`)\n",
            age.refresh_trend, age.recent_refreshes, age.prior_refreshes
        );
        if !age.buckets.is_empty() {
            out.push_str("|Bucket|Min days|Max days|Files|Pct|\n");
            out.push_str("|---|---:|---:|---:|---:|\n");
            for bucket in &age.buckets {
                let max = bucket
                    .max_days
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "∞".to_string());
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{}|{}|",
                    bucket.label,
                    bucket.min_days,
                    max,
                    bucket.files,
                    fmt_pct(bucket.pct)
                );
            }
            out.push('\n');
        }
    }
    if !git.coupling.is_empty() {
        // Minimum-support filter: only render rows with count >= 2 to prevent
        // lift spikes on rare pairs. JSON always includes all rows.
        let filtered: Vec<_> = git.coupling.iter().filter(|r| r.count >= 2).collect();
        if !filtered.is_empty() {
            out.push_str("### Coupling\n\n");
            out.push_str("|Left|Right|Count|Jaccard|Lift|\n");
            out.push_str("|---|---|---:|---:|---:|\n");
            for row in filtered.iter().take(10) {
                let jaccard = row
                    .jaccard
                    .map(|v| fmt_f64(v, 4))
                    .unwrap_or_else(|| "-".to_string());
                let lift = row
                    .lift
                    .map(|v| fmt_f64(v, 4))
                    .unwrap_or_else(|| "-".to_string());
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{}|{}|",
                    row.left, row.right, row.count, jaccard, lift
                );
            }
            out.push('\n');
        }
    }

    if let Some(intent) = &git.intent {
        out.push_str("### Commit intent\n\n");
        out.push_str("|Type|Count|\n");
        out.push_str("|---|---:|\n");
        let o = &intent.overall;
        let entries = [
            ("feat", o.feat),
            ("fix", o.fix),
            ("refactor", o.refactor),
            ("docs", o.docs),
            ("test", o.test),
            ("chore", o.chore),
            ("ci", o.ci),
            ("build", o.build),
            ("perf", o.perf),
            ("style", o.style),
            ("revert", o.revert),
            ("other", o.other),
        ];
        for (name, count) in entries {
            if count > 0 {
                let _ = writeln!(out, "|{}|{}|", name, count);
            }
        }
        let _ = writeln!(out, "|**total**|{}|", o.total);
        let _ = writeln!(out, "\n- Unknown: `{}`", fmt_pct(intent.unknown_pct));
        if let Some(cr) = intent.corrective_ratio {
            let _ = writeln!(
                out,
                "- Corrective ratio (fix+revert/total): `{}`",
                fmt_pct(cr)
            );
        }
        out.push('\n');

        // Maintenance hotspots: modules with highest fix+revert share
        let mut maintenance: Vec<_> = intent
            .by_module
            .iter()
            .filter(|m| m.counts.total > 0)
            .map(|m| {
                let fix_revert = m.counts.fix + m.counts.revert;
                let share = fix_revert as f64 / m.counts.total as f64;
                (m, share)
            })
            .filter(|(_, share)| *share > 0.0)
            .collect();
        maintenance.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.module.cmp(&b.0.module))
        });

        if !maintenance.is_empty() {
            out.push_str("#### Maintenance hotspots\n\n");
            out.push_str("|Module|Fix+Revert|Total|Share|\n");
            out.push_str("|---|---:|---:|---:|\n");
            for (m, share) in maintenance.iter().take(10) {
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{}|",
                    m.module,
                    m.counts.fix + m.counts.revert,
                    m.counts.total,
                    fmt_pct(*share)
                );
            }
            out.push('\n');
        }
    }
}
