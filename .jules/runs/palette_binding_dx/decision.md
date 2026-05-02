# Option A: Expand Input Support in Runner (Recommended)

- **What it is**: Enhance `web/runner/messages.js` and `web/runner/worker.js` to extract in-memory inputs from either `args.inputs` or `args.scan.inputs`, matching the behavior of `tokmd-core`. Add test coverage for both.
- **Why it fits**: The JS runner and WASM binding currently reject valid API calls that provide `inputs` via the nested `scan` field, causing interface drift. Fixing this directly aligns bindings with the core API capability matrix as required.
- **Trade-offs**:
  - *Structure*: Small localized change that strictly enforces the requirement that only one location provides inputs at a time.
  - *Velocity*: Quick to implement and verify through existing JS tests.
  - *Governance*: Requires no core code changes, keeping risk purely within the browser/node layer.

# Option B: Remove Support from tokmd-core

- **What it is**: Modify `tokmd-core` to only allow `inputs` at the root, and drop `args.scan.inputs` support.
- **When to choose**: If nested inputs were proven to be a deprecated mistake in the core API.
- **Trade-offs**: Changes core API contracts and likely breaks existing native clients relying on `args.scan.inputs`.
