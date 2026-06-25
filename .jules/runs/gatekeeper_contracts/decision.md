# Decision

## Option A (recommended)
- what it is: Update `policy/non-rust-allowlist.toml` to cover all untracked test fixtures, policies, and scripts that currently cause `cargo xtask check-file-policy --strict` to fail on a fresh clone.
- why it fits this repo and shard: The `tooling-governance` shard governs repository invariants and file policies. `cargo xtask check-file-policy --strict` is a core deterministic gate. The fact that it fails out of the box is a broken invariant. Fixing it tightens deterministic behavior.
- trade-offs: Structure / Velocity / Governance: Improves Governance (lock in file policy) and Velocity (developers don't have to manually skip untracked files) with minimal Structure overhead.

## Option B
- what it is: Delete the untracked fixtures.
- when to choose it instead: If the fixtures were truly unintended garbage.
- trade-offs: This would break the tests that rely on them.

## Decision
Option A. The fixtures are real and required for tests, they were simply missing from the policy allowlist, causing a broken file-policy gate out of the box. I will add them to `policy/non-rust-allowlist.toml` and prove the invariant holds.
