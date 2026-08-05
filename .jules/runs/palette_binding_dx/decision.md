Option A: Fix parse error messages to show the exact path to the failed JSON field (e.g. `inputs[0].path` instead of just `inputs[0]`).
Option B: Expose a detailed schema response. Takes longer.
Decision: Option A provides clear developer experience by pointing directly to the invalid field, preventing guessing in JSON FFI integrations.
