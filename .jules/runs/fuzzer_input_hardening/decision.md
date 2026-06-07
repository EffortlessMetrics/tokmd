### Option A: Fake a fix
- **What it is:** Create arbitrary tests around inputs or rename existing files.
- **Trade-offs:** Fails the core rules "Do not claim a win you did not prove" and "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."

### Option B: Acknowledge the blocked environment and record a Learning PR (Recommended)
- **What it is:** I attempted to run `cargo fuzz` locally, but `libfuzzer-sys` requires nightly toolchains to build (due to `-Zsanitizer=address` errors) and fails out of the box in this environment. There are no obvious missing invariant test cases since `cli_parser_properties.rs` already exists and covers the CLI interface exhaustively via `proptest`. So instead of fabricating a "fake fix", I will output a learning PR and record the friction item.
- **Why it fits this repo and shard:** Adheres to the memory instructions: "If environmental issues block primary goals and no honest, in-scope patch is justified, do not pivot to out-of-scope tasks... strictly follow fallback instructions to abort the code patch and immediately create a Learning PR that documents the blocker as a friction item".
- **Trade-offs:**
  - *Structure:* Complies perfectly with the "no tool cargo-culting" and "no fake fixes" rules.
  - *Governance:* Records the environmental deficiency for future `cargo-fuzz` runs.

**Decision**
**Option B**. Fuzz tooling (`cargo-fuzz`) requires a nightly compiler and ASAN which are not available, causing linker failures or build blocks. Deterministic regressions for the CLI already exist in `cli_parser_properties.rs`. I will output a learning PR to document this blocker and create a friction item.
