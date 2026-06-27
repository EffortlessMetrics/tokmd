## 💡 Summary
Added executable integration tests for the CLI examples documented in `docs/reference-cli.md`. This ensures that the examples for the `context`, `analyze`, and `gate` commands will not silently drift from the application's actual behavior.

## 🎯 Why
The documentation in `docs/reference-cli.md` provides crucial examples for end-users, but without test coverage, these examples are vulnerable to becoming outdated when CLI flags or behaviors change. This addresses the "missing executable coverage for common usage" requirement for the `Librarian` persona.

## 🔎 Evidence
- `docs/reference-cli.md` contains examples for `context`, `analyze`, and `gate` commands.
- We added `crates/tokmd/tests/cli_docs_examples.rs` to execute exactly those commands via `assert_cmd`.
- Execution receipt: `cargo test --test cli_docs_examples` resulted in `5 passed`.

## 🧭 Options considered
### Option A (recommended)
- Add integration tests mirroring CLI docs examples.
- **Why**: High velocity using existing `assert_cmd` infrastructure, strong guarantees against drift.
- **Trade-offs**: Requires manual synchronization if documentation examples are updated, but ensures failure if they diverge.

### Option B
- Custom markdown parser/executor.
- **When to choose**: When parsing arbitrary markdown blocks as shell scripts across the entire repo is desired.
- **Trade-offs**: High complexity and fragility.

## ✅ Decision
Chosen Option A. It provides the highest signal-to-noise ratio and immediately solves the missing test coverage for public interface examples.

## 🧱 Changes made (SRP)
- Added `crates/tokmd/tests/cli_docs_examples.rs` containing executable tests for `docs/reference-cli.md` examples.

## 🧪 Verification receipts
```text
cargo test --test cli_docs_examples
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
```

## 🧭 Telemetry
- Change shape: Added integration tests.
- Blast radius: None (test addition only).
- Risk class: Low (no production code changed).
- Rollback: Revert the added test file.
- Gates run: docs-executable (cargo test)

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`

## 🔜 Follow-ups
None.
