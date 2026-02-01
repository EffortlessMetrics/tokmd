"""tokmd - Code inventory receipts and analytics.

This module provides Python bindings for tokmd, a fast code analysis tool
built on top of tokei. It generates "inventory receipts" and derived analytics
of code repositories.

Quick Start:
    >>> import tokmd
    >>> # Get language summary
    >>> result = tokmd.lang(paths=["src"])
    >>> for row in result["rows"]:
    ...     print(f"{row['lang']}: {row['code']} lines")
    >>>
    >>> # Get module breakdown
    >>> result = tokmd.module(paths=["."])
    >>> for row in result["rows"]:
    ...     print(f"{row['module']}: {row['code']} lines")
    >>>
    >>> # Run analysis
    >>> result = tokmd.analyze(paths=["."], preset="health")
    >>> if result.get("derived"):
    ...     print(f"Total: {result['derived']['totals']['code']} lines")

Features:
    - Language summary (lines of code, files, tokens by language)
    - Module breakdown (group by directory prefixes)
    - File-level export (CSV, JSONL, JSON, CycloneDX SBOM)
    - Analysis with multiple presets (health, risk, supply, etc.)
    - Diff comparison between receipts
    - Path redaction for safe LLM sharing

For more information, see https://github.com/EffortlessMetrics/tokmd
"""

from tokmd._tokmd import (
    TokmdError,
    __version__,
    SCHEMA_VERSION,
    version,
    schema_version,
    run_json,
    run,
    lang,
    module,
    export,
    analyze,
    diff,
)

__all__ = [
    "TokmdError",
    "__version__",
    "SCHEMA_VERSION",
    "version",
    "schema_version",
    "run_json",
    "run",
    "lang",
    "module",
    "export",
    "analyze",
    "diff",
]
