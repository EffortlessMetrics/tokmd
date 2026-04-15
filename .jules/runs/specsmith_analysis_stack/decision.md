# Decision

## 🧭 Options considered

### Option A (recommended)
- Add missing integration scenario tests for the `estimate` preset under `analyze`.
- The `tokmd analyze --preset estimate` behavior is part of the `analysis-stack` and represents an important user-facing feature. As per my assignment to improve BDD/scenario coverage around analysis behavior, there were missing BDD tests for the `estimate` output structure.
- Trade-offs:
    - **Structure**: High. Matches the BDD paradigm established in `bdd_analyze_scenarios_w50.rs`.
    - **Velocity**: High. Quick to implement with clear success criteria.
    - **Governance**: High. Locks in the expected output JSON structure (`effort` object containing `model` and `drivers`).

### Option B
- Focus on unit tests inside the `tokmd-analysis-effort` crate.
- We could verify `EffortDriver` initialization and model computations at the code level.
- Trade-offs:
    - Misses the integration layer (CLI flag `preset estimate` output verification), which has higher risk of breaking silently and matches the instruction to improve "scenario coverage" and "edge-case polish around analysis behavior" at the surface.
    - Potentially redundant if unit tests already exist (which they often do).

## ✅ Decision
Option A. Adding `given_project_when_analyze_estimate_then_effort_model_present` directly fulfills the prompt's instruction to add missing BDD/integration coverage for important analysis paths. It locks in the JSON structure output behavior for `--preset estimate`.
