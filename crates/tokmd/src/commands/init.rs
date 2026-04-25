use crate::cli;
use anyhow::Result;
use tokmd_scan as tokeignore;

#[cfg(feature = "ui")]
use crate::interactive::{self, wizard};
#[cfg(feature = "ui")]
use anyhow::Context;
#[cfg(feature = "ui")]
use std::fs;

pub(crate) fn handle(args: cli::InitArgs) -> Result<()> {
    // Non-interactive modes: print or explicit non-interactive flag or no ui feature
    #[cfg(not(feature = "ui"))]
    let use_wizard = false;
    #[cfg(feature = "ui")]
    let use_wizard =
        !args.print && !args.non_interactive && interactive::tty::should_be_interactive();

    if !use_wizard {
        let tokeignore_args = to_tokeignore_args(&args);
        if let Some(path) = tokeignore::init_tokeignore(&tokeignore_args)? {
            // Friendly success message
            let template_name = format!("{:?}", args.template).to_lowercase();
            eprintln!(
                "Initialized {} using '{}' template.",
                path.display(),
                template_name
            );

            if matches!(args.template, cli::InitProfile::Default) {
                eprintln!(
                    "Hint: Use --template <NAME> for specific defaults (rust, node, python...)."
                );
            }

            eprintln!("Ready! Run 'tokmd' to scan your code.");
        }
        return Ok(());
    }

    // Run interactive wizard (only available with ui feature)
    #[cfg(feature = "ui")]
    {
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
                    let modified_tokeignore_args = to_tokeignore_args(&modified_args);
                    tokeignore::init_tokeignore(&modified_tokeignore_args)?;
                    eprintln!("Created .tokeignore");
                }

                // Write tokmd.toml if requested
                if result.write_config {
                    let config_path = args.dir.join("tokmd.toml");

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

fn to_tokeignore_args(args: &cli::InitArgs) -> tokeignore::InitArgs {
    let template = match args.template {
        cli::InitProfile::Default => tokeignore::InitProfile::Default,
        cli::InitProfile::Rust => tokeignore::InitProfile::Rust,
        cli::InitProfile::Node => tokeignore::InitProfile::Node,
        cli::InitProfile::Mono => tokeignore::InitProfile::Mono,
        cli::InitProfile::Python => tokeignore::InitProfile::Python,
        cli::InitProfile::Go => tokeignore::InitProfile::Go,
        cli::InitProfile::Cpp => tokeignore::InitProfile::Cpp,
    };

    tokeignore::InitArgs {
        dir: args.dir.clone(),
        force: args.force,
        print: args.print,
        template,
        non_interactive: args.non_interactive,
    }
}
