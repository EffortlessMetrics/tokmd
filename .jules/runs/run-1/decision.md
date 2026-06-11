# Decision

## Option A (recommended)
Produce a learning PR documenting that workspace structural changes (like consolidating internal crates to use workspace dependencies) are wrong-repo intake for `tokmd`. Such architectural changes must originate in `tokmd-swarm`.

- Why it fits: We are acting as Surveyor, and our initial code patch was rejected due to repository topology constraints. The prompt allows a learning PR as a successful outcome.
- Trade-offs: Abandons the immediate code fix, but prevents future agents from wasting effort on direct `tokmd` structural refactors.

## Option B
Do nothing and drop the work.
- Trade-offs: Fails to capture the learning that `tokmd` is not the origin for structural changes.

We will proceed with Option A.
