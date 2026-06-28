## 💡 Summary
This is a learning PR. Attempted to add executable coverage for `docs/reference-cli.md` examples, but learned that standard `unwrap()` usage in tests triggers massive cascade renumbering in the no-panic policy. The PR was declined due to baseline cost and topology issues, so we are closing with recorded friction items.

## 🎯 Why
The attempt to fix the target issue directly in the `tokmd` repo caused a baseline explosion. We need to document this failure mode so future agents can avoid it by using fallible helpers or targeting `tokmd-swarm`.

## 🔎 Evidence
- Pull request comment: `Declining this PR after review. ... Baseline cost dominates the change ... The new test-helper unwrap/expect entries are inserted mid-file and cascade-renumber every subsequent panic-NNNN id`.

## 🧭 Options considered
### Option A (recommended)
- Convert to a learning PR and record the friction.
- **Why**: The maintainer explicitly declined the code patch and requested that the task be dropped or moved to `tokmd-swarm`.

### Option B
- Refactor the code patch to use `?` and submit against `tokmd-swarm`.
- **Why not**: The instructions for this run restrict us to the current clone/shard, and the user closed the pull request as obsolete, signaling a halt.

## ✅ Decision
Chosen Option A. Recording the learning PR as requested by the pipeline rules for closed/declined patches.

## 🧱 Changes made (SRP)
- Added a friction item explaining the `unwrap()` / no-panic cascade issue.

## 🧪 Verification receipts
N/A - Learning PR only.

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None
- Risk class: Low
- Rollback: N/A
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`
- `.jules/friction/open/librarian-doctest-baseline-cost.md`

## 🔜 Follow-ups
None.
