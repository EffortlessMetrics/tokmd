# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Replaces multiple unprotected `.unwrap()` calls with `.expect("...")` for static `Regex::new` allocations in `tokmd-content`. This adheres to the goal of burning down undocumented panics while preserving performance.

## 🎯 Why / Threat model
Static regex patterns are known at compile time and should never fail at runtime, but naked `unwrap()` calls pollute the codebase with unhandled panic sites. Replacing them with `expect()` clearly signals to reviewers and developers that the panic is an intentional, documented invariant rather than an oversight.

## 🔎 Finding (evidence)
- `crates/tokmd-content/src/complexity.rs`:
- Unhandled `unwrap()`s observed on `LazyLock::new(|| Regex::new(...).unwrap())`

## 🧭 Options considered
### Option A (recommended)
- Replace `.unwrap()` with `.expect("Static regex ... must compile")`.
- Why it fits this repo: Addresses the panic burn-down goal by explicitly documenting the safety invariant while maintaining performance (regexes remain compiled statically).
- Trade-offs: Still technically a panic, but it is now intentional.

### Option B
- Refactor the codebase to return `Result<Regex>` or use error propagation.
- When to choose it instead: When the pattern is dynamic or failure is recoverable.
- Trade-offs: Incurs unnecessary structural complexity for hardcoded strings.

## ✅ Decision
Option A was chosen. It maintains tight scope (SRP) and directly addresses the panic burn-down objective for static invariants.

## 🧱 Changes made (SRP)
- `crates/tokmd-content/src/complexity.rs`: Replaced 6 `.unwrap()` instances with explicit `.expect("...")` messages.

## 🧪 Verification receipts
```json
[
  {
    "cmd": "cargo build --verbose",
    "exit_status": "0",
    "summary": "PASS",
    "key_lines": "..."
  },
  {
    "cmd": "cargo test -p tokmd-content --verbose",
    "exit_status": "0",
    "summary": "PASS",
    "key_lines": "\nrunning 4 tests\ntest crates/tokmd-content/src/complexity.rs - complexity::analyze_nesting_depth (line 1166) ... ok\ntest crates/tokmd-content/src/complexity.rs - complexity::analyze_functions (line 119) ... ok\ntest crates/tokmd-content/src/complexity.rs - complexity::estimate_cognitive_complexity (line 824) ... ok\ntest crates/tokmd-content/src/complexity.rs - complexity::estimate_cyclomatic_complexity (line 403) ... ok\n\ntest result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s"
  },
  {
    "cmd": "cargo fmt",
    "exit_status": "0",
    "summary": "PASS",
    "key_lines": "Formatted"
  },
  {
    "cmd": "cargo clippy -p tokmd-content -- -D warnings",
    "exit_status": "0",
    "summary": "PASS",
    "key_lines": "    Checking tokmd-content v1.8.1 (/app/crates/tokmd-content)\n    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.22s"
  }
]
```

## 🧭 Telemetry
- Change shape: Quality (panic reduction)
- Blast radius (API / IO / config / schema / concurrency): Localized to internal regex compilation.
- Risk class + why: Extremely low; only replaces string allocation methods for identical logic.
- Rollback: Revert PR.
- Merge-confidence gates (what ran): build, test, clippy, fmt.

## 🗂️ .jules updates
- Appended a run envelope to `.jules/security/envelopes/`.
- Written decision documentation to `.jules/security/runs/`.
- Appended a ledger entry to `.jules/security/ledger.json`.

## 📝 Notes (freeform)
N/A
