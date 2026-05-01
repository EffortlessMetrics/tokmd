## 🧭 Options considered

### Option A (recommended)
- what it is: Update `crates/tokmd-git/src/lib.rs` and `crates/tokmd/src/git_support.rs` to harden trust boundaries around the `git` command execution. We will remove not only `GIT_DIR` and `GIT_WORK_TREE`, but also `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES` and `GIT_CEILING_DIRECTORIES`. We will also harden `std::env::var("TOKMD_GIT_BASE_REF")` and `std::env::var("GITHUB_BASE_REF")` which currently pass unfiltered strings directly into `rev_exists` / git command arguments, risking argument injection or unexpected behaviors.
- why it fits this repo and shard: The `interfaces` shard includes `crates/tokmd` which has `git_support.rs`. The `git` command boundaries are a subprocess trust boundary.
- trade-offs: Structure: Minor changes to Command setup. Velocity: Easy to implement. Governance: Hardens environment boundary on command execution.

### Option B
- what it is: Create a learning PR instead.
- when to choose it instead: If the targeted files are missing or no viable targets exist within the primary shard paths.
- trade-offs: Misses the opportunity to provide actual security hardening patches.

## ✅ Decision
Option A. The `crates/tokmd/src/git_support.rs` and `crates/tokmd-git/src/lib.rs` files directly execute the `git` subcommand and pass unsanitized environment variable values into git arguments. Hardening the environment cleanup on `Command::new("git")` and validating the branch ref bounds is a strong "subprocess / environment boundary hardening" candidate.
