# Option A: Fix Cargo Subprocesses in tokmd-cockpit
- **What it is:** Create a `cargo_cmd` factory function to harden cargo subprocess execution by unsetting wrappers like `RUSTC_WRAPPER` to avoid environment poisoning, then update `tokmd-cockpit/src/supply_chain.rs` and `gates/contracts.rs` to use it.
- **Why it fits this repo and shard:** It provides real trust boundary hardening for subprocess execution which matches the Gate Profile. However, it violates the Shard path constraints (modifying `crates/tokmd-cockpit` which is out of bounds).
- **Trade-offs:**
  - *Structure:* Better security.
  - *Velocity:* Quick fix.
  - *Governance:* Violates shard boundary rules.

# Option B: Produce a Learning PR (Recommended)
- **What it is:** Instead of modifying `tokmd-cockpit` and violating the shard constraint, or implementing a fake/weak fix inside `tokmd-core`, produce a learning PR. The finding regarding Cargo subprocesses in `tokmd-cockpit` is recorded as a friction item.
- **When to choose it instead:** When the strongest target found is outside the assigned shard and no honest code patch is justified inside the shard.
- **Trade-offs:** Doesn't ship a code fix, but accurately aligns with constraints and correctly escalates the issue via friction item.

## ✅ Decision
Option B. The strongest hardening target discovered (un-isolated `cargo` subprocesses) is located in `crates/tokmd-cockpit`, which falls outside the allowed shard paths (`tokmd-core`, `tokmd-config`, `tokmd`). According to strict instructions, we must not chase out-of-shard targets. Since no stronger, viable boundary-hardening target exists inside the primary shard (test-only panic cleanup is anti-drift unless no other target exists), we will produce a Learning PR and record the friction.
