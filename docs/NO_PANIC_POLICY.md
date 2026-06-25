# No-panic policy

Tokmd treats panic-family behavior as governed engineering debt rather than a
local code-style preference. The goal is *no unreceipted panic-family behavior
in production or tests*. The policy is enforced by two complementary rails:

1. **Clippy** (rail A) catches panic-family shapes close to the code, with full
   editor and IDE feedback. Workspace-level `clippy::unwrap_used`,
   `clippy::expect_used`, `clippy::panic`, `clippy::todo`,
   `clippy::unimplemented`, `clippy::unreachable`, `clippy::indexing_slicing`,
   `clippy::string_slice`, `clippy::get_unwrap`, and friends are denied. There
   are no test carveouts (`allow-unwrap-in-tests` and similar are explicitly
   forbidden).
2. **The semantic checker** (rail B), `cargo xtask check-no-panic-family`, is
   the authoritative exception mechanism. It parses every workspace `.rs` file
   with `syn`, finds panic-family expressions, and matches each finding against
   `policy/no-panic-allowlist.toml`. Stale and expired entries fail the gate in
   both modes; findings without an allowlist entry become blocking when the
   checker is run with `--strict`.

## Families

The semantic checker currently classifies the following families:

| Family             | Detected shape                                                |
|--------------------|---------------------------------------------------------------|
| `unwrap`           | method call `.unwrap()`                                       |
| `expect`           | method call `.expect(...)`                                    |
| `get_unwrap`       | method call `.get_unwrap(...)`                                |
| `panic_macro`      | macro invocation `panic!(...)`                                |
| `todo`             | macro invocation `todo!(...)`                                 |
| `unimplemented`    | macro invocation `unimplemented!(...)`                        |
| `unreachable`      | macro invocation `unreachable!(...)`                          |
| `element_indexing` | indexing expression `x[idx]` with a non-range index           |
| `range_indexing`   | indexing expression `x[a..b]`                                 |

Without type information, the semantic checker cannot prove that a range
index targets a `&str`/`String` (Clippy's `string_slice` lint does that with
type info on rail A), so range and element indexing are reported as separate
families and left to rail A for the type-narrowed call.

Assertion macros (`assert!`, `assert_eq!`, `debug_assert!`, …) are not yet
classified; making them part of the no-panic ledger is a separate decision and
requires fallible test helpers to land first.

## Identity

Every allowlist entry is keyed by

```text
identity = path + family + selector
```

where `selector` is the four-tuple `(kind, container, callee, receiver_fingerprint)`:

* `kind` — `method_call`, `macro_invocation`, or `indexing`.
* `container` — the textual name of the enclosing function (or `<top>` if at module
  scope). For methods, the impl type is included as a `Type::method` prefix.
* `callee` — the method name, the macro path, or `[]` for indexing.
* `receiver_fingerprint` — the textual form of the receiver expression, with
  whitespace collapsed.

Line and column are recorded under `[allow.last_seen]` for review only. They are
never part of the identity, so re-formatting source files does not invalidate
allowlist entries.

## Allowlist schema

```toml
schema_version = "0.3"

[[allow]]
id = "panic-0001"
path = "crates/tokmd-format/src/render.rs"
family = "unwrap"
classification = "test_helper"   # production | test_helper | fixture | tooling
owner = "format"
explanation = "Fixture setup helper; migrate to fallible fixture builder."
expires = "2026-07-01"           # ISO-8601, future date

[allow.selector]
kind = "method_call"
container = "renders_boundary_fixture"
callee = "unwrap"
receiver_fingerprint = "std::fs::read_to_string(path)"

[allow.last_seen]
line = 42
column = 17
```

## Workflow

1. Try to remove the panic-family call. The first response to a finding is
   *make it fallible*, not *receipt it*.
2. If the panic is intentional and bounded (a hard invariant, a fixture
   helper, a one-shot CLI exit), run

   ```bash
   cargo xtask no-panic-propose
   ```

   to write `target/no-panic-proposed-allowlist.toml` with selector blocks for
   every current unallowlisted finding.
