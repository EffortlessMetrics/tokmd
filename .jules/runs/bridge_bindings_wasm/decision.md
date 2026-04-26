# Option A: Fix isRunMessage in web/runner/messages.js to support 'paths' and 'scan' in addition to 'inputs'
- **What it is:** Update the validation logic in `isRunMessage` so that `args.paths` and `args.scan` are considered valid alongside the default `args.inputs`, adhering to the memory instruction "run message arguments can be passed via inputs ... paths ... or scan objects".
- **Why it fits:** Reduces drift across interfaces by bringing the web runner protocol validation in sync with the CLI and underlying wasm capabilities. Fits the "Bridge" persona.
- **Trade-offs:** Minimal risk; purely widens the accepted interface to align with the documented capability.

# Option B: No-op / Learning PR
- **What it is:** Do not change any code, record friction.
- **Why it fits:** Used if a patch isn't straightforward.
- **Trade-offs:** We miss fixing a clear bug that is directly specified in the task context.

# Decision
Option A. The prompt memory strictly calls out that `isRunMessage` validation logic must accept `inputs`, `paths`, or `scan` objects instead of strictly requiring `inputs`.
