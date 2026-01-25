# tokmd Receipt Schema (v1)

> **Note**: A machine-readable JSON Schema definition is available at [schema.json](schema.json).

`tokmd` produces structured JSON outputs called "receipts". These schemas are stable and intended for machine consumption.

All JSON outputs share a common envelope structure.

## Common Fields

Every receipt includes:

| Field | Type | Description |
| :--- | :--- | :--- |
| `schema_version` | `integer` | The schema version (currently `1`). Incremented on breaking changes. |
| `generated_at_ms` | `integer` | Unix timestamp (milliseconds) when the scan ran. |
| `tool` | `object` | Information about the tool version. |
| `tool.name` | `string` | Always `"tokmd"`. |
| `tool.version` | `string` | The version of tokmd used (e.g., `"0.2.0"`). |
| `mode` | `string` | One of `"lang"`, `"module"`, or `"export"`. |
| `scan` | `object` | The configuration used for the file scan (paths, excludes, etc.). |

---

## 1. Language Receipt (`mode: "lang"`)

Produced by `tokmd --format json`.

```json
{
  "schema_version": 1,
  "mode": "lang",
  "tool": { ... },
  "scan": { ... },
  "args": {
    "top": 0,
    "with_files": false,
    "children": "collapse"
  },
  "total": {
    "code": 1200,
    "lines": 1500,
    "files": 15,
    "avg_lines": 100
  },
  "rows": [
    {
      "lang": "Rust",
      "code": 1000,
      "lines": 1200,
      "files": 10,
      "avg_lines": 120
    },
    {
      "lang": "Markdown",
      "code": 200,
      "lines": 300,
      "files": 5,
      "avg_lines": 60
    }
  ]
}
```

## 2. Module Receipt (`mode: "module"`)

Produced by `tokmd module --format json`.

```json
{
  "schema_version": 1,
  "mode": "module",
  "tool": { ... },
  "scan": { ... },
  "args": {
    "module_roots": ["crates", "packages"],
    "module_depth": 2,
    "children": "separate",
    "top": 0
  },
  "total": { ... },
  "rows": [
    {
      "module": "crates/cli",
      "code": 500,
      "lines": 600,
      "files": 4,
      "avg_lines": 150
    }
  ]
}
```

## 3. Export Data (`mode: "export"`)

Produced by `tokmd export --format jsonl` (default).

Export output consists of a **Meta Record** (first line) followed by **Data Rows**.

### Meta Record (Line 1)

```json
{
  "type": "meta",
  "schema_version": 1,
  "mode": "export",
  "tool": { ... },
  "scan": { ... },
  "args": {
    "format": "jsonl",
    "min_code": 0,
    "max_rows": 0,
    "redact": "none",
    "strip_prefix": null,
    ...
  }
}
```

### Data Row (Lines 2+)

```json
{
  "type": "row",
  "path": "src/main.rs",
  "module": "src",
  "lang": "Rust",
  "kind": "parent",  // or "child" for embedded languages
  "code": 120,
  "comments": 10,
  "blanks": 5,
  "lines": 135
}
```

### Redaction

If `--redact paths` or `--redact all` is used:
- `path`: Replaced with a hash (preserving extension).
- `module`: Replaced with a hash (if `all`).

```json
{
  "type": "row",
  "path": "a1b2c3d4e5f6.rs",
  ...
}
```
