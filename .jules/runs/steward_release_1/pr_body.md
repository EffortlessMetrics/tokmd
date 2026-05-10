## 💡 Summary
Updated active documentation surfaces to align with the current workspace version (1.11.0). This resolves version drift where active docs were still referencing 1.10.0.

## 🎯 Why
The `tokmd` workspace version in `Cargo.toml` is `1.11.0`, but various active documentation files (`docs/github-action.md`, `docs/sensor-report-v1.md`, `docs/SCHEMA.md`, `docs/design.md`, and `docs/install.md`) still reference `1.10.0`. This publish-plan/version-consistency drift can cause confusion for users copying commands, checking action setups, or interpreting schema payloads.

## 🔎 Evidence
- `Cargo.toml` specifies workspace `version = "1.11.0"`.
- `grep -r "1.10.0" docs/` revealed multiple active files containing `1.10.0`.

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Update the version strings strictly within active documentation surfaces (`docs/github-action.md`, `docs/sensor-report-v1.md`, `docs/SCHEMA.md`, `docs/design.md`, and `docs/install.md`). Skip historical documents (e.g., ADRs, past audits, historical implementation plans) that deliberately document points in time.
- **Why it fits**: Directly addresses the "release metadata or changelog mismatch" and "publish-plan/version-consistency drift" targets assigned to the Steward persona. It maintains the integrity of historical documents while fixing active release-surface documentation.
- **Trade-offs**:
    - Structure: Preserves the semantic meaning of historical files.
    - Velocity: Slightly slower than a global find/replace.
    - Governance: High alignment with release-safety work and minimum behavior changes.

### Option B
- **What it is**: Run a global `sed` script or find-and-replace to blindly bump all `1.10.0` references to `1.11.0`.
- **When to choose it instead**: Only if there are no historical records in the workspace that rely on old version strings.
- **Trade-offs**:
    - High risk of corrupting historical records, like `docs/adr/0005-release-train-and-rc-semantics.md`, test cases, or past implementation plans that accurately describe the state at v1.10.0.

## ✅ Decision
Option A was chosen. It is a low-risk, high-confidence improvement that directly fixes version drift in the documentation without corrupting historical records, aligning perfectly with the Steward persona's mission.

## 🧱 Changes made (SRP)
- `docs/github-action.md`: Bumped `1.10.0` references to `1.11.0`.
- `docs/sensor-report-v1.md`: Bumped `1.10.0` references to `1.11.0`.
- `docs/SCHEMA.md`: Bumped `1.10.0` references to `1.11.0`.
- `docs/design.md`: Bumped `1.10.0` references to `1.11.0`.
- `docs/install.md`: Bumped `1.10.0` references to `1.11.0`.

## 🧪 Verification receipts
```text
{"command": "cargo xtask version-consistency", "outcome": "Version consistency checks passed."}
{"command": "cargo xtask publish --plan --verbose", "outcome": "Workspace version: 1.11.0"}
{"command": "cargo xtask docs --check", "outcome": "Documentation is up to date."}
{"command": "cargo deny --all-features check", "outcome": "advisories ok, bans ok, licenses ok, sources ok"}
{"command": "grep -r \"1.10.0\" docs/", "outcome": "Revealed multiple files in docs/ containing version 1.10.0 that need updating to 1.11.0."}
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: Docs only
- Risk class: Low - Only updates version strings in markdown files.
- Rollback: Revert the PR
- Gates run: `cargo xtask version-consistency`, `cargo xtask publish --plan --verbose`, `cargo xtask docs --check`, `cargo deny --all-features check`

## 🗂️ .jules artifacts
- `.jules/runs/steward_release_1/envelope.json`
- `.jules/runs/steward_release_1/decision.md`
- `.jules/runs/steward_release_1/receipts.jsonl`
- `.jules/runs/steward_release_1/result.json`
- `.jules/runs/steward_release_1/pr_body.md`

## 🔜 Follow-ups
None