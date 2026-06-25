//! Evidence packet workflow command parser types.
//!
//! `tokmd packet generate` is the thin orchestrator over the existing
//! `analyze`, `context`, `syntax`, and `evidence-packet` surfaces. It owns no
//! new analysis model; it only coordinates the receipt-producing commands so a
//! complete `sensors/tokmd/` packet can be generated from one command.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use super::AnalysisPreset;

/// Default packet output directory, matching the evidence packet contract.
pub const DEFAULT_PACKET_DIR: &str = "sensors/tokmd";

/// Default context budget for the packet `context.md` artifact.
pub const DEFAULT_PACKET_CONTEXT_BUDGET: &str = "64000";

#[derive(Args, Debug, Clone)]
pub struct PacketArgs {
    #[command(subcommand)]
    pub command: PacketCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum PacketCommand {
    /// Generate a complete evidence packet over the existing receipts.
    Generate(PacketGenerateArgs),
}

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd packet generate --base origin/main --head HEAD src/runtime/api\n  tokmd packet generate --preset bun-ub --out sensors/tokmd --no-syntax src/runtime/api/MarkdownObject.rs"
)]
pub struct PacketGenerateArgs {
    /// Analysis preset used to generate analyze.md and analyze.json.
    #[arg(long, value_enum, default_value_t = AnalysisPreset::BunUb)]
    pub preset: AnalysisPreset,

    /// Base reference shared by every generated artifact.
    #[arg(long, default_value = "origin/main")]
    pub base: String,

    /// Head reference shared by every generated artifact.
    #[arg(long, default_value = "HEAD")]
    pub head: String,

    /// Output directory for the packet artifacts and manifest.
    #[arg(long = "out", value_name = "DIR", default_value = DEFAULT_PACKET_DIR)]
    pub out: PathBuf,

    /// Request optional syntax evidence (`syntax.json`). On by default; this
    /// flag exists so the documented workflow can pass `--syntax` explicitly.
    #[arg(long, overrides_with = "no_syntax")]
    pub syntax: bool,

    /// Skip optional syntax evidence generation.
    #[arg(long = "no-syntax", overrides_with = "syntax")]
    pub no_syntax: bool,

    /// Token budget for the context artifact.
    #[arg(long = "context-budget", default_value = DEFAULT_PACKET_CONTEXT_BUDGET)]
    pub context_budget: String,

    /// Changed paths or scoped review inputs used to generate the packet.
    #[arg(value_name = "PATH", required = true)]
    pub paths: Vec<PathBuf>,
}

impl PacketGenerateArgs {
    /// Whether optional syntax evidence should be requested.
    ///
    /// `--no-syntax` wins over the default-on `--syntax` flag.
    pub fn want_syntax(&self) -> bool {
        !self.no_syntax
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::cli::parser::{Cli, Commands};

    fn parse_generate(args: &[&str]) -> PacketGenerateArgs {
        let mut argv = vec!["tokmd", "packet", "generate"];
        argv.extend_from_slice(args);
        let cli = Cli::try_parse_from(argv).unwrap();
        match cli.command.unwrap() {
            Commands::Packet(packet) => match packet.command {
                PacketCommand::Generate(args) => args,
            },
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn generate_defaults_match_evidence_packet_contract() {
        let args = parse_generate(&["src/runtime/api"]);
        assert_eq!(args.preset, AnalysisPreset::BunUb);
        assert_eq!(args.base, "origin/main");
        assert_eq!(args.head, "HEAD");
        assert_eq!(args.out, PathBuf::from(DEFAULT_PACKET_DIR));
        assert_eq!(args.context_budget, DEFAULT_PACKET_CONTEXT_BUDGET);
        assert_eq!(args.paths, vec![PathBuf::from("src/runtime/api")]);
        assert!(args.want_syntax());
    }

    #[test]
    fn generate_no_syntax_disables_syntax() {
        let args = parse_generate(&["--no-syntax", "src/runtime/api"]);
        assert!(!args.want_syntax());
    }

    #[test]
    fn generate_explicit_syntax_keeps_syntax() {
        let args = parse_generate(&["--syntax", "src/runtime/api"]);
        assert!(args.want_syntax());
    }

    #[test]
    fn generate_requires_paths() {
        assert!(Cli::try_parse_from(["tokmd", "packet", "generate"]).is_err());
    }
}
