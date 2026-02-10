# tokmd Recipes

Examples of how to use `tokmd` in real-world scenarios.

## 1. Packing Code into an LLM Context Window

When you need to feed actual code to an LLM (not just metadata), use the `context` command to intelligently select files within a token budget.

**Goal**: Get the most valuable code files that fit in your context window.

```bash
# Pack files into 128k tokens (Claude's context window)
tokmd context --budget 128k --output bundle --out context.txt

# Spread coverage across modules instead of just largest files
tokmd context --budget 128k --strategy spread --output bundle --out context.txt

# Strip blank lines for maximum density
tokmd context --budget 128k --output bundle --compress --out context.txt

# Use module roots for better organization
tokmd context --budget 128k --module-roots crates,src --strategy spread --out context.txt
```

**Why**:
- `greedy` strategy maximizes code coverage by taking largest files first.
- `spread` strategy ensures you get representation from all modules.
- `--compress` strips blank lines for more content per token.
- `--module-roots` groups files by directory structure for better spread coverage.

> **Tip**: Use `--rank-by churn` or `--rank-by hotspot` to prioritize recently-changed or high-complexity files (requires git history).

## 2. Getting a File Inventory for LLM Context Planning

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

## 3. Quick Health Check with Analysis

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

## 4. Context Window Planning

Before dumping files into an LLM, check if they'll fit.

```bash
# Check against Claude's 200k context window
tokmd analyze --preset receipt --window 200000 --format md
```

The output shows:
- Total estimated tokens
- Percentage of context window used
- Whether the codebase fits

## 5. Tracking Repo Growth Over Time

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

## 6. Auditing Vendor Dependencies

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

## 7. Finding "Heavy" Files

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

## 8. Generating Badges for README

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

## 9. Effort Estimation (COCOMO)

Get a rough effort estimate for the codebase.

```bash
tokmd analyze --preset receipt --format json | jq '.derived.cocomo'
```

Returns:
- KLOC (thousands of lines of code)
- Effort in person-months
- Duration in months
- Suggested team size

## 10. CI Gate: Policy-Based Quality Gates

Use `tokmd gate` to enforce code quality policies in CI with JSON pointer rules.

**Goal**: Enforce multiple quality standards in a single command.

```bash
# Gate using inline rules from tokmd.toml
tokmd gate

# Gate with explicit policy file
tokmd gate --policy policy.toml

# Compute analysis then gate
tokmd gate --preset health

# JSON output for CI parsing
tokmd gate --format json
```

**Example policy.toml**:
```toml
[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "Codebase exceeds 500k token budget"

[[rules]]
name = "min_docs"
pointer = "/derived/doc_density/total/ratio"
op = "gte"
value = 0.1
level = "warn"
message = "Documentation below 10%"
```

**Exit codes**:
- `0`: All rules passed
- `1`: One or more rules failed
- `2`: Policy error (invalid file, parse error)

## 10a. CI Gate: Simple File Size Check

For simpler checks without a policy file.

**Goal**: Fail the build if any source file exceeds 2000 lines.

```bash
COUNT=$(tokmd export --min-code 2000 --format csv | tail -n +2 | wc -l)

if [ "$COUNT" -gt 0 ]; then
  echo "Error: Found $COUNT files larger than 2000 lines."
  tokmd export --min-code 2000 --format csv
  exit 1
fi
```

## 10b. CI Gate: Ratchet Rules (Gradual Improvement)

Ensure that code metrics (like complexity) do not get worse over time.

**Goal**: Fail CI if average complexity increases compared to the `main` branch baseline.

1. **Generate baseline on main**:
   ```bash
   tokmd baseline --output baseline.json
   ```

2. **Configure ratchet in `tokmd.toml`**:
   ```toml
   [[gate.ratchet]]
   name = "complexity_ratchet"
   metric = "avg_cyclomatic"
   max_increase_pct = 0.0  # Strict no-regression
   level = "error"
   ```

3. **Run gate in PR**:
   ```bash
   tokmd gate --baseline baseline.json
   ```

**How it works**:
- The gate compares current metrics against `baseline.json`.
- If `avg_cyclomatic` in the PR is higher than the baseline, the gate fails.
- `max_increase_pct` allows a small buffer (e.g., 5.0 = 5% increase allowed) to prevent flaky failures.

## 11. Configuring Ignores

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

