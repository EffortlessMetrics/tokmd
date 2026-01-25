# TODO

## Documentation
- [ ] Document "first wins" file deduplication logic in README.
- [ ] Explain embedded language behavior in README.
- [ ] Verify `Cargo.toml` metadata (homepage, repository, keywords) is final.

## Verification
- [ ] Run full test suite with `cargo test`.
- [ ] Manually verify `tokmd export --redact all` output.
- [ ] Manually verify `tokmd export --min-code 100` filtering.

## Infrastructure
- [ ] Verify CI (GitHub Actions) passes on all platforms.
- [ ] Create a release tag `v0.2.0`.

## Future
- [ ] Add golden tests for deterministic output validation.
