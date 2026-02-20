//! # tokmd-analysis-format
//!
//! **Tier 3 (Formatting)**
//!
//! Rendering for analysis receipts. Supports multiple output formats including
//! Markdown, JSON, JSON-LD, XML, SVG, Mermaid, and optional fun outputs.
//!
//! ## What belongs here
//! * Analysis receipt rendering to various formats
//! * Format-specific transformations
//! * Fun output integration (OBJ, MIDI when enabled)
//!
//! ## What does NOT belong here
//! * Analysis computation (use tokmd-analysis)
//! * CLI argument parsing
//! * Base receipt formatting (use tokmd-format)

use anyhow::Result;
use time::OffsetDateTime;
use time::macros::format_description;
use tokmd_analysis_types::{AnalysisReceipt, FileStatRow};
use tokmd_config::AnalysisFormat;

pub enum RenderedOutput {
    Text(String),
    Binary(Vec<u8>),
}

pub fn render(receipt: &AnalysisReceipt, format: AnalysisFormat) -> Result<RenderedOutput> {
    match format {
        AnalysisFormat::Md => Ok(RenderedOutput::Text(render_md(receipt))),
        AnalysisFormat::Json => Ok(RenderedOutput::Text(serde_json::to_string_pretty(receipt)?)),
        AnalysisFormat::Jsonld => Ok(RenderedOutput::Text(render_jsonld(receipt))),
        AnalysisFormat::Xml => Ok(RenderedOutput::Text(render_xml(receipt))),
        AnalysisFormat::Svg => Ok(RenderedOutput::Text(render_svg(receipt))),
        AnalysisFormat::Mermaid => Ok(RenderedOutput::Text(render_mermaid(receipt))),
        AnalysisFormat::Obj => Ok(RenderedOutput::Text(render_obj(receipt)?)),
        AnalysisFormat::Midi => Ok(RenderedOutput::Binary(render_midi(receipt)?)),
        AnalysisFormat::Tree => Ok(RenderedOutput::Text(render_tree(receipt))),
        AnalysisFormat::Html => Ok(RenderedOutput::Text(render_html(receipt))),
    }
}

