# TODO (v1.0 Roadmap)

## Milestone 5: The Test Harness (v0.9.0)
- [ ] **Setup**: Add `dev-dependencies`: `assert_cmd`, `predicates`, `tempfile` to `Cargo.toml`.
- [ ] **Infrastructure**: Create `tests/integration_tests.rs`.
- [ ] **Fixture**: Create a `tests/data/` folder with a known file structure (e.g., nested Rust files, a JS file, a hidden file).
- [ ] **Golden Test (Lang)**: Verify `tokmd` (default) output matches a saved snapshot.
- [ ] **Golden Test (Module)**: Verify `tokmd module` matches snapshot.
- [ ] **Golden Test (Export)**: Verify `tokmd export` (JSONL) matches snapshot.
- [ ] **Regression**: Verify `redact` hashes are stable across runs.

## Milestone 6: Documentation & Polish (v0.9.5)
- [ ] **README**: Document how to use `tokmd export` for CI (e.g., "Tracking repo size").
- [ ] **Help Text**: Review `tokmd --help` and subcommand help strings for consistency.
- [ ] **Schema**: Create a `docs/SCHEMA.md` explaining the JSON structure of a receipt.
- [ ] **Error Handling**: Verify behavior when `tokei` fails (e.g., on a locked file).

## Milestone 7: Release v1.0.0
- [ ] **Versioning**: Bump `Cargo.toml` to `1.0.0`.
- [ ] **Tag**: `git tag v1.0.0`.
- [ ] **Publish**: `cargo publish`.
