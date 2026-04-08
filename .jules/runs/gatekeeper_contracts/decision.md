# Decision

## Option A (recommended)
Fix `cargo fmt` and `cargo clippy` violations in `tokmd-core` and `tokmd-python` that are causing the `cargo xtask gate --check` CI gate to fail.

- **What it is**: Formats multiline assertions properly and fixes Rust standard clippy warnings like `single_match`, `assertions_on_constants`, `redundant_pattern_matching`, and `unused_doc_comments` around Python integration bindings.
- **Why it fits this repo and shard**: CI governance and workspace tooling gates are enforced under `tooling-governance`. Unblocking the fundamental `cargo xtask gate` ensures downstream contracts, coverage, and snapshot rules can be evaluated successfully for all users.
- **Trade-offs**: Structure (removes small friction for subsequent toolchain bumps), Governance (forces strict CI compliance), Velocity (requires manual fixes for macros and unused doc comments, but unblocks wider usage).

## Option B
Disable the strict CI format/clippy gates in `xtask` or create exceptions.
- **What it is**: Update `cargo xtask gate` to not fail on standard warnings or to skip `cargo fmt --check`.
- **When to choose it instead**: If the ecosystem has transitioned to a loosely-governed format standard, or if third-party macro expansion breaks `fmt` unpredictably.
- **Trade-offs**: Reduces immediate gate friction but damages the deterministic code standard and allows continuous technical debt accumulation.

## Decision
Option A. Enforcing the gate keeps the codebase clean, readable, and deterministic. Fixing the warnings directly is straightforward and aligns with the Gatekeeper persona's mandate to protect the contract boundaries.
