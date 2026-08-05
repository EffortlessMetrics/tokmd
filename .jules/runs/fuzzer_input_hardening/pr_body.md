## 💡 Summary
This is a learning PR. Attempted to add an explicit fuzz target for the FFI in-memory input path parsing layer. PR was closed as stale and deferred due to 1.15.1 release timing constraints.

## 🎯 Why
The parser validation logic in `crates/tokmd-core/src/ffi/inputs.rs` includes numerous path defense rules. A PR was authored to add `fuzz_in_memory_inputs.rs` to harden this, but it predates the 1.15.1 reliability lane and is not a release blocker. The underlying idea was not rejected, so this learning PR captures the friction to remove stale bot work from the active queue.

## 🔎 Evidence
- Pull Request Comment ID: 5188376462 (Steward deferred as stale/non-release blocker).
- `crates/tokmd-core/src/ffi/inputs.rs` (the surface investigated).

## 🧭 Options considered
### Option A
- Continue trying to push the code change despite the steward's direct comment.
- Trade-offs: Directly violates queue discipline and repository ownership rules.

### Option B (recommended)
- Revert the patch and publish a learning PR documenting the friction.
- Trade-offs: Respects maintainer queue discipline, clears out stale bot work, preserves the context for a future salvage.

## ✅ Decision
Chosen Option B. Reverted all code changes and created a learning PR with a friction item.

## 🧱 Changes made (SRP)
- `.jules/friction/open/fuzzer_input_hardening_deferred.md`: Documented the triage deferral.
- `.jules/runs/fuzzer_input_hardening/*`: Updated run artifacts for a learning PR outcome.

## 🧪 Verification receipts
```text
None (Code changes reverted)
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None
- Risk class: Safe / Documentation
- Rollback: Revert PR
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/friction/open/fuzzer_input_hardening_deferred.md`
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`

## 🔜 Follow-ups
The original patch is available in the PR history for a future salvage if the explicit `fuzz_in_memory_inputs.rs` target is still wanted.
