use anyhow::Result;
use clap::Parser;

mod cli;
mod proof;
mod tasks;

use cli::{PublishArgs, XtaskCli};

fn main() -> Result<()> {
    let cli = XtaskCli::parse();

    match cli.command {
        Some(cli::Commands::Bump(args)) => tasks::bump::run(args),
        Some(cli::Commands::Publish(args)) => tasks::publish::run(args),
        Some(cli::Commands::PublishSurface(args)) => tasks::publish_surface::run(args),
        Some(cli::Commands::Cockpit(args)) => tasks::cockpit::run(args),
        Some(cli::Commands::Docs(args)) => tasks::docs::run(args),
        Some(cli::Commands::ProofPolicy(args)) => tasks::proof_policy::run(args),
        Some(cli::Commands::Affected(args)) => tasks::affected::run(args),
        Some(cli::Commands::Proof(args)) => tasks::proof_plan::run(args),
        Some(cli::Commands::ProofArtifactsCheck(args)) => tasks::proof_artifacts_check::run(args),
        Some(cli::Commands::ProofExecutionArtifactsCheck(args)) => {
            tasks::proof_artifacts_check::run_execution(args)
        }
        Some(cli::Commands::ProofExecutionObservation(args)) => {
            tasks::proof_artifacts_check::run_observation(args)
        }
        Some(cli::Commands::ProofExecutionObservationsSummary(args)) => {
            tasks::proof_artifacts_check::run_observations_summary(args)
        }
        Some(cli::Commands::VersionConsistency(args)) => tasks::version_consistency::run(args),
        Some(cli::Commands::BoundariesCheck(args)) => tasks::boundaries_check::run(args),
        Some(cli::Commands::FixtureBlobsCheck(args)) => tasks::fixture_blobs_check::run(args),
        Some(cli::Commands::Gate(args)) => tasks::gate::run(args),
        Some(cli::Commands::CheckLintPolicy(args)) => tasks::lint_policy::run(args),
        Some(cli::Commands::CheckNoPanicFamily(args)) => tasks::no_panic::run_check(args),
        Some(cli::Commands::NoPanicPropose(args)) => tasks::no_panic::run_propose(args),
        Some(cli::Commands::LintFix(args)) => tasks::lint_fix::run(args),
        Some(cli::Commands::Sccache(args)) => tasks::sccache::run(args),
        Some(cli::Commands::TrimTarget(args)) => tasks::trim_target::run(args),
        None => tasks::publish::run(PublishArgs::default()),
    }
}
