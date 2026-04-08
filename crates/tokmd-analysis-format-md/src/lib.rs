//! # tokmd-analysis-format-md
//!
//! **Tier 3 (Formatting)**
//!
//! Markdown formatter for tokmd analysis receipts.
//!
//! ## Purpose
//!
//! Renders `AnalysisReceipt` structures as Markdown reports. This microcrate
//! is extracted from `tokmd-analysis-format` to follow the Single Responsibility
//! Principle.
//!
//! ## Usage
//!
//! ```rust
//! use tokmd_analysis_types::AnalysisReceipt;
//! use tokmd_analysis_format_md::render_md;
//!
//! fn generate_report(receipt: &AnalysisReceipt) -> String {
//!     render_md(receipt)
//! }
//! ```

use std::fmt::Write;
use tokmd_analysis_types::{
    AnalysisReceipt, CocomoReport, DerivedReport, EffortDriverDirection,
    EffortEstimateReport, FileStatRow,
};

/// Renders an analysis receipt as a Markdown report.
pub fn render_md(receipt: &AnalysisReceipt) -> String {
    let mut out = String::new();
    out.push_str("# tokmd analysis\n\n");
    let _ = writeln!(out, "Preset: `{}`\n", receipt.args.preset);

    if !receipt.source.inputs.is_empty() {
        out.push_str("## Inputs\n\n");
        for input in &receipt.source.inputs {
            let _ = writeln!(out, "- `{}`", input);
        }
        out.push('\n');
    }

    if let Some(archetype) = &receipt.archetype {
        out.push_str("## Archetype\n\n");
        let _ = writeln!(out, "- Kind: `{}`", archetype.kind);
        if !archetype.evidence.is_empty() {
            let _ = writeln!(out, "- Evidence: `{}`", archetype.evidence.join("`, `"));
        }
        out.push('\n');
    }

    if let Some(topics) = &receipt.topics {
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

    if let Some(entropy) = &receipt.entropy {
        out.push_str("## Entropy profiling\n\n");
        if entropy.suspects.is_empty() {
            out.push_str("- No entropy outliers detected.\n\n");
        } else {
            out.push_str("|Path|Module|Entropy|Sample bytes|Class|\n");
            out.push_str("|---|---|---:|---:|---|\n");
            for row in entropy.suspects.iter().take(10) {
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{}|{:?}|",
                    row.path,
                    row.module,
                    fmt_f64(row.entropy_bits_per_byte as f64, 2),
                    row.sample_bytes,
                    row.class
                );
            }
            out.push('\n');
        }
    }

    if let Some(license) = &receipt.license {
        out.push_str("## License radar\n\n");
        if let Some(effective) = &license.effective {
            let _ = writeln!(out, "- Effective: `{}`", effective);
        }
        out.push_str("- Heuristic detection; not legal advice.\n\n");
        if !license.findings.is_empty() {
            out.push_str("|SPDX|Confidence|Source|Kind|\n");
            out.push_str("|---|---:|---|---|\n");
            for row in license.findings.iter().take(10) {
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{:?}|",
                    row.spdx,
                    fmt_f64(row.confidence as f64, 2),
                    row.source_path,
                    row.source_kind
                );
            }
            out.push('\n');
        }
    }

    if let Some(fingerprint) = &receipt.corporate_fingerprint {
        out.push_str("## Corporate fingerprint\n\n");
        if fingerprint.domains.is_empty() {
            out.push_str("- No commit domains detected.\n\n");
        } else {
            out.push_str("|Domain|Commits|Pct|\n");
            out.push_str("|---|---:|---:|\n");
            for row in fingerprint.domains.iter().take(10) {
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|",
                    row.domain,
                    row.commits,
                    fmt_pct(row.pct as f64)
                );
            }
            out.push('\n');
        }
    }

    if let Some(churn) = &receipt.predictive_churn {
        out.push_str("## Predictive churn\n\n");
        let mut rows: Vec<_> = churn.per_module.iter().collect();
        rows.sort_by(|a, b| {
            b.1.slope
                .partial_cmp(&a.1.slope)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(b.0))
        });
        if rows.is_empty() {
            out.push_str("- No churn signals detected.\n\n");
        } else {
            out.push_str("|Module|Slope|R²|Recent change|Class|\n");
            out.push_str("|---|---:|---:|---:|---|\n");
            for (module, trend) in rows.into_iter().take(10) {
                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{}|{:?}|",
                    module,
                    fmt_f64(trend.slope, 4),
                    fmt_f64(trend.r2, 2),
                    trend.recent_change,
                    trend.classification
                );
            }
            out.push('\n');
        }
    }

    if let Some(derived) = &receipt.derived {
        render_derived_section(&mut out, receipt, derived);
    }

    if let Some(assets) = &receipt.assets {
        render_assets_section(&mut out, assets);
    }

    if let Some(deps) = &receipt.deps {
        render_deps_section(&mut out, deps);
    }

    if let Some(git) = &receipt.git {
        render_git_section(&mut out, git);
    }

    if let Some(imports) = &receipt.imports {
        render_imports_section(&mut out, imports);
    }

    if let Some(dup) = &receipt.dup {
        render_dup_section(&mut out, dup);
    }

    if let Some(cx) = &receipt.complexity {
        render_complexity_section(&mut out, cx);
    }

    if let Some(api) = &receipt.api_surface {
        render_api_section(&mut out, api);
    }

    if let Some(fun) = &receipt.fun
        && let Some(label) = &fun.eco_label
    {
        out.push_str("## Eco label\n\n");
        let _ = writeln!(
            out,
            "- Label: `{}`\n- Score: `{}`\n- Bytes: `{}`\n- Notes: `{}`\n",
            label.label,
            fmt_f64(label.score, 1),
            label.bytes,
            label.notes
        );
    }

    out
}

