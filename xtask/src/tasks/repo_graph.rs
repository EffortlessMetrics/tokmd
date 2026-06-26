use crate::cli::{RepoGraphArgs, RepoGraphExpectation};
use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

const SCHEMA: &str = "tokmd.repo_graph.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GraphRelation {
    Aligned,
    SwarmAhead,
    PublicationAhead,
    Diverged,
    Unrelated,
}

#[derive(Debug, Serialize)]
pub(crate) struct RepoGraphReport {
    pub(crate) schema: &'static str,
    pub(crate) ok: bool,
    pub(crate) expectation: String,
    pub(crate) relation: GraphRelation,
    pub(crate) next_action: &'static str,
    pub(crate) publication: RefReport,
    pub(crate) swarm: RefReport,
    pub(crate) merge_base: Option<String>,
    pub(crate) ahead_behind: AheadBehind,
}

#[derive(Debug, Serialize)]
pub(crate) struct RefReport {
    pub(crate) name: String,
    pub(crate) sha: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct AheadBehind {
    pub(crate) publication_ahead: u64,
    pub(crate) swarm_ahead: u64,
}

pub fn run(args: RepoGraphArgs) -> Result<()> {
    let report = repo_graph_report(&args)?;

    if let Some(path) = &args.json {
        write_json(path, &report)?;
    }

    print_human_report(&report);

    if report.ok {
        Ok(())
    } else {
        let hint = expectation_failure_hint(&report);
        bail!(
            "repo graph expectation {} was not met: relation {:?}, publication_ahead={}, swarm_ahead={}. next: {}",
            report.expectation,
            report.relation,
            report.ahead_behind.publication_ahead,
            report.ahead_behind.swarm_ahead,
            hint
        )
    }
}

fn repo_graph_report(args: &RepoGraphArgs) -> Result<RepoGraphReport> {
    let (publication_ref, publication_sha) = resolve_publication_revision(&args.publication)?;
    let swarm_sha = rev_parse(&args.swarm)?;
    let merge_base = merge_base(&publication_ref, &args.swarm)?;
    let ahead_behind = ahead_behind(&publication_ref, &args.swarm)?;
    let relation = classify_relation(merge_base.as_deref(), ahead_behind);
    let ok = expectation_matches(args.expect, relation);
    let expectation = expectation_name(args.expect).to_string();
    let next_action = graph_next_action(&expectation, relation);

    Ok(RepoGraphReport {
        schema: SCHEMA,
        ok,
        expectation,
        relation,
        next_action,
        publication: RefReport {
            name: publication_ref,
            sha: publication_sha,
        },
        swarm: RefReport {
            name: args.swarm.clone(),
            sha: swarm_sha,
        },
        merge_base,
        ahead_behind,
    })
}

/// Resolve the publication revision, returning the ref name actually used and
/// its SHA.
///
/// The default publication ref is `public/main`, which assumes the publication
/// remote is named `public`. Some clones name that remote `publication`
/// instead. When the requested `public/<branch>` ref does not resolve but the
/// equivalent `publication/<branch>` ref does, fall back to it so the check
/// works without forcing an explicit `--publication` override. A genuinely
/// missing ref still surfaces the original git error.
fn resolve_publication_revision(requested: &str) -> Result<(String, String)> {
    match rev_parse(requested) {
        Ok(sha) => Ok((requested.to_string(), sha)),
        Err(first_err) => {
            if let Some(alternate) = alternate_publication_ref(requested)
                && let Ok(sha) = rev_parse(&alternate)
            {
                return Ok((alternate, sha));
            }
            Err(first_err)
        }
    }
}

/// Map a `public/<branch>` ref to the `publication/<branch>` equivalent used
/// when the publication remote is named `publication`.
fn alternate_publication_ref(requested: &str) -> Option<String> {
    requested
        .strip_prefix("public/")
        .map(|branch| format!("publication/{branch}"))
}

fn rev_parse(revision: &str) -> Result<String> {
    let rev = format!("{revision}^{{commit}}");
    let output = Command::new("git")
        .args(["rev-parse", "--verify", &rev])
        .output()
        .with_context(|| format!("failed to run `git rev-parse --verify {rev}`"))?;

    if !output.status.success() {
        return Err(git_error("git rev-parse", &output.stderr));
    }

    let sha = String::from_utf8(output.stdout)
        .context("git rev-parse produced non-UTF-8 output")?
        .trim()
        .to_string();
    Ok(sha)
}

fn merge_base(left: &str, right: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["merge-base", left, right])
        .output()
        .with_context(|| format!("failed to run `git merge-base {left} {right}`"))?;

