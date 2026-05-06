## 💡 Summary
This patch hardens the `git` subprocess boundary by explicitly stripping additional potentially malicious or path-mutating environment variables. This prevents environment inheritance from overriding the path boundaries during git execution.

## 🎯 Why
In multiple crates (`tokmd-git`, `tokmd-scan`, `tokmd`), `Command::new("git")` was correctly stripping `GIT_DIR` and `GIT_WORK_TREE` but still inheriting variables like `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, and `GIT_COMMON_DIR`. If these variables are maliciously injected into the process environment, they can force the `git` subprocess to interact with the filesystem outside the intended repository bounds, bypassing directory boundary constraints (`-C`). Removing them ensures robust isolation.

## 🔎 Evidence
File paths:
- `crates/tokmd-git/src/lib.rs`
- `crates/tokmd/src/git_support.rs`
- `crates/tokmd-scan/src/walk/mod.rs`

Observed behavior:
The `git_cmd()` helper only removed `GIT_DIR` and `GIT_WORK_TREE`.

Receipt demonstrating the gap and the target surface:
```text
$ grep -rn "git_cmd" crates/
crates/tokmd-git/src/lib.rs:30:pub fn git_cmd() -> Command {
crates/tokmd/src/git_support.rs:4:pub(crate) fn git_cmd() -> Command {
crates/tokmd-scan/src/walk/mod.rs:31:fn git_cmd() -> Command {
```

## 🧭 Options considered
### Option A (recommended)
- Hardening the `git_cmd()` wrappers by explicitly adding `env_remove` for `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_COMMON_DIR`, and `GIT_CEILING_DIRECTORIES`.
- Fits the `security-boundary` profile by neutralizing path manipulation vectors inside subprocess execution across the interface surfaces.
- Trade-offs: Structure/Velocity are unaffected, Governance improves (safer defaults).

### Option B
- Only hard-code the path to the `git` executable or only sanitize FFI JSON bounds.
- Choose this if Git is known to run in a totally isolated container environment.
- Trade-offs: Still leaves local execution and CI testing vulnerable to accidental or intentional environment interference.

## ✅ Decision
Option A was chosen as it correctly and comprehensively sanitizes the environment variables known to break repository bounds in `git`.

## 🧱 Changes made (SRP)
- `crates/tokmd-git/src/lib.rs`: Added `env_remove` for additional `GIT_*` path variables.
- `crates/tokmd/src/git_support.rs`: Replicated the same subprocess hardening.
- `crates/tokmd-scan/src/walk/mod.rs`: Replicated the same subprocess hardening.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-git
...
test result: ok. 81 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.17s
```

## 🧭 Telemetry
- Change shape: Patching subprocess constructors to scrub the execution environment.
- Blast radius: API/IO. Any git command executed by tokmd will no longer accidentally pick up object/index directory overrides from the system.
- Risk class: Low risk. Legitimate usage is always bound by explicit CLI args or the default CWD, never relying on `GIT_OBJECT_DIRECTORY`.
- Rollback: Revert the added `.env_remove()` lines.
- Gates run: `cargo test -p tokmd-git`, `cargo test -p tokmd`, `cargo test -p tokmd-scan`, `cargo test -p tokmd-core`.

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_boundaries/envelope.json`
- `.jules/runs/sentinel_boundaries/decision.md`
- `.jules/runs/sentinel_boundaries/receipts.jsonl`
- `.jules/runs/sentinel_boundaries/result.json`
- `.jules/runs/sentinel_boundaries/pr_body.md`

## 🔜 Follow-ups
None.