fn render_md(receipt: &AnalysisReceipt) -> String {
    let mut out = String::new();
    out.push_str("# tokmd analysis\n\n");
    out.push_str(&format!("Preset: `{}`\n\n", receipt.args.preset));

    if !receipt.source.inputs.is_empty() {
        out.push_str("## Inputs\n\n");
        for input in &receipt.source.inputs {
            out.push_str(&format!("- `{}`\n", input));
        }
        out.push('\n');
    }

    if let Some(archetype) = &receipt.archetype {
        out.push_str("## Archetype\n\n");
        out.push_str(&format!("- Kind: `{}`\n", archetype.kind));
        if !archetype.evidence.is_empty() {
            out.push_str(&format!(
                "- Evidence: `{}`\n",
                archetype.evidence.join("`, `")
            ));
        }
        out.push('\n');
    }

    if let Some(topics) = &receipt.topics {
        out.push_str("## Topics\n\n");
        if !topics.overall.is_empty() {
            out.push_str(&format!(
                "- Overall: `{}`\n",
                topics
                    .overall
                    .iter()
                    .map(|t| t.term.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
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
            out.push_str(&format!("- `{}`: {}\n", module, line));
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
                out.push_str(&format!(
                    "|{}|{}|{}|{}|{:?}|\n",
                    row.path,
                    row.module,
                    fmt_f64(row.entropy_bits_per_byte as f64, 2),
                    row.sample_bytes,
                    row.class
                ));
            }
            out.push('\n');
        }
    }

    if let Some(license) = &receipt.license {
        out.push_str("## License radar\n\n");
        if let Some(effective) = &license.effective {
            out.push_str(&format!("- Effective: `{}`\n", effective));
        }
        out.push_str("- Heuristic detection; not legal advice.\n\n");
        if !license.findings.is_empty() {
            out.push_str("|SPDX|Confidence|Source|Kind|\n");
            out.push_str("|---|---:|---|---|\n");
            for row in license.findings.iter().take(10) {
                out.push_str(&format!(
                    "|{}|{}|{}|{:?}|\n",
                    row.spdx,
                    fmt_f64(row.confidence as f64, 2),
                    row.source_path,
                    row.source_kind
                ));
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
                out.push_str(&format!(
                    "|{}|{}|{}|\n",
                    row.domain,
                    row.commits,
                    fmt_pct(row.pct as f64)
                ));
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
                out.push_str(&format!(
                    "|{}|{}|{}|{}|{:?}|\n",
                    module,
                    fmt_f64(trend.slope, 4),
                    fmt_f64(trend.r2, 2),
                    trend.recent_change,
                    trend.classification
                ));
            }
            out.push('\n');
        }
    }

    if let Some(derived) = &receipt.derived {
        out.push_str("## Totals\n\n");
        out.push_str("|Files|Code|Comments|Blanks|Lines|Bytes|Tokens|\n");
        out.push_str("|---:|---:|---:|---:|---:|---:|---:|\n");
        out.push_str(&format!(
            "|{}|{}|{}|{}|{}|{}|{}|\n\n",
            derived.totals.files,
            derived.totals.code,
            derived.totals.comments,
            derived.totals.blanks,
            derived.totals.lines,
            derived.totals.bytes,
            derived.totals.tokens
        ));

        out.push_str("## Ratios\n\n");
        out.push_str("|Metric|Value|\n");
        out.push_str("|---|---:|\n");
        out.push_str(&format!(
            "|Doc density|{}|\n",
            fmt_pct(derived.doc_density.total.ratio)
        ));
        out.push_str(&format!(
            "|Whitespace ratio|{}|\n",
            fmt_pct(derived.whitespace.total.ratio)
        ));
        out.push_str(&format!(
            "|Bytes per line|{}|\n\n",
            fmt_f64(derived.verbosity.total.rate, 2)
        ));

        out.push_str("### Doc density by language\n\n");
        out.push_str("|Lang|Doc%|Comments|Code|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for row in derived.doc_density.by_lang.iter().take(10) {
            out.push_str(&format!(
                "|{}|{}|{}|{}|\n",
                row.key,
                fmt_pct(row.ratio),
                row.numerator,
                row.denominator.saturating_sub(row.numerator)
            ));
        }
        out.push('\n');

        out.push_str("### Whitespace ratio by language\n\n");
        out.push_str("|Lang|Blank%|Blanks|Code+Comments|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for row in derived.whitespace.by_lang.iter().take(10) {
            out.push_str(&format!(
                "|{}|{}|{}|{}|\n",
                row.key,
                fmt_pct(row.ratio),
                row.numerator,
                row.denominator
            ));
        }
        out.push('\n');

        out.push_str("### Verbosity by language\n\n");
        out.push_str("|Lang|Bytes/Line|Bytes|Lines|\n");
        out.push_str("|---|---:|---:|---:|\n");
        for row in derived.verbosity.by_lang.iter().take(10) {
            out.push_str(&format!(
                "|{}|{}|{}|{}|\n",
                row.key,
                fmt_f64(row.rate, 2),
                row.numerator,
                row.denominator
            ));
        }
        out.push('\n');

        out.push_str("## Distribution\n\n");
        out.push_str("|Count|Min|Max|Mean|Median|P90|P99|Gini|\n");
        out.push_str("|---:|---:|---:|---:|---:|---:|---:|---:|\n");
        out.push_str(&format!(
            "|{}|{}|{}|{}|{}|{}|{}|{}|\n\n",
            derived.distribution.count,
            derived.distribution.min,
            derived.distribution.max,
            fmt_f64(derived.distribution.mean, 2),
            fmt_f64(derived.distribution.median, 2),
            fmt_f64(derived.distribution.p90, 2),
            fmt_f64(derived.distribution.p99, 2),
            fmt_f64(derived.distribution.gini, 4)
        ));

        out.push_str("## File size histogram\n\n");
        out.push_str("|Bucket|Min|Max|Files|Pct|\n");
        out.push_str("|---|---:|---:|---:|---:|\n");
        for bucket in &derived.histogram {
            let max = bucket
                .max
                .map(|v| v.to_string())
                .unwrap_or_else(|| "∞".to_string());
            out.push_str(&format!(
                "|{}|{}|{}|{}|{}|\n",
                bucket.label,
                bucket.min,
                max,
                bucket.files,
                fmt_pct(bucket.pct)
            ));
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
        out.push_str(&format!(
            "- Max depth: `{}`\n- Avg depth: `{}`\n\n",
            derived.nesting.max,
            fmt_f64(derived.nesting.avg, 2)
        ));

        out.push_str("## Test density\n\n");
        out.push_str(&format!(
            "- Test lines: `{}`\n- Prod lines: `{}`\n- Test ratio: `{}`\n\n",
            derived.test_density.test_lines,
            derived.test_density.prod_lines,
            fmt_pct(derived.test_density.ratio)
        ));

        if let Some(todo) = &derived.todo {
            out.push_str("## TODOs\n\n");
            out.push_str(&format!(
                "- Total: `{}`\n- Density (per KLOC): `{}`\n\n",
                todo.total,
                fmt_f64(todo.density_per_kloc, 2)
            ));
            out.push_str("|Tag|Count|\n");
            out.push_str("|---|---:|\n");
            for tag in &todo.tags {
                out.push_str(&format!("|{}|{}|\n", tag.tag, tag.count));
            }
            out.push('\n');
        }

        out.push_str("## Boilerplate ratio\n\n");
        out.push_str(&format!(
            "- Infra lines: `{}`\n- Logic lines: `{}`\n- Infra ratio: `{}`\n\n",
            derived.boilerplate.infra_lines,
            derived.boilerplate.logic_lines,
            fmt_pct(derived.boilerplate.ratio)
        ));

        out.push_str("## Polyglot\n\n");
        out.push_str(&format!(
            "- Languages: `{}`\n- Dominant: `{}` ({})\n- Entropy: `{}`\n\n",
            derived.polyglot.lang_count,
            derived.polyglot.dominant_lang,
            fmt_pct(derived.polyglot.dominant_pct),
            fmt_f64(derived.polyglot.entropy, 4)
        ));

        out.push_str("## Reading time\n\n");
        out.push_str(&format!(
            "- Minutes: `{}` ({} lines/min)\n\n",
            fmt_f64(derived.reading_time.minutes, 2),
            derived.reading_time.lines_per_minute
        ));

        if let Some(context) = &derived.context_window {
            out.push_str("## Context window\n\n");
            out.push_str(&format!(
                "- Window tokens: `{}`\n- Total tokens: `{}`\n- Utilization: `{}`\n- Fits: `{}`\n\n",
                context.window_tokens,
                context.total_tokens,
                fmt_pct(context.pct),
                context.fits
            ));
        }

        if let Some(cocomo) = &derived.cocomo {
            out.push_str("## COCOMO estimate\n\n");
            out.push_str(&format!(
                "- Mode: `{}`\n- KLOC: `{}`\n- Effort (PM): `{}`\n- Duration (months): `{}`\n- Staff: `{}`\n\n",
                cocomo.mode,
                fmt_f64(cocomo.kloc, 4),
                fmt_f64(cocomo.effort_pm, 2),
                fmt_f64(cocomo.duration_months, 2),
                fmt_f64(cocomo.staff, 2)
            ));
        }

        out.push_str("## Integrity\n\n");
        out.push_str(&format!(
            "- Hash: `{}` (`{}`)\n- Entries: `{}`\n\n",
            derived.integrity.hash, derived.integrity.algo, derived.integrity.entries
        ));
    }

    if let Some(assets) = &receipt.assets {
        out.push_str("## Assets\n\n");
        out.push_str(&format!(
            "- Total files: `{}`\n- Total bytes: `{}`\n\n",
            assets.total_files, assets.total_bytes
        ));
        if !assets.categories.is_empty() {
            out.push_str("|Category|Files|Bytes|Extensions|\n");
            out.push_str("|---|---:|---:|---|\n");
            for row in &assets.categories {
                out.push_str(&format!(
                    "|{}|{}|{}|{}|\n",
                    row.category,
                    row.files,
                    row.bytes,
                    row.extensions.join(", ")
                ));
            }
            out.push('\n');
        }
        if !assets.top_files.is_empty() {
            out.push_str("|File|Bytes|Category|\n");
            out.push_str("|---|---:|---|\n");
            for row in &assets.top_files {
                out.push_str(&format!("|{}|{}|{}|\n", row.path, row.bytes, row.category));
            }
            out.push('\n');
        }
    }

    if let Some(deps) = &receipt.deps {
        out.push_str("## Dependencies\n\n");
        out.push_str(&format!("- Total: `{}`\n\n", deps.total));
        if !deps.lockfiles.is_empty() {
            out.push_str("|Lockfile|Kind|Dependencies|\n");
            out.push_str("|---|---|---:|\n");
            for row in &deps.lockfiles {
                out.push_str(&format!(
                    "|{}|{}|{}|\n",
                    row.path, row.kind, row.dependencies
                ));
            }
            out.push('\n');
        }
    }

    if let Some(git) = &receipt.git {
        out.push_str("## Git metrics\n\n");
        out.push_str(&format!(
            "- Commits scanned: `{}`\n- Files seen: `{}`\n\n",
            git.commits_scanned, git.files_seen
        ));
        if !git.hotspots.is_empty() {
            out.push_str("### Hotspots\n\n");
            out.push_str("|File|Commits|Lines|Score|\n");
            out.push_str("|---|---:|---:|---:|\n");
            for row in git.hotspots.iter().take(10) {
                out.push_str(&format!(
                    "|{}|{}|{}|{}|\n",
                    row.path, row.commits, row.lines, row.score
                ));
            }
            out.push('\n');
        }
        if !git.bus_factor.is_empty() {
            out.push_str("### Bus factor\n\n");
            out.push_str("|Module|Authors|\n");
            out.push_str("|---|---:|\n");
            for row in git.bus_factor.iter().take(10) {
                out.push_str(&format!("|{}|{}|\n", row.module, row.authors));
            }
            out.push('\n');
        }
        out.push_str("### Freshness\n\n");
        out.push_str(&format!(
            "- Stale threshold (days): `{}`\n- Stale files: `{}` / `{}` ({})\n\n",
            git.freshness.threshold_days,
            git.freshness.stale_files,
            git.freshness.total_files,
            fmt_pct(git.freshness.stale_pct)
        ));
        if !git.freshness.by_module.is_empty() {
            out.push_str("|Module|Avg days|P90 days|Stale%|\n");
            out.push_str("|---|---:|---:|---:|\n");
            for row in git.freshness.by_module.iter().take(10) {
                out.push_str(&format!(
                    "|{}|{}|{}|{}|\n",
                    row.module,
                    fmt_f64(row.avg_days, 2),
                    fmt_f64(row.p90_days, 2),
                    fmt_pct(row.stale_pct)
                ));
            }
            out.push('\n');
        }
        if let Some(age) = &git.age_distribution {
            out.push_str("### Code age\n\n");
            out.push_str(&format!(
                "- Refresh trend: `{:?}` (recent: `{}`, prior: `{}`)\n\n",
                age.refresh_trend, age.recent_refreshes, age.prior_refreshes
            ));
            if !age.buckets.is_empty() {
                out.push_str("|Bucket|Min days|Max days|Files|Pct|\n");
                out.push_str("|---|---:|---:|---:|---:|\n");
                for bucket in &age.buckets {
                    let max = bucket
                        .max_days
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "∞".to_string());
                    out.push_str(&format!(
                        "|{}|{}|{}|{}|{}|\n",
                        bucket.label,
                        bucket.min_days,
                        max,
                        bucket.files,
                        fmt_pct(bucket.pct)
                    ));
                }
                out.push('\n');
            }
        }
        if !git.coupling.is_empty() {
            out.push_str("### Coupling\n\n");
            out.push_str("|Left|Right|Count|Jaccard|Lift|\n");
            out.push_str("|---|---|---:|---:|---:|\n");
            for row in git.coupling.iter().take(10) {
                let jaccard = row
                    .jaccard
                    .map(|v| fmt_f64(v, 4))
                    .unwrap_or_else(|| "-".to_string());
                let lift = row
                    .lift
                    .map(|v| fmt_f64(v, 4))
                    .unwrap_or_else(|| "-".to_string());
                out.push_str(&format!(
                    "|{}|{}|{}|{}|{}|\n",
                    row.left, row.right, row.count, jaccard, lift
                ));
            }
            out.push('\n');
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
                    out.push_str(&format!("|{}|{}|\n", name, count));
                }
            }
            out.push_str(&format!("|**total**|{}|\n", o.total));
            out.push_str(&format!(
                "\n- Unknown: `{}`\n\n",
                fmt_pct(intent.unknown_pct)
            ));

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
                    out.push_str(&format!(
                        "|{}|{}|{}|{}|\n",
                        m.module,
                        m.counts.fix + m.counts.revert,
                        m.counts.total,
                        fmt_pct(*share)
                    ));
                }
                out.push('\n');
            }
        }
    }

    if let Some(imports) = &receipt.imports {
        out.push_str("## Imports\n\n");
        out.push_str(&format!("- Granularity: `{}`\n\n", imports.granularity));
        if !imports.edges.is_empty() {
            out.push_str("|From|To|Count|\n");
            out.push_str("|---|---|---:|\n");
            for row in imports.edges.iter().take(20) {
                out.push_str(&format!("|{}|{}|{}|\n", row.from, row.to, row.count));
            }
            out.push('\n');
        }
    }

    if let Some(dup) = &receipt.dup {
        out.push_str("## Duplicates\n\n");
        out.push_str(&format!(
            "- Wasted bytes: `{}`\n- Strategy: `{}`\n\n",
            dup.wasted_bytes, dup.strategy
        ));
        if let Some(density) = &dup.density {
            out.push_str("### Duplication density\n\n");
            out.push_str(&format!(
                "- Duplicate groups: `{}`\n- Duplicate files: `{}`\n- Duplicated bytes: `{}`\n- Waste vs codebase: `{}`\n\n",
                density.duplicate_groups,
                density.duplicate_files,
                density.duplicated_bytes,
                fmt_pct(density.wasted_pct_of_codebase)
            ));
            if !density.by_module.is_empty() {
                out.push_str(
                    "|Module|Dup files|Wasted files|Dup bytes|Wasted bytes|Module bytes|Density|\n",
                );
                out.push_str("|---|---:|---:|---:|---:|---:|---:|\n");
                for row in density.by_module.iter().take(10) {
                    out.push_str(&format!(
                        "|{}|{}|{}|{}|{}|{}|{}|\n",
                        row.module,
                        row.duplicate_files,
                        row.wasted_files,
                        row.duplicated_bytes,
                        row.wasted_bytes,
                        row.module_bytes,
                        fmt_pct(row.density)
                    ));
                }
                out.push('\n');
            }
        }
        if !dup.groups.is_empty() {
            out.push_str("|Hash|Bytes|Files|\n");
            out.push_str("|---|---:|---:|\n");
            for row in dup.groups.iter().take(10) {
                out.push_str(&format!(
                    "|{}|{}|{}|\n",
                    row.hash,
                    row.bytes,
                    row.files.len()
                ));
            }
            out.push('\n');
        }

        if let Some(near) = &dup.near {
            out.push_str("### Near duplicates\n\n");
            out.push_str(&format!(
                "- Files analyzed: `{}`\n- Files skipped: `{}`\n- Threshold: `{}`\n- Scope: `{:?}`\n\n",
                near.files_analyzed,
                near.files_skipped,
                fmt_f64(near.params.threshold, 2),
                near.params.scope
            ));
            if near.pairs.is_empty() {
                out.push_str("- No near-duplicate pairs detected.\n\n");
            } else {
                out.push_str("|Left|Right|Similarity|Shared FPs|\n");
                out.push_str("|---|---|---:|---:|\n");
                for pair in near.pairs.iter().take(20) {
                    out.push_str(&format!(
                        "|{}|{}|{}|{}|\n",
                        pair.left,
                        pair.right,
                        fmt_pct(pair.similarity),
                        pair.shared_fingerprints
                    ));
                }
                out.push('\n');
            }
        }
    }

    if let Some(cx) = &receipt.complexity {
        out.push_str("## Complexity\n\n");
        out.push_str("|Metric|Value|\n");
        out.push_str("|---|---:|\n");
        out.push_str(&format!("|Total functions|{}|\n", cx.total_functions));
        out.push_str(&format!(
            "|Avg function length|{}|\n",
            fmt_f64(cx.avg_function_length, 1)
        ));
        out.push_str(&format!(
            "|Max function length|{}|\n",
            cx.max_function_length
        ));
        out.push_str(&format!(
            "|Avg cyclomatic|{}|\n",
            fmt_f64(cx.avg_cyclomatic, 2)
        ));
        out.push_str(&format!("|Max cyclomatic|{}|\n", cx.max_cyclomatic));
        if let Some(cog) = cx.avg_cognitive {
            out.push_str(&format!("|Avg cognitive|{}|\n", fmt_f64(cog, 2)));
        }
        if let Some(cog) = cx.max_cognitive {
            out.push_str(&format!("|Max cognitive|{}|\n", cog));
        }
        if let Some(avg_nesting) = cx.avg_nesting_depth {
            out.push_str(&format!(
                "|Avg nesting depth|{}|\n",
                fmt_f64(avg_nesting, 2)
            ));
        }
        if let Some(max_nesting) = cx.max_nesting_depth {
            out.push_str(&format!("|Max nesting depth|{}|\n", max_nesting));
        }
        out.push_str(&format!("|High risk files|{}|\n\n", cx.high_risk_files));

        if !cx.files.is_empty() {
            out.push_str("### Top complex files\n\n");
            out.push_str("|Path|CC|Functions|Max fn length|\n");
            out.push_str("|---|---:|---:|---:|\n");
            for f in cx.files.iter().take(10) {
                out.push_str(&format!(
                    "|{}|{}|{}|{}|\n",
                    f.path, f.cyclomatic_complexity, f.function_count, f.max_function_length
                ));
            }
            out.push('\n');
        }
    }

    if let Some(api) = &receipt.api_surface {
        out.push_str("## API surface\n\n");
        out.push_str("|Metric|Value|\n");
        out.push_str("|---|---:|\n");
        out.push_str(&format!("|Total items|{}|\n", api.total_items));
        out.push_str(&format!("|Public items|{}|\n", api.public_items));
        out.push_str(&format!("|Internal items|{}|\n", api.internal_items));
        out.push_str(&format!("|Public ratio|{}|\n", fmt_pct(api.public_ratio)));
        out.push_str(&format!(
            "|Documented ratio|{}|\n\n",
            fmt_pct(api.documented_ratio)
        ));

        if !api.by_language.is_empty() {
            out.push_str("### By language\n\n");
            out.push_str("|Language|Total|Public|Internal|Public%|\n");
            out.push_str("|---|---:|---:|---:|---:|\n");
            for (lang, data) in &api.by_language {
                out.push_str(&format!(
                    "|{}|{}|{}|{}|{}|\n",
                    lang,
                    data.total_items,
                    data.public_items,
                    data.internal_items,
                    fmt_pct(data.public_ratio)
                ));
            }
            out.push('\n');
        }

        if !api.by_module.is_empty() {
            out.push_str("### By module\n\n");
            out.push_str("|Module|Total|Public|Public%|\n");
            out.push_str("|---|---:|---:|---:|\n");
            for row in api.by_module.iter().take(20) {
                out.push_str(&format!(
                    "|{}|{}|{}|{}|\n",
                    row.module,
                    row.total_items,
                    row.public_items,
                    fmt_pct(row.public_ratio)
                ));
            }
            out.push('\n');
        }

        if !api.top_exporters.is_empty() {
            out.push_str("### Top exporters\n\n");
            out.push_str("|Path|Language|Public|Total|\n");
            out.push_str("|---|---|---:|---:|\n");
            for item in api.top_exporters.iter().take(10) {
                out.push_str(&format!(
                    "|{}|{}|{}|{}|\n",
                    item.path, item.lang, item.public_items, item.total_items
                ));
            }
            out.push('\n');
        }
    }

    if let Some(fun) = &receipt.fun
        && let Some(label) = &fun.eco_label
    {
        out.push_str("## Eco label\n\n");
        out.push_str(&format!(
            "- Label: `{}`\n- Score: `{}`\n- Bytes: `{}`\n- Notes: `{}`\n\n",
            label.label,
            fmt_f64(label.score, 1),
            label.bytes,
            label.notes
        ));
    }

    out
}