## 12. Git Risk Analysis

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

## 13. Architecture Visualization

Generate a module dependency graph.

```bash
# Mermaid diagram for docs
tokmd analyze --preset architecture --format mermaid > deps.mmd

# JSON for custom processing
tokmd analyze --preset architecture --format json
```

## 14. License Audit

Check for license files and SPDX identifiers.

```bash
tokmd analyze --preset security --format json | jq '.license'
```

## 14a. Generate CycloneDX SBOM

Export your codebase inventory as a CycloneDX Software Bill of Materials.

```bash
# Generate CycloneDX SBOM to file
tokmd export --format cyclonedx > bom.json

# Or write directly to file
tokmd export --format cyclonedx --out bom.json

# Combine with filtering
tokmd export --format cyclonedx --min-code 10 --max-rows 500 > bom.json
```

The output follows CycloneDX 1.6 specification and includes:
- `bomFormat`: "CycloneDX"
- `specVersion`: "1.6"
- `metadata`: Tool information and timestamp
- `components`: List of source files with type, name, and version

## 15. Quick PR Summary

Paste a summary of the languages used in your PR description.

```bash
tokmd --format md --top 5
```

## 15a. Generating LLM Tool Definitions

Export tokmd's CLI schema for AI agent integration.

**Goal**: Enable LLMs to programmatically invoke tokmd commands.

```bash
# OpenAI function calling format
tokmd tools --format openai --pretty > tools.json

# Anthropic tool use format
tokmd tools --format anthropic --pretty > tools.json

# JSON Schema for documentation
tokmd tools --format jsonschema --pretty > schema.json
```

**Use case**: Feed the schema to an AI agent so it can analyze repositories autonomously.

## 15b. PR Cockpit Metrics

Generate comprehensive PR metrics for code review automation with evidence gates.

**Goal**: Automate PR review with structured metrics and quality gates.

```bash
# Generate JSON metrics for CI parsing
tokmd cockpit

# Markdown summary for PR description
tokmd cockpit --format md

# Compare specific refs
tokmd cockpit --base origin/main --head feature-branch --format md

# Generate sections for PR template filling
tokmd cockpit --format sections --output pr-metrics.txt
```

**What you get**:
- **Change surface**: Files added/modified/deleted, lines added/removed
- **Composition**: Production vs test vs config breakdown
- **Code health**: Complexity, doc coverage, test coverage
- **Risk assessment**: Hotspots, coupling, freshness
- **Evidence gates**: Mutation testing, diff coverage, contracts, supply chain
- **Review plan**: Prioritized list of files to review

**GitHub Actions integration**:
```yaml
name: PR Cockpit
on:
  pull_request:

jobs:
  cockpit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install tokmd
        run: cargo install tokmd

      - name: Generate cockpit metrics
        run: |
          tokmd cockpit --base origin/${{ github.base_ref }} --head HEAD --format md > cockpit.md

      - name: Post PR comment
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const body = fs.readFileSync('cockpit.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });
```

## 16. Troubleshooting Ignored Files

When files unexpectedly appear or disappear from scans, use `check-ignore` to debug.

**Goal**: Understand why a file is being ignored.

```bash
# Check if a specific file is ignored
tokmd check-ignore target/debug/myapp

# Verbose output showing the exact rule that matched
tokmd check-ignore -v node_modules/lodash/index.js

# Check multiple files at once
tokmd check-ignore src/main.rs vendor/lib.js target/release/bin
```

**Exit codes**:
- Exit code `0` means the file IS ignored (and shows why)
- Exit code `1` means the file is NOT ignored

**What it checks**:
- `.gitignore` patterns (via `git check-ignore`)
- `.tokeignore` patterns
- `--exclude` command-line patterns

> **Note**: Tracked files are not considered ignored by gitignore rules. If a file is already tracked by git, `.gitignore` patterns do not apply to it—you need to `git rm --cached` the file first.

## 17. Full Deep Analysis

When you need everything for a comprehensive review.

```bash
# All metrics except fun outputs
tokmd analyze --preset deep --format json --output-dir analysis/

# Include fun outputs (eco-label, etc.)
tokmd analyze --preset fun --format json
```

---

## CI/CD Integration

### 18. GitHub Actions Integration

Use `tokmd` in GitHub Actions for automated code metrics and PR checks.

