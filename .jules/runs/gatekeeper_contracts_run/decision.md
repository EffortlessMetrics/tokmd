# Decision

## Option A (recommended)
- What it is: Update `tokmd-analysis-types/src/lib.rs` and tests to use `ENVELOPE_SCHEMA_VERSION` as the backward compatibility alias for `SENSOR_REPORT_SCHEMA_VERSION`, matching what's documented in `docs/SCHEMA.md`. Also change the actual constants being used/referred to: currently `ENVELOPE_SCHEMA` points to `SENSOR_REPORT_SCHEMA`, but `docs/SCHEMA.md` explicitly mentions `ENVELOPE_SCHEMA_VERSION` and `SENSOR_REPORT_SCHEMA_VERSION`. However, looking closely, `tokmd-envelope::SENSOR_REPORT_SCHEMA` exists, but there is no `SENSOR_REPORT_SCHEMA_VERSION`. The docs are slightly wrong/drifted: the alias name is `ENVELOPE_SCHEMA` not `ENVELOPE_SCHEMA_VERSION`, and the canonical is `SENSOR_REPORT_SCHEMA` not `SENSOR_REPORT_SCHEMA_VERSION`.
- Wait, I should correct `docs/SCHEMA.md` to reflect reality (`ENVELOPE_SCHEMA` and `SENSOR_REPORT_SCHEMA`), OR correct reality to match the docs (`ENVELOPE_SCHEMA_VERSION` and `SENSOR_REPORT_SCHEMA_VERSION`).
Let me look at `crates/tokmd-types/src/lib.rs`: there is `SCHEMA_VERSION`, `CONTEXT_SCHEMA_VERSION`, `CONTEXT_BUNDLE_SCHEMA_VERSION`, `HANDOFF_SCHEMA_VERSION`.
Wait, the envelope schema is a string "sensor.report.v1", NOT an integer. The other schemas are integers.
Therefore, `ENVELOPE_SCHEMA` and `SENSOR_REPORT_SCHEMA` (without `_VERSION`) is the correct naming since it's a string identifier, not an integer version number!
So Option A is to fix `docs/SCHEMA.md` to remove `_VERSION` from `ENVELOPE_SCHEMA_VERSION` and `SENSOR_REPORT_SCHEMA_VERSION`.

## Option B
- Rename `ENVELOPE_SCHEMA` to `ENVELOPE_SCHEMA_VERSION` and `SENSOR_REPORT_SCHEMA` to `SENSOR_REPORT_SCHEMA_VERSION` in the code.
- Why it fits: Aligns code with docs.
- Trade-offs: Incorrect conceptually since it's a string like "sensor.report.v1", not an integer like the others.

## Decision
Go with Option A. `docs/SCHEMA.md` has drifted and incorrectly added `_VERSION` to the string schema identifiers. I will fix `docs/SCHEMA.md`.
