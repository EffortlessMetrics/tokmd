# Spec: WASM FFI Byte Mode for Archive Upload

- Status: active
- Implementation state: the core byte FFI entrypoint
  (`tokmd_core::ffi::run_json_bytes`, behind the `archive-zip` feature) and the
  WASM `Uint8Array` binding (`tokmd-wasm::runJsonBytes`) have landed with
  native and `wasm-bindgen-test` parity coverage; the capability matrix rows are
  promoted. Remaining follow-on: wire `runJsonBytes` into the browser runner UI
  (`web/runner` `zipball` path) and byte/host parity oracle.
- Schema family, if any: none yet (no new serialized receipt schema is introduced by this seam)
- Related ADRs: none yet
- Related proof scopes: `scan`, `model`, `io_port`
- Related crates: `crates/tokmd-core`, `crates/tokmd-wasm`, `crates/tokmd-scan`, `crates/tokmd-io-port`
- Related specs: `docs/specs/repo-snapshot.md`
- Related docs: `docs/browser-capability-matrix.md`, `docs/capabilities/wasm.json`

## Contract

This spec names the missing FFI seam that lets a browser/worker caller hand
**raw archive bytes** (initially a ZIP) into tokmd for in-memory scanning,
without a host filesystem. It is a forward-looking contract stub: it fixes the
intended shape and boundaries so a later implementing PR has a stable target. It
introduces no runtime behavior change on its own.

The archive admission and scan layers already exist and are proven by their own
crate tests (see the incremental status in `docs/specs/repo-snapshot.md`):

- `crates/tokmd-io-port/src/archive.rs` admits already-decoded entries
  fail-closed and, behind the `archive-zip` feature, decodes ZIP bytes into a
  snapshot.
- `crates/tokmd-scan/src/in_memory.rs` runs the existing aggregation over an
  in-memory snapshot, and a ZIP→scan consumer composes the codec with that path.

What is missing is the **transport seam**: the only FFI entrypoint the bindings
consume, `tokmd_core::ffi::run_json` in `crates/tokmd-core/src/ffi/mod.rs`, takes
a JSON string and an ordered list of in-memory `{ path, text }` UTF-8 inputs. It
has no binary input path, and `crates/tokmd-wasm/src/lib.rs` therefore cannot
reach the landed ZIP→scan consumer. `docs/browser-capability-matrix.md` records
this as the standing blocker for browser ZIP upload.

This spec fixes the byte-mode contract that closes that gap. Three roles make up
the seam.

- **Byte FFI mode (landed)** — `tokmd_core::ffi::run_json_bytes(mode,
  options_json, archive_bytes)` accepts an archive byte buffer plus a small JSON
  options object (logical `root`, scan options, `archive_limits` ingestion caps,
  per-mode settings) and returns the same envelope shape
  (`{ "ok": ..., "data": ..., "error": ... }`) that the JSON modes already
  return. It decodes the bytes through the landed
  `tokmd_scan::inputs_from_zip_bytes` consumer and routes the admitted inputs
  through the existing `run_mode` dispatch, so it introduces no new scan path
  and no second admission path. The byte mode is gated by the same
  decompression-dependency feature (`archive-zip`) so the default core/binding
  surface stays free of decompression dependencies. It serves the
  input-consuming scan modes (`lang`/`module`/`export`/`analyze`); host-only
  modes (`diff`/`cockpit`/`version`) are rejected, and the `inputs`/`paths`
  conventions are mutually exclusive with the archive byte source.

- **WASM byte binding (landed)** — a thin wasm-bindgen function that accepts a
  JS byte view (`Uint8Array`) plus options and forwards to the core byte mode,
  matching the thin-wrapper rule the rest of `crates/tokmd-wasm/src/lib.rs`
  follows. The crate keeps `#![forbid(unsafe_code)]`; the byte view is copied
  into an owned buffer at the boundary rather than aliased.

- **Capability promotion (landed)** — the archive rows in
  `docs/browser-capability-matrix.md` and `docs/capabilities/wasm.json` are
  marked supported for the `runJsonBytes` binding with `archive-zip` enabled,
  gated on the `wasm-bindgen-test` parity coverage in `crates/tokmd-wasm`.

The boundary this seam protects:

- The byte mode is **additive**: the existing JSON modes
  (`lang`/`module`/`export`/`analyze`) and the `{ path, text }` input convention
  are unchanged, and host-backed receipts are untouched.
- Untrusted archive bytes remain **trust-boundary-crossing input**. The byte
  mode delegates all path-safety and zip-bomb admission to the landed engine in
  `crates/tokmd-io-port/src/archive.rs`; it must not add a second, weaker
  admission path. Ingestion fails closed: a single rejected entry fails the whole
  call with a named error rather than producing a partial scan.
- Output for in-scope files must be **indistinguishable** from the equivalent
  host-extracted scan, matching the parity oracle the snapshot seam already
  defines.

Out of scope for this stub: streaming/chunked upload (the first mode is
buffered), tar-family containers, a serialized on-disk snapshot format, and any
non-scan command (git history, `run`, `diff`, `cockpit`, and friends stay
native-only per `docs/browser-capability-matrix.md`).

## Inputs

A future byte-mode call takes:

- The archive bytes as a single buffer (browser `Uint8Array`, copied to an owned
  buffer at the FFI boundary). Streaming is explicitly deferred.
