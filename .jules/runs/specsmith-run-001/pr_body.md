## 💡 Summary
Modified existing tests in `cli_error_paths_w51.rs` to lock in the documented edge-case behavior where unrecognized subcommands fall back to the default `lang` parsing. The test now explicitly asserts the "Path not found" fallback and verifies that helpful typo suggestions are output, completing a proof-improvement patch.

## 🎯 Why
As noted in the prompt memory, `tokmd` parses unrecognized subcommands as file paths using `CliLangArgs` due to the implicit fallback for the default `lang` subcommand, emitting "Path not found" instead of clap's standard "unrecognized subcommand". Rather than attempting to break this useful fallback behavior, this patch locks in the exact edge-case errors and helpful hints, ensuring regression coverage for this important interface boundary.

## 🔎 Evidence
- `crates/tokmd/tests/cli_error_paths_w51.rs` generic assertion `.stderr(predicate::str::is_empty().not());`
- Observed behavior: `tokmd foo_unrecognized_subcommand` produces `Error: Path not found: foo_unrecognized_subcommand` alongside "intended as a subcommand" hints.
- Output from `cargo run -p tokmd -- foo_unrecognized_subcommand` showing the fallback error.

## 🧭 Options considered
### Option A (recommended)
- Update the BDD test in `cli_error_paths_w51.rs` to replace the generic assertion with specific assertions matching the exact fallback behavior and hint output.
- This fits the `Specsmith` persona perfectly, which focuses on "edge-case regression not locked in by tests" without breaking existing structural behaviors.
- Trade-offs: Increases test rigidity but successfully prevents accidental regressions in subcommand fallback parsing.

### Option B
- Refactor the clap configuration to explicitly parse subcommands or use `disable_help_subcommand`.
- This was discarded because doing so would break the core repository convention allowing `tokmd .` or `tokmd src/` to implicitly trigger `lang` analysis.

## ✅ Decision
Selected Option A. The CLI fallback is intentional, but the test lacked specific assertions. Locking in the edge case output and adding typo suggestion checks improves regression coverage and accurately documents the CLI's behavior.

## 🧱 Changes made (SRP)
- Updated `crates/tokmd/tests/cli_error_paths_w51.rs` to assert the "Path not found" fallback and "intended as a subcommand" hint.
- Added test `typo_subcommand_suggests_correction` to assert the spelling suggestions functionality in error paths.

## 🧪 Verification receipts
```text
cargo test -p tokmd --test cli_error_paths_w51
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.20s
```

## 🧭 Telemetry
- Change shape: Test/Proof improvement
- Blast radius: Test-only, confined to `tokmd` tests.
- Risk class: Low
- Rollback: Revert the test modifications.
- Gates run: `cargo test -p tokmd --test cli_error_paths_w51`, `cargo check`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith-run-001/envelope.json`
- `.jules/runs/specsmith-run-001/decision.md`
- `.jules/runs/specsmith-run-001/receipts.jsonl`
- `.jules/runs/specsmith-run-001/result.json`
- `.jules/runs/specsmith-run-001/pr_body.md`

## 🔜 Follow-ups
None.
