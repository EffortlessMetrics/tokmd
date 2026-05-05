# Option A: Expand Input Support in Runner (Recommended)

- **What it is**: Enhance `web/runner/messages.js` and `web/runner/worker.js` to extract in-memory inputs from either `args.inputs` or `args.scan.inputs`, matching the behavior of `tokmd-core`. Add test coverage for both.
- **Why it fits**: The JS runner and WASM binding currently reject valid API calls that provide `inputs` via the nested `scan` field, causing interface drift. Fixing this directly aligns bindings with the core API capability matrix as required.
- **Trade-offs**:
  - *Structure*: Small localized change that strictly enforces the requirement that only one location provides inputs at a time.
  - *Velocity*: Quick to implement and verify through existing JS tests.
  - *Governance*: Requires no core code changes, keeping risk purely within the browser/node layer.

# Option B: Create a Learning PR for Superseded Work (Chosen)

- **What it is**: The intended code changes for Option A have already been merged via #1594. Therefore, instead of forcing redundant changes, gracefully abort the task and create a learning PR documenting the workflow edge case.
- **Why it fits**: Complies with the `Superseded work fallback rule` constraint in memory, which dictates ending the run safely when a patch has been preempted.
- **Trade-offs**:
  - *Velocity*: Quickest path to safely exit without stepping on existing commits.
