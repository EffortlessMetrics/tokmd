# Friction Item

id: mutant_high_value
persona: Mutant
style: Prover
shard: core-pipeline
status: closed

## Problem
During the assignment `mutant_high_value`, the run was prompted to target a
high-value core surface and provide mutation-style proof improvements.

However, `cargo mutants -p tokmd-types` showed 25 mutants tested, 21 caught,
and 4 unviable. Exactly 0 mutants were missed.

Attempting to enforce a test patch anyway caused a violation of the output
honesty constraint by forcing a fake fix on tests that did not improve mutation
coverage.

## Evidence
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`
- `.jules/personas/mutant/README.md`

## Why it matters
Mutation prompts should improve real proof gaps. When a selected surface is
already structurally tight, the honest result is the mutation receipt plus a
zero-drift outcome, not a fabricated proof-improvement patch.

## Done when
- [x] Mutant guidance explicitly permits a zero-drift exit when targeted
  mutation evidence reports zero missed mutants.
- [x] The friction item no longer sits in the open queue as an implied patch
  request.
