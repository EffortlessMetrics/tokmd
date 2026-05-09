//! Markdown renderer for tokmd analysis receipts.
//!
//! This module owns all Markdown formatting logic behind the `tokmd-format`
//! analysis facade.
//!
//! ## Effort rendering
//!
//! Effort sections are rendered in two tiers:
//!
//! 1. `receipt.effort` — preferred path for the newer effort-estimation
//!    receipt surface. Renders size basis, confidence, drivers,
//!    assumptions, and optional delta data.
//! 2. `derived.cocomo` — legacy fallback used when the richer `effort`
//!    section is absent but classic derived COCOMO data is present.
//!
//! The formatter intentionally renders whatever the receipt contains without
//! inferring missing estimate data.

use std::fmt::Write;
use tokmd_analysis_types::{AnalysisReceipt, FileStatRow};

mod api_surface;
mod assets;
mod complexity;
mod duplicates;
mod effort;
mod git;
mod imports;

/// Render an [`AnalysisReceipt`] to a Markdown string.
///
/// This is the sole public entry point. All subsections (derived metrics,
/// effort, duplicates, complexity, etc.) are rendered internally.
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
        let _ = writeln!(
            out,
            "|Doc density|{}|",
            fmt_pct(derived.doc_density.total.ratio)
        );
        let _ = writeln!(
            out,
            "|Whitespace ratio|{}|",
            fmt_pct(derived.whitespace.total.ratio)
        );
        let _ = writeln!(
            out,
            "|Bytes per line|{}|\n",
            fmt_f64(derived.verbosity.total.rate, 2)
        );

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
            let max = bucket
                .max
                .map(|v| v.to_string())
                .unwrap_or_else(|| "∞".to_string());
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

        // Prefer the richer top-level effort contract when present; fall back
        // to legacy derived COCOMO output for older receipts.
        if let Some(effort) = &receipt.effort {
            effort::render_effort_report(&mut out, effort);
        } else if let Some(cocomo) = &derived.cocomo {
            effort::render_legacy_cocomo_report(&mut out, derived, cocomo);
        }

        out.push_str("## Integrity\n\n");
        let _ = writeln!(
            out,
            "- Hash: `{}` (`{}`)\n- Entries: `{}`\n",
            derived.integrity.hash, derived.integrity.algo, derived.integrity.entries
        );
    }

    if let Some(assets) = &receipt.assets {
        assets::render_asset_report(&mut out, assets);
    }

    if let Some(deps) = &receipt.deps {
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

    if let Some(git) = &receipt.git {
        git::render_git_report(&mut out, git);
    }

    if let Some(imports) = &receipt.imports {
        imports::render_import_report(&mut out, imports);
    }

    if let Some(dup) = &receipt.dup {
        duplicates::render_duplicate_report(&mut out, dup);
    }

    if let Some(cx) = &receipt.complexity {
        complexity::render_complexity_report(&mut out, cx);
    }

    if let Some(api) = &receipt.api_surface {
        api_surface::render_api_surface_report(&mut out, api);
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

fn render_file_table(rows: &[FileStatRow]) -> String {
    use std::fmt::Write;
    // Heuristic: (rows + 3) * 80 chars per row
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

fn fmt_pct(ratio: f64) -> String {
    format!("{:.1}%", ratio * 100.0)
}

fn fmt_f64(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
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
            effort: None,
        }
    }

    #[test]
    fn minimal_receipt_renders_without_panic() {
        let receipt = minimal_receipt();
        let md = render_md(&receipt);
        assert!(md.starts_with("# tokmd analysis\n"));
        assert!(md.contains("Preset: `receipt`"));
        assert!(md.contains("## Inputs\n"));
    }

    #[test]
    fn fmt_pct_output_format() {
        assert_eq!(fmt_pct(0.456), "45.6%");
        assert_eq!(fmt_pct(0.0), "0.0%");
        assert_eq!(fmt_pct(1.0), "100.0%");
    }

    #[test]
    fn fmt_f64_output_format() {
        assert_eq!(fmt_f64(std::f64::consts::PI, 2), "3.14");
        assert_eq!(fmt_f64(1.0, 4), "1.0000");
    }
}
