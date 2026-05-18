# Config Resolver Doctest Coverage

id: config_resolver_doctests
persona: librarian
style: prover
shard: interfaces
status: closed

## Problem
While investigating the `crates/tokmd/src/config/resolve/` interfaces (such as
`resolve_lang`, `resolve_export`, and `resolve_module`), the run discovered
that comprehensive executable `rust` doctests were already in place for the
targeted public config resolver interfaces.

## Evidence
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/pr_body.md`
- `.jules/personas/librarian/README.md`

## Why it matters
Forcing duplicate doctests or prose edits would have weakened output honesty.
The Librarian persona now has an explicit already-covered exit for this case.

## Done when
- [x] Librarian guidance says accurate prose plus executable coverage can end
  as a zero-drift result.
- [x] The friction item no longer sits in the open queue as an implied patch
  request.
