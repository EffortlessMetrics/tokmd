# Friction Item: specsmith_baseline_fallback_obsolete
- **Persona**: Specsmith
- **Target**: Baseline Fallback Logic (`ComplexityBaseline::from_analysis`)
- **Friction**: The issue described in the memory prompting the fix to extract structural counts (`total_files`, `total_code_lines`) from `receipt.derived` if `receipt.complexity` is `None` was actually already resolved on `main`. Attempting to implement it led to a duplicate PR that was closed as obsolete.
- **Action/Follow-up**: Rely strictly on the actual file content state during exploration before trying to "fix" something noted in memory, as memory might be stale relative to `main`.
