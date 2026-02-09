use anyhow::Result;
use tokmd_config as cli;
use tokmd_tokeignore as tokeignore;

#[cfg(feature = "ui")]
use crate::interactive::{self, wizard};
#[cfg(feature = "ui")]
use anyhow::Context;
#[cfg(feature = "ui")]
use std::fs;

pub(crate) fn handle(args: cli::InitArgs) -> Result<()> {
    let target_dir = args
        .path
        .clone()
        .or_else(|| args.dir.clone())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    // Non-interactive modes: print or explicit non-interactive flag or no ui feature
    #[cfg(not(feature = "ui"))]
    let use_wizard = false;
    #[cfg(feature = "ui")]
    let use_wizard =
        !args.print && !args.non_interactive && interactive::tty::should_be_interactive();

    if !use_wizard {
        return tokeignore::init_tokeignore(&args).map(|_| ());
    }

    // Run interactive wizard (only available with ui feature)
    #[cfg(feature = "ui")]
    {
        match wizard::run_init_wizard(&target_dir)? {
            Some(result) => {
                // Write .tokeignore if requested
                if result.write_tokeignore {
                    let profile = wizard::project_type_to_profile(result.project_type);
                    let modified_args = cli::InitArgs {
                        path: Some(target_dir.clone()),
                        dir: None,
                        force: args.force,
                        print: false,
                        template: profile,
                        non_interactive: true,
                    };
                    tokeignore::init_tokeignore(&modified_args)?;
                    eprintln!("Created .tokeignore");
                }

                // Write tokmd.toml if requested
                if result.write_config {
                    let config_path = target_dir.join("tokmd.toml");

                    if config_path.exists() && !args.force {
                        eprintln!("tokmd.toml already exists. Use --force to overwrite.");
                    } else {
                        let config_content = wizard::generate_toml_config(&result)?;
                        fs::write(&config_path, config_content).with_context(|| {
                            format!("Failed to write {}", config_path.display())
                        })?;
                        eprintln!("Created tokmd.toml");
                    }
                }

                eprintln!("\nInit complete! Run 'tokmd' to scan your project.");
            }
            None => {
                eprintln!("Init cancelled.");
            }
        }
    }

    Ok(())
}
