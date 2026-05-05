# Decision

## Focus
Documenting a workflow edge case where an intended patch for determinism extraction was superseded.

## Options Considered

### Option A: Attempt alternative determinism fix
- **What it is:** Find another missing test coverage area in the shard.
- **Why it fits:** The original assignment asks for proof-improvement patches.
- **Trade-offs:** We have already spent time on the original patch, and the memory explicitly states we must create a learning PR when superseded.

### Option B: Abort redundant fix and create learning PR
- **What it is:** Gracefully abort the superseded patch as per memory instructions, log a friction item, and conclude the run with a learning PR packet.
- **Why it fits:** Aligns with the memory instruction: "If an intended patch is found to be superseded by another merged PR during execution, gracefully abort the redundant fix and create a 'learning PR'. This involves generating the standard run artifacts and a new friction item (in `.jules/friction/open/`) documenting the workflow edge case."
- **Trade-offs:** No code patch is landed, but policy is correctly followed to record systemic friction.

## Selection
**Option B** is selected. Policy requires documenting superseded work as a learning PR with a friction item rather than fighting the merged state.
