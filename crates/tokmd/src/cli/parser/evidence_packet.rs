//! Evidence packet manifest command parser types.

use std::path::PathBuf;

use clap::Args;
use serde::{Deserialize, Serialize};

use super::AnalysisPreset;

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
#[command(
    after_help = "Examples:\n  tokmd evidence-packet --base origin/main --head HEAD src/runtime/api\n  tokmd evidence-packet --output sensors/tokmd/manifest.json --preset bun-ub src/runtime/api/MarkdownObject.rs"
)]
pub struct EvidencePacketArgs {
    /// Analysis preset used to generate analyze.md and analyze.json.
    #[arg(long, value_enum, default_value_t = AnalysisPreset::BunUb)]
    pub preset: AnalysisPreset,

    /// Base reference used by analyze artifacts.
    #[arg(long, default_value = "origin/main")]
    pub base: String,

    /// Head reference used by analyze artifacts.
    #[arg(long, default_value = "HEAD")]
    pub head: String,

    /// Output path for the evidence packet manifest.
    #[arg(
        long,
        value_name = "PATH",
        default_value = "sensors/tokmd/manifest.json"
    )]
    pub output: PathBuf,

    /// Path to the Markdown analysis artifact.
    #[arg(long = "analyze-md", value_name = "PATH")]
    pub analyze_md: Option<PathBuf>,

    /// Path to the JSON analysis artifact.
    #[arg(long = "analyze-json", value_name = "PATH")]
    pub analyze_json: Option<PathBuf>,

    /// Path to the context Markdown artifact.
    #[arg(long = "context-md", value_name = "PATH")]
    pub context_md: Option<PathBuf>,

    /// Context budget used for the context artifact reproduction command.
    #[arg(long = "context-budget", default_value = "64000")]
    pub context_budget: String,

    /// Changed paths or scoped review inputs used to generate the packet.
    #[arg(value_name = "PATH", required = true)]
    pub paths: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::cli::parser::{Cli, Commands};

    #[test]
    fn evidence_packet_defaults_to_bun_ub_manifest_path() {
        let cli = Cli::try_parse_from(["tokmd", "evidence-packet", "src/runtime/api"]).unwrap();
        match cli.command.unwrap() {
            Commands::EvidencePacket(args) => {
                assert_eq!(args.preset, AnalysisPreset::BunUb);
                assert_eq!(args.base, "origin/main");
                assert_eq!(args.head, "HEAD");
                assert_eq!(args.output, PathBuf::from("sensors/tokmd/manifest.json"));
                assert_eq!(args.paths, vec![PathBuf::from("src/runtime/api")]);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
