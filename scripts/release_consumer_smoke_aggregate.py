"""Build the fail-closed receipt for release consumer smoke jobs."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any

REQUIRED_SURFACES = (
    "release-assets",
    "binary-linux-amd64",
    "binary-linux-arm64",
    "binary-macos-amd64",
    "binary-macos-arm64",
    "binary-windows-amd64",
    "nix",
    "wasm",
    "action-binary",
    "action-container",
)

JOB_SURFACES = {
    "release-assets": ("release-assets",),
    "binary-smoke": (
        "binary-linux-amd64",
        "binary-linux-arm64",
        "binary-macos-amd64",
        "binary-macos-arm64",
        "binary-windows-amd64",
    ),
    "nix-smoke": ("nix",),
    "wasm-smoke": ("wasm",),
    "action-binary-smoke": ("action-binary",),
    "action-container-smoke": ("action-container",),
}


def aggregate_entries(
    receipts: list[dict[str, Any]],
    job_results: dict[str, dict[str, str]],
    release_kind: str,
    allowed_not_supported: set[str] | None = None,
) -> dict[str, Any]:
    entries: dict[str, dict[str, Any]] = {}
    for index, entry in enumerate(receipts):
        kind = entry.get("kind")
        key = entry.get("artifact") or kind
        if kind == "binary":
            key = entry.get("surface")
        if not key:
            key = f"invalid-receipt-{index}"
            entry = {
                "schema": "tokmd.release_consumer_smoke.v1",
                "kind": "invalid-receipt",
                "status": "failed",
                "reason": "receipt has no canonical surface key",
            }
        entries[key] = entry

    for job, surfaces in JOB_SURFACES.items():
        if job_results.get(job, {}).get("result") == "success":
            continue
        for surface in surfaces:
            if surface not in entries:
                entries[surface] = {
                    "schema": "tokmd.release_consumer_smoke.v1",
                    "kind": surface,
                    "status": "failed",
                    "reason": f"{job} did not complete successfully or produce a receipt",
                }

    for surface in REQUIRED_SURFACES:
        if surface not in entries:
            entries[surface] = {
                "schema": "tokmd.release_consumer_smoke.v1",
                "kind": surface,
                "status": "failed",
                "reason": "required surface did not produce a receipt",
            }

    permitted = allowed_not_supported or set()
    statuses = []
    for key, entry in entries.items():
        status = entry.get("status", "failed")
        if status == "not_supported" and release_kind == "rc" and key in permitted:
            continue
        statuses.append(status)

    overall = "passed" if statuses and all(status == "passed" for status in statuses) else "failed"
    return {"overall": overall, "entries": entries}


def main() -> None:
    receipts: list[dict[str, Any]] = []
    for path in Path("receipts").glob("*.json"):
        try:
            receipts.append(json.loads(path.read_text(encoding="utf-8")))
        except (OSError, ValueError) as error:
            receipts.append(
                {
                    "kind": "invalid-receipt",
                    "status": "failed",
                    "reason": f"could not parse {path.name}: {error}",
                }
            )

    result = aggregate_entries(
        receipts,
        json.loads(os.environ.get("NEEDS_RESULTS", "{}")),
        os.environ.get("RELEASE_KIND", "stable"),
        set(filter(None, os.environ.get("ALLOWED_NOT_SUPPORTED", "").split(","))),
    )
    entries = result["entries"]
    receipt = {
        "schema": "tokmd.release_consumer_smoke.v1",
        "repository": os.environ["RELEASE_REPOSITORY"],
        "tag": os.environ["TAG"],
        "expected_version": os.environ["EXPECTED_VERSION"],
        "release_kind": os.environ["RELEASE_KIND"],
        "overall": result["overall"],
        "entries": [entries[key] for key in sorted(entries)],
    }
    Path("tokmd.release_consumer_smoke.v1.json").write_text(
        json.dumps(receipt, indent=2) + "\n", encoding="utf-8"
    )
    with Path("release-consumer-smoke.md").open("w", encoding="utf-8") as output:
        output.write(f"# Release consumer smoke: {os.environ['TAG']}\n\n")
        output.write(f"Overall: `{result['overall']}`\n\n")
        output.write("| Surface | Status |\n| --- | --- |\n")
        for key in sorted(entries):
            output.write(f"| `{key}` | `{entries[key].get('status', 'failed')}` |\n")


if __name__ == "__main__":
    main()