fn render_derived_section(out: &mut String, receipt: &AnalysisReceipt, derived: &DerivedReport) {
    out.push_str("## Totals\n\n");
    out.push_str("|Files|Code|Comments|Blanks|Lines|Bytes|Tokens|\n");
    out.push_str("|---:|---:|---:|---:|---:|---:|---:|\n");
    let _ = writeln!(
        out,
        "|{}|{}|{}|{}|{}|{}|{}|\n",
        derived.totals.files,
        derived.totals.code,
        derived.totals.comments,
        derived.totals.blanks,
        derived.totals.lines,
        derived.totals.bytes,
        derived.totals.tokens
    );

    out.push_str("## Ratios\n\n");
    out.push_str("|Metric|Value|\n");
    out.push_str("|---|---:|\n");
    let _ = writeln!(out, "|Doc density|{}|", fmt_pct(derived.doc_density.total.ratio));
    let _ = writeln!(out, "|Whitespace ratio|{}|", fmt_pct(derived.whitespace.total.ratio));
    let _ = writeln!(out, "|Bytes per line|{}|\n", fmt_f64(derived.verbosity.total.rate, 2));

    if !derived.doc_density.by_lang.is_empty() {
        out.push_str("### Doc density by language\n\n");
        out.push_str("|Lang|Doc%|Comments|Code|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for row in derived.doc_density.by_lang.iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.key,
                fmt_pct(row.ratio),
                row.numerator,
                row.denominator.saturating_sub(row.numerator)
            );
        }
        out.push('\n');
    }

    out.push_str("### Whitespace ratio by language\n\n");
    out.push_str("|Lang|Blank%|Blanks|Code+Comments|\n");
    out.push_str("|---|---:|---:|---:|\n");
    for row in derived.whitespace.by_lang.iter().take(10) {
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|",
            row.key,
            fmt_pct(row.ratio),
            row.numerator,
            row.denominator
        );
    }
    out.push('\n');

    out.push_str("### Verbosity by language\n\n");
    out.push_str("|Lang|Bytes/Line|Bytes|Lines|\n");
    out.push_str("|---|---:|---:|---:|\n");
    for row in derived.verbosity.by_lang.iter().take(10) {
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|",
            row.key,
            fmt_f64(row.rate, 2),
            row.numerator,
            row.denominator
        );
    }
    out.push('\n');

    out.push_str("## Distribution\n\n");
    out.push_str("|Count|Min|Max|Mean|Median|P90|P99|Gini|\n");
    out.push_str("|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    let _ = writeln!(
        out,
        "|{}|{}|{}|{}|{}|{}|{}|{}|\n",
        derived.distribution.count,
        derived.distribution.min,
        derived.distribution.max,
        fmt_f64(derived.distribution.mean, 2),
        fmt_f64(derived.distribution.median, 2),
        fmt_f64(derived.distribution.p90, 2),
        fmt_f64(derived.distribution.p99, 2),
        fmt_f64(derived.distribution.gini, 4)
    );

    out.push_str("## File size histogram\n\n");
    out.push_str("|Bucket|Min|Max|Files|Pct|\n");
    out.push_str("|---|---:|---:|---:|---:|\n");
    for bucket in &derived.histogram {
        let max = bucket.max.map(|v| v.to_string()).unwrap_or_else(|| "∞".to_string());
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|{}|",
            bucket.label,
            bucket.min,
            max,
            bucket.files,
            fmt_pct(bucket.pct)
        );
    }
    out.push('\n');

    out.push_str("## Top offenders\n\n");
    out.push_str("### Largest files by lines\n\n");
    out.push_str(&render_file_table(&derived.top.largest_lines));
    out.push('\n');

    out.push_str("### Largest files by tokens\n\n");
    out.push_str(&render_file_table(&derived.top.largest_tokens));
    out.push('\n');

    out.push_str("### Largest files by bytes\n\n");
    out.push_str(&render_file_table(&derived.top.largest_bytes));
    out.push('\n');

    out.push_str("### Least documented (min LOC)\n\n");
    out.push_str(&render_file_table(&derived.top.least_documented));
    out.push('\n');

    out.push_str("### Most dense (bytes/line)\n\n");
    out.push_str(&render_file_table(&derived.top.most_dense));
    out.push('\n');

    out.push_str("## Structure\n\n");
    let _ = writeln!(
        out,
        "- Max depth: `{}`\n- Avg depth: `{}`\n",
        derived.nesting.max,
        fmt_f64(derived.nesting.avg, 2)
    );

    out.push_str("## Test density\n\n");
    let _ = writeln!(
        out,
        "- Test lines: `{}`\n- Prod lines: `{}`\n- Test ratio: `{}`\n",
        derived.test_density.test_lines,
        derived.test_density.prod_lines,
        fmt_pct(derived.test_density.ratio)
    );

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

    out.push_str("## Boilerplate ratio\n\n");
    let _ = writeln!(
        out,
        "- Infra lines: `{}`\n- Logic lines: `{}`\n- Infra ratio: `{}`\n",
        derived.boilerplate.infra_lines,
        derived.boilerplate.logic_lines,
        fmt_pct(derived.boilerplate.ratio)
    );

    out.push_str("## Polyglot\n\n");
    let _ = writeln!(
        out,
        "- Languages: `{}`\n- Dominant: `{}` ({})\n- Entropy: `{}`\n",
        derived.polyglot.lang_count,
        derived.polyglot.dominant_lang,
        fmt_pct(derived.polyglot.dominant_pct),
        fmt_f64(derived.polyglot.entropy, 4)
    );

    out.push_str("## Reading time\n\n");
    let _ = writeln!(
        out,
        "- Minutes: `{}` ({} lines/min)\n",
        fmt_f64(derived.reading_time.minutes, 2),
        derived.reading_time.lines_per_minute
    );

    if let Some(context) = &derived.context_window {
        out.push_str("## Context window\n\n");
        let _ = writeln!(
            out,
            "- Window tokens: `{}`\n- Total tokens: `{}`\n- Utilization: `{}`\n- Fits: `{}`\n",
            context.window_tokens,
            context.total_tokens,
            fmt_pct(context.pct),
            context.fits
        );
    }

    if let Some(effort) = &receipt.effort {
        render_effort_report(out, effort);
    } else if let Some(cocomo) = &derived.cocomo {
        render_legacy_cocomo_report(out, derived, cocomo);
    }

    out.push_str("## Integrity\n\n");
    let _ = writeln!(
        out,
        "- Hash: `{}` (`{}`)\n- Entries: `{}`\n",
        derived.integrity.hash,
        derived.integrity.algo,
        derived.integrity.entries
    );
}

