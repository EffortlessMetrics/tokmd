# Tutorial: First Steps with tokmd

This guide will walk you through using `tokmd` to understand a codebase you've just cloned.

**Prerequisites**:
- `tokmd` installed (see below)
- A git repository to analyze (we'll assume you are in the root of one)

## Step 0: Installation

First, ensure the tool is installed.

### Nix (recommended)
```bash
nix profile install github:EffortlessMetrics/tokmd
```

### Cargo (alternative)
```bash
cargo install tokmd
```

Verify it works:

```bash
tokmd --version
```

---

## Step 0.5: Quick Setup with Interactive Wizard (Optional)

For first-time setup, run the interactive wizard to configure tokmd for your project:

```bash
tokmd init --interactive
```

The wizard will:
1. Detect your project type (Rust, Node, Python, Go, etc.)
2. Suggest appropriate module roots
3. Configure module depth and context budget
4. Optionally create both `.tokeignore` and `tokmd.toml`

If you prefer to dive in without configuration, skip to Step 1.

---

## Step 1: The "High Level" View

First, let's see what languages are in this project. This helps you verify your assumptions (e.g., "Is this mostly Rust, or is there a lot of Python glue code?").

Run:
```bash
tokmd
```

**What to look for**:
- Look at the `Code` column vs the `Files` column.
- Is there a language you didn't expect?
- Is there a massive amount of "JSON" or "YAML" implying heavy configuration?

## Step 2: Where is the code?

Knowing the languages is good, but *where* are they? Let's check the module structure.

Run:
```bash
tokmd module
```

This groups files by their top-level directory.

**Refining the view**:
If your repo puts everything in `src` or `packages`, the default view might be too coarse. Let's look deeper:

```bash
# Look 2 levels deep
tokmd module --module-depth 2
```

Now you can see `src/cli` vs `src/server`, or `packages/ui` vs `packages/backend`.

## Step 3: Finding "Heavy" Files

Often, 80% of the complexity lives in 20% of the files. Let's find the biggest files in the repo.

Run:
```bash
tokmd export --format csv --max-rows 10
```

This prints the top 10 largest files. These are often candidates for refactoring or documentation.

> **Note**: Output is automatically sorted by lines of code (descending), then by path. This ensures consistent, deterministic ordering across all runs.

## Step 4: Packing Code for an LLM

You want to paste actual code into an LLM, but your repo is too large. Use `context` to intelligently select files within a token budget:

```bash
# Pack the most valuable files into 128k tokens
tokmd context --budget 128k --output bundle --output context.txt
```

**What happened?**
- `--budget 128k`: Set a token limit matching Claude's context window.
- `--output bundle`: Concatenated selected files into a single text file.
- `--output context.txt`: Write output to a file instead of stdout.
- Files are selected by size (largest = most valuable) until the budget is exhausted.

**Alternative strategies**:
```bash
# Spread coverage across all modules
tokmd context --budget 128k --strategy spread --output bundle --output context.txt

# Strip blank lines for maximum density
tokmd context --budget 128k --output bundle --compress --output context.txt

# Use module roots for better organization
tokmd context --budget 128k --module-roots src,crates --strategy spread --output bundle --output context.txt
```

## Step 5: Creating a File Inventory for AI

For metadata about your codebase (not actual code), generate a "receipt":

```bash
tokmd export \
  --format jsonl \
  --min-code 10 \
  --redact paths \
  > repo_context.jsonl
```

**What happened?**
- `--format jsonl`: Created a streamable, machine-readable format.
- `--min-code 10`: Ignored empty/trivial files to save tokens.
- `--redact paths`: Hashed filenames so you don't leak internal project structure to a public LLM.

You can now upload `repo_context.jsonl` to an LLM and ask: *"Based on this file inventory, what is the architecture of this application?"*

---

## Step 6: Analyzing Code Quality

Now let's get deeper insights about the codebase structure and quality.

Run:
```bash
tokmd analyze --preset receipt --format md
```

**What you get**:
- **Totals**: Files, lines, bytes, and estimated tokens
- **Doc Density**: How much of the code is documented?
- **Test Density**: Ratio of test code to production code
- **Distribution**: File size statistics (median, p90, p99)
- **Top Offenders**: Largest files, least documented files

## Step 7: Checking Context Window Fit

Before feeding code to an LLM, check if it fits:

```bash
# Check against a 128k token window
tokmd analyze --preset receipt --window 128000 --format md
```

The output tells you:
- Total estimated tokens in your codebase
- What percentage of the context window it would use
- Whether it fits or needs filtering

## Step 8: Understanding Risk Areas

If the repo has git history, you can identify risky areas:

```bash
tokmd analyze --preset risk --format md
```

**What you get**:
- **Hotspots**: Files that change frequently AND are large (complexity risk)
- **Bus Factor**: Modules with few contributors (knowledge risk)
- **Freshness**: Stale files that may be outdated
- **Coupling**: Files that always change together

## Step 9: Generating a Badge

Add a lines-of-code badge to your README:

```bash
tokmd badge --metric lines --output badge.svg
```

Then add to your README:
```markdown
![Lines of Code](badge.svg)
```

---

## Step 10: Saving a Run

To track changes over time, save a complete analysis:

```bash
tokmd run --output-dir .runs/baseline
```

This creates:
- `lang.json` — Language summary
- `module.json` — Module breakdown
- `export.jsonl` — File inventory
- `analysis.json` — Derived metrics

Later, you can diff against this baseline:

```bash
tokmd diff .runs/baseline .
```

---

## Step 11: Troubleshooting Missing Files

Sometimes files don't appear in your scans when you expect them to. The `check-ignore` command helps diagnose why.

**Checking a single file**:
```bash
tokmd check-ignore path/to/missing/file.rs
```

**Understanding exit codes**:
- Exit code `0`: The file **is ignored** (output shows why)
- Exit code `1`: The file **is not ignored**

This makes it easy to use in scripts:
```bash
if tokmd check-ignore some/file.rs; then
  echo "File is ignored"
else
  echo "File should appear in scans"
fi
```

**Verbose mode for details**:
```bash
tokmd check-ignore -v node_modules/package/index.js
```

Verbose output shows:
- Which ignore file matched (`.gitignore`, `.tokeignore`)
- The specific pattern that caused the match
- Whether the file is tracked by git

**Common scenarios**:

1. **File in `.gitignore` but tracked by git**:
   - Gitignore patterns don't apply to tracked files
   - Solution: `git rm --cached <file>` to untrack it

2. **Unexpected pattern matching**:
   - Use `-v` to see which pattern matched
   - Check parent directories for ignore files

3. **File should be ignored but isn't**:
   - Ensure the pattern is correct in `.tokeignore` or `.gitignore`
   - Remember: patterns without `/` match anywhere in the path

See the [Troubleshooting Guide](troubleshooting.md) for more detailed scenarios.

---

## Step 12: Setting Up CI Quality Gates

Enforce code quality standards in your CI pipeline with policy-based gates:

```bash
# Run gate with rules from tokmd.toml
tokmd gate

# Or with an explicit policy file
tokmd gate --policy policy.toml
```

**Example inline rules in tokmd.toml**:
```toml
[[gate.rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "Codebase exceeds token budget"
```

**Exit codes**:
- `0`: All rules passed
- `1`: One or more rules failed (use this to fail CI)
- `2`: Policy error

See the [CLI Reference](reference-cli.md#tokmd-gate) for available operators and policy options.

**Preventing Regression with Ratchet Rules**:

You can also enforce that metrics improve (or don't get worse) over time using **Ratchet Rules**. This compares the current state against a baseline:

1. Generate a baseline: `tokmd baseline`
2. Define a ratchet rule in `tokmd.toml`:
   ```toml
   [[gate.ratchet]]
   pointer = "/complexity/avg_cyclomatic"
   max_increase_pct = 0.0
   description = "Average cyclomatic complexity"
   ```
3. Run with baseline: `tokmd gate --baseline .tokmd/baseline.json`

This ensures your code quality acts like a ratchet—it can go up, but never down.

---

## Step 13: Generating PR Metrics with `tokmd cockpit`

**Goal**: Get comprehensive metrics for a pull request to aid code review.

When preparing or reviewing a PR, you want to understand its scope, risk factors, and quality evidence. The `cockpit` command generates a complete metrics dashboard comparing two git references.

**Basic usage** (compare current branch against main):
```bash
tokmd cockpit
```

**Specify different base and head**:
```bash
tokmd cockpit --base develop --head feature/my-branch
```

**Output in Markdown for PR descriptions**:
```bash
tokmd cockpit --format md --output pr-metrics.md
```

**What you get**:

The cockpit receipt includes several sections:

- **Change Surface**: Files added, modified, deleted, and net line changes
- **Composition**: Breakdown of changes by language and type (production vs. test code)
- **Code Health**: Comment density, complexity indicators
- **Risk**: Hotspot analysis, coupling detection
- **Contracts**: API surface changes (if applicable)
- **Evidence**: Hard gates with pass/fail status

**Understanding Evidence Gates**:

The evidence section provides automated quality checks:

| Gate | Description |
|------|-------------|
| `mutation` | Mutation testing results (were tests effective?) |
| `diff_coverage` | Test coverage for changed lines |
| `contracts` | Contract/API compatibility verification |
| `supply_chain` | Dependency security checks |
| `determinism` | Build reproducibility verification |

Each gate has a status:
- `pass`: Gate passed
- `fail`: Gate failed (blocks merge if required)
- `skipped`: No relevant files changed
- `pending`: Results not yet available

**Example workflow for CI**:
```bash
# Generate cockpit metrics for the PR
tokmd cockpit --base $BASE_SHA --head $HEAD_SHA --format json --output cockpit.json

# Use in PR template
tokmd cockpit --format sections >> $GITHUB_STEP_SUMMARY
```

---

## Step 14: Exporting Tool Schemas with `tokmd tools`

**Goal**: Generate schema definitions of tokmd commands for LLM integration.

When building AI agents or automation that uses tokmd, you need schema definitions in a format your LLM understands. The `tools` command exports all tokmd commands as structured schemas.

**Generate OpenAI function calling format**:
```bash
tokmd tools --format openai --pretty
```

**Generate Anthropic tool use format**:
```bash
tokmd tools --format anthropic --pretty
```

**Generate standard JSON Schema**:
```bash
tokmd tools --format jsonschema --pretty
```

**Available formats**:

| Format | Description |
|--------|-------------|
| `jsonschema` | JSON Schema Draft 7 (default) |
| `openai` | OpenAI function calling format |
| `anthropic` | Anthropic tool use format |
| `clap` | Raw clap structure for debugging |

**What the output includes**:

Each tokmd command is represented with:
- **name**: Command name (e.g., `analyze`, `export`)
- **description**: What the command does
- **parameters**: Array of arguments with types, descriptions, and constraints

**Example: Integrating with an AI agent**:

```bash
# Export schema for your agent
tokmd tools --format anthropic --pretty > tokmd-tools.json

# The agent can now call tokmd commands with proper parameter validation
```

**Using in an LLM system prompt**:

```python
import json

# Load the schema
with open("tokmd-tools.json") as f:
    tools = json.load(f)

# Pass to your LLM API
response = client.messages.create(
    model="claude-sonnet-4-5-20250929",  # replace with a current Claude model ID
    tools=tools["tools"],
    messages=[{"role": "user", "content": "Analyze this codebase for me"}]
)
```

This enables your AI agent to intelligently invoke tokmd commands with validated parameters.

---

## Step 15: Creating a Complexity Baseline

**Goal**: Capture a snapshot of your codebase's complexity metrics at a known-good state, so you can track trends and prevent regressions.

```bash
tokmd baseline
```

**What happened?**
- tokmd scanned the codebase and computed complexity metrics (cyclomatic complexity, function length, Halstead metrics).
- It captured the current git commit SHA to anchor the baseline.
- A JSON baseline file was written with per-file and aggregate complexity data.

**Customizing the baseline**:
```bash
# Baseline a specific directory
tokmd baseline src

# Write to a specific output file
tokmd baseline --output .tokmd/baseline.json

# Overwrite if it exists
tokmd baseline --force
```

The baseline is used by the ratchet system to enforce that complexity does not regress across commits. See the [Recipes](recipes.md) for CI integration examples.

---

## Step 16: Bundling Code for LLM Handoff

**Goal**: Create a structured bundle of your codebase optimized for handing off to an AI assistant.

```bash
tokmd handoff
```

This creates a `.handoff/` directory with four artifacts:

| File | Purpose |
|------|---------|
| `manifest.json` | Bundle metadata, token budgets, capabilities |
| `map.jsonl` | Complete file inventory (streaming format) |
| `intelligence.json` | Tree, hotspots, complexity, and derived metrics |
| `code.txt` | Token-budgeted code bundle |

**Choosing an intelligence preset**:

| Preset | Includes |
|--------|----------|
| `minimal` | Tree + map only |
| `standard` | + complexity, derived metrics |
| `risk` | + hotspots, coupling (default) |
| `deep` | Everything |

```bash
# Minimal bundle for quick context
tokmd handoff --preset minimal

# Deep analysis for thorough review
tokmd handoff --preset deep

# Custom token budget
tokmd handoff --budget 200k

# Custom output directory
tokmd handoff --out-dir my-handoff/
```

**What to do with the output**: Feed the `.handoff/` directory contents to your LLM. The manifest tells the AI what's available, the map provides the full file inventory, the intelligence file gives structural insights, and the code bundle contains the actual source within your token budget.

---

## Next Steps

- Check out the **[Recipes](recipes.md)** for more advanced workflows.
- Read the **[CLI Reference](reference-cli.md)** for all available flags.
- See **[Schema](SCHEMA.md)** for output format details.
