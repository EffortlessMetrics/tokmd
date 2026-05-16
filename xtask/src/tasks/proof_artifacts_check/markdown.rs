use crate::cli::ProofExecutionObservationsSummaryArgs;

use super::read_collector_runs;
use super::types::{ProofExecutionObservationCollection, ProofRunObservationCollection};

pub(super) fn render_proof_run_observation_collection_markdown(
    collection: &ProofRunObservationCollection,
) -> String {
    let mut out = String::new();
    out.push_str("# Proof Run Observation Collection\n\n");
    out.push_str("| Metric | Count |\n");
    out.push_str("| --- | ---: |\n");
    push_count_row(&mut out, "Observations", collection.counts.observations);
    push_count_row(
        &mut out,
        "Planned commands",
        collection.counts.commands_total,
    );
    push_count_row(
        &mut out,
        "Required commands",
        collection.counts.required_planned,
    );
    push_count_row(
        &mut out,
        "Advisory skipped commands",
        collection.counts.advisory_skipped,
    );
    push_count_row(&mut out, "Executed commands", collection.counts.executed);
    push_count_row(&mut out, "Passed commands", collection.counts.passed);
    push_count_row(&mut out, "Failed commands", collection.counts.failed);
    push_count_row(&mut out, "Unknown files", collection.counts.unknown_files);
    push_count_row(&mut out, "Distinct scopes", collection.scopes.len());

    if let Some(window) = &collection.window {
        out.push_str("\n## Observation Window\n\n");
        out.push_str(&format!("Source: `{}`\n\n", md_cell(&window.source)));
        out.push_str("| Metric | Count |\n");
        out.push_str("| --- | ---: |\n");
        push_count_row(
            &mut out,
            "Expected successful proof runs",
            window.expected_runs,
        );
        push_count_row(
            &mut out,
            "Observed runs with artifacts",
            window.observed_runs,
        );
        push_count_row(&mut out, "Missing runs", window.missing_runs);
        push_count_row(
            &mut out,
            "Unmatched observation artifacts",
            window.unmatched_observations,
        );

        if !window.missing.is_empty() {
            out.push_str("\n| Missing run | Branch | Created | URL |\n");
            out.push_str("| ---: | --- | --- | --- |\n");
            for run in &window.missing {
                out.push_str(&format!(
                    "| {} | `{}` | `{}` | {} |\n",
                    run.database_id,
                    md_cell(run.head_branch.as_deref().unwrap_or("")),
                    md_cell(run.created_at.as_deref().unwrap_or("")),
                    md_cell(run.url.as_deref().unwrap_or(""))
                ));
            }
        }
    }

    if !collection.profiles.is_empty() {
        out.push_str("\n## Profiles\n\n");
        out.push_str("| Profile | Observations | Required | Executed | Passed | Failed |\n");
        out.push_str("| --- | ---: | ---: | ---: | ---: | ---: |\n");
        for profile in &collection.profiles {
            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} |\n",
                md_cell(&profile.profile),
                profile.observations,
                profile.required_planned,
                profile.executed,
                profile.passed,
                profile.failed
            ));
        }
    }

    if !collection.scopes.is_empty() {
        out.push_str("\n## Scopes\n\n");
        out.push_str("| Scope | Kind | Observations | Executed |\n");
        out.push_str("| --- | --- | ---: | ---: |\n");
        for scope in &collection.scopes {
            out.push_str(&format!(
                "| `{}` | `{}` | {} | {} |\n",
                md_cell(&scope.name),
                md_cell(&scope.kind),
                scope.observations,
                scope.executed
            ));
        }
    }

    if !collection.guards.is_empty() {
        out.push_str("\n## Guards\n\n");
        out.push_str("| Reason | Observations | CI observations |\n");
        out.push_str("| --- | ---: | ---: |\n");
        for guard in &collection.guards {
            out.push_str(&format!(
                "| `{}` | {} | {} |\n",
                md_cell(&guard.reason),
                guard.observations,
                guard.ci_observations
            ));
        }
    }

    if !collection.sources.is_empty() {
        out.push_str("\n## Sources\n\n");
        out.push_str("| Source | Profile | Executed | Passed | Guard |\n");
        out.push_str("| --- | --- | ---: | ---: | --- |\n");
        for source in &collection.sources {
            out.push_str(&format!(
                "| `{}` | `{}` | {} | {} | `{}` |\n",
                md_cell(&source.path),
                md_cell(&source.profile),
                source.executed,
                source.passed,
                md_cell(&source.guard_reason)
            ));
        }
    }

    out
}

