## Options Considered

### Option A: Deepen determinism testing for `RunReceipt`
`RunReceipt` is part of the core outputs (`lang`, `module`, `export`, `diff`, `run`) but isn't explicitly checked for byte-for-byte determinism in the `determinism_regression.rs` or `determinism.rs` suites like the other receipt types are. Adding this fills a blind spot in the determinism contract proof suite.
- **Why it fits**: Fits the Gatekeeper persona's goal of protecting deterministic outputs and tightening invariants with tests.

### Option B: Check for snapshot drift in `cli_snapshot_golden.rs`
The snapshot tests cover most CLI formats (`lang`, `export`, `module`), but we could add more complex scenarios (e.g. `diff` or `run`) to the golden snapshots to ensure that those schemas don't drift silently.
- **Why it fits**: Covers the "snapshot/golden drift or weak coverage" target ranking for Gatekeeper.

### Decision
I will pursue **Option A** (with potentially a bit of Option B by adding `RunReceipt` to snapshot tests if missing, though it might be harder to mock run environments for golden tests). I will add a determinism regression test for `run` output (`tokmd run`) to `crates/tokmd/tests/determinism_regression.rs`.
