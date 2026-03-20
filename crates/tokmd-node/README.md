# @tokmd/core

Node.js bindings for [tokmd](https://github.com/EffortlessMetrics/tokmd): deterministic repo receipts, analysis, cockpit metrics, and diff workflows from JavaScript.

## Installation

```bash
npm install @tokmd/core
# or
yarn add @tokmd/core
# or
pnpm add @tokmd/core
```

## Quick Start

```javascript
import { lang, analyze, diff } from '@tokmd/core';

// Language summary
const langReceipt = await lang({ paths: ['src'], top: 5 });
for (const row of langReceipt.report.rows) {
  console.log(`${row.lang}: ${row.code} lines`);
}

// Effort-focused analysis (1.8.0 preset)
const analysisReceipt = await analyze({
  paths: ['.'],
  preset: 'estimate',
  effort_base_ref: 'main',
  effort_head_ref: 'HEAD',
});

if (analysisReceipt.effort) {
  console.log(`P50 effort: ${analysisReceipt.effort.results.effort_pm_p50}`);
}

// Compare two saved receipts or run directories
const diffReceipt = await diff('.runs/base/lang.json', '.runs/current/lang.json');
console.log(diffReceipt.mode);
```

## API Surface

High-level Promise-based helpers:

- `lang(options?)`
- `module(options?)`
- `export(options?)`
- `analyze(options?)`
- `cockpit(options?)`
- `diff(fromPath, toPath)`

Low-level helpers:

- `run(mode, args)`
- `runJson(mode, argsJson)`
- `version()`
- `schemaVersion()`

These wrappers sit on top of `tokmd-core` and use `spawn_blocking` internally so long-running scans do not block the Node event loop.

## Low-Level Modes

`run()` and `runJson()` target the shared JSON/FFI workflow boundary. Supported modes are:

- `lang`
- `module`
- `export`
- `analyze`
- `cockpit`
- `diff`
- `version`

The JSON envelope is stable:

- success: `{"ok": true, "data": {...}}`
- error: `{"ok": false, "error": {...}}`

## Notes

- Current analysis presets include `estimate`, `risk`, `deep`, and `fun`.
- `estimate` is the effort-focused preset added in `1.8.0`.
- For exact TypeScript shapes, use the bundled declarations in [`index.d.ts`](./index.d.ts).

## Platform Support

Prebuilt binaries are available for:

- macOS (`x64`, `arm64`)
- Linux (`x64`, `arm64`)
- Windows (`x64`)

## Development

Building from source requires Rust and napi-rs:

```bash
cd crates/tokmd-node
npm install
npm run build
npm test
```

## License

MIT OR Apache-2.0
