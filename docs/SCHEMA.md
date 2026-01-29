# tokmd Receipt Schema

> **Note**: A machine-readable JSON Schema definition is available at [schema.json](schema.json).

`tokmd` produces structured JSON outputs called "receipts". These schemas are stable and intended for machine consumption.

**Current schema version**: 2

All JSON outputs share a common envelope structure.

## Common Fields

Every receipt includes:

| Field | Type | Description |
| :--- | :--- | :--- |
| `schema_version` | `integer` | The schema version (currently 2). Incremented on breaking changes. |
| `generated_at_ms` | `integer` | Unix timestamp (milliseconds) when the scan ran. |
| `tool` | `object` | Information about the tool version. |
| `tool.name` | `string` | Always `"tokmd"`. |
| `tool.version` | `string` | The version of tokmd used (e.g., `"1.2.0"`). |
| `mode` | `string` | One of `"lang"`, `"module"`, `"export"`, or `"analysis"`. |
| `status` | `string` | Scan status: `"complete"` or `"partial"`. |
| `warnings` | `array` | Array of warning strings generated during the scan. |
| `scan` | `object` | The configuration used for the file scan. |

### Scan Configuration (`scan`)

| Field | Type | Description |
| :--- | :--- | :--- |
| `paths` | `array` | Input paths scanned. |
| `excluded` | `array` | Patterns excluded from scan. |
| `excluded_redacted` | `boolean` | True if excluded patterns were redacted (replaced with hashes). |
| `config` | `string` | Configuration mode: `"auto"` or `"none"`. |
| `hidden` | `boolean` | Whether hidden files were included. |
| `no_ignore` | `boolean` | Whether all ignore files were disregarded. |
| `no_ignore_parent` | `boolean` | Whether parent ignore files were disregarded. |
| `no_ignore_dot` | `boolean` | Whether .ignore files were disregarded. |
| `no_ignore_vcs` | `boolean` | Whether VCS ignore files (.gitignore) were disregarded. |
| `treat_doc_strings_as_comments` | `boolean` | Whether doc strings were counted as comments. |

---

## 1. Language Receipt (`mode: "lang"`)

Produced by `tokmd --format json` or `tokmd lang --format json`.

**Schema version**: 2

```json
{
  "schema_version": 2,
  "generated_at_ms": 1706350000000,
  "tool": { "name": "tokmd", "version": "1.0.0" },
  "mode": "lang",
  "status": "complete",
  "warnings": [],
  "scan": { ... },
  "args": {
    "format": "json",
    "top": 0,
    "with_files": false,
    "children": "collapse"
  },
  "rows": [
    {
      "lang": "Rust",
      "code": 1000,
      "lines": 1200,
      "files": 10,
      "bytes": 36000,
      "tokens": 9000,
      "avg_lines": 120
    },
    {
      "lang": "Markdown",
      "code": 200,
      "lines": 300,
      "files": 5,
      "bytes": 9000,
      "tokens": 2250,
      "avg_lines": 60
    }
  ],
  "total": {
    "code": 1200,
    "lines": 1500,
    "files": 15,
    "bytes": 45000,
    "tokens": 11250,
    "avg_lines": 100
  },
  "with_files": false,
  "children": "collapse",
  "top": 0
}
```

### Language Receipt Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `args.format` | `string` | Output format used. |
| `args.top` | `integer` | Top N languages to show (0 = all). |
| `args.with_files` | `boolean` | Whether file counts were included. |
| `args.children` | `string` | How embedded languages are handled: `"collapse"` or `"separate"`. |
| `rows` | `array` | Array of language rows. |
| `total` | `object` | Aggregate totals across all languages. |
| `with_files` | `boolean` | Flattened from report: whether file counts were included. |
| `children` | `string` | Flattened from report: children handling mode. |
| `top` | `integer` | Flattened from report: top N limit used. |

### Language Row Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `lang` | `string` | Language name (e.g., "Rust", "Markdown"). |
| `code` | `integer` | Lines of code. |
| `lines` | `integer` | Total lines (code + comments + blanks). |
| `files` | `integer` | Number of files. |
| `bytes` | `integer` | Total file size in bytes for this language. |
| `tokens` | `integer` | Estimated token count for this language. |
| `avg_lines` | `integer` | Average lines per file. |

