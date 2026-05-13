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
        Some(cli::Commands::DocArtifacts(args)) => tasks::doc_artifacts::run(args),
        Some(cli::Commands::ProofPolicy(args)) => tasks::proof_policy::run(args),
        Some(cli::Commands::ProofObservationThresholds(args)) => {
            tasks::proof_observation_thresholds::run(args)
        }
        Some(cli::Commands::ProofObservationRunIds(args)) => {
            tasks::proof_observation_run_ids::run(args)
        }
        Some(cli::Commands::Affected(args)) => tasks::affected::run(args),
        Some(cli::Commands::Proof(args)) => tasks::proof_plan::run(args),
        Some(cli::Commands::ProofArtifactsCheck(args)) => tasks::proof_artifacts_check::run(args),
        Some(cli::Commands::ProofExecutionArtifactsCheck(args)) => {
            tasks::proof_artifacts_check::run_execution(args)
        }
        Some(cli::Commands::ProofRunArtifactsCheck(args)) => {
            tasks::proof_artifacts_check::run_proof_run(args)
        }
        Some(cli::Commands::ProofRunObservation(args)) => {
            tasks::proof_artifacts_check::run_proof_run_observation(args)
        }
        Some(cli::Commands::ProofRunObservationsSummary(args)) => {
            tasks::proof_artifacts_check::run_proof_run_observations_summary(args)
        }
        Some(cli::Commands::ProofExecutionObservation(args)) => {
            tasks::proof_artifacts_check::run_observation(args)
        }
        Some(cli::Commands::ProofExecutionObservationsSummary(args)) => {
            tasks::proof_artifacts_check::run_observations_summary(args)
        }
        Some(cli::Commands::ReviewPacketCheck(args)) => tasks::review_packet_check::run(args),
        Some(cli::Commands::VersionConsistency(args)) => tasks::version_consistency::run(args),
        Some(cli::Commands::BoundariesCheck(args)) => tasks::boundaries_check::run(args),
        Some(cli::Commands::FixtureBlobsCheck(args)) => tasks::fixture_blobs_check::run(args),
        Some(cli::Commands::Gate(args)) => tasks::gate::run(args),
        Some(cli::Commands::CiPlan(args)) => tasks::ci_plan::run(args),
        Some(cli::Commands::JulesIndex(args)) => tasks::jules_index::run(args),
        Some(cli::Commands::CheckLintPolicy(args)) => tasks::lint_policy::run(args),
        Some(cli::Commands::CoverageReceipt(args)) => tasks::coverage_receipt::run(args),
        Some(cli::Commands::CiActuals(args)) => tasks::ci_actuals::run(args),
        Some(cli::Commands::CheckFilePolicy(args)) => tasks::file_policy::run(args),
        Some(cli::Commands::CheckClippyExceptions(args)) => tasks::clippy_exceptions::run(args),
        Some(cli::Commands::CiLaneWhitelist(args)) => tasks::ci_lane_whitelist::run(args),
        Some(cli::Commands::CheckNoPanicFamily(args)) => tasks::no_panic::run_check(args),
        Some(cli::Commands::NoPanicPropose(args)) => tasks::no_panic::run_propose(args),
        Some(cli::Commands::LintFix(args)) => tasks::lint_fix::run(args),
        Some(cli::Commands::Sccache(args)) => tasks::sccache::run(args),
        Some(cli::Commands::TrimTarget(args)) => tasks::trim_target::run(args),
        Some(cli::Commands::PerfSmoke(args)) => tasks::perf_smoke::run(args),
        Some(cli::Commands::Badges(args)) => tasks::badges::run(args),
        Some(cli::Commands::RiprPr(args)) => tasks::ripr_pr::run_pr(args),
        Some(cli::Commands::RiprReviewComments(args)) => tasks::ripr_pr::run_review_comments(args),
        None => tasks::publish::run(PublishArgs::default()),
    }
}
