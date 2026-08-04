#!/usr/bin/env bash
set -euo pipefail

# `tokmd-types` and `tokmd-envelope` have development-only references to
# later workspace crates. Cargo resolves those references while verifying a
# normal publish, so bootstrap only these two packages without verification;
# every other package uses the normal verified cargo publish path.

if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
  echo "CARGO_REGISTRY_TOKEN is required" >&2
  exit 1
fi

release_tag="${RELEASE_TAG:-${GITHUB_REF_NAME:-}}"
if [[ ! "$release_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "a stable release tag is required via RELEASE_TAG or GITHUB_REF_NAME" >&2
  exit 1
fi
expected_version="${release_tag#v}"
actual_version="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json, sys; metadata = json.load(sys.stdin); print(next(package["version"] for package in metadata["packages"] if package["name"] == "tokmd"))')"
if [[ "$actual_version" != "$expected_version" ]]; then
  echo "release tag ${release_tag} does not match workspace version ${actual_version}" >&2
  exit 1
fi

expected_crates=(
  tokmd-gate
  tokmd-io-port
  tokmd-types
  tokmd-model
  tokmd-settings
  tokmd-scan
  tokmd-git
  tokmd-envelope
  tokmd-sensor
  tokmd-analysis-types
  tokmd-format
  tokmd-analysis
  tokmd-cockpit
  tokmd-core
  tokmd-wasm
  tokmd
)

echo "Running release publish preflight"
cargo xtask version-consistency
cargo xtask publish-surface --json

plan_output="$(cargo xtask publish --plan)"
mapfile -t planned_crates < <(
  printf '%s\n' "$plan_output" |
    sed -nE 's/^[[:space:]]+[0-9]+\. ([a-z0-9-]+)$/\1/p'
)
if [[ "${#planned_crates[@]}" -ne "${#expected_crates[@]}" ]]; then
  echo "publish plan contains ${#planned_crates[@]} crates; expected ${#expected_crates[@]}" >&2
  exit 1
fi
for crate in "${expected_crates[@]}"; do
  if ! printf '%s\n' "${planned_crates[@]}" | grep -Fxq "$crate"; then
    echo "publish plan is missing ${crate}; refusing to publish" >&2
    exit 1
  fi
done

publish_one() {
  local crate="$1"
  local no_verify="${2:-false}"
  local attempt output status
  local -a args=(publish --package "$crate" --locked)

  if [[ "$no_verify" == "true" ]]; then
    args+=(--no-verify)
  fi

  for attempt in 1 2 3 4 5; do
    echo "Publishing ${crate} (attempt ${attempt}/5; no_verify=${no_verify})"
    set +e
    output="$(cargo "${args[@]}" 2>&1)"
    status=$?
    set -e
    printf '%s\n' "$output"

    if [[ "$status" -eq 0 ]] || printf '%s\n' "$output" | grep -Eqi \
      'already uploaded|crate version.*already exists|already exists on crates.io index'; then
      echo "Published or already present: ${crate}"
      return 0
    fi

    if ! printf '%s\n' "$output" | grep -Eqi \
      'failed to select a version|no matching package|no matching version|failed to get|network|connection|timed out|timeout'; then
      echo "Publishing ${crate} failed with a non-retryable error" >&2
      return "$status"
    fi

    if [[ "$attempt" -lt 5 ]]; then
      echo "Transient crates.io failure for ${crate}; waiting 30s" >&2
      sleep 30
    fi
  done

  echo "Publishing ${crate} failed after five attempts" >&2
  return 1
}

publish_one tokmd-gate
publish_one tokmd-io-port
publish_one tokmd-types true
publish_one tokmd-model
publish_one tokmd-settings
publish_one tokmd-scan
publish_one tokmd-git
publish_one tokmd-envelope true
publish_one tokmd-sensor
publish_one tokmd-analysis-types
publish_one tokmd-format
publish_one tokmd-analysis
publish_one tokmd-cockpit
publish_one tokmd-core
publish_one tokmd-wasm
publish_one tokmd

echo "Verifying the published crate surface"
cargo xtask publish-surface --json --verify-publish

echo "All publishable tokmd crates are published or already present."
