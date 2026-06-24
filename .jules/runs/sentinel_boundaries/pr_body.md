## 💡 Summary
Hardens the FFI trust boundary in `tokmd-core` by strictly validating that nested configuration payloads (e.g., `lang`, `scan`) are valid JSON objects. Previously, malformed types like strings were silently ignored and fell back to the root `args` object.

## 🎯 Why
Untrusted JSON payloads hitting the FFI layer were silently bypassing nested property validation due to a liberal `unwrap_or(args)` pattern. If a user provided `"lang": "string"`, the parser would silently ignore the string, drop the nested configuration attempt, and fall back to scanning the root arguments. This is a vulnerability in boundary determinism. Explicitly validating `is_object()` before extracting properties ensures that the parsing layer predictably enforces contract requirements.

## 🔎 Evidence
- File paths: `crates/tokmd-core/src/ffi/parse.rs`, `crates/tokmd-core/src/ffi/settings_parse.rs`
- Finding: `args.get("lang").unwrap_or(args)` returns the root object if `"lang"` is a string, completely bypassing the intended configuration and any type enforcement for the nested field.
- Proof: `cargo test -p tokmd-core --test ffi_trust_boundary_w80` passes, demonstrating that providing `"lang": "string"` now returns a strict `invalid_field` error rather than silently succeeding.

## 🧭 Options considered
### Option A (recommended)
- Strictly validate nested JSON objects at the FFI boundary using an explicit `extract_nested_object` helper that verifies `is_object()`.
- Replaces the unsafe `unwrap_or` pattern across all FFI settings parsers.
- Trade-offs: Structure is improved by centralizing validation; Velocity is slightly impacted by deeper parsing checks; Governance enforces strict deterministic schema behavior.

### Option B
- Ignore the loose validation and assume callers provide correct structures.
- When to choose: Never, untrusted inputs must be validated.
- Trade-offs: Leaves a gap where clients might think their configuration applied, but it silently fell back to defaults.

## ✅ Decision
Option A. Enforcing strict JSON type boundaries on untrusted payloads prevents unpredictable fallback behavior.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/ffi/parse.rs`: Added `extract_nested_object` to strictly validate nested JSON objects.
- `crates/tokmd-core/src/ffi/settings_parse.rs`: Migrated all config parsers (`lang`, `scan`, `module`, etc.) to use the strict object extraction.
- `crates/tokmd-core/src/ffi/inputs.rs`: Updated `nested_inputs` logical checks to safely interact with the new bounded parsing logic.
- `crates/tokmd-core/tests/ffi_trust_boundary_w80.rs`: Added a targeted test proving the silent failure has been mitigated.

## 🧪 Verification receipts
```text
cargo test -p tokmd-core --test ffi_trust_boundary_w80
cargo test -p tokmd-core --test ffi_in_memory_w81
cargo build --verbose -p tokmd-core
cargo fmt -- --check
cargo clippy -p tokmd-core -- -D warnings
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: FFI payload parsing boundary
- Risk class: Low - strengthens validation, but requires callers emitting malformed strings in config blocks (which was previously ignored) to fix their inputs.
- Rollback: Revert the PR
- Gates run: `security-boundary` fallback expectations (cargo test, fmt, clippy, build)

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_boundaries/envelope.json`
- `.jules/runs/sentinel_boundaries/decision.md`
- `.jules/runs/sentinel_boundaries/receipts.jsonl`
- `.jules/runs/sentinel_boundaries/result.json`
- `.jules/runs/sentinel_boundaries/pr_body.md`

## 🔜 Follow-ups
None.