**Badge updates on push**:
```yaml
name: Update Badges
on:
  push:
    branches: [main]

jobs:
  badges:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install tokmd
        run: cargo install tokmd

      - name: Generate badges
        run: |
          mkdir -p badges
          tokmd badge --metric lines --out badges/lines.svg
          tokmd badge --metric tokens --out badges/tokens.svg
          tokmd badge --metric doc --out badges/doc.svg

      - name: Commit badges
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add badges/
          git diff --staged --quiet || git commit -m "Update code metrics badges"
          git push
```

**PR size check**:
```yaml
name: PR Size Check
on:
  pull_request:

jobs:
  size-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Need history for diff

      - name: Install tokmd
        run: cargo install tokmd

      - name: Check PR size
        run: |
          # Get diff between base and head
          DIFF=$(tokmd diff origin/${{ github.base_ref }} HEAD --format json)

          # Extract added lines
          ADDED=$(echo "$DIFF" | jq '.delta.code // 0')

          if [ "$ADDED" -gt 1000 ]; then
            echo "::warning::Large PR: $ADDED lines added"
          fi

          echo "## Code Metrics Diff" >> $GITHUB_STEP_SUMMARY
          tokmd diff origin/${{ github.base_ref }} HEAD --format md >> $GITHUB_STEP_SUMMARY
```

**Store artifacts for historical tracking**:
```yaml
name: Code Metrics
on:
  push:
    branches: [main]

jobs:
  metrics:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install tokmd
        run: cargo install tokmd

      - name: Generate metrics
        run: |
          tokmd run --output-dir .runs/$(git rev-parse --short HEAD)

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: tokmd-metrics-${{ github.sha }}
          path: .runs/
          retention-days: 90
```

### 19. GitLab CI Integration

**Basic metrics pipeline**:
```yaml
stages:
  - analyze

code-metrics:
  stage: analyze
  image: rust:latest
  before_script:
    - cargo install tokmd
  script:
    - tokmd run --output-dir metrics/
    - tokmd analyze --preset health --format md > metrics/report.md
  artifacts:
    paths:
      - metrics/
    expire_in: 30 days
  only:
    - main
    - merge_requests
```

**Merge request comment with metrics**:
```yaml
mr-metrics:
  stage: analyze
  image: rust:latest
  before_script:
    - cargo install tokmd
  script:
    - |
      # Generate diff report
      tokmd diff origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME HEAD --format md > diff.md

      # Post as MR comment (requires CI_JOB_TOKEN with api scope)
      curl --request POST \
        --header "PRIVATE-TOKEN: $CI_JOB_TOKEN" \
        --form "body=$(cat diff.md)" \
        "$CI_API_V4_URL/projects/$CI_PROJECT_ID/merge_requests/$CI_MERGE_REQUEST_IID/notes"
  only:
    - merge_requests
```

### 20. Pre-commit Hook for Large File Warnings

Warn developers before committing large files.

**Setup** (add to `.pre-commit-config.yaml`):
```yaml
repos:
  - repo: local
    hooks:
      - id: check-file-size
        name: Check for large files
        entry: bash -c 'tokmd export --min-code 2000 --format csv | tail -n +2 | grep -q . && echo "Warning: Files over 2000 lines detected" && tokmd export --min-code 2000 --format csv || true'
        language: system
        pass_filenames: false
```

**Manual git hook** (save as `.git/hooks/pre-commit`):
```bash
#!/bin/bash

# Check for files exceeding line threshold
THRESHOLD=2000
LARGE_FILES=$(tokmd export --min-code $THRESHOLD --format csv 2>/dev/null | tail -n +2)

if [ -n "$LARGE_FILES" ]; then
  echo "Warning: The following files exceed $THRESHOLD lines:"
  echo "$LARGE_FILES" | cut -d',' -f1
  echo ""
  echo "Consider refactoring before committing."
  # To make this a hard fail, uncomment:
  # exit 1
fi

exit 0
```

### 21. LLM Handoff Bundles

Create optimized code bundles for handing off to AI assistants.

**Goal**: Package your codebase with intelligence metadata for effective LLM collaboration.

**Basic handoff**:
```bash
# Create a handoff bundle with default settings (risk preset)
tokmd handoff
```

This creates a `.handoff/` directory with `manifest.json`, `map.jsonl`, `intelligence.json`, and `code.txt`.

