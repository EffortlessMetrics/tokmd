## 💡 Summary
Added a new test file `crates/tokmd/tests/determinism_hash_rule.rs` to enforce the "no HashMap/HashSet" rule in the core pipeline crates. This prevents non-deterministic outputs which break golden snapshot tests.

## 🎯 Why
The core pipeline (`tokmd-types`, `tokmd-scan`, `tokmd-model`, `tokmd-format`) has a strict determinism contract. Relying on `HashMap` or `HashSet` in these areas often leads to unpredictable outputs, flaking golden snapshot tests and test failures across OSes. We want to programmatically catch this drift before it's merged.

## 🔎 Evidence
- `crates/tokmd-types/tests/determinism_props.rs` explicitly mentions "BTreeMap as collection type never HashMap".
- `crates/tokmd/tests/determinism_hash_rule.rs` was added to run a source scan to find violations.
- An initial grep confirmed no violations currently exist in the core crates.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add an integration test that checks the source code of the core pipeline for `HashMap` and `HashSet`. We use a native rust file-system walk instead of shelling out to avoid OS-specific dependency failures (like `grep` missing on Windows) and resolve paths properly from the crate root.
- why it fits this repo and shard: It protects determinism directly, enforcing the rules mentioned in the comments. It fits the Gatekeeper persona's focus on deterministic output boundaries and contract protection.
- trade-offs: Structure / Velocity / Governance: It relies on a source scan instead of AST analysis, which is simple and robust.

### Option B
- what it is: Add a custom Clippy lint or use the `disallowed-types` in `clippy.toml`.
- when to choose it instead: If we want to ban `HashMap` globally. Since we only want to strictly enforce it for the core pipeline (and other crates like `tokmd-analysis` are already safely using `HashMap` in their tests or components where ordering is not as strict or matters less), an explicit directory scoped test provides a surgical, less-disruptive barrier.
- trade-offs: `clippy.toml` applies workspace-wide or is awkward to apply to exactly these 4 specific crates without creating false positives elsewhere.

## ✅ Decision
I chose Option A, to provide immediate robust enforcement of the determinism contract without global disruption or OS-dependency issues.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/determinism_hash_rule.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: New test file
- Blast radius: Tests only. No runtime behavior change.
- Risk class + why: Very low risk.
- Rollback: Revert the added test file.
- Gates run: `cargo test -p tokmd`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`

## 🔜 Follow-ups
None.
