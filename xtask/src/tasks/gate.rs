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

const TRACKED_AGENT_RUNTIME_PATHS: &[&str] = &[
    ".claude/worktrees",
    ".claude/cache",
    ".claude/transcripts",
    ".claude/runtime",
    ".jules/worktrees",
    ".jules/runs",
    ".jules/cache",
    ".jules/transcripts",
    ".jules/runtime",
    ".jules/tmp",
];

fn ensure_no_tracked_agent_runtime_state() -> Result<()> {
    let output = Command::new("git")
        .arg("ls-files")
        .arg("--")
        .args(TRACKED_AGENT_RUNTIME_PATHS)
        .output()?;

    if !output.status.success() {
        bail!("failed to query tracked agent runtime state with git ls-files");
    }

    let tracked: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if tracked.is_empty() {
        return Ok(());
    }

    println!("Tracked agent runtime state detected:");
    for path in &tracked {
        println!("  - {path}");
    }
    println!();
    println!("Remove these paths from the Git index and re-run the gate.");
    println!("Curated `.jules/deps/**` history is allowed and intentionally excluded.");

    bail!(
        "tracked agent runtime state found in {} path(s)",
        tracked.len()
    );
}

pub fn run(args: GateArgs) -> Result<()> {
    ensure_no_tracked_agent_runtime_state()?;

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