**Different intelligence presets**:
```bash
# Minimal: just tree and map (fastest)
tokmd handoff --preset minimal

# Standard: add complexity and derived metrics
tokmd handoff --preset standard

# Risk: add hotspots and coupling analysis (default)
tokmd handoff --preset risk

# Deep: everything for thorough review
tokmd handoff --preset deep
```

**Customizing token budget and strategy**:
```bash
# Larger budget for models with bigger context windows
tokmd handoff --budget 200k

# Spread strategy for broader coverage
tokmd handoff --budget 128k --strategy spread

# Prioritize recently-changed files
tokmd handoff --rank-by churn
```

**Using handoff output with AI assistants**:
```bash
# Generate the bundle
tokmd handoff --preset risk --budget 128k

# The manifest tells the AI what's available
cat .handoff/manifest.json | jq '.artifacts'

# Feed the code bundle to your LLM
cat .handoff/code.txt | pbcopy  # macOS
```

**CI integration for automated handoffs**:
```yaml
name: Generate Handoff
on:
  pull_request:

jobs:
  handoff:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install tokmd
        run: cargo install tokmd

      - name: Generate handoff bundle
        run: tokmd handoff --preset risk --budget 128k --out-dir .handoff

      - name: Upload handoff artifact
        uses: actions/upload-artifact@v4
        with:
          name: handoff-${{ github.sha }}
          path: .handoff/
          retention-days: 30
```

### 22. Baseline Tracking Workflow

Track code metrics over time with automated baseline management.

**Initial baseline setup**:
```bash
# Create initial baseline
mkdir -p .tokmd/baselines
tokmd run --output-dir .tokmd/baselines/initial

# Commit the baseline
git add .tokmd/baselines/initial
git commit -m "chore: add tokmd baseline"
```

**Complexity baseline**:
```bash
# Capture a complexity baseline at the current commit
tokmd baseline --out .tokmd/baseline.json

# Commit for CI ratchet tracking
git add .tokmd/baseline.json
git commit -m "chore: add complexity baseline"
```

**Weekly baseline update (CI)**:
```yaml
name: Weekly Baseline
on:
  schedule:
    - cron: '0 0 * * 0'  # Every Sunday at midnight
  workflow_dispatch:  # Allow manual trigger

jobs:
  baseline:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install tokmd
        run: cargo install tokmd

      - name: Generate baseline
        run: |
          DATE=$(date +%Y%m%d)
          tokmd run --output-dir .tokmd/baselines/$DATE

      - name: Compare to previous
        run: |
          PREV=$(ls -1 .tokmd/baselines/ | sort | tail -2 | head -1)
          CURR=$(ls -1 .tokmd/baselines/ | sort | tail -1)

          echo "## Weekly Metrics Report" >> $GITHUB_STEP_SUMMARY
          echo "Comparing $PREV to $CURR" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          tokmd diff .tokmd/baselines/$PREV .tokmd/baselines/$CURR --format md >> $GITHUB_STEP_SUMMARY

      - name: Commit new baseline
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add .tokmd/baselines/
          git commit -m "chore: weekly baseline update"
          git push
```

**Release comparison**:
```bash
# Before release: compare to last release
tokmd diff .tokmd/baselines/v1.0.0 . --format md

# After release: save new baseline
tokmd run --output-dir .tokmd/baselines/v1.1.0
```

**Detecting codebase bloat**:
```bash
#!/bin/bash
# detect-bloat.sh - Run in CI to catch unexpected growth

BASELINE=".tokmd/baselines/initial"
THRESHOLD=10  # Alert if growth exceeds 10%

# Get baseline and current totals
BASELINE_LINES=$(jq '.total.code' "$BASELINE/lang.json")
CURRENT_LINES=$(tokmd --format json | jq '.total.code')

# Calculate growth percentage
GROWTH=$(echo "scale=2; (($CURRENT_LINES - $BASELINE_LINES) / $BASELINE_LINES) * 100" | bc)

echo "Baseline: $BASELINE_LINES lines"
echo "Current: $CURRENT_LINES lines"
echo "Growth: $GROWTH%"

if (( $(echo "$GROWTH > $THRESHOLD" | bc -l) )); then
  echo "::warning::Codebase has grown by $GROWTH% since baseline"
  exit 1
fi
```
