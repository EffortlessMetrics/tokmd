# Spec: Config Explainability (`--show-config`)

- Status: active
- Schema family, if any: n/a (human-readable diagnostic surface)
- Related ADRs: n/a
- Related proof scopes: `tokmd_cli`, `project_truth_docs`

## Contract

`tokmd` resolves runtime options from several layered sources. The layering is
intentional but not visible to users: a `tokmd.toml` discovered by walking up
from the current directory can silently shadow a different file, a legacy
`~/.config/tokmd/config.json` profile can still apply, and `TOKMD_CONFIG` /
`TOKMD_PROFILE` environment variables can change which file or profile is
active without any on-screen indication.

The global `--show-config` flag makes that resolution observable. When present,
`tokmd` prints a human-readable report of the discovered configuration sources
and the values they resolve to, then exits `0` **without scanning** any paths or
running the selected subcommand.

`--show-config` is a diagnostic surface only. It must not:

- change command stdout for normal runs,
- emit or alter any receipt, JSON, JSONL, or CSV output,
- change exit-code policy for normal commands,
- read or write files beyond the existing config-discovery reads.

The report is written to **stdout** because it is the requested output of the
invocation (the same way `--help` and `--version` own stdout). It is
human-readable text, not a machine contract; its exact wording may change
between releases.

## Inputs

| Input | Owner | Effect |
| --- | --- | --- |
| `--show-config` | CLI flag (global) | Print the config report and exit `0` before dispatching any subcommand. |
| `TOKMD_CONFIG` | Process environment | Explicit `tokmd.toml` path; highest-precedence TOML source. Reported as the active TOML path when it resolves. |
| `tokmd.toml` (cwd walk-up) | Filesystem | TOML config discovered from the current directory upward to the filesystem root. |
| `~/.config/tokmd/tokmd.toml` | Filesystem | User-level TOML config fallback. |
| `~/.config/tokmd/config.json` | Filesystem | Legacy JSON profile config fallback. |
| `--profile` / `--view` | CLI flag (global) | Selects a named profile/view. Highest-precedence profile selector. |
| `TOKMD_PROFILE` | Process environment | Profile/view name used when `--profile` is absent. |

`--show-config` reports exactly the sources that `tokmd` already resolves at
startup. It does not introduce a new discovery path.

## Outputs

The report names, in resolution order:

1. The active TOML config path, or that none was found.
2. Whether a legacy JSON config was loaded.
3. The active profile/view name and where it came from (`--profile`,
   `TOKMD_PROFILE`, or none), plus whether it matched a TOML view and/or a
   legacy JSON profile.
4. The resolved profile-layered values (`format`, `top`, `files`,
   `module_roots`, `module_depth`, `children`, `min_code`, `max_rows`,
   `redact`, `meta`), each shown as its resolved value or `(default)` when no
   source set it.

A profile name that resolves to no matching TOML view and no JSON profile is
reported as unmatched so users can detect typos or missing config files.

## Compatibility

`--show-config` is additive. It introduces no schema and does not change the
behavior of any existing command when the flag is absent. Removing the flag
would only remove the diagnostic surface; no receipt or machine output depends
on it.

## Proof Requirements

```bash
cargo test -p tokmd config_explain --verbose
cargo test -p tokmd show_config --verbose
cargo fmt-check
cargo clippy -p tokmd -- -D warnings
git diff --check
```

## Open Questions

- Whether a `--show-config --format json` machine projection is worth adding if
  a tool or agent consumer is named. Until then the surface stays
  human-readable only.
