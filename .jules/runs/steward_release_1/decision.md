# Decision: Fix Documentation Version Drift

## Problem
The `tokmd` workspace version in `Cargo.toml` is `1.11.0`, but various active documentation files (`docs/github-action.md`, `docs/sensor-report-v1.md`, `docs/SCHEMA.md`, `docs/design.md`, and `docs/install.md`) still reference `1.10.0`. This represents a publish-plan/version-consistency drift.

## Options Considered

### Option A: Manual Targeted Replacement (Recommended)
- **What it is**: Update the version strings strictly within active documentation surfaces (`docs/github-action.md`, `docs/sensor-report-v1.md`, `docs/SCHEMA.md`, `docs/design.md`, and `docs/install.md`). Skip historical documents (e.g., ADRs, past audits, historical implementation plans) that deliberately document points in time.
- **Why it fits**: Directly addresses the "release metadata or changelog mismatch" and "publish-plan/version-consistency drift" targets assigned to the Steward persona. It maintains the integrity of historical documents while fixing active release-surface documentation.
- **Trade-offs**:
    - Structure: Preserves the semantic meaning of historical files.
    - Velocity: Slightly slower than a global find/replace.
    - Governance: High alignment with release-safety work and minimum behavior changes.

### Option B: Automated Global Replacement
- **What it is**: Run a global `sed` script or find-and-replace to blindly bump all `1.10.0` references to `1.11.0`.
- **Why to choose it**: Can be faster to execute.
- **Trade-offs**:
    - High risk of corrupting historical records, like `docs/adr/0005-release-train-and-rc-semantics.md`, test cases, or past implementation plans that accurately describe the state at v1.10.0.

## Decision
**Option A** is chosen. It is a low-risk, high-confidence improvement that directly fixes version drift in the documentation without corrupting historical records, aligning perfectly with the Steward persona's mission.