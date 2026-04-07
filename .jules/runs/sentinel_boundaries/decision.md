# Decision

## Option A
Replace raw `std::process::Command::new("git")` calls in the core `tokmd` crate with the shared `tokmd_git::git_cmd()` wrapper.
This ensures that sensitive environment variables like `GIT_DIR` and `GIT_WORK_TREE` are explicitly scrubbed by the wrapper when spawning git subprocesses, improving boundary hardening.

## Option B
Do not modify the `tokmd` crate, but instead update the `tokmd-cockpit` and `tokmd-context-git` crates which also use raw `Command::new("git")` calls.
This would also improve security but would not address the core `tokmd` crate.

## Decision
Chose Option A. The core `tokmd` crate interfaces directly with user input and environment variables via the CLI, making it the most critical trust boundary to harden against environment contamination when spawning subprocesses.
