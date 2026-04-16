# Decision

## Investigation
- Memory note explicitly calls out: `In JavaScript environments interacting with WebAssembly or Web Workers (like web/runner), error objects often lose their prototype chain during serialization. Error extraction logic must handle duck-typed objects by checking for .message and .code properties instead of strictly relying on error instanceof Error.`
- I checked `web/runner/runtime.js` and saw `extractRunnerError` only checks `error instanceof Error` and `typeof error === "string"`. It does not handle duck-typed objects missing the prototype chain but having `.message` and potentially `.code`.
- I ran a quick node test, which confirmed duck typed objects missing the Error prototype fail the `extractRunnerError` check and return "unknown runner error".

## Options considered

### Option A: Update `extractRunnerError` to check for duck-typed error objects
- Update `web/runner/runtime.js` `extractRunnerError` to handle duck-typed error objects with a `.message` property and optionally a `.code` property.
- Update `web/runner/runtime.test.mjs` to include tests covering duck-typed errors.

### Option B: Investigate FFI envelope error mismatch
- The FFI boundary in `tokmd_core::ffi` converts errors using a specific JSON envelope. The wasm-bindgen implementation uses `js_sys::Error`.
- We could change how errors are bubbled up from WebAssembly to standard JS objects via the binding layer.

## Decision
- Option A is the right approach. It aligns with the known gap described in the shard memory notes regarding prototype chains being lost during Web Worker or WebAssembly boundary crossing. It is also a very scoped, direct fix that requires minimal blast radius.
