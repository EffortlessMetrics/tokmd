# Decision Record: Browser Runner Memory Input Schema Alignment

## Context
The core `tokmd` FFI API supports in-memory inputs either at the root level (`args.inputs`) or nested within a scan object (`args.scan.inputs`). However, the JavaScript `web/runner/messages.js` implementation for validating run messages was strictly enforcing `inputs` only at the root level (`args.inputs`). This caused interface drift between the Rust core and the browser runner, where runner messages mimicking valid core FFI payloads containing `scan: { inputs: [...] }` were silently rejected.

## Options Considered

### Option A: Align runner validation to match core Rust API interface
- **What it is:** Update `isRunArgsForMode` in `web/runner/messages.js` to correctly accept in-memory inputs under `args.scan.inputs` in addition to `args.inputs`, while explicitly rejecting cases where both are provided (matching core FFI constraints).
- **Why it fits this repo and shard:** Resolves documented memory friction ("In `tokmd`, the core API (`tokmd-core` FFI) accepts in-memory inputs either at the root level... Bindings and runners (like `web/runner`) must support both input locations identically to prevent interface drift").
- **Trade-offs:**
  - *Structure:* Better alignment between the WASM boundary and the JS protocol layer.
  - *Velocity:* Small targeted patch.
  - *Governance:* Closes a gap where valid protocol schemas were dropped without diagnostics.

### Option B: Forbid `scan.inputs` entirely in the JS runner
- **What it is:** Maintain the current JS runner validation logic but add a specific runtime error indicating `scan.inputs` is unsupported via the runner.
- **When to choose it instead:** If `scan.inputs` caused significant unresolvable architectural problems inside the JS runtime or WASM worker.
- **Trade-offs:** Worsens interface drift; forces callers to manually transform payloads between standard `tokmd` FFI configurations and runner configurations.

## Decision
**Option A** is the best path. It resolves a clear developer experience friction point by preventing valid API payloads from being rejected as invalid protocol messages due to missing `scan.inputs` validation. It completely adheres to the `Palette` persona's objective of reducing confusion and fixing public API ergonomics across code-facing surfaces in the JS runner layer.
