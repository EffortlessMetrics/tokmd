## 💡 Summary
Implemented `std::fmt::Display` for `ExclusionReason` in `xtask` publish command and updated formatting. This improves the readability of the release publish plan drift output by providing clear, human-readable reasons instead of internal Debug enum variants.

## 🎯 Why
Running `cargo xtask publish --plan --verbose` previously exposed internal Rust `Debug` enum formatting (e.g., `NotPublishable`) for excluded crates. This was an unpolished presentation detail for a release governance surface.

## 🔎 Evidence
- `xtask/src/tasks/publish.rs`
- Observed behavior: `Excluded crates:\n  - tokmd-fuzz: NotPublishable`
- Command: `cargo run -p xtask -- publish --plan --verbose`

## 🧭 Options considered
### Option A (recommended)
- Implement `Display` for `ExclusionReason` to map variants to clear descriptions (e.g., "publish = false in manifest") and use `{}` instead of `{:?}` in `print_plan`.
- Fits repo and shard: Directly improves the presentation of the release publish plan, a stated priority in the governance shard.
- Trade-offs: Structure: Adds a standard boilerplate trait impl. Velocity: Fast. Governance: Polishes a core release artifact.

### Option B
- Look for drift in `.github/workflows/` or `CHANGELOG.md` instead.
- When to choose: If the presentation of the publish plan is deemed insufficiently impactful.
- Trade-offs: Misses an easy win to fulfill the core prompt directive of low-risk, high-confidence release-surface fixes.

## ✅ Decision
Chose Option A to cleanly polish the release publish plan without changing behavioral semantics.

## 🧱 Changes made (SRP)
- `xtask/src/tasks/publish.rs`: Added `Display` implementation for `ExclusionReason` and updated the format string in `print_plan`.

## 🧪 Verification receipts
```text
=== Publish Plan ===
...
Excluded crates:
  - tokmd-fuzz: publish = false in manifest
...
```

## 🧭 Telemetry
- Change shape: Presentation improvement
- Blast radius: Internal release tooling output only
- Risk class: Very Low - No behavioral changes
- Rollback: Revert the PR
- Gates run: `cargo xtask publish --plan --verbose`, `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo build/test`, `cargo fmt/clippy`

## 🗂️ .jules artifacts
- `.jules/runs/steward_release/envelope.json`
- `.jules/runs/steward_release/decision.md`
- `.jules/runs/steward_release/receipts.jsonl`
- `.jules/runs/steward_release/result.json`
- `.jules/runs/steward_release/pr_body.md`

## 🔜 Follow-ups
None.
