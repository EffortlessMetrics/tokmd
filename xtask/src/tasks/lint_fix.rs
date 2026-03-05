use crate::cli::LintFixArgs;
use anyhow::{Result, bail};
use std::process::Command;

pub fn run(args: LintFixArgs) -> Result<()> {
    // Step 1: fmt
    if args.check {
        println!("[1/2] cargo fmt --all -- --check");
        let status = Command::new("cargo")
            .args(["fmt", "--all", "--", "--check"])
            .status()?;
        if !status.success() {
            bail!("fmt check failed");
        }
    } else {
        println!("[1/{}] cargo fmt --all", if args.no_clippy { 1 } else { 3 });
        let status = Command::new("cargo").args(["fmt", "--all"]).status()?;
        if !status.success() {
            bail!("fmt failed");
        }
    }

    if args.no_clippy {
        println!("lint-fix: clippy skipped (--no-clippy)");
        println!("lint-fix: all steps passed");
        return Ok(());
    }

    // Step 2 (non-check): clippy --fix (best-effort)
    if !args.check {
        println!("[2/3] cargo clippy --fix (best-effort)");
        let status = Command::new("cargo")
            .args([
                "clippy",
                "--fix",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--allow-dirty",
                "--allow-staged",
            ])
            .status()?;
        if !status.success() {
            eprintln!("warning: clippy --fix returned non-zero (continuing to verify)");
        }
    }

    // Step 3 (or 2 in check mode): strict clippy verify
    let verify_step = if args.check { 2 } else { 3 };
    let total = verify_step;
    println!("[{verify_step}/{total}] cargo clippy (verify)");
    let status = Command::new("cargo")
        .args([
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .status()?;
    if !status.success() {
        bail!("clippy verify failed");
    }

    println!("lint-fix: all steps passed");
    Ok(())
}
