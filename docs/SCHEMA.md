# tokmd Receipt Schema

> **Note**: Core/analysis receipts are defined in [schema.json](schema.json).  
> Handoff manifests are defined in [handoff.schema.json](handoff.schema.json).

`tokmd` produces structured JSON outputs called "receipts". These schemas are stable and intended for machine consumption.

All JSON outputs share a common envelope structure.

---

## Schema Version History

tokmd uses **separate schema versions** for different receipt families. Each receipt type declares its own `schema_version` in the JSON output.

### Current Versions

| Receipt Family | Current Version | Constant | Applies To |
|----------------|-----------------|----------|------------|
| **Core** | 2 | `SCHEMA_VERSION` | `lang`, `module`, `export`, `diff`, `context`, `run` |
| **Analysis** | 4 | `ANALYSIS_SCHEMA_VERSION` | `analyze` |
| **Cockpit** | 3 | (local) | `cockpit` |
| **Envelope** | `"sensor.report.v1"` | `ENVELOPE_SCHEMA` | ecosystem envelope |
| **Baseline** | 1 | `BASELINE_VERSION` | complexity/determinism baselines |
| **Handoff** | 3 | `HANDOFF_SCHEMA_VERSION` | `handoff` manifest |

### Version Changelog

#### Core Receipts (`SCHEMA_VERSION`)

| Version | Changes |
|---------|---------|
| **2** | Added `bytes` and `tokens` fields to all rows; added `tokens`, `bytes`, `avg_lines` to totals; added `excluded_redacted` and `strip_prefix_redacted` flags |
| **1** | Initial release with `code`, `lines`, `files` metrics |

#### Analysis Receipts (`ANALYSIS_SCHEMA_VERSION`)

| Version | Changes |
|---------|---------|
| **4** | Added cognitive complexity, nesting depth, and function-level details to `ComplexityReport` |
| **3** | Added `complexity` section with cyclomatic complexity metrics |
| **2** | Initial analysis receipt structure with presets (receipt, health, risk, supply, architecture, topics, security, identity, git, deep, fun) |

#### Cockpit Receipts

| Version | Changes |
|---------|---------|
| **3** | Initial cockpit receipt for PR metrics with evidence gates, change surface, composition, code health, risk assessment, contracts, and review plan |

#### Envelope

| Version | Changes |
|---------|---------|
| **1** | Initial envelope specification for multi-sensor integration with findings, gates, and artifacts |

#### Baseline

| Version | Changes |
|---------|---------|
| **1** | Initial baseline format with complexity tracking (`ComplexityBaseline`) and determinism verification (`DeterminismBaseline`) |

#### Handoff Manifest

| Version | Changes |
|---------|---------|
| **3** | Added `output_dir`, formalized manifest schema (see `docs/handoff.schema.json`) |

### Code References

- **Core**: `crates/tokmd-types/src/lib.rs` - `pub const SCHEMA_VERSION: u32 = 2;`
- **Analysis**: `crates/tokmd-analysis-types/src/lib.rs` - `pub const ANALYSIS_SCHEMA_VERSION: u32 = 4;`
- **Cockpit**: `crates/tokmd/src/commands/cockpit.rs` - `const SCHEMA_VERSION: u32 = 3;`
- **Envelope**: `crates/tokmd-analysis-types/src/lib.rs` - `pub const ENVELOPE_SCHEMA: &str = "sensor.report.v1";`
- **Baseline**: `crates/tokmd-analysis-types/src/lib.rs` - `pub const BASELINE_VERSION: u32 = 1;`
- **Handoff**: `crates/tokmd-types/src/lib.rs` - `pub const HANDOFF_SCHEMA_VERSION: u32 = 3;`

---

## Common Fields

Every receipt includes:

