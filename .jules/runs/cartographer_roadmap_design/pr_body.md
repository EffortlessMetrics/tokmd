## 💡 Summary
Updated `ROADMAP.md` to reflect that `v1.12.0` (Evidence Workbench) and `v1.13.0` (Syntax-Aware Evidence Packets) have shipped.

## 🎯 Why
There is significant factual drift between the shipped reality (currently at v1.13.1) and `ROADMAP.md` which tracked status only up to `v1.11.0`. The roadmap still described `v1.12.x` as planned and AST parsing as "shadow-only", missing the actual features delivered.

## 🔎 Evidence
- `ROADMAP.md` missing v1.12.0 and v1.13.0
- `CHANGELOG.md` shows `v1.12.0` and `v1.13.0` features actually released
- `Cargo.toml` shows `version = "1.13.1"`

## 🧭 Options considered
### Option A (recommended)
- Update `ROADMAP.md` with factual status of v1.12 and v1.13 based on CHANGELOG.md
- Fits repo and shard by fixing documentation drift
- Structure: Keeps the document chronologically accurate.
- Velocity: Helps reviewers understand what is actually shipped vs planned.
- Governance: Accurate historical tracking.

### Option B
- Fix missing schema constants in `xtask/src/tasks/bump.rs`
- Better suited for a Gatekeeper or Steward persona fixing release tooling, so recorded as friction instead.

## ✅ Decision
Proceeded with Option A to accurately map the current repository state and recorded Option B as friction.

## 🧱 Changes made (SRP)
- `ROADMAP.md`: Added v1.12.0 and v1.13.0 to Status Summary and Completed Milestones, and updated Current Roadmap Status to remove stale items.

## 🧪 Verification receipts
```text
cargo xtask publish --plan --verbose
cargo xtask version-consistency
cargo xtask docs --check
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: Docs only
- Risk class: Low, no runtime impact
- Rollback: Revert PR
- Gates run: `governance-release` profile checks

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`
- `.jules/friction/open/FRIC-20260611-001.md`

## 🔜 Follow-ups
- FRIC-20260611-001: Missing schema version constants in xtask bump command
