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
- Merged #1559: synthesized keeper for FFI/env/path sanitization. Added in-memory input path length/control-character validation and config/profile selector trim/control-character sanitization while preserving case-sensitive FFI modes and existing path semantics. Gates: `cargo test -p tokmd-core in_memory_inputs_rejects_`; `cargo test -p tokmd-core --test error_boundary`; `cargo test -p tokmd-core parse_scan_settings`; `cargo test -p tokmd sanitize_selector`; `cargo test -p tokmd get_profile_name`; `cargo clippy -p tokmd-core -p tokmd -- -D warnings`; `cargo fmt-check`; `git diff --check`; GitHub CI.
- Closed #1403/#1404/#1405/#1406 as superseded or declined by #1559.
- Merged #1564: synthesized keeper for browser GitHub ingest cache partitioning. Added a token-derived auth partition to in-memory cache keys without storing raw token text and proved token-specific misses plus same-token cache reuse. Gates: `npm --prefix web/runner run check`; `npm --prefix web/runner test`; `node --test web/runner/ingest.test.mjs`; `node --check web/runner/ingest.js`; `git diff --check`; GitHub CI.
- Closed #1411/#1413/#1415/#1417 as superseded by #1564.
- Merged #1565: synthesized keeper for `tokmd-types` optional-field serde omission coverage. Added source-local tests for omitted `CapabilityStatus.reason` and `ArtifactEntry.hash`. Gates: `cargo test -p tokmd-types omits_`; `cargo test -p tokmd-types`; `cargo fmt-check`; `git diff --check`; GitHub CI.
- Closed #1479 as superseded by #1565.
- Closed #1480 as duplicate coverage: current main already has all-variant cockpit enum serde coverage in integration/contract tests and snapshots.
- Merged #1566: synthesized keeper for `tokmd-git` intent helper coverage. Added source-local intent classification tests and direct private word-boundary helper assertions after fixing the stale branch formatting issue. Gates: `cargo test -p tokmd-git classify_intent_`; `cargo test -p tokmd-git contains_word_respects_word_boundaries`; `cargo test -p tokmd-git`; `cargo fmt-check`; `git diff --check`; GitHub CI.
- Closed #1478 as superseded by #1566.
- Merged #1568: synthesized keeper for `tokmd-io-port` `MemFs` property tests. Added properties for byte-length file size reporting and sorted unique path listing. Gates: `cargo test -p tokmd-io-port --test properties`; `cargo test -p tokmd-io-port`; `cargo fmt-check`; `git diff --check`; GitHub CI.
- Closed #1463 as superseded by #1568.
- Merged #1573: synthesized keeper for `tokmd-scan` config property tests. Added real-helper properties proving monotonic opt-in flag behavior and the `no_ignore` implication across scan config mapping. Gates: `cargo test -p tokmd-scan --test properties`; `cargo test -p tokmd-scan`; `cargo fmt-check`; `git diff --check`; GitHub CI.
- Closed #1466 as superseded by #1573.
- Next cluster: Property/proptest improvements (#1464, #1465).

## Operating decisions

- Treat `tokmd-config` as retired. Do not merge restore-to-workspace PRs; salvage only tests or cleanup that fit `tokmd-settings` / `tokmd-core` / `tokmd` ownership.
- Treat `.jules` as an allowed knowledge workspace for durable specs, investigations, friction notes, persona learnings, and generated indexes. Prefer concise summaries over raw logs or repeated run transcripts.
- Prefer SRP submodules inside stable public crates over new implementation microcrates. Public crates should represent contracts, facades, adapters, or products.
- For cockpit/review work, improve cockpit as the current PR-review surface first. Do not merge competing `review` implementations until the artifact contract is clear.
