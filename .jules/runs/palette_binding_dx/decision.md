## Decision

**Problem:** In `web/runner`, errors thrown across Web Workers (or from WASM boundaries) often lose their prototype chain due to structured cloning or cross-realm boundaries. When this happens, `error instanceof Error` evaluates to `false`. Several error handling points (`runtime.js`, `main.js`, and `worker.js`) strictly required `instanceof Error` to extract the `.message` or `.code`, resulting in generic "unknown runner error" or "String([object Object])" logs for legitimate failure scenarios.

### Option A (recommended)
Update error handlers in `web/runner` to accept duck-typed error objects. If `error instanceof Error` is false, check if `error` is an object with a string `.message` property. This directly satisfies the memory constraint: *"In JavaScript environments interacting with WebAssembly or Web Workers (like `web/runner`), error objects often lose their prototype chain during serialization. Error extraction logic must handle duck-typed objects by checking for `.message` and `.code` properties instead of strictly relying on `error instanceof Error`."*

**Trade-offs:**
- **Structure:** Improves error clarity directly in the JS bindings.
- **Velocity:** Low-risk, high-value change targeting the browser runner.
- **Governance:** Keeps the changes strictly to the JS layer, aligned with the `bindings-targets` shard.

### Option B
Wait to fix this upstream in `tokmd-wasm` by ensuring the rust-wasm bindings only throw explicit `JsValue::String` or carefully wrapped objects that always survive the boundary.

**Trade-offs:**
- Requires deeper WASM/Rust changes, potentially destabilizing the core.
- Fails to fix Web Worker boundary issues which are standard structured cloning limitations.
- Slower to implement and verify.

### ✅ Decision
**Option A**. It's the standard, robust way to handle cross-realm and serialized errors in JavaScript, and perfectly aligns with the prompt's focus on runtime DX in the assigned shard.
