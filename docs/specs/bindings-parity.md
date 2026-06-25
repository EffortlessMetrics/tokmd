# Spec: Bindings FFI Envelope Parity

- Status: active
- Schema family: tokmd-core FFI response envelope (`ok` / `data` / `error`)
- Related agent charter: `.github/agents/bindings-parity-keeper.agent.md`
- Related proof command: `cargo xtask bindings-parity --check`

## Contract

`tokmd_core::ffi::run_json(mode, args_json)` is the single JSON entrypoint shared by
`tokmd-python` and `tokmd-node`. All bindings must preserve the envelope semantics
implemented in `tokmd-envelope::ffi` and surfaced by `tokmd-core::ffi::run_json`.

Success:

```json
{ "ok": true, "data": { "...receipt or version payload..." } }
```

Failure:

```json
{
  "ok": false,
  "error": {
    "code": "machine_code",
    "message": "human-readable message",
    "details": "optional field key or context"
  }
}
```

Convenience APIs (`lang`, `module`, `export`, `run`, etc.) extract the `data` field
via `tokmd_envelope::ffi::extract_data_json` and map upstream failures to binding-local
exceptions (`TokmdError` / N-API `Error`).

## Inputs

- Manifest: `fixtures/bindings-parity/manifest.json` (`tokmd.bindings_parity_manifest.v1`)
- Optional golden envelopes under `fixtures/bindings-parity/golden/` for stable error shapes
- In-memory `inputs` arrays for deterministic scan cases (no filesystem dependence)

## Outputs

- Fixture verification receipt (optional): `target/tokmd/reports/bindings-parity-report.json`
- CI artifact upload from `.github/workflows/bindings-parity.yml`

## Compatibility

- Core receipt `schema_version` must match `tokmd_types::SCHEMA_VERSION` (currently `2`).
- Breaking envelope or error-code changes require manifest/golden updates and an intentional
  semver/support-tier note in the binding crates.
- Python and Node bindings are **experimental** (`publish = false`); this lane guards
  contract drift, not npm/PyPI publish readiness.

## Proof Requirements

Local:

```bash
cargo xtask bindings-parity --check
```

The command:

1. Executes manifest cases against `tokmd_core::ffi::run_json`.
2. Compares selected cases to golden partial envelopes.
3. Runs existing Rust binding guard tests:
   - `cargo test -p tokmd-core --test bindings_parity --all-features`
   - `cargo test -p tokmd-node --lib --all-features`
   - `cargo test -p tokmd-python --test property_tests --all-features`

CI: advisory job in `.github/workflows/bindings-parity.yml` on PRs touching
`tokmd-core`, `tokmd-python`, `tokmd-node`, `tokmd-envelope`, or parity fixtures.

## Open Questions

- When to promote the lane from advisory to merge gate after stable CI history.
- Whether to add opt-in Python (`pytest`) / Node (`ava`) runtime checks once native
  artifacts are built in CI without widening scope beyond envelope parity.
