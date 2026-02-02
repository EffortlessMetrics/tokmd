# @tokmd/core

Node.js bindings for [tokmd](https://github.com/EffortlessMetrics/tokmd) - fast code inventory receipts and analytics.

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
import { lang, module, analyze } from '@tokmd/core';

// Get language summary
const langResult = await lang({ paths: ['src'] });
for (const row of langResult.rows) {
  console.log(`${row.lang}: ${row.code} lines`);
}

// Get module breakdown
const moduleResult = await module({ paths: ['.'] });
for (const row of moduleResult.rows) {
  console.log(`${row.module}: ${row.code} lines`);
}

// Run analysis
const analysisResult = await analyze({ paths: ['.'], preset: 'health' });
if (analysisResult.derived) {
  console.log(`Total: ${analysisResult.derived.totals.code} lines`);
}
```

## API Reference

All functions return Promises and are non-blocking (they use tokio's spawn_blocking to not block the event loop).

### Functions

#### `lang(options?: LangOptions): Promise<LangReceipt>`

Scan paths and return a language summary.

```typescript
interface LangOptions {
  paths?: string[]              // Paths to scan (default: ["."])
  top?: number                  // Show only top N languages (0 = all)
  files?: boolean               // Include file counts
  children?: 'collapse' | 'separate'  // How to handle embedded languages
  redact?: 'none' | 'paths' | 'all'  // Redaction mode
  excluded?: string[]           // Glob patterns to exclude
  hidden?: boolean              // Include hidden files
}
```

#### `module(options?: ModuleOptions): Promise<ModuleReceipt>`

Scan paths and return a module summary.

```typescript
interface ModuleOptions {
  paths?: string[]              // Paths to scan (default: ["."])
  top?: number                  // Show only top N modules (0 = all)
  module_roots?: string[]       // Module root directories
  module_depth?: number         // Path segments for module roots (default: 2)
  children?: 'separate' | 'parents-only'  // Embedded language handling
  // ...other options
}
```

#### `export(options?: ExportOptions): Promise<ExportReceipt>`

Scan paths and return file-level export data.

```typescript
interface ExportOptions {
  paths?: string[]
  format?: 'jsonl' | 'json' | 'csv' | 'cyclonedx'
  min_code?: number             // Minimum lines of code
  max_rows?: number             // Maximum rows (0 = unlimited)
  // ...other options
}
```

#### `analyze(options?: AnalyzeOptions): Promise<AnalysisReceipt>`

Run analysis on paths and return derived metrics.

```typescript
interface AnalyzeOptions {
  paths?: string[]
  preset?: 'receipt' | 'health' | 'risk' | 'supply' | 'architecture' |
           'topics' | 'security' | 'identity' | 'git' | 'deep' | 'fun'
  window?: number               // Context window size in tokens
  git?: boolean                 // Force enable/disable git metrics
  max_files?: number
  max_bytes?: number
  max_commits?: number
}
```

#### `diff(fromPath: string, toPath: string): Promise<DiffReceipt>`

Compare two receipts or paths and return a diff.

### Low-Level API

#### `runJson(mode: string, argsJson: string): Promise<string>`

Run any tokmd operation with JSON string arguments.

```javascript
const result = await runJson('lang', JSON.stringify({ paths: ['.'], top: 10 }));
const data = JSON.parse(result);
```

#### `run(mode: string, args: object): Promise<object>`

Run any tokmd operation with an object.

```javascript
const result = await run('lang', { paths: ['.'], top: 10 });
console.log(result.rows[0].lang);
```

### Constants

#### `version(): string`

Returns the tokmd version string.

#### `schemaVersion(): number`

Returns the current JSON schema version.

## TypeScript

Full TypeScript support with type definitions included.

```typescript
import { lang, LangReceipt, LangRow } from '@tokmd/core';

const result: LangReceipt = await lang({ paths: ['src'] });
const rows: LangRow[] = result.rows;
```

## Platform Support

Pre-built binaries are available for:
- macOS (x64, arm64)
- Linux (x64, arm64) - glibc
- Windows (x64)

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
