use anyhow::{Context, Result};
use std::fs;
use tokmd_config as cli;
use tokmd_tokeignore as tokeignore;

use crate::interactive::{self, wizard};

pub(crate) fn handle(args: cli::InitArgs) -> Result<()> {
    // Non-interactive modes: print or explicit non-interactive flag
    if args.print || args.non_interactive || !interactive::should_be_interactive() {
        return tokeignore::init_tokeignore(&args).map(|_| ());
    }

    // Run interactive wizard
    match wizard::run_init_wizard(&args.dir)? {
        Some(result) => {
            // Write .tokeignore if requested
            if result.write_tokeignore {
                let profile = wizard::project_type_to_profile(result.project_type);
                let modified_args = cli::InitArgs {
                    dir: args.dir.clone(),
                    force: args.force,
                    print: false,
                    template: profile,
                    non_interactive: true,
                };
                tokeignore::init_tokeignore(&modified_args)?;
                println!("Created .tokeignore");
            }

            // Write tokmd.toml if requested
            if result.write_config {
                let config_path = args.dir.join("tokmd.toml");

                if config_path.exists() && !args.force {
                    eprintln!(
                        "tokmd.toml already exists. Use --force to overwrite."
                    );
                } else {
                    let config_content = wizard::generate_toml_config(&result);
                    fs::write(&config_path, config_content)
                        .with_context(|| format!("Failed to write {}", config_path.display()))?;
                    println!("Created tokmd.toml");
                }
            }

            println!("\nInit complete! Run 'tokmd' to scan your project.");
        }
        None => {
            eprintln!("Init cancelled.");
        }
    }

    Ok(())
}
