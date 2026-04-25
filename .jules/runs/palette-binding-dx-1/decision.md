# Decision

## Option A (recommended)
Fix `web/runner/messages.js` to correctly validate `run` messages that use `paths` or `scan` arguments instead of strictly requiring `inputs`.
- **Why it fits:** The runtime memory strictly says "In the `tokmd` `web/runner`, run message arguments can be passed via `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` objects. Validation logic (e.g., `isRunMessage`) must accept payloads utilizing any of these valid structures, not strictly requiring `inputs` in all cases."
- **Trade-offs:** Minimal code footprint. Aligns the browser runner with other bindings that use `paths` and `scan`. Easy to verify with existing test modifications.

## Option B
Also implement full mock runner overrides for testing `paths` and `scan` messages throughout the worker tests.
- **Why to choose it:** Provides deeper coverage in the mock runtime for non-input executions.
- **Trade-offs:** Larger scope than strictly required for fixing the message parsing itself, which already resolves the protocol violation. Tests already skip WASM testing due to missing tools, so mock-heavy changes are riskier for real fidelity.

## Decision
Option A. I will patch `web/runner/messages.js` to handle `inputs`, `paths`, and `scan` natively within `isRunMessage`, and verify this by adding a `test_message.js` or directly via standard runner test modifications if they failed initially. We've already shown in previous commands that replacing `isRunMessage` allows these message types to pass correctly. I'll make sure tests pass. This squarely resolves the runtime DX friction for web runner consumers who use `paths`/`scan`.
