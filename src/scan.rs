use std::path::PathBuf;
use anyhow::Result;
use tokei::{Config, Languages};

use crate::cli::{ConfigMode, GlobalArgs};

pub fn scan(paths: &[PathBuf], args: &GlobalArgs) -> Result<Languages> {
    let mut cfg = match args.config {
        ConfigMode::Auto => Config::from_config_files(),
        ConfigMode::None => Config::default(),
    };

    // Only override config file settings when the user explicitly asked for it.
    if args.hidden {
        cfg.hidden = Some(true);
    }
    if args.no_ignore {
        cfg.no_ignore = Some(true);
        cfg.no_ignore_dot = Some(true);
        cfg.no_ignore_parent = Some(true);
        cfg.no_ignore_vcs = Some(true);
    }
    if args.no_ignore_dot {
        cfg.no_ignore_dot = Some(true);
    }
    if args.no_ignore_parent {
        cfg.no_ignore_parent = Some(true);
    }
    if args.no_ignore_vcs {
        cfg.no_ignore_vcs = Some(true);
    }
    if args.treat_doc_strings_as_comments {
        cfg.treat_doc_strings_as_comments = Some(true);
    }

    let ignores: Vec<&str> = args.excluded.iter().map(|s| s.as_str()).collect();

    let mut languages = Languages::new();
    languages.get_statistics(paths, &ignores, &cfg);

    // Tokei's get_statistics doesn't return an Result, it just logs errors to stderr if it can't read files.
    // However, if the paths provided don't exist, it might just return empty stats.
    // We should probably check if we got *anything* back if the user provided specific paths,
    // but for now, we trust tokei's behavior of "best effort".

    Ok(languages)
}
