//! `tokmd render` CLI parser types.

use std::path::PathBuf;

use clap::{Args, ValueEnum};

/// Audience-specific packet presets for Bun UB manual-candidate bundles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PacketRenderPreset {
    #[value(name = "bun-ub-handoff")]
    BunUbHandoff,
    #[value(name = "bun-ub-pr-body")]
    BunUbPrBody,
    #[value(name = "bun-ub-ledger-note")]
    BunUbLedgerNote,
    #[value(name = "bun-ub-review-map")]
    BunUbReviewMap,
    #[value(name = "bun-ub-next-pick")]
    BunUbNextPick,
}

impl PacketRenderPreset {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BunUbHandoff => "bun-ub-handoff",
            Self::BunUbPrBody => "bun-ub-pr-body",
            Self::BunUbLedgerNote => "bun-ub-ledger-note",
            Self::BunUbReviewMap => "bun-ub-review-map",
            Self::BunUbNextPick => "bun-ub-next-pick",
        }
    }
}

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd render --from-packets ./bundle --preset bun-ub-handoff\n  tokmd render --from-packets ./bundle --preset bun-ub-pr-body --output handoff.md"
)]
pub struct RenderArgs {
    /// Packet bundle directory containing `tokmd-packets.json`.
    #[arg(long = "from-packets", value_name = "DIR")]
    pub from_packets: PathBuf,

    /// Audience-specific packet preset to render.
    #[arg(long, value_enum)]
    pub preset: PacketRenderPreset,

    /// Optional output file. Prints to stdout when omitted.
    #[arg(long, short, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::cli::parser::{Cli, Commands};

    #[test]
    fn render_requires_from_packets_and_preset() {
        let cli = Cli::try_parse_from([
            "tokmd",
            "render",
            "--from-packets",
            "./bundle",
            "--preset",
            "bun-ub-handoff",
        ])
        .unwrap();
        match cli.command.unwrap() {
            Commands::Render(args) => {
                assert_eq!(args.from_packets, PathBuf::from("./bundle"));
                assert_eq!(args.preset, PacketRenderPreset::BunUbHandoff);
                assert!(args.output.is_none());
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
