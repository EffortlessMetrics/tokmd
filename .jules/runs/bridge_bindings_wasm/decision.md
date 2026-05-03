# Option A
Update `web/runner/messages.js`, `web/runner/messages.test.mjs`, `web/runner/worker.js`, and `web/runner/worker.test.mjs` to allow inputs to be passed via `args.scan.inputs` instead of just `args.inputs`, and explicitly enforce that they cannot be passed in both places.

# Option B
Update `crates/tokmd-core/src/ffi.rs` to stop accepting inputs via `args.scan.inputs`.

# Decision
Option A was chosen. It fits the 'Bridge' persona mission of reducing cross-interface drift, and matches the fact that the core FFI (which is the source of truth) supports both locations but not simultaneously.
