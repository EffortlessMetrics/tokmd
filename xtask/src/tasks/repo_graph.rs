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
        bail!(
            "repo graph expectation {} was not met: relation {:?}, publication_ahead={}, swarm_ahead={}",
            report.expectation,
            report.relation,
            report.ahead_behind.publication_ahead,
            report.ahead_behind.swarm_ahead
        )
    }
}

fn repo_graph_report(args: &RepoGraphArgs) -> Result<RepoGraphReport> {
    let publication_sha = rev_parse(&args.publication)?;
    let swarm_sha = rev_parse(&args.swarm)?;
    let merge_base = merge_base(&args.publication, &args.swarm)?;
    let ahead_behind = ahead_behind(&args.publication, &args.swarm)?;
    let relation = classify_relation(merge_base.as_deref(), ahead_behind);
    let ok = expectation_matches(args.expect, relation);

    Ok(RepoGraphReport {
        schema: SCHEMA,
        ok,
        expectation: expectation_name(args.expect).to_string(),
        relation,
        publication: RefReport {
            name: args.publication.clone(),
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
        RepoGraphExpectation::SwarmDescendsPublication => "swarm-descends-publication",
        RepoGraphExpectation::PublicationDescendsSwarm => "publication-descends-swarm",
        RepoGraphExpectation::NoDivergence => "no-divergence",
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
    use super::{AheadBehind, GraphRelation, classify_relation, expectation_matches};
    use crate::cli::RepoGraphExpectation;

    fn counts(publication_ahead: u64, swarm_ahead: u64) -> AheadBehind {
        AheadBehind {
            publication_ahead,
            swarm_ahead,
        }
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
}
