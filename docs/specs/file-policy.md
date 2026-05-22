# Spec: Non-Rust File Policy

- Status: active
- Schema family, if any: `policy/non-rust-allowlist.toml` schema version `1.0`
- Related ADRs: none
- Related proof scopes: `file_policy`, `project_truth_docs`,
  `proof_control_plane`

## Contract

Tokmd is a Rust-first repository that still contains legitimate non-Rust
surfaces: GitHub Actions YAML, Nix packaging, documentation, JSON schemas,
browser assets, package manifests, release packaging, fixtures, and agent
runbooks. The non-Rust file policy makes those surfaces explicit, owned, and
covered instead of accepting them by silence.

The policy is enforced by:

```bash
cargo xtask check-file-policy
```

The checker walks the workspace, skips Rust source files, and reports every
non-Rust file that does not match a `[[allow]]` glob in
`policy/non-rust-allowlist.toml`.

The checker is advisory by default:

- parse, walk, glob compilation, and unsupported schema-version failures are
  hard errors;
- allow-entry shape problems and unmatched non-Rust files are reported as
  findings;
- findings become blocking only when the checker is run with `--strict`.

This spec records current behavior. It does not promote file-policy findings
to a required strict gate, change release behavior, change public `tokmd` CLI
behavior, or change product receipt schemas.

## Inputs

The checker consumes checked repository state:

| Input | Owner | Used for |
| --- | --- | --- |
| `policy/non-rust-allowlist.toml` | Machine policy | Glob-based allowlist, ownership metadata, coverage declarations, and advisory status. |
| Workspace file tree | Repository state | Non-Rust file discovery and unmatched-file reporting. |
| `xtask/src/tasks/file_policy.rs` | Checker implementation | Walk behavior, skip directories, entry validation, report rendering, and advisory/strict semantics. |
| `docs/FILE_POLICY.md` | User guide | Contributor workflow and current allowlist field descriptions. |
| `ci/proof.toml` `file_policy` scope | Proof policy | Local proof routing for file-policy docs, policy, and checker changes. |

The checker normalizes path separators to `/` before matching globs so the
allowlist can be authored and reviewed consistently across operating systems.

## Scope

The checker walks regular files under the workspace root with these exclusions:

- `.git/`
- `target/`
- `node_modules/`
- `run-artifacts/`
- `plans/`

Files ending in `.rs` are counted as Rust files skipped and are not governed by
this policy. Rust files remain governed by workspace lints, no-panic policy,
proof policy, tests, and crate ownership.

Generated output, downloaded dependencies, build products, and local run
artifacts should stay outside the checked tree or under skipped directories
unless they are intentionally committed. Intentionally committed non-Rust files
must have allowlist coverage.

## Allowlist Entries

`policy/non-rust-allowlist.toml` must use:

```toml
schema_version = "1.0"
```

Each `[[allow]]` entry declares a glob plus review metadata:

| Field | Required | Meaning |
| --- | --- | --- |
| `glob` | yes | Repo-relative glob that matches one or more allowed non-Rust files or surfaces. |
| `kind` | yes | Structural category such as `documentation`, `ci_declarative`, `schema`, or `test_fixture`. |
| `owner` | yes | Responsible team or role. Empty owners are invalid. |
| `surface` | yes | Consumer surface such as `build`, `ci`, `docs`, `release`, `agents`, `contract`, or `downstream`. |
| `classification` | yes | Governance classification. Current entries use values such as `production`, `config`, `documentation`, `vendor`, `test_fixture`, and `generated`. |
| `reason` | yes | Why the non-Rust file belongs in a Rust-first repo. |
| `covered_by` | conditional | Proof obligation that exercises the surface. Required for `classification = "production"`. |
| `generated_by` | optional | Generator command or owner for committed generated artifacts. |

The checker reports duplicate `glob` values, empty `kind`, empty `owner`, empty
`surface`, empty `classification`, empty `reason`, and production entries with
no `covered_by` obligation.

Allow entries may be broad when they represent a real surface owner, such as
`docs/**` or `.github/workflows/*.yml`. Broad globs should not be used to hide
uncategorized generated output or unmanaged tool state.

## Outputs

When there are no findings, the checker prints a compact success summary with
the number of allow entries, covered non-Rust files, and skipped Rust files.

When findings exist, the checker prints:

- the total finding count;
- up to the first 50 findings;
- an advisory-mode note when `--strict` is not set.

With `--report-dir <DIR>`, the checker writes:

```text
<DIR>/file-policy-report.txt
```

The text report includes policy metadata, allow-entry count, covered non-Rust
file count, skipped Rust file count, unmatched-file count, finding count, and
the unmatched/finding lists.

The report is review evidence for the file-policy checker. It is not a public
`tokmd` product receipt and does not replace `ci/proof.toml` proof planning.

## Compatibility

This spec is compatible with the current advisory rollout:

- `cargo xtask check-file-policy` remains advisory by default;
- `--strict` remains opt-in;
- `policy/non-rust-allowlist.toml` remains the source of truth for committed
  non-Rust file coverage;
- `docs/FILE_POLICY.md` remains the contributor-facing guide;
- no release, publish, signing, Docker, Nix-full, Codecov, or public CLI
  behavior changes.

Any change that flips strict mode in CI, changes the allowlist schema version,
changes skipped directories, changes required entry fields, removes production
coverage requirements, emits a machine-readable product receipt, or changes the
definition of which files are governed must update this spec, the user guide,
checker tests, and proof policy in the same review.

## Proof Requirements

For documentation-only changes to this contract:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask check-file-policy
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-file-policy-spec.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-file-policy-spec.json --evidence-json target/proof/proof-evidence-file-policy-spec.json
cargo fmt-check
git diff --check
```

For checker or policy implementation changes, also run focused current-behavior
checks:

```bash
cargo xtask check-file-policy --report-dir target/file-policy
cargo test -p xtask file_policy --verbose
```

Hosted PR evidence should include the affected proof plan and Docs Check when
documentation-control surfaces change.

## Open Questions

- Whether the text report should eventually get a named JSON receipt schema.
- Whether advisory unmatched-file findings should become strict by default
  after the allowlist reaches a stable zero-drift state.
- Whether broad allowlist globs should eventually require explicit owner-level
  subcategories for generated docs, fixtures, and agent state.
