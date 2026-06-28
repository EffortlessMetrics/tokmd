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
| `lang` | supported | language receipt from in-memory inputs |
| `module` | supported | module receipt from in-memory inputs |
| `export` | supported | file inventory from in-memory inputs |
| `analyze` (`receipt`, `estimate`) | partial | rootless presets only; richer presets need host backing |
| `capabilities()` / `version()` / `schemaVersion()` | supported | introspection helpers |

## Experimental: snapshot and archive ingestion

These seams exist in the crate graph and are covered by their own crate tests,
but they are **not exposed through `tokmd-wasm` yet**. They are experimental
until a browser/worker caller wires them with browser-level tests. See the
[repo-snapshot spec](specs/repo-snapshot.md) for the contract and incremental
status.

| Capability | Where it lives | Browser status | Marker |
| --- | --- | --- | --- |
| `RepoSnapshot` / `MemFs` in-memory file set | `tokmd-io-port`, `tokmd-scan` | experimental, no WASM caller | host-free seam |
| `scan_snapshot` (snapshot-backed scan) | `tokmd-scan` | experimental, no WASM caller | host-free seam |
| `snapshot_from_zip_bytes` (ZIP codec) | `tokmd-io-port` (`archive-zip`) | experimental, no WASM caller | trust-surface feature |
| `scan_snapshot_from_zip` (ZIP → scan) | `tokmd-scan` (`archive-zip`) | experimental, no WASM caller | trust-surface feature |

The `archive-zip` feature is decompression-dependency-gated: the default
`tokmd-scan` surface stays free of decompression dependencies, and the audited
deflate-only `zip` crate only enters when `archive-zip` is enabled.

## WASM blockers for ZIP upload

`scan_snapshot_from_zip` is the natural browser entry point for an
archive-upload flow, but it is not yet callable from `tokmd-wasm`. The blockers
are concrete, not philosophical:

- `tokmd-wasm` routes every mode through `tokmd_core::ffi::run_json(mode,
  args_json)`, which takes a **JSON string** and has no binary/byte input path.
  Raw ZIP bytes cannot be passed without either a base64 argument convention or
  a dedicated `&[u8]` / `Uint8Array` binding.
- `tokmd-core` does not currently expose an `archive-zip` FFI mode, and does not
  depend on `tokmd-scan`'s `archive-zip` feature, so the byte-admission path is
  not reachable from the core entrypoint the WASM crate consumes.
- There is no `wasm-bindgen-test` coverage for an upload path, so claiming
  browser ZIP support would be unproven.

Until those are addressed in a dedicated PR (core FFI byte mode + `tokmd-wasm`
binding + `wasm-bindgen-test` parity coverage), ZIP upload in the browser is
**not available** and should not be advertised. This doc is the standing record
of that boundary; do not mark the archive rows "supported" until the binding and
its tests land.

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
  `tokmd-wasm`, and an honest experimental/native-only split for snapshot,
  archive, git, and filesystem capabilities.
- **Does not establish**: browser ZIP upload, in-browser git history, or any
  promotion of an experimental seam to a shipped browser capability. Those
  require the binding and tests described above before any "supported" claim.

## See also

- [browser.md](browser.md) — narrative browser runner overview and boundaries.
- [browser-to-native.md](browser-to-native.md) — bridge from browser receipts to
  native review packets, handoff bundles, and CI evidence.
- [specs/repo-snapshot.md](specs/repo-snapshot.md) — the in-memory snapshot and
  archive ingestion contract, including next integration points.
- [`docs/capabilities/wasm.json`](capabilities/wasm.json) — machine-readable
  per-command browser capability contract.
