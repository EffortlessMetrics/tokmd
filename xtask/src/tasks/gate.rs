use crate::cli::GateArgs;
use anyhow::{Result, bail};
use std::process::Command;

struct Step {
    label: &'static str,
    cmd: &'static str,
    args: &'static [&'static str],
    check_args: Option<&'static [&'static str]>,
}

const STEPS: &[Step] = &[
    Step {
        label: "fmt",
        cmd: "cargo",
        args: &["fmt", "--all"],
        check_args: Some(&["fmt", "--all", "--", "--check"]),
    },
    Step {
        label: "check (warm graph)",
        cmd: "cargo",
        args: &[
            "check",
            "--workspace",
            "--all-features",
            "--exclude",
            "tokmd-python",
        ],
        check_args: None,
    },
    Step {
        label: "clippy",
        cmd: "cargo",
        args: &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--exclude",
            "tokmd-python",
            "--",
            "-D",
            "warnings",
        ],
        check_args: None,
    },
    Step {
        label: "test (compile-only)",
        cmd: "cargo",
        args: &[
            "test",
            "--workspace",
            "--all-features",
            "--exclude",
            "tokmd-python",
            "--no-run",
        ],
        check_args: None,
    },
];

pub fn run(args: GateArgs) -> Result<()> {
    let total = STEPS.len();
    let mut failures = Vec::new();

    for (i, step) in STEPS.iter().enumerate() {
        let idx = i + 1;
        let effective_args = if args.check {
            step.check_args.unwrap_or(step.args)
        } else {
            step.args
        };

        println!("[{idx}/{total}] {}", step.label);

        let status = Command::new(step.cmd).args(effective_args).status()?;

        if !status.success() {
            println!("   \u{274C} Step {} ({}) failed", idx, step.label);
            failures.push((step.label, status.code().unwrap_or(-1)));
        } else {
            println!("   \u{2705} Step {} ({}) passed", idx, step.label);
        }
    }

    let passed = total - failures.len();
    println!("\ngate result: {passed}/{total} steps passed");

    if !failures.is_empty() {
        println!("\nFailures:");
        for (label, code) in &failures {
            println!("  - {label} (exit code: {code})");
        }

        if args.check {
            println!(
                "\nTip: Run 'cargo xtask gate' (without --check) to auto-fix formatting issues."
            );
        }

        bail!("quality gate failed with {} failure(s)", failures.len());
    }

    Ok(())
}