- An options object (JSON for the core entrypoint, a JS object or JSON string for
  the binding) carrying: the logical repository root the admitted entries are
  rooted under; the scan options already accepted by the JSON modes; the
  ingestion limit set (per-entry uncompressed cap, total uncompressed cap, entry
  count cap, compression-ratio guard) with the conservative defaults already
  shipped by the admission engine; and the requested scan mode
  (`lang`/`module`/`export`, plus rootless `analyze` presets where supported).

The byte mode does not accept `{ path, text }` inputs at the same time as
archive bytes; the two input conventions are mutually exclusive, mirroring the
existing "paths cannot be combined with in-memory inputs" rule in
`crates/tokmd-core/src/ffi/mod.rs`.

## Outputs

- On success: the same response envelope and receipt payloads the corresponding
  JSON mode already returns for the equivalent `{ path, text }` input set, with
  the same `schema_version` and tool metadata. No new receipt schema is defined.
- On rejection: a fail-closed error envelope identifying the first violated
  path-safety or resource limit (which entry, which bound), surfaced through the
  existing typed error → envelope path. Rejected archives produce no partial
  receipt.

If a serialized snapshot or archive-manifest output is later required, it must
get its own schema version and a follow-on spec; this stub does not define one.

## Compatibility

- Additive only. The byte mode is a new entrypoint; existing modes, the
  `{ path, text }` convention, and host-backed receipts are unchanged.
- Feature-gated trust surface. The byte mode and its decompression dependency sit
  behind the `archive-zip` feature so the default `crates/tokmd-core` and
  `crates/tokmd-wasm` builds keep zero decompression dependencies. Enabling the
  feature is what pulls the audited deflate-only decoder already selected by the
  snapshot seam.
- The ingestion limits are part of the support promise: tightening a default is a
  compatible hardening; loosening or removing one is a security-relevant change
  that must update this spec, `docs/specs/repo-snapshot.md`, and their proof.
- Capability docs are claims. `docs/browser-capability-matrix.md` and
  `docs/capabilities/wasm.json` must not mark archive rows "supported" until the
  binding and `wasm-bindgen-test` parity coverage land.
- Any new or widened dependency surface follows the normal dependency rules:
  record license/security/support impact in the implementing PR and route it
  through the existing `deny.toml` / dependency-maintenance proof. This stub
  picks no new crate.

## Proof Requirements

These are the proof obligations for the full seam. Items met by the landed core
byte entrypoint are marked; the remaining items gate the WASM binding and
capability promotion.

- Byte/JSON-mode parity (MET, core): for the same logical file set, the
  byte-mode envelope must match the envelope produced by the existing
  `{ path, text }` JSON mode (same `schema_version`, same payload modulo
  volatile timestamps). Covered by the
  `byte_mode_lang_envelope_matches_json_mode_inputs` test in
  `crates/tokmd-core/tests/archive_zip_ffi_bytemode.rs` and the `tokmd-scan`
  decode parity in `crates/tokmd-core/tests/archive_zip_bytemode.rs`.
- Fail-closed admission (MET, core): a traversal entry, a malformed archive, and
  a breached ingestion cap each fail closed with no partial receipt, with no
  second admission path bypassing `crates/tokmd-io-port/src/archive.rs`. Covered
  by the `byte_mode_fails_closed_on_hostile_entry`,
  `byte_mode_rejects_malformed_archive`, and `byte_mode_enforces_archive_limits`
  tests in the same file. Broader hostile-fixture coverage
  (absolute/drive-prefix/NUL/non-regular) is already proven at the admission
  engine in `crates/tokmd-io-port`.
- Byte/host parity (PENDING): scanning a benign archive fixture through the byte
  mode must yield the same normalized file set and aggregated receipt as
  scanning the equivalent extracted tree through the host path, reusing the
  parity oracle the snapshot seam defines.
- WASM boundary coverage (MET): a `wasm-bindgen-test` exercises the
  `Uint8Array` binding end to end and asserts the browser payload matches the
  core payload, in the same style as the existing boundary tests in
  `crates/tokmd-wasm/src/lib.rs` (`run_json_bytes_lang_matches_inline_inputs_over_js_boundary`).
- Default-surface guard (MET, CI): the Wasm Compile & Test lane checks and tests
  the default (non-`archive-zip`) feature set separately from the
  `archive-zip`-gated binding, confirming decompression dependencies only enter
  when the feature is enabled.

Doc-shape checks for this spec stub itself:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
```

## Open Questions

- RESOLVED: the byte mode is a separate byte-taking entrypoint
  (`run_json_bytes(mode, options_json, archive_bytes)`) alongside `run_json`,
  not a base64 mode discriminator. This avoids base64 inflation across the FFI
  boundary while still reusing the existing `run_mode` dispatch and envelope
  plumbing by decoding the bytes into the in-memory input list first. The `mode`
  argument selects the scan mode, leaving room for the WASM binding to expose
  either one archive function per mode or a single mode-taking entrypoint (see
  the next question).
- Should the WASM binding expose one archive function per scan mode
  (mirroring `runLang`/`runModule`/`runExport`) or a single archive entrypoint
  that takes the mode as an argument?
- Should ingestion limits be surfaced as explicit binding arguments with
  documented defaults, or read from the options object only? Explicit arguments
  are more discoverable; an options object keeps the signature stable.
- Should rejected-entry diagnostics aggregate all violations or fail on the
  first? The admission engine currently fails fast; this stub assumes the same.
- Does the byte mode need to report ingestion statistics (admitted entry count,
  total inflated bytes, which limit was closest) in the success envelope, or does
  that belong in a later observability follow-on?
