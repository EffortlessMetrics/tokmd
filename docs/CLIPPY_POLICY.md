# Clippy policy

Tokmd treats Clippy as a governed engineering surface rather than a local taste
file. The root manifest declares one Effortless Metrics lint baseline, the same lints
are tracked in `policy/clippy-lints.toml`, and repo-local exceptions live in
explicit policy ledgers. Member inheritance is staged behind this policy gate so
that a follow-up cleanup PR can make the stricter block blocking without
silently weakening it.

## Baseline

The target baseline is intentionally workspace-wide:

- panic-free production and test code (`unwrap`, `expect`, `panic!`, `todo!`,
  `unimplemented!`, `unreachable!`, and related result-collapse shapes are
  denied);
- silent-failure prevention for discarded futures, locks, must-use values,
  ignored errors, and lossy line iteration;
- AST, UTF-8, string slicing, and indexing safety for parser/reporting code;
- async, concurrency, unsafe/memory, numeric correctness, filesystem/process,
  API-correctness, and reviewability lints;
- suppression governance: blanket `#[allow]` usage is not the normal escape
  hatch.

Tests are not a carveout. Prefer tests that return `Result<(), Box<dyn
std::error::Error>>`, propagate setup failures with `?`, and use assertion
helpers that return structured errors when panic-free fixtures are needed.

## Suppression style

New suppressions must be narrow and reviewed:

```rust
#[expect(
    clippy::indexing_slicing,
    reason = "generated parser table access is tracked in policy/clippy-debt.toml"
)]
fn lookup_generated_table(...) { ... }
```

Do not add broad Clippy carveouts to `clippy.toml`, and do not enable test
carveouts such as `allow-unwrap-in-tests`, `allow-expect-in-tests`,
`allow-panic-in-tests`, `allow-indexing-slicing-in-tests`, or
`allow-dbg-in-tests`.

## Policy files

- `policy/clippy-lints.toml` is the machine-readable source of truth for active
  lints, MSRV, posture flags, and planned Rust 1.94/1.95 flips.
- `policy/clippy-debt.toml` is for temporary repo-specific debt. Each debt entry
  must include `lint`, `path`, `owner`, `reason`, and `expires`.
- `policy/clippy-exceptions.toml` is the AST-backed Clippy exception ledger.
  Every `#[expect(clippy::<lint>, reason = "policy:clippy-NNNN ...")]` attribute
  in source must link to an entry here. Validated by
  `cargo xtask check-clippy-exceptions`.
- `clippy.toml` is reserved for repo-specific `disallowed-*` policy. It must not
  weaken the workspace baseline.

## Upgrade ledger

The workspace currently targets Rust 1.95. Planned Clippy flips for future
releases are recorded before the MSRV bump so that upgrades are reviewable,
searchable, and gated by policy.

## Gate

Run the policy gate with:

```bash
cargo xtask check-lint-policy
```

The gate verifies that MSRV agrees with the policy ledger, active lint levels
match the root manifest, planned lint flips are not activated too early,
workspace members do not define repo-local lint overrides, `clippy.toml` does
not contain test carveouts, and debt entries are complete and unexpired. A
follow-up cleanup PR can then switch members to `[lints] workspace = true` once
the inherited lint debt is reduced enough for CI.
