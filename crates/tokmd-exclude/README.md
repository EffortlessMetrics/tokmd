# tokmd-exclude

Single-responsibility microcrate for deterministic exclude-pattern handling.

## API

- `normalize_exclude_pattern(root, path)` - normalize path separators, strip `./`,
  and make absolute paths root-relative when possible.
- `has_exclude_pattern(existing, pattern)` - membership check with normalized
  matching (slash and `./` insensitive).
- `add_exclude_pattern(existing, pattern)` - push only when the normalized pattern
  is non-empty and not already present.