fn render_file_table(rows: &[FileStatRow]) -> String {
    let mut out = String::new();
    out.push_str("|Path|Lang|Lines|Code|Bytes|Tokens|Doc%|B/Line|\n");
    out.push_str("|---|---|---:|---:|---:|---:|---:|---:|\n");
    for row in rows {
        out.push_str(&format!(
            "|{}|{}|{}|{}|{}|{}|{}|{}|\n",
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
        ));
    }
    out
}

fn fmt_pct(ratio: f64) -> String {
    format!("{:.1}%", ratio * 100.0)
}

fn fmt_f64(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

fn render_jsonld(receipt: &AnalysisReceipt) -> String {
    let name = receipt
        .source
        .inputs
        .first()
        .cloned()
        .unwrap_or_else(|| "tokmd".to_string());
    let totals = receipt.derived.as_ref().map(|d| &d.totals);
    let payload = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "SoftwareSourceCode",
        "name": name,
        "codeLines": totals.map(|t| t.code).unwrap_or(0),
        "commentCount": totals.map(|t| t.comments).unwrap_or(0),
        "lineCount": totals.map(|t| t.lines).unwrap_or(0),
        "fileSize": totals.map(|t| t.bytes).unwrap_or(0),
        "interactionStatistic": {
            "@type": "InteractionCounter",
            "interactionType": "http://schema.org/ReadAction",
            "userInteractionCount": totals.map(|t| t.tokens).unwrap_or(0)
        }
    });
    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn render_xml(receipt: &AnalysisReceipt) -> String {
    let totals = receipt.derived.as_ref().map(|d| &d.totals);
    let mut out = String::new();
    out.push_str("<analysis>");
    if let Some(totals) = totals {
        out.push_str(&format!(
            "<totals files=\"{}\" code=\"{}\" comments=\"{}\" blanks=\"{}\" lines=\"{}\" bytes=\"{}\" tokens=\"{}\"/>",
            totals.files,
            totals.code,
            totals.comments,
            totals.blanks,
            totals.lines,
            totals.bytes,
            totals.tokens
        ));
    }
    out.push_str("</analysis>");
    out
}

fn render_svg(receipt: &AnalysisReceipt) -> String {
    let (label, value) = if let Some(derived) = &receipt.derived {
        if let Some(ctx) = &derived.context_window {
            ("context".to_string(), format!("{:.1}%", ctx.pct * 100.0))
        } else {
            ("tokens".to_string(), derived.totals.tokens.to_string())
        }
    } else {
        ("tokens".to_string(), "0".to_string())
    };

    let width = 240;
    let height = 32;
    let label_width = 80;
    let value_width = width - label_width;
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" role=\"img\"><rect width=\"{label_width}\" height=\"{height}\" fill=\"#555\"/><rect x=\"{label_width}\" width=\"{value_width}\" height=\"{height}\" fill=\"#4c9aff\"/><text x=\"{lx}\" y=\"{ty}\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"12\" text-anchor=\"middle\">{label}</text><text x=\"{vx}\" y=\"{ty}\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"12\" text-anchor=\"middle\">{value}</text></svg>",
        width = width,
        height = height,
        label_width = label_width,
        value_width = value_width,
        lx = label_width / 2,
        vx = label_width + value_width / 2,
        ty = 20,
        label = label,
        value = value
    )
}

fn render_mermaid(receipt: &AnalysisReceipt) -> String {
    let mut out = String::from("graph TD\n");
    if let Some(imports) = &receipt.imports {
        for edge in imports.edges.iter().take(200) {
            let from = sanitize_mermaid(&edge.from);
            let to = sanitize_mermaid(&edge.to);
            out.push_str(&format!("  {} -->|{}| {}\n", from, edge.count, to));
        }
    }
    out
}

fn render_tree(receipt: &AnalysisReceipt) -> String {
    receipt
        .derived
        .as_ref()
        .and_then(|d| d.tree.clone())
        .unwrap_or_else(|| "(tree unavailable)".to_string())
}

// --- fun enabled impls ---
#[cfg(feature = "fun")]
fn render_obj_fun(receipt: &AnalysisReceipt) -> Result<String> {
    if let Some(derived) = &receipt.derived {
        let buildings: Vec<tokmd_fun::ObjBuilding> = derived
            .top
            .largest_lines
            .iter()
            .enumerate()
            .map(|(idx, row)| {
                let x = (idx % 5) as f32 * 2.0;
                let y = (idx / 5) as f32 * 2.0;
                let h = (row.lines as f32 / 10.0).max(0.5);
                tokmd_fun::ObjBuilding {
                    name: row.path.clone(),
                    x,
                    y,
                    w: 1.5,
                    d: 1.5,
                    h,
                }
            })
            .collect();
        return Ok(tokmd_fun::render_obj(&buildings));
    }
    Ok("# tokmd code city\n".to_string())
}

