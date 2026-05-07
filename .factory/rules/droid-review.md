# Droid Review Rules

## Standards

- **No naked LGTM**: Every approval includes substantive validation or clear acknowledgment of scope bounds.
- **No arbitrary comment cap**: Review depth matches finding severity. Suppress low-value comments; elevate critical findings.
- **No extra @mentions in Droid-generated review body**: Mention author only if direct action required; mention maintainers only if architectural approval needed.
- **Actionable findings are repair packets**: Each comment includes failure mode, fix direction, and validation approach.
- **Clean reviews include inspection record**: When no findings are emitted, explain what was inspected and why no comments were generated.
- **Evidence is split into Observed / Reported / Not verified**: Distinguish signals that came from direct analysis vs. PR description vs. unconfirmed claims.

## Finding Format

```
[P0|P1|P2] Short title

Failure mode:
[Description of what would break and how]

Why here:
[Root cause or context for this risk in this PR]

Fix direction:
[Concrete suggestion for the patch]

Validation:
[How to verify the fix works]

Confidence:
[High|Medium|Low and why]
```

## Clean Review Format

```
No actionable findings emitted.

Inspected surfaces:
[What was examined: modules, entry points, tests, etc.]

Checks performed:
[What validation was applied: style, arch, performance, security, etc.]

Why no comments:
[Explanation of what passed or what was intentionally in scope]

Residual risk:
[What was not inspected and why: external APIs, platform-specific, future-phase, etc.]

Validation signal:
  Observed:
  [Signals from code inspection, tests, CI, git history]
  
  Reported:
  [PR description claims verified]
  
  Not verified:
  [Claims in PR description that require deployment or external validation]
```

## Scope Notes

- **tokmd** is a Rust-based code inventory and analytics library with a microcrate architecture.
- Focus on determinism, performance, and safety in the inventory pipeline.
- Path normalization, schema versioning, and git-history analysis are critical invariants.
- Prefer snapshot testing with `insta` for output validation.
- Mutation testing and fuzzing are part of the validation suite.

## Special Considerations

- Droid reviews focus on code quality and correctness, not on project management or release timing.
- Architectural changes should be flagged early; surface-level refactorings can be handled in minimal review.
- Schema version bumps must align with receipt version fields.
- Feature flags (`git`, `content`, `walk`, `halstead`) affect public API surface; changes there require audit.

## Exclusions

- Grammar and typo fixes are acceptable without explicit Droid comment unless architectural docs are affected.
- Dependency updates are reviewed for security and compatibility, not for novelty.
- Formatting and linting fixes are pre-validated by CI; Droid defers to those gates.