fn render_assets_section(out: &mut String, assets: &tokmd_analysis_types::AssetReport) {
    out.push_str("## Assets\n\n");
    let _ = writeln!(
        out,
        "- Total files: `{}`\n- Total bytes: `{}`\n",
        assets.total_files, assets.total_bytes
    );
    if !assets.categories.is_empty() {
        out.push_str("|Category|Files|Bytes|Extensions|\n");
        out.push_str("|---|---:|---:|---|\n");
        for row in &assets.categories {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.category,
                row.files,
                row.bytes,
                row.extensions.join(", ")
            );
        }
        out.push('\n');
    }
    if !assets.top_files.is_empty() {
        out.push_str("|File|Bytes|Category|\n");
        out.push_str("|---|---:|---|\n");
        for row in &assets.top_files {
            let _ = writeln!(out, "|{}|{}|{}|", row.path, row.bytes, row.category);
        }
        out.push('\n');
    }
}

fn render_deps_section(out: &mut String, deps: &tokmd_analysis_types::DependencyReport) {
    out.push_str("## Dependencies\n\n");
    let _ = writeln!(out, "- Total: `{}`\n", deps.total);
    if !deps.lockfiles.is_empty() {
        out.push_str("|Lockfile|Kind|Dependencies|\n");
        out.push_str("|---|---|---:|\n");
        for row in &deps.lockfiles {
            let _ = writeln!(out, "|{}|{}|{}|", row.path, row.kind, row.dependencies);
        }
        out.push('\n');
    }
}

