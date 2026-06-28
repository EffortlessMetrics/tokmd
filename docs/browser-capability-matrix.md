# Browser Capability Matrix

This is a capability-honest map of what the browser/WASM surface of `tokmd`
can do today, what is experimental and not yet wired to a browser caller, and
what stays native-only. It complements the narrative in [browser.md](browser.md)
and the machine-readable contract in
[`docs/capabilities/wasm.json`](capabilities/wasm.json).

It exists so that browser-adoption work does not claim capabilities the shipped
WASM bundle cannot perform. When this doc and `wasm.json` disagree, treat
`wasm.json` as the machine source of truth for per-command browser status and
fix whichever is stale.

## Shipped browser-safe surface

These are wired through `tokmd-wasm` (`crates/tokmd-wasm`), which reuses
`tokmd_core::ffi::run_json` over ordered in-memory `{ path, text }` inputs. They
are exercised by the `tokmd-wasm` test suite (native and `wasm-bindgen-test`).

| Capability | Browser status | Notes |
| --- | --- | --- |
| `lang` | supported | language receipt from in-memory inputs or ZIP archive bytes (`runJsonBytes`) |
| `module` | supported | module receipt from in-memory inputs or ZIP archive bytes (`runJsonBytes`) |
| `export` | supported | file inventory from in-memory inputs or ZIP archive bytes (`runJsonBytes`) |
| `analyze` (`receipt`, `estimate`) | partial | rootless presets only; richer presets need host backing; archive bytes via `runJsonBytes` |
| `runJsonBytes` (`archive-zip`) | supported | raw ZIP `Uint8Array` upload; modes `lang`/`module`/`export`/`analyze` (rootless presets) |
| `capabilities()` / `version()` / `schemaVersion()` | supported | introspection helpers |

## Archive ingestion (ZIP byte upload)

The `runJsonBytes` binding (`tokmd-wasm`, `feature = archive-zip`) accepts a
browser `Uint8Array` of raw ZIP bytes plus a JSON options object and forwards to
`tokmd_core::ffi::run_json_bytes`. Untrusted bytes are admitted fail-closed by
the single authoritative engine in `tokmd-io-port` / `tokmd-scan`; there is no
second admission path. Coverage:

- native parity: `core_run_json_bytes_lang_matches_inline_inputs` in
  `crates/tokmd-wasm/src/lib.rs`
- `wasm-bindgen-test` boundary: `run_json_bytes_lang_matches_inline_inputs_over_js_boundary`
  in the same file

The underlying snapshot/scan seams remain host-free infrastructure; they are
now reachable from the browser through this binding when the `archive-zip`
feature is enabled at build time.

| Capability | Where it lives | Browser status | Marker |
| --- | --- | --- | --- |
| `RepoSnapshot` / `MemFs` in-memory file set | `tokmd-io-port`, `tokmd-scan` | supported via `runJsonBytes` | host-free seam |
| `scan_snapshot` (snapshot-backed scan) | `tokmd-scan` | supported via `runJsonBytes` | host-free seam |
| `snapshot_from_zip_bytes` (ZIP codec) | `tokmd-io-port` (`archive-zip`) | supported via `runJsonBytes` | trust-surface feature |
| `scan_snapshot_from_zip` / `inputs_from_zip_bytes` (ZIP → scan) | `tokmd-scan` (`archive-zip`) | supported via `runJsonBytes` | trust-surface feature |

The `archive-zip` feature is decompression-dependency-gated: the default
`tokmd-wasm` build stays free of decompression dependencies, and the audited
deflate-only `zip` crate only enters when `archive-zip` is enabled.

## WASM blockers for ZIP upload (resolved)

Browser ZIP upload is now available through the `runJsonBytes` binding when
`tokmd-wasm` is built with the `archive-zip` feature. The prior blockers are
closed:

- `tokmd_core::ffi::run_json_bytes(mode, options_json, archive_bytes)` accepts
  raw archive bytes and returns the same envelope as the JSON modes.
- `tokmd-wasm` exposes `runJsonBytes(mode, optionsJson, archiveBytes:
  Uint8Array)`, copying the view into an owned buffer at the boundary.
- `wasm-bindgen-test` coverage exercises the `Uint8Array` path end-to-end and
  asserts byte-mode parity with inline `{ path, text }` inputs.

Remaining follow-on (out of scope for this slice): wire `runJsonBytes` into the
browser runner UI (`web/runner`) so `zipball` upload is user-facing; streaming
upload; tar-family containers.

## Native-only

These stay native-first and are not part of the browser surface. This mirrors
the boundaries in [browser.md](browser.md#native-only-boundaries) and the
`native_only` rows in [`docs/capabilities/wasm.json`](capabilities/wasm.json).

| Capability | Reason |
| --- | --- |
| native git history (churn, hotspots, freshness, coupling) | requires a git repository and `git log` |
| filesystem walk / ignore traversal without a snapshot | requires host `std::fs` and validated roots |
| `run`, `diff`, `cockpit`, `sensor`, `gate`, `context`, `handoff`, `baseline`, `packet` | require filesystem, validated roots, host clock, or git history |
| `badge`, `init`, `check-ignore`, `completions`, `tools` | native CLI surfaces |

## Claim boundary

- **Establishes**: the current browser-safe command set wired through
  `tokmd-wasm`, including ZIP archive byte upload via `runJsonBytes` when built
  with `archive-zip`, and an honest experimental/native-only split for git and
  filesystem capabilities.
- **Does not establish**: in-browser git history, browser-runner UI wiring for
  ZIP upload (`web/runner` `zipball` remains false until a follow-on PR),
  streaming upload, or tar-family containers.

## See also

- [browser.md](browser.md) — narrative browser runner overview and boundaries.
- [browser-to-native.md](browser-to-native.md) — bridge from browser receipts to
  native review packets, handoff bundles, and CI evidence.
- [specs/repo-snapshot.md](specs/repo-snapshot.md) — the in-memory snapshot and
  archive ingestion contract, including next integration points.
- [specs/wasm-ffi-byte-mode.md](specs/wasm-ffi-byte-mode.md) — the FFI byte-mode
  transport contract for browser archive upload (the remaining ZIP-upload seam).
- [`docs/capabilities/wasm.json`](capabilities/wasm.json) — machine-readable
  per-command browser capability contract.
