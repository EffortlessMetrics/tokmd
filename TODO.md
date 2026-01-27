# TODO

## v1.0.0 — Production Readiness (Completed)

- [x] **Formal Schema**:
  - [x] Define JSON schema for `lang`, `module`, and `export`.
  - [x] Add `schema_version`, `tool`, `inputs` metadata.
- [x] **Export Command**:
  - [x] Implement `export` subcommand.
  - [x] Support JSONL and CSV formats.
  - [x] Add filters: `--min-code`, `--max-rows`.
  - [x] Add redaction: `--redact paths`, `--redact all`.
- [x] **Consistency**:
  - [x] Unified `--children` flag behavior.
- [x] **Quality Assurance**:
  - [x] Integration tests with `insta` snapshots.
  - [x] BDD-style test scenarios.
  - [x] CI workflow hardening.
- [x] **Documentation**:
  - [x] Diataxis README.
  - [x] Recipe book.
  - [x] Schema reference.
- [x] **Release**:
  - [x] Bump version to 1.0.0.

## Future / v1.1+

- [x] **GitHub Action**: A first-party action to run tokmd in CI.
- [x] **Diff Mode**: `tokmd diff <ref1> <ref2>` to see changes in stats.
- [x] **Binary Releases**: Pre-built binaries attached to GitHub Releases (via `.github/workflows/release.yml`).