fn render_git_section(out: &mut String, git: &tokmd_analysis_types::GitReport) {
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

fn render_imports_section(out: &mut String, imports: &tokmd_analysis_types::ImportReport) {
    out.push_str("## Imports\n\n");
    let _ = writeln!(out, "- Granularity: `{}`\n", imports.granularity);
    if !imports.edges.is_empty() {
        out.push_str("|From|To|Count|\n");
        out.push_str("|---|---|---:|\n");
        for row in imports.edges.iter().take(20) {
            let _ = writeln!(out, "|{}|{}|{}|", row.from, row.to, row.count);
        }
        out.push('\n');
    }
}

fn render_dup_section(out: &mut String, dup: &tokmd_analysis_types::DuplicateReport) {
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

        if let Some(stats) = &near.stats {
            let _ = writeln!(
                out,
                "> Near-dup stats: fingerprinting {}ms, pairing {}ms, {} bytes processed\n",
                stats.fingerprinting_ms, stats.pairing_ms, stats.bytes_processed
            );
        }
    }
}

fn render_complexity_section(out: &mut String, cx: &tokmd_analysis_types::ComplexityReport) {
    out.push_str("## Complexity\n\n");
    out.push_str("|Metric|Value|\n");
    out.push_str("|---|---:|\n");
    let _ = writeln!(out, "|Total functions|{}|", cx.total_functions);
    let _ = writeln!(out, "|Avg function length|{}|", fmt_f64(cx.avg_function_length, 1));
    let _ = writeln!(out, "|Max function length|{}|", cx.max_function_length);
    let _ = writeln!(out, "|Avg cyclomatic|{}|", fmt_f64(cx.avg_cyclomatic, 2));
    let _ = writeln!(out, "|Max cyclomatic|{}|", cx.max_cyclomatic);
    if let Some(cog) = cx.avg_cognitive {
        let _ = writeln!(out, "|Avg cognitive|{}|", fmt_f64(cog, 2));
    }
    if let Some(cog) = cx.max_cognitive {
        let _ = writeln!(out, "|Max cognitive|{}|", cog);
    }
    if let Some(avg_nesting) = cx.avg_nesting_depth {
        let _ = writeln!(out, "|Avg nesting depth|{}|", fmt_f64(avg_nesting, 2));
    }
    if let Some(max_nesting) = cx.max_nesting_depth {
        let _ = writeln!(out, "|Max nesting depth|{}|", max_nesting);
    }
    let _ = writeln!(out, "|High risk files|{}|\n", cx.high_risk_files);

    if !cx.files.is_empty() {
        out.push_str("### Top complex files\n\n");
        out.push_str("|Path|CC|Functions|Max fn length|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for f in cx.files.iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                f.path, f.cyclomatic_complexity, f.function_count, f.max_function_length
            );
        }
        out.push('\n');
    }
}

