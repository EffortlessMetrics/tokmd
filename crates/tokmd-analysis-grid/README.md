# tokmd-analysis-grid

This microcrate owns the analysis preset matrix and feature-gate warning
catalog used by `tokmd-analysis`.

## Purpose

- Keep preset-to-component mapping centralized.
- Keep disabled-feature warning messages consistent.
- Make the BDD-style feature matrix explicit and easy to audit.
- Keep feature-gated behavior interoperable across crates.

## API

- `PresetKind` - preset identifiers.
- `PresetPlan` - enabled analysis components for one preset.
- `preset_plan_for` - retrieve plan by `PresetKind`.
- `preset_plan_for_name` - retrieve plan by preset name.
- `PRESET_GRID` - canonical BDD-style matrix table.
- `DisabledFeature::warning()` - stable disabled-gate messages.
