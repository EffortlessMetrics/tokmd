## 💡 Summary
Learning PR: The extraction of fuzz targets to proptests was superseded by #1590, which already merged these invariants into their owner crates.

## 🎯 Why
The `cargo fuzz` toolchain requires nightly compiler features (`-Zsanitizer=address`) that fail to build in standard sandbox environments. I originally converted the existing fuzz targets to `proptest` suites inside `crates/tokmd/tests/` to lock in this coverage. However, PR feedback indicated this work is obsolete because #1590 has already merged the high-signal fuzz-derived invariants into their respective owner crates on `main`.

## 🔎 Evidence
Minimal proof:
- PR feedback: "Superseded by #1590, which merged the high-signal fuzz-derived scan-args and context-policy invariants into their owner crates on current main."
- Reverted the local patches to prevent duplicating superseded invariant logic.

## 🧭 Options considered
### Option A
- what it is: Continue pushing the `proptest` extraction.
- when to choose it instead: If the existing invariants in #1590 missed coverage that this patch provided.
- trade-offs: Risks duplicating tests, creating drift between `tokmd/tests/` and the actual owner crate test suites.

### Option B (recommended)
- what it is: Abandon the test additions and submit a learning PR acknowledging the superseded state.
- why it fits this repo and shard: Avoids duplicate work and honors the reviewer's guidance.
- trade-offs: Structure / Velocity / Governance: Aligns perfectly with the governance instruction to defer to merged upstream work.

## ✅ Decision
Option B was chosen. The previous code patch was aborted and reverted.

## 🧱 Changes made (SRP)
- Reverted all new `crates/tokmd/tests/fuzz_*_proptests.rs` files.
- Recorded a learning artifact acknowledging #1590.

## 🧪 Verification receipts
```text
$ git reset --hard HEAD && git clean -fd
HEAD is now at bea6e23 test(types): cover optional serde omissions
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): None.
- Risk class + why: Zero. No code changes landed.
- Rollback: N/A
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/run-fuzzer-input-hardening-1/envelope.json`
- `.jules/runs/run-fuzzer-input-hardening-1/decision.md`
- `.jules/runs/run-fuzzer-input-hardening-1/receipts.jsonl`
- `.jules/runs/run-fuzzer-input-hardening-1/result.json`
- `.jules/runs/run-fuzzer-input-hardening-1/pr_body.md`
- `.jules/friction/open/fuzzer_superseded_by_1590.md`

## 🔜 Follow-ups
None.
