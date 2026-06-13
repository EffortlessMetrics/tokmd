## 💡 Summary
This is a learning PR. Investigated subprocess execution boundaries in the `interfaces` shard. Found that the strongest trust-boundary target (unescaped `cargo` invocations that inherit execution wrappers) resides in `tokmd-cockpit`, which falls outside the allowed paths. Therefore, no code patch was made, and the finding was escalated as friction.

## 🎯 Why
Subprocesses must explicitly drop execution-shaping environment variables (like `RUSTC_WRAPPER` for cargo, or `GIT_SSH` for git) so that ambient hooks cannot execute arbitrary host binaries or poison the build output. `tokmd-cockpit` currently calls `Command::new("cargo")` directly, ignoring this security boundary.

## 🔎 Evidence
- `crates/tokmd-cockpit/src/supply_chain.rs`
- `crates/tokmd-cockpit/src/gates/contracts.rs`
- The `cargo audit` and `cargo semver-checks` commands are invoked directly.

## 🧭 Options considered
### Option A
- what it is: Create a `cargo_cmd` factory function to harden cargo subprocess execution by unsetting wrappers like `RUSTC_WRAPPER` to avoid environment poisoning, then update `tokmd-cockpit/src/supply_chain.rs` and `gates/contracts.rs` to use it.
- why it fits this repo and shard: It provides real trust boundary hardening for subprocess execution which matches the Gate Profile. However, it violates the Shard path constraints (modifying `crates/tokmd-cockpit` which is out of bounds).
- trade-offs: Structure: Better security / Velocity: Quick fix / Governance: Violates shard boundary rules.

### Option B (recommended)
- what it is: Instead of modifying `tokmd-cockpit` and violating the shard constraint, or implementing a fake/weak fix inside `tokmd-core`, produce a learning PR. The finding regarding Cargo subprocesses in `tokmd-cockpit` is recorded as a friction item.
- when to choose it instead: When the strongest target found is outside the assigned shard and no honest code patch is justified inside the shard.
- trade-offs: Doesn't ship a code fix, but accurately aligns with constraints and correctly escalates the issue via friction item.

## ✅ Decision
Option B. The strongest hardening target discovered (un-isolated `cargo` subprocesses) is located in `crates/tokmd-cockpit`, which falls outside the allowed shard paths (`tokmd-core`, `tokmd-config`, `tokmd`). According to strict instructions, we must not chase out-of-shard targets. Since no stronger, viable boundary-hardening target exists inside the primary shard (test-only panic cleanup is anti-drift unless no other target exists), we will produce a Learning PR and record the friction.

## 🧱 Changes made (SRP)
- None. Generated learning PR.

## 🧪 Verification receipts
```text
{"command": "grep -rI \"Command::new\" crates/tokmd*", "output": "crates/tokmd-cockpit/src/supply_chain.rs:    let check = Command::new(\"cargo\").arg(\"audit\").arg(\"--version\").output();\ncrates/tokmd-cockpit/src/supply_chain.rs:    let audit_output = Command::new(\"cargo\")\ncrates/tokmd-cockpit/src/gates/contracts.rs:    let available = Command::new(\"cargo\")\ncrates/tokmd-cockpit/src/gates/contracts.rs:    let output = match Command::new(\"cargo\")"}
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None (API / IO / docs / schema / concurrency / compatibility / dependencies)
- Risk class: Low
- Rollback: N/A
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_boundaries/envelope.json`
- `.jules/runs/sentinel_boundaries/decision.md`
- `.jules/runs/sentinel_boundaries/receipts.jsonl`
- `.jules/runs/sentinel_boundaries/result.json`
- `.jules/runs/sentinel_boundaries/pr_body.md`
- Friction item added: `.jules/friction/open/FRIC-20231024-001.md`

## 🔜 Follow-ups
- Address FRIC-20231024-001: Extract a `cargo_cmd` utility (analogous to `tokmd_git::git_cmd`) to securely execute cargo subprocesses across `tokmd-cockpit` without breaking structural environment expectations like `CARGO_HOME`.
