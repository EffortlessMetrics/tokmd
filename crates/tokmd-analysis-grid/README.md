# tokmd-analysis-grid

Preset and feature-gate metadata for analysis orchestration.

## Problem
You need one source of truth for preset composition and feature warnings.

## What it gives you
- `PresetKind`, `PRESET_KINDS`
- `PresetPlan`, `PRESET_GRID`
- `preset_plan_for`, `preset_plan_for_name`
- `DisabledFeature`

## Integration notes
- Pure data and serialization, with deterministic ordering at the type boundary.
- Central source of preset plans and disabled-feature warnings.
- `halstead` depends on `content` and `walk`; the other preset features are independent gates.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Schema](../../docs/SCHEMA.md), [Schema JSON](../../docs/schema.json)
- Explanation: [Explanation](../../docs/explanation.md)
