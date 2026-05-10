## 💡 Summary
Hardened the `git` subprocess boundary by stripping execution-shaping environment variables. By ensuring we drop `GIT_SSH`, `GIT_ASKPASS`, `GIT_EDITOR`, and others, we eliminate the risk of ambient execution context poisoning during `Command::new("git")` calls in the CLI.

## 🎯 Why
When `tokmd` invokes Git to calculate churn or other metrics, it originally only wiped repo-shaping environment variables (`GIT_DIR`, `GIT_WORK_TREE`, etc.). However, Git also respects many overrides (like `GIT_SSH_COMMAND` and `GIT_ASKPASS`) that can cause arbitrary execution or altered behavior based on the host environment. This represents a trust-boundary vulnerability when `tokmd` runs inside CI, bindings, or untrusted pipelines.

## 🔎 Evidence
- `crates/tokmd/src/git_support.rs` used `GIT_REPO_SHAPING_ENV` to isolate only the repo targets.
- A user or CI context setting `GIT_ASKPASS` or `GIT_PAGER` could intercept execution.
- Command receipt: `cargo test -p tokmd --lib` confirming deterministic Git environment stripping logic works.

## 🧭 Options considered
### Option A (recommended)
- Add execution-shaping environment variables to the `GIT_REPO_SHAPING_ENV` array in `git_support.rs`.
- Perfectly aligns with `Sentinel` target (subprocess/environment trust boundary).
- Minimal structural impact, high security leverage. Trade-off: might break users who specifically relied on hijacking `tokmd`'s internal git calls for debugging, which is an anti-pattern.

### Option B
- Refactor all CLI execution paths to strip environment explicitly at the OS layer.
- Better coverage if multiple binaries were involved, but much more invasive and risks breaking tools outside Git.

## ✅ Decision
Chose Option A to focus strictly on Git's known execution hooks, providing a robust execution boundary without altering how other potential processes are mapped.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/git_support.rs`: Added `GIT_SSH`, `GIT_SSH_COMMAND`, `GIT_ASKPASS`, `GIT_PAGER`, `GIT_EDITOR`, `GIT_PROXY_COMMAND`, and `GIT_EXTERNAL_DIFF` to the `GIT_REPO_SHAPING_ENV` array.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd --lib
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/lib.rs (target/debug/deps/tokmd-2a0e1a1cb22700fd)
...
test result: ok. 505 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: Internal git shelling within `tokmd` crate
- Risk class: Low, only restricts execution context.
- Rollback: Revert `git_support.rs`.
- Gates run: `cargo test -p tokmd`, `cargo clippy -- -D warnings`.

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_boundaries/envelope.json`
- `.jules/runs/sentinel_boundaries/decision.md`
- `.jules/runs/sentinel_boundaries/receipts.jsonl`
- `.jules/runs/sentinel_boundaries/result.json`
- `.jules/runs/sentinel_boundaries/pr_body.md`
- `.jules/friction/open/tokmd_git_boundary.md` (Noted the same issue exists in `crates/tokmd-git` but outside allowed shard paths)

## 🔜 Follow-ups
- The `tokmd-git` crate has the same issue, logged as a friction item to be resolved by an agent authorized on that shard.
