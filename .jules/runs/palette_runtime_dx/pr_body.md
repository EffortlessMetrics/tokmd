## 💡 Summary
Added parse-time validation to `--max-commits` and `--max-commit-files` in `tokmd analyze` and `tokmd badge` to reject zero values.

## 🎯 Why
Previously, passing `--max-commits 0` to `analyze` or `badge` was allowed by the CLI parser, but it would either silently do nothing or cause confusing errors downstream. `tokmd context` already used `value_parser = super::validate::positive_usize` to catch this at parse time. This change brings `analyze` and `badge` in line with that pattern, providing a clear error message to the user immediately.

## 🔎 Evidence
- `crates/tokmd/src/cli/parser/analysis.rs`
- `crates/tokmd/src/cli/parser/badge.rs`
- Passing `--max-commits 0` now produces a helpful parse error.

## 🧭 Options considered
### Option A (recommended)
- Add `value_parser = super::validate::positive_usize` to the clap arguments.
- Matches existing repo patterns (`context.rs`), keeps errors at the system boundary, and provides excellent DX.
- Trade-offs: Small codebase addition for high user value.

### Option B
- Handle `0` deep in the git scanning logic.
- Delays the error, making it harder for the user to understand which flag caused the problem.

## ✅ Decision
Option A. It's the standard clap way to handle this and aligns perfectly with the goal of improving CLI ergonomics and error messaging.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/cli/parser/analysis.rs`
- `crates/tokmd/src/cli/parser/badge.rs`

## 🧪 Verification receipts
```text
$ bash -c "cargo run --bin tokmd -- analyze --max-commits 0"
error: invalid value '0' for '--max-commits <MAX_COMMITS>': value must be at least 1
```

## 🧭 Telemetry
- Change shape: CLI argument validation tightening.
- Blast radius: Low. Only affects users who were passing invalid `0` values to these specific flags.
- Risk class: Low.
- Rollback: Revert the added `value_parser` attributes.
- Gates run: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/palette_runtime_dx/envelope.json`
- `.jules/runs/palette_runtime_dx/decision.md`
- `.jules/runs/palette_runtime_dx/receipts.jsonl`
- `.jules/runs/palette_runtime_dx/result.json`
- `.jules/runs/palette_runtime_dx/pr_body.md`

## 🔜 Follow-ups
None.