#[cfg(feature = "fun")]
fn render_midi_fun(receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
    let mut notes = Vec::new();
    if let Some(derived) = &receipt.derived {
        for (idx, row) in derived.top.largest_lines.iter().enumerate() {
            let key = 60u8 + (row.depth as u8 % 12);
            let velocity = (40 + (row.lines.min(127) as u8 / 2)).min(120);
            let start = (idx as u32) * 240;
            notes.push(tokmd_fun::MidiNote {
                key,
                velocity,
                start,
                duration: 180,
                channel: 0,
            });
        }
    }
    tokmd_fun::render_midi(&notes, 120)
}

// --- fun disabled impls (errors) ---
#[cfg(not(feature = "fun"))]
fn render_obj_disabled(_receipt: &AnalysisReceipt) -> Result<String> {
    anyhow::bail!(
        "OBJ format requires the `fun` feature: tokmd-analysis-format = {{ version = \"1.3\", features = [\"fun\"] }}"
    )
}

#[cfg(not(feature = "fun"))]
fn render_midi_disabled(_receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
    anyhow::bail!(
        "MIDI format requires the `fun` feature: tokmd-analysis-format = {{ version = \"1.3\", features = [\"fun\"] }}"
    )
}

// --- stable API names used by the rest of the code ---
fn render_obj(receipt: &AnalysisReceipt) -> Result<String> {
    #[cfg(feature = "fun")]
    {
        render_obj_fun(receipt)
    }
    #[cfg(not(feature = "fun"))]
    {
        render_obj_disabled(receipt)
    }
}

fn render_midi(receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
    #[cfg(feature = "fun")]
    {
        render_midi_fun(receipt)
    }
    #[cfg(not(feature = "fun"))]
    {
        render_midi_disabled(receipt)
    }
}

fn sanitize_mermaid(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn render_html(receipt: &AnalysisReceipt) -> String {
    const TEMPLATE: &str = include_str!("templates/report.html");

    // Generate timestamp
    let timestamp = chrono_lite_timestamp();

    // Build metrics cards
    let metrics_cards = build_metrics_cards(receipt);

    // Build table rows
    let table_rows = build_table_rows(receipt);

    // Build JSON data for treemap
    let report_json = build_report_json(receipt);

    TEMPLATE
        .replace("{{TIMESTAMP}}", &timestamp)
        .replace("{{METRICS_CARDS}}", &metrics_cards)
        .replace("{{TABLE_ROWS}}", &table_rows)
        .replace("{{REPORT_JSON}}", &report_json)
}

fn chrono_lite_timestamp() -> String {
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second] UTC");
    OffsetDateTime::now_utc()
        .format(&format)
        .unwrap_or_else(|_| "1970-01-01 00:00:00 UTC".to_string())
}

fn build_metrics_cards(receipt: &AnalysisReceipt) -> String {
    let mut cards = String::new();

    if let Some(derived) = &receipt.derived {
        let metrics = [
            ("Files", derived.totals.files.to_string()),
            ("Lines", format_number(derived.totals.lines)),
            ("Code", format_number(derived.totals.code)),
            ("Tokens", format_number(derived.totals.tokens)),
            ("Doc%", fmt_pct(derived.doc_density.total.ratio)),
        ];

        for (label, value) in metrics {
            cards.push_str(&format!(
                r#"<div class="metric-card"><span class="value">{}</span><span class="label">{}</span></div>"#,
                value, label
            ));
        }

        // Context fit if available
        if let Some(ctx) = &derived.context_window {
            cards.push_str(&format!(
                r#"<div class="metric-card"><span class="value">{}</span><span class="label">Context Fit</span></div>"#,
                fmt_pct(ctx.pct)
            ));
        }
    }

    cards
}

fn build_table_rows(receipt: &AnalysisReceipt) -> String {
    let mut rows = String::new();

    if let Some(derived) = &receipt.derived {
        // Use top files from the analysis
        for row in derived.top.largest_lines.iter().take(100) {
            rows.push_str(&format!(
                r#"<tr><td class="path" data-path="{path}">{path}</td><td data-module="{module}">{module}</td><td data-lang="{lang}"><span class="lang-badge">{lang}</span></td><td class="num" data-lines="{lines}">{lines_fmt}</td><td class="num" data-code="{code}">{code_fmt}</td><td class="num" data-tokens="{tokens}">{tokens_fmt}</td><td class="num" data-bytes="{bytes}">{bytes_fmt}</td></tr>"#,
                path = html_escape(&row.path),
                module = html_escape(&row.module),
                lang = html_escape(&row.lang),
                lines = row.lines,
                lines_fmt = format_number(row.lines),
                code = row.code,
                code_fmt = format_number(row.code),
                tokens = row.tokens,
                tokens_fmt = format_number(row.tokens),
                bytes = row.bytes,
                bytes_fmt = format_number(row.bytes),
            ));
        }
    }

    rows
}

