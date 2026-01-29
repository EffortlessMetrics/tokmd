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
tokmd export --format csv --max-rows 10 --sort code
```

This prints the top 10 largest files. These are often candidates for refactoring or documentation.

## Step 4: Packing Code for an LLM

You want to paste actual code into an LLM, but your repo is too large. Use `context` to intelligently select files within a token budget:

```bash
# Pack the most valuable files into 128k tokens
tokmd context --budget 128k --output bundle > context.txt
```

**What happened?**
- `--budget 128k`: Set a token limit matching Claude's context window.
- `--output bundle`: Concatenated selected files into a single text file.
- Files are selected by size (largest = most valuable) until the budget is exhausted.

**Alternative strategies**:
```bash
# Spread coverage across all modules
tokmd context --budget 128k --strategy spread --output bundle

# Strip comments and blank lines for maximum density
tokmd context --budget 128k --output bundle --compress

# Use module roots for better organization
tokmd context --budget 128k --module-roots src,crates --strategy spread --output bundle
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
tokmd badge --metric lines --out badge.svg
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

## Next Steps

- Check out the **[Recipes](recipes.md)** for more advanced workflows.
- Read the **[CLI Reference](reference-cli.md)** for all available flags.
- See **[Schema](SCHEMA.md)** for output format details.
