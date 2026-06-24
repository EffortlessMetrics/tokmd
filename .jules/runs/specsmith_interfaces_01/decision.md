## 🧭 Options considered

### Option A (recommended)
Add tests that enforce correct `tokmd export` and `tokmd module` deterministic behavior with `--children separate` and `--children parents-only`.
Also, add BDD scenarios testing that the `--children` flag successfully affects the CLI output modes.
Currently we lack BDD scenarios for `--children separate` and `parents-only` for `export`, and determinism tests for these modes on `module` and `export`.

### Option B
Look for other edge cases, for instance in `tokmd init` or `tokmd analyze`. Since I've already discovered missing BDD and determinism tests for `export --children` and `module --children`, filling this test gap clearly aligns with the Specsmith persona goals (improving scenario coverage and edge-case polish).

## ✅ Decision
Option A. It closes a BDD test gap for `tokmd export` and adds missing determinism checks for `tokmd module` and `tokmd export` when the `--children` flag is used. This fulfills the requirement for a proof-improvement patch.
