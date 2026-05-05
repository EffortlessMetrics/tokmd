# Option A
Fix the cross-interface drift between Rust core FFI and the browser runner.

# Option B
Abort the fix and create a learning PR.

# Decision
Option B was chosen. The intended patch is superseded by merged PR #1594, which merged the current browser runner `args.scan.inputs` parity synthesis with strict validation and worker coverage. I will gracefully abort the redundant fix and create a learning PR as per memory instructions.
