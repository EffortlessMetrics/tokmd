#!/usr/bin/env python3
"""Emit non-blocking GitHub annotations from RIPR line-placeable comments."""

import json
import sys
from pathlib import Path


def escape_data(value):
    """Escape a GitHub workflow command data field."""
    return (
        str(value)
        .replace("%", "%25")
        .replace("\r", "%0D")
        .replace("\n", "%0A")
        .replace(":", "%3A")
    )


def escape_property(value):
    """Escape a GitHub workflow command property value."""
    return escape_data(value).replace("=", "%3D").replace(",", "%2C")


path = Path("target/ripr/review/comments.json")
if not path.exists():
    print("::warning::No RIPR review comments JSON found; skipping annotations.", file=sys.stderr)
    raise SystemExit(0)

try:
    data = json.loads(path.read_text(encoding="utf-8"))
except json.JSONDecodeError as error:
    print(
        f"::warning::{escape_data(f'Invalid RIPR review comments JSON in {path}: {error}')}",
        file=sys.stderr,
    )
    raise SystemExit(0)

for item in data.get("comments", []):
    file = item.get("path") or item.get("file")
    line = item.get("line")
    title = item.get("title") or "RIPR"
    body = item.get("body") or item.get("message") or ""

    if not file or not line:
        continue

    print(
        "::warning "
        f"file={escape_property(file)},"
        f"line={escape_property(line)},"
        f"title={escape_property(title)}::"
        f"{escape_data(body)}"
    )
