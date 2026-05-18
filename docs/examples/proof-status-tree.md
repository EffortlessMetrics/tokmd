# Proof Status Tree

Use this when your job is:

```text
Read CI proof evidence.
```

Run first:

```bash
cargo xtask affected \
  --base origin/main \
  --head HEAD \
  --json-output target/proof/affected.json

cargo xtask proof \
  --profile affected \
  --base origin/main \
  --head HEAD \
  --plan \
  --plan-json target/proof/proof-plan.json \
  --evidence-json target/proof/proof-evidence.json
```

Sample layout:

```text
target/proof/
  affected.json
  proof-plan.json
  proof-evidence.json
  proof-run-summary.json
  proof-run-artifacts-check.json
  executor-summary.json
  executor-manifest.json
  proof-artifacts-check.json

target/proof-observations/
  proof-observation-decision.json
  proof-observation-decision.md
  proof-observation-decision-check.json
```

Open first:

1. `target/proof/affected.json`
2. `target/proof/proof-plan.json`
3. `target/proof/proof-evidence.json`
4. `target/proof-observations/proof-observation-decision.md`

What each file owns:

| File | Owns |
| --- | --- |
| `affected.json` | Changed files, matched scopes, and unknown files. |
| `proof-plan.json` | Planned required and advisory commands. |
| `proof-evidence.json` | Planned evidence state for proof families. |
| `proof-run-summary.json` | Executed required proof commands, when run. |
| `proof-run-artifacts-check.json` | Verifier receipt for executed required proof summary. |
| `executor-summary.json` | Advisory executor command status. |
| `proof-observation-decision.md` | Human-readable observation/promotion-readiness summary. |
| `proof-observation-decision-check.json` | Verifier receipt for the observation decision packet. |

What not to infer:

- A proof plan does not mean proof ran.
- Advisory scoped coverage and mutation are not gates by default.
- Codecov upload is not enabled by default.
- Promotion criteria are evidence for a maintainer decision, not an automatic
  policy change.

Next action:

- Resolve unknown files before trusting scoped proof routing.
- Run required proof when the PR needs executed evidence.
- Verify source receipts before relying on a status or decision packet.
