## Problem
The initial documentation patch was rejected because the `tokmd` repository acts as a downstream merge sink for the `tokmd-swarm` topology. Any documentation fixes must originate in the `tokmd-swarm` repo first. Because the agent cannot wait or interact with the `tokmd-swarm` repo in this current context (it was instructed to operate on a fresh clone of `tokmd`), the code patch approach is invalid.

## Option A
Stop work and report failure.
**When to choose:** If no other value can be derived.

## Option B (recommended)
Pivoting to the required fallback: **create a learning PR**. Document the structural topology constraint as a friction item to prevent future agents from wasting effort modifying the `tokmd` documentation directly when changes belong in `tokmd-swarm`.

**Why it fits:** The prompt explicitly states: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix." Because of the topology rules, a patch here is *not* justified.

## Decision
Proceeded with Option B. Created a friction item documenting the swarm vs. downstream topology and a learning PR packet.