    if output.status.success() {
        let base = String::from_utf8(output.stdout)
            .context("git merge-base produced non-UTF-8 output")?
            .trim()
            .to_string();
        Ok(Some(base))
    } else if output.status.code() == Some(1) {
        Ok(None)
    } else {
        Err(git_error("git merge-base", &output.stderr))
    }
}

fn ahead_behind(publication: &str, swarm: &str) -> Result<AheadBehind> {
    let range = format!("{publication}...{swarm}");
    let output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", &range])
        .output()
        .with_context(|| format!("failed to run `git rev-list --left-right --count {range}`"))?;

    if !output.status.success() {
        return Err(git_error("git rev-list", &output.stderr));
    }

    let text =
        String::from_utf8(output.stdout).context("git rev-list produced non-UTF-8 output")?;
    let mut parts = text.split_whitespace();
    let publication_ahead = parts
        .next()
        .context("missing publication-ahead count from git rev-list")?
        .parse::<u64>()
        .context("invalid publication-ahead count from git rev-list")?;
    let swarm_ahead = parts
        .next()
        .context("missing swarm-ahead count from git rev-list")?
        .parse::<u64>()
        .context("invalid swarm-ahead count from git rev-list")?;

    Ok(AheadBehind {
        publication_ahead,
        swarm_ahead,
    })
}

fn git_error(command: &str, stderr: &[u8]) -> anyhow::Error {
    let stderr = String::from_utf8_lossy(stderr);
    anyhow!("{command} failed: {}", stderr.trim())
}

pub(crate) fn classify_relation(
    merge_base: Option<&str>,
    ahead_behind: AheadBehind,
) -> GraphRelation {
    if merge_base.is_none() {
        return GraphRelation::Unrelated;
    }

    match (ahead_behind.publication_ahead, ahead_behind.swarm_ahead) {
        (0, 0) => GraphRelation::Aligned,
        (0, _) => GraphRelation::SwarmAhead,
        (_, 0) => GraphRelation::PublicationAhead,
        _ => GraphRelation::Diverged,
    }
}

pub(crate) fn expectation_matches(
    expectation: RepoGraphExpectation,
    relation: GraphRelation,
) -> bool {
    match expectation {
        RepoGraphExpectation::Aligned => relation == GraphRelation::Aligned,
        RepoGraphExpectation::SwarmAhead => relation == GraphRelation::SwarmAhead,
        RepoGraphExpectation::PublicationAhead => relation == GraphRelation::PublicationAhead,
        RepoGraphExpectation::SwarmDescendsPublication => {
            matches!(relation, GraphRelation::Aligned | GraphRelation::SwarmAhead)
        }
        RepoGraphExpectation::PublicationDescendsSwarm => {
            matches!(
                relation,
                GraphRelation::Aligned | GraphRelation::PublicationAhead
            )
        }
        RepoGraphExpectation::NoDivergence => matches!(
            relation,
            GraphRelation::Aligned | GraphRelation::SwarmAhead | GraphRelation::PublicationAhead
        ),
    }
}

fn expectation_name(expectation: RepoGraphExpectation) -> &'static str {
    match expectation {
        RepoGraphExpectation::Aligned => "aligned",
        RepoGraphExpectation::SwarmAhead => "swarm-ahead",
        RepoGraphExpectation::PublicationAhead => "publication-ahead",
        RepoGraphExpectation::SwarmDescendsPublication => "swarm-descends-publication",
        RepoGraphExpectation::PublicationDescendsSwarm => "publication-descends-swarm",
        RepoGraphExpectation::NoDivergence => "no-divergence",
    }
}

fn expectation_failure_hint(report: &RepoGraphReport) -> &'static str {
    graph_next_action(&report.expectation, report.relation)
}

