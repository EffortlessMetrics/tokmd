## 💡 Summary
This is a learning PR. I initially added missing doctests to the `get_profile_name` and `resolve_profile` functions in the config interface. However, the PR was closed because it was superseded by concurrent work in #1592. I am submitting this packet to record the effort and friction.

## 🎯 Why
The `config.rs` facade lacked doctests for its CLI configuration resolution logic. Implementing these doctests was aimed at locking down expected fallback behaviors to prevent factual drift. While the code changes were successful, they became obsolete due to concurrent changes.

## 🔎 Evidence
- The PR was superseded by #1592.

## 🧭 Options considered
### Option A (recommended)
- Revert code changes and submit as a learning PR to document the friction and preserve run artifacts without polluting the main branch with redundant work.

## ✅ Decision
I have reverted the code changes to `crates/tokmd/src/config.rs` and created a friction item to document the obsolescence. Submitting this as a learning PR.

## 🧱 Changes made (SRP)
- Recorded a friction item regarding obsolete work.

## 🧪 Verification receipts
None.

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None
- Risk class: None
- Rollback: None
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/run-librarian_api_doctests/envelope.json`
- `.jules/runs/run-librarian_api_doctests/decision.md`
- `.jules/runs/run-librarian_api_doctests/receipts.jsonl`
- `.jules/runs/run-librarian_api_doctests/result.json`
- `.jules/runs/run-librarian_api_doctests/pr_body.md`
- `.jules/friction/open/obsolete-doctest.md`

## 🔜 Follow-ups
None.
