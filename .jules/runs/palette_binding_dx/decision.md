# Decision: Fix `web/runner` message validation for non-in-memory inputs

## Problem
The `isRunMessage` function in `web/runner/messages.js` explicitly required the `inputs` array to be present and contain in-memory inputs, strictly returning `false` otherwise:
```javascript
export function isRunMessage(value) {
    return Boolean(
        // ...
            value.args &&
            typeof value.args === "object" &&
            !Array.isArray(value.args) &&
            Array.isArray(value.args.inputs) &&
            value.args.inputs.every(isInMemoryInput)
    );
}
```
This is incorrect because users can run analyses by providing `paths` directly at the top level or nested inside a `scan` object in `args`, as validated by the core Rust implementation (and the JS bindings should delegate to core, which understands these different structures).

By failing validation early in JS, the runtime intercepted valid commands like `{ args: { paths: ["."] } }` and returned a confusing "expected { type: \"run\", requestId, mode, args }" `invalid_message` error instead of allowing them to proceed.

## Options considered

### Option A: Relax `isRunMessage` to allow valid argument shapes
Modify `isRunMessage` to enforce that if `inputs` are present, they are valid, but otherwise allow `paths` or `scan` payloads to proceed. This aligns the browser runner's runtime validation with what the core WASM actually accepts.

- **Structure**: Updates the protocol validation to support the known argument structures.
- **Velocity**: Low risk, passes existing tests, and directly fixes the CLI-facing error surface in the browser runner DX.
- **Governance**: Aligns with core.

### Option B: Delegate all `args` validation to WASM core
Remove `isRunMessage` argument inspection entirely and just check `type`, `requestId`, `mode`, and that `args` is an object. Let WASM return structured error payload envelopes if the schema is totally invalid.

- **Structure**: More "purely" delegated validation.
- **Velocity**: Might lead to more WASM context-switching overhead for obviously malformed basic structures.
- **Governance**: Less explicit protocol enforcement at the JS boundary layer.

## Decision
**Option A**.
It fixes the immediate bug where `{ args: { paths: ["src"] } }` fails, ensures `inputs` arrays are still validated gracefully early if provided, and unblocks basic path-based scanning in the browser/Node worker layer.
