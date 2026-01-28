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

## 2. Quick Health Check with Analysis

Get a comprehensive overview of your codebase's structure and quality signals.

```bash
# Generate a health report with TODO density
tokmd analyze --preset health --format md

# Include git metrics for risk assessment
tokmd analyze --preset risk --format md
```

**What you get**:
- Doc density (how much is documented?)
- Test density (test-to-production ratio)
- TODO/FIXME counts and density per KLOC
- Git hotspots (frequently changed files)
- Freshness (stale code detection)

## 3. Context Window Planning

Before dumping files into an LLM, check if they'll fit.

```bash
# Check against Claude's 200k context window
tokmd analyze --preset receipt --window 200000 --format md
```

The output shows:
- Total estimated tokens
- Percentage of context window used
- Whether the codebase fits

## 4. Tracking Repo Growth Over Time

Use `tokmd` in CI to generate a "receipt" of the repo size for every commit or release.

**Goal**: Spot sudden bloat in specific modules.

```bash
# Generate a module report in JSON format
tokmd module --format json > tokmd_report.json

# Or use run to save all artifacts
tokmd run --output-dir .runs/$(date +%Y%m%d)
```

**Analysis**:
Compare `total.code` or `rows[].code` between two reports.

```bash
# Diff two runs
tokmd diff .runs/20260120 .runs/20260127
```

## 5. Auditing Vendor Dependencies

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

## 6. Finding "Heavy" Files

Identify files that might need refactoring because they are too large.

```bash
# Quick view: top 10 largest files
tokmd export --format csv --max-rows 10

# Detailed analysis with distribution stats
tokmd analyze --preset receipt --format md
```

The analysis shows:
- File size distribution (p90, p99, Gini coefficient)
- Top offenders by lines, tokens, and bytes
- Histogram of file sizes (tiny/small/medium/large/huge)

## 7. Generating Badges for README

Add live metrics to your project README.

```bash
# Lines of code badge
tokmd badge --metric lines --out badges/lines.svg

# Token count badge
tokmd badge --metric tokens --out badges/tokens.svg

# Documentation percentage badge
tokmd badge --metric doc --out badges/doc.svg
```

Then embed in your README:
```markdown
![Lines](badges/lines.svg) ![Tokens](badges/tokens.svg) ![Docs](badges/doc.svg)
```

## 8. Effort Estimation (COCOMO)

Get a rough effort estimate for the codebase.

```bash
tokmd analyze --preset receipt --format json | jq '.derived.cocomo'
```

Returns:
- KLOC (thousands of lines of code)
- Effort in person-months
- Duration in months
- Suggested team size

## 9. CI Gate: Fail if Files are Too Large

Enforce a "no monolithic files" policy in CI.

**Goal**: Fail the build if any source file exceeds 2000 lines.

```bash
COUNT=$(tokmd export --min-code 2000 --format csv | tail -n +2 | wc -l)

if [ "$COUNT" -gt 0 ]; then
  echo "Error: Found $COUNT files larger than 2000 lines."
  tokmd export --min-code 2000 --format csv
  exit 1
fi
```

## 10. Configuring Ignores

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

## 11. Git Risk Analysis

Identify risky areas of the codebase based on git history.

```bash
# Full risk analysis
tokmd analyze --preset risk --format md

# Limit git history scan for large repos
tokmd analyze --preset risk --max-commits 1000 --max-commit-files 100
```

**What you get**:
- Hotspots: Files with high churn AND high complexity
- Bus factor: Modules with single-author risk
- Coupling: Files that change together
- Freshness: Stale modules that may need attention

## 12. Architecture Visualization

Generate a module dependency graph.

```bash
# Mermaid diagram for docs
tokmd analyze --preset architecture --format mermaid > deps.mmd

# JSON for custom processing
tokmd analyze --preset architecture --format json
```

## 13. License Audit

Check for license files and SPDX identifiers.

```bash
tokmd analyze --preset security --format json | jq '.license'
```

## 14. Quick PR Summary

Paste a summary of the languages used in your PR description.

```bash
tokmd --format md --top 5
```

## 15. Full Deep Analysis

When you need everything for a comprehensive review.

```bash
# All metrics except fun outputs
tokmd analyze --preset deep --format json --output-dir analysis/

# Include fun outputs (eco-label, etc.)
tokmd analyze --preset fun --format json
```
