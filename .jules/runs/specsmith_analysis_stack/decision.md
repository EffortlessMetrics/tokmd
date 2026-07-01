## Options considered

### Option A (recommended)
- Add a new explicit BDD-style test in `crates/tokmd-analysis/tests/analysis_deep_w64.rs` to comprehensively assert that the Health preset accurately extracts and tallies all configured "TODO" tags (`TODO`, `FIXME`, `XXX`, `HACK`).
- **Why it fits**: Directly satisfies the Specsmith persona objective of "scenario coverage" and "edge-case regression not locked in by tests". The current suite implicitly verifies only `TODO` and `FIXME` in `health_preset_populates_todo_metrics_from_real_files`.
- **Trade-offs**: Minor test execution overhead (creating a temporary directory and writing a single small file) vs the benefit of explicit, deterministic coverage of all tag variants in the analysis layer.

### Option B
- Focus on extracting and unifying all tag extraction assertions directly in `crates/tokmd-analysis/src/content/io/tags.rs` (unit tests).
- **When to choose it instead**: If the goal were purely unit-level coverage for the text scanning itself.
- **Trade-offs**: Fails to provide integration-level "behavior-level tests" around the orchestrator/preset pipeline as requested by the persona, and might violate the anti-drift rule by becoming generic test cleanup.

## ✅ Decision
Option A. It adds a concrete scenario-driven integration test, directly matching the Specsmith focus on behavior-level coverage, explicitly ensuring the full `["TODO", "FIXME", "XXX", "HACK"]` tag vocabulary correctly surfaces through the pipeline into the final receipt.