pub(super) fn render_observation_collection_markdown(
    collection: &ProofExecutionObservationCollection,
    args: &ProofExecutionObservationsSummaryArgs,
) -> String {
    let mut out = String::new();
    out.push_str("# Proof Executor Observation Collection\n\n");
    out.push_str("| Metric | Count |\n");
    out.push_str("| --- | ---: |\n");
    push_count_row(&mut out, "Observations", collection.counts.observations);
    push_count_row(&mut out, "Selected commands", collection.counts.selected);
    push_count_row(&mut out, "Executed commands", collection.counts.executed);
    push_count_row(&mut out, "Passed commands", collection.counts.passed);
    push_count_row(&mut out, "Failed commands", collection.counts.failed);
    push_count_row(&mut out, "Artifacts", collection.counts.artifacts);
    push_count_row(&mut out, "Distinct scopes", collection.scopes.len());

    if let Some(window) = &collection.window {
        out.push_str("\n## Observation Window\n\n");
        out.push_str(&format!("Source: `{}`\n\n", md_cell(&window.source)));
        out.push_str("| Metric | Count |\n");
        out.push_str("| --- | ---: |\n");
        push_count_row(
            &mut out,
            "Expected successful executor runs",
            window.expected_runs,
        );
        push_count_row(
            &mut out,
            "Observed runs with artifacts",
            window.observed_runs,
        );
        push_count_row(&mut out, "Missing runs", window.missing_runs);
        push_count_row(
            &mut out,
            "Unmatched observation artifacts",
            window.unmatched_observations,
        );

        if !window.missing.is_empty() {
            out.push_str("\n| Missing run | Branch | Created | URL |\n");
            out.push_str("| ---: | --- | --- | --- |\n");
            for run in &window.missing {
                out.push_str(&format!(
                    "| {} | `{}` | `{}` | {} |\n",
                    run.database_id,
                    md_cell(run.head_branch.as_deref().unwrap_or("")),
                    md_cell(run.created_at.as_deref().unwrap_or("")),
                    md_cell(run.url.as_deref().unwrap_or(""))
                ));
            }
        }
    }

    out.push_str("\n## Thresholds\n\n");
    out.push_str("| Threshold | Required | Actual | Status |\n");
    out.push_str("| --- | ---: | ---: | --- |\n");
    push_threshold_row(
        &mut out,
        "Observations",
        args.min_observations,
        collection.counts.observations,
    );
    push_threshold_row(
        &mut out,
        "Executed commands",
        args.min_executed,
        collection.counts.executed,
    );
    push_threshold_row(
        &mut out,
        "Distinct scopes",
        args.min_scopes,
        collection.scopes.len(),
    );
    push_threshold_row(
        &mut out,
        "Artifacts",
        args.min_artifacts,
        collection.counts.artifacts,
    );
    if let Some(collector_runs_json) = &args.collector_runs_json {
        let passing_collector_runs = read_collector_runs(collector_runs_json)
            .map(|runs| runs.len())
            .unwrap_or(0);
        push_threshold_row(
            &mut out,
            "Passing collector runs",
            args.min_passing_collector_runs,
            passing_collector_runs,
        );
    }

    if !collection.families.is_empty() {
        out.push_str("\n## Families\n\n");
        out.push_str("| Family | Observations | Executed | Artifacts |\n");
        out.push_str("| --- | ---: | ---: | ---: |\n");
        for family in &collection.families {
            out.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                md_cell(&family.family),
                family.observations,
                family.executed,
                family.artifacts
            ));
        }
    }

    if !collection.scopes.is_empty() {
        out.push_str("\n## Scopes\n\n");
        out.push_str("| Scope | Kind | Family | Observations | Executed | Artifacts |\n");
        out.push_str("| --- | --- | --- | ---: | ---: | ---: |\n");
        for scope in &collection.scopes {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} | {} | {} |\n",
                md_cell(&scope.name),
                md_cell(&scope.kind),
                md_cell(&scope.family),
                scope.observations,
                scope.executed,
                scope.artifacts
            ));
        }
    }

    if !collection.sources.is_empty() {
        out.push_str("\n## Sources\n\n");
        out.push_str("| Source | Family | Executed | Artifacts | Guard |\n");
        out.push_str("| --- | --- | ---: | ---: | --- |\n");
        for source in &collection.sources {
            out.push_str(&format!(
                "| `{}` | `{}` | {} | {} | `{}` |\n",
                md_cell(&source.path),
                md_cell(&source.family),
                source.executed,
                source.artifacts,
                md_cell(&source.guard_reason)
            ));
        }
    }

    out
}

fn push_count_row(out: &mut String, label: &str, count: usize) {
    out.push_str(&format!("| {label} | {count} |\n"));
}

fn push_threshold_row(out: &mut String, label: &str, required: usize, actual: usize) {
    let status = if actual >= required { "ok" } else { "below" };
    out.push_str(&format!("| {label} | {required} | {actual} | {status} |\n"));
}

fn md_cell(value: &str) -> String {
    value.replace('|', "\\|")
}
