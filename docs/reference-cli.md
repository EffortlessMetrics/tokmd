# tokmd CLI Reference

This document details the command-line interface for `tokmd`.

## Global Arguments

These arguments apply to all subcommands (`lang`, `module`, `export`, `init`).

| Flag | Description |
| :--- | :--- |
| `-p, --paths <PATHS>...` | Files or directories to scan. Defaults to current directory (`.`). |
| `-e, --exclude <PATTERN>` | Glob pattern to exclude (e.g., `*.lock`, `vendor/`). Can be used multiple times. |
| `--config <MODE>` | Config file strategy: `auto` (default, reads `tokei.toml`/`.tokeirc`) or `none`. |
| `--hidden` | Count hidden files and directories (start with `.`). |
| `--no-ignore` | Disable all ignore files (`.gitignore`, `.ignore`, `.tokeignore`). |
| `--no-ignore-parent` | Do not traverse parent directories for ignore files. |
| `--no-ignore-dot` | Do not read `.ignore` or `.tokeignore` files. |
| `--no-ignore-vcs` | Do not read `.gitignore` files. |
| `--doc-comments` | Treat doc strings (e.g., `///`) as comments instead of code. |
| `-v, --verbose` | Enable verbose logging. |

---

## Commands

### `tokmd` (Default / `lang`)

Generates a summary of code statistics grouped by **Language**.

**Usage**: `tokmd [FLAGS] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `-f, --format <FMT>` | Output format: `md` (Markdown table), `tsv`, `json`. | `md` |
| `-t, --top <N>` | Only show the top N languages (by lines of code). Others grouped as "Other". | `0` (all) |
| `--children <MODE>` | How to handle embedded languages (e.g., JS inside HTML). | `collapse` |
| | `collapse`: Embedded code counts toward the parent file's language. | |
| | `separate`: Embedded code is counted separately under its own language. | |

**Example**:
```bash
# Top 5 languages, JSON output, including hidden files
tokmd --format json --top 5 --hidden
```

### `tokmd module`

Generates a summary grouped by **Module** (directory structure).

**Usage**: `tokmd module [FLAGS] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `-f, --format <FMT>` | Output format: `md` (Markdown table), `tsv`, `json`. | `md` |
| `-t, --top <N>` | Only show the top N modules. | `0` (all) |
| `--children <MODE>` | Handling of embedded languages. | `collapse` |
| `--module-roots <DIRS>` | Comma-separated list of root directories to group by (e.g., `src,tests`). | `.` |
| `--module-depth <N>` | How deep to group modules. | `1` |

**Example**:
```bash
# Analyze 'crates' and 'packages' directories, 2 levels deep
tokmd module --module-roots crates,packages --module-depth 2
```

### `tokmd export`

Generates a row-level inventory of files. Best for machine processing.

**Usage**: `tokmd export [FLAGS] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `-f, --format <FMT>` | Output format: `jsonl`, `csv`. | `jsonl` |
| `--min-code <N>` | Exclude files with fewer than N lines of code. | `0` |
| `--max-rows <N>` | Limit output to the top N largest files. | `0` (unlimited) |
| `--children <MODE>` | Handling of embedded languages. | `separate` |
| `--redact <MODE>` | Redaction strategy for paths/names. | `none` |
| | `none`: Show full paths. | |
| | `paths`: Hash file paths (preserve extension). | |
| | `all`: Hash paths and module names. | |
| `--strip-prefix <PATH>` | Remove a prefix from file paths in the output. | `None` |

**Example**:
```bash
# Export top 100 files > 10 LOC, redacted, as JSONL
tokmd export --min-code 10 --max-rows 100 --redact paths
```

### `tokmd init`

Creates a default `.tokeignore` file in the current directory.

**Usage**: `tokmd init`
