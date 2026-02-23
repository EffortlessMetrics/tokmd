# tokmd-context-policy

Single-responsibility microcrate for deterministic context/handoff policy rules.

## API

- `smart_exclude_reason(path)` - classify lockfiles/minified files/sourcemaps for
  payload exclusion.
- `is_spine_file(path)` - match must-include project spine files.
- `classify_file(path, tokens, lines, dense_threshold)` - deterministic file hygiene
  classification.
- `compute_file_cap(budget, max_file_pct, max_file_tokens)` - per-file token cap
  calculation.
- `assign_policy(tokens, file_cap, classifications)` - decide `InclusionPolicy`
  plus reason text.
