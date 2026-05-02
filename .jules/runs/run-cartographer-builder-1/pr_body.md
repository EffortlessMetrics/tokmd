## 💡 Summary
Fixed factual drift in `docs/NOW.md` regarding the shipped status of in-browser receipt generation, and added the missing "No Green By Omission" capabilities design pattern to `docs/design.md` to align architectural documentation with the shipped `schema.json` and `capabilities` payload surfaces.

## 🎯 Why
- `docs/NOW.md` incorrectly listed "in-browser receipt generation" under LATER despite it already shipping in v1.9.0 via `tokmd-wasm` and `web/runner`.
- `docs/design.md` was missing documentation for the critical `capabilities` reporting pattern. The `capabilities` check is a deliberate architectural defense against false positives in CI pipelines ("No Green By Omission"). Without it in `design.md`, the documented architecture drifts from the shipped runtime guarantees.

## 🔎 Evidence
- `ROADMAP.md` correctly states for `v1.9.0`: "web/runner boots the real tokmd-wasm bundle ... renders the latest successful result, and supports JSON download."
- `docs/schema.json` lines 2072/2085 enforce the `capabilities` array in receipts to detect silent failures.
- `docs/sensor-report-v1.md` notes: "The capabilities field prevents false positives from missing checks. Directors can distinguish between 'all checks passed' and 'no checks ran'."

## 🧭 Options considered
### Option A (recommended)
- what it is: Fix the factual drift in `docs/NOW.md` and add the `capabilities` explanation to the core design principles in `docs/design.md`.
- why it fits this repo and shard: Directly targets the Cartographer persona's goals of fixing factual drift between shipped reality and roadmap/design docs. Keeps the `.jules` artifacts and `docs/` folder truthfully aligned.
- trade-offs: Structure / Velocity / Governance: Requires editing two core governance files, but ensures that both short-horizon planning (`NOW.md`) and structural reference (`design.md`) are accurate.

### Option B
- what it is: Only update `docs/NOW.md` to remove the LATER label for in-browser receipt generation.
- when to choose it instead: If the design principle of "No Green By Omission" was out of scope for the Cartographer persona.
- trade-offs: Misses the opportunity to fix a critical design/reference gap regarding the `capabilities` field, leaving a factual gap blocking clear future work.

## ✅ Decision
Selected Option A. The `capabilities` pattern is a fundamental part of the system's runtime contract (as evidenced by `SCHEMA.md` and `sensor-report-v1.md`), so it must be represented in `docs/design.md`. Simultaneously, `docs/NOW.md` was updated to accurately reflect the v1.9.0 shipped state.

## 🧱 Changes made (SRP)
- `docs/NOW.md`
- `docs/design.md`

## 🧪 Verification receipts
```text
cargo xtask publish --plan --verbose
cargo xtask version-consistency
cargo xtask docs --check
cargo check
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test -p xtask
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: documentation
- Risk class + why: Lowest - purely factual and architectural documentation updates, no functional code changes.
- Rollback: Revert the PR
- Gates run: `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo test -p xtask`, `cargo fmt -- --check`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/run-cartographer-builder-1/envelope.json`
- `.jules/runs/run-cartographer-builder-1/decision.md`
- `.jules/runs/run-cartographer-builder-1/receipts.jsonl`
- `.jules/runs/run-cartographer-builder-1/result.json`
- `.jules/runs/run-cartographer-builder-1/pr_body.md`

## 🔜 Follow-ups
None at this time.
