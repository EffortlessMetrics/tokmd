## 💡 Summary
Learning PR. Attempted to execute fuzz targets around interface parsing, but `cargo fuzz` is blocked by the lack of a nightly toolchain in the default environment. Created a friction item instead of forcing a fake fix.

## 🎯 Why
Fuzzing config/input surfaces is the primary goal, but environment blocks like missing nightly toolchain prevent execution. Documenting this friction ensures future Fuzzer runs or environment maintainers address the gap.

## 🔎 Evidence
- file path(s): `fuzz/fuzz_targets/fuzz_policy_toml.rs`
- observed behavior / finding: `cargo fuzz` cannot build targets because `-Zsanitizer=address` requires the nightly compiler.
- command receipt:
```text
error: the option `Z` is only accepted on the nightly compiler
help: consider switching to a nightly toolchain: `rustup default nightly`
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Record ASAN/nightly friction and create a learning PR.
- why it fits this repo and shard: Directly follows the memory instruction for handling `cargo fuzz` environmental blocks without hallucinating out-of-scope work.
- trade-offs: Structure / Velocity / Governance - Slows immediate patch velocity, but improves governance and learning.

### Option B
- what it is: Attempt to write proptests without a clear invariant in the config files.
- when to choose it instead: If a clear mathematical invariant was obvious.
- trade-offs: Risks adding arbitrary proptests that don't effectively prove correctness just to claim a patch.

## ✅ Decision
Chosen Option A to stay honest to the environment blocker and follow the Fuzzer persona fallback rules.

## 🧱 Changes made (SRP)
- `.jules/friction/open/FRIC-20260428-001.md`

## 🧪 Verification receipts
```text
cargo fuzz run fuzz_policy_toml -- -max_total_time=10
error: the option `Z` is only accepted on the nightly compiler
```

## 🧭 Telemetry
- Change shape: Documentation addition
- Blast radius: None
- Risk class + why: None, learning PR
- Rollback: Delete the friction item
- Gates run: `cargo check`

## 🗂️ .jules artifacts
- `.jules/runs/20260428130048-fuzzer/envelope.json`
- `.jules/runs/20260428130048-fuzzer/decision.md`
- `.jules/runs/20260428130048-fuzzer/receipts.jsonl`
- `.jules/runs/20260428130048-fuzzer/result.json`
- `.jules/runs/20260428130048-fuzzer/pr_body.md`
- `.jules/friction/open/FRIC-20260428-001.md`

## 🔜 Follow-ups
- Mentioned in FRIC-20260428-001: Evaluate updating the sandbox image to include rustup nightly for fuzzing.
