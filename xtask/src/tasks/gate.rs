use crate::cli::GateArgs;
use anyhow::{bail, Result};
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
        args: &["check", "--workspace", "--all-features"],
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
            "--",
            "-D",
            "warnings",
        ],
        check_args: None,
    },
    Step {
        label: "test (compile-only)",
        cmd: "cargo",
        args: &["test", "--workspace", "--all-features", "--no-run"],
        check_args: None,
    },
];

pub fn run(args: GateArgs) -> Result<()> {
    let total = STEPS.len();
    let mut passed = 0usize;

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
            bail!(
                "Step {idx}/{total} ({}) failed with exit code {}",
                step.label,
                status.code().unwrap_or(-1)
            );
        }
        passed += 1;
    }

    println!("gate: {passed}/{total} steps passed");
    Ok(())
}
