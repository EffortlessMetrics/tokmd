# Decision

## 🧭 Options considered

### Option A (recommended)
- Add a new integration test `pipeline_export_cyclonedx_full` to `crates/tokmd/tests/full_pipeline_w55.rs` that explicitly verifies the behavior and output shape of `write_export_cyclonedx_to`.
- **Why it fits**: The instruction explicitly targets "improve tests around a high-value production surface with weak assertions" in the `core-pipeline` shard (`crates/tokmd-format` is part of it). `tokmd` SBOM export functionality via CycloneDX lacked a dedicated integration test in the W55 full pipeline test suite.
- **Trade-offs**: Structure / Velocity / Governance - minimal impact on structure, small time investment, strengthens governance over the CycloneDX export shape determinism.

### Option B
- Add unit tests directly to `crates/tokmd-format/src/export/cyclonedx.rs`.
- **When to choose it instead**: If we wanted to test only the serialization logic in isolation without the broader scan and model generation context.
- **Trade-offs**: Wouldn't exercise the full type conversion pipeline (`types` -> `scan` -> `model` -> `format`). The full pipeline test provides stronger behavioral assertions about what users actually experience.

## ✅ Decision
Option A. Adding an integration test to the existing full pipeline test suite strengthens behavioral checks on a high-value data export feature (SBOM/CycloneDX) where regressions could slip through, perfectly matching the Mutant persona's goals for W55 testing.
