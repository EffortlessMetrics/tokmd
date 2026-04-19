## 🧭 Options considered

### Option A (recommended)
Add `analyze` snapshots to `crates/tokmd/tests/cli_snapshot_golden.rs`.
The `analyze` feature is a core part of the tokmd API contract, generating deeply enriched receipts. While other base modes (`lang`, `module`, `export`) are covered, the `analyze` command is absent from `cli_snapshot_golden.rs`.
Using the normalizer function and the memory instructions (`#[cfg(feature = "analysis")]`, normalizing `target_path`, `base_signature`), we can effectively prevent golden drift for `analyze` markdown and JSON outputs. I will add:
1. `snapshot_analyze_markdown`
2. `snapshot_analyze_json`

### Option B
Add a new check to `version-consistency.rs`. However, Option A aligns perfectly with our memory constraint to normalize `analyze` output and add snapshots in `cli_snapshot_golden.rs`.

## ✅ Decision
Option A. Adding CLI snapshots for `analyze` closes a significant contract regression gap (Target 2: "snapshot/golden drift or weak coverage") and satisfies the instructions exactly.

I will implement `snapshot_analyze_markdown` and `snapshot_analyze_json` in `crates/tokmd/tests/cli_snapshot_golden.rs`.
