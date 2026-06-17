# Decision

The target of this run is compatibility across bindings targets (`crates/tokmd-wasm`, `crates/tokmd-node`, `crates/tokmd-python`, `web/runner`).

Based on testing:
1. `tokmd-wasm`: `wasm-pack test --node` passes without issue when executed in the `crates/tokmd-wasm` directory.
2. `tokmd-node`: `cargo test -p tokmd-node --no-default-features` and `--all-features` pass.
3. `tokmd-python`: `cargo test -p tokmd-python --no-default-features` and `--all-features` pass.
4. `web/runner`: `npm test` fails initially if run from the workspace root workspace context but passes when executed inside `web/runner` via `cd web/runner && npm test`. There's an explicit skipped test for `# SKIP built tokmd-wasm bundle not present` which indicates the runner assumes `tokmd-wasm` output should be present.

We are within the `bindings-targets` shard, so the goal is to land a coherent reviewer story improving compatibility, proof, or determinism here.

However, all core binding matrices passed on current master (`cargo test -p <binding> --no-default-features` and `--all-features`).
The node, python, and wasm tests are solid. The runner `npm test` passed 65/66 tests (one skipped).

Option A: Add a missing matrix test (e.g., Python on old versions, or test cross-platform Python wheel building issues).
Option B: Notice that there's no failing tests but Wasm test skipped because wasm bundle isn't built. Document this in a learning PR or improve CI matrix to ensure `npm test` runs with a built `tokmd-wasm` bundle.
Option C: Ensure deterministic output from WASM integration test.

I will investigate `crates/tokmd-wasm` tests more closely to see if there is any target interaction that is missing.

The tests in this shard are passing natively across `tokmd-wasm`, `tokmd-python`, `tokmd-node`, and `web/runner`. However, looking closely at `.github/workflows/` (or general expectations), CI is running the integration test matrix.
Wait, let's look at `web/runner/tests/runner.test.js` where the SKIP happened:
"SKIP built tokmd-wasm bundle not present"

It skips it because the `tokmd-wasm` bundle wasn't built into `pkg/` prior to running the test. In CI, it's presumably built or skipped. Since this isn't an explicit "failing" test, it's just a runner fallback. Let's see if we can actually build it to see if it passes.

Testing confirms that all target compatibilities are healthy.
- `cargo test -p tokmd-wasm --no-default-features` passes
- `cargo test -p tokmd-wasm --all-features` passes
- `wasm-pack test --node --features analysis` passes natively
- `cargo test -p tokmd-node --no-default-features` passes
- `cargo test -p tokmd-node --all-features` passes
- `cargo test -p tokmd-python --no-default-features` passes
- `cargo test -p tokmd-python --all-features` passes
- `npm test -w web/runner` passes natively (when ignoring the skipped missing build artifact)
- Manually verifying the wasm bundle build allows the runner to pass without skips.

Since there is no genuine broken compatibility surface, we should issue a Learning PR documenting this health across the bindings targets matrix.

Option A: Forge a tiny compatibility patch in some error message format just to make a diff. This violates the prompt's `No tool cargo-culting` and honesty expectations.
Option B: Land a Learning PR declaring that the matrix targets (`tokmd-wasm`, `tokmd-node`, `tokmd-python`, `web/runner`) are all completely passing their target-specific capability modes (`--no-default-features` and `--all-features`, plus wasm and node test boundaries) out of the box.

I will proceed with Option B. I'll write the learning PR artifacts and exit.