## 2. Module Receipt (`mode: "module"`)

Produced by `tokmd module --format json`.

**Schema version**: 2

```json
{
  "schema_version": 2,
  "generated_at_ms": 1706350000000,
  "tool": { "name": "tokmd", "version": "1.0.0" },
  "mode": "module",
  "status": "complete",
  "warnings": [],
  "scan": { ... },
  "args": {
    "format": "json",
    "module_roots": ["crates", "packages"],
    "module_depth": 2,
    "children": "separate",
    "top": 0
  },
  "rows": [
    {
      "module": "crates/cli",
      "code": 500,
      "lines": 600,
      "files": 4,
      "bytes": 18000,
      "tokens": 4500,
      "avg_lines": 150
    }
  ],
  "total": {
    "code": 500,
    "lines": 600,
    "files": 4,
    "bytes": 18000,
    "tokens": 4500,
    "avg_lines": 150
  },
  "module_roots": ["crates", "packages"],
  "module_depth": 2,
  "children": "separate",
  "top": 0
}
```

### Module Receipt Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `args.format` | `string` | Output format used. |
| `args.module_roots` | `array` | Module root directories. |
| `args.module_depth` | `integer` | Module depth limit. |
| `args.children` | `string` | How embedded languages are handled: `"separate"` or `"parents-only"`. |
| `args.top` | `integer` | Top N modules to show (0 = all). |
| `rows` | `array` | Array of module rows. |
| `total` | `object` | Aggregate totals across all modules. |
| `module_roots` | `array` | Flattened from report: module root directories. |
| `module_depth` | `integer` | Flattened from report: module depth limit. |
| `children` | `string` | Flattened from report: children handling mode. |
| `top` | `integer` | Flattened from report: top N limit used. |

### Module Row Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `module` | `string` | Module path/name. |
| `code` | `integer` | Lines of code. |
| `lines` | `integer` | Total lines (code + comments + blanks). |
| `files` | `integer` | Number of files. |
| `bytes` | `integer` | Total file size in bytes for this module. |
| `tokens` | `integer` | Estimated token count for this module. |
| `avg_lines` | `integer` | Average lines per file. |

## 3. Export Data (`mode: "export"`)

Produced by `tokmd export`. The default format is JSONL, but JSON and CSV are also available.

**Schema version**: 2

### JSONL Format (default)

JSONL output consists of a **Meta Record** (first line) followed by **Data Rows**.

#### Meta Record (Line 1)

```json
{
  "type": "meta",
  "schema_version": 2,
  "generated_at_ms": 1706350000000,
  "tool": { "name": "tokmd", "version": "1.0.0" },
  "mode": "export",
  "status": "complete",
  "warnings": [],
  "scan": { ... },
  "args": {
    "format": "jsonl",
    "module_roots": ["crates", "packages"],
    "module_depth": 2,
    "children": "separate",
    "min_code": 0,
    "max_rows": 0,
    "redact": "none",
    "strip_prefix": null
  }
}
```

#### Data Row (Lines 2+)

```json
{
  "type": "row",
  "path": "src/main.rs",
  "module": "src",
  "lang": "Rust",
  "kind": "parent",
  "code": 120,
  "comments": 10,
  "blanks": 5,
  "lines": 135,
  "bytes": 4200,
  "tokens": 1050
}
```

### JSON Format

When using `--format json`, the output is a single JSON object:

```json
{
  "schema_version": 2,
  "generated_at_ms": 1706350000000,
  "tool": { "name": "tokmd", "version": "1.0.0" },
  "mode": "export",
  "status": "complete",
  "warnings": [],
  "scan": { ... },
  "args": { ... },
  "rows": [
    { "path": "src/main.rs", "module": "src", "lang": "Rust", ... }
  ],
  "module_roots": ["crates", "packages"],
  "module_depth": 2,
  "children": "separate"
}
```

