## 🧭 Options considered

### Option A: Fix file policy tracking drift (recommended)
- I found that 18 tracked fixtures, JSON policy files, and bash scripts were missing from `policy/non-rust-allowlist.toml`, causing `cargo xtask check-file-policy --strict` to fail.
- I will add `fixtures/**`, `policy/*.json`, and `scripts/**` globs to the file policy allowlist so that all currently committed untracked-but-valid files pass the file policy check.
- trade-offs: Structure / Velocity / Governance: Fixes broken governance checks in `xtask` related to release readiness/safety. High confidence, low risk, exact fit for the Steward persona.

### Option B: Produce a Learning PR
- Leave the allowlist broken and just document it.
- when to choose it instead: If the files were actually malicious and needed removal rather than tracking.
- trade-offs: Missed opportunity to fix the file policy check, which is an important governance tool.

## ✅ Decision
Option A. I will fix the `policy/non-rust-allowlist.toml` to cover the untracked file policy drift.
