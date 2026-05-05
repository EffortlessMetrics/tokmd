## 💡 Summary
Attempted to add executable doctests to the core library workflows in `tokmd-core`, but the patch was superseded by #1592. Recording learning PR instead.

## 🎯 Why
The user requested improved factual documentation and executable example coverage for the core/config/CLI public APIs. The primary workflows (`lang_workflow`, `export_workflow`, `module_workflow`, `diff_workflow`, `analyze_workflow`) only had `/// # Example` headings without corresponding executable code blocks, increasing the risk of the documentation drifting out of sync with actual behavior. The fix was implemented but superseded during execution.

## 🔎 Evidence
- Looked at `crates/tokmd-core/src/lib.rs` and saw that public API functions lacked executable code blocks.
- `cargo test --doc -p tokmd-core` showed the existing doctests.
- PR review indicated #1592 merged equivalent coverage.

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
Selected Option B to add executable doctests. However, due to the work being superseded by #1592, the code patch is aborted and a learning PR is created instead.

## 🧱 Changes made (SRP)
- Added a friction item to track the superseded work.
- Created learning PR artifacts.

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
```

## 🧭 Telemetry
- Change shape: Learning PR.
- Blast radius: None.
- Risk class: None.
- Rollback: None.
- Gates run: None for the learning PR itself.

## 🗂️ .jules artifacts
- `.jules/friction/open/librarian_api_doctests_superseded.md`
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`

## 🔜 Follow-ups
None.
