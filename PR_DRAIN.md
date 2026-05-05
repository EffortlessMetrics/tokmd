## 2026-05-05 PR drain

- Merged #1548: dependabot rust minor/patch dependency bump for `jsonschema`, `napi`, `napi-derive`, `tokio`, and `wasm-bindgen-test`. Gates: `cargo check --workspace`; `cargo test --workspace`; `cargo deny --all-features check`; `npm --prefix web/runner test`.
- Held #1541: external Factory Droid review workflow requires maintainer approval and secret/API-key policy.
- Merged #1549: synthesized keeper for stale `deny.toml` advisory cleanup. Removed only the obsolete `RUSTSEC-2023-0071` ignore. Gates: `cargo deny --all-features check`; `git diff --check`; GitHub CI.
- Closed #1546/#1540/#1523 as superseded by #1549.
- Merged #1551: synthesized keeper for JS deterministic sorting. Replaced `localeCompare` with explicit Unicode code point comparison and added a browser-runner regression test. Gates: `npm --prefix web/runner run check`; `npm --prefix web/runner test`; `git diff --check`; GitHub CI.
- Closed #1542/#1512 as superseded by #1551.
- Merged #1552: synthesized keeper for `export_bundle` no-default-features warning. Gated the module behind `analysis` instead of suppressing dead-code warnings. Gates: `cargo clippy -p tokmd --no-default-features -- -D warnings`; `cargo clippy -p tokmd --no-default-features --features analysis -- -D warnings`; `cargo test -p tokmd --no-default-features`; `git diff --check`; GitHub CI.
- Closed #1530/#1510/#1502 as superseded by #1552.
- Merged #1553: synthesized keeper for redaction hardening. Preserved only allowlisted path extensions and stripped short untrusted tokens such as `passwd`, `secret`, `pass1234`, and `token`. Gates: `cargo test -p tokmd-format test_redact_path_leak`; `cargo test -p tokmd-format redact`; `cargo test -p tokmd-types --test determinism_proptest`; `cargo fmt-check`; `git diff --check`; GitHub CI.
- Closed #1521/#1507/#1497 as superseded by #1553.
- Merged #1554: synthesized keeper for Git subprocess hardening. Added an explicit `--end-of-options` boundary to revision verification and validated fallback base refs without stripping legitimate alternate-index or object-store Git environment variables. Gates: `cargo test -p tokmd-git --verbose`; `cargo test -p tokmd --verbose`; `cargo clippy -p tokmd-git -- -D warnings`; `cargo fmt-check`; `git diff --check`; GitHub CI.
- Closed #1503 as superseded by #1554.
- Next cluster: FFI/env/path sanitization (#1403, #1404, #1405, #1406).

## Operating decisions

- Treat `tokmd-config` as retired. Do not merge restore-to-workspace PRs; salvage only tests or cleanup that fit `tokmd-settings` / `tokmd-core` / `tokmd` ownership.
- Treat `.jules` as an allowed knowledge workspace for durable specs, investigations, friction notes, persona learnings, and generated indexes. Prefer concise summaries over raw logs or repeated run transcripts.
- Prefer SRP submodules inside stable public crates over new implementation microcrates. Public crates should represent contracts, facades, adapters, or products.
- For cockpit/review work, improve cockpit as the current PR-review surface first. Do not merge competing `review` implementations until the artifact contract is clear.