fn graph_next_action(expectation: &str, relation: GraphRelation) -> &'static str {
    match (expectation, relation) {
        ("aligned", GraphRelation::Aligned) => {
            "graph is aligned; no publication or swarm fast-forward action is needed"
        }
        ("swarm-descends-publication", GraphRelation::Aligned)
        | ("swarm-ahead", GraphRelation::Aligned)
        | ("publication-ahead", GraphRelation::Aligned)
        | ("publication-descends-swarm", GraphRelation::Aligned)
        | ("no-divergence", GraphRelation::Aligned) => {
            "graph is aligned; continue with the operating step that requested this check"
        }
        ("swarm-ahead", GraphRelation::SwarmAhead)
        | ("swarm-descends-publication", GraphRelation::SwarmAhead)
        | ("no-divergence", GraphRelation::SwarmAhead) => {
            "swarm descends from publication; open a tokmd publication PR and merge it with a merge commit before fast-forwarding swarm"
        }
        ("publication-ahead", GraphRelation::PublicationAhead)
        | ("publication-descends-swarm", GraphRelation::PublicationAhead)
        | ("no-divergence", GraphRelation::PublicationAhead) => {
            "publication descends from swarm; fast-forward tokmd-swarm/main to the publication commit before routine swarm work continues"
        }
        ("aligned", GraphRelation::SwarmAhead)
        | ("publication-ahead", GraphRelation::SwarmAhead)
        | ("publication-descends-swarm", GraphRelation::SwarmAhead) => {
            "publication has not imported the swarm head; create a tokmd publication PR and merge it with a merge commit before fast-forwarding swarm"
        }
        ("aligned", GraphRelation::PublicationAhead)
        | ("swarm-ahead", GraphRelation::PublicationAhead)
        | ("swarm-descends-publication", GraphRelation::PublicationAhead) => {
            "swarm is behind publication; fast-forward tokmd-swarm/main to the publication commit before routine swarm work continues"
        }
        (_, GraphRelation::Diverged) => {
            "publication and swarm both have unique commits; inspect the graph and use an explicit sync merge rather than a squash/content sync"
        }
        (_, GraphRelation::Unrelated) => {
            "the refs share no merge base; stop normal work and run an admin realignment instead of merging unrelated histories"
        }
        _ => "inspect the requested expectation and refs before changing either main branch",
    }
}

fn write_json(path: &Path, report: &RepoGraphReport) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, serde_json::to_string_pretty(report)?)
        .with_context(|| format!("write repo graph receipt {}", path.display()))?;
    Ok(())
}

