## 💡 Summary
Added `#![forbid(unsafe_code)]` to the primary interface crates (`tokmd`, `tokmd-core`, and `tokmd-config`) to statically guarantee memory safety at critical security boundaries.

## 🎯 Why
The `security-boundary` gate profile demands trust-boundary hardening and FFI/CLI input validation. The simplest and most powerful tool Rust offers to ensure there is no memory corruption at these boundaries (e.g., `tokmd-core` FFI) is to forbid `unsafe` code structurally across these crates, fulfilling the "unsafe minimization / justification" goal.

## 🔎 Evidence
- `crates/tokmd/src/lib.rs`
- `crates/tokmd/src/bin/tokmd.rs`
- `crates/tokmd/src/bin/tok.rs`
- `crates/tokmd-core/src/lib.rs`
- `crates/tokmd-config/src/lib.rs`
- Observed behavior: None of these boundary crates previously had `#![forbid(unsafe_code)]`.
- Receipt:
```text
sed -i '1i #![forbid(unsafe_code)]' crates/tokmd-core/src/lib.rs
cargo check -p tokmd -p tokmd-core -p tokmd-config
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Expand `#![forbid(unsafe_code)]` to Boundary Crates (`tokmd`, `tokmd-core`, `tokmd-config`).
- why it fits this repo and shard: Directly fulfills "unsafe minimization / justification" target for the `interfaces` shard.
- trade-offs: Structure is improved by making security explicit, Velocity is fast, Governance easily prevents future regressions.

### Option B
- what it is: Fix subprocess isolation by removing `std::process::Command` in `tokmd/src/git_support.rs`.
- when to choose it instead: If preventing environment leakage to child processes is a higher priority than absolute memory safety.
- trade-offs: Structure centralizes logic, Velocity is moderate refactoring, Governance requires tracking down all `Command` usages.

## ✅ Decision
Option A is chosen because it directly satisfies the Sentinel mandate "unsafe minimization / justification" in the explicitly assigned shard (interfaces: `tokmd`, `tokmd-core`, `tokmd-config`) by structurally forbidding `unsafe` at the rust compiler level in the very crates that receive untrusted data from the FFI boundary, configs, and CLI.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/lib.rs`: Added `#![forbid(unsafe_code)]`
- `crates/tokmd/src/bin/tokmd.rs`: Added `#![forbid(unsafe_code)]`
- `crates/tokmd/src/bin/tok.rs`: Added `#![forbid(unsafe_code)]`
- `crates/tokmd-core/src/lib.rs`: Added `#![forbid(unsafe_code)]`
- `crates/tokmd-config/src/lib.rs`: Added `#![forbid(unsafe_code)]`

## 🧪 Verification receipts
```text
sed -i '1i #![forbid(unsafe_code)]' crates/tokmd/src/lib.rs
sed -i '1i #![forbid(unsafe_code)]' crates/tokmd/src/bin/tokmd.rs
sed -i '1i #![forbid(unsafe_code)]' crates/tokmd/src/bin/tok.rs
sed -i '1i #![forbid(unsafe_code)]' crates/tokmd-core/src/lib.rs
sed -i '1i #![forbid(unsafe_code)]' crates/tokmd-config/src/lib.rs
cargo check -p tokmd -p tokmd-core -p tokmd-config
cargo test -p tokmd -p tokmd-core -p tokmd-config
```

## 🧭 Telemetry
- Change shape: Attribute addition.
- Blast radius: API / CI compatibility (stops future unsafe code).
- Risk class + why: Low, strict compile-time check with zero runtime effect on current code.
- Rollback: Revert the attribute lines.
- Gates run: `cargo check`, `cargo clippy`, `cargo test` on affected crates.

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_boundaries/envelope.json`
- `.jules/runs/sentinel_boundaries/decision.md`
- `.jules/runs/sentinel_boundaries/receipts.jsonl`
- `.jules/runs/sentinel_boundaries/result.json`
- `.jules/runs/sentinel_boundaries/pr_body.md`

## 🔜 Follow-ups
None.
