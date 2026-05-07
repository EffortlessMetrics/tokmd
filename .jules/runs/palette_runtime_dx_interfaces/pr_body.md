## 💡 Summary
Added `--depth` as a visible alias for `--module-depth` across `module`, `export`, `context`, and `handoff` subcommands.

## 🎯 Why
Users intuitively try `tokmd module --depth 1` based on the help text (e.g., `How many path segments to include for module roots [default: 2]. Example: crates/foo/src/lib.rs (depth=2)`), but it failed with an `unexpected argument '--depth' found` error. Adding the alias improves CLI ergonomics without breaking backward compatibility.

## 🔎 Evidence
- `cargo run --bin tokmd -- module --depth 1` resulted in `error: unexpected argument '--depth' found`.
- `crates/tokmd/src/cli/parser.rs` showed `--module-depth` defined without a `--depth` alias.

## 🧭 Options considered
### Option A (recommended)
- Add `visible_alias = "depth"` to `--module-depth`.
- Fits the repo and shard by using standard clap features for improved DX.
- Trade-offs: Structure is minimal, Velocity is high, Governance is unaffected.

### Option B
- Change the argument name to `--depth` everywhere.
- Choose when backward compatibility is not a concern.
- Trade-offs: Breaks existing scripts and requires extensive documentation updates.

## ✅ Decision
Implemented Option A to improve ergonomics while preserving backward compatibility.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/cli/parser.rs`: Added `visible_alias = "depth"` to `module_depth` fields in `CliModuleArgs`, `CliExportArgs`, `CliContextArgs`, and `HandoffArgs`.
- `crates/tokmd/tests/error_handling_w70.rs`: Updated `module_depth_flag_with_non_numeric_value_fails` test to reflect the new clap error message.

## 🧪 Verification receipts
```text
cargo build --verbose
cargo test -p tokmd --verbose
cargo run --bin tokmd -- module --depth 1
cargo run --bin tokmd -- export --depth 1
```

## 🧭 Telemetry
- Change shape: Feature addition (alias).
- Blast radius: API (CLI arguments).
- Risk class: Low, only adds an alias.
- Rollback: Revert the parser.rs changes.
- Gates run: `cargo build --verbose`, `cargo test -p tokmd --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`.

## 🗂️ .jules artifacts
- `.jules/runs/palette_runtime_dx_interfaces/envelope.json`
- `.jules/runs/palette_runtime_dx_interfaces/decision.md`
- `.jules/runs/palette_runtime_dx_interfaces/receipts.jsonl`
- `.jules/runs/palette_runtime_dx_interfaces/result.json`
- `.jules/runs/palette_runtime_dx_interfaces/pr_body.md`

## 🔜 Follow-ups
None.
