## 💡 Summary
Fixed a logic gap in `module_key` where only a single `./` prefix was stripped. Using `while let` robustly handles paths with multiple leading `./` components (e.g. `././src`). Added test coverage to lock in this behavior.

## 🎯 Why
While investigating `module_key` in `crates/tokmd-model/src/module_key/mod.rs` as a high-value core surface, I discovered that its path normalization only handled a single leading `./` prefix via an `if let` check. A path like `././src/main.rs` would only have the first `./` stripped, resulting in `./src/main.rs`. Then `p.trim_start_matches('/')` would leave the path as `./src/main.rs`, and a subsequent string split operations would incorrectly yield `.` as the module key instead of the expected `src`. This patch addresses this missing assertion gap.

## 🔎 Evidence
File path: `crates/tokmd-model/src/module_key/mod.rs`
Observed behavior: `module_key("././src/main.rs", &[], 1)` incorrectly returned `.` due to unstripped trailing `./`.

Commands run:
- Wrote a local test script to observe the behavior of `module_key("././src/main.rs")` before and after the fix.
- Ran `cargo test -p tokmd-model -- module_key_multiple_dot_slash_stripped` to verify the new test passes.

## 🧭 Options considered
### Option A (recommended)
- Change the `if let` to a `while let` loop to continuously strip `./` prefixes until none remain.
- Why it fits: It is a simple, deterministic, and localized fix inside the `core-pipeline` shard.
- Trade-offs: Structure is solid, velocity is high, and governance is low-impact.

### Option B
- Document the behavior as a known limitation via a learning PR.
- When to choose: If fixing it introduced a massive behavioral change that broke tests.
- Trade-offs: Would leave a real bug in a core function without test coverage.

## ✅ Decision
I chose **Option A** because it is a low-risk, concrete improvement to an important path normalization function and correctly closes an assertion gap, perfectly aligning with the `Mutant` persona.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/module_key/mod.rs`: Changed `if let` to `while let` for stripping `./` prefixes.
- `crates/tokmd-model/src/module_key/mod.rs`: Added `module_key_multiple_dot_slash_stripped` test.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-model -- module_key_multiple_dot_slash_stripped
test module_key::tests::module_key_multiple_dot_slash_stripped ... ok

$ CI=true cargo test -p tokmd-model
test result: ok. 512 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🧭 Telemetry
- Change shape: Bug fix + Test Coverage
- Blast radius: API (internal module grouping)
- Risk class: Low - strengthens normalization, unlikely to break valid paths.
- Rollback: Revert `crates/tokmd-model/src/module_key/mod.rs`.
- Gates run: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`, `cargo xtask check-file-policy --strict`.

## 🗂️ .jules artifacts
- `.jules/runs/mutant-1/envelope.json`
- `.jules/runs/mutant-1/decision.md`
- `.jules/runs/mutant-1/receipts.jsonl`
- `.jules/runs/mutant-1/result.json`
- `.jules/runs/mutant-1/pr_body.md`

## 🔜 Follow-ups
None.