fn build_report_json(receipt: &AnalysisReceipt) -> String {
    // Build a simplified JSON for the treemap
    let mut files = Vec::new();

    if let Some(derived) = &receipt.derived {
        for row in &derived.top.largest_lines {
            files.push(serde_json::json!({
                "path": row.path,
                "module": row.module,
                "lang": row.lang,
                "code": row.code,
                "lines": row.lines,
                "tokens": row.tokens,
            }));
        }
    }

    // Escape < and > to prevent </script> breakout XSS attacks.
    // JSON remains valid because \u003c and \u003e are valid JSON string escapes.
    serde_json::json!({ "files": files })
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_analysis_types::*;

    fn minimal_receipt() -> AnalysisReceipt {
        AnalysisReceipt {
            schema_version: 2,
            generated_at_ms: 0,
            tool: tokmd_types::ToolInfo {
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
        }
    }

    fn sample_derived() -> DerivedReport {
        DerivedReport {
            totals: DerivedTotals {
                files: 10,
                code: 1000,
                comments: 200,
                blanks: 100,
                lines: 1300,
                bytes: 50000,
                tokens: 2500,
            },
            doc_density: RatioReport {
                total: RatioRow {
                    key: "total".to_string(),
                    numerator: 200,
                    denominator: 1200,
                    ratio: 0.1667,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            whitespace: RatioReport {
                total: RatioRow {
                    key: "total".to_string(),
                    numerator: 100,
                    denominator: 1300,
                    ratio: 0.0769,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            verbosity: RateReport {
                total: RateRow {
                    key: "total".to_string(),
                    numerator: 50000,
                    denominator: 1300,
                    rate: 38.46,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            max_file: MaxFileReport {
                overall: FileStatRow {
                    path: "src/lib.rs".to_string(),
                    module: "src".to_string(),
                    lang: "Rust".to_string(),
                    code: 500,
                    comments: 100,
                    blanks: 50,
                    lines: 650,
                    bytes: 25000,
                    tokens: 1250,
                    doc_pct: Some(0.167),
                    bytes_per_line: Some(38.46),
                    depth: 1,
                },
                by_lang: vec![],
                by_module: vec![],
            },
            lang_purity: LangPurityReport { rows: vec![] },
            nesting: NestingReport {
                max: 3,
                avg: 1.5,
                by_module: vec![],
            },
            test_density: TestDensityReport {
                test_lines: 200,
                prod_lines: 1000,
                test_files: 5,
                prod_files: 5,
                ratio: 0.2,
            },
            boilerplate: BoilerplateReport {
                infra_lines: 100,
                logic_lines: 1100,
                ratio: 0.083,
                infra_langs: vec!["TOML".to_string()],
            },
            polyglot: PolyglotReport {
                lang_count: 2,
                entropy: 0.5,
                dominant_lang: "Rust".to_string(),
                dominant_lines: 1000,
                dominant_pct: 0.833,
            },
            distribution: DistributionReport {
                count: 10,
                min: 50,
                max: 650,
                mean: 130.0,
                median: 100.0,
                p90: 400.0,
                p99: 650.0,
                gini: 0.3,
            },
            histogram: vec![HistogramBucket {
                label: "Small".to_string(),
                min: 0,
                max: Some(100),
                files: 5,
                pct: 0.5,
            }],
            top: TopOffenders {
                largest_lines: vec![FileStatRow {
                    path: "src/lib.rs".to_string(),
                    module: "src".to_string(),
                    lang: "Rust".to_string(),
                    code: 500,
                    comments: 100,
                    blanks: 50,
                    lines: 650,
                    bytes: 25000,
                    tokens: 1250,
                    doc_pct: Some(0.167),
                    bytes_per_line: Some(38.46),
                    depth: 1,
                }],
                largest_tokens: vec![],
                largest_bytes: vec![],
                least_documented: vec![],
                most_dense: vec![],
            },
            tree: Some("test-tree".to_string()),
            reading_time: ReadingTimeReport {
                minutes: 65.0,
                lines_per_minute: 20,
                basis_lines: 1300,
            },
            context_window: Some(ContextWindowReport {
                window_tokens: 100000,
                total_tokens: 2500,
                pct: 0.025,
                fits: true,
            }),
            cocomo: Some(CocomoReport {
                mode: "organic".to_string(),
                kloc: 1.0,
                effort_pm: 2.4,
                duration_months: 2.5,
                staff: 1.0,
                a: 2.4,
                b: 1.05,
                c: 2.5,
                d: 0.38,
            }),
            todo: Some(TodoReport {
                total: 5,
                density_per_kloc: 5.0,
                tags: vec![TodoTagRow {
                    tag: "TODO".to_string(),
                    count: 5,
                }],
            }),
            integrity: IntegrityReport {
                algo: "blake3".to_string(),
                hash: "abc123".to_string(),
                entries: 10,
            },
        }
    }

    // Test fmt_pct
    #[test]
    fn test_fmt_pct() {
        assert_eq!(fmt_pct(0.5), "50.0%");
        assert_eq!(fmt_pct(0.0), "0.0%");
        assert_eq!(fmt_pct(1.0), "100.0%");
        assert_eq!(fmt_pct(0.1234), "12.3%");
    }

    // Test fmt_f64
    #[test]
    #[allow(clippy::approx_constant)]
    fn test_fmt_f64() {
        assert_eq!(fmt_f64(3.14159, 2), "3.14");
        assert_eq!(fmt_f64(3.14159, 4), "3.1416");
        assert_eq!(fmt_f64(0.0, 2), "0.00");
        assert_eq!(fmt_f64(100.0, 0), "100");
    }

    // Test format_number
    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1000), "1.0K");
        assert_eq!(format_number(1500), "1.5K");
        assert_eq!(format_number(1000000), "1.0M");
        assert_eq!(format_number(2500000), "2.5M");
        // Edge cases for comparison operators
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(999999), "1000.0K");
    }

    // Test html_escape
    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("hello"), "hello");
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(html_escape("it's"), "it&#x27;s");
        // All special characters together
        assert_eq!(
            html_escape("<a href=\"test\">&'"),
            "&lt;a href=&quot;test&quot;&gt;&amp;&#x27;"
        );
    }

    // Test sanitize_mermaid
    #[test]
    fn test_sanitize_mermaid() {
        assert_eq!(sanitize_mermaid("hello"), "hello");
        assert_eq!(sanitize_mermaid("hello-world"), "hello_world");
        assert_eq!(sanitize_mermaid("src/lib.rs"), "src_lib_rs");
        assert_eq!(sanitize_mermaid("test123"), "test123");
        assert_eq!(sanitize_mermaid("a b c"), "a_b_c");
    }

    // Test render_file_table
    #[test]
    fn test_render_file_table() {
        let rows = vec![FileStatRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            code: 100,
            comments: 20,
            blanks: 10,
            lines: 130,
            bytes: 5000,
            tokens: 250,
            doc_pct: Some(0.167),
            bytes_per_line: Some(38.46),
            depth: 1,
        }];
        let result = render_file_table(&rows);
        assert!(result.contains("|Path|Lang|Lines|Code|Bytes|Tokens|Doc%|B/Line|"));
        assert!(result.contains("|src/lib.rs|Rust|130|100|5000|250|16.7%|38.46|"));
    }

    // Test render_file_table with None values
    #[test]
    fn test_render_file_table_none_values() {
        let rows = vec![FileStatRow {
            path: "test.txt".to_string(),
            module: "root".to_string(),
            lang: "Text".to_string(),
            code: 50,
            comments: 0,
            blanks: 5,
            lines: 55,
            bytes: 1000,
            tokens: 100,
            doc_pct: None,
            bytes_per_line: None,
            depth: 0,
        }];
        let result = render_file_table(&rows);
        assert!(result.contains("|-|-|")); // Should have dashes for None values
    }

    // Test render_xml
    #[test]
    fn test_render_xml() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_xml(&receipt);
        assert!(result.starts_with("<analysis>"));
        assert!(result.ends_with("</analysis>"));
        assert!(result.contains("files=\"10\""));
        assert!(result.contains("code=\"1000\""));
    }

    // Test render_xml without derived
    #[test]
    fn test_render_xml_no_derived() {
        let receipt = minimal_receipt();
        let result = render_xml(&receipt);
        assert_eq!(result, "<analysis></analysis>");
    }

    // Test render_jsonld
    #[test]
    fn test_render_jsonld() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_jsonld(&receipt);
        assert!(result.contains("\"@context\": \"https://schema.org\""));
        assert!(result.contains("\"@type\": \"SoftwareSourceCode\""));
        assert!(result.contains("\"name\": \"test\""));
        assert!(result.contains("\"codeLines\": 1000"));
    }

    // Test render_jsonld without inputs
    #[test]
    fn test_render_jsonld_empty_inputs() {
        let mut receipt = minimal_receipt();
        receipt.source.inputs.clear();
        let result = render_jsonld(&receipt);
        assert!(result.contains("\"name\": \"tokmd\""));
    }

    // Test render_svg
    #[test]
    fn test_render_svg() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_svg(&receipt);
        assert!(result.contains("<svg"));
        assert!(result.contains("</svg>"));
        assert!(result.contains("context")); // has context_window
        assert!(result.contains("2.5%")); // pct value
    }

    // Test render_svg without context_window
    #[test]
    fn test_render_svg_no_context() {
        let mut receipt = minimal_receipt();
        let mut derived = sample_derived();
        derived.context_window = None;
        receipt.derived = Some(derived);
        let result = render_svg(&receipt);
        assert!(result.contains("tokens"));
        assert!(result.contains("2500")); // total tokens
    }

    // Test render_svg without derived
    #[test]
    fn test_render_svg_no_derived() {
        let receipt = minimal_receipt();
        let result = render_svg(&receipt);
        assert!(result.contains("tokens"));
        assert!(result.contains(">0<")); // default 0 value
    }

    // Test render_svg arithmetic (width - label_width = value_width)
    #[test]
    fn test_render_svg_dimensions() {
        let receipt = minimal_receipt();
        let result = render_svg(&receipt);
        // width=240, label_width=80, value_width should be 160
        assert!(result.contains("width=\"160\"")); // value_width = 240 - 80
    }

    // Test render_mermaid
    #[test]
    fn test_render_mermaid() {
        let mut receipt = minimal_receipt();
        receipt.imports = Some(ImportReport {
            granularity: "module".to_string(),
            edges: vec![ImportEdge {
                from: "src/main".to_string(),
                to: "src/lib".to_string(),
                count: 5,
            }],
        });
        let result = render_mermaid(&receipt);
        assert!(result.starts_with("graph TD\n"));
        assert!(result.contains("src_main -->|5| src_lib"));
    }

    // Test render_mermaid no imports
    #[test]
    fn test_render_mermaid_no_imports() {
        let receipt = minimal_receipt();
        let result = render_mermaid(&receipt);
        assert_eq!(result, "graph TD\n");
    }

    // Test render_tree
    #[test]
    fn test_render_tree() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_tree(&receipt);
        assert_eq!(result, "test-tree");
    }

    // Test render_tree without derived
    #[test]
    fn test_render_tree_no_derived() {
        let receipt = minimal_receipt();
        let result = render_tree(&receipt);
        assert_eq!(result, "(tree unavailable)");
    }

    // Test render_tree with no tree in derived
    #[test]
    fn test_render_tree_none() {
        let mut receipt = minimal_receipt();
        let mut derived = sample_derived();
        derived.tree = None;
        receipt.derived = Some(derived);
        let result = render_tree(&receipt);
        assert_eq!(result, "(tree unavailable)");
    }

    // Test render_obj (non-fun feature) returns error
    #[cfg(not(feature = "fun"))]
    #[test]
    fn test_render_obj_no_fun() {
        let receipt = minimal_receipt();
        let result = render_obj(&receipt);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fun"));
    }

    // Test render_midi (non-fun feature) returns error
    #[cfg(not(feature = "fun"))]
    #[test]
    fn test_render_midi_no_fun() {
        let receipt = minimal_receipt();
        let result = render_midi(&receipt);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fun"));
    }

    // Test render_obj with fun feature - verify coordinate calculations
    // This test uses precise vertex extraction to catch arithmetic mutants:
    // - idx % 5 vs idx / 5 (grid position)
    // - * 2.0 multiplier
    // - lines / 10.0 for height
    // - .max(0.5) clamping
    #[cfg(feature = "fun")]
    #[test]
    fn test_render_obj_coordinate_math() {
        let mut receipt = minimal_receipt();
        let mut derived = sample_derived();
        // Build test data with specific indices and line counts to verify:
        // x = (idx % 5) * 2.0
        // y = (idx / 5) * 2.0
        // h = (lines / 10.0).max(0.5)
        //
        // idx=0: x=0*2=0, y=0*2=0
        // idx=4: x=4*2=8, y=0*2=0 (tests % 5 at boundary)
        // idx=5: x=0*2=0, y=1*2=2 (tests % 5 wrap and / 5 increment)
        // idx=6: x=1*2=2, y=1*2=2
        derived.top.largest_lines = vec![
            FileStatRow {
                path: "file0.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 100, // h = 100/10 = 10.0
                bytes: 1000,
                tokens: 200,
                doc_pct: None,
                bytes_per_line: None,
                depth: 1,
            },
            FileStatRow {
                path: "file1.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 50,
                comments: 5,
                blanks: 2,
                lines: 3, // h = 3/10 = 0.3 -> clamped to 0.5 by .max(0.5)
                bytes: 500,
                tokens: 100,
                doc_pct: None,
                bytes_per_line: None,
                depth: 2,
            },
            FileStatRow {
                path: "file2.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 200,
                comments: 20,
                blanks: 10,
                lines: 200, // h = 200/10 = 20.0
                bytes: 2000,
                tokens: 400,
                doc_pct: None,
                bytes_per_line: None,
                depth: 3,
            },
            FileStatRow {
                path: "file3.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 75,
                comments: 7,
                blanks: 3,
                lines: 75, // h = 75/10 = 7.5
                bytes: 750,
                tokens: 150,
                doc_pct: None,
                bytes_per_line: None,
                depth: 0,
            },
            FileStatRow {
                path: "file4.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 150,
                comments: 15,
                blanks: 8,
                lines: 150, // h = 150/10 = 15.0
                bytes: 1500,
                tokens: 300,
                doc_pct: None,
                bytes_per_line: None,
                depth: 1,
            },
            // idx=5: x = (5%5)*2 = 0, y = (5/5)*2 = 2
            FileStatRow {
                path: "file5.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 80,
                comments: 8,
                blanks: 4,
                lines: 80, // h = 80/10 = 8.0
                bytes: 800,
                tokens: 160,
                doc_pct: None,
                bytes_per_line: None,
                depth: 2,
            },
            // idx=6: x = (6%5)*2 = 2, y = (6/5)*2 = 2
            FileStatRow {
                path: "file6.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 60,
                comments: 6,
                blanks: 3,
                lines: 60, // h = 60/10 = 6.0
                bytes: 600,
                tokens: 120,
                doc_pct: None,
                bytes_per_line: None,
                depth: 1,
            },
        ];
        receipt.derived = Some(derived);
        let result = render_obj(&receipt).expect("render_obj should succeed with fun feature");

        // Parse the OBJ output into objects with their vertices
        // Each object starts with "o <name>" followed by 8 vertices
        #[allow(clippy::type_complexity)]
        let objects: Vec<(&str, Vec<(f32, f32, f32)>)> = result
            .split("o ")
            .skip(1)
            .map(|section| {
                let lines: Vec<&str> = section.lines().collect();
                let name = lines[0];
                let vertices: Vec<(f32, f32, f32)> = lines[1..]
                    .iter()
                    .filter(|l| l.starts_with("v "))
                    .take(8)
                    .map(|l| {
                        let parts: Vec<f32> = l[2..]
                            .split_whitespace()
                            .map(|p| p.parse().unwrap())
                            .collect();
                        (parts[0], parts[1], parts[2])
                    })
                    .collect();
                (name, vertices)
            })
            .collect();

        // Verify we have 7 objects
        assert_eq!(objects.len(), 7, "expected 7 buildings");

        // Helper to get first vertex (base corner) of each object
        fn base_corner(obj: &(&str, Vec<(f32, f32, f32)>)) -> (f32, f32, f32) {
            obj.1[0]
        }
        fn top_corner(obj: &(&str, Vec<(f32, f32, f32)>)) -> (f32, f32, f32) {
            obj.1[4] // 5th vertex is top of first corner
        }

        // idx=0: x=0, y=0, h=10
        assert_eq!(
            base_corner(&objects[0]),
            (0.0, 0.0, 0.0),
            "file0 base position"
        );
        assert_eq!(
            top_corner(&objects[0]).2,
            10.0,
            "file0 height should be 10.0 (100/10)"
        );

        // idx=1: x=2, y=0, h=0.5 (clamped from 0.3)
        // Tests: * 2.0 multiplier, .max(0.5) clamping
        assert_eq!(
            base_corner(&objects[1]),
            (2.0, 0.0, 0.0),
            "file1 base position"
        );
        assert_eq!(
            top_corner(&objects[1]).2,
            0.5,
            "file1 height should be 0.5 (clamped from 3/10=0.3)"
        );

        // idx=2: x=4, y=0, h=20
        assert_eq!(
            base_corner(&objects[2]),
            (4.0, 0.0, 0.0),
            "file2 base position"
        );
        assert_eq!(
            top_corner(&objects[2]).2,
            20.0,
            "file2 height should be 20.0 (200/10)"
        );

        // idx=3: x=6, y=0, h=7.5
        assert_eq!(
            base_corner(&objects[3]),
            (6.0, 0.0, 0.0),
            "file3 base position"
        );
        assert_eq!(
            top_corner(&objects[3]).2,
            7.5,
            "file3 height should be 7.5 (75/10)"
        );

        // idx=4: x=8, y=0, h=15
        // Tests: % 5 at boundary (4 % 5 = 4, not 0)
        assert_eq!(
            base_corner(&objects[4]),
            (8.0, 0.0, 0.0),
            "file4 base position (x = 4*2 = 8)"
        );
        assert_eq!(
            top_corner(&objects[4]).2,
            15.0,
            "file4 height should be 15.0 (150/10)"
        );

        // idx=5: x=0, y=2, h=8
        // Tests: % 5 wrapping (5 % 5 = 0), / 5 incrementing (5 / 5 = 1)
        // Catches mutations: % -> / would give x=2, / -> % would give y=0
        assert_eq!(
            base_corner(&objects[5]),
            (0.0, 2.0, 0.0),
            "file5 base position (x=0 from 5%5, y=2 from 5/5*2)"
        );
        assert_eq!(
            top_corner(&objects[5]).2,
            8.0,
            "file5 height should be 8.0 (80/10)"
        );

        // idx=6: x=2, y=2, h=6
        // Tests: both % and / together at idx=6
        assert_eq!(
            base_corner(&objects[6]),
            (2.0, 2.0, 0.0),
            "file6 base position (x=2 from 6%5*2, y=2 from 6/5*2)"
        );
        assert_eq!(
            top_corner(&objects[6]).2,
            6.0,
            "file6 height should be 6.0 (60/10)"
        );

        // Verify face definitions exist (basic structural check)
        assert!(result.contains("f 1 2 3 4"), "missing face definition");
    }

    // Test render_midi with fun feature - verify note calculations using midly parser
    // This test verifies arithmetic correctness for:
    // - key = 60 + (depth % 12)
    // - velocity = min(40 + min(lines, 127) / 2, 120)
    // - start = idx * 240
    #[cfg(feature = "fun")]
    #[test]
    fn test_render_midi_note_math() {
        use midly::{MidiMessage, Smf, TrackEventKind};

        let mut receipt = minimal_receipt();
        let mut derived = sample_derived();
        // Create rows with specific depths and lines to verify math
        // Each row maps to a note:
        //   key = 60 + (depth % 12)
        //   velocity = (40 + (lines.min(127) / 2)).min(120)
        //   start = idx * 240
        derived.top.largest_lines = vec![
            // idx=0: key=60+(5%12)=65, vel=40+(60/2)=70, start=0*240=0
            FileStatRow {
                path: "a.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 50,
                comments: 5,
                blanks: 2,
                lines: 60,
                bytes: 500,
                tokens: 100,
                doc_pct: None,
                bytes_per_line: None,
                depth: 5,
            },
            // idx=1: key=60+(15%12)=63, vel=40+(127/2)=103, start=1*240=240
            // Tests: % 12 wrapping (15 % 12 = 3), lines clamped at 127
            FileStatRow {
                path: "b.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 200, // clamped to 127 for velocity calc
                bytes: 1000,
                tokens: 200,
                doc_pct: None,
                bytes_per_line: None,
                depth: 15,
            },
            // idx=2: key=60+(0%12)=60, vel=40+(20/2)=50, start=2*240=480
            FileStatRow {
                path: "c.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 20,
                comments: 2,
                blanks: 1,
                lines: 20,
                bytes: 200,
                tokens: 40,
                doc_pct: None,
                bytes_per_line: None,
                depth: 0,
            },
            // idx=3: key=60+(12%12)=60, vel=40+(min(160,127)/2)=40+(127/2)=40+63=103, start=3*240=720
            // Tests: % 12 at boundary (12 % 12 = 0)
            FileStatRow {
                path: "d.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                code: 160,
                comments: 16,
                blanks: 8,
                lines: 160,
                bytes: 1600,
                tokens: 320,
                doc_pct: None,
                bytes_per_line: None,
                depth: 12,
            },
        ];
        receipt.derived = Some(derived);

        let result = render_midi(&receipt).unwrap();

        // Parse with midly
        let smf = Smf::parse(&result).expect("should parse as valid MIDI");

        // Collect NoteOn events with their absolute times
        let mut notes: Vec<(u32, u8, u8)> = Vec::new(); // (time, key, velocity)
        let mut abs_time = 0u32;

        for event in &smf.tracks[0] {
            abs_time += event.delta.as_int();
            if let TrackEventKind::Midi {
                message: MidiMessage::NoteOn { key, vel },
                ..
            } = event.kind
            {
                notes.push((abs_time, key.as_int(), vel.as_int()));
            }
        }

        // Should have 4 NoteOn events
        assert_eq!(notes.len(), 4, "expected 4 NoteOn events, got {:?}", notes);

        // Verify each note precisely
        // Note 0: time=0, key=65, velocity=70
        assert_eq!(
            notes[0],
            (0, 65, 70),
            "note 0: expected (time=0, key=65=60+5, vel=70=40+60/2), got {:?}",
            notes[0]
        );

        // Note 1: time=240, key=63, velocity=103
        // key=60+(15%12)=60+3=63, vel=40+(127/2)=40+63=103
        assert_eq!(
            notes[1],
            (240, 63, 103),
            "note 1: expected (time=240=1*240, key=63=60+(15%12), vel=103=40+127/2), got {:?}",
            notes[1]
        );

        // Note 2: time=480, key=60, velocity=50
        assert_eq!(
            notes[2],
            (480, 60, 50),
            "note 2: expected (time=480=2*240, key=60=60+0, vel=50=40+20/2), got {:?}",
            notes[2]
        );

        // Note 3: time=720, key=60, velocity=103
        // key=60+(12%12)=60+0=60, vel=40+(min(160,127)/2)=40+63=103
        assert_eq!(
            notes[3],
            (720, 60, 103),
            "note 3: expected (time=720=3*240, key=60=60+(12%12), vel=103=40+127/2), got {:?}",
            notes[3]
        );

        // Verify NoteOff timing too (duration=180)
        let mut note_offs: Vec<(u32, u8)> = Vec::new(); // (time, key)
        abs_time = 0;
        for event in &smf.tracks[0] {
            abs_time += event.delta.as_int();
            if let TrackEventKind::Midi {
                message: MidiMessage::NoteOff { key, .. },
                ..
            } = event.kind
            {
                note_offs.push((abs_time, key.as_int()));
            }
        }

        // NoteOff times should be start + 180
        assert!(
            note_offs.iter().any(|&(t, k)| t == 180 && k == 65),
            "expected NoteOff for key 65 at time 180, got {:?}",
            note_offs
        );
        assert!(
            note_offs.iter().any(|&(t, k)| t == 420 && k == 63),
            "expected NoteOff for key 63 at time 420 (240+180), got {:?}",
            note_offs
        );
        assert!(
            note_offs.iter().any(|&(t, k)| t == 660 && k == 60),
            "expected NoteOff for key 60 at time 660 (480+180), got {:?}",
            note_offs
        );
        assert!(
            note_offs.iter().any(|&(t, k)| t == 900 && k == 60),
            "expected NoteOff for key 60 at time 900 (720+180), got {:?}",
            note_offs
        );
    }

    // Test render_midi with empty derived - should still produce valid MIDI
    #[cfg(feature = "fun")]
    #[test]
    fn test_render_midi_no_derived() {
        use midly::Smf;

        let receipt = minimal_receipt();
        let result = render_midi(&receipt).unwrap();

        // Should produce a valid MIDI (not empty, parseable)
        assert!(!result.is_empty(), "MIDI output should not be empty");
        assert!(
            result.len() > 14,
            "MIDI should have header (14 bytes) + track data"
        );

        // Parse and verify structure
        let smf = Smf::parse(&result).expect("should be valid MIDI even with no notes");
        assert_eq!(smf.tracks.len(), 1, "should have exactly one track");
    }

    // Test render_obj with no derived data
    #[cfg(feature = "fun")]
    #[test]
    fn test_render_obj_no_derived() {
        let receipt = minimal_receipt();
        let result = render_obj(&receipt).expect("render_obj should succeed");

        // Should return fallback string when no derived data
        assert_eq!(result, "# tokmd code city\n");
    }

    // Test render_md basic structure
    #[test]
    fn test_render_md_basic() {
        let receipt = minimal_receipt();
        let result = render_md(&receipt);
        assert!(result.starts_with("# tokmd analysis\n"));
        assert!(result.contains("Preset: `receipt`"));
    }

    // Test render_md with inputs
    #[test]
    fn test_render_md_inputs() {
        let mut receipt = minimal_receipt();
        receipt.source.inputs = vec!["path1".to_string(), "path2".to_string()];
        let result = render_md(&receipt);
        assert!(result.contains("## Inputs"));
        assert!(result.contains("- `path1`"));
        assert!(result.contains("- `path2`"));
    }

    // Test render_md empty inputs - should NOT have inputs section
    #[test]
    fn test_render_md_empty_inputs() {
        let mut receipt = minimal_receipt();
        receipt.source.inputs.clear();
        let result = render_md(&receipt);
        assert!(!result.contains("## Inputs"));
    }

    // Test render_md with archetype
    #[test]
    fn test_render_md_archetype() {
        let mut receipt = minimal_receipt();
        receipt.archetype = Some(Archetype {
            kind: "library".to_string(),
            evidence: vec!["Cargo.toml".to_string(), "src/lib.rs".to_string()],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Archetype"));
        assert!(result.contains("- Kind: `library`"));
        assert!(result.contains("- Evidence: `Cargo.toml`, `src/lib.rs`"));
    }

    // Test render_md with archetype empty evidence
    #[test]
    fn test_render_md_archetype_no_evidence() {
        let mut receipt = minimal_receipt();
        receipt.archetype = Some(Archetype {
            kind: "app".to_string(),
            evidence: vec![],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Archetype"));
        assert!(result.contains("- Kind: `app`"));
        assert!(!result.contains("Evidence"));
    }

    // Test render_md with topics
    #[test]
    fn test_render_md_topics() {
        use std::collections::BTreeMap;
        let mut per_module = BTreeMap::new();
        per_module.insert(
            "src".to_string(),
            vec![TopicTerm {
                term: "parser".to_string(),
                score: 1.5,
                tf: 10,
                df: 2,
            }],
        );
        let mut receipt = minimal_receipt();
        receipt.topics = Some(TopicClouds {
            overall: vec![TopicTerm {
                term: "code".to_string(),
                score: 2.0,
                tf: 20,
                df: 5,
            }],
            per_module,
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Topics"));
        assert!(result.contains("- Overall: `code`"));
        assert!(result.contains("- `src`: parser"));
    }

    // Test render_md with topics empty module terms
    #[test]
    fn test_render_md_topics_empty_module() {
        use std::collections::BTreeMap;
        let mut per_module = BTreeMap::new();
        per_module.insert("empty_module".to_string(), vec![]);
        let mut receipt = minimal_receipt();
        receipt.topics = Some(TopicClouds {
            overall: vec![],
            per_module,
        });
        let result = render_md(&receipt);
        // Empty module should be skipped
        assert!(!result.contains("empty_module"));
    }

    // Test render_md with entropy
    #[test]
    fn test_render_md_entropy() {
        let mut receipt = minimal_receipt();
        receipt.entropy = Some(EntropyReport {
            suspects: vec![EntropyFinding {
                path: "secret.bin".to_string(),
                module: "root".to_string(),
                entropy_bits_per_byte: 7.5,
                sample_bytes: 1024,
                class: EntropyClass::High,
            }],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Entropy profiling"));
        assert!(result.contains("|secret.bin|root|7.50|1024|High|"));
    }

    // Test render_md with entropy no suspects
    #[test]
    fn test_render_md_entropy_no_suspects() {
        let mut receipt = minimal_receipt();
        receipt.entropy = Some(EntropyReport { suspects: vec![] });
        let result = render_md(&receipt);
        assert!(result.contains("## Entropy profiling"));
        assert!(result.contains("No entropy outliers detected"));
    }

    // Test render_md with license
    #[test]
    fn test_render_md_license() {
        let mut receipt = minimal_receipt();
        receipt.license = Some(LicenseReport {
            effective: Some("MIT".to_string()),
            findings: vec![LicenseFinding {
                spdx: "MIT".to_string(),
                confidence: 0.95,
                source_path: "LICENSE".to_string(),
                source_kind: LicenseSourceKind::Text,
            }],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## License radar"));
        assert!(result.contains("- Effective: `MIT`"));
        assert!(result.contains("|MIT|0.95|LICENSE|Text|"));
    }

    // Test render_md with license empty findings
    #[test]
    fn test_render_md_license_no_findings() {
        let mut receipt = minimal_receipt();
        receipt.license = Some(LicenseReport {
            effective: None,
            findings: vec![],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## License radar"));
        assert!(result.contains("Heuristic detection"));
        assert!(!result.contains("|SPDX|")); // No table header
    }

    // Test render_md with corporate fingerprint
    #[test]
    fn test_render_md_corporate_fingerprint() {
        let mut receipt = minimal_receipt();
        receipt.corporate_fingerprint = Some(CorporateFingerprint {
            domains: vec![DomainStat {
                domain: "example.com".to_string(),
                commits: 50,
                pct: 0.75,
            }],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Corporate fingerprint"));
        assert!(result.contains("|example.com|50|75.0%|"));
    }

    // Test render_md with corporate fingerprint no domains
    #[test]
    fn test_render_md_corporate_fingerprint_no_domains() {
        let mut receipt = minimal_receipt();
        receipt.corporate_fingerprint = Some(CorporateFingerprint { domains: vec![] });
        let result = render_md(&receipt);
        assert!(result.contains("## Corporate fingerprint"));
        assert!(result.contains("No commit domains detected"));
    }

    // Test render_md with predictive churn
    #[test]
    fn test_render_md_churn() {
        use std::collections::BTreeMap;
        let mut per_module = BTreeMap::new();
        per_module.insert(
            "src".to_string(),
            ChurnTrend {
                slope: 0.5,
                r2: 0.8,
                recent_change: 5,
                classification: TrendClass::Rising,
            },
        );
        let mut receipt = minimal_receipt();
        receipt.predictive_churn = Some(PredictiveChurnReport { per_module });
        let result = render_md(&receipt);
        assert!(result.contains("## Predictive churn"));
        assert!(result.contains("|src|0.5000|0.80|5|Rising|"));
    }

    // Test render_md with predictive churn empty
    #[test]
    fn test_render_md_churn_empty() {
        use std::collections::BTreeMap;
        let mut receipt = minimal_receipt();
        receipt.predictive_churn = Some(PredictiveChurnReport {
            per_module: BTreeMap::new(),
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Predictive churn"));
        assert!(result.contains("No churn signals detected"));
    }

    // Test render_md with assets
    #[test]
    fn test_render_md_assets() {
        let mut receipt = minimal_receipt();
        receipt.assets = Some(AssetReport {
            total_files: 5,
            total_bytes: 1000000,
            categories: vec![AssetCategoryRow {
                category: "images".to_string(),
                files: 3,
                bytes: 500000,
                extensions: vec!["png".to_string(), "jpg".to_string()],
            }],
            top_files: vec![AssetFileRow {
                path: "logo.png".to_string(),
                bytes: 100000,
                category: "images".to_string(),
                extension: "png".to_string(),
            }],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Assets"));
        assert!(result.contains("- Total files: `5`"));
        assert!(result.contains("|images|3|500000|png, jpg|"));
        assert!(result.contains("|logo.png|100000|images|"));
    }

    // Test render_md with assets empty categories
    #[test]
    fn test_render_md_assets_empty() {
        let mut receipt = minimal_receipt();
        receipt.assets = Some(AssetReport {
            total_files: 0,
            total_bytes: 0,
            categories: vec![],
            top_files: vec![],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Assets"));
        assert!(result.contains("- Total files: `0`"));
        assert!(!result.contains("|Category|")); // No table
    }

    // Test render_md with deps
    #[test]
    fn test_render_md_deps() {
        let mut receipt = minimal_receipt();
        receipt.deps = Some(DependencyReport {
            total: 50,
            lockfiles: vec![LockfileReport {
                path: "Cargo.lock".to_string(),
                kind: "cargo".to_string(),
                dependencies: 50,
            }],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Dependencies"));
        assert!(result.contains("- Total: `50`"));
        assert!(result.contains("|Cargo.lock|cargo|50|"));
    }

    // Test render_md with deps empty lockfiles
    #[test]
    fn test_render_md_deps_empty() {
        let mut receipt = minimal_receipt();
        receipt.deps = Some(DependencyReport {
            total: 0,
            lockfiles: vec![],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Dependencies"));
        assert!(!result.contains("|Lockfile|"));
    }

    // Test render_md with git
    #[test]
    fn test_render_md_git() {
        let mut receipt = minimal_receipt();
        receipt.git = Some(GitReport {
            commits_scanned: 100,
            files_seen: 50,
            hotspots: vec![HotspotRow {
                path: "src/lib.rs".to_string(),
                commits: 25,
                lines: 500,
                score: 12500,
            }],
            bus_factor: vec![BusFactorRow {
                module: "src".to_string(),
                authors: 3,
            }],
            freshness: FreshnessReport {
                threshold_days: 90,
                stale_files: 5,
                total_files: 50,
                stale_pct: 0.1,
                by_module: vec![ModuleFreshnessRow {
                    module: "src".to_string(),
                    avg_days: 30.0,
                    p90_days: 60.0,
                    stale_pct: 0.05,
                }],
            },
            coupling: vec![CouplingRow {
                left: "src/a.rs".to_string(),
                right: "src/b.rs".to_string(),
                count: 10,
                jaccard: Some(0.5),
                lift: Some(1.2),
            }],
            age_distribution: Some(CodeAgeDistributionReport {
                buckets: vec![CodeAgeBucket {
                    label: "0-30d".to_string(),
                    min_days: 0,
                    max_days: Some(30),
                    files: 10,
                    pct: 0.2,
                }],
                recent_refreshes: 12,
                prior_refreshes: 8,
                refresh_trend: TrendClass::Rising,
            }),
            intent: None,
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Git metrics"));
        assert!(result.contains("- Commits scanned: `100`"));
        assert!(result.contains("|src/lib.rs|25|500|12500|"));
        assert!(result.contains("|src|3|"));
        assert!(result.contains("Stale threshold (days): `90`"));
        assert!(result.contains("|src|30.00|60.00|5.0%|"));
        assert!(result.contains("### Code age"));
        assert!(result.contains("Refresh trend: `Rising`"));
        assert!(result.contains("|0-30d|0|30|10|20.0%|"));
        assert!(result.contains("|src/a.rs|src/b.rs|10|"));
    }

    // Test render_md with git empty sections
    #[test]
    fn test_render_md_git_empty() {
        let mut receipt = minimal_receipt();
        receipt.git = Some(GitReport {
            commits_scanned: 0,
            files_seen: 0,
            hotspots: vec![],
            bus_factor: vec![],
            freshness: FreshnessReport {
                threshold_days: 90,
                stale_files: 0,
                total_files: 0,
                stale_pct: 0.0,
                by_module: vec![],
            },
            coupling: vec![],
            age_distribution: None,
            intent: None,
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Git metrics"));
        assert!(!result.contains("### Hotspots"));
        assert!(!result.contains("### Bus factor"));
        assert!(!result.contains("### Coupling"));
    }

    // Test render_md with imports
    #[test]
    fn test_render_md_imports() {
        let mut receipt = minimal_receipt();
        receipt.imports = Some(ImportReport {
            granularity: "file".to_string(),
            edges: vec![ImportEdge {
                from: "src/main.rs".to_string(),
                to: "src/lib.rs".to_string(),
                count: 5,
            }],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Imports"));
        assert!(result.contains("- Granularity: `file`"));
        assert!(result.contains("|src/main.rs|src/lib.rs|5|"));
    }

    // Test render_md with imports empty
    #[test]
    fn test_render_md_imports_empty() {
        let mut receipt = minimal_receipt();
        receipt.imports = Some(ImportReport {
            granularity: "module".to_string(),
            edges: vec![],
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Imports"));
        assert!(!result.contains("|From|To|"));
    }

    // Test render_md with dup
    #[test]
    fn test_render_md_dup() {
        let mut receipt = minimal_receipt();
        receipt.dup = Some(DuplicateReport {
            wasted_bytes: 50000,
            strategy: "content".to_string(),
            groups: vec![DuplicateGroup {
                hash: "abc123".to_string(),
                bytes: 1000,
                files: vec!["a.txt".to_string(), "b.txt".to_string()],
            }],
            density: Some(DuplicationDensityReport {
                duplicate_groups: 1,
                duplicate_files: 2,
                duplicated_bytes: 2000,
                wasted_bytes: 1000,
                wasted_pct_of_codebase: 0.1,
                by_module: vec![ModuleDuplicationDensityRow {
                    module: "src".to_string(),
                    duplicate_files: 2,
                    wasted_files: 1,
                    duplicated_bytes: 2000,
                    wasted_bytes: 1000,
                    module_bytes: 10_000,
                    density: 0.1,
                }],
            }),
            near: None,
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Duplicates"));
        assert!(result.contains("- Wasted bytes: `50000`"));
        assert!(result.contains("### Duplication density"));
        assert!(result.contains("Waste vs codebase: `10.0%`"));
        assert!(result.contains("|src|2|1|2000|1000|10000|10.0%|"));
        assert!(result.contains("|abc123|1000|2|")); // 2 files
    }

    // Test render_md with dup empty
    #[test]
    fn test_render_md_dup_empty() {
        let mut receipt = minimal_receipt();
        receipt.dup = Some(DuplicateReport {
            wasted_bytes: 0,
            strategy: "content".to_string(),
            groups: vec![],
            density: None,
            near: None,
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Duplicates"));
        assert!(!result.contains("|Hash|Bytes|"));
    }

    // Test render_md with fun eco_label
    #[test]
    fn test_render_md_fun() {
        let mut receipt = minimal_receipt();
        receipt.fun = Some(FunReport {
            eco_label: Some(EcoLabel {
                label: "A+".to_string(),
                score: 95.5,
                bytes: 10000,
                notes: "Very efficient".to_string(),
            }),
        });
        let result = render_md(&receipt);
        assert!(result.contains("## Eco label"));
        assert!(result.contains("- Label: `A+`"));
        assert!(result.contains("- Score: `95.5`"));
    }

    // Test render_md with fun no eco_label
    #[test]
    fn test_render_md_fun_no_label() {
        let mut receipt = minimal_receipt();
        receipt.fun = Some(FunReport { eco_label: None });
        let result = render_md(&receipt);
        // No eco label section should appear
        assert!(!result.contains("## Eco label"));
    }

    // Test render_md with derived
    #[test]
    fn test_render_md_derived() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_md(&receipt);
        assert!(result.contains("## Totals"));
        assert!(result.contains("|10|1000|200|100|1300|50000|2500|"));
        assert!(result.contains("## Ratios"));
        assert!(result.contains("## Distribution"));
        assert!(result.contains("## File size histogram"));
        assert!(result.contains("## Top offenders"));
        assert!(result.contains("## Structure"));
        assert!(result.contains("## Test density"));
        assert!(result.contains("## TODOs"));
        assert!(result.contains("## Boilerplate ratio"));
        assert!(result.contains("## Polyglot"));
        assert!(result.contains("## Reading time"));
        assert!(result.contains("## Context window"));
        assert!(result.contains("## COCOMO estimate"));
        assert!(result.contains("## Integrity"));
    }

    // Test render function dispatch
    #[test]
    fn test_render_dispatch_md() {
        let receipt = minimal_receipt();
        let result = render(&receipt, tokmd_config::AnalysisFormat::Md).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.starts_with("# tokmd analysis")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_json() {
        let receipt = minimal_receipt();
        let result = render(&receipt, tokmd_config::AnalysisFormat::Json).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.contains("\"schema_version\": 2")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_xml() {
        let receipt = minimal_receipt();
        let result = render(&receipt, tokmd_config::AnalysisFormat::Xml).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.contains("<analysis>")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_tree() {
        let receipt = minimal_receipt();
        let result = render(&receipt, tokmd_config::AnalysisFormat::Tree).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.contains("(tree unavailable)")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_svg() {
        let receipt = minimal_receipt();
        let result = render(&receipt, tokmd_config::AnalysisFormat::Svg).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.contains("<svg")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_mermaid() {
        let receipt = minimal_receipt();
        let result = render(&receipt, tokmd_config::AnalysisFormat::Mermaid).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.starts_with("graph TD")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    #[test]
    fn test_render_dispatch_jsonld() {
        let receipt = minimal_receipt();
        let result = render(&receipt, tokmd_config::AnalysisFormat::Jsonld).unwrap();
        match result {
            RenderedOutput::Text(s) => assert!(s.contains("@context")),
            RenderedOutput::Binary(_) => panic!("expected text"),
        }
    }

    // Test chrono_lite_timestamp produces valid format
    #[test]
    fn test_chrono_lite_timestamp() {
        let ts = chrono_lite_timestamp();
        // Should be in format "YYYY-MM-DD HH:MM:SS UTC"
        assert!(ts.contains("UTC"));
        assert!(ts.len() > 10); // Should be reasonably long
    }

    // Test build_metrics_cards
    #[test]
    fn test_build_metrics_cards() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = build_metrics_cards(&receipt);
        assert!(result.contains("class=\"metric-card\""));
        assert!(result.contains("Files"));
        assert!(result.contains("Lines"));
        assert!(result.contains("Code"));
        assert!(result.contains("Context Fit")); // Has context_window
    }

    // Test build_metrics_cards without derived
    #[test]
    fn test_build_metrics_cards_no_derived() {
        let receipt = minimal_receipt();
        let result = build_metrics_cards(&receipt);
        assert!(result.is_empty());
    }

    // Test build_table_rows
    #[test]
    fn test_build_table_rows() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = build_table_rows(&receipt);
        assert!(result.contains("<tr>"));
        assert!(result.contains("src/lib.rs"));
    }

    // Test build_table_rows without derived
    #[test]
    fn test_build_table_rows_no_derived() {
        let receipt = minimal_receipt();
        let result = build_table_rows(&receipt);
        assert!(result.is_empty());
    }

    // Test build_report_json
    #[test]
    fn test_build_report_json() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = build_report_json(&receipt);
        assert!(result.contains("files"));
        assert!(result.contains("src/lib.rs"));
        // XSS escaping
        assert!(!result.contains("<"));
        assert!(!result.contains(">"));
    }

    // Test build_report_json without derived
    #[test]
    fn test_build_report_json_no_derived() {
        let receipt = minimal_receipt();
        let result = build_report_json(&receipt);
        assert!(result.contains("\"files\":[]"));
    }

    // Test render_html
    #[test]
    fn test_render_html() {
        let mut receipt = minimal_receipt();
        receipt.derived = Some(sample_derived());
        let result = render_html(&receipt);
        assert!(result.contains("<!DOCTYPE html>") || result.contains("<html"));
    }
}