### Export Args Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `format` | `string` | Output format: `"csv"`, `"jsonl"`, or `"json"`. |
| `module_roots` | `array` | Module root directories. |
| `module_depth` | `integer` | Module depth limit. |
| `children` | `string` | How embedded languages are handled: `"separate"` or `"parents-only"`. |
| `min_code` | `integer` | Minimum code lines filter. |
| `max_rows` | `integer` | Maximum rows to output (0 = unlimited). |
| `redact` | `string` | Redaction mode: `"none"`, `"paths"`, or `"all"`. |
| `strip_prefix` | `string\|null` | Path prefix to strip from output paths. |
| `strip_prefix_redacted` | `boolean` | True if strip_prefix was redacted (replaced with a hash). |

### Data Row Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `type` | `string` | Always `"row"` for data rows (JSONL only). |
| `path` | `string` | Normalized file path (forward slashes). |
| `module` | `string` | Module key based on configured roots/depth. |
| `lang` | `string` | Detected language. |
| `kind` | `string` | `"parent"` for physical files, `"child"` for embedded code blocks. |
| `code` | `integer` | Lines of code. |
| `comments` | `integer` | Lines of comments. |
| `blanks` | `integer` | Blank lines. |
| `lines` | `integer` | Total lines (code + comments + blanks). |
| `bytes` | `integer` | File size in bytes. |
| `tokens` | `integer` | Estimated token count. |

### Redaction

If `--redact paths` or `--redact all` is used:
- `path`: Replaced with a BLAKE3 hash (preserving extension).
- `module`: Replaced with a hash (if `all`).
- `excluded` patterns in scan args are also redacted.
- `strip_prefix` is redacted if present, and `strip_prefix_redacted` is set to `true`.

```json
{
  "type": "row",
  "path": "a1b2c3d4e5f67890.rs",
  ...
}
```

### Totals Object

The `total` object in language and module receipts contains aggregate metrics:

| Field | Type | Description |
| :--- | :--- | :--- |
| `code` | `integer` | Total lines of code. |
| `lines` | `integer` | Total lines (code + comments + blanks). |
| `files` | `integer` | Total number of files processed. |
| `bytes` | `integer` | Total file size in bytes. |
| `tokens` | `integer` | Estimated total token count. |
| `avg_lines` | `integer` | Average lines per file. |

---

## 4. Analysis Receipt (`mode: "analysis"`)

Produced by `tokmd analyze --format json`.

**Schema version**: 2

Analysis receipts contain derived metrics and optional enrichments. All sections except `source`, `args`, and `derived` are optional based on the preset used.

### Envelope

```json
{
  "schema_version": 2,
  "generated_at_ms": 1706350000000,
  "tool": { "name": "tokmd", "version": "1.2.0" },
  "mode": "analysis",
  "status": "complete",
  "warnings": [],
  "source": { ... },
  "args": { ... },
  "derived": { ... },
  "archetype": { ... },
  "topics": { ... },
  "entropy": { ... },
  "predictive_churn": { ... },
  "corporate_fingerprint": { ... },
  "license": { ... },
  "assets": { ... },
  "deps": { ... },
  "git": { ... },
  "imports": { ... },
  "dup": { ... },
  "fun": { ... }
}
```

### Source Metadata

```json
{
  "source": {
    "inputs": ["."],
    "export_path": null,
    "base_receipt_path": null,
    "export_schema_version": 1,
    "export_generated_at_ms": 1706350000000,
    "base_signature": "abc123...",
    "module_roots": ["crates"],
    "module_depth": 2,
    "children": "collapse"
  }
}
```

### Args Metadata

```json
{
  "args": {
    "preset": "receipt",
    "format": "json",
    "window_tokens": 128000,
    "git": true,
    "max_files": null,
    "max_bytes": null,
    "max_commits": 1000,
    "max_commit_files": 100,
    "max_file_bytes": null,
    "import_granularity": "module"
  }
}
```

### Derived Metrics

Always present. Computed from receipt data without additional I/O.