fn render_api_section(out: &mut String, api: &tokmd_analysis_types::ApiSurfaceReport) {
    out.push_str("## API surface\n\n");
    out.push_str("|Metric|Value|\n");
    out.push_str("|---|---:|\n");
    let _ = writeln!(out, "|Total items|{}|", api.total_items);
    let _ = writeln!(out, "|Public items|{}|", api.public_items);
    let _ = writeln!(out, "|Internal items|{}|", api.internal_items);
    let _ = writeln!(out, "|Public ratio|{}|", fmt_pct(api.public_ratio));
    let _ = writeln!(out, "|Documented ratio|{}|\n", fmt_pct(api.documented_ratio));

    if !api.by_language.is_empty() {
        out.push_str("### By language\n\n");
        out.push_str("|Language|Total|Public|Internal|Public%|\n");
        out.push_str("|---|---:|---:|---:|---:|\n");
        for (lang, data) in &api.by_language {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|{}|",
                lang,
                data.total_items,
                data.public_items,
                data.internal_items,
                fmt_pct(data.public_ratio)
            );
        }
        out.push('\n');
    }

    if !api.by_module.is_empty() {
        out.push_str("### By module\n\n");
        out.push_str("|Module|Total|Public|Public%|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for row in api.by_module.iter().take(20) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.module,
                row.total_items,
                row.public_items,
                fmt_pct(row.public_ratio)
            );
        }
        out.push('\n');
    }

    if !api.top_exporters.is_empty() {
        out.push_str("### Top exporters\n\n");
        out.push_str("|Path|Language|Public|Total|\n");
        out.push_str("|---|---|---:|---:|\n");
        for item in api.top_exporters.iter().take(10) {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                item.path, item.lang, item.public_items, item.total_items
            );
        }
        out.push('\n');
    }
}

fn render_file_table(rows: &[FileStatRow]) -> String {
    let mut out = String::with_capacity((rows.len() + 3) * 80);
    out.push_str("|Path|Lang|Lines|Code|Bytes|Tokens|Doc%|B/Line|\n");
    out.push_str("|---|---|---:|---:|---:|---:|---:|---:|\n");
    for row in rows {
        let _ = writeln!(
            out,
            "|{}|{}|{}|{}|{}|{}|{}|{}|",
            row.path,
            row.lang,
            row.lines,
            row.code,
            row.bytes,
            row.tokens,
            row.doc_pct.map(fmt_pct).unwrap_or_else(|| "-".to_string()),
            row.bytes_per_line
                .map(|v| fmt_f64(v, 2))
                .unwrap_or_else(|| "-".to_string())
        );
    }
    out
}

