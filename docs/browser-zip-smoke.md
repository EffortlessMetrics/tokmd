# Browser ZIP Archive Manual Smoke

Manual verification recipe for the shipped `archive-zip` byte-mode chain:
`tokmd_core::ffi::run_json_bytes` → `tokmd-wasm::runJsonBytes` → `web/runner`
ZIP upload UI.

Use this when you need human-observed evidence that a real browser can load an
`archive-zip` WASM bundle, upload a ZIP file, and receive a successful receipt.
Automated coverage already exercises the worker/runtime protocol and byte-mode
parity in Rust tests; this doc closes the remaining **manual browser** gap called
out in [`browser-capability-matrix.md`](browser-capability-matrix.md) and
[`NOW.md`](NOW.md).

## Claim boundary

- **Establishes**: a maintainer can reproduce browser ZIP upload end-to-end with
  a locally built `archive-zip` WASM bundle and a hand-made ZIP fixture.
- **Does not establish**: streaming/large-archive upload, tar-family containers,
  release-artifact parity without rebuilding with `archive-zip`, or CI automation
  of browser UI tests.

Record outcomes in PR comments or release notes only after completing the browser
steps below; passing `npm test` or `cargo test -p tokmd-wasm --features archive-zip`
alone does not discharge this lane.

## Prerequisites

From the repository root:

- Rust stable with the `wasm32-unknown-unknown` target:
  `rustup target add wasm32-unknown-unknown`
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/) on `PATH`
- Node.js 20+ (for `npm --prefix web/runner test` and optional static server)

## 1. Build the `archive-zip` WASM bundle

The default `build:wasm` script builds analysis support **without**
decompression dependencies. ZIP upload requires the explicit feature gate:

```bash
npm --prefix web/runner run build:wasm:archive-zip
```

This writes the browser bundle to `web/runner/vendor/tokmd-wasm/` (same layout
expected by `worker.js`).

Optional pre-check (does not replace manual browser smoke):

```bash
npm --prefix web/runner test
cargo test -p tokmd-wasm --features analysis,archive-zip run_json_bytes
```

## 2. Create a ZIP fixture

The repo does not commit binary `.zip` files. Build a small fixture locally.
The entries below mirror `tokmd-wasm` archive parity tests (`src/lib.rs`,
`src/main.rs`, `tests/basic.py`) so a successful `lang` run should report
**3 files** and include Rust plus Python rows.

### Bash / macOS / Linux

```bash
ROOT="$(mktemp -d)/tokmd-smoke"
mkdir -p "${ROOT}/src" "${ROOT}/tests"
printf 'pub fn alpha() -> usize { 1 }\n' > "${ROOT}/src/lib.rs"
printf 'fn main() {}\n' > "${ROOT}/src/main.rs"
printf '# TODO: keep smoke\nprint('"'"'ok'"'"')\n' > "${ROOT}/tests/basic.py"
( cd "${ROOT}" && zip -r ../tokmd-smoke.zip . )
echo "fixture: ${ROOT%/tokmd-smoke}/tokmd-smoke.zip"
```

### PowerShell (Windows)

```powershell
$root = Join-Path $env:TEMP "tokmd-smoke"
New-Item -ItemType Directory -Force -Path "$root\src", "$root\tests" | Out-Null
Set-Content -NoNewline "$root\src\lib.rs" "pub fn alpha() -> usize { 1 }`n"
Set-Content -NoNewline "$root\src\main.rs" "fn main() {}`n"
Set-Content -NoNewline "$root/tests/basic.py" "# TODO: keep smoke`nprint('ok')`n"
$zip = Join-Path $env:TEMP "tokmd-smoke.zip"
if (Test-Path $zip) { Remove-Item $zip }
Compress-Archive -Path "$root\*" -DestinationPath $zip
Write-Host "fixture: $zip"
```

## 3. Serve the browser runner

WASM workers require HTTP(S); opening `index.html` via `file://` will fail.

From the repository root:

```bash
npx --yes serve web/runner -p 8080
```

Or, from `web/runner/`:

```bash
python -m http.server 8080
```

Open `http://localhost:8080/` in a current Chromium, Firefox, or Safari build.

## 4. Manual browser steps

1. Wait for the status panel to report the WASM worker initialized.
2. Confirm capability hints show **`zipball: yes`** (or equivalent log line
   `zipballCapability: wasm runJsonBytes`). If **`zipball: no`**, the loaded
   bundle was not built with `archive-zip`; rebuild step 1.
3. Under **ZIP Archive**, choose the fixture from step 2.
4. Click **Load ZIP Archive**. Expect a success status with the byte size and
   `strategy: zip-archive-bytes` in the load log.
5. Set **Mode** to `lang`. The args JSON should be byte-mode (no `inputs` or
   `paths` keys).
6. Click **Run**. Expect progress phases including **Decoding ZIP archive
   bytes**, then a completed run.
7. Inspect the result pane:
   - envelope `ok: true`
   - `data.mode` is `lang`
   - `data.total.files` is **3**
   - language rows include Rust and Python
8. Download the JSON artifact and keep it as optional smoke evidence.

Repeat once with **Mode** `export` if you want extra confidence; the load path
is the same and only the receipt shape changes.

## 5. Failure signals

| Symptom | Likely cause |
| --- | --- |
| ZIP controls disabled | WASM bundle lacks `runJsonBytes`; rebuild with `archive-zip` |
| `loaded wasm bundle does not expose runJsonBytes` | Same as above |
| `ZIP load failed` / decode error in run | corrupt fixture or unsupported compression; recreate fixture |
| `archive runs require byte-mode options without inputs or paths` | args JSON still contains `inputs`/`paths`; reload ZIP to reset args |
| Worker init error / blank page | served over `file://` or missing `vendor/tokmd-wasm` build output |

## 6. Optional native parity check

After a successful browser run, compare against the Rust byte-mode entrypoint
using the same ZIP bytes (base64-free file read):

```bash
# Requires archive-zip enabled in tokmd-core (same feature chain as tokmd-wasm).
python - <<'PY'
import json, pathlib, subprocess, sys
zip_path = pathlib.Path("/tmp/tokmd-smoke.zip")  # adjust if needed
b = zip_path.read_bytes()
# Use cargo test helper path only when integrating into CI; for manual smoke,
# rely on the browser receipt from step 4.
print(f"zip bytes: {len(b)}")
PY
```

For strict parity during development, prefer:

```bash
cargo test -p tokmd-wasm --features analysis,archive-zip core_run_json_bytes_lang_matches_inline_inputs -- --nocapture
```

## See also

- [`browser-capability-matrix.md`](browser-capability-matrix.md) — capability
  honesty map including ZIP upload status.
- [`browser.md`](browser.md) — browser runner overview and native boundaries.
- [`specs/wasm-ffi-byte-mode.md`](specs/wasm-ffi-byte-mode.md) — FFI byte-mode
  contract.
- [`web/runner/README.md`](../web/runner/README.md) — runner integration notes.
