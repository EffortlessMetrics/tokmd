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

## Rootless analyze preset feasibility

Browser `analyze` is **partial**: only `receipt` and `estimate` are wired today. The
authority chain is explicit in code and mirrored in
[`docs/capabilities/wasm.json`](capabilities/wasm.json):

1. **`tokmd_core::supports_rootless_in_memory_analyze_preset`** — the core gate
   (`crates/tokmd-core/src/workflows/analyze.rs`). Only `receipt` and `estimate`
   stay on the pure in-memory row path; every other preset materializes a
   temporary scan root via `prepare_materialized_in_memory_export`.
2. **`ROOTLESS_ANALYZE_PRESETS`** — the wasm binding constant
   (`crates/tokmd-wasm/src/lib.rs`) advertised through `capabilities()`.
3. **`commands.analyze.browser_analyze_presets`** — the machine-readable row in
   `wasm.json`, kept in sync by `xtask/tests/docs_schema_w72.rs`.

The wasm FFI rejects any other preset at the boundary with a `not_implemented`
error before analysis runs.

### Currently browser-safe analyze presets

| Preset | Browser status | Why it works rootless |
| --- | --- | --- |
| `receipt` | supported | Pure in-memory export rows; derived metrics from scan totals only |
| `estimate` | supported | Same row path; effort estimation degrades git/file-backed signals with named warnings when `root` is empty (see `estimate_with_rootless_context_emits_host_root_warnings` in `tokmd-analysis`) |

### Blocked presets and why

All other presets are blocked at one or more layers. The table below is grouped
by the **primary** blocker; several presets hit multiple layers.

| Preset | Primary blocker | What the preset needs | Current rootless behaviour |
| --- | --- | --- | --- |
| `health` | browser FFI gate + file-backed enrichers | TODO density, complexity histogram (`needs_files()`) | Not in `ROOTLESS_ANALYZE_PRESETS`; would materialize temp scan root |
| `risk` | git history + file-backed enrichers | Hotspots, coupling, freshness, complexity | Git enrichers require host `git log`; not browser-safe |
| `supply` | file-backed enrichers | Asset discovery, dependency lockfiles | Requires filesystem walk beyond in-memory rows |
| `architecture` | file-backed enrichers | Import graph | Requires content/import scanning on host paths |
| `topics` | file-backed enrichers | Semantic topic clouds | Requires content scanning |
| `security` | file-backed enrichers | License radar, entropy profiling | Requires filesystem/content surfaces |
| `identity` | git history | Archetype detection, corporate fingerprint | Fingerprint/churn need commit history |
| `git` | git history | Advanced git metrics | Native `git log` only |
| `deep` | git + file-backed enrichers | Everything except fun | Combines all blockers above |
| `fun` | browser FFI gate | Eco-label (no `needs_files()`) | Could run on in-memory rows but is not wired through wasm |
| `bun-ub` | git history + native-only command | UB review evidence packet | Native `evidence-packet` / `packet generate`; not a browser command |

Shared infrastructure gaps:

- **Git history** — seven presets enable git enrichers (`receipt`, `estimate`,
  `bun-ub`, `risk`, `identity`, `git`, `deep`). In a rootless/browser context
  git is skipped and warnings are emitted; full signals need a host repository.
- **Filesystem walk** — enrichers gated by `PresetPlan::needs_files()` (assets,
  deps, TODOs, duplicates, imports, entropy, license, complexity, API surface)
  expect a validated host root or materialized scan directory, not bare in-memory
  rows alone.
- **WASM preset gate** — even presets that could theoretically degrade (e.g.
  `fun`) are rejected until explicitly added to `ROOTLESS_ANALYZE_PRESETS` and
  proven through the wasm parity tests.

### What would unblock additional presets

Work proceeds preset-by-preset; there is no single switch.