fn render_effort_report(out: &mut String, effort: &EffortEstimateReport) {
    out.push_str("## Effort estimate\n\n");

    out.push_str("### Size basis\n\n");
    let _ = writeln!(
        out,
        "- Model: `{}`\n- Total LOC lines: `{}`\n- Authored LOC lines: `{}`\n- Generated LOC lines: `{}`\n- Vendored LOC lines: `{}`\n- Authoring KLOC: `{}`\n- Total KLOC: `{}`\n- Generated share: `{}`\n- Vendored share: `{}`\n- Classification confidence: `{}`\n",
        effort.model,
        effort.size_basis.total_lines,
        effort.size_basis.authored_lines,
        effort.size_basis.generated_lines,
        effort.size_basis.vendored_lines,
        fmt_f64(effort.size_basis.kloc_authored, 4),
        fmt_f64(effort.size_basis.kloc_total, 4),
        fmt_pct(effort.size_basis.generated_pct),
        fmt_pct(effort.size_basis.vendored_pct),
        effort.size_basis.classification_confidence
    );

    if !effort.size_basis.by_tag.is_empty() {
        out.push_str("### Size by tag\n\n");
        out.push_str("|Tag|Lines|Authored|Share|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for row in &effort.size_basis.by_tag {
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.tag,
                row.lines,
                row.authored_lines,
                fmt_pct(row.pct_of_total)
            );
        }
        out.push('\n');
    }

    out.push_str("### Headline\n\n");
    let _ = writeln!(
        out,
        "- Effort p50: `{}` person-months (low `{}` / p80 `{}`)\n- Schedule p50: `{}` months (low `{}` / p80 `{}`)\n- Staff p50: `{}` FTE (low `{}` / p80 `{}`)\n",
        fmt_f64(effort.results.effort_pm_p50, 4),
        fmt_f64(effort.results.effort_pm_low, 4),
        fmt_f64(effort.results.effort_pm_p80, 4),
        fmt_f64(effort.results.schedule_months_p50, 4),
        fmt_f64(effort.results.schedule_months_low, 4),
        fmt_f64(effort.results.schedule_months_p80, 4),
        fmt_f64(effort.results.staff_p50, 4),
        fmt_f64(effort.results.staff_low, 4),
        fmt_f64(effort.results.staff_p80, 4),
    );

    out.push_str("### Why\n\n");
    let _ = writeln!(out, "- Confidence level: `{}`", effort.confidence.level);
    if let Some(coverage) = effort.confidence.data_coverage_pct {
        let _ = writeln!(out, "- Data coverage: `{}`", fmt_pct(coverage));
    }
    if !effort.confidence.reasons.is_empty() {
        out.push_str("- Reasons:\n");
        for reason in &effort.confidence.reasons {
            let _ = writeln!(out, "  - {reason}");
        }
    }
    out.push('\n');

    out.push_str("### Drivers\n\n");
    if effort.drivers.is_empty() {
        out.push_str("- No material drivers were inferred.\n\n");
    } else {
        out.push_str("|Driver|Direction|Weight|Evidence|\n");
        out.push_str("|---|---|---:|---|\n");
        for row in effort.drivers.iter().take(35) {
            let direction = match row.direction {
                EffortDriverDirection::Raises => "raises",
                EffortDriverDirection::Lowers => "lowers",
                EffortDriverDirection::Neutral => "neutral",
            };
            let _ = writeln!(
                out,
                "|{}|{}|{}|{}|",
                row.label,
                direction,
                fmt_f64(row.weight, 4),
                row.evidence
            );
        }
        out.push('\n');
    }

    if !effort.assumptions.notes.is_empty() {
        out.push_str("### Assumptions\n\n");
        for note in &effort.assumptions.notes {
            let _ = writeln!(out, "- {note}");
        }
        out.push('\n');
    }

    if !effort.assumptions.overrides.is_empty() {
        out.push_str("### Assumption overrides\n\n");
        out.push_str("|Setting|Value|\n");
        out.push_str("|---|---|\n");
        for (key, value) in &effort.assumptions.overrides {
            let _ = writeln!(out, "|{key}|{value}|");
        }
        out.push('\n');
    }

    out.push_str("### Delta\n\n");
    if let Some(delta) = &effort.delta {
        let _ = writeln!(
            out,
            "- Reference window: `{}`..`{}`\n- Files changed: `{}`\n- Modules changed: `{}`\n- Languages changed: `{}`\n- Hotspots touched: `{}`\n- Coupled neighbors touched: `{}`\n- Blast radius: `{}`\n- Classification: `{}`\n- Effort p50 impact: `{}`\n- Effort p80 impact: `{}`\n",
            delta.base,
            delta.head,
            delta.files_changed,
            delta.modules_changed,
            delta.langs_changed,
            delta.hotspot_files_touched,
            delta.coupled_neighbors_touched,
            fmt_f64(delta.blast_radius, 4),
            delta.classification,
            fmt_f64(delta.effort_pm_est, 4),
            fmt_f64(delta.effort_pm_high, 4)
        );
        let _ = writeln!(out, "- Effort low bound (delta): `{}`\n", fmt_f64(delta.effort_pm_low, 4));
    } else {
        out.push_str("- Baseline comparison is not available for this receipt.\n\n");
    }
}

