# @tokmd/core

Async Node.js bindings for tokmd.

## Problem

Use tokmd from JavaScript without blocking the event loop or hand-parsing JSON envelopes.

## What it gives you

- Promise-based helpers: `lang`, `module`, `export`, `analyze`, `cockpit`, and `diff`
- Low-level access: `run` and `runJson`
- Sync metadata helpers: `version` and `schemaVersion`
- Async Rust bridge built on `napi-rs` and `spawn_blocking`

## Quick use / integration notes

```javascript
import { lang, analyze, diff } from '@tokmd/core';

const summary = await lang({ paths: ['src'], top: 5 });
const analysis = await analyze({ paths: ['.'], preset: 'estimate' });
const delta = await diff({ from: '.runs/base/lang.json', to: '.runs/current/lang.json' });
```

The published npm package is `@tokmd/core`. Exact TypeScript shapes live in `index.d.ts`.

## Go deeper

### Tutorial

- `../../docs/tutorial.md`

### How-to

- `../../docs/reference-cli.md`

### Reference

- `index.d.ts`
- `src/lib.rs`
