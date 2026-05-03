## 💡 Summary
Added executable doctests to the core library workflows in `tokmd-core` to ensure the documentation stays in sync with the API behavior and does not silently drift.

## 🎯 Why
The user requested improved factual documentation and executable example coverage for the core/config/CLI public APIs. The primary workflows (`lang_workflow`, `export_workflow`, `module_workflow`, `diff_workflow`, `analyze_workflow`) only had `/// # Example` headings without corresponding executable code blocks, increasing the risk of the documentation drifting out of sync with actual behavior.

## 🔎 Evidence
- Looked at `crates/tokmd-core/src/lib.rs` and saw that public API functions lacked executable code blocks.
- `cargo test --doc -p tokmd-core` showed the existing doctests.

## 🧭 Options considered
### Option A (rejected)
- Use `sed` to replace `/// Example:` with `/// # Examples` in `crates/tokmd/src/cli/parser.rs`.
- Why it fits: Aligns clap `/// # Examples` parsing logic.
- Trade-offs: Caused formatting regressions in `docs/reference-cli.md` by crushing newlines into a single broken block, and didn't add actual test coverage.

### Option B (recommended)
- Add executable `/// ```rust` blocks to the workflow functions in `tokmd-core/src/lib.rs`.
- Why it fits: This directly satisfies the goal of improving "factual docs quality and executable examples". It allows `cargo test --doc` to verify the examples against the actual APIs.
- Trade-offs: Requires a bit more effort to construct the inputs.

## ✅ Decision
Selected Option B. Adding executable doctests ensures the examples cannot silently drift from the actual code. It provides concrete, verifiable usage examples for down-stream consumers of the library.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/lib.rs`: Added `/// ```rust ... ```` doctest blocks to `lang_workflow`, `export_workflow`, `module_workflow`, `diff_workflow`, and `analyze_workflow`.

## 🧪 Verification receipts
```text
running 14 tests
test crates/tokmd-core/src/lib.rs - analysis_facade (line 685) ... ok
test crates/tokmd-core/src/ffi.rs - ffi::run_json (line 57) ... ok
test crates/tokmd-core/src/lib.rs - (line 24) ... ok
test crates/tokmd-core/src/lib.rs - cockpit_workflow (line 599) - compile ... ok
test crates/tokmd-core/src/lib.rs - (line 42) ... ok
test crates/tokmd-core/src/lib.rs - analyze_workflow_from_inputs (line 431) ... ok
test crates/tokmd-core/src/lib.rs - export_workflow_from_inputs (line 299) ... ok
test crates/tokmd-core/src/lib.rs - export_workflow (line 264) ... ok
test crates/tokmd-core/src/lib.rs - lang_workflow_from_inputs (line 138) ... ok
test crates/tokmd-core/src/lib.rs - lang_workflow (line 113) ... ok
test crates/tokmd-core/src/lib.rs - module_workflow_from_inputs (line 213) ... ok
test crates/tokmd-core/src/lib.rs - diff_workflow (line 358) ... ok
test crates/tokmd-core/src/lib.rs - module_workflow (line 178) ... ok
test crates/tokmd-core/src/lib.rs - analyze_workflow (line 395) ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s

all doctests ran in 1.51s; merged doctests compilation took 0.95s
```

## 🧭 Telemetry
- Change shape: Documentation enhancements.
- Blast radius: API docs. Safe.
- Risk class: Low
- Rollback: `git reset HEAD~1 --hard`
- Gates run: `cargo test -p tokmd --test integration`, `cargo test --doc -p tokmd-core --all-features`, `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`

## 🔜 Follow-ups
None.
