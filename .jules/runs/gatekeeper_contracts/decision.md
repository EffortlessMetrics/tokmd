# Decision

## Option A (recommended)
Update `xtask/src/tasks/bump.rs` to include `BASELINE_VERSION` from `crates/tokmd-analysis-types/src/baseline.rs` and `SENSOR_REPORT_SCHEMA` from `crates/tokmd-envelope/src/lib.rs` in `SCHEMA_LOCATIONS`.
This ensures version/schema string consistency when bumping workspace versions and aligns with the memory: "In the `tokmd` project, workspace schema versions are tracked in `xtask/src/tasks/bump.rs` via the `SCHEMA_LOCATIONS` array. Any new contract schema constants (e.g., `BASELINE_VERSION`, `SENSOR_REPORT_SCHEMA`) must be registered here so that workspace tools like `cargo xtask bump --schema` can manage them properly and prevent version drift."

## Option B
Do not track `BASELINE_VERSION` and `SENSOR_REPORT_SCHEMA`.
This allows schema versions to drift which is bad.

## Decision
I choose Option A. It explicitly fulfills the memory directive and reduces schema drift risk.
