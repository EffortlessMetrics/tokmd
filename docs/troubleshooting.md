# Troubleshooting Guide

This guide covers common issues when using `tokmd` and how to resolve them.

## Files Not Appearing in Scans

### Symptom
A file exists in your repository but doesn't appear in `tokmd` output.

### Diagnosis

Use `check-ignore` to understand why:

```bash
tokmd check-ignore path/to/file.rs
```

**Exit codes**:
- `0` = File is ignored (shows why)
- `1` = File is not ignored

**Verbose mode** shows the exact rule that matched:
```bash
tokmd check-ignore -v path/to/file.rs
```

### Common Causes

**1. File is gitignored**

The file matches a pattern in `.gitignore`:
```bash
# Check if git ignores it
git check-ignore -v path/to/file.rs
```

**2. File is tracked but gitignored**

If a file was committed before being added to `.gitignore`, gitignore patterns don't apply:
```bash
# Untrack the file (keeps the local copy)
git rm --cached path/to/file.rs
```

**3. File matches .tokeignore pattern**

Check your `.tokeignore` file for patterns that might match.

**4. File excluded via --exclude flag**

If using `--exclude` patterns, ensure they don't match:
```bash
# Check what files are found without excludes
tokmd export --no-ignore
```

**5. File type not recognized by tokei**

Some file extensions aren't recognized as code. Check tokei's supported languages:
```bash
tokei --languages
```

---

## Exit Codes Reference

### Standard Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Invalid arguments / CLI parsing error |

### Command-Specific Exit Codes

**`check-ignore`**:
| Code | Meaning |
|------|---------|
| `0` | File IS ignored |
| `1` | File is NOT ignored |

**`diff`**:
| Code | Meaning |
|------|---------|
| `0` | Comparison successful, changes found or no changes |
| `1` | Error during comparison |

---

## Inconsistent Byte Counts

### Symptom
Byte counts differ between runs or between systems.

### Causes

**Line endings (CRLF vs LF)**:
Windows uses CRLF (`\r\n`) while Unix uses LF (`\n`). This affects byte counts.

**Solution**: Normalize line endings in your repository:
```bash
# Add to .gitattributes
* text=auto
```

**Encoding differences**:
Files with different encodings may report different sizes.

---

## Context Packing Issues

### Symptom
`tokmd context` selects unexpected files or doesn't fit the expected content.

### Diagnosis

**Check what's selected**:
```bash
# List mode shows what would be packed
tokmd context --budget 128k --output list
```

**Check token estimates**:
```bash
tokmd export --format csv | head -20
```

### Common Issues

**1. Token estimates differ from actual LLM counts**

`tokmd` uses a simple heuristic (~4 characters per token). Actual tokenization varies by model and content type.

**Workaround**: Use a smaller budget than your actual context window:
```bash
# For 128k context, use 100k budget
tokmd context --budget 100k --output bundle
```

**2. Wrong files selected with greedy strategy**

Greedy takes largest files first. For better coverage:
```bash
tokmd context --budget 128k --strategy spread
```

**3. Comments and blanks consuming budget**

Strip them for maximum density:
```bash
tokmd context --budget 128k --output bundle --compress
```

---

## Git Metrics Not Working

### Symptom
Git-related analysis (hotspots, freshness, coupling) shows empty or missing data.

### Diagnosis

**Check if git feature is enabled**:
```bash
# Force git metrics
tokmd analyze --preset risk --git
```

**Check git repository**:
```bash
git status  # Ensure you're in a git repo
git log --oneline -5  # Ensure there's history
```

### Common Causes

**1. Not in a git repository**

`tokmd` must be run from within a git repository for git metrics.

**2. Shallow clone**

CI systems often use shallow clones. Git metrics need history:
```bash
# In CI, fetch more history
git fetch --unshallow
# Or fetch specific depth
git fetch --depth=100
```

**3. No commits in analyzed paths**

If you're scanning a subdirectory with no commit history, git metrics will be empty.

**4. Git feature disabled at compile time**

If compiled without the `git` feature:
```bash
# Check if git support is available
tokmd analyze --preset risk --git 2>&1 | grep -i git
```

---

## Performance Issues on Large Repos

### Symptom
`tokmd` runs slowly or uses excessive memory on large repositories.

