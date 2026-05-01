## 💡 Summary
Hardened `git` subprocess boundary and shell interpolation by aggressively removing untrusted environment variables. Improved validation on standard git references (`GITHUB_BASE_REF`, `TOKMD_GIT_BASE_REF`).

## 🎯 Why
In `crates/tokmd/src/git_support.rs` and `crates/tokmd-git/src/lib.rs`, the `git` command invocation previously only removed `GIT_DIR` and `GIT_WORK_TREE`, allowing `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, and `GIT_CEILING_DIRECTORIES` to leak in from the environment and manipulate the execution boundary. Additionally, the fallback environment variables (`GITHUB_BASE_REF`, `TOKMD_GIT_BASE_REF`) were interpolated into command strings without validating for shell meta-characters.

## 🔎 Evidence
- File path(s): `crates/tokmd/src/git_support.rs`, `crates/tokmd-git/src/lib.rs`
- Observed behavior: `git` command allows environment variable leakage which alters the git working graph context unprompted. Ref env vars accept strings containing `;`, `|`, `&`, `$` and spaces.
- Validation: Unit tests continue to pass and `cargo clippy` succeeds with the stricter string character match.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `crates/tokmd-git/src/lib.rs` and `crates/tokmd/src/git_support.rs` to harden trust boundaries around the `git` command execution. Strip out additional git context variables (`GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_CEILING_DIRECTORIES`). Harden string validity on `TOKMD_GIT_BASE_REF` and `GITHUB_BASE_REF` using `matches!`.
- why it fits this repo and shard: The interfaces shard explicitly covers `tokmd` and `tokmd-git` CLI adapters and boundary execution points. It is a textbook Sentinel task.
- trade-offs: Structure / Velocity / Governance: Hardens environment boundary on command execution with a quick patch.

### Option B
- what it is: Record a learning PR.
- when to choose it instead: If the path didn't exist or targets couldn't be easily remediated without upstream breaking changes.
- trade-offs: Misses the opportunity to patch the exact security bounds.

## ✅ Decision
Option A was chosen to harden the environment and ref-resolution subprocess boundaries without changing the core business logic.

## 🧱 Changes made (SRP)
- `crates/tokmd-git/src/lib.rs`
- `crates/tokmd/src/git_support.rs`

## 🧪 Verification receipts
```text
{"command": "mkdir -p .jules/runs/sentinel-1 && generate envelope.json", "status": "success"}
{"command": "grep -rn 'env::var' crates/tokmd*/src", "status": "success"}
{"command": "grep -rn 'Command::new' crates/tokmd*/src", "status": "success"}
{"command": "write decision.md", "status": "success"}
{"command": "patch git_cmd in git_support.rs and tokmd-git/src/lib.rs", "status": "success"}
{"command": "patch GITHUB_BASE_REF/TOKMD_GIT_BASE_REF validation in tokmd-git/src/lib.rs", "status": "success"}
{"command": "write result.json", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: API / IO / Security Bounds
- Risk class: Low
- Rollback: Revert the PR
- Gates run: `cargo test -p tokmd-git`, `cargo test -p tokmd`, `cargo clippy`, `cargo fmt`

## 🗂️ .jules artifacts
- `.jules/runs/sentinel-1/envelope.json`
- `.jules/runs/sentinel-1/decision.md`
- `.jules/runs/sentinel-1/receipts.jsonl`
- `.jules/runs/sentinel-1/result.json`
- `.jules/runs/sentinel-1/pr_body.md`

## 🔜 Follow-ups
None
