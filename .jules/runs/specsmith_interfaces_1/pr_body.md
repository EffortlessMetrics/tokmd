## 💡 Summary
Migrated unguarded `unwrap()` calls in `tokmd-core` and `tokmd-config` logic to `expect()` with highly descriptive error messages documenting static invariants.

## 🎯 Why
Unguarded unwraps fail silently with generic panics if an assumption breaks. Replacing them with `.expect()` containing an explicit reason for why the invariant holds improves debugging when invariants are violated, increasing safety around config and enum (de)serialization boundaries per the Sentinel pattern. This aligns with the Specsmith mission to polish edge cases and lock down regressions.

## 🔎 Evidence
- `crates/tokmd-core/src/lib.rs`
- `crates/tokmd-core/src/ffi.rs`
- `crates/tokmd-config/src/lib.rs`

These files contained logic using `.unwrap()` on expected-infallible serialization steps or preset parsing without documenting the rationale.

## 🧭 Options considered
### Option A (recommended)
- what it is: Replace unguarded `.unwrap()` calls in `tokmd-core` and `tokmd-config` with `.expect()` and descriptive invariants.
- why it fits this repo and shard: It locks in behavior expectations (the Sentinel memory pattern) around the interface shard without altering business logic.
- trade-offs: Structure / Velocity / Governance: Slightly more verbose code, but much clearer panic semantics.

### Option B
- what it is: Focus solely on the parsing implementations in `tokmd-config`.
- when to choose it instead: If the impact of changing `tokmd-core` unwraps was considered too high.
- trade-offs: Incomplete resolution of the unguarded `unwrap` pattern across the interfaces shard.

## ✅ Decision
Chose Option A to cleanly improve invariant tracking across the interfaces shard.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/lib.rs`
- `crates/tokmd-core/src/ffi.rs`
- `crates/tokmd-config/src/lib.rs`

## 🧪 Verification receipts
```text
running 24 tests
...
test module_empty_dir_returns_no_rows ... ok
test module_depth_1 ... ok
test module_depth_5 ... ok
test module_top_setting ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

...
gate result: 4/4 steps passed
```

## 🧭 Telemetry
- Change shape: Improvement
- Blast radius: Internal panic formatting / documentation invariants.
- Risk class: Low - no logic altered, purely changes panic string formatting.
- Rollback: Revert the commit.
- Gates run: `cargo xtask gate`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces_1/envelope.json`
- `.jules/runs/specsmith_interfaces_1/decision.md`
- `.jules/runs/specsmith_interfaces_1/receipts.jsonl`
- `.jules/runs/specsmith_interfaces_1/result.json`
- `.jules/runs/specsmith_interfaces_1/pr_body.md`

## 🔜 Follow-ups
None.
