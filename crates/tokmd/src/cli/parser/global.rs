//! Global CLI scan options shared by all commands.
//!
//! This module owns clap parsing for workspace-wide scan behavior and the
//! conversion into clap-free `tokmd-settings` scan options.

use clap::Args;

use super::ConfigMode;

#[derive(Args, Debug, Clone, Default)]
pub struct GlobalArgs {
    /// Exclude pattern(s) using gitignore syntax. Repeatable.
    ///
    /// Examples:
    ///   --exclude target
    ///   --exclude "**/*.min.js"
    #[arg(
        long = "exclude",
        visible_alias = "ignore",
        value_name = "PATTERN",
        global = true
    )]
    pub excluded: Vec<String>,

    /// Whether to load scan config files (`tokei.toml` / `.tokeirc`).
    #[arg(long, value_enum, value_name = "MODE", default_value_t = ConfigMode::Auto)]
    pub config: ConfigMode,

    /// Count hidden files and directories.
    #[arg(long)]
    pub hidden: bool,

    /// Don't respect ignore files (.gitignore, .ignore, etc.).
    ///
    /// Implies --no-ignore-parent, --no-ignore-dot, and --no-ignore-vcs.
    #[arg(long)]
    pub no_ignore: bool,

    /// Don't respect ignore files in parent directories.
    #[arg(long)]
    pub no_ignore_parent: bool,

    /// Don't respect .ignore and .tokeignore files (including in parent directories).
    #[arg(long)]
    pub no_ignore_dot: bool,

    /// Don't respect VCS ignore files (.gitignore, .hgignore, etc.), including in parents.
    #[arg(long, visible_alias = "no-ignore-git")]
    pub no_ignore_vcs: bool,

    /// Treat doc strings as comments (language-dependent).
    #[arg(long)]
    pub treat_doc_strings_as_comments: bool,

    /// Verbose output (repeat for more detail).
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Disable progress spinners.
    #[arg(long, global = true)]
    pub no_progress: bool,
}

impl From<&GlobalArgs> for tokmd_settings::ScanOptions {
    fn from(g: &GlobalArgs) -> Self {
        Self {
            excluded: g.excluded.clone(),
            config: g.config.into(),
            hidden: g.hidden,
            no_ignore: g.no_ignore,
            no_ignore_parent: g.no_ignore_parent,
            no_ignore_dot: g.no_ignore_dot,
            no_ignore_vcs: g.no_ignore_vcs,
            treat_doc_strings_as_comments: g.treat_doc_strings_as_comments,
        }
    }
}

impl From<GlobalArgs> for tokmd_settings::ScanOptions {
    fn from(g: GlobalArgs) -> Self {
        Self::from(&g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_args_default() {
        let g = GlobalArgs::default();
        assert!(g.excluded.is_empty());
        assert_eq!(g.config, ConfigMode::Auto);
        assert!(!g.hidden);
        assert!(!g.no_ignore);
        assert_eq!(g.verbose, 0);
    }

    #[test]
    fn global_args_to_scan_options() {
        let g = GlobalArgs {
            excluded: vec!["target".into()],
            config: ConfigMode::None,
            hidden: true,
            no_ignore: true,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: true,
            verbose: 0,
            no_progress: false,
        };
        let opts: tokmd_settings::ScanOptions = (&g).into();
        assert_eq!(opts.excluded, vec!["target"]);
        assert_eq!(opts.config, tokmd_types::ConfigMode::None);
        assert!(opts.hidden);
        assert!(opts.no_ignore);
        assert!(opts.treat_doc_strings_as_comments);
    }

    #[test]
    fn global_args_owned_to_scan_options() {
        let g = GlobalArgs {
            excluded: vec!["vendor".into()],
            config: ConfigMode::Auto,
            hidden: false,
            ..GlobalArgs::default()
        };
        let opts: tokmd_settings::ScanOptions = g.into();
        assert_eq!(opts.excluded, vec!["vendor"]);
        assert!(!opts.hidden);
    }
}
