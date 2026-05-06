## 💡 Summary
This is a learning PR documenting a friction item. Attempting to standardize `clap` docstrings to `/// # Examples` in `parser.rs` fixes markdown drift but unfortunately renders the literal `#` in user-facing CLI help outputs. The run falls back to documenting this friction as no safe fix is actionable.

## 🎯 Why
The 'Fuzzer' persona was assigned to harden input surfaces, specifically `parser.rs`. The code was observed to use non-standard `/// Examples:` headers which causes `rustdoc` drift when `cargo xtask docs --check` is run. The initial fix correctly updated the headers to `/// # Examples`, but a PR review found this injects a literal `#` into the generated CLI help. A friction item must be recorded to document this conflict between rustdoc format validation and clap parsing. Additionally, `cargo-fuzz` was missing in the environment, blocking the alternative target.

## 🔎 Evidence
- `crates/tokmd/src/cli/parser.rs` contains `/// Examples:` at line 56 and `/// Example:` at line 297.
- Modifying these to `/// # Examples` and regenerating the snapshots explicitly shows the `# Examples` appearing in `tokmd --help`.
- See the newly created friction item: `.jules/friction/open/fuzzer_clap_doc_headers.md`.

## 🧭 Options considered
### Option A (rejected)
- Change `/// Examples:` to `/// # Examples` in `tokmd/src/cli/parser.rs`.
- Trade-offs: Fixes `xtask` docs markdown drift but severely degrades user experience by showing a literal hash mark (`#`) in CLI help outputs, making it unmergeable.

### Option B (rejected)
- Attempt to use `cargo fuzz`.
- Trade-offs: `cargo-fuzz` lacks ASAN capabilities in the native environment and exits with compilation errors, preventing it from yielding valid fuzz targets without substantial toolchain adjustments.

## ✅ Decision
Neither code option was safely viable. We chose to submit a Learning PR, capturing the `.jules` artifacts and recording a friction item documenting the `clap` struct docstring conflict so future runs understand this edge case.

## 🧱 Changes made (SRP)
- `.jules/friction/open/fuzzer_clap_doc_headers.md`

## 🧪 Verification receipts
```text
{"command": "cat .jules/friction/open/fuzzer_clap_doc_headers.md", "exit_code": 0}
```

## 🧭 Telemetry
- Change shape: Friction documentation.
- Blast radius: Zero code changes. Only writes local `.jules` friction and run packets.
- Risk class: No risk.
- Rollback: Revert the PR safely at any time.
- Gates run: `cargo xtask docs --check`, `cargo test -p tokmd --test cli_snapshot_golden`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening_1/envelope.json`
- `.jules/runs/fuzzer_input_hardening_1/decision.md`
- `.jules/runs/fuzzer_input_hardening_1/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening_1/result.json`
- `.jules/runs/fuzzer_input_hardening_1/pr_body.md`
- `.jules/friction/open/fuzzer_clap_doc_headers.md`

## 🔜 Follow-ups
Address the `clap` struct docstring conflict or instruct future formatting rules to avoid generating a literal `#` in `clap` output.
