# ADR-0010: Diff input classification before git resolution

- Status: accepted
- Date: 2026-05-22

## Context

`tokmd diff` accepts either receipt paths or git revisions for its two inputs.
That dual use creates an ambiguity when an input does not exist on disk:

- `main`, `HEAD`, and tag names may be valid git revisions;
- `missing.json`, `./missing.json`, `../missing.json`, and absolute paths are
  usually intended as file paths;
- when a missing file-like input is interpreted as a git revision first, a
  caller outside a git repository can receive `not inside a git repository`
  instead of a useful missing-path error.

This ambiguity surfaced during the v1.12.0 release repair path. A Nix tag-check
run exercised `tokmd diff` from a non-git sandbox with missing receipt paths,
and the command reported the git-repository precondition rather than the
invalid path input.

## Decision

`tokmd diff` classifies inputs in this order:

1. Existing filesystem paths are loaded as receipt inputs.
2. Missing inputs that are clearly path-like fail as missing paths before git
   revision resolution.
3. Remaining inputs are resolved as git revisions.

The path-like classification intentionally covers conservative, user-obvious
path shapes:

- absolute paths;
- inputs with a file extension;
- inputs prefixed with `./`;
- inputs prefixed with `../`;
- inputs prefixed with `.\\`;
- inputs prefixed with `..\\`.

The command preserves git revision resolution for ordinary non-path tokens such
as `HEAD`, branch names, tags, and other extensionless ref names.

## Consequences

- Missing receipt files produce an input error without requiring the current
  directory to be inside a git repository.
- Release validation and Nix sandboxes get deterministic path-input errors.
- Branch and tag names remain valid diff inputs when they do not look like
  paths.
- The heuristic is intentionally conservative. Some extensionless missing
  relative paths may still fall through to git revision resolution.

## Alternatives

- Try git resolution before path classification. This was rejected because it
  can hide a missing file behind an unrelated git-repository precondition.
- Treat every missing token as a path. This was rejected because it would break
  normal git revision inputs.
- Require explicit flags for path versus git inputs. This was rejected as a
  larger CLI compatibility change; the current command already accepts both
  forms.

## Enforcement

- `crates/tokmd/src/commands/diff.rs` owns the input classification order.
- `crates/tokmd/tests/cli_error_paths_w51.rs` verifies that missing file-like
  diff inputs fail as invalid references/paths and do not require a git
  repository.
- `crates/tokmd/tests/cli_errors_w66.rs` keeps the broader CLI error behavior
  covered.
- Any future change to the heuristic must update the related spec, CLI tests,
  and release-validation expectations together.

## Related specs

- `docs/specs/diff-input-classification.md`
