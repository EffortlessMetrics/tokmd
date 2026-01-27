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

## Step 4: Creating a Context File for AI

You want to ask an LLM about your code, but you can't paste 10,000 files. You need a compact summary of *what exists*.

Generate a "receipt" of the repo structure:

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

## Next Steps

- Check out the **[Recipes](recipes.md)** for more advanced workflows.
- Read the **[CLI Reference](reference-cli.md)** for all available flags.
