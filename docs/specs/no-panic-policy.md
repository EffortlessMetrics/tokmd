# Spec: No-Panic Policy

- Status: active
- Schema family, if any: `policy/no-panic-allowlist.toml` schema version `0.3`
- Related ADRs: none
- Related proof scopes: `no_panic_policy`, `project_truth_docs`,
  `proof_control_plane`

## Contract

The no-panic policy treats panic-family behavior as governed engineering debt,
not as a local style preference. The durable goal is no unreceipted
panic-family behavior in production code or tests.

The policy has two complementary rails:

- Rail A is Clippy. Workspace lint policy catches panic-family shapes close to
  the code and must not use test carveouts for panic-family lints.
- Rail B is `cargo xtask check-no-panic-family`. The semantic checker parses
  workspace Rust files, classifies panic-family findings, and matches each
  finding against `policy/no-panic-allowlist.toml`.

The semantic checker is the authoritative exception mechanism. An exception is
valid only when the finding identity matches an allowlist entry with a valid
shape, non-expired review date, non-empty owner, non-empty explanation, accepted
classification, and selector fields that still match a current finding.

The checker has two modes:

- Advisory mode is the default. Schema, shape, stale-entry, and expired-entry
  failures are blocking. Unallowlisted findings are reported but are not
  blocking.
- Strict mode is enabled with `--strict`. It keeps the advisory failures and
  also makes every unallowlisted finding blocking.

This spec records current behavior. It does not flip the default checker mode,
promote no-panic to a stricter required check, change Clippy configuration, or
change any public `tokmd` receipt schema.

## Inputs

The policy consumes checked repository state:

| Input | Owner | Used for |
| --- | --- | --- |
| `policy/no-panic-allowlist.toml` | Machine policy | Receipted panic-family exceptions and allowlist schema version. |
| Workspace `*.rs` files | Source code | Semantic panic-family finding discovery. |
| `xtask/src/tasks/no_panic.rs` | Checker implementation | Family taxonomy, selector identity, report shape, and gate behavior. |
| `docs/NO_PANIC_POLICY.md` | User guide | Contributor workflow, examples, and rollout guidance. |
| `.github/workflows/no-panic-policy.yml` | GitHub Actions workflow | Hosted advisory report generation and artifact upload. |
| `ci/proof.toml` `no_panic_policy` scope | Proof policy | Local proof commands for checker and workflow changes. |

The semantic checker must not depend on line and column for matching. Line and
column are review metadata only.

## Findings

The checker currently classifies these families:

| Family | Detected surface |
| --- | --- |
| `unwrap` | `.unwrap()` method calls. |
| `expect` | `.expect(...)` method calls. |
| `get_unwrap` | `.get(...).unwrap()` chains. |
| `panic_macro` | `panic!(...)` macro invocations. |
| `todo` | `todo!(...)` macro invocations. |
| `unimplemented` | `unimplemented!(...)` macro invocations. |
| `unreachable` | `unreachable!(...)` macro invocations. |
| `element_indexing` | Element indexing such as `value[index]`. |
| `range_indexing` | Range indexing such as `value[start..end]`. |

Assertion macros are outside the current taxonomy. Adding them to the policy
requires a separate contract change because it affects test-helper design and
the expected fallible-test workflow.

## Allowlist Identity

Every allowlist entry is keyed by:

```text
identity = path + family + selector
```

The selector is the tuple:

```text
kind + container + callee + receiver_fingerprint
```

Selector fields have these meanings:

- `kind`: `method_call`, `macro_invocation`, or `indexing`.
- `container`: enclosing function or method name, or `<top>` for module scope.
- `callee`: method name, macro path, or `[]` for indexing.
- `receiver_fingerprint`: normalized receiver text with collapsed whitespace.

Matching must use identity, not source location. A location-only match is not a
stable receipt because formatting, inserted code, or unrelated edits can move a
finding without changing its meaning.

## Allowlist Entries

Each `[[allow]]` entry must include:

- `id`, using the local `panic-NNNN` convention;
- `path`, as a repo-relative Rust source path;
- `family`, from the current taxonomy;
- `classification`, currently one of `production`, `test_helper`, `fixture`,
  `tooling`, or `ffi`;
- `owner`;
- `explanation`;
- `expires`, as a future ISO-8601 date;
- `[allow.selector]` with non-empty identity fields;
- optional `[allow.last_seen]` review metadata.

An expired entry is a failure in advisory and strict modes. A stale entry, where
the allowlist identity no longer matches any current finding, is also a failure
in advisory and strict modes.

## Outputs

The checker produces a human summary and exit code. The summary includes counts
for total findings, matched findings, unallowlisted findings, stale entries,
expired entries, and shape errors.

The checker can also emit machine-readable JSON:

```bash
cargo xtask check-no-panic-family --json
cargo xtask check-no-panic-family --json-output target/tokmd/reports/no-panic-report.json
```

The hosted workflow uploads the JSON report as the `no-panic-policy-report`
artifact. The JSON report is gate evidence for the checker run, but it is not a
public `tokmd` product receipt and does not replace `ci/proof.toml` proof
planning.

The proposal helper:

```bash
cargo xtask no-panic-propose
```

writes proposed entries to `target/no-panic-proposed-allowlist.toml`. Proposed
entries are drafting material. They become policy only after a maintainer
copies them into `policy/no-panic-allowlist.toml`, fills the required review
fields, and validates the checker.

The baseline helper:

```bash
cargo xtask no-panic-baseline --receipt-output policy/no-panic-baseline-receipt.json
```

materializes the full receipted ledger from current findings, deduplicated by
selector identity. CI runs `cargo xtask check-no-panic-family --strict` against
the committed baseline in `policy/no-panic-allowlist.toml`.

## Compatibility

Strict rollout is active as of 2026-06-22:

- `cargo xtask check-no-panic-family` remains advisory by default for local
  exploration;
- CI and `ci/proof.toml` run `--strict`;
- `policy/no-panic-allowlist.toml` holds the receipted baseline (21,783 unique
  identities at rollout);
- `policy/no-panic-baseline-receipt.json` records baseline generation stats;
- `.github/workflows/no-panic-policy.yml` publishes strict JSON report
  artifacts;
- `docs/NO_PANIC_POLICY.md` remains the contributor-facing guide;
- no release, publish, signing, Nix, Codecov, or product CLI behavior changes.

Any change that flips strict mode in CI, changes the allowlist schema version,
adds a finding family, removes a classification, changes identity matching, or
makes no-panic results part of another product receipt must update this spec,
the user guide, checker tests, and proof policy in the same review.

## Proof Requirements

For documentation-only changes to this contract:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-no-panic-policy-spec.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-no-panic-policy-spec.json --evidence-json target/proof/proof-evidence-no-panic-policy-spec.json
cargo fmt-check
git diff --check
```

For checker, workflow, or policy implementation changes, also run the focused
current-behavior checks:

```bash
cargo xtask check-no-panic-family --strict
cargo test -p xtask no_panic --verbose
cargo clippy -p xtask --all-targets -- -D warnings
```

Hosted PR evidence should include the `No-panic Policy` workflow when workflow
behavior or uploaded report paths change.

## Open Questions

- Whether the JSON checker report should eventually get a named schema family.
- When strict mode should become a required hosted check after the advisory
  rollout reaches an exact or zero-finding state.
- Whether assertion macros should join the no-panic taxonomy.
