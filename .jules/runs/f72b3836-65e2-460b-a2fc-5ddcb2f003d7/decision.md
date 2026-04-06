# Cartographer Decision

## Option A: Align `docs/implementation-plan.md` with shipped reality
The `docs/implementation-plan.md` file is stale. It shows all WASM work items as unchecked, whereas `ROADMAP.md`, `docs/architecture.md`, and the actual source code (`crates/tokmd-wasm`, `web/runner`) show that the core browser runner MVP, `tokmd-io-port`, and wasm bindings have landed. Furthermore, the `docs/implementation-plan.md` completely omits the completed Python and Node.js language bindings (`tokmd-python` and `tokmd-node`), which are explicitly marked as completed in the `ROADMAP.md` and shipped in the codebase. This option updates `docs/implementation-plan.md` to reflect the shipped reality.

## Option B: Demote `ROADMAP.md` and `docs/architecture.md`
Revert the "landed" claims in `ROADMAP.md` and `docs/architecture.md` to match the stale `docs/implementation-plan.md`.

## Decision
**Option A** is the only correct choice because it respects the actual shipped truth (crates exist on disk). We will update `docs/implementation-plan.md` to check off the landed WASM items and add the completed "Language Bindings (FFI)" phase.
