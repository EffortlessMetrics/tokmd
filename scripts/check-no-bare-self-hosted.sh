#!/usr/bin/env bash
set -euo pipefail

bad=0
workflow_dir="${1:-.github/workflows}"

echo "Checking for bare self-hosted runner usage in ${workflow_dir}..."

if [ ! -d "${workflow_dir}" ]; then
  echo "Workflow directory not found: ${workflow_dir}" >&2
  exit 1
fi

if rg -n --no-heading 'runs-on:[[:space:]]*self-hosted[[:space:]]*(#.*)?$' "${workflow_dir}"; then
  echo "Bare scalar self-hosted runs-on is forbidden; use an explicit runner group/labels or a hosted runner." >&2
  bad=1
fi

if rg -n --no-heading 'runs-on:[[:space:]]*\[[^]]*self-hosted[^]]*linux[^]]*x64[^]]*\]' "${workflow_dir}"; then
  echo "Bare inline self-hosted/linux/x64 runs-on is forbidden." >&2
  bad=1
fi

if rg -n --no-heading 'repos/[^[:space:]"'"'"']+/[^[:space:]"'"'"']+/actions/runners|repos/\$\{[^}]+\}/\$\{[^}]+\}/actions/runners' "${workflow_dir}"; then
  echo "Repository-scoped runner discovery is forbidden; use org runner discovery." >&2
  bad=1
fi

while IFS=: read -r file line _; do
  window="$(sed -n "${line},$((line+16))p" "$file")"

  if printf '%s\n' "$window" | rg -q '^[[:space:]]*-[[:space:]]*linux[[:space:]]*$' &&
     printf '%s\n' "$window" | rg -q '^[[:space:]]*-[[:space:]]*x64[[:space:]]*$' &&
     ! printf '%s\n' "$window" | rg -q 'group:[[:space:]]*em-ci-' &&
     ! printf '%s\n' "$window" | rg -q '^[[:space:]]*-[[:space:]]*(em-ci|ci-nano|policy-nano|workflow-nano|rust-tiny|rust-medium|rust-large|rust-16gb|cx23|cx33|cx43|cx53|cpx42)[[:space:]]*$'; then
    echo "$file:$line: bare self-hosted block lacks group/capacity labels" >&2
    bad=1
  fi
done < <(rg -n --no-heading '^[[:space:]]*-[[:space:]]*self-hosted[[:space:]]*$' "${workflow_dir}" || true)

exit "$bad"