```json
{
  "derived": {
    "totals": {
      "files": 120,
      "code": 10000,
      "comments": 1500,
      "blanks": 800,
      "lines": 12300,
      "bytes": 450000,
      "tokens": 112500
    },
    "doc_density": {
      "total": { "key": "total", "numerator": 1500, "denominator": 10000, "ratio": 0.15 },
      "by_lang": [...],
      "by_module": [...]
    },
    "whitespace": { ... },
    "verbosity": { ... },
    "max_file": {
      "overall": { "path": "src/big.rs", "code": 2000, ... },
      "by_lang": [...],
      "by_module": [...]
    },
    "lang_purity": { ... },
    "nesting": { "max": 8, "avg": 3.2, "by_module": [...] },
    "test_density": {
      "test_lines": 2000,
      "prod_lines": 8000,
      "test_files": 15,
      "prod_files": 105,
      "ratio": 0.25
    },
    "boilerplate": { ... },
    "polyglot": { "lang_count": 5, "entropy": 1.2, "dominant_lang": "Rust", ... },
    "distribution": {
      "count": 120,
      "min": 10,
      "max": 2000,
      "mean": 83.3,
      "median": 45,
      "p90": 200,
      "p99": 800,
      "gini": 0.42
    },
    "histogram": [
      { "label": "tiny", "min": 0, "max": 50, "files": 60, "pct": 0.5 },
      { "label": "small", "min": 51, "max": 200, "files": 40, "pct": 0.33 },
      ...
    ],
    "top": {
      "largest_lines": [...],
      "largest_tokens": [...],
      "largest_bytes": [...],
      "least_documented": [...],
      "most_dense": [...]
    },
    "tree": "crates/\n  cli/\n  core/\n...",
    "reading_time": { "minutes": 45.5, "lines_per_minute": 200, "basis_lines": 10000 },
    "context_window": { "window_tokens": 128000, "total_tokens": 112500, "pct": 0.88, "fits": true },
    "cocomo": {
      "mode": "organic",
      "kloc": 10.0,
      "effort_pm": 25.2,
      "duration_months": 8.1,
      "staff": 3.1,
      "a": 2.4, "b": 1.05, "c": 2.5, "d": 0.38
    },
    "todo": { "total": 42, "density_per_kloc": 4.2, "tags": [...] },
    "integrity": { "algo": "blake3", "hash": "abc123...", "entries": 120 }
  }
}
```

### Git Metrics (Optional)

Present when `--git` is enabled or preset includes git analysis.

```json
{
  "git": {
    "commits_scanned": 500,
    "files_seen": 200,
    "hotspots": [
      { "path": "src/main.rs", "commits": 47, "lines": 500, "score": 23500 }
    ],
    "bus_factor": [
      { "module": "crates/core", "authors": 1 }
    ],
    "freshness": {
      "threshold_days": 180,
      "stale_files": 15,
      "total_files": 120,
      "stale_pct": 0.125,
      "by_module": [...]
    },
    "coupling": [
      { "left": "src/a.rs", "right": "src/b.rs", "count": 12 }
    ]
  }
}
```

### Other Optional Sections

| Section | Preset | Description |
| :--- | :--- | :--- |
| `archetype` | `identity` | Project type detection (CLI, library, web app, etc.) |
| `topics` | `topics` | TF-IDF semantic analysis of paths |
| `entropy` | `security` | High-entropy file detection |
| `predictive_churn` | `git` | Trend analysis from commit history |
| `corporate_fingerprint` | `identity` | Author domain statistics |
| `license` | `security` | SPDX license detection |
| `assets` | `supply` | Non-code file inventory |
| `deps` | `supply` | Lockfile dependency counts |
| `imports` | `architecture` | Module dependency graph |
| `dup` | `deep` | Duplicate file detection |
| `fun` | `fun` | Novelty outputs (eco-label) |

---

## Schema Evolution

- **Additive changes** (new optional fields) do not increment `schema_version`.
- **Breaking changes** (renamed/removed fields, type changes) increment `schema_version`.
- Consumers should ignore unknown fields for forward compatibility.
- The `integrity.hash` field can be used to verify receipt contents.

### Forward Compatibility Policy

The JSON schema intentionally does **not** use `additionalProperties: false`. This means:

1. **New fields may appear** in any receipt at any time without a schema version bump
2. **Consumers must ignore unknown fields** rather than failing on them
3. **Field removal or renaming** is a breaking change and will bump `schema_version`

This policy allows tokmd to add observability signals, debugging info, or new metrics without breaking existing integrations. If you need strict validation, pin to a specific tokmd version.
