#!/usr/bin/env bash
# Post-edit hook: auto-format Rust files after Claude Code edits.
# Reads tool input JSON from stdin and formats the target file if it's .rs.
cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}"

INPUT=$(cat)

if command -v jq &>/dev/null; then
  FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')
  if [[ "$FILE_PATH" == *.rs ]]; then
    rustfmt "$FILE_PATH" 2>/dev/null
  else
    cargo fmt
  fi
else
  # Fallback: run cargo fmt if a .rs file was edited
  if echo "$INPUT" | grep -q '\.rs"'; then
    cargo fmt
  fi
fi
exit 0
