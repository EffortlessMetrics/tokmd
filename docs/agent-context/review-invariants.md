# Review Invariants for tokmd

This document specifies the architectural and operational invariants that must be respected by code review (including Droid auto-review).

## Determinism Invariant

**Rule**: All outputs must be byte-stable and reproducible.

- Use `BTreeMap` instead of `HashMap` for all sorted collections
- Sort descending by code lines, then by name
- Normalize all paths to forward slashes (`/`) before output
- Do not use system time, random numbers, or process IDs in output
- Golden snapshot tests must pass bit-for-bit across runs

**Scope**: Applies to all CLI output and JSON receipt generation.

**Enforcement**: Golden snapshot tests in `crates/tokmd/tests/` must pass without review suppression.

---

## Schema Version Invariant

**Rule**: JSON structure changes must include version bump and documentation update.

Each receipt family has its own schema version constant:

| Family | Constant | Location |
|--------|----------|----------|
| Core receipts (lang, module, export, diff, run) | `SCHEMA_VERSION` | `tokmd-types` |
| Analysis receipts | `ANALYSIS_SCHEMA_VERSION` | `tokmd-analysis-types` |
| Cockpit receipts | `COCKPIT_SCHEMA_VERSION` | `tokmd-types` |
| Handoff manifests | `HANDOFF_SCHEMA_VERSION` | `tokmd-types` |
| Context receipts | `CONTEXT_SCHEMA_VERSION` | `tokmd-types` |
| Context bundles | `CONTEXT_BUNDLE_SCHEMA_VERSION` | `tokmd-types` |
| Tool schemas | `TOOL_SCHEMA_VERSION` | `tokmd` |

**Changes to JSON structure require**:
1. Bump the relevant schema version constant in source code
2. Update the corresponding version in `CLAUDE.md` and `agents/shared/repo.md`
3. Document the change in `docs/SCHEMA.md`
4. Update formal JSON Schema in `docs/schema.json`
5. Add snapshot test covering the new structure
6. Include migration or compatibility note in PR if needed

**Scope**: Applies to all JSON output modes (`lang`, `module`, `export`, `analyze`, `run`, `cockpit`, `handoff`, `context`).

**Enforcement**: Schema version tests in `tokmd-types` verify constants are in sync. Snapshot test failures block merge.

---

## Dependency Tier Invariant

**Rule**: Lower-numbered tiers must never import from higher-numbered tiers.

### Tier Hierarchy

```
Tier 0: Contracts and settings
  tokmd-types, tokmd-analysis-types, tokmd-settings, tokmd-envelope, tokmd-io-port

Tier 1: Core scan and aggregation
  tokmd-scan, tokmd-model, tokmd-sensor

Tier 2: Adapters and rendering
  tokmd-format, tokmd-git

Tier 3: Analysis and review orchestration
  tokmd-analysis, tokmd-cockpit, tokmd-gate

Tier 4: Library facade
  tokmd-core

Tier 5: End-user products
  tokmd, tokmd-python, tokmd-node, tokmd-wasm
```

**Rule**: Crates in Tier N may depend on crates in Tier N-1 or lower, but never on Tier N+1 or higher.

**Scope**: Applies to all `Cargo.toml` dependencies.

**Enforcement**: `cargo tree` and `cargo xtask gate` verify no upward dependencies.

---

## Feature Flag Invariant

**Rule**: Feature gates must be consistent across all dependents.

Supported feature flags:

| Flag | Purpose | Tier Gate |
|------|---------|-----------|
| `git` | Git history analysis | Gate at Tier 2+ (tokmd-git) |
| `content` | File content scanning | Gate at Tier 3+ (`tokmd-analysis` content modules) |
| `walk` | Filesystem traversal helpers | Gate at Tier 1+ (`tokmd-scan` owner modules) |
| `halstead` | Halstead metrics (requires `content` + `walk`) | Gate at Tier 3+ (`tokmd-analysis`) |

**Rules**:
- Feature gate must be applied at the crate that introduces the feature
- All transitive dependents must also gate on the same feature
- `halstead` requires both `content` and `walk` (specify in `Cargo.toml`)
- Feature-gated code must compile when feature is disabled

