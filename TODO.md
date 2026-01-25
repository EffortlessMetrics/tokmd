# TODO (v1.0 Roadmap)

## Milestone 5: The Test Harness (v0.9.0)
- [x] **Setup**: Add `dev-dependencies`: `assert_cmd`, `predicates`, `tempfile` to `Cargo.toml`.
- [x] **Infrastructure**: Create `tests/integration_tests.rs`.
- [x] **Fixture**: Create a `tests/data/` folder with a known file structure (e.g., nested Rust files, a JS file, a hidden file).
- [x] **Golden Test (Lang)**: Verify `tokmd` (default) output matches a saved snapshot.
- [x] **Golden Test (Module)**: Verify `tokmd module` matches snapshot.
- [x] **Golden Test (Export)**: Verify `tokmd export` (JSONL) matches snapshot.
- [x] **Regression**: Verify `redact` hashes are stable across runs.

## Milestone 6: Documentation & Polish (v0.9.5)
- [x] **README**: Document how to use `tokmd export` for CI (e.g., "Tracking repo size").
- [x] **Help Text**: Review `tokmd --help` and subcommand help strings for consistency.
- [x] **Schema**: Create a `docs/SCHEMA.md` explaining the JSON structure of a receipt.
- [x] **Formal Schema**: Create `docs/schema.json` compliant with JSON Schema Draft 07.
- [x] **Error Handling**: Verify behavior when `tokei` fails (e.g., on a locked file).

## Milestone 7: Release v1.0.0
- [x] **Versioning**: Bump `Cargo.toml` to `1.0.0`.
- [ ] **Tag**: `git tag v1.0.0`.
- [ ] **Publish**: `cargo publish`.
