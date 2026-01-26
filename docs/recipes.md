# tokmd Recipes

Examples of how to use `tokmd` in real-world scenarios.

## 1. Feeding a Codebase to an LLM

When asking an LLM to refactor or understand a large repo, you need a high-signal, low-noise representation of the file structure.

**Goal**: Get a compact list of files, sorted by size, without sensitive paths.

```bash
# 1. Export as JSONL (streaming friendly)
# 2. Redact paths (replace sensitive names with hashes)
# 3. Filter out tiny files (noise)
# 4. Limit to top 500 files to fit context
tokmd export \
  --format jsonl \
  --redact paths \
  --min-code 10 \
  --max-rows 500 \
  > repo_context.jsonl
```

**Why**:
- JSONL is easily parsed by Python scripts or LLM context loaders.
- Redaction prevents leaking internal project names.
- `min-code` removes config files and empty boilerplate.

## 2. Tracking Repo Growth Over Time

Use `tokmd` in CI to generate a "receipt" of the repo size for every commit or release.

**Goal**: Spot sudden bloat in specific modules.

```bash
# Generate a module report in JSON format
tokmd module --format json > tokmd_report.json
```

**Analysis**:
Compare `total.code` or `rows[].code` between two reports.

```json
// tokmd_report.json excerpt
{
  "mode": "module",
  "total": { "code": 15430, ... },
  "rows": [
    { "module": "crates/core", "code": 5000 },
    { "module": "crates/cli", "code": 2000 }
  ]
}
```

## 3. Auditing Vendor Dependencies

If you vendor dependencies (e.g., in `vendor/` or `node_modules/` that are checked in), you want to know how much weight they add.

**Goal**: See split between your code and vendor code.

```bash
# Assuming 'vendor' is a top-level directory
tokmd module --module-roots vendor,src --children collapse
```

Output:
| Module | Code | ... |
| :--- | ---: | --- |
| vendor | 150,000 | ... |
| src | 25,000 | ... |

## 4. Finding "Heavy" Files

Identify files that might need refactoring because they are too large.

```bash
# List all files, sorted by code lines (default sort)
tokmd export --format csv --out files.csv
```

Open `files.csv` in Excel/Google Sheets and look at the top rows.

## 5. Quick Repo Summary for PR Descriptions

Paste a summary of the languages used in your PR description.

```bash
tokmd --format md --top 5
```

## 6. Configuring Ignores

By default, `tokmd` respects `.gitignore`. Sometimes you want to ignore *more* (like tests or vendored code) without changing git behavior.

**Option A: Command Line**
```bash
# Ignore the 'test' directory and all CSV files
tokmd --exclude "tests/" --exclude "*.csv"
```

**Option B: .tokeignore file**
Create a `.tokeignore` file in your root. It uses standard gitignore syntax.

```gitignore
# .tokeignore
tests/
fixtures/
*.lock
```

This file is specific to `tokmd` (and `tokei`) and won't affect git.

## 7. CI Gate: Fail if Files are Too Large

Enforce a "no monolithic files" policy in CI.

**Goal**: Fail the build if any source file exceeds 2000 lines.

```bash
# Export all files > 2000 lines
# If output is empty, grep fails (exit 1) -> we want the opposite
# Better: count lines of output

COUNT=$(tokmd export --min-code 2000 --format csv | tail -n +2 | wc -l)

if [ "$COUNT" -gt 0 ]; then
  echo "Error: Found $COUNT files larger than 2000 lines."
  tokmd export --min-code 2000 --format csv
  exit 1
fi
```
