//! # tokmd-analysis-format
//!
//! Rendering for analysis receipts.

use anyhow::Result;
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
        AnalysisFormat::Obj => Ok(RenderedOutput::Text(render_obj(receipt))),
        AnalysisFormat::Midi => Ok(RenderedOutput::Binary(render_midi(receipt)?)),
        AnalysisFormat::Tree => Ok(RenderedOutput::Text(render_tree(receipt))),
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
                .then_with(|| a.0.cmp(&b.0))
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
        if !git.coupling.is_empty() {
            out.push_str("### Coupling\n\n");
            out.push_str("|Left|Right|Count|\n");
            out.push_str("|---|---|---:|\n");
            for row in git.coupling.iter().take(10) {
                out.push_str(&format!("|{}|{}|{}|\n", row.left, row.right, row.count));
            }
            out.push('\n');
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
    }

    if let Some(fun) = &receipt.fun {
        if let Some(label) = &fun.eco_label {
            out.push_str("## Eco label\n\n");
            out.push_str(&format!(
                "- Label: `{}`\n- Score: `{}`\n- Bytes: `{}`\n- Notes: `{}`\n\n",
                label.label,
                fmt_f64(label.score, 1),
                label.bytes,
                label.notes
            ));
        }
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
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" role=\"img\"><rect width=\"{label_width}\" height=\"{height}\" fill=\"#555\"/><rect x=\"{label_width}\" width=\"{value_width}\" height=\"{height}\" fill=\"#4c9aff\"/><text x=\"{lx}\" y=\"{ty}\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"12\">{label}</text><text x=\"{vx}\" y=\"{ty}\" fill=\"#fff\" font-family=\"Verdana\" font-size=\"12\">{value}</text></svg>",
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

#[cfg(feature = "fun")]
fn render_obj(receipt: &AnalysisReceipt) -> String {
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
        return tokmd_fun::render_obj(&buildings);
    }
    "# obj".to_string()
}

#[cfg(not(feature = "fun"))]
fn render_obj(_receipt: &AnalysisReceipt) -> String {
    "# obj (fun feature disabled)".to_string()
}

#[cfg(feature = "fun")]
fn render_midi(receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
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

#[cfg(not(feature = "fun"))]
fn render_midi(_receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
    Ok(Vec::new())
}

fn sanitize_mermaid(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}