| Unblock step | Enables | Owner seam |
| --- | --- | --- |
| Run content enrichers on in-memory `{ path, text }` export rows without host-root file walks | `health`, parts of `security` | `tokmd-analysis` content enrichers + `tokmd-scan` snapshot path |
| Snapshot-backed asset/dep/import discovery (no `std::fs` walk) | `supply`, `architecture` | `tokmd-scan::scan_snapshot`, `tokmd-io-port` |
| Explicit partial/degraded receipts for skipped git enrichers in browser | `risk`, `identity`, `git`, `deep` (partial) | `tokmd-analysis` git enrichers + wasm error policy |
| Extend `supports_rootless_in_memory_analyze_preset` + `ROOTLESS_ANALYZE_PRESETS` + `wasm.json` + parity tests | Any newly proven preset | `tokmd-core`, `tokmd-wasm`, `docs/capabilities/wasm.json` |
| Browser git history adapter (host-provided commit stream) | Full git presets in browser | New host port; out of scope for current wasm bundle |

Until a preset clears all three layers (core rootless path, wasm FFI gate, and
parity proof), it stays **native-only** in the capability matrix regardless of
whether native in-memory APIs could materialize a temp scan root.

## Archive ingestion (ZIP byte upload)

The `runJsonBytes` binding (`tokmd-wasm`, `feature = archive-zip`) accepts a
browser `Uint8Array` of raw ZIP bytes plus a JSON options object and forwards to
`tokmd_core::ffi::run_json_bytes`. Untrusted bytes are admitted fail-closed by
the single authoritative engine in `tokmd-io-port` / `tokmd-scan`; there is no
second admission path. Every browser-supported byte-mode is proven to match the
equivalent inline `{ path, text }` envelope, both natively and across the JS
boundary. Coverage (all in `crates/tokmd-wasm/src/lib.rs`):

- native parity: `core_run_json_bytes_lang_matches_inline_inputs`,
  `core_run_json_bytes_module_matches_inline_inputs`,
  `core_run_json_bytes_export_matches_inline_inputs`,
  `core_run_json_bytes_analyze_receipt_matches_inline_inputs`,
  `core_run_json_bytes_analyze_estimate_matches_inline_inputs`
- `wasm-bindgen-test` boundary:
  `run_json_bytes_lang_matches_inline_inputs_over_js_boundary`,
  `run_json_bytes_module_matches_inline_inputs_over_js_boundary`,
  `run_json_bytes_export_matches_inline_inputs_over_js_boundary`,
  `run_json_bytes_analyze_receipt_matches_inline_inputs_over_js_boundary`,
  `run_json_bytes_analyze_estimate_matches_inline_inputs_over_js_boundary`

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

Remaining follow-on (out of scope for this slice): streaming
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
  with `archive-zip`; an honest experimental/native-only split for git and
  filesystem capabilities; and a code-backed rootless analyze preset
  feasibility map (`receipt`/`estimate` only today, with per-preset blockers).
- **Does not establish**: in-browser git history, manual browser smoke of the
  runner ZIP upload path (see [browser-zip-smoke.md](browser-zip-smoke.md) for
  the maintainer recipe), streaming upload, tar-family containers, or a timeline
  for widening `ROOTLESS_ANALYZE_PRESETS`.

## See also

- [browser-zip-smoke.md](browser-zip-smoke.md) — manual browser verification
  steps for ZIP archive upload with an `archive-zip` WASM build.
- [browser.md](browser.md) — narrative browser runner overview and boundaries.
- [browser-to-native.md](browser-to-native.md) — bridge from browser receipts to
  native review packets, handoff bundles, and CI evidence.
- [specs/repo-snapshot.md](specs/repo-snapshot.md) — the in-memory snapshot and
  archive ingestion contract, including next integration points.
- [specs/wasm-ffi-byte-mode.md](specs/wasm-ffi-byte-mode.md) — the FFI byte-mode
  transport contract for browser archive upload (the remaining ZIP-upload seam).
- [`docs/capabilities/wasm.json`](capabilities/wasm.json) — machine-readable
  per-command browser capability contract.
