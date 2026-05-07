## Target Selection
Target: `tokmd-analysis` derived property testing.
Invariant: The `test_density` and `boilerplate` ratio metrics must always be bounded mathematically to the unit range `[0.0, 1.0]`.

### Option A (recommended)
Add property tests asserting the unit-range boundary invariants of `test_density` and `boilerplate` against the arbitrary `FileRow` generation corpus.
- Fits this repo and shard as it strictly reinforces model guarantees.
- Structure: high, guarantees expected output schema constraints.
- Velocity: medium, low runtime impact, high stability.
- Governance: matches the 'property' gate expectations for invariant testing.

### Option B
Manually exhaust all possible `lines`, `code`, `infra` and `test` combinations with targeted edge case unit tests.
- High manual toil, does not fully lock the invariant against combinations proptest might surface later.

## Decision
Chosen Option A. Generating arbitrary file rows and evaluating `derive_report` is the highest-signal proof that the derived model does not break fundamental unit range mathematical constraints. However, because the patch was superseded, this decision is being recorded purely as a learning artifact.
