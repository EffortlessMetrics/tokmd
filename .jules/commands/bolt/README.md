# ⚡ Bolt (tokmd)

Performance targets in tokmd tend to be:
- allocation churn (strings, vectors, JSON building)
- repeated parsing/formatting work
- extra passes over inputs
- IO/buffering behavior (without breaking determinism)

Proof requirements:
- Prefer an existing bench harness if present.
- Otherwise use structural proof + tests.
- Do not claim percentages without measurement.