| Field | Type | Description |
| :--- | :--- | :--- |
| `schema_version` | `integer` | The schema version for this receipt type. See [Schema Version History](#schema-version-history) for current values. |
| `generated_at_ms` | `integer` | Unix timestamp (milliseconds) when the scan ran. |
| `tool` | `object` | Information about the tool version. |
| `tool.name` | `string` | Always `"tokmd"`. |
| `tool.version` | `string` | The version of tokmd used (e.g., `"1.2.0"`). |
| `mode` | `string` | One of `"lang"`, `"module"`, `"export"`, `"analysis"`, or `"cockpit"`. |
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

**Schema version**: 4

Analysis receipts contain derived metrics and optional enrichments. All sections except `source`, `args`, and `derived` are optional based on the preset used.

### Envelope

```json
{
  "schema_version": 4,
  "generated_at_ms": 1706350000000,
  "tool": { "name": "tokmd", "version": "1.3.0" },
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

---

## 6. Ecosystem Envelope

The ecosystem envelope provides a standardized JSON format for multi-sensor integration. It allows tokmd to integrate with external orchestrators ("directors") that aggregate reports from multiple code quality sensors into a unified PR view.

**Schema**: `"sensor.report.v1"`

### Envelope Structure

```json
{
  "schema": "sensor.report.v1",
  "tool": {
    "name": "tokmd",
    "version": "1.5.0",
    "mode": "cockpit"
  },
  "generated_at": "2024-01-27T10:30:00Z",
  "verdict": "warn",
  "summary": "2 warnings: hotspot touched, low test coverage",
  "findings": [
    {
      "check_id": "risk",
      "code": "hotspot",
      "severity": "warn",
      "title": "Hotspot file touched",
      "message": "src/core/engine.rs is a high-churn file",
      "location": { "path": "src/core/engine.rs", "line": 42 }
    }
  ],
  "gates": {
    "status": "pass",
    "items": [
      { "id": "mutation", "status": "pass", "threshold": 0.8, "actual": 0.92 }
    ]
  },
  "artifacts": [
    { "type": "receipt", "path": "tokmd-cockpit.json" }
  ],
  "data": { /* tool-specific payload */ }
}
```

### Envelope Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `schema` | `string` | Schema identifier (e.g., `"sensor.report.v1"`). |
| `tool` | `object` | Tool identification (name, version, mode). |
| `generated_at` | `string` | ISO 8601 timestamp. |
| `verdict` | `string` | Overall result: `pass`, `fail`, `warn`, `skip`, or `pending`. |
| `summary` | `string` | Human-readable one-line summary. |
| `findings` | `array` | List of findings (may be empty). |
| `gates` | `object\|null` | Evidence gate status (optional). |
| `artifacts` | `array\|null` | Related artifact paths (optional). |
| `data` | `any\|null` | Tool-specific payload, opaque to director (optional). |

### Verdict Enum

| Value | Description |
| :--- | :--- |
| `pass` | All checks passed, no significant findings. |
| `fail` | Hard failure (evidence gate failed, policy violation). |
| `warn` | Soft warnings present, review recommended. |
| `skip` | Sensor skipped (missing inputs, not applicable). |
| `pending` | Awaiting external data (CI artifacts, etc.). |

Directors aggregate verdicts with precedence: `fail` > `pending` > `warn` > `pass` > `skip`.

### Finding Structure

Findings use `(check_id, code)` for identity. Combined with `tool.name`, this forms the triple `(tool, check_id, code)` (e.g., `("tokmd", "risk", "hotspot")`).

| Field | Type | Description |
| :--- | :--- | :--- |
| `check_id` | `string` | Check category (e.g., `"risk"`, `"contract"`). |
| `code` | `string` | Finding code within the category (e.g., `"hotspot"`). |
| `severity` | `string` | Severity: `error`, `warn`, or `info`. |
| `title` | `string` | Short title. |
| `message` | `string` | Detailed message. |
| `location` | `object\|null` | Source location (`path`, `line`, `column`). |
| `evidence` | `any\|null` | Additional evidence data. |
| `docs_url` | `string\|null` | Documentation URL for this finding type. |

### GatesEnvelope Structure

| Field | Type | Description |
| :--- | :--- | :--- |
| `status` | `string` | Overall gate status (Verdict enum). |
| `items` | `array` | Individual gate items. |

### GateItem Structure

| Field | Type | Description |
| :--- | :--- | :--- |
| `id` | `string` | Gate identifier (e.g., `mutation`, `diff_coverage`). |
| `status` | `string` | Gate status (Verdict enum). |
| `threshold` | `number\|null` | Threshold value. |
| `actual` | `number\|null` | Actual measured value. |
| `reason` | `string\|null` | Reason for status. |
| `source` | `string\|null` | Data source (e.g., `ci_artifact`, `computed`). |
| `artifact_path` | `string\|null` | Path to source artifact. |

---

## 7. Baseline Types

Baseline files support complexity tracking and build determinism verification for the ratchet system.

**Schema version**: 1

### ComplexityBaseline

Used to track complexity trends over time and enforce that metrics do not regress.

```json
{
  "baseline_version": 1,
  "generated_at": "2024-01-27T10:30:00Z",
  "commit": "abc123def456",
  "metrics": {
    "total_code_lines": 10000,
    "total_files": 120,
    "avg_cyclomatic": 3.5,
    "max_cyclomatic": 25,
    "avg_cognitive": 5.2,
    "max_cognitive": 42,
    "avg_nesting_depth": 2.1,
    "max_nesting_depth": 8,
    "function_count": 450,
    "avg_function_length": 15.3
  },
  "files": [
    {
      "path": "src/main.rs",
      "code_lines": 200,
      "cyclomatic": 12,
      "cognitive": 18,
      "max_nesting": 4,
      "function_count": 8,
      "content_hash": "abc123..."
    }
  ]
}
```

### BaselineMetrics Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `total_code_lines` | `integer` | Total lines of code. |
| `total_files` | `integer` | Total source files. |
| `avg_cyclomatic` | `number` | Average cyclomatic complexity. |
| `max_cyclomatic` | `integer` | Maximum cyclomatic complexity. |
| `avg_cognitive` | `number` | Average cognitive complexity. |
| `max_cognitive` | `integer` | Maximum cognitive complexity. |
| `avg_nesting_depth` | `number` | Average nesting depth. |
| `max_nesting_depth` | `integer` | Maximum nesting depth. |
| `function_count` | `integer` | Total functions analyzed. |
| `avg_function_length` | `number` | Average function length. |

### FileBaselineEntry Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `path` | `string` | Normalized file path. |
| `code_lines` | `integer` | Lines of code. |
| `cyclomatic` | `integer` | Cyclomatic complexity. |
| `cognitive` | `integer` | Cognitive complexity. |
| `max_nesting` | `integer` | Maximum nesting depth. |
| `function_count` | `integer` | Number of functions. |
| `content_hash` | `string\|null` | BLAKE3 hash for change detection. |

### DeterminismBaseline

Tracks build artifact hashes for reproducibility verification.

```json
{
  "baseline_version": 1,
  "generated_at": "2024-01-27T10:30:00Z",
  "build_hash": "abc123...",
  "source_hash": "def456...",
  "cargo_lock_hash": "789abc..."
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `baseline_version` | `integer` | Schema version (currently 1). |
| `generated_at` | `string` | ISO 8601 timestamp. |
| `build_hash` | `string` | Hash of the final build artifact. |
| `source_hash` | `string` | Hash of all source files combined. |
| `cargo_lock_hash` | `string\|null` | Hash of Cargo.lock (Rust projects). |

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

---

## 5. Cockpit Receipt (`mode: "cockpit"`)

Produced by `tokmd cockpit --format json`.

**Schema version**: 3

Cockpit receipts provide PR-focused metrics for code review automation, including change surface analysis, risk assessment, code health indicators, and evidence gates for quality assurance.

> **Note**: The cockpit receipt uses a different envelope structure than other receipts because it is specifically designed for PR/diff analysis rather than codebase scanning.

### Envelope

```json
{
  "schema_version": 3,
  "generated_at_ms": 1706350000000,
  "base_ref": "main",
  "head_ref": "feature/my-branch",
  "change_surface": { ... },
  "composition": { ... },
  "code_health": { ... },
  "risk": { ... },
  "contracts": { ... },
  "evidence": { ... },
  "review_plan": [ ... ]
}
```

### Cockpit Receipt Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `schema_version` | `integer` | The schema version (3 for cockpit receipts). |
| `generated_at_ms` | `integer` | Unix timestamp (milliseconds) when the analysis ran. |
| `base_ref` | `string` | The base git ref (branch/commit) for comparison. |
| `head_ref` | `string` | The head git ref being analyzed. |
| `change_surface` | `object` | Metrics about the scope of changes. |
| `composition` | `object` | Breakdown of file types in the changeset. |
| `code_health` | `object` | Health indicators for developer experience. |
| `risk` | `object` | Risk assessment for the changes. |
| `contracts` | `object` | Contract change indicators (API, CLI, schema). |
| `evidence` | `object` | Evidence gates with pass/fail status. |
| `review_plan` | `array` | Prioritized list of files to review. |

### Change Surface (`change_surface`)

Metrics quantifying the scope of changes between base and head refs.

```json
{
  "change_surface": {
    "commits": 5,
    "files_changed": 12,
    "insertions": 350,
    "deletions": 120,
    "net_lines": 230,
    "churn_velocity": 94.0,
    "change_concentration": 0.65
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `commits` | `integer` | Number of commits in the diff. |
| `files_changed` | `integer` | Number of files modified. |
| `insertions` | `integer` | Total lines added. |
| `deletions` | `integer` | Total lines removed. |
| `net_lines` | `integer` | Net line change (insertions - deletions). |
| `churn_velocity` | `float` | Average lines changed per commit. |
| `change_concentration` | `float` | Ratio of changes in top 20% of files (0.0-1.0). |

### Composition (`composition`)

Breakdown of changed files by category.

```json
{
  "composition": {
    "code_pct": 0.65,
    "test_pct": 0.20,
    "docs_pct": 0.10,
    "config_pct": 0.05,
    "test_ratio": 0.31
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `code_pct` | `float` | Percentage of changes in production code files. |
| `test_pct` | `float` | Percentage of changes in test files. |
| `docs_pct` | `float` | Percentage of changes in documentation files. |
| `config_pct` | `float` | Percentage of changes in configuration files. |
| `test_ratio` | `float` | Ratio of test files to code files changed. |

### Code Health (`code_health`)

Health indicators for developer experience.

```json
{
  "code_health": {
    "score": 85,
    "grade": "B",
    "large_files_touched": 2,
    "avg_file_size": 150,
    "complexity_indicator": "medium",
    "warnings": [
      {
        "path": "src/large_module.rs",
        "warning_type": "large_file",
        "message": "File has 650 lines, consider splitting"
      }
    ]
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `score` | `integer` | Overall health score (0-100). |
| `grade` | `string` | Health grade (A-F). |
| `large_files_touched` | `integer` | Number of large files (>500 lines) being changed. |
| `avg_file_size` | `integer` | Average file size in changed files (lines). |
| `complexity_indicator` | `string` | Complexity level: `"low"`, `"medium"`, `"high"`, or `"critical"`. |
| `warnings` | `array` | Array of health warnings for specific files. |

#### Health Warning Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `path` | `string` | File path. |
| `warning_type` | `string` | Type: `"large_file"`, `"high_churn"`, `"low_test_coverage"`, `"complex_change"`, or `"bus_factor"`. |
| `message` | `string` | Human-readable warning message. |

### Risk (`risk`)

Risk assessment for the changes.

```json
{
  "risk": {
    "hotspots_touched": ["src/core/engine.rs"],
    "bus_factor_warnings": ["crates/parser"],
    "level": "medium",
    "score": 45
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `hotspots_touched` | `array` | List of high-churn files being modified. |
| `bus_factor_warnings` | `array` | Modules with bus factor concerns (single maintainer). |
| `level` | `string` | Risk level: `"low"`, `"medium"`, `"high"`, or `"critical"`. |
| `score` | `integer` | Risk score (0-100). |

### Contracts (`contracts`)

Indicators of contract-level changes.

```json
{
  "contracts": {
    "api_changed": true,
    "cli_changed": false,
    "schema_changed": false,
    "breaking_indicators": 1
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `api_changed` | `boolean` | Whether API surface files were modified. |
| `cli_changed` | `boolean` | Whether CLI command files were modified. |
| `schema_changed` | `boolean` | Whether schema files were modified. |
| `breaking_indicators` | `integer` | Count of potential breaking change indicators. |

### Evidence (`evidence`)

Hard gates for quality assurance. Contains gate results with pass/fail status.

```json
{
  "evidence": {
    "overall_status": "pass",
    "mutation": { ... },
    "diff_coverage": { ... },
    "contracts": { ... },
    "supply_chain": { ... },
    "determinism": { ... }
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `overall_status` | `string` | Aggregate status of all gates (see GateStatus). |
| `mutation` | `object` | Mutation testing gate (always present). |
| `diff_coverage` | `object\|null` | Diff coverage gate (optional). |
| `contracts` | `object\|null` | Contract diff gate (optional). |
| `supply_chain` | `object\|null` | Supply chain gate (optional). |
| `determinism` | `object\|null` | Determinism gate (optional). |

#### GateStatus Enum

All gate status fields use one of these values:

| Value | Description |
| :--- | :--- |
| `"pass"` | Gate passed all checks. |
| `"fail"` | Gate failed one or more checks. |
| `"skipped"` | No relevant files changed; gate not applicable. |
| `"pending"` | Results not available and couldn't run locally. |

The `overall_status` is computed as follows:
- If any gate is `"fail"` → `"fail"`
- If all gates are `"pass"` → `"pass"`
- If any gate is `"pending"` (and none failed) → `"pending"`
- Otherwise (mix of pass and skipped) → `"pass"`

#### Gate Metadata (`GateMeta`)

All gates include common metadata fields (flattened into the gate object):

| Field | Type | Description |
| :--- | :--- | :--- |
| `status` | `string` | Gate status (see GateStatus). |
| `source` | `string` | Evidence source: `"ci_artifact"`, `"cached"`, or `"ran_local"`. |
| `commit_match` | `string` | Match quality: `"exact"`, `"partial"`, `"stale"`, or `"unknown"`. |
| `scope` | `object` | Scope coverage information. |
| `evidence_commit` | `string\|null` | SHA this evidence was generated for. |
| `evidence_generated_at_ms` | `integer\|null` | Timestamp when evidence was generated. |

#### Scope Coverage (`scope`)

| Field | Type | Description |
| :--- | :--- | :--- |
| `relevant` | `array` | Files in scope for the gate. |
| `tested` | `array` | Files actually tested. |
| `ratio` | `float` | Coverage ratio (tested/relevant, 0.0-1.0). |
| `lines_relevant` | `integer\|null` | Lines in scope (for line-level gates). |
| `lines_tested` | `integer\|null` | Lines actually tested (for line-level gates). |

#### Mutation Gate (`mutation`)

```json
{
  "mutation": {
    "status": "pass",
    "source": "ci_artifact",
    "commit_match": "exact",
    "scope": { ... },
    "evidence_commit": "abc123",
    "evidence_generated_at_ms": 1706350000000,
    "survivors": [],
    "killed": 42,
    "timeout": 3,
    "unviable": 5
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `survivors` | `array` | Mutations that survived (escaped detection). |
| `killed` | `integer` | Number of mutations killed by tests. |
| `timeout` | `integer` | Number of mutations that caused timeouts. |
| `unviable` | `integer` | Number of unviable mutations. |

##### Mutation Survivor

| Field | Type | Description |
| :--- | :--- | :--- |
| `file` | `string` | File path containing the survivor. |
| `line` | `integer` | Line number of the mutation. |
| `mutation` | `string` | Description of the mutation. |

#### Diff Coverage Gate (`diff_coverage`)

```json
{
  "diff_coverage": {
    "status": "pass",
    "source": "ran_local",
    "commit_match": "exact",
    "scope": { ... },
    "lines_added": 100,
    "lines_covered": 85,
    "coverage_pct": 0.85,
    "uncovered_hunks": [
      { "file": "src/new.rs", "start_line": 45, "end_line": 52 }
    ]
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `lines_added` | `integer` | Total lines added in the diff. |
| `lines_covered` | `integer` | Lines covered by tests. |
| `coverage_pct` | `float` | Coverage percentage (0.0-1.0). |
| `uncovered_hunks` | `array` | Hunks of uncovered code. |

#### Contract Diff Gate (`contracts`)

A compound gate checking API semver, CLI, and schema compatibility.

```json
{
  "contracts": {
    "status": "pending",
    "source": "ran_local",
    "commit_match": "unknown",
    "scope": { ... },
    "semver": {
      "status": "pending",
      "breaking_changes": []
    },
    "cli": {
      "status": "pending",
      "diff_summary": null
    },
    "schema": {
      "status": "pending",
      "diff_summary": null
    },
    "failures": 0
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `semver` | `object\|null` | Semver compatibility sub-gate. |
| `cli` | `object\|null` | CLI compatibility sub-gate. |
| `schema` | `object\|null` | Schema compatibility sub-gate. |
| `failures` | `integer` | Count of failed sub-gates. |

#### Supply Chain Gate (`supply_chain`)

```json
{
  "supply_chain": {
    "status": "pass",
    "source": "ran_local",
    "commit_match": "exact",
    "scope": { ... },
    "vulnerabilities": [],
    "denied": [],
    "advisory_db_version": "2024-01-15"
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `vulnerabilities` | `array` | Detected vulnerabilities from cargo-audit. |
| `denied` | `array` | Denied packages from cargo-deny. |
| `advisory_db_version` | `string\|null` | Version of the advisory database used. |

##### Vulnerability

| Field | Type | Description |
| :--- | :--- | :--- |
| `id` | `string` | Advisory ID (e.g., RUSTSEC-2024-0001). |
| `package` | `string` | Affected package name. |
| `severity` | `string` | Severity level. |
| `title` | `string` | Advisory title. |

#### Determinism Gate (`determinism`)

```json
{
  "determinism": {
    "status": "pass",
    "source": "ran_local",
    "commit_match": "exact",
    "scope": { ... },
    "expected_hash": "abc123...",
    "actual_hash": "abc123...",
    "algo": "blake3",
    "differences": []
  }
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `expected_hash` | `string\|null` | Expected hash from baseline. |
| `actual_hash` | `string\|null` | Actual computed hash. |
| `algo` | `string` | Hash algorithm used (e.g., `"blake3"`). |
| `differences` | `array` | List of files that differ from baseline. |

### Review Plan (`review_plan`)

Prioritized list of files requiring review.

```json
{
  "review_plan": [
    {
      "path": "src/core/engine.rs",
      "reason": "High-churn hotspot",
      "priority": 1,
      "complexity": 4,
      "lines_changed": 85
    },
    {
      "path": "src/api/handlers.rs",
      "reason": "API surface change",
      "priority": 2,
      "complexity": 3,
      "lines_changed": 42
    }
  ]
}
```

| Field | Type | Description |
| :--- | :--- | :--- |
| `path` | `string` | File path. |
| `reason` | `string` | Why this file needs review. |
| `priority` | `integer` | Review priority (lower = higher priority). |
| `complexity` | `integer\|null` | Estimated review complexity (1-5). |
| `lines_changed` | `integer\|null` | Lines changed in this file. |

### Complete Cockpit Receipt Example

```json
{
  "schema_version": 3,
  "generated_at_ms": 1706350000000,
  "base_ref": "main",
  "head_ref": "feature/add-cockpit",
  "change_surface": {
    "commits": 3,
    "files_changed": 8,
    "insertions": 520,
    "deletions": 45,
    "net_lines": 475,
    "churn_velocity": 188.3,
    "change_concentration": 0.72
  },
  "composition": {
    "code_pct": 0.70,
    "test_pct": 0.15,
    "docs_pct": 0.10,
    "config_pct": 0.05,
    "test_ratio": 0.21
  },
  "code_health": {
    "score": 78,
    "grade": "C",
    "large_files_touched": 1,
    "avg_file_size": 185,
    "complexity_indicator": "medium",
    "warnings": [
      {
        "path": "crates/tokmd/src/commands/cockpit.rs",
        "warning_type": "large_file",
        "message": "File has 850 lines, consider splitting"
      }
    ]
  },
  "risk": {
    "hotspots_touched": [],
    "bus_factor_warnings": [],
    "level": "low",
    "score": 25
  },
  "contracts": {
    "api_changed": false,
    "cli_changed": true,
    "schema_changed": true,
    "breaking_indicators": 0
  },
  "evidence": {
    "overall_status": "pending",
    "mutation": {
      "status": "pending",
      "source": "ran_local",
      "commit_match": "unknown",
      "scope": {
        "relevant": ["crates/tokmd/src/commands/cockpit.rs"],
        "tested": [],
        "ratio": 0.0,
        "lines_relevant": null,
        "lines_tested": null
      },
      "evidence_commit": null,
      "evidence_generated_at_ms": null,
      "survivors": [],
      "killed": 0,
      "timeout": 0,
      "unviable": 0
    },
    "diff_coverage": null,
    "contracts": {
      "status": "pending",
      "source": "ran_local",
      "commit_match": "unknown",
      "scope": {
        "relevant": ["crates/tokmd/src/commands/cockpit.rs"],
        "tested": ["crates/tokmd/src/commands/cockpit.rs"],
        "ratio": 1.0,
        "lines_relevant": null,
        "lines_tested": null
      },
      "evidence_commit": null,
      "evidence_generated_at_ms": null,
      "semver": null,
      "cli": {
        "status": "pending",
        "diff_summary": null
      },
      "schema": {
        "status": "pending",
        "diff_summary": null
      },
      "failures": 0
    },
    "supply_chain": null,
    "determinism": null
  },
  "review_plan": [
    {
      "path": "crates/tokmd/src/commands/cockpit.rs",
      "reason": "New command implementation",
      "priority": 1,
      "complexity": 4,
      "lines_changed": 450
    },
    {
      "path": "docs/SCHEMA.md",
      "reason": "Documentation update",
      "priority": 3,
      "complexity": 2,
      "lines_changed": 200
    }
  ]
}
```