3. Copy the relevant block(s) into `policy/no-panic-allowlist.toml` and fill in
   `owner`, `classification`, `explanation`, and a future `expires` date. Use
   `id = "panic-NNNN"` with a four-digit zero-padded counter that is unique
   across the file.
4. In source code, pair the entry with a narrow

   ```rust
   #[expect(
       clippy::unwrap_used,
       reason = "policy:no-panic:panic-0001 — fixture builder, expires 2026-07-01"
   )]
   ```

   so reviewers can reach the receipt from the suppression site.
5. Run `cargo xtask check-no-panic-family`. Stale entries (selector no longer
   matches a finding) and expired entries fail the gate, so removed code or
   missed expirations are caught.

## Anti-patterns

* Do not enable test carveouts in `clippy.toml`.
* Do not write `#[allow(clippy::unwrap_used)]`; use `#[expect(...)]` with a
  non-empty `reason` and a `policy:no-panic:<id>` link.
* Do not edit `[allow.last_seen]` by hand. The checker treats it as advisory.
* Do not rely on path+line+column identity. Selector identity is path + family
  + (kind, container, callee, receiver_fingerprint).

## Gate and staging

```bash
cargo xtask check-no-panic-family            # advisory (default)
cargo xtask check-no-panic-family --strict   # blocking (CI mode)
cargo xtask no-panic-baseline                # regenerate receipted ledger
cargo xtask check-no-panic-family --json-output target/tokmd/reports/no-panic-report.json
```

The gate runs in two modes:

* **Advisory (default)** — schema/shape errors, expired allowlist entries, and
  stale entries (allowlist entries with no matching finding) are blocking.
  Unallowlisted findings are reported but do not fail the gate.
* **Strict (`--strict`)** — every unallowlisted finding is also blocking.
  CI and the `no_panic_policy` proof scope run strict mode after the receipted
  baseline landed in `policy/no-panic-allowlist.toml`.

Regenerate the baseline when panic-family debt changes:

```bash
cargo xtask no-panic-baseline --receipt-output policy/no-panic-baseline-receipt.json
cargo xtask check-no-panic-family --strict
```

The checker is wired into the `no_panic_policy` proof scope in
`ci/proof.toml` and runs as a strict job in
`.github/workflows/no-panic-policy.yml` on every PR. The workflow writes the
uploaded JSON report through `--json-output` so the report path is owned by the
Rust checker instead of shell stdout redirection. Baseline stats live in
`policy/no-panic-baseline-receipt.json`.

## Family taxonomy

The current detector recognises the following families
(see `xtask/src/tasks/no_panic.rs :: Family`):

| Family | Surface |
|--------|---------|
| `unwrap` | `.unwrap()` method calls. |
| `expect` | `.expect("…")` method calls. |
| `get_unwrap` | `.get(idx).unwrap()` chains. |
| `panic_macro` | `panic!(…)`. |
| `todo` | `todo!()`. |
| `unimplemented` | `unimplemented!()`. |
| `unreachable` | `unreachable!()`. |
| `element_indexing` | `slice[idx]` element indexing. |
| `range_indexing` | `slice[a..b]` range indexing. |

Future families considered in the rollout plan but not yet implemented:
`string_slice`, `unchecked_time_subtraction`. These can be added by
extending `Family` and the AST visitor in `xtask/src/tasks/no_panic.rs`.

## Adding entries

```bash
cargo xtask no-panic-propose
cargo xtask no-panic-baseline --receipt-output policy/no-panic-baseline-receipt.json
```

`no-panic-propose` writes proposed allowlist entries to
`target/no-panic-proposed-allowlist.toml`. Edit each proposed entry to
fill in `owner`, `classification`, `explanation`, and `expires`, then
move the entry into `policy/no-panic-allowlist.toml`.

`no-panic-baseline` materializes the full receipted ledger from current
findings (deduplicated by selector identity) and is the supported way to
refresh the committed baseline after debt changes. Re-run
`cargo xtask check-no-panic-family` to confirm the receipt covers the
finding.
