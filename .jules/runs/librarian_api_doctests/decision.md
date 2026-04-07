# Option A: Add doctests for `analyze_workflow_from_inputs`, `Cli`, and `ConfigContext`
This option focuses on adding doctests to the key public interfaces: `analyze_workflow_from_inputs` in `tokmd-core`, `Cli` in `tokmd-config`, and `ConfigContext` in `tokmd` (which required marking the `config` module as `pub` in `tokmd`).
- Fits the requirement to prioritize doctest coverage for public APIs and configuration.
- Solves missing doctests without making sweeping prose rewrites.
- Requires exposing `config` in `tokmd` to `pub` which helps document the load workflow for ConfigContext.

# Option B: Add Doctests to only `analyze_workflow_from_inputs` and `Cli`
This option limits the doctests to `tokmd-core` and `tokmd-config` because `tokmd` `config` module is currently private.
- Doesn't require any structural changes to the `tokmd` crate, but leaves out `tokmd` public interface doctests.

# Decision
Option A is chosen because it thoroughly completes the task of adding executable docs to all key crates inside the shard: `tokmd-core`, `tokmd-config` and `tokmd`. Marking `config` as `pub` in `tokmd` aligns with standard Rust documentation patterns for exposed configuration contexts.
