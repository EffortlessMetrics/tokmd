use crate::cli::FixtureBlobsCheckArgs;
use crate::cli::GateArgs;
use crate::tasks::build_guard::{ScopedTempDir, ensure_min_free_space};
use crate::tasks::fixture_blobs_check;
use crate::tasks::workspace::run_workspace_fmt;
use anyhow::{Result, bail};
use std::process::Command;

struct Step {
    label: &'static str,
    cmd: &'static str,
    args: &'static [&'static str],
    check_args: Option<&'static [&'static str]>,
    use_workspace_fmt: bool,
}

const STEPS: &[Step] = &[
    Step {
        label: "fmt",
        cmd: "cargo",
        args: &["fmt", "--all"],
        check_args: Some(&["fmt", "--all", "--", "--check"]),
        use_workspace_fmt: true,
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
        use_workspace_fmt: false,
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
        use_workspace_fmt: false,
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
        use_workspace_fmt: false,
    },
];

const TRACKED_AGENT_RUNTIME_PATHS: &[&str] = &[
    ".claude/worktrees",
    ".claude/cache",
    ".claude/transcripts",
    ".claude/runtime",
    ".jules/worktrees",
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
    fixture_blobs_check::run(FixtureBlobsCheckArgs::default())?;

    let ephemeral_target = if std::env::var_os("CARGO_TARGET_DIR").is_none() {
        let dir = ScopedTempDir::new("gate-target")?;
        println!("gate: using disposable target dir {}", dir.path().display());
        Some(dir)
    } else {
        None
    };

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

        let exit_code = if step.use_workspace_fmt {
            match run_workspace_fmt(args.check) {
                Ok(()) => 0,
                Err(error) => {
                    eprintln!("{error:#}");
                    1
                }
            }
        } else {
            ensure_min_free_space(
                ephemeral_target
                    .as_ref()
                    .map(ScopedTempDir::path)
                    .unwrap_or(std::path::Path::new(".")),
                step.label,
            )?;

            let mut command = Command::new(step.cmd);
            command.args(effective_args);
            command.env("CARGO_INCREMENTAL", "0");
            if let Some(dir) = ephemeral_target.as_ref() {
                command.env("CARGO_TARGET_DIR", dir.path());
            }
            if cfg!(windows) && step.label == "test (compile-only)" {
                // Windows keeps the running xtask binary locked, so compile
                // the rest of the workspace here and let xtask's own tests
                // cover the binary crate separately.
                command.args(["--exclude", "xtask"]);
            }

            let status = command.status()?;
            if status.success() {
                0
            } else {
                status.code().unwrap_or(-1)
            }
        };

        if exit_code != 0 {
            println!("   ❌ Step {} ({}) failed", idx, step.label);
            failures.push((step.label, exit_code));
        } else {
            println!("   ✅ Step {} ({}) passed", idx, step.label);
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
