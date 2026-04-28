# Decision

## Investigated
- Tried to run `cargo fuzz` but encountered the nightly toolchain requirement and ASAN errors which block the primary target of fuzzing.
- Based on Jules memory: "When operating under a strictly-scoped persona (like 'Fuzzer'), if environmental issues (like `cargo fuzz` ASAN errors) block primary goals and no honest, in-scope patch (e.g., `proptest`) is justified, do not pivot to out-of-scope tasks (like UX improvements). Instead, strictly follow fallback instructions to abort the code patch and immediately create a Learning PR that documents the blocker as a friction item in `.jules/friction/open/`."
- Searched for other parser inputs, like `crates/tokmd/src/config.rs` but a good amount of unit/integration tests exist, and without `fuzz` tooling working easily and `cargo-fuzz` requiring nightly, the honest and compliant path is to record friction and create a learning PR.
- There are already a few proptests in `crates/tokmd-config/tests` and `crates/tokmd-core/tests`. Creating random properties without a clear invariant feels like guessing without `fuzz` working.

## Options Considered
### Option A (Record ASAN/nightly friction and create a learning PR)
- What it is: Acknowledge the environmental block on `cargo fuzz`, abort forcing a patch, and write a learning PR per the memory constraints.
- Why it fits: Aligns precisely with the memory rules for Fuzzer encountering `cargo fuzz` ASAN/nightly errors. It prevents hallucinated work and correctly reports the blocker.
- Trade-offs: Velocity on the immediate codebase remains unchanged, but governance improves by making the problem visible.

### Option B (Attempt `proptest` workarounds on config logic)
- What it is: Instead of fuzzing, write some deterministic property tests in `crates/tokmd/src/config.rs`.
- When to choose it instead: If the prompt explicitly asked for `proptest` or if there was an obvious untested property in the interface parsing logic.
- Trade-offs: High risk of writing "arbitrary proptests without a clearly stated invariant" (which the fuzzer profile says not to do), taking time away from accurate reporting.

## Decision
Chosen **Option A**. The environment block is a classic friction item that must be documented rather than trying to force a patch out of scope. I will write a friction item and finish with a learning PR.
