## Option A (recommended)
Update the `isRunMessage` validation logic in `web/runner/messages.js` to accept alternative input shapes beyond just `inputs`. The underlying `tokmd-core` Rust runner allows specifying inputs via an `inputs` array, `paths` array, or within a nested `scan` object. By only enforcing `inputs` currently, `web/runner` unnecessarily restricts valid payloads. This option aligns the browser runner's payload validation with the core engine's capabilities, eliminating drift. It requires modifying `isRunMessage` in `web/runner/messages.js` to correctly reflect these options and updating tests in `web/runner/messages.test.mjs` and `web/runner/runtime.test.mjs`.

- Why it fits: Specifically targets interface drift between the browser-runner and Rust core, falling under the Bridge persona and `bindings-targets` shard.
- Trade-offs: Structure is improved by syncing the expected schema; Velocity is positive due to smaller scope; Governance is maintained by keeping tests passing.

## Option B
Update the `tokmd-wasm` interface to normalize payload shapes before executing them, such that all inputs are uniformly transformed into `inputs` objects.
- When to choose: If we wanted the browser side to strictly use `inputs` only, removing flexibility.
- Trade-offs: Creates an artificial barrier/discrepancy between Wasm and the CLI, rather than fixing the drift where the restriction is incorrectly enforced.

## Decision
Choose Option A. It correctly fixes the interface drift between `web/runner` and the `tokmd-core` engine by allowing valid payload structures (`paths`, `scan.paths`, `scan.inputs`) to pass validation. This strictly aligns the JS boundary with the core engine capabilities.
