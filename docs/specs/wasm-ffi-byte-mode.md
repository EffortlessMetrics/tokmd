# Spec: WASM FFI Byte Mode for Archive Upload

- Status: draft
- Schema family, if any: none yet (no new serialized receipt schema is introduced by this stub)
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

- **Byte FFI mode (proposed)** — a core entrypoint that accepts an archive byte
  buffer plus a small JSON options object (logical root, scan options, ingestion
  limits, output mode) and returns the same envelope shape
  (`{ "ok": ..., "data": ..., "error": ... }`) that the JSON modes already
  return. It composes the landed ZIP→scan consumer; it does not introduce a new
  scan path. The byte mode is gated by the same decompression-dependency feature
  (`archive-zip`) so the default core/binding surface stays free of
  decompression dependencies.

- **WASM byte binding (proposed)** — a thin wasm-bindgen function that accepts a
  JS byte view (`Uint8Array`) plus options and forwards to the core byte mode,
  matching the thin-wrapper rule the rest of `crates/tokmd-wasm/src/lib.rs`
  follows. The crate keeps `#![forbid(unsafe_code)]`; the byte view is copied
  into an owned buffer at the boundary rather than aliased.

- **Capability promotion (gated)** — the experimental archive rows in
  `docs/browser-capability-matrix.md` and `docs/capabilities/wasm.json` move from
  "experimental, no WASM caller" to "supported" **only** once the binding and its
  `wasm-bindgen-test` parity coverage land. Until then the rows stay
  experimental and browser ZIP upload is not advertised.

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

These are the proof obligations a future implementing PR must satisfy; this stub
asserts none of them as already met.

- Byte/host parity: scanning a benign archive fixture through the byte mode must
  yield the same normalized file set and aggregated receipt as scanning the
  equivalent extracted tree through the host path, reusing the parity oracle the
  snapshot seam defines.
- Byte/JSON-mode parity: for the same logical file set, the byte-mode receipt
  must match the receipt produced by the existing `{ path, text }` JSON mode
  (same `schema_version`, same payload modulo volatile timestamps).
- Fail-closed admission: traversal/absolute/drive-prefix/NUL/non-regular and
  zip-bomb fixtures must each be rejected with a named error and produce no
  partial receipt, with no second admission path bypassing
  `crates/tokmd-io-port/src/archive.rs`.
- WASM boundary coverage: a `wasm-bindgen-test` must exercise the `Uint8Array`
  binding end to end and assert the browser payload matches the core payload, in
  the same style as the existing boundary tests in
  `crates/tokmd-wasm/src/lib.rs`.
- Default-surface guard: a build/test of the default (non-`archive-zip`) feature
  set must confirm no decompression dependency entered the default core or WASM
  surface.

Doc-shape checks for this spec stub itself:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
```

## Open Questions

- Should the byte mode be a new `run_json`-style mode discriminator (e.g. an
  `archive-zip` mode whose JSON args carry a base64 byte field) or a separate
  byte-taking entrypoint alongside `run_json`? A dedicated byte entrypoint avoids
  base64 inflation across the FFI boundary; a mode discriminator reuses the
  existing dispatch and envelope plumbing. The browser matrix notes both options.
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
