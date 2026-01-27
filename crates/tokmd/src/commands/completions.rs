use anyhow::Result;
use clap::CommandFactory;
use tokmd_config as cli;

pub(crate) fn handle(args: cli::CompletionsArgs) -> Result<()> {
    use clap_complete::generate;
    let mut cmd = cli::Cli::command();
    let name = cmd.get_name().to_string();
    let shell = match args.shell {
        cli::Shell::Bash => clap_complete::Shell::Bash,
        cli::Shell::Elvish => clap_complete::Shell::Elvish,
        cli::Shell::Fish => clap_complete::Shell::Fish,
        cli::Shell::Powershell => clap_complete::Shell::PowerShell,
        cli::Shell::Zsh => clap_complete::Shell::Zsh,
    };
    generate(shell, &mut cmd, name, &mut std::io::stdout());
    Ok(())
}