**Scope**: Applies to Tier 2 and all code that depends on it.

**Enforcement**: CI builds with and without each feature to verify gating.

---

## Path Normalization Invariant

**Rule**: All output paths use forward slashes (`/`) regardless of platform.

**Requirements**:
- Call `normalize_path()` before emitting any file path to output
- Module keys must be computed from normalized paths
- Internal path comparisons may use OS-native separators, but output must normalize
- Snapshot tests verify forward-slash consistency across platforms

**Scope**: Applies to all JSON output, Markdown tables, and CLI reports.

**Enforcement**: Golden snapshots fail if paths use backslashes on any platform.

---

## Output Correctness Invariant

**Rule**: Receipt structure must match documented schema, and all fields must be populated.

**Requirements**:
- Every field in the receipt JSON schema must be present in output
- Null values are allowed only where schema explicitly permits
- Line counts, file counts, and derived metrics must be accurate
- Error cases must either populate fields with zeros or return an error, never mixed

**Scope**: Applies to all JSON output modes.

**Enforcement**: Snapshot tests and formal JSON Schema validation.

---

## Git History Invariant

**Rule**: Git history analysis must respect bounds and not consume unbounded history.

**Requirements**:
- Use `--since` / `--until` to bound the commit range
- Respect user-provided `--days` or `--since` options
- Return an error if git command fails, do not crash
- Handle shallow clones gracefully

**Scope**: Applies to all code that invokes `git log`, `git diff`, or `git blame`.

**Enforcement**: Integration tests with shallow clone fixtures.

---

## Performance Invariant

**Rule**: Scan time should grow roughly linearly with codebase size.

**Targets**:
- Single-pass file walk
- Streaming JSON output where possible
- No O(n²) or higher behavior

**Scope**: Applies to the `tokei` scan loop and output generation.

**Enforcement**: Benchmarks in `Cargo.toml` with CI regression tracking.

---

## Testing Invariant

**Rule**: All new code must have corresponding tests.

**Requirements by code type**:

| Code Type | Test Type | Location |
|-----------|-----------|----------|
| Tier 0 type | Unit test | In-crate tests |
| Tier 1 aggregation logic | Property test (proptest) | In-crate tests |
| Tier 2 output rendering | Golden snapshot (insta) | Integration tests or in-crate |
| Tier 2 parser-like code | Fuzz target (libfuzzer) | `fuzz/` directory |
| CLI command | Integration test (assert_cmd) | `crates/tokmd/tests/` |
| Edge case | Regression test | Related test file |

**Snapshot tests** must cover:
- Normal case output
- Edge cases (empty repo, single file, deeply nested)
- Error conditions (missing file, invalid path)

**Scope**: Applies to all new code.

**Enforcement**: PR checks verify test coverage; Droid flags untested code.

---

## Error Handling Invariant

**Rule**: Errors must be actionable and include root cause.

**Requirements**:
- Use `anyhow::Context` to add context at each layer
- Error messages must suggest how to fix the problem
- Exit code must match error severity (1 for error, 2 for usage error)
- Do not panic on user input; return a Result instead

**Scope**: Applies to all error paths.

**Enforcement**: Integration tests verify exit codes and error messages.

---

## Approval Invariant

**Rule**: No approval without substantive validation.

**Requirements**:
- Every approval includes either:
  - Substantive validation of the code, or
  - Clear acknowledgment of scope bounds (e.g., "Approved for release-prep queue only")
- Naked LGTM without context is not acceptable
- If only surfaces were inspected, say so ("Reviewed surfaces: X, Y, Z")

**Scope**: Applies to all Droid reviews and maintainer approvals.

**Enforcement**: Droid review guidelines in `.factory/rules/droid-review.md`.

---

## References

- `agents/shared/repo.md` — Canonical shared repo guidance
- `CLAUDE.md` — Runtime-specific notes
- `.factory/rules/droid-review.md` — Droid review standards
- `.factory/skills/review-guidelines/SKILL.md` — Detailed review heuristics
- `docs/SCHEMA.md` — Receipt format documentation
- `docs/schema.json` — Formal JSON Schema