fn print_human_report(report: &RepoGraphReport) {
    println!(
        "repo-graph: {:?} publication_ahead={} swarm_ahead={} expectation={} ok={}",
        report.relation,
        report.ahead_behind.publication_ahead,
        report.ahead_behind.swarm_ahead,
        report.expectation,
        report.ok
    );
    println!("next_action {}", report.next_action);
    println!(
        "publication {} {}",
        report.publication.name, report.publication.sha
    );
    println!("swarm {} {}", report.swarm.name, report.swarm.sha);
    if let Some(merge_base) = &report.merge_base {
        println!("merge_base {merge_base}");
    } else {
        println!("merge_base none");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AheadBehind, GraphRelation, RefReport, RepoGraphReport, alternate_publication_ref,
        classify_relation, expectation_failure_hint, expectation_matches,
    };
    use crate::cli::RepoGraphExpectation;

    fn counts(publication_ahead: u64, swarm_ahead: u64) -> AheadBehind {
        AheadBehind {
            publication_ahead,
            swarm_ahead,
        }
    }

    #[test]
    fn alternate_publication_ref_maps_public_prefix_to_publication() {
        assert_eq!(
            alternate_publication_ref("public/main").as_deref(),
            Some("publication/main")
        );
        assert_eq!(
            alternate_publication_ref("public/release").as_deref(),
            Some("publication/release")
        );
    }

    #[test]
    fn alternate_publication_ref_ignores_non_public_refs() {
        assert_eq!(alternate_publication_ref("publication/main"), None);
        assert_eq!(alternate_publication_ref("origin/main"), None);
        assert_eq!(alternate_publication_ref("HEAD"), None);
    }

    #[test]
    fn classifies_aligned_refs() {
        assert_eq!(
            classify_relation(Some("abc"), counts(0, 0)),
            GraphRelation::Aligned
        );
    }

    #[test]
    fn classifies_swarm_ahead_refs() {
        assert_eq!(
            classify_relation(Some("abc"), counts(0, 2)),
            GraphRelation::SwarmAhead
        );
    }

    #[test]
    fn classifies_publication_ahead_refs() {
        assert_eq!(
            classify_relation(Some("abc"), counts(3, 0)),
            GraphRelation::PublicationAhead
        );
    }

    #[test]
    fn classifies_diverged_refs() {
        assert_eq!(
            classify_relation(Some("abc"), counts(1, 2)),
            GraphRelation::Diverged
        );
    }

    #[test]
    fn classifies_unrelated_refs_without_merge_base() {
        assert_eq!(
            classify_relation(None, counts(1, 2)),
            GraphRelation::Unrelated
        );
    }

    #[test]
    fn aligned_expectation_requires_exact_alignment() {
        assert!(expectation_matches(
            RepoGraphExpectation::Aligned,
            GraphRelation::Aligned
        ));
        assert!(!expectation_matches(
            RepoGraphExpectation::Aligned,
            GraphRelation::SwarmAhead
        ));
    }

    #[test]
    fn ahead_expectations_require_exact_ahead_side() {
        assert!(expectation_matches(
            RepoGraphExpectation::SwarmAhead,
            GraphRelation::SwarmAhead
        ));
        assert!(!expectation_matches(
            RepoGraphExpectation::SwarmAhead,
            GraphRelation::Aligned
        ));
        assert!(!expectation_matches(
            RepoGraphExpectation::SwarmAhead,
            GraphRelation::PublicationAhead
        ));
        assert!(expectation_matches(
            RepoGraphExpectation::PublicationAhead,
            GraphRelation::PublicationAhead
        ));
        assert!(!expectation_matches(
            RepoGraphExpectation::PublicationAhead,
            GraphRelation::Aligned
        ));
        assert!(!expectation_matches(
            RepoGraphExpectation::PublicationAhead,
            GraphRelation::SwarmAhead
        ));
    }

    #[test]
    fn swarm_descends_publication_accepts_aligned_or_swarm_ahead() {
        assert!(expectation_matches(
            RepoGraphExpectation::SwarmDescendsPublication,
            GraphRelation::Aligned
        ));
        assert!(expectation_matches(
            RepoGraphExpectation::SwarmDescendsPublication,
            GraphRelation::SwarmAhead
        ));
        assert!(!expectation_matches(
            RepoGraphExpectation::SwarmDescendsPublication,
            GraphRelation::PublicationAhead
        ));
    }

    #[test]
    fn publication_descends_swarm_accepts_aligned_or_publication_ahead() {
        assert!(expectation_matches(
            RepoGraphExpectation::PublicationDescendsSwarm,
            GraphRelation::Aligned
        ));
        assert!(expectation_matches(
            RepoGraphExpectation::PublicationDescendsSwarm,
            GraphRelation::PublicationAhead
        ));
        assert!(!expectation_matches(
            RepoGraphExpectation::PublicationDescendsSwarm,
            GraphRelation::SwarmAhead
        ));
    }

    #[test]
    fn no_divergence_rejects_diverged_and_unrelated_refs() {
        assert!(expectation_matches(
            RepoGraphExpectation::NoDivergence,
            GraphRelation::PublicationAhead
        ));
        assert!(!expectation_matches(
            RepoGraphExpectation::NoDivergence,
            GraphRelation::Diverged
        ));
        assert!(!expectation_matches(
            RepoGraphExpectation::NoDivergence,
            GraphRelation::Unrelated
        ));
    }

    fn report(expectation: &str, relation: GraphRelation) -> RepoGraphReport {
        let ahead_behind = match relation {
            GraphRelation::Aligned => counts(0, 0),
            GraphRelation::SwarmAhead => counts(0, 1),
            GraphRelation::PublicationAhead => counts(1, 0),
            GraphRelation::Diverged | GraphRelation::Unrelated => counts(1, 1),
        };

        RepoGraphReport {
            schema: super::SCHEMA,
            ok: false,
            expectation: expectation.to_string(),
            relation,
            next_action: super::graph_next_action(expectation, relation),
            publication: RefReport {
                name: "publication/main".to_string(),
                sha: "publication-sha".to_string(),
            },
            swarm: RefReport {
                name: "origin/main".to_string(),
                sha: "swarm-sha".to_string(),
            },
            merge_base: Some("base-sha".to_string()),
            ahead_behind,
        }
    }

    #[test]
    fn failure_hint_names_publication_import_for_swarm_ahead() {
        let report = report("publication-descends-swarm", GraphRelation::SwarmAhead);

        let hint = expectation_failure_hint(&report);

        assert!(hint.contains("publication has not imported the swarm head"));
        assert!(hint.contains("merge commit"));
    }

    #[test]
    fn failure_hint_names_swarm_fast_forward_for_publication_ahead() {
        let report = report(
            "swarm-descends-publication",
            GraphRelation::PublicationAhead,
        );

        let hint = expectation_failure_hint(&report);

        assert!(hint.contains("swarm is behind publication"));
        assert!(hint.contains("fast-forward tokmd-swarm/main"));
    }

    #[test]
    fn failure_hint_names_admin_realignment_for_unrelated_refs() {
        let report = report("no-divergence", GraphRelation::Unrelated);

        let hint = expectation_failure_hint(&report);

        assert!(hint.contains("share no merge base"));
        assert!(hint.contains("admin realignment"));
    }

    #[test]
    fn next_action_names_no_sync_when_aligned() {
        let report = report("aligned", GraphRelation::Aligned);

        assert!(
            report
                .next_action
                .contains("no publication or swarm fast-forward")
        );
    }

    #[test]
    fn next_action_names_publication_import_when_swarm_is_validly_ahead() {
        let report = report("swarm-descends-publication", GraphRelation::SwarmAhead);

        assert!(
            report
                .next_action
                .contains("swarm descends from publication")
        );
        assert!(report.next_action.contains("publication PR"));
    }
}