### Solutions

**1. Limit analysis scope**:
```bash
# Only analyze specific directories
tokmd -p src crates

# Limit file walking
tokmd analyze --preset supply --max-files 10000
```

**2. Limit git history scanning**:
```bash
tokmd analyze --preset risk --max-commits 500 --max-commit-files 50
```

**3. Limit content scanning**:
```bash
tokmd analyze --preset supply --max-bytes 100000000 --max-file-bytes 1000000
```

**4. Use lighter presets**:
```bash
# Instead of 'deep', use targeted presets
tokmd analyze --preset receipt  # Fastest
tokmd analyze --preset health   # Adds TODO scanning
```

**5. Exclude heavy directories**:
```bash
tokmd --exclude "vendor/" --exclude "node_modules/"
```

**6. Use .tokeignore**:
Create a `.tokeignore` file to exclude paths from all `tokmd` runs:
```gitignore
# .tokeignore
vendor/
node_modules/
*.lock
testdata/large/
```

---

## Memory Usage Optimization

### Symptom
`tokmd` uses excessive memory on very large codebases.

### Solutions

**1. Process in chunks**:
Instead of analyzing everything at once, process directories separately:
```bash
for dir in crates/*; do
  tokmd analyze -p "$dir" --preset receipt --format json > "$dir.json"
done
```

**2. Use export for large repos**:
The `export` command streams output and uses less memory:
```bash
tokmd export --format jsonl > inventory.jsonl
```

**3. Limit the number of files**:
```bash
tokmd export --max-rows 5000
```

---

## Configuration Not Loading

### Symptom
Settings in `tokmd.toml` aren't being applied.

### Diagnosis

**Check file location**:
`tokmd` looks for configuration in this order:
1. `./tokmd.toml` (current directory)
2. Parent directories (walking up to root)
3. `~/.config/tokmd/tokmd.toml` (user config)

**Verify TOML syntax**:
```bash
# Check for syntax errors
cat tokmd.toml | python -c "import sys, tomllib; tomllib.loads(sys.stdin.read())"
```

### Common Issues

**1. Wrong section names**

Use the correct section structure:
```toml
[scan]
paths = ["."]

[module]
roots = ["src"]

[analyze]
preset = "receipt"
```

**2. Profile not specified**

Named profiles require `--profile`:
```bash
tokmd --profile llm
```

**3. Environment variable override**

Check if `TOKMD_CONFIG` is set:
```bash
echo $TOKMD_CONFIG
```

---

## JSON Schema Validation Errors

### Symptom
External tools reject `tokmd` JSON output as invalid.

### Diagnosis

Check the schema version:
```bash
tokmd export --format jsonl | head -1 | jq '.schema_version'
```

### Solutions

**1. Update downstream tools**

Ensure tools expect the current schema version.

**2. Check schema documentation**

See `docs/SCHEMA.md` and `docs/schema.json` for the formal schema definition.

---

## Path Does Not Exist Error

### Symptom
`tokmd` fails with an error like "path does not exist: /path/to/file".

### Explanation

As of v1.3.0, `tokmd` now returns an error when input paths don't exist, rather than silently succeeding with empty output. This prevents silent failures in CI pipelines and scripts.

### Solutions

**1. Verify paths exist**:
```bash
ls -la path/to/scan
```

**2. Use glob patterns carefully**:
Shell expansion happens before `tokmd` sees the paths. If no files match, the shell may pass the literal pattern:
```bash
# May fail if no .rs files exist
tokmd -p "src/*.rs"

# Use quotes to let tokmd handle the pattern
tokmd -p src --exclude "*.txt"
```

**3. Handle missing paths in scripts**:
```bash
if [[ -d "$DIR" ]]; then
  tokmd -p "$DIR"
else
  echo "Directory $DIR not found"
  exit 1
fi
```

---

## Getting More Help

If you're still stuck:

1. **Run with verbose output**: Add `-v` or `--verbose` to commands
2. **Check the version**: `tokmd --version`
3. **Report issues**: https://github.com/EffortlessMetrics/tokmd/issues

Include in bug reports:
- `tokmd --version` output
- Operating system
- Minimal reproduction steps
- Actual vs expected behavior
