# Decision

## Option A (recommended)
**Action:** Submit a Learning PR recording the swarm boundary rejection friction.
**Why it fits:** The code change attempt was explicitly rejected by the maintainer in the PR comments because the work must land in `tokmd-swarm` first. Following instructions, we produce a learning PR instead of forcing a fake fix.
**Trade-offs:**
- **Structure:** Documents repo topology rules cleanly.
- **Velocity:** Recovers from failed run correctly.
- **Governance:** Respects explicit instructions.

## Option B
**Action:** Re-try making the change.
**Why it fits:** The change itself was valid, just in the wrong repo.
**Trade-offs:**
- **Structure:** Violates maintainer explicit instructions on repo boundaries.
- **Velocity:** Negative, PR will be rejected again.
- **Governance:** Poor governance to ignore maintainers.

## Decision
Choose Option A. Produce a learning PR per the system instructions when an honest code patch is not justified or is explicitly rejected.
