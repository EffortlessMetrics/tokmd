## Problem
- I attempted to land structural improvements to `tokmd-analysis` tests directly in the `tokmd` repository, but this surface area is actually governed by `tokmd-swarm`.
- The PR was closed as wrong-repo intake.
- This creates friction because it wasn't immediately obvious from local repo state that these specific files should not be modified here.

## Proposed solution
- If surfaces are imported entirely from a swarm repository, consider adding a clear `AGENTS.md` or header comment in those directories warning against direct modification in `tokmd`.