fn render_legacy_cocomo_report(out: &mut String, derived: &DerivedReport, cocomo: &CocomoReport) {
    out.push_str("## Effort estimate\n\n");

    out.push_str("### Size basis\n\n");
    let _ = writeln!(
        out,
        "- Source lines: `{}`\n- Total lines: `{}`\n- KLOC: `{}`\n",
        derived.totals.code,
        derived.totals.lines,
        fmt_f64(cocomo.kloc, 4)
    );

    out.push_str("### Headline\n\n");
    let _ = writeln!(
        out,
        "- Effort: `{}` person-months\n- Duration: `{}` months\n- Staff: `{}`\n",
        fmt_f64(cocomo.effort_pm, 2),
        fmt_f64(cocomo.duration_months, 2),
        fmt_f64(cocomo.staff, 2)
    );

    out.push_str("### Why\n\n");
    let _ = writeln!(
        out,
        "- Model: `COCOMO` (`{}` mode)\n- Formula: `E = a * KLOC^b`\n- Coefficients: `a={}`, `b={}`, `c={}`, `d={}`\n",
        cocomo.mode,
        fmt_f64(cocomo.a, 2),
        fmt_f64(cocomo.b, 2),
        fmt_f64(cocomo.c, 2),
        fmt_f64(cocomo.d, 2)
    );

    out.push_str("### Delta\n\n");
    out.push_str("- Baseline comparison is not available for this receipt.\n\n");
}

fn fmt_pct(ratio: f64) -> String {
    format!("{:.1}%", ratio * 100.0)
}

fn fmt_f64(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource, AnalysisReceipt};
    use tokmd_types::ToolInfo;

    fn minimal_receipt() -> AnalysisReceipt {
        AnalysisReceipt {
            schema_version: 2,
            generated_at_ms: 0,
            tool: ToolInfo {
                name: "tokmd".to_string(),
                version: "0.0.0".to_string(),
            },
            mode: "analysis".to_string(),
            status: tokmd_types::ScanStatus::Complete,
            warnings: vec![],
            source: AnalysisSource {
                inputs: vec!["test".to_string()],
                export_path: None,
                base_receipt_path: None,
                export_schema_version: None,
                export_generated_at_ms: None,
                base_signature: None,
                module_roots: vec![],
                module_depth: 1,
                children: "collapse".to_string(),
            },
            args: AnalysisArgsMeta {
                preset: "receipt".to_string(),
                format: "md".to_string(),
                window_tokens: None,
                git: None,
                max_files: None,
                max_bytes: None,
                max_commits: None,
                max_commit_files: None,
                max_file_bytes: None,
                import_granularity: "module".to_string(),
            },
            archetype: None,
            topics: None,
            entropy: None,
            predictive_churn: None,
            corporate_fingerprint: None,
            license: None,
            derived: None,
            assets: None,
            deps: None,
            git: None,
            imports: None,
            dup: None,
            complexity: None,
            api_surface: None,
            fun: None,
            effort: None,
        }
    }

    #[test]
    fn test_render_md_minimal() {
        let receipt = minimal_receipt();
        let output = render_md(&receipt);
        assert!(output.contains("# tokmd analysis"));
        assert!(output.contains("Preset:"));
        assert!(output.contains("test"));
    }

    #[test]
    fn test_fmt_pct() {
        assert_eq!(fmt_pct(0.5), "50.0%");
        assert_eq!(fmt_pct(0.123), "12.3%");
    }

    #[test]
    fn test_fmt_f64() {
        assert_eq!(fmt_f64(3.14159, 2), "3.14");
        assert_eq!(fmt_f64(3.14159, 4), "3.1416");
    }
}
