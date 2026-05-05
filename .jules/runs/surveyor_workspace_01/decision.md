# Decision

## Option A (recommended)
**Create a learning PR documenting the out-of-shard friction item regarding missing workspace crates**
- What it is: Acknowledge that the `cargo-machete` warnings expose workspace configuration drift. Specifically, `crates/tokmd-config`, `vendor/home-0.5.12`, and `crates/tokmd/tests/data` believe they are in the workspace but aren't explicitly included in the `[workspace.members]` block of `Cargo.toml`. Also `tokmd-config` has compilation issues (missing re-exports/types from `tokmd_config`) preventing testing.
- Why it fits this repo and shard: It correctly identifies a workspace structural boundary issue, but since `cargo machete` is an auditor tool and `tokmd-config` is not explicitly tested, the scope may slightly cross into the `Auditor` persona's purview for `machete`, and fixing it fully involves significant test rewrites inside `tokmd-config`.
- Trade-offs:
    - Structure: Good for recording friction.
    - Velocity: Faster than rewriting tests for `tokmd-config`.
    - Governance: Complies with the prompt ("If the strongest target you find is outside the shard, record it as friction instead of chasing it. If no honest code/docs/test patch is justified, finish with a learning PR").

## Option B
**Fix workspace members and patch `tokmd-config`**
- What it is: Add `crates/tokmd-config` to `workspace.members`, then rewrite the tests in `tokmd-config` to import the correct items from `tokmd::cli` / `tokmd_settings` / `tokmd_types` instead of `tokmd_config`.
- When to choose it instead: If a clean, fast fix exists that doesn't involve touching dozens of test files to fix re-export paths.
- Trade-offs: Changes many test files in an older crate, making for a larger diff than requested.

## ✅ Decision
Option A. The `tokmd-config` crate appears to be largely deprecated or demoted in favor of `tokmd::cli` and `tokmd-settings`, as evidenced by its `src/lib.rs` ("this crate remains as a compatibility shim"). The fact that it isn't in the workspace members, and fixing it to be in the workspace causes its tests to fail (because they rely on outdated type locations that were moved into `tokmd-settings` or `tokmd-types`), means fixing it is a substantial refactor (rewriting ~8 test files) that isn't cleanly providing a forward-moving architectural win. It's more of a generic cleanup. Therefore, a learning PR and a friction item is the correct Surveyor outcome.
However, since this branch is superseded by #1585 which retires `tokmd-config`, this PR will be closed and work stopped.
